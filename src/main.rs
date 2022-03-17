use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

const MESSAGE_SIZE: usize = 1024;

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
                println!("Read message: {}", message);
                let terminated = message.ends_with("\r\n\r\n");
                if terminated {
                    println!("Received terminated message: {}", message);
                    process_http_request(message, &stream);
                    received = vec![];
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

fn process_http_request(message: &str, mut out_stream: &TcpStream) {
    let request = try_parse_http_request(message);
    match request {
        Ok(req) => match (req.method, req.path.as_str()) {
            (HttpMethod::GET, "/") => {
                println!("Sending response to client");
                let content = "<div>test<div>";
                let res = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}", content.len(), content);
                out_stream.write(res.as_bytes()).unwrap();
                out_stream.flush().unwrap();
            },
            _ => println!("noop")
        },
        Err(e) => println!("noop")
    }
}

fn try_parse_http_request(request: &str) -> Result<HttpRequest, &str> {
    let request_lines: Vec<Vec<&str>> = request.split("\r\n").map(|line| line.split(' ').collect()).collect();
    let request = match request_lines.as_slice() {
        [] => Err("Empty request"),
        // [m] => Some(HttpRequest {method: m[0], path: m[1], headers: HashMap::new()}),
        [m, rest @ ..] => Ok(HttpRequest {method: http_method_from_string(m[0]).expect("Invalid http method"), path: String::from(m[1]), headers: HashMap::new()})
    };
    return match request {
        Ok(req) => {
            println!("HttpRequest [method={:?}, path={:?}]", req.method, req.path);
            Ok(req)
        },
        Err(e) => {
            println!("{}", e);
            Err(e)
        }
    };
}

fn try_parse_get_request(request_lines: Vec<&str>) {

}

fn http_method_from_string(method: &str) -> Result<HttpMethod, &str> {
    return match method {
        "GET" => Ok(HttpMethod::GET),
        "POST" => Ok(HttpMethod::POST),
        _ => Err("Invalid http method")
    }
}

struct HttpRequest {
    method: HttpMethod,
    path: String,
    headers: HashMap<String, String>
}

#[derive(Debug)]
enum HttpMethod {
    GET, POST
}
