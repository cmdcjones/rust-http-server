use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use ::my_http_server::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let mut pool = match ThreadPool::build(5) {
        Ok(pool) => pool,
        Err(e) => panic!("{}", e),
    };

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("sending job");
        pool.execute(|| {
            handle_connection(stream);
        })
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    let request_path: &str = http_request[0].split(" ").collect::<Vec<_>>()[1];
    let (status_line, filename) = match request_path {
        "/" => ("HTTP1.1 200 OK", "index.html"),
        "/sleep" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP1.1 200 OK", "index.html")
        }
        _ => ("HTTP1.1 404 Not Found", "404.html"),
    };
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}
