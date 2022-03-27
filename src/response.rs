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
            ResponseHandler::create(Box::from(DefaultResponseWriter::uncompressed()), Option::None)
        }

        pub fn gzip() -> Box<ResponseHandler> {
            ResponseHandler::create(Box::from(DefaultResponseWriter::gzip()), Option::Some(String::from("gzip")))
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
    }

    struct DefaultResponseWriter {
        header_encoder: Box<dyn ResponseEncoder>,
        content_encoder: Box<dyn ResponseEncoder>,
    }

    impl DefaultResponseWriter {
        pub fn uncompressed() -> DefaultResponseWriter {
            DefaultResponseWriter {
                header_encoder: Box::from(PlainResponseEncoder {}),
                content_encoder: Box::from(PlainResponseEncoder {})
            }
        }

        pub fn gzip() -> DefaultResponseWriter {
            DefaultResponseWriter {
                header_encoder: Box::from(PlainResponseEncoder {}),
                content_encoder: Box::from(GzipResponseEncoder {})
            }
        }

        fn write_bytes(&self, bytes: &[u8], mut out_stream: &TcpStream) -> Result<(), String> {
            match out_stream.write(bytes) {
                Err(_) => Err(String::from("Failed to write response")),
                Ok(_) => match out_stream.flush() {
                    Err(e) => {
                        Err(String::from("Failed to write response: ") + e.to_string().as_str())
                    }
                    Ok(()) => Ok(()),
                },
            }
        }
    }

    impl ResponseWriter for DefaultResponseWriter {
        fn write(&self, headers: &str, content: Option<String>, out_stream: &TcpStream) -> Result<(), String> {
            let encoded_header_res = self.header_encoder.encode(headers);
            if let Err(e) = encoded_header_res {
                return Err(format!("Failed to write response: {}", e))
            }
            let encoded_header = encoded_header_res.unwrap();
            let encoded_content_res = match content {
                Some(c) => self.content_encoder.encode(&c),
                None => {
                    Ok(vec![])
                }
            };
            if let Err(e) = encoded_content_res {
                return Err(format!("Failed to write response: {}", e))
            }
            let encoded_content = encoded_content_res.unwrap();

            let content_length_header = format!("Content-Length: {}\r\n\r\n", encoded_content.len());
            let content_length_header_bytes = content_length_header.as_bytes().to_vec();

            let res_bytes: Vec<u8> = [encoded_header, content_length_header_bytes, encoded_content].concat();
            self.write_bytes(&*res_bytes, out_stream)
        }
    }

    trait ResponseEncoder {
        fn encode(&self, val: & str) -> Result<Vec<u8>, String>;
    }

    struct PlainResponseEncoder {}
    impl ResponseEncoder for PlainResponseEncoder {
        fn encode(&self, val: & str) -> Result<Vec<u8>, String> {
            Ok(val.as_bytes().to_vec())
        }
    }

    struct GzipResponseEncoder {}
    impl ResponseEncoder for GzipResponseEncoder {
        fn encode(&self, val: & str) -> Result<Vec<u8>, String> {
            let mut compressed = GzEncoder::new(Vec::new(), Compression::default());
            if let Err(e) = compressed.write_all(val.as_bytes()) {
                return Err(format!("Failed to write to compressor: {}", e));
            }
            let compressed_bytes_res = compressed.finish();
            match compressed_bytes_res {
                Ok(compressed_bytes) => {
                    return Ok(compressed_bytes)
                }
                Err(e) => Err(format!("Failed to compress content: {}", e))
            }
        }
    }
}
