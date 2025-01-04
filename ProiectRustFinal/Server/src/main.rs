use chrono::{DateTime, Duration, Local, Utc};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;

#[derive(Serialize, Deserialize, Debug)]
struct Person {
    username: String,
    token: String,
    created_at: String,
    metadata: HashMap<String, (String, String)>,
}

fn main() {
    let connected = Arc::new(Mutex::new(0));

    let currentuser = Arc::new(Mutex::new(String::new()));

    let listener = TcpListener::bind("127.0.0.1:80").unwrap();
    println!("Server running on 127.0.0.1:80");

    let pool = ThreadPool::new(4);

    let _ = std::thread::spawn(|| {
        remove_expired_accounts();
    });

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let connected = Arc::clone(&connected);
                let currentuser = Arc::clone(&currentuser);
                pool.execute(move || {
                    handle_client(stream, currentuser, connected);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

fn handle_client(
    mut stream: TcpStream,
    currentuser: Arc<Mutex<String>>,
    connected: Arc<Mutex<i32>>,
) {
    let mut file = File::open("src/Info.json").expect("Unable to open Info.json");
    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect("Unable to read Info.json");
    let mut json: Value = serde_json::from_str(&data).expect("Unable to parse Info.json");
    'outer: loop {
        let mut buffer = [0; 1024];
        match stream.read(&mut buffer) {
            Ok(bytes_read) if bytes_read > 0 => {
                // Convert the message to a string and print it
                let message = String::from_utf8_lossy(&buffer[..bytes_read]);
                if *connected.lock().unwrap() == 0 && message.starts_with("Register :") {
                    let username = message.trim_start_matches("Register :").trim();
                    println!("Username: {}", username);
                    let mut register_success = true;
                    if let Some(users) = json.as_array() {
                        for user in users {
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
                        let response = format!("Register successful. Your token is: {}", token);
                        stream.write_all(response.as_bytes()).unwrap();
                        let new_person = json!({
                            "username": username.to_string(),
                            "token": token.clone(),
                            "created_at": created_at,
                            "metadata": HashMap::<String,(String,String)>::new(),
                        });
                        if let Some(users) = json.as_array_mut() {
                            users.push(new_person);
                        }
                        let new_data =
                            serde_json::to_string(&json).expect("Unable to serialize JSON");
                        let mut file =
                            File::create("src/Info.json").expect("Unable to open Info.json");
                        file.write_all(new_data.as_bytes())
                            .expect("Unable to write Info.json");
                    } else {
                        let response = "Register failed: Username already exists";
                        stream.write_all(response.as_bytes()).unwrap();
                    }
                } else if message.starts_with("Login :") && 0 == *connected.lock().unwrap() {
                    let parts: Vec<&str> = message
                        .trim_start_matches("Login :")
                        .trim()
                        .split_whitespace()
                        .collect();
                    if parts.len() != 2 {
                        let response = "Login failed: Invalid format";
                        stream.write_all(response.as_bytes()).unwrap();
                        continue;
                    }
                    let username = parts[0];
                    let token = parts[1];
                    let mut login_success = false;
                    let mut token_valid = false;
                    if let Some(users) = json.as_array() {
                        for user in users {
                            if let Some(user_name) = user.get("username") {
                                if user_name == username {
                                    login_success = true;
                                    if let Some(user_token) = user.get("token") {
                                        if user_token == token {
                                            currentuser.lock().unwrap().clear();
                                            match currentuser.lock() {
                                                Ok(mut currentuser_v) => {
                                                    *currentuser_v = user_name.to_string();
                                                }
                                                Err(e) => {
                                                    eprintln!("Failed to lock connected: {}", e);
                                                }
                                            }
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
                    if message.starts_with("GET /favicon.ico") {
                        break 'outer;
                    } 
                    else {
                        println!("Intram pe site cu mesajul : {}", message);
                        let request_line = message.lines().next().unwrap_or("");
                        let token = request_line
                            .split_whitespace()
                            .nth(1)
                            .unwrap_or("")
                            .trim_start_matches('/');
                        println!("Current user: {}", currentuser.lock().unwrap());
                        println!("Token : {}", token);
                        if token.is_empty() {
                            let mut response_body = String::new();
                            let currentuser_v = currentuser.lock().unwrap();
                            let currentuser_v = currentuser_v.trim().trim_matches('"');
                            response_body.push_str(&format!("{} : ", currentuser_v));

                            if let Some(users) = json.as_array() {
                                for user in users {
                                    if let Some(user_name) = user.get("username") {
                                        let user_name = user_name.as_str().unwrap_or("").trim();
                                        if user_name == currentuser_v {
                                            if let Some(metadata) = user.get("metadata") {
                                                let metadata_map: HashMap<
                                                    String,
                                                    (String, String),
                                                > = serde_json::from_value(metadata.clone())
                                                    .unwrap_or_default();
                                                for (token, _) in metadata_map {
                                                    response_body.push_str(&format!("tpaste.fii/{} ",token));
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            let response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n{}",
                                response_body
                            );

                            stream.write_all(response.as_bytes()).unwrap();
                            stream.flush().unwrap();
                            break 'outer;
                        }
                        let mut response_body = String::new();
                        if let Some(users) = json.as_array() {
                            println!("Users array found");
                            for user in users {
                                if let Some(user_name) = user.get("username") {
                                    let user_name = user_name.as_str().unwrap_or("").trim();
                                    let currentuser_v = currentuser.lock().unwrap();
                                    let currentuser_v = currentuser_v.trim().trim_matches('"');
                                    if user_name == currentuser_v {
                                        if let Some(metadata) = user.get("metadata") {
                                            let metadata_map: HashMap<String, (String, String)> =
                                                serde_json::from_value(metadata.clone())
                                                    .unwrap_or_default();
                                            if let Some((output, timestamp)) =
                                                metadata_map.get(token)
                                            {
                                                let datetime: DateTime<Utc> =
                                                    timestamp.parse().unwrap();
                                                let local_datetime = datetime.with_timezone(&Local);
                                                let formatted_timestamp = local_datetime
                                                    .format("%Y-%m-%d %H:%M:%S")
                                                    .to_string();
                                                response_body = format!(
                                                    "Output: {}\nTimestamp: {}",
                                                    output, formatted_timestamp
                                                );
                                                println!("Output: {}", output);
                                            } else {
                                                println!(
                                                    "Token not found in metadata for user: {}",
                                                    currentuser_v
                                                );
                                            }
                                        } else {
                                            println!(
                                                "Metadata not found for user: {}",
                                                currentuser_v
                                            );
                                        }
                                    } else {
                                        println!("User does not match current user: {}", user_name);
                                    }
                                } else {
                                    println!("Username not found in user object");
                                }
                            }
                        } else {
                            println!("Users array not found in JSON");
                        }
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n{}",
                            response_body
                        );

                        stream.write_all(response.as_bytes()).unwrap();
                        break 'outer;
                    }
                } else if message.starts_with("COMMAND_OUTPUT:") && 1 == *connected.lock().unwrap()
                {
                    let command = message.trim_start_matches("COMMAND_OUTPUT:").trim();
                    if let Some(users) = json.as_array_mut() {
                        for user in users.iter_mut() {
                            if let Some(user_name) = user.get("username") {
                                let user_name = user_name.as_str().unwrap_or("").trim();
                                let currentuser_v = currentuser.lock().unwrap();
                                let currentuser_v = currentuser_v.trim().trim_matches('"');
                                if user_name == currentuser_v {
                                    if let Some(metadata) = user.get_mut("metadata") {
                                        let mut metadata_map: HashMap<String, (String, String)> =
                                            serde_json::from_value(metadata.take())
                                                .unwrap_or_default();
                                        let random_token: String = thread_rng()
                                            .sample_iter(&Alphanumeric)
                                            .take(10)
                                            .map(char::from)
                                            .collect();
                                        let time_called = Utc::now().to_rfc3339();
                                        metadata_map.insert(
                                            random_token.clone(),
                                            (command.to_string(), time_called.to_string()),
                                        );
                                        *metadata = serde_json::to_value(metadata_map).unwrap();
                                        let link = format!("http://tpaste.fii/{}", random_token);
                                        let response = format!("Output saved. Access it at: {}", link);
                                        println!("Response: {}", response);
                                        stream.write_all(response.as_bytes()).unwrap();
                                    }
                                }
                            }
                        }
                    }
                    let new_data = serde_json::to_string(&json).expect("Unable to serialize JSON");
                    let mut file = File::create("src/Info.json").expect("Unable to open Info.json");
                    file.write_all(new_data.as_bytes())
                        .expect("Unable to write Info.json");
                } else {
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
        file.read_to_string(&mut data)
            .expect("Unable to read Info.json");
        let mut json: Value = serde_json::from_str(&data).expect("Unable to parse Info.json");

        let now = Utc::now();
        if let Some(users) = json.as_array_mut() {
            users.retain(|user| {
                if let Some(created_at) = user.get("created_at") {
                    if let Some(created_at_str) = created_at.as_str() {
                        if let Ok(created_at_date) = DateTime::parse_from_rfc3339(created_at_str) {
                            return now.signed_duration_since(created_at_date.with_timezone(&Utc))
                                < Duration::days(60);
                        }
                    }
                }
                false
            });
        }

        let new_data = serde_json::to_string(&json).expect("Unable to serialize JSON");
        let mut file = File::create("src/Info.json").expect("Unable to open Info.json");
        file.write_all(new_data.as_bytes())
            .expect("Unable to write Info.json");

        std::thread::sleep(std::time::Duration::from_secs(10));
        //Verificare la fiecare 10 secunde
    }
}
