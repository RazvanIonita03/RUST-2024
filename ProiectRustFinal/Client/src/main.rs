use std::io::{prelude::*, stdin};
use std::net::TcpStream;
use std::process::Command;

fn main() {
    let mut connected = 0;

    match TcpStream::connect("127.0.0.1:7878") {
        Ok(mut stream) => {
            println!("Connected!");

            loop {
                let mut buffer = String::new();
                let mut response = [0;1024];
                let _ = stdin().read_line(&mut buffer).unwrap();
                if connected == 0 {
                    stream.write(&buffer.as_bytes()).unwrap();
                    match stream.read(&mut response) {
                        Ok(bytes_read) => {
                            let response = String::from_utf8_lossy(&response[..bytes_read]);
                            println!("Server response: {}", response);

                            // Update connected state if login is successful
                            if response.contains("Login successful") {
                                connected = 1;
                            }
                            if response.contains("Register successful") {
                                connected = 1;
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to read response: {}", e);
                            break;
                        }
                    }
                } 
                else {
                    let resultat = run_piped_command(&buffer);
                    let formatted_message = format!("COMMAND_OUTPUT:{}", resultat);
                    stream.write(formatted_message.as_bytes()).unwrap();
                }
            }
        }
        Err(_) => {
            //error
        }
    }
}
fn run_piped_command(command: &str) -> String {
    match Command::new("cmd") // Use cmd.exe on Windows
        .arg("/C") // Use /C to execute the command
        .arg(command)
        .output()
    {
        Ok(output) => {
            if !output.stdout.is_empty() {
                String::from_utf8_lossy(&output.stdout).to_string()
            } else if !output.stderr.is_empty() {
                format!("Command error: {}", String::from_utf8_lossy(&output.stderr))
            } else {
                "Command executed with no output.".to_string()
            }
        }
        Err(e) => format!("Failed to execute command: {}", e),
    }
}
