pub mod response {
    use std::io::Write;
    use std::net::TcpStream;

    pub fn ok(mut out_stream: &TcpStream, content: &str) -> Result<(), String> {
        let res = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            content.len(),
            content
        );
        return write(res.as_str(), out_stream);
    }

    pub fn not_found(mut out_stream: &TcpStream) -> Result<(), String> {
        let res = format!("HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
        return write(res.as_str(), out_stream)
    }

    fn write(content: &str, mut out_stream: &TcpStream) -> Result<(), String> {
        return match out_stream.write(content.as_bytes()) {
            Err(_) => Err(String::from("Failed to write response")),
            Ok(_) => match out_stream.flush() {
                Err(e) => Err(String::from("Failed to write response")),
                Ok(()) => Ok(())
            }
        };
    }
}
