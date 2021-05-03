use rust_web_server::*;
use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

static GET_REQUEST: &str = "GET / HTTP/1.1\r\n";

fn send_response(
    first_line: &str,
    file: &str,
    stream: &mut TcpStream,
) -> std::result::Result<(), std::io::Error> {
    fs::read_to_string(file).and_then(|body: String| {
        let response = format!("{}{}", first_line, body);
        stream
            .write(response.as_bytes())
            .and_then(|_sent_bytes| stream.flush())
    })
}

fn connect(mut stream: TcpStream, pool: &mut ThreadPool) -> () {
    let mut buf = Box::new([0; 512]);
    let read = stream.read(&mut *buf);
    match read {
        Ok(_size) => {
            let data = String::from_utf8_lossy(&buf[..]);
            let v: Box<dyn FnMut() -> () + Send + Sync> = if data.starts_with(GET_REQUEST) {
                Box::new(move || {
                    let _ = send_response("HTTP/1.1 200 OK\r\n\r\n", "page.html", &mut stream);
                })
            } else {
                Box::new(move || {
                    let _ =
                        send_response("HTTP/1.1 404 NOT FOUND\r\n\r\n", "error.html", &mut stream);
                })
            };
            pool.execute(v);
        }
        Err(e) => panic!("{}", e),
    }
}

fn start(tcp_listener: &TcpListener, pool: &mut ThreadPool) -> ! {
    for stream in tcp_listener.incoming() {
        match stream {
            Ok(stream) => {
                let _ = connect(stream, pool);
                ()
            }
            Err(e) => panic!("{}", e),
        }
    }
    panic!("Will never hit");
}

fn main() {
    let mut tp = ThreadPool::make(3);
    match TcpListener::bind("127.0.0.1:9001") {
        Ok(listener) => start(&listener, &mut tp),
        Err(e) => panic!("Error: {}", e),
    }
}
