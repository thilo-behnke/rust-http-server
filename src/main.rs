use std::collections::HashMap;
use std::io::Read;
use std::net::{TcpListener, TcpStream};

use crate::endpoint::endpoint::{EndpointHandler, EndpointProvider};
use file::file::read_file;

use crate::parser::parser::parse;
use crate::response::response::{bad_request, not_found, ok};
use crate::threads::threads::ThreadHandler;
use crate::types::types::HttpMethod;
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
