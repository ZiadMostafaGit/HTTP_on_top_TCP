use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::any::type_name;

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

#[derive(Deserialize)]
struct UserInfo {
    name: String,
    email: String,
}

#[derive(Serialize)]
struct Response {
    message: String,
}

fn type_of<T>(_: &T) -> &'static str {
    type_name::<T>()
}

impl UserInfo {
    fn insert_user_info(&self) -> Result<()> {
        let conn = Connection::open("user_info.db")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT, email TEXT UNIQUE)",
            [],
        )?;

        let mut stmt = conn.prepare("SELECT COUNT(*) FROM users WHERE email = ?1")?;
        let count: i32 = stmt.query_row(params![&self.email], |row| row.get(0))?;

        if count == 0 {
            conn.execute(
                "INSERT INTO users (name, email) VALUES (?1, ?2)",
                params![&self.name, &self.email],
            )?;
            Ok(())
        } else {
            Err(rusqlite::Error::QueryReturnedNoRows)
        }
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:5000")?;
    println!("Server running on 0.0.0.0:5000");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                thread::spawn(move || {
                    println!("New connection received!");
                    handle_request(&mut stream);
                });
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

    loop {
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();

        let (method, url, body, is_http, keep_alive) = is_http(stream);

        println!("{}", method);
        println!("{}", url);
        println!("{}", body);
        println!("{}", keep_alive);
        if !is_http {
            let response = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
            stream.write_all(response.as_bytes()).unwrap();
            break;
        }

        if method == get {
            handle_get(stream, url);
        } else if method == post {
            if url == store_user_data_post_request {
                handle_post_for_store_user_data(stream, url, body);
            }
        }

        if !keep_alive {
            break;
        }
    }
}

fn is_http(stream: &mut TcpStream) -> (String, String, String, bool, bool) {
    let mut buffer = [0; 1024];
    let mut request = String::new();

    match stream.read(&mut buffer) {
        Ok(size) => {
            request.push_str(&String::from_utf8_lossy(&buffer[..size]));
        }
        Err(_) => return ("".to_string(), "".to_string(), "".to_string(), false, false),
    }

    let mut lines = request.lines();
    println!("{}", type_of(&lines));
    let first_line = match lines.next() {
        Some(line) => line,
        None => return ("".to_string(), "".to_string(), "".to_string(), false, false),
    };

    for line in lines.clone() {
        println!("{:?}", line);
    }

    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 3 {
        return ("".to_string(), "".to_string(), "".to_string(), false, false);
    }

    let method = parts[0].to_string();
    let url = parts[1].to_string();
    let version = parts[2].to_string();

    if !version.starts_with("HTTP/") {
        return ("".to_string(), "".to_string(), "".to_string(), false, false);
    }

    let mut headers_finished = false;
    let mut body = String::new();
    let mut keep_alive = false;

    for line in lines {
        if headers_finished {
            body.push_str(line);
        } else if line.is_empty() {
            headers_finished = true;
        } else if line.starts_with("Connection:") {
            keep_alive = line.contains("keep-alive");
        }
    }

    (method, url, body, true, keep_alive)
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
                "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nConnection: keep-alive\r\nContent-Length: {}\r\n\r\n",
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

fn handle_post_for_store_user_data(stream: &mut TcpStream, _url: String, body: String) {
    match serde_json::from_str::<UserInfo>(&body) {
        Ok(data) => match data.insert_user_info() {
            Ok(_) => {
                let response = Response {
                    message: "Your data is saved successfully".to_string(),
                };
                let json_message = serde_json::to_string(&response).unwrap();
                let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: keep-alive\r\nContent-Type: application/json\r\n\r\n{}",
                        json_message.len(),
                        json_message
                    );
                stream.write_all(response.as_bytes()).unwrap();
            }
            Err(_) => {
                let response = Response {
                    message: "The email is already registered. Try a new email.".to_string(),
                };
                let json_message = serde_json::to_string(&response).unwrap();
                let response = format!(
                        "HTTP/1.1 409 Conflict\r\nContent-Length: {}\r\nConnection: keep-alive\r\nContent-Type: application/json\r\n\r\n{}",
                        json_message.len(),
                        json_message
                    );
                stream.write_all(response.as_bytes()).unwrap();
            }
        },
        Err(_) => {
            let response = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
            stream.write_all(response.as_bytes()).unwrap();
        }
    }
}

fn map_url_to_file(base_dir: &str, url_path: &str) -> Option<PathBuf> {
    let mut safe_path = url_path.trim_start_matches('/');
    if safe_path.is_empty() {
        safe_path = "index.html";
    }
    let full_path = Path::new(base_dir).join(safe_path);

    if full_path.is_file() {
        Some(full_path)
    } else {
        None
    }
}
