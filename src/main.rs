use crate::web_server::web_server::WebServer;

mod endpoint;
mod file;
mod parser;
mod path;
mod response;
mod threads;
mod types;
mod web_server;

fn main() -> std::io::Result<()> {
    WebServer::create().run()
}
