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
    let get = "Get";
    let post = "Post";

    let (method, is_http) = is_http(&stream);

    if is_http == false {
        println!("the server said its not http request so it will get droped ");
    } else {
        match method {
            get => {
                handle_get(stream);
            }
            post => {
                handle_post(stream);
            }
            _ => {
                println!("Unsupported HTTP method: {}", method);
            }
        }
    }
}

fn is_http(stream: &TcpStream) -> (String, bool) {
    let mut buffer = [0; 512]; // Create a buffer to store the incoming data
    let mut request = String::new();

    // Read data from the stream
    match stream.read(&mut buffer) {
        Ok(size) => {
            request.push_str(&String::from_utf8_lossy(&buffer[..size]));
        }
        Err(_) => {
            return ("".to_string(), false); // Return empty if there's an error
        }
    }

    // Step 1: Ensure it's a valid HTTP request (check the first line)
    let mut lines = request.lines();
    let first_line = match lines.next() {
        Some(line) => line,
        None => return ("".to_string(), false), // If no first line, it's not HTTP
    };

    // Step 2: Check if the first line matches the expected HTTP format
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 3 {
        return ("".to_string(), false); // Invalid first line (method, URL, version expected)
    }

    let method = parts[0].to_string();
    let url = parts[1].to_string();
    let version = parts[2].to_string();

    // Step 3: Check if it's a valid HTTP version (e.g., HTTP/1.1)
    if !version.starts_with("HTTP/") {
        return ("".to_string(), false); // Not an HTTP request if version is incorrect
    }

    // Step 4: Skip headers (we can ignore them for now)
    // Continue until we find an empty line (headers are separated from body by an empty line)
    for line in lines {
        if line.is_empty() {
            break; // Found the end of the headers section
        }
    }

    // Step 5: If there's any data after the headers, it's the body
    let body: String = lines.collect::<Vec<&str>>().join("\n");
    (method, true)
}

fn handle_get(stream: &mut TcpStream) {
    //send the index.html to clint
}
fn handle_post(stream: &mut TcpStream) {
    //handle the data clint should be sent it and maybe return to the html to show that the form
    //submeted sucssisfully
}
