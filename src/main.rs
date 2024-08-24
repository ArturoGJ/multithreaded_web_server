use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use multithreaded_web_server::ThreadPool;

// Notes:
// HTTP Request format:
// Method Request-URI HTTP-Version CRLF
// headers CRLF
// message-body

// HTTP Response format:
// HTTP-Version Status-Code Reason-Phrase CRLF
// headers CRLF
// message-body

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let full_http_request = get_full_http_request(&mut stream);
    let request_line = &full_http_request[0];

    // println!("Request: {full_http_request:#?}");

    let (status_line, filename) = match request_line.trim() {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            // In a single threaded web server this would block all other requests
            // from being served until this is done executing.
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 Not Found", "404.html"),
    };

    let contents = fs::read_to_string(format!("./resources/{filename}")).unwrap();
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}

fn get_full_http_request(stream: &mut TcpStream) -> Vec<String> {
    let mut buf_reader = BufReader::new(stream);
    let http_request: Vec<_> = buf_reader
        .by_ref()
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    http_request
}
