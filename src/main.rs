use std::io::Read;
use std::net::{TcpListener, TcpStream};

use file::file::read_file;

use crate::parser::parser::parse;
use crate::response::response::{bad_request, not_found, ok};
use crate::threads::threads::ThreadHandler;
use crate::types::types::HttpMethod;

mod parser;
mod types;
mod file;
mod response;
mod threads;

const MESSAGE_SIZE: usize = 1024;

fn main() -> std::io::Result<()> {
    println!("Starting tcp bind to 8080.");
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Unable to bind to port.");
    println!("Tcp bind established, now listening.");
    let mut thread_handler: &ThreadHandler = ThreadHandler::create();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!(
                    "Successfully created tcp connection with client {:?}",
                    _stream.peer_addr()
                );
                thread_handler.spawn(|| {
                    handle_client(_stream)
                });
            }
            Err(e) => {
                println!("Failed to establish tcp connection with client: {:?}", e);
                break;
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
                // println!("Read message: {}", message);
                let terminated = message.ends_with("\r\n\r\n");
                if terminated {
                    println!("Received terminated message, try processing as http request...");
                    process_http_request(message, &stream);
                    received = vec![];
                }
            }
            Err(e) => {
                println!("Connection terminated: {:?}", e);
                break;
            }
        };
    }
    Ok(())
}

fn process_http_request(message: &str, out_stream: &TcpStream) {
    let request = parse(message);
    match request {
        Ok(req) => match (req.general.method, req.general.path) {
            (HttpMethod::Get, path) => {
                process_get_request(out_stream, path);
            }
            _ => {
                not_found(out_stream).map_or_else(|e| println!("{}", e), |val| val)
            }
        },
        Err(e) => {
            println!("{}", e);
            bad_request(out_stream).map_or_else(|e| println!("{}", e), |val| val)
        },
    }
}

fn process_get_request(out_stream: &TcpStream, path: &str) {
    println!("Received GET request to path {}", path);
    match get_file_content(path) {
        Ok(content) => {
            ok(out_stream, content.as_str()).map_or_else(|e| println!("{}", e), |val| val);
        },
        Err(_) => {
            println!("--> not found");
            not_found(out_stream).map_or_else(|e| println!("{}", e), |val| val)
        }
    };
}

fn get_file_content(path: &str) -> Result<String, String> {
    let file_path = match path {
        "/" => "/index.html",
        p => p
    };
    return read_file(file_path);
}
