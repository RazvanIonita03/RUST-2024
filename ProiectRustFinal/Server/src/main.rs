use chrono::{DateTime, Duration, Local, Utc};
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

    let _ = std::thread::spawn(|| {
        if let Err(e) = remove_expired_accounts() {
            eprintln!("Failed to remove expired accounts: {}", e);
        }
    });

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
                            } else {
                                eprintln!("Username not found in JSON");
                            }
                        }
                    } else {
                        eprintln!("Users array not found in JSON");
                    }
                    if register_success {
                        let token: String = thread_rng()
                            .sample_iter(&Alphanumeric)
                            .take(10)
                            .map(char::from)
                            .collect();
                        let created_at = Utc::now().to_rfc3339();
                        let response = format!("Register successful. Your token is: {}", token);
                        if let Err(e) = stream.write_all(response.as_bytes()) {
                            eprintln!("Failed to write to stream: {}", e);
                            continue 'outer;
                        }
                        let new_person = json!({
                            "username": username.to_string(),
                            "token": token.clone(),
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
                    } else {
                        let response = "Register failed: Username already exists";
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
                    let token = parts[1].trim();
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
                    } else {
                        eprintln!("Users array not found in JSON");
                    }
                    if login_success && token_valid {
                        println!("Login successful!");
                        match connected.lock() {
                            Ok(mut connected_v) => {
                                *connected_v = 1;
                                let response = "Login successful";
                                stream.write_all(response.as_bytes())?;
                                println!("Sent login message: {}", response);
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
                                                    "Output: {}",
                                                    output
                                                );
                                                println!("Output: {}", output);
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
                    let command = message.trim_end_matches(" | tpaste").trim();
                    match run_piped_command(command){
                        Ok(output) => {
                            if output == "Command executed with no output." {
                                let response = "Command executed with no output.";
                                if let Err(e) = stream.write_all(response.as_bytes()) {
                                    eprintln!("Failed to write to stream: {}", e);
                                }
                            } else if output.starts_with("Command error: ") {
                                let response = format!("{}", output);
                                if let Err(e) = stream.write_all(response.as_bytes()) {
                                    eprintln!("Failed to write to stream: {}", e);
                                }
                            } else {
                                if let Some(users) = json.as_array_mut() {
                                    for user in users.iter_mut() {
                                        if let Some(user_name) = user.get("username") {
                                            let user_name = user_name.as_str().unwrap_or("").trim();
                                            let currentuser_v = currentuser.lock().unwrap();
                                            let currentuser_v = currentuser_v.trim().trim_matches('"');
                                            if user_name == currentuser_v {
                                                if let Some(metadata) = user.get_mut("metadata") {
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
                                        }
                                    }
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
                                }

                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to run command: {}", e);
                        }
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

fn remove_expired_accounts() -> Result<(), ErrorType> {
    loop {
        let mut file = File::open("src/Info.json")?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
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

        let new_data = serde_json::to_string(&json)?;
        let mut file = File::create("src/Info.json")?;
        file.write_all(new_data.as_bytes())?;

        std::thread::sleep(std::time::Duration::from_secs(10));
        //Verificare la fiecare 10 secunde
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