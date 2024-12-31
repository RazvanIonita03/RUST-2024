use std::net::TcpStream;
use std::io::{Read, Write};

fn main() {
    match TcpStream::connect("127.0.0.1:7878") {
        Ok(mut stream) => {
            println!("Connected to the server!");

            let message = "Message";
            stream.write(message.as_bytes()).unwrap();
            println!("Sent: {}", message);

            let mut buffer = [0; 1024];
            match stream.read(&mut buffer) {
                Ok(bytes_read) if bytes_read > 0 => {
                    let response = String::from_utf8_lossy(&buffer[..bytes_read]);
                    println!("Received: {}", response);
                }
                Ok(_) => {
                    println!("Server sent an empty response.");
                }
                Err(e) => {
                    eprintln!("Failed to read from server: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to connect to server: {}", e);
        }
    }
}