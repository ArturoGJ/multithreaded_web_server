use std::{
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread, usize,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for i in 0..size {
            workers.push(Worker::new(i, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        // Note: If the operating system can’t create a thread because there
        // aren’t enough system resources, thread::spawn will panic.
        // That will cause our whole server to panic, even though the creation
        // of some threads might succeed. For simplicity’s sake, this behavior
        // is fine, but in a production thread pool implementation, you’d likely
        // want to use std::thread::Builder and its spawn method that returns
        // Result instead.
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();

            println!("Worker {id} got a job: executing.");

            job();
        });
        Worker { id, thread }
    }
}
