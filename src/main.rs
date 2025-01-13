use std::fs;
use std::path::{Path, PathBuf};

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

    let (method, url, body, is_http) = is_http(stream);

    if is_http == false {
        println!("the server said its not http request so it will get droped ");
    } else {
        match method {
            get => {
                handle_get(stream, url);
            }
            post => {
                handle_post(stream, url, body);
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
    let base_bath = "/home/ziad/git/HTTP_on_top_TCP/src/";
    let new_url = url.as_str();
    let path_for_resource = map_url_to_file(&base_bath, &new_url);

    if path_for_resource.is_none() {
        let response = "HTTP/1.1 200 Ok\r\nContent-Type: {}\r\nContent-Length:0\r\n\r\n";
        stream.write_all(response.as_bytes()).unwrap();
    } else {
        let path = path_for_resource.unwrap();
        let content = fs::read_to_string(&path).unwrap();
        let content_type = if path.ends_with(".html") {
            "text/html"
        } else if path.ends_with(".css") {
            "text/css"
        } else if path.ends_with(".js") {
            "application/javascript"
        } else if path.ends_with(".png") {
            "image/png"
        } else if path.ends_with(".jpg") || path.ends_with("jpeg") {
            "image/jpg"
        } else {
            "text/plain"
        };
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {} \r\nContent-Length: {} \r\n\r\n{}",
            content_type,
            content.len(),
            content
        );
        stream.write_all(response.as_bytes()).unwrap();
    }
}
fn handle_post(stream: &mut TcpStream, url: String, body: String) {}

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
