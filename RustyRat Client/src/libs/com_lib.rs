use std::net::{TcpStream, TcpListener};
use std::io::{Read, Write, BufReader, BufRead};
use std::sync::Arc;
use std::thread;
use std::io;
use std::fs::File;

use crate::libs::debug_info::writeDebugInfo;

// Receive file from server and save it locally
pub fn receive_file(stream: &mut TcpStream, save_path: &str) -> Result<(), String> {
    // Read file size (8 bytes, little-endian)
    let mut size_buf = [0u8; 8];
    stream.read_exact(&mut size_buf).map_err(|e| format!("Error reading file size: {}", e))?;
    let file_size = u64::from_le_bytes(size_buf);
    
    println!("Receiving file of size: {} bytes", file_size);
    
    // Create file
    let mut file = File::create(save_path).map_err(|e| format!("Error creating file: {}", e))?;
    
    // Read file data
    let mut received = 0u64;
    let mut buffer = [0u8; 512];
    
    while received < file_size {
        let to_read = std::cmp::min(512, (file_size - received) as usize);
        let bytes_read = stream.read(&mut buffer[..to_read]).map_err(|e| format!("Error reading: {}", e))?;
        if bytes_read == 0 {
            return Err("Connection closed".to_string());
        }
        file.write_all(&buffer[..bytes_read]).map_err(|e| format!("Error writing: {}", e))?;
        received += bytes_read as u64;
    }
    
    // Flush and sync to ensure all data is written to disk
    file.flush().map_err(|e| format!("Error flushing file: {}", e))?;
    file.sync_all().map_err(|e| format!("Error syncing file: {}", e))?;
    
    // Read and discard the trailing newline sent by server
    let mut trailing = [0u8; 1];
    let _ = stream.read(&mut trailing);
    
    println!("File saved to: {}", save_path);
    Ok(())
}

pub fn start_communication(stream : TcpStream) {
    // Placeholder for communication logic
    let stream = Arc::new(stream);
    let write_thread_handle = Arc::clone(&stream);
    let read_thread_handle = Arc::clone(&stream);
    writeDebugInfo("Communication started");
    
    // Reading Handle Thread
    thread::spawn(move || {
        let reader = BufReader::new(&*read_thread_handle);
        for line in reader.lines() {
            match line {
                Ok(msg) => println!("Received: {}", msg),
                Err(_) => break,
            }
        }
    });

    // Writing Handle Thread
    thread::spawn(move || {
        let mut buffer = String::new();
        loop {
            buffer.clear();
            print!("[COM_LIB] Enter message to send: ");
            io::stdout().flush().expect("Error at flushing stdout");
            io::stdin().read_line(&mut buffer).expect("Error at reading from stdin");
            buffer = buffer.trim().to_string();
            
            let mut thread_guard = write_thread_handle.try_clone().expect("Error at writing thread handle");
            thread_guard.write_all(buffer.as_bytes()).expect("Error at writing thread");
            
            // Check if copyfile command - format: /copyfile <remote_path>
            if buffer.starts_with("/copyfile ") {
                let parts: Vec<&str> = buffer.splitn(2, ' ').collect();
                if parts.len() >= 2 {
                    let remote_path = parts[1];
                    let file_name = std::path::Path::new(remote_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("downloaded_file");
                    println!("Starting file receive...");
                    match receive_file(&mut thread_guard, file_name) {
                        Ok(_) => println!("File copied successfully!"),
                        Err(e) => eprintln!("Error copying file: {}", e),
                    }
                } else {
                    eprintln!("Usage: /copyfile <remote_path>");
                }
            }
        }
    });

    loop {
        thread::park();
    }


}