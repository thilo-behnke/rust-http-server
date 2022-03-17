use std::collections::HashMap;
use std::io::{Read};
use std::net::{TcpListener, TcpStream};

const MESSAGE_SIZE: usize = 5;

fn main() -> std::io::Result<()> {
    println!("Starting tcp bind to 8080.");
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Unable to bind to port.");
    println!("Tcp bind established, now listening.");

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("Successfully created tcp connection with client {:?}", _stream.peer_addr());
                handle_client(_stream)?;
            },
            Err(e) => {
                println!("Failed to establish tcp connection with client: {:?}", e);
                break
            }
        }
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut received: Vec<u8> = vec![];
    let mut buf = [0u8; MESSAGE_SIZE];
    let mut message;
    loop {
        match stream.read(&mut buf) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    println!("Tcp stream exhausted.");
                    break;
                }
                received.extend_from_slice(&buf[..bytes_read]);
                message = std::str::from_utf8(&received).expect("invalid ut8");
                let terminated = message.ends_with("\n\n");
                if terminated {
                    println!("Received terminated message: {}", message);
                    try_parse_http_request(message)
                }
            }
            Err(e) => {
                println!("Connection terminated: {:?}", e);
                break
            }
        };
    }
    Ok(())
}

// fn try_parse_http_request(request: String) -> std::io::Result<HttpRequest>  {
fn try_parse_http_request(request: &str) {
    let request_lines: Vec<&str> = request.split("\\n").collect();
    let method_line = request_lines.first();
    let header_lines = request_lines[1..];
    match method_line {
        Some(&val) => match val.split(" ").collect::<Vec<&str>>().first() {
            Some(&"GET") => {
                println!("Found GET request")
            },
            _ => println!("Failed to parse method of http request: {}", val)
        },
        None => println!("Unable to split request into lines")
    }
}

fn try_parse_get_request(request_lines: Vec<&str>) {

}

struct HttpRequest {
    method: HttpMethod,
    path: String,
    headers: HashMap<str, str>
}

enum HttpMethod {
    GET, POST
}
