use chrono::format;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct user_info {
    name: String,
    email: String,
}
#[derive(Serialize)]
struct Response {
    message: String,
}
impl user_info {
    fn insert_user_info(&self) -> Result<()> {
        let conn = Connection::open("user_info.db")?;

        let mut prep = conn.prepare("SELECT COUNT(*) FROM users WHERE email =?1")?;
        let count: i32 = prep.query_row(params![&self.email], |row| row.get(0))?;
        if count == 0 {
            conn.execute(
                "INSERT INTO users (name, email) VALUES (?1, ?2)",
                params![&self.name, &self.email],
            )?;

            Ok(())
        } else {
            return Err(rusqlite::Error::QueryReturnedNoRows); // Or a custom error
        }
    }
}

use if_addrs::get_if_addrs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:5000")?;
    println!("Server running on 0.0.0.0:5000");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("New connection received!");
                // Pass the connection to the handler
                handle_request(&mut stream);
            }
            Err(e) => {
                eprintln!("Connection error: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_request(stream: &mut TcpStream) {
    let get = String::from("GET");
    let post = String::from("POST");

    let store_user_data_post_request = "/send_form";
    let (method, url, body, is_http) = is_http(stream);
    println!("{}", method);
    println!("{}", body);
    println!("{}", url);

    if is_http == false {
        println!("the server said its not http request so it will get droped ");
    } else {
        if method == get {
            handle_get(stream, url);
        } else if method == post {
            if url == store_user_data_post_request.to_string() {
                handle_post_for_store_user_data(stream, url, body);
            }
        }
    }
}

fn is_http(stream: &mut TcpStream) -> (String, String, String, bool) {
    let mut buffer = [0; 512]; // Create a buffer to store the incoming data
    let mut request = String::new();

    // Read data from the stream
    match stream.read(&mut buffer) {
        Ok(size) => {
            request.push_str(&String::from_utf8_lossy(&buffer[..size]));
        }
        Err(_) => {
            return ("".to_string(), "".to_string(), "".to_string(), false);
        }
    }

    let mut lines = request.lines();
    let first_line = match lines.next() {
        Some(line) => line,
        None => return ("".to_string(), "".to_string(), "".to_string(), false),
    };

    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 3 {
        return ("".to_string(), "".to_string(), "".to_string(), false);
    }

    let method = parts[0].to_string();
    let url = parts[1].to_string();
    let version = parts[2].to_string();

    if !version.starts_with("HTTP/") {
        return ("".to_string(), "".to_string(), "".to_string(), false);
    }
    let mut headers_finished = false;
    let mut body = String::new();

    if method == "POST" || method == "PUT" {
        for line in lines {
            if headers_finished {
                body.push_str(line);
            } else if line.is_empty() {
                headers_finished = true;
            }
        }
    }
    (method, url, body, true)
}
fn handle_get(stream: &mut TcpStream, url: String) {
    let base_path = "/home/ziad/git/HTTP_on_top_TCP/src/";
    let new_url = url.as_str();
    let path_for_resource = map_url_to_file(&base_path, &new_url);
    if let Some(path) = path_for_resource {
        if let Ok(content) = fs::read(&path) {
            let content_type = match path.extension().and_then(|ext| ext.to_str()) {
                Some("html") => "text/html",
                Some("css") => "text/css",
                Some("js") => "application/javascript",
                Some("png") => "image/png",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                _ => "text/plain",
            };
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
                content_type,
                content.len()
            );
            stream.write_all(response.as_bytes()).unwrap();
            stream.write_all(&content).unwrap();
        } else {
            let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
            stream.write_all(response.as_bytes()).unwrap();
        }
    } else {
        let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
        stream.write_all(response.as_bytes()).unwrap();
    }
}

fn handle_post_for_store_user_data(stream: &mut TcpStream, url: String, body: String) {
    let form_data: Result<user_info, serde_json::Error> = serde_json::from_str(&body);

    match form_data {
        Ok(data) => {
            println!("Data received successfully, converted to JSON and stored in struct for ease of use");

            match data.insert_user_info() {
                Ok(_) => {
                    // Create a response message
                    let response = Response {
                        message: "Your data is saved successfully".to_string(),
                    };

                    // Serialize the response to JSON
                    let json_message =
                        serde_json::to_string(&response).expect("cannot Serialize the response");

                    stream
                        .write_all(
                            format!(
                                "HTTP1.1 200 OK\r\nContent-Length: {}
                         \r\nContent-Type: application/json\r\n\r\n{}",
                                json_message.len(),
                                json_message
                            )
                            .as_bytes(),
                        )
                        .expect("error");
                    // Send the JSON response to the client or log it
                }
                Err(e) => {
                    let response = Response {
                        message: "The email is already there try new email ".to_string(),
                    };

                    let json_message =
                        serde_json::to_string(&response).expect("cannot Serialize the response");

                    stream
                        .write_all(
                            format!(
                                "HTTP1.1 200 OK\r\nContent-Length: {}
                         \r\nContent-Type: application/json\r\n\r\n{}",
                                json_message.len(),
                                json_message
                            )
                            .as_bytes(),
                        )
                        .expect("error");
                }
            }
        }
        Err(data) => {
            println!("data received but not correctly and dont match the format should be so it will be droped");
        }
    }
}

fn map_url_to_file(base_dir: &str, url_path: &str) -> Option<PathBuf> {
    let mut safe_path = url_path.trim_start_matches('/');
    if safe_path.is_empty() {
        safe_path = "index.html";
    }
    if safe_path.ends_with("/") {
        safe_path = &safe_path[..safe_path.len() - 1];
    }
    let full_path = Path::new(base_dir).join(safe_path);

    if full_path.is_file() {
        Some(full_path)
    } else {
        None
    }
}
