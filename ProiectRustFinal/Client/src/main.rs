use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::File;
use std::io::{self, stdin, Read, Write};
use std::net::TcpStream;
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    username: String,
    token: String,
    created_at: String,
}

#[derive(Error, Debug)]
enum ErrorType {
    #[error("I/O error")]
    Io(#[from] io::Error),
    #[error("JSON error")]
    Json(#[from] serde_json::Error),
}

fn main() -> Result<(), ErrorType> {
    let mut connected = 0;

    let _ = std::thread::spawn(|| {
        if let Err(_e) = remove_expired_accounts() {
            println!("There is no account saved on this machine");
        }
    });

    match TcpStream::connect("127.0.0.1:80") {
        Ok(mut stream) => {
            println!("Connected!");
            let mut jsonstate = 0; // presupunem ca nu avem niciun token salvat in masina
            let mut file = File::open("src/TokenInfo.json")?;
            let mut data = String::new();
            file.read_to_string(&mut data)?;
            if data.trim().is_empty() {
                println!("Please register or login first.");
            } else {
                let json: Value = serde_json::from_str(&data)?;
                if json.as_object().map_or(false, |obj| !obj.is_empty()) {
                    jsonstate = 1;
                }
                else {
                    println!("Please register or login first.");
                }
            }
            if jsonstate == 1 {
                let buffer = String::from("The machine is already connected");
                let json: Value = serde_json::from_str(&data)?;
                if let Some(username) = json.get("username") {
                    if let Some(token) = json.get("token") {
                        let username = username.as_str().unwrap_or("");
                        let response = format!("{} | {} | {}", buffer, username, token);
                        if let Err(e) = stream.write(response.as_bytes()) {
                            eprintln!("Failed to write the buffer : {}", e);
                        };
                        connected = 1;
                        let mut response = [0; 1024];
                        match stream.read(&mut response) {
                            Ok(bytes_read) => {
                                let response = String::from_utf8_lossy(&response[..bytes_read]);
                                println!("{}", response);
                            }
                            Err(e) => {
                                eprintln!("Failed to read response: {}", e);
                            }
                        }
                    }
                }
            }
            loop {
                let mut buffer = String::new();
                let mut response = [0; 1024];

                match stdin().read_line(&mut buffer) {
                    Ok(_) => {
                        buffer = buffer.trim().to_string();
                        if connected == 0 && jsonstate == 0 {
                            if let Err(e) = stream.write(buffer.as_bytes()) {
                                eprintln!("Failed to write the buffer : {}", e);
                            };
                            match stream.read(&mut response) {
                                Ok(bytes_read) => {
                                    let response = String::from_utf8_lossy(&response[..bytes_read]);
                                    println!("{}", response);
                                    if let Some((user, token)) = response
                                        .split_once("You have logged in succesfully. Your username is: ")
                                        .and_then(|(_, rest)| rest.split_once(" and your token is: "))
                                    {
                                        connected = 1;
                                        let created_at = Utc::now().to_rfc3339();
                                        let new_entry = json!({
                                            "username": user,
                                            "created_at": created_at,
                                            "token": token
                                        });
                                        let new_data = match serde_json::to_string(&new_entry) {
                                            Ok(data) => data,
                                            Err(e) => {
                                                return Err(ErrorType::Json(e));
                                            }
                                        };
                                        data = new_data.clone();
                                        let mut file = match File::create("src/TokenInfo.json") {
                                            Ok(file) => file,
                                            Err(e) => {
                                                return Err(ErrorType::Io(e));
                                            }
                                        };
                                        if let Err(e) = file.write_all(new_data.as_bytes()) {
                                            return Err(ErrorType::Io(e));
                                        }
                                        jsonstate = 1;
                                    } else if let Some((user, token)) = response
                                        .split_once("You have registered succesfully. Your username is: ")
                                        .and_then(|(_, rest)| {
                                            rest.split_once(" and your token is: ")
                                        })
                                    {
                                        connected = 1;
                                        let created_at = Utc::now().to_rfc3339();
                                        let new_entry = json!({
                                            "username": user,
                                            "created_at": created_at,
                                            "token": token
                                        });
                                        let new_data = match serde_json::to_string(&new_entry) {
                                            Ok(data) => data,
                                            Err(e) => {
                                                return Err(ErrorType::Json(e));
                                            }
                                        };
                                        data = new_data.clone();
                                        let mut file = match File::create("src/TokenInfo.json") {
                                            Ok(file) => file,
                                            Err(e) => {
                                                return Err(ErrorType::Io(e));
                                            }
                                        };
                                        if let Err(e) = file.write_all(new_data.as_bytes()) {
                                            return Err(ErrorType::Io(e));
                                        }
                                        jsonstate = 1;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to read response: {}", e);
                                }
                            }
                        } else if buffer.ends_with(" | tpaste") && connected == 1 && jsonstate == 1
                        {
                            let json: Value = serde_json::from_str(&data)?;
                            if let Some(token) = json.get("token") {
                                let token_str = token.as_str().unwrap_or("");
                                let send = format!("{} | {}", buffer, token_str);
                                if let Err(e) = stream.write_all(send.as_bytes()) {
                                    eprintln!("Failed to write the buffer: {}", e);
                                };
                                let mut new_response = [0; 1024];
                                match stream.read(&mut new_response) {
                                    Ok(bytes_read) => {
                                        let response =
                                            String::from_utf8_lossy(&new_response[..bytes_read]);
                                            println!("{}", response);
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to read response: {}", e);
                                        break;
                                    }
                                }
                            } else {
                                println!("Token not found in JSON.");
                            }
                        } else if connected == 1 && jsonstate == 1 {
                            println!("Try using this format : <command> | tpaste");
                        } else {
                            println!("Login first");
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
    Ok(())
}
fn remove_expired_accounts() -> Result<(), ErrorType> {
    loop {
        let mut file = File::open("src/TokenInfo.json")?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        let mut json: Value = serde_json::from_str(&data)?;

        let now = Utc::now();
        let mut is_valid = false;

        if let Some(created_at) = json.get("created_at") {
            if let Some(created_at_str) = created_at.as_str() {
                if let Ok(created_at_date) = DateTime::parse_from_rfc3339(created_at_str) {
                    is_valid = now.signed_duration_since(created_at_date.with_timezone(&Utc))
                        < Duration::days(60);
                }
            }
        }

        if !is_valid {
            json = json!({});
        }

        let new_data = serde_json::to_string(&json)?;
        let mut file = File::create("src/TokenInfo.json")?;
        file.write_all(new_data.as_bytes())?;

        std::thread::sleep(std::time::Duration::from_secs(5));
        //Verificare la fiecare 5 secunde
    }
}
