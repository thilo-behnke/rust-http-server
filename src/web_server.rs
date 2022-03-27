pub mod web_server {
    use std::collections::HashMap;
    use crate::endpoint::endpoint::{EndpointHandler, EndpointProvider, EndpointType};
    use crate::file::file::read_file;
    use crate::parser::parser::parse;
    use crate::path::path::remap;
    use crate::resource::resource::{
        ResourceHandler, ResourceParameter, ResourceParameterLocation,
    };
    use crate::threads::threads::ThreadHandler;
    use crate::types::types::{HttpMethod, HttpRequest};
    use std::io::Read;
    use std::net::{TcpListener, TcpStream};
    use std::path::Path;
    use crate::response::response::ResponseHandler;
    use crate::template_engine::template_engine::TemplateEngine;

    const MESSAGE_SIZE: usize = 1024;

    pub struct WebServer {
        tcp_listener: TcpListener,
        thread_handler: ThreadHandler,
        endpoint_handler: EndpointHandler,
        template_engine: TemplateEngine
    }

    impl WebServer {
        pub fn create() -> WebServer {
            println!("Starting tcp bind to 8080.");
            let tcp_listener =
                TcpListener::bind("127.0.0.1:8080").expect("Unable to bind to port.");
            println!("Tcp bind established, now listening.");
            let thread_handler = ThreadHandler::create();
            let endpoint_handler = EndpointHandler::create();
            let template_engine = TemplateEngine {};
            return WebServer {
                tcp_listener,
                thread_handler,
                endpoint_handler,
                template_engine
            };
        }

        pub fn run(&mut self) -> std::io::Result<()> {
            self.endpoint_handler
                .register_static(String::from("files/dummy-website"), String::from("website"));
            self.endpoint_handler
                .register_assets(String::from("files/storage/"), String::from("storage"));
            self.endpoint_handler.register_resource(
                String::from("math/sqr"),
                String::from("sqr"),
                Box::new(ResourceHandler::new(
                    Box::from({|| {
                        let template = "<div>${sqr}</div>\r\n";
                        let res = (4 * 4).to_string();
                        let context: HashMap<String, String> = HashMap::from([("sqr".to_string(), res)]);
                        self.template_engine.render(template, context)
                    }}),
                    vec![ResourceParameter::p_i8(
                        String::from("n"),
                        ResourceParameterLocation::Query,
                    )],
                )),
            );

            for stream in self.tcp_listener.incoming() {
                match stream {
                    Ok(_stream) => {
                        println!(
                            "Successfully created tcp connection with client {:?}",
                            _stream.peer_addr()
                        );
                        // TODO: How to pass closure to other thread?
                        // https://users.rust-lang.org/t/how-to-send-function-closure-to-another-thread/43549/2
                        let endpoint_provider = Box::new(self.endpoint_handler.to_provider());
                        match self.thread_handler.spawn(|| {
                            let web_server_thread_handler = WebServerThreadHandler {
                                endpoint_handler: endpoint_provider,
                            };
                            web_server_thread_handler.handle_client(_stream)
                        }) {
                            Ok(()) => (),
                            Err(e) => println!("{}", e),
                        };
                    }
                    Err(e) => {
                        println!("Failed to establish tcp connection with client: {:?}", e);
                        break;
                    }
                }
            }
            Ok(())
        }
    }

    struct WebServerThreadHandler {
        endpoint_handler: Box<EndpointProvider>,
    }

    impl WebServerThreadHandler {
        fn handle_client(&self, mut stream: TcpStream) -> std::io::Result<()> {
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
                            println!(
                                "Received terminated message, try processing as http request..."
                            );
                            self.process_http_request(message, &stream);
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

        fn process_http_request(&self, message: &str, out_stream: &TcpStream) {
            let request = parse(message);
            match request {
                Ok(req) => {
                    println!("Received http request: {:?}", req);
                    let compress = match req.headers.iter().find(|(name, value)| **name == String::from("accept-encoding")) {
                        Some((_, val)) => val.split(",").map(|it| it.trim_start().trim_end()).collect::<Vec<&str>>().contains(&"gzip"),
                        None => false
                    };
                    let response_handler = match compress {
                        true => ResponseHandler::gzip(),
                        false => ResponseHandler::uncompressed()
                    };
                    match (req.general.method, req.general.path) {
                        (HttpMethod::Get, _) => {
                            self.process_get_request(out_stream, response_handler, &req);
                        }
                        _ => response_handler.not_found(out_stream).map_or_else(|e| println!("{}", e), |val| val),
                    }
                }
                Err(e) => {
                    println!("{}", e);
                    let response_handler = ResponseHandler::uncompressed();
                    response_handler.bad_request(out_stream).map_or_else(|e| println!("{}", e), |val| val)
                }
            }
        }

        fn process_get_request(&self, out_stream: &TcpStream, response_handler: Box<ResponseHandler>, request: &HttpRequest) {
            let path = &request.general.path;
            let corrected_path = match path.len() > 1 && path.ends_with("/") {
                true => &path[..path.len() - 1],
                false => &path,
            };
            println!("Received GET request to path {}", corrected_path);
            match self.get_file_content(corrected_path, request) {
                Ok(content) => {
                    response_handler.ok(out_stream, content.as_str()).map_or_else(|e| println!("{}", e), |val| val);
                }
                Err(_) => {
                    println!("--> not found");
                    response_handler.not_found(out_stream).map_or_else(|e| println!("{}", e), |val| val)
                }
            };
        }

        fn get_file_content(&self, path: &str, request: &HttpRequest) -> Result<String, String> {
            let endpoint = self
                .endpoint_handler
                .match_endpoint(String::from(path), HttpMethod::Get);
            match endpoint {
                Some(e) => {
                    let endpoint_type = &e.endpoint_type;
                    match endpoint_type {
                        EndpointType::StaticAsset(static_endpoint) => {
                            let asset_path = &static_endpoint.asset_path;
                            return read_file(asset_path);
                        }
                        EndpointType::Assets(asset_endpoint) => {
                            let asset_path = remap(
                                Path::new(path),
                                Path::new(&e.path),
                                Path::new(&asset_endpoint.asset_base),
                            )
                            .into_os_string()
                            .into_string()
                            .unwrap();
                            return read_file(&asset_path);
                        }
                        EndpointType::Resource(resource_endpoint) => {
                            return Ok(self.endpoint_handler.execute(resource_endpoint, request));
                        }
                        _ => panic!("Unable to handle endpoint type: {:?}", e.endpoint_type),
                    }
                }
                None => {
                    let mut error = String::from("Unable to get file content for: ");
                    error.push_str(path);
                    Err(error)
                }
            }
        }
    }
}
