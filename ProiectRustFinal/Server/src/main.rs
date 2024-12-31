use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() {
    // Bind to the address and port
    let messages = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));

    let mut connected = 0;

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Server running on 127.0.0.1:7878");

    // Loop to handle incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let messages = messages.clone();
                thread::spawn(move || {
                    handle_client(&mut stream, messages, &mut connected);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

// Function to handle client messages
fn handle_client(
    stream: &mut TcpStream,
    messages: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    connected: &mut i32,
) {

    'outer : loop {
        let mut buffer = [0; 1024]; // Buffer to hold the incoming message
        match stream.read(&mut buffer) {
            Ok(bytes_read) if bytes_read > 0 => {
                // Convert the message to a string and print it
                let message = String::from_utf8_lossy(&buffer[..bytes_read]);
                println!("Received message: {}", message);
                if message.starts_with("Login :") && 0 == *connected {
                    println!("A mers login!");
                    *connected = 1;
                    let response = "Login successful";
                    stream.write_all(response.as_bytes()).unwrap();
                } else if message.starts_with("GET") && 0 == *connected {
                    // Handle an HTTP request
                    println!("Intram pe site");
                    let response_body = {
                        let messages = messages.lock().unwrap();
                        messages.join("\n")
                    }; // Concatenate messages with newlines
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n{}",
                        response_body
                    );

                    stream.write_all(response.as_bytes()).unwrap();
                    break 'outer;
                } else if message.starts_with("GET") && 0 == *connected {
                    let response = "Conecteaza-te prima oara boss";
                    stream.write_all(response.as_bytes()).unwrap();
                } else if message.starts_with("COMMAND_OUTPUT:") && 1 == *connected {
                    // Handle a custom message from the client
                    println!("Merge Comanda!");
                    {
                        let mut messages = messages.lock().unwrap();
                        messages.push(message.trim().to_string());
                    } // Save the message
                    println!("Saved message: {}", message.trim());
                }
            }
            Ok(_) => {
                //
            }
            Err(e) => eprintln!("Failed to read message: {}", e),
        }
    }
}
