use std::io::{prelude::*, stdin};
use std::net::TcpStream;

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
                        buffer = buffer.trim().to_string();
                        if connected == 0 {
                            if let Err(e) = stream.write(buffer.as_bytes()) {
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
                        } else if buffer.ends_with(" | tpaste") {
                            
                            if let Err(e) = stream.write(buffer.as_bytes()) {
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
                        } else {
                            println!("Try using this format : <command> | tpaste");
                        };
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