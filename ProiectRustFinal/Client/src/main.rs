use std::io::{prelude::*, stdin};
use std::net::TcpStream;
use std::process::Command;

fn main() {
    let mut connected = 0;

    match TcpStream::connect("127.0.0.1:80") {
        Ok(mut stream) => {
            println!("Connected!");

            loop {
                let mut buffer = String::new();
                let mut response = [0; 1024];
                match stdin().read_line(&mut buffer) {
                    Ok(_) => {
                        if connected == 0 {
                            if let Err(e) = stream.write(&buffer.as_bytes()) {
                                eprintln!("Failed to write the buffer : {}", e);
                            };
                            match stream.read(&mut response) {
                                Ok(bytes_read) => {
                                    let response = String::from_utf8_lossy(&response[..bytes_read]);
                                    println!("{}", response);

                                    if response.contains("Login successful") {
                                        connected = 1;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to read response: {}", e);
                                    break;
                                }
                            }
                        } else {
                            match run_piped_command(&buffer) {
                                Ok(command) => {
                                    let formatted_message = format!("COMMAND_OUTPUT:{}", command);
                                    if let Err(e) = stream.write(formatted_message.as_bytes()) {
                                        eprintln!("Failed to write the buffer : {}", e);
                                    };
                                    match stream.read(&mut response) {
                                        Ok(bytes_read) => {
                                            let response =
                                                String::from_utf8_lossy(&response[..bytes_read]);
                                            println!("{}", response);
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to read response: {}", e);
                                            break;
                                        }
                                    };
                                }
                                Err(e) => {
                                    eprintln!("Failed to run command: {}", e);
                                    break;
                                }
                            };
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read from stdin: {}", e);
                        break;
                    }
                };
            }
        }
        Err(e) => {
            eprintln!("Failed to connect to 127.0.0.1:80 due to: {}", e);
        }
    }
}
fn run_piped_command(command: &str) -> Result<std::string::String, std::io::Error> {
    match Command::new("cmd").arg("/C").arg(command).output() {
        Ok(output) => {
            if !output.stdout.is_empty() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else if !output.stderr.is_empty() {
                Ok(format!(
                    "Command error: {}",
                    String::from_utf8_lossy(&output.stderr)
                ))
            } else {
                Ok("Command executed with no output.".to_string())
            }
        }
        Err(e) => Err(e),
    }
}
