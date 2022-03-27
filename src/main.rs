use crate::web_server::web_server::WebServer;

mod endpoint;
mod file;
mod parser;
mod path;
mod request_helper;
mod resource;
mod response;
mod threads;
mod types;
mod web_server;
mod template_engine;

fn main() -> std::io::Result<()> {
    let mut server = WebServer::create();
    server.run()
}
