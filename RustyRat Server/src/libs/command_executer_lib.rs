use std::process;



pub fn execute_command(command: &str) -> String {
    let output = if cfg!(target_os = "windows") {
        process::Command::new("cmd")
            .args(&["/C", command])
            .output()
    } else {
        process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
    };

    match output {
        Ok(output) => {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).to_string()
            } else {
                format!(
                    "Error executing command: {}",
                    String::from_utf8_lossy(&output.stderr)
                )
            }
        }
        Err(e) => format!("Failed to execute command: {}", e),
    }
}