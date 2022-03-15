use std::io::{Read};
use std::net::{TcpListener, TcpStream};

const MESSAGE_SIZE: usize = 5;

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut received: Vec<u8> = vec![];
    let mut buf = [0u8; MESSAGE_SIZE];
    loop {
        match stream.read(&mut buf) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    println!("Tcp stream exhausted.");
                    break;
                }
                received.extend_from_slice(&buf[..bytes_read]);
            }
            Err(e) => {
                println!("Connection terminated: {:?}", e);
                break
            }
        };
    }
    println!("Received message:");
    let message = std::str::from_utf8(&received).expect("valid ut8");
    println!("{}", message);
    Ok(())
}

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
