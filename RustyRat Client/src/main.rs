use std::net::TcpStream;

pub mod libs;
use crate::libs::com_lib;
use crate::libs::debug_info::writeDebugInfo;

fn main() -> std::io::Result<()> {
    if let (stream) = TcpStream::connect("127.0.0.1:8080") {
        writeDebugInfo("Connected to the server");
        com_lib::start_communication(stream?);
    } else {
        writeDebugInfo("Failed to connect to the server");
    }



    Ok(())
}