use crate::web_server::web_server::WebServer;

mod endpoint;
mod file;
mod parser;
mod path;
mod resource;
mod response;
mod threads;
mod types;
mod web_server;
mod request_helper;

fn main() -> std::io::Result<()> {
    WebServer::create().run()
}
