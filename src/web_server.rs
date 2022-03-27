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
    use crate::request_helper::request_helper::RequestParameter;
    use crate::response::response::ResponseHandler;
    use crate::template_engine::template_engine::TemplateEngine;

    const MESSAGE_SIZE: usize = 1024;

    pub struct WebServer<'a> {
        tcp_listener: TcpListener,
        thread_handler: ThreadHandler,
        endpoint_handler: EndpointHandler<'a>,
        template_engine: TemplateEngine
    }

    impl WebServer<'static> {
        pub fn create() -> WebServer<'static> {
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

        pub fn run(&'static mut self) -> std::io::Result<()> {
            self.endpoint_handler
                .register_static(String::from("files/dummy-website"), String::from("website"));
            self.endpoint_handler
                .register_assets(String::from("files/storage/"), String::from("storage"));
            let template_engine = self.template_engine.clone();
            let handler: Box<dyn FnOnce(&Vec<RequestParameter>) -> String + Sync + Send> = Box::from({move |args: &Vec<RequestParameter>| {
                let template = "<div>${sqr}</div>\r\n";
                let res = (4 * 4).to_string();
                let context: HashMap<String, String> = HashMap::from([("sqr".to_string(), res)]);
                // template_engine.render(template, context)
                return "test".to_string()
            }});
            self.endpoint_handler.register_resource(
                String::from("math/sqr"),
                String::from("sqr"),
                &Box::new(
                    ResourceHandler::new(&handler,
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

    struct WebServerThreadHandler<'a> {
        endpoint_handler: Box<&'a EndpointProvider<'a>>,
    }

    impl WebServerThreadHandler<'_> {
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
                    let compress = match req.headers.iter().find(|(name, _)| **name == String::from("accept-encoding")) {
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
                    return match endpoint_type {
                        EndpointType::StaticAsset(static_endpoint) => {
                            let asset_path = &static_endpoint.asset_path;
                            read_file(asset_path)
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
                            read_file(&asset_path)
                        }
                        EndpointType::Resource(resource_endpoint) => {
                            Ok(self.endpoint_handler.execute(resource_endpoint, request))
                        }
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
