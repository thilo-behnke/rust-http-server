pub mod response {
    use std::io::Write;
    use std::net::TcpStream;

    pub fn ok(mut out_stream: &TcpStream, content: &str) {
        let res = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            content.len(),
            content
        );
        out_stream.write(res.as_bytes()).unwrap();
        out_stream.flush().unwrap();
    }

    pub fn not_found(mut out_stream: &TcpStream) {
        let res = format!("HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
        out_stream.write(res.as_bytes()).unwrap();
        out_stream.flush().unwrap();
    }
}
