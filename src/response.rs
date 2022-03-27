pub mod response {
    use std::io::Write;
    use std::net::TcpStream;
    use std::ops::Deref;
    use std::ptr::write_bytes;
    use flate2::Compression;
    use flate2::write::{GzEncoder, ZlibEncoder};

    pub struct ResponseHandler {
        content_encoding: Option<String>,
        writer: Box<dyn ResponseWriter>
    }

    impl ResponseHandler {

        pub fn uncompressed() -> Box<ResponseHandler> {
            ResponseHandler::create(Box::from(DefaultResponseWriter{}), Option::None)
        }

        pub fn gzip() -> Box<ResponseHandler> {
            ResponseHandler::create(Box::from(GzipResponseWriter{ delegate: Box::from(DefaultResponseWriter {}) }), Option::Some(String::from("gzip")))
        }

        fn create(writer: Box<dyn ResponseWriter>, content_encoding: Option<String>) -> Box<ResponseHandler> {
            Box::from(ResponseHandler {
                content_encoding,
                writer
            })
        }

        pub fn ok(&self, out_stream: &TcpStream, content: &str) -> Result<(), String> {
            let additional_headers = match &self.content_encoding {
                Some(val) => format!("Content-Encoding: {}\r\n", val),
                None => String::from("")
            };
            let headers = format!(
                // "HTTP/1.1 200 OK\r\n{}ContentLength: {}\r\n\r\n",
                "HTTP/1.1 200 OK\r\n{}",
                additional_headers
            );
            return self.write(headers.as_str(), Option::Some(String::from(content)), out_stream);
        }

        pub fn not_found(&self, out_stream: &TcpStream) -> Result<(), String> {
            let res = format!("HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
            return self.write(res.as_str(), None, out_stream);
        }

        pub fn bad_request(&self, out_stream: &TcpStream) -> Result<(), String> {
            let res = format!("HTTP/1.1 404 Bad Request\r\nContent-Length: 0\r\n\r\n");
            return self.write(res.as_str(), None, out_stream);
        }

        pub fn write(&self, headers: &str, content: Option<String>, mut out_stream: &TcpStream) -> Result<(), String> {
            self.writer.write(headers, content, out_stream)
        }
    }


    trait ResponseWriter {
        fn write(&self, headers: &str, content: Option<String>, out_stream: &TcpStream) -> Result<(), String>;
        fn write_bytes(&self, content: &[u8], out_stream: &TcpStream) -> Result<(), String>;
    }

    struct DefaultResponseWriter {}
    impl ResponseWriter for DefaultResponseWriter {
        fn write(&self, headers: &str, content: Option<String>, mut out_stream: &TcpStream) -> Result<(), String> {
            self.write_bytes(headers.as_bytes(), out_stream);
            if let Some(c) = content {
                let content_bytes = c.as_bytes();
                let content_length_header = format!("Content-Length: {}\r\n\r\n", content_bytes.len());
                let content_length_header_bytes = content_length_header.as_bytes();
                self.write_bytes(content_length_header_bytes, out_stream);
                self.write_bytes(content_bytes, out_stream);
            }
            // TODO: Fix.
            return Ok(())
        }
        fn write_bytes(&self, content: &[u8], mut out_stream: &TcpStream) -> Result<(), String> {
            return match out_stream.write(content) {
                Err(_) => Err(String::from("Failed to write response")),
                Ok(_) => match out_stream.flush() {
                    Err(e) => {
                        Err(String::from("Failed to write response: ") + e.to_string().as_str())
                    }
                    Ok(()) => Ok(()),
                },
            };
        }
    }

    struct GzipResponseWriter {
        delegate: Box<dyn ResponseWriter>
    }
    impl ResponseWriter for GzipResponseWriter {
        fn write(&self, headers: &str, content: Option<String>, mut out_stream: &TcpStream) -> Result<(), String> {
            self.delegate.write_bytes(headers.as_bytes(), out_stream);
            if let Some(c) = content {
                let content_bytes = c.as_bytes();
                self.write_bytes(content_bytes, out_stream);
            }
            Ok(())
        }

        fn write_bytes(&self, content: &[u8], out_stream: &TcpStream) -> Result<(), String> {
            let mut compressed = GzEncoder::new(Vec::new(), Compression::default());
            compressed.write_all(content).unwrap();
            let compressed_bytes = compressed.finish().unwrap();

            let content_length_header = format!("Content-Length: {}\r\n\r\n", compressed_bytes.len());
            let content_length_header_bytes = content_length_header.as_bytes();

            self.delegate.write_bytes(content_length_header_bytes, out_stream);
            self.delegate.write_bytes(&*compressed_bytes, out_stream)
        }
    }
}
