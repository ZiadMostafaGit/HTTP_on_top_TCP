use std::fs;
use std::path::{Path, PathBuf};

use if_addrs::get_if_addrs;
use std::io::Read;
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
    let _get = "GET";
    let _post = "POST";

    let (method, url, is_http) = is_http(stream);

    if is_http == false {
        println!("the server said its not http request so it will _get droped ");
    } else {
        match method {
            _get => {
                handle_get(stream, url);
            }
            _post => {
                handle_post(stream, url);
            }
            _ => {
                println!("Unsupported HTTP method: {}", method);
            }
        }
    }
}

fn is_http(stream: &mut TcpStream) -> (String, String, bool) {
    let mut buffer = [0; 512]; // Create a buffer to store the incoming data
    let mut request = String::new();

    // Read data from the stream
    match stream.read(&mut buffer) {
        Ok(size) => {
            request.push_str(&String::from_utf8_lossy(&buffer[..size]));
        }
        Err(_) => {
            return ("".to_string(), "".to_string(), false);
        }
    }

    // Step 1: Ensure it's a valid HTTP request (check the first line)
    let mut lines = request.lines();
    let first_line = match lines.next() {
        Some(line) => line,
        None => return ("".to_string(), "".to_string(), false),
    };

    // Step 2: Check if the first line matches the expected HTTP format
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 3 {
        return ("".to_string(), "".to_string(), false);
    }

    let method = parts[0].to_string();
    let url = parts[1].to_string();
    let version = parts[2].to_string();

    // Step 3: Check if it's a valid HTTP version (e.g., HTTP/1.1)
    if !version.starts_with("HTTP/") {
        return ("".to_string(), "".to_string(), false);
    }
    let mut headers_finished = false;
    let mut body = String::new();

    for line in lines {
        if headers_finished {
            // After the headers section, accumulate the body
            body.push_str(line);
            body.push('\n'); // Preserve line breaks
        } else if line.is_empty() {
            // An empty line indicates the end of the headers section
            headers_finished = true;
        }
    }

    (method, url, true)
}

fn handle_get(stream: &mut TcpStream, url: String) {
    let base_bath = "~/git/HTTP_on_top_TCP/src/";
    let new_url = url.as_str();
    let path_for_resource = map_url_to_file(&base_bath, &new_url);
}
fn handle_post(stream: &mut TcpStream, url: String) {
    //handle the data clint should be sent it and maybe return to the html to show that the form
    //submeted sucssisfully
}

/// Maps a URL to a file path on the server
fn map_url_to_file(base_dir: &str, url_path: &str) -> Option<PathBuf> {
    // Sanitize the path to avoid directory traversal attacks
    let safe_path = url_path.trim_start_matches('/');
    let full_path = Path::new(base_dir).join(safe_path);

    // Check if the file exists and is a file (not a directory)
    if full_path.is_file() {
        Some(full_path)
    } else {
        None
    }
}
