use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

fn main() {

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Server running on 127.0.0.1:7878");


    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                handle_client(&mut stream);
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}


fn handle_client(stream: &mut TcpStream) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(bytes_read) if bytes_read > 0 => {

            let message = String::from_utf8_lossy(&buffer[..bytes_read]);
            println!("Received message: {}", message);

            let mut response = String::from("HTTP/1.1 200 OK\r\n\r\n");
            response.push_str(&message);
            stream.write_all(response.as_bytes()).unwrap();
        }
        Ok(_) => println!("Empty message received."),
        Err(e) => eprintln!("Failed to read message: {}", e),
    }
}