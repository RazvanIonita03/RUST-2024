use chrono::{DateTime, Local, Utc};
use core::fmt;
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
use std::process::Command;

#[derive(Serialize, Deserialize, Debug)]
struct Person {
    username: String,
    password: String,
    token: String,
    created_at: String,
    metadata: HashMap<String, (String, String)>,
}

#[derive(Debug)]
enum ErrorType {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorType::IoError(e) => write!(f, "IO Error: {}", e),
            ErrorType::JsonError(e) => write!(f, "JSON Error: {}", e),
        }
    }
}

impl From<std::io::Error> for ErrorType {
    fn from(e: std::io::Error) -> Self {
        ErrorType::IoError(e)
    }
}

impl From<serde_json::Error> for ErrorType {
    fn from(e: serde_json::Error) -> Self {
        ErrorType::JsonError(e)
    }
}

fn main() -> Result<(), ErrorType> {
    let connected = Arc::new(Mutex::new(0));

    let currentuser = Arc::new(Mutex::new(String::new()));

    let listener = TcpListener::bind("127.0.0.1:80")?;
    println!("Server running on 127.0.0.1:80");

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let connected = Arc::clone(&connected);
                let currentuser = Arc::clone(&currentuser);
                pool.execute(move|| {
                    if let Err(e) = handle_client(stream, currentuser, connected) {
                        eprintln!("Failed to handle client: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
    Ok(())
}

fn handle_client(
    mut stream: TcpStream,
    currentuser: Arc<Mutex<String>>,
    connected: Arc<Mutex<i32>>,
) -> Result<(), ErrorType> {
    let mut file = File::open("src/Info.json")?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let mut json: Value = serde_json::from_str(&data)?;
    'outer: loop {
        let mut buffer = [0; 1024];
        match stream.read(&mut buffer) {
            Ok(bytes_read) if bytes_read > 0 => {
                let message = String::from_utf8_lossy(&buffer[..bytes_read]);
                if message.starts_with("The machine is already connected") {
                    let message_parts: Vec<&str> = message.split(" | ").collect();
                    if message_parts.len() == 3 {
                        let username = message_parts[1].trim();
                        let user_token = message_parts[2].trim();
                        if let Some(users) = json.as_array() {
                            for user in users {
                                if let Some(user_name) = user.get("username") {
                                    if user_name == username {
                                        if let Some(token) = user.get("token") {
                                            let token_str = token.as_str().unwrap_or("").trim();
                                            let user_token_trimmed = user_token.trim().trim_matches('"');
                                            if token_str == user_token_trimmed {
                                                match currentuser.lock() {
                                                    Ok(mut currentuser_v) => {
                                                        *currentuser_v = username.to_string();
                                                    }
                                                    Err(e) => {
                                                        eprintln!("Failed to lock connected: {}", e);
                                                    }
                                                }
                                                match connected.lock() {
                                                    Ok(mut connected_v) => {
                                                        *connected_v = 1;
                                                    }
                                                    Err(e) => {
                                                        eprintln!("Failed to lock connected: {}", e);
                                                    }
                                                }
                                                stream.write_all("You are already connected. Please input your commands".as_bytes())?;
                                            }
                                        } else {
                                            stream.write_all("Token is not valid".as_bytes())?;
                                            continue 'outer;
                                        }
                                    }
                                } 
                                else {
                                    eprintln!("Username not found in JSON");
                                    continue 'outer;
                                }
                            }
                        } else {
                            eprintln!("Users array not found in JSON");
                            continue 'outer;
                        }
                    }
                }
                else if *connected.lock().unwrap() == 0 && message.starts_with("Register :") {
                    let mut register_success = true;
                    let parts: Vec<&str> = message
                        .trim_start_matches("Register :")
                        .split_whitespace()
                        .collect();
                    if parts.len() != 2 {
                        let response = "Register failed. Invalid format";
                        if let Err(e) = stream.write_all(response.as_bytes()) {
                            eprintln!("Failed to write to stream: {}", e);
                        }
                        continue;
                    }
                    let username = parts[0].trim();
                    let password = parts[1].trim();
                    if let Some(users) = json.as_array() {
                        for user in users {
                            if let Some(user_name) = user.get("username") {
                                println!("User: {}", user_name);
                                if user_name == username {
                                    register_success = false;
                                    break;
                                }
                            } else {
                                eprintln!("Username not found in JSON");
                            }
                        }
                    } else {
                        eprintln!("Users array not found in JSON");
                    }
                    if register_success {
                        currentuser.lock().unwrap().clear();
                        match currentuser.lock() {
                            Ok(mut currentuser_v) => {
                                *currentuser_v = username.to_string();
                            }
                            Err(e) => {
                                eprintln!("Failed to lock connected: {}", e);
                            }
                        }
                        match connected.lock() {
                            Ok(mut connected_v) => {
                                *connected_v = 1;
                            }
                            Err(e) => {
                                eprintln!("Failed to lock connected: {}", e);
                            }
                        }
                        let token: String = thread_rng()
                            .sample_iter(&Alphanumeric)
                            .take(10)
                            .map(char::from)
                            .collect();
                        let created_at = Utc::now().to_rfc3339();
                        let new_person = json!({
                            "username": username.to_string(),
                            "token": token.clone(),
                            "password": password.to_string(),
                            "created_at": created_at,
                            "metadata": HashMap::<String,(String,String)>::new(),
                        });
                        if let Some(users) = json.as_array_mut() {
                            users.push(new_person);
                        } else {
                            eprintln!("Users array not found in JSON");
                        }
                        let new_data = match serde_json::to_string(&json) {
                            Ok(data) => data,
                            Err(e) => {
                                eprintln!("Failed to serialize JSON: {}", e);
                                continue 'outer;
                            }
                        };
                        let mut file = match File::create("src/Info.json") {
                            Ok(file) => file,
                            Err(e) => {
                                eprintln!("Failed to create Info.json: {}", e);
                                continue 'outer;
                            }
                        };
                        if let Err(e) = file.write_all(new_data.as_bytes()) {
                            eprintln!("Failed to write to Info.json: {}", e);
                            continue 'outer;
                        }
                        let response = format!("You have registered succesfully. Your username is: {} and your token is: {}", username,token);
                        if let Err(e) = stream.write_all(response.as_bytes()) {
                            eprintln!("Failed to write to stream: {}", e);
                            continue 'outer;
                        }
                    } else {
                        let response = "Registration has failed. The username already exists in the database";
                        if let Err(e) = stream.write_all(response.as_bytes()) {
                            eprintln!("Failed to write to stream: {}", e);
                        }
                    }
                } else if message.starts_with("Login :") && 0 == *connected.lock().unwrap() {
                    let parts: Vec<&str> = message
                        .trim_start_matches("Login :")
                        .split_whitespace()
                        .collect();
                    if parts.len() != 2 {
                        let response = "Login failed: Invalid format";
                        if let Err(e) = stream.write_all(response.as_bytes()) {
                            eprintln!("Failed to write to stream: {}", e);
                        }
                        continue;
                    }
                    let username = parts[0].trim();
                    let password = parts[1].trim();
                    let mut login_success = false;
                    let mut password_valid = false;
                    if let Some(users) = json.as_array() {
                        for user in users {
                            if let Some(user_name) = user.get("username") {
                                if user_name == username {
                                    login_success = true;
                                    if let Some(user_password) = user.get("password") {
                                        if user_password == password {
                                            currentuser.lock().unwrap().clear();
                                            match currentuser.lock() {
                                                Ok(mut currentuser_v) => {
                                                    *currentuser_v = user_name.to_string();
                                                }
                                                Err(e) => {
                                                    eprintln!("Failed to lock connected: {}", e);
                                                }
                                            }
                                            password_valid = true;
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                    } else {
                        eprintln!("Users array not found in JSON");
                    }
                    if login_success && password_valid {
                        match connected.lock() {
                            Ok(mut connected_v) => {
                                *connected_v = 1;
                                let new_token: String = thread_rng()
                                    .sample_iter(&Alphanumeric)
                                    .take(10)
                                    .map(char::from)
                                    .collect();
                                if let Some(users) = json.as_array_mut() {
                                    for user in users.iter_mut() {
                                        if let Some(user_name) = user.get("username") {
                                            if user_name == username {
                                                if let Some(token) = user.get_mut("token") {
                                                    *token = Value::String(new_token.clone());
                                                }
                                            }
                                        }
                                    }
                                }
                                let new_data = match serde_json::to_string(&json) {
                                    Ok(data) => data,
                                    Err(e) => {
                                        eprintln!("Failed to serialize JSON: {}", e);
                                        return Err(ErrorType::JsonError(e));
                                    }
                                };
                                let mut file = match File::create("src/Info.json") {
                                    Ok(file) => file,
                                    Err(e) => {
                                        eprintln!("Failed to create Info.json: {}", e);
                                        return Err(ErrorType::IoError(e));
                                    }
                                };
                                if let Err(e) = file.write_all(new_data.as_bytes()) {
                                    eprintln!("Failed to write to Info.json: {}", e);
                                    return Err(ErrorType::IoError(e));
                                }
                                let response = format!("You have logged in succesfully. Your username is: {} and your token is: {}", username, new_token);
                                stream.write_all(response.as_bytes())?;
                                continue 'outer;
                            }
                            Err(e) => {
                                eprintln!("Failed to lock connected: {}", e);
                            }
                        }
                    } else if login_success {
                        let response = "Login failed: Incorrect token";
                        if let Err(e) = stream.write_all(response.as_bytes()) {
                            eprintln!("Failed to write to stream: {}", e);
                        }
                    } else {
                        let response = "Login failed: Username not found";
                        if let Err(e) = stream.write_all(response.as_bytes()) {
                            eprintln!("Failed to write to stream: {}", e);
                        }
                    }
                } else if message.starts_with("GET") && 1 == *connected.lock().unwrap() {
                    if message.starts_with("GET /favicon.ico") {
                        return Ok(());
                    } else {
                        let request_line = message.lines().next().unwrap_or("");
                        println!("Request line: {}", request_line);
                        let token = request_line
                            .split_whitespace()
                            .nth(1)
                            .unwrap_or("")
                            .trim_start_matches('/');
                        println!("Token = {} ", token);
                        if token.is_empty() {
                            let mut response_body = String::new();
                            let currentuser_v = currentuser.lock().unwrap();
                            let currentuser_v = currentuser_v.trim().trim_matches('"');
                            response_body.push_str(&format!("{} :\n\n", currentuser_v));

                            if let Some(users) = json.as_array() {
                                for user in users {
                                    if let Some(user_name) = user.get("username") {
                                        let user_name = user_name.as_str().unwrap_or("").trim();
                                        if user_name == currentuser_v {
                                            if let Some(metadata) = user.get("metadata") {
                                                let metadata_map: HashMap<
                                                    String,
                                                    (String, String),
                                                > = match serde_json::from_value(metadata.clone()) {
                                                    Ok(map) => map,
                                                    Err(e) => {
                                                        eprintln!(
                                                            "Failed to deserialize metadata: {}",
                                                            e
                                                        );
                                                        continue 'outer;
                                                    }
                                                };
                                                for (token, (_output,timestamp)) in metadata_map { 
                                                    let datetime: DateTime<Utc> =
                                                    match timestamp.parse() {
                                                        Ok(dt) => dt,
                                                        Err(e) => {
                                                            eprintln!(
                                                                "Failed to parse timestamp: {}",
                                                                e
                                                            );
                                                            continue 'outer;
                                                        }
                                                    };
                                                    let local_datetime = datetime.with_timezone(&Local);
                                                    let formatted_timestamp = local_datetime
                                                        .format("%Y-%m-%d %H:%M:%S")
                                                        .to_string();
                                                    response_body.push_str(&format!(
                                                        "tpaste.fii/{}         Time of creation :{}\n",
                                                        token, formatted_timestamp
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                eprintln!("Users array not found in JSON");
                            }

                            let response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n{}",
                                response_body
                            );

                            if let Err(e) = stream.write_all(response.as_bytes()) {
                                eprintln!("Failed to write to stream: {}", e);
                            }
                            if let Err(e) = stream.flush() {
                                eprintln!("Failed to flush stream: {}", e);
                            }
                            return Ok(());
                        }
                        let mut response_body = String::new();
                        if let Some(users) = json.as_array() {
                            for user in users {
                                if let Some(user_name) = user.get("username") {
                                    let user_name = user_name.as_str().unwrap_or("").trim();
                                    let currentuser_v = currentuser.lock().unwrap();
                                    let currentuser_v = currentuser_v.trim().trim_matches('"');
                                    if user_name == currentuser_v {
                                        if let Some(metadata) = user.get("metadata") {
                                            let metadata_map: HashMap<String, (String, String)> =
                                                match serde_json::from_value(metadata.clone()) {
                                                    Ok(map) => map,
                                                    Err(e) => {
                                                        eprintln!(
                                                            "Failed to deserialize metadata: {}",
                                                            e
                                                        );
                                                        continue 'outer;
                                                    }
                                                };
                                            if let Some((output, _)) =
                                                metadata_map.get(token)
                                            {
                                                response_body = format!(
                                                    "Command output: {}",
                                                    output
                                                );
                                            } else {
                                                eprintln!(
                                                    "Token not found in metadata for user: {}",
                                                    currentuser_v
                                                );
                                            }
                                        } else {
                                            eprintln!(
                                                "Metadata not found for user: {}",
                                                currentuser_v
                                            );
                                        }
                                    } else {
                                        eprintln!(
                                            "User does not match current user: {}",
                                            user_name
                                        );
                                    }
                                } else {
                                    eprintln!("Username not found in user object");
                                }
                            }
                        } else {
                            eprintln!("Users array not found in JSON");
                        }
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n{}",
                            response_body
                        );

                        if let Err(e) = stream.write_all(response.as_bytes()) {
                            eprintln!("Failed to write to stream: {}", e);
                        }
                        return Ok(());
                    }
                } else if 1 == *connected.lock().unwrap() {
                    let message_parts: Vec<&str> = message.split(" | ").collect();
                    if message_parts.len() == 3 {
                        let command = message_parts[0].trim();
                        let user_token = message_parts[2].trim();
                        match currentuser.lock() {
                            Ok(currentuser_v) => {
                                println!("Current user: {}", currentuser_v);
                                if let Some(users) = json.as_array_mut() {
                                    for user in users.iter_mut() {
                                        if let Some(user_name) = user.get("username") {
                                            let user_name = user_name.as_str().unwrap_or("").trim().trim_matches('"');
                                            if user_name == currentuser_v.trim().trim_matches('"') {
                                                if let Some(token) = user.get("token") {
                                                    let token_str = token.as_str().unwrap_or("").trim();
                                                    let user_token_trimmed = user_token.trim().trim_matches('"');
                                                    if token_str == user_token_trimmed {
                                                        match run_piped_command(command){
                                                            Ok(output) => {
                                                                if output == "Command executed with no output." {
                                                                    let response = "Command executed with no output.";
                                                                    if let Err(e) = stream.write_all(response.as_bytes()) {
                                                                        eprintln!("Failed to write to stream: {}", e);
                                                                    }
                                                                } else if output.starts_with("Command error: ") {
                                                                    if let Err(e) = stream.write_all(output.as_bytes()) {
                                                                        eprintln!("Failed to write to stream: {}", e);
                                                                    }
                                                                } else if let Some(metadata) = user.get_mut("metadata") {
                                                                    let mut metadata_map: HashMap<String, (String, String)> =
                                                                        match serde_json::from_value(metadata.take()) {
                                                                            Ok(map) => map,
                                                                            Err(e) => {
                                                                                eprintln!(
                                                                                    "Failed to deserialize metadata: {}",
                                                                                    e
                                                                                );
                                                                                continue 'outer;
                                                                            }
                                                                        };
                                                                    let random_token: String = thread_rng()
                                                                        .sample_iter(&Alphanumeric)
                                                                        .take(10)
                                                                        .map(char::from)
                                                                        .collect();
                                                                    let time_called = Utc::now().to_rfc3339();
                                                                    match metadata_map.insert(
                                                                        random_token.clone(),
                                                                        (output.to_string(), time_called.to_string()),
                                                                    ) {
                                                                        Some(_) => {
                                                                            eprintln!("Token already exists in metadata");
                                                                        }
                                                                        None => {
                                                                            *metadata = serde_json::to_value(metadata_map)?;
                                                                            let link =
                                                                                format!("http://tpaste.fii/{}", random_token);
                                                                            let response =
                                                                                format!("Output saved. Access it at: {}", link);
                                                                            println!("Response: {}", response);
                                                                            stream.write_all(response.as_bytes())?;
                                                                        }
                                                                    }
                                                                
                                                                }
                                                            }
                                                            Err(e) => {
                                                                eprintln!("Failed to run command: {}", e);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to lock currentuser: {}", e);
                            }
                        }
                        
                    }
                    let new_data = match serde_json::to_string(&json) {
                        Ok(data) => data,
                        Err(e) => {
                            eprintln!("Failed to serialize JSON: {}", e);
                            continue 'outer;
                        }
                    };
                    let mut file = match File::create("src/Info.json") {
                        Ok(file) => file,
                        Err(e) => {
                            eprintln!("Failed to create Info.json: {}", e);
                            continue 'outer;
                        }
                    };
                    if let Err(e) = file.write_all(new_data.as_bytes()) {
                        eprintln!("Failed to write to Info.json: {}", e);
                    }
                } else {
                    let response = "Invalid command";
                    if let Err(e) = stream.write_all(response.as_bytes()) {
                        eprintln!("Failed to write to stream: {}", e);
                    }
                }
            }
            Ok(_) => {
                //
            }
            Err(e) => eprintln!("Failed to read message: {}", e),
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