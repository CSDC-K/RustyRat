use std::net::{TcpListener, TcpStream};
use std::thread;

pub mod libs;

use crate::libs::handle_client_lib;
use crate::libs::debug_info::writeDebugInfo;

fn main() -> std::io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    writeDebugInfo("Server started, waiting for clients...");
    for stream in listener.incoming() {
        handle_client_lib::handle_client(stream?);
    }

    Ok(())

}
