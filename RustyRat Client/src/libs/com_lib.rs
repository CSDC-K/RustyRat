use std::net::TcpStream;
use std::io::{Read, Write, BufReader, BufRead};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
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

// Flag to pause reading during file transfer
static RECEIVING_FILE: AtomicBool = AtomicBool::new(false);

pub fn start_communication(stream : TcpStream) {
    // Use Mutex to protect stream access
    let stream = Arc::new(Mutex::new(stream));
    let write_stream = Arc::clone(&stream);
    let read_stream = Arc::clone(&stream);
    
    writeDebugInfo("Communication started");
    
    // Reading Handle Thread
    thread::spawn(move || {
        // Get a clone of the stream for reading
        let read_clone = {
            let guard = read_stream.lock().unwrap();
            guard.try_clone().expect("Failed to clone stream for reading")
        };
        
        // Set read timeout so we can periodically check the flag
        let _ = read_clone.set_read_timeout(Some(std::time::Duration::from_millis(100)));
        
        // Use a persistent BufReader to not lose buffered data
        let mut reader = BufReader::new(read_clone);
        
        loop {
            // Skip reading while file is being received
            if RECEIVING_FILE.load(Ordering::SeqCst) {
                thread::sleep(std::time::Duration::from_millis(50));
                continue;
            }
            
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break, // Connection closed
                Ok(_) => {
                    if !line.trim().is_empty() && !RECEIVING_FILE.load(Ordering::SeqCst) {
                        println!("Received: {}", line.trim());
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || 
                              e.kind() == std::io::ErrorKind::TimedOut => {
                    // Timeout, continue loop to check flag
                    continue;
                }
                Err(_) => continue,
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
            
            // Check if copyfile command - format: /copyfile <remote_path>
            if buffer.starts_with("/copyfile ") {
                let parts: Vec<&str> = buffer.splitn(2, ' ').collect();
                if parts.len() >= 2 {
                    let remote_path = parts[1];
                    let file_name = std::path::Path::new(remote_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("downloaded_file");
                    
                    // Pause reading thread
                    RECEIVING_FILE.store(true, Ordering::SeqCst);
                    thread::sleep(std::time::Duration::from_millis(200));
                    
                    // Lock stream for exclusive file transfer
                    let mut stream_guard = write_stream.lock().unwrap();
                    
                    // Send command
                    stream_guard.write_all(buffer.as_bytes()).expect("Error sending command");
                    stream_guard.flush().expect("Error flushing");
                    
                    println!("Starting file receive...");
                    match receive_file(&mut stream_guard, file_name) {
                        Ok(_) => println!("File copied successfully!"),
                        Err(e) => eprintln!("Error copying file: {}", e),
                    }
                    
                    drop(stream_guard);
                    
                    // Resume reading thread
                    RECEIVING_FILE.store(false, Ordering::SeqCst);
                } else {
                    eprintln!("Usage: /copyfile <remote_path>");
                }
            } else {
                // Normal message - just send it
                let mut stream_guard = write_stream.lock().unwrap();
                stream_guard.write_all(buffer.as_bytes()).expect("Error at writing");
                stream_guard.flush().expect("Error flushing");
            }
        }
    });

    loop {
        thread::park();
    }
}