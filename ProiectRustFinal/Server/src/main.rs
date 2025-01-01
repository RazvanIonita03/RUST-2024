use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use serde::{Deserialize, Serialize};
use serde_json::{Value,json};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

#[derive(Serialize,Deserialize,Debug)]
struct Person{
    username : String,
    token : String,
}

fn main() {
    println!("{}", std::env::current_dir().unwrap().display());
    // Bind to the address and port
    let messages = Arc::new(Mutex::new(Vec::new()));

    let connected = Arc::new(Mutex::new(0));

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Server running on 127.0.0.1:7878");

    // Loop to handle incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let messages = Arc::clone(&messages);
                let connected = Arc::clone(&connected);
                thread::spawn(move || {
                    handle_client(&mut stream, messages, connected);
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
    messages: Arc<Mutex<Vec<String>>>,
    connected: Arc<Mutex<i32>>,
) {

    let mut file = File::open("src/Info.json").expect("Unable to open Info.json");
    let mut data = String::new();
    file.read_to_string(&mut data).expect("Unable to read Info.json");
    let mut json: Value = serde_json::from_str(&data).expect("Unable to parse Info.json");
    'outer : loop {
        let mut buffer = [0; 1024]; // Buffer to hold the incoming message
        match stream.read(&mut buffer) {
            Ok(bytes_read) if bytes_read > 0 => {
                // Convert the message to a string and print it
                let message = String::from_utf8_lossy(&buffer[..bytes_read]);
                println!("Received message: {}", message);
                if *connected.lock().unwrap() == 0 && message.starts_with("Register :"){
                    let username = message.trim_start_matches("Register :").trim();
                    println!("Username: {}", username);
                    let mut register_success = true;
                    if let Some(users) = json.as_array(){
                        for user in users{
                            if let Some(user_name) = user.get("username") {
                                println!("User: {}", user_name);
                                if user_name == username {
                                    register_success = false;
                                    break;
                                }
                            }
                        }
                    }
                    if register_success {
                        println!("A mers register!");
                        let token: String = thread_rng()
                            .sample_iter(&Alphanumeric)
                            .take(10)
                            .map(char::from)
                            .collect();
                        let response = format!("Register successful. Your token is: {}",token);
                        stream.write_all(response.as_bytes()).unwrap();
                        let new_person = json!({
                            "username": username.to_string(),
                            "token": token.clone(),
                        });
                        if let Some(users) = json.as_array_mut() {
                            users.push(new_person);
                        }
                        let new_data = serde_json::to_string(&json).expect("Unable to serialize JSON");
                        let mut file = File::create("src/Info.json").expect("Unable to open Info.json");
                        file.write_all(new_data.as_bytes()).expect("Unable to write Info.json");
                    }
                    else {
                        let response = "Register failed: Username already exists";
                        stream.write_all(response.as_bytes()).unwrap();
                    }
                } else
                if message.starts_with("Login :") && 0 == *connected.lock().unwrap() {
                    let username = message.trim_start_matches("Login :").trim();
                    println!("Username: {}", username);
                    let mut login_success = false;
                    if let Some(users) = json.as_array(){
                        for user in users{
                            if let Some(user_name) = user.get("username") {
                                println!("User: {}", user_name);
                                if user_name == username {
                                    login_success = true;
                                    break;
                                }
                            }
                        }
                    }
                    if login_success {
                        println!("A mers login!");
                        {
                            let mut connected = connected.lock().unwrap();
                            *connected = 1;
                        }
                        let response = "Login successful";
                        stream.write_all(response.as_bytes()).unwrap();
                        println!("Am scris mesajul de login : {}", response);
                    }
                    else {
                        let response = "Login failed: Username not found";
                        stream.write_all(response.as_bytes()).unwrap();
                    }
                    
                } else if message.starts_with("GET") && 1 == *connected.lock().unwrap() {
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
                } 
                else if message.starts_with("COMMAND_OUTPUT:") && 1 == *connected.lock().unwrap() {
                    // Handle a custom message from the client
                    println!("Merge Comanda!");
                    {
                        let mut messages = messages.lock().unwrap();
                        messages.push(message.trim().to_string());
                    } // Save the message
                    println!("Saved message: {}", message.trim());
                }
                else{
                    let response = "Comanda invalida";
                    stream.write_all(response.as_bytes()).unwrap();
                }
            }
            Ok(_) => {
                //
            }
            Err(e) => eprintln!("Failed to read message: {}", e),
        }
    }
}
