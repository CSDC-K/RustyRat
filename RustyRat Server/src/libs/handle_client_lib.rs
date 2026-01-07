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
            if let Some(rest) = command.strip_prefix("/copy_file ") {
                let file_bytes = command_lib::_DIRECTFUNC_COPY_FILE(rest);
                func_write(write_handle.clone(), String::new(), true, Some(file_bytes));
            } else {
                let response = command_lib::hook_to_command(&command);
                func_write(write_handle.clone(), "".to_string(), false, None);
                func_write(write_handle.clone(), response, false, None);
            }

            

            
        }
    });

    // func_write(write_handle.clone(), "Example To Send Msg".to_string());
    loop {
        
    }

}


fn func_write(write_clone: Arc<Mutex<TcpStream>>, content_of_msg: String, byte_send: bool, file_bytes: Option<Vec<u8>>) { 
    let mut stream = write_clone.lock().unwrap();

    if byte_send {
        // Send raw file bytes
        if let Some(bytes) = file_bytes {
            // First send the file size so client knows how many bytes to expect
            let size = bytes.len() as u64;
            match stream.write_all(&size.to_be_bytes()) {
                Ok(_) => {
                    if DEBUG_MODE { println!("Sent file size: {} bytes", size); }
                }
                Err(e) => {
                    if DEBUG_MODE { eprintln!("Error sending file size: {}", e); }
                    return;
                }
            }

            // Then send the actual file bytes
            match stream.write_all(&bytes) {
                Ok(_) => {
                    if let Err(e) = stream.flush() {
                        if DEBUG_MODE { eprintln!("Error flushing stream: {}", e); }
                    } else {
                        if DEBUG_MODE { println!("File bytes sent: {} bytes", bytes.len()); }
                    }
                }
                Err(e) => {
                    if DEBUG_MODE { eprintln!("Error writing file bytes to client: {}", e); }
                }
            }
        }
    } else {
        // Send text message
        let msg = if content_of_msg.ends_with('\n') {
            content_of_msg
        } else {
            format!("{}\n", content_of_msg)
        };
        
        match stream.write_all(msg.as_bytes()) {
            Ok(_) => {
                if let Err(e) = stream.flush() {
                    if DEBUG_MODE { eprintln!("Error flushing stream: {}", e); }
                } else {
                    if DEBUG_MODE { println!("Message sent: {}", msg.trim()); }
                }
            }
            Err(e) => {
                if DEBUG_MODE { eprintln!("Error writing to client: {}", e); }
            }
        }
    }
}