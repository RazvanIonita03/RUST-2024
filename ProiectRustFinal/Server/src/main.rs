use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use serde_json::{Value,json};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use threadpool::ThreadPool;
use chrono::{Utc, Duration, DateTime};

#[derive(Serialize,Deserialize,Debug)]
struct Person{
    username : String,
    token : String,
    created_at : String,
}

fn main() {
    println!("{}", std::env::current_dir().unwrap().display());
    // Bind to the address and port
    let messages = Arc::new(Mutex::new(Vec::new()));

    let connected = Arc::new(Mutex::new(0));

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Server running on 127.0.0.1:7878");

    let pool = ThreadPool::new(4);

    let _ = std::thread::spawn(|| {
        remove_expired_accounts();
    });

    // Loop to handle incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let messages = Arc::clone(&messages);
                let connected = Arc::clone(&connected);
                pool.execute(move || {
                    handle_client(stream, messages, connected);
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
    mut stream: TcpStream,
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
                        let created_at = Utc::now().to_rfc3339();
                        let response = format!("Register successful. Your token is: {}",token);
                        stream.write_all(response.as_bytes()).unwrap();
                        let new_person = json!({
                            "username": username.to_string(),
                            "token": token.clone(),
                            "created_at": created_at,
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
                    let parts: Vec<&str> = message.trim_start_matches("Login :").trim().split_whitespace().collect();
                    if parts.len() != 2 {
                        let response = "Login failed: Invalid format";
                        stream.write_all(response.as_bytes()).unwrap();
                        continue;
                    }
                    let username = parts[0];
                    let token = parts[1];
                    println!("Username: {}, Token: {}", username, token);
                    let mut login_success = false;
                    let mut token_valid = false;
                    if let Some(users) = json.as_array() {
                        for user in users {
                            if let Some(user_name) = user.get("username") {
                                if user_name == username {
                                    login_success = true;
                                    if let Some(user_token) = user.get("token") {
                                        if user_token == token {
                                            token_valid = true;
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                    }
                    if login_success && token_valid {
                        println!("Login successful!");
                        match connected.lock() {
                            Ok(mut connected_v) => {
                                *connected_v = 1;
                                let response = "Login successful";
                                stream.write_all(response.as_bytes()).unwrap();
                                println!("Sent login message: {}", response);
                            }
                            Err(e) => {
                                eprintln!("Failed to lock connected: {}", e);
                            }
                        }
                    } else if login_success {
                        let response = "Login failed: Incorrect token";
                        stream.write_all(response.as_bytes()).unwrap();
                    } else {
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
                    let command = message.trim_start_matches("COMMAND_OUTPUT:").trim();
                    println!("Merge Comanda!");
                    {
                        let mut messages = messages.lock().unwrap();
                        messages.push(command.to_string());
                    } // Save the message
                    println!("Saved message: {}", command);
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

fn remove_expired_accounts() {
    loop {
        let mut file = File::open("src/Info.json").expect("Unable to open Info.json");
        let mut data = String::new();
        file.read_to_string(&mut data).expect("Unable to read Info.json");
        let mut json: Value = serde_json::from_str(&data).expect("Unable to parse Info.json");

        let now = Utc::now();
        if let Some(users) = json.as_array_mut() {
            users.retain(|user| {
                if let Some(created_at) = user.get("created_at") {
                    if let Some(created_at_str) = created_at.as_str() {
                        if let Ok(created_at_date) = DateTime::parse_from_rfc3339(created_at_str) {
                            return now.signed_duration_since(created_at_date.with_timezone(&Utc)) < Duration::days(60);
                        }
                    }
                }
                false
            });
        }

        let new_data = serde_json::to_string(&json).expect("Unable to serialize JSON");
        let mut file = File::create("src/Info.json").expect("Unable to open Info.json");
        file.write_all(new_data.as_bytes()).expect("Unable to write Info.json");

        // Sleep for a day before checking again
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}