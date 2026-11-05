use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Duration;

fn main() {
    let mut reject_once = cfg!(uor_reject_once);
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind slow sidecar");
    println!("uor-chat OpenAI loopback listening on http://127.0.0.1:{}", listener.local_addr().expect("address").port());
    std::io::stdout().flush().expect("flush listening port");
    for incoming in listener.incoming() {
        let Ok(mut stream) = incoming else { break };
        let mut request = Vec::new();
        let mut buffer = [0u8; 4096];
        while !request.windows(4).any(|window| window == b"\r\n\r\n") {
            match stream.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(read) => request.extend_from_slice(&buffer[..read]),
            }
        }
        let header_end = request.windows(4).position(|window| window == b"\r\n\r\n").map(|position| position + 4).unwrap_or(request.len());
        let content_length = String::from_utf8_lossy(&request[..header_end]).lines().find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length").then(|| value.trim().parse::<usize>().ok()).flatten()
        }).unwrap_or(0);
        while request.len() < header_end + content_length {
            match stream.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(read) => request.extend_from_slice(&buffer[..read]),
            }
        }
        let request = String::from_utf8_lossy(&request);
        let line = request.lines().next().unwrap_or_default();
        std::thread::sleep(Duration::from_millis(250));
        if line.starts_with("GET /v1/models ") {
            let rejected = reject_once;
            reject_once = false;
            let status = if rejected { 401 } else { 200 };
            let body = br#"{"data":[{"id":"uor-chat"}]}"#;
            write!(stream, "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).expect("models headers");
            stream.write_all(body).expect("models body");
        } else if request.contains("\"stream\":true") {
            stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\ndata: par").expect("SSE prefix");
            stream.flush().expect("flush SSE prefix");
            std::thread::sleep(Duration::from_millis(250));
            stream.write_all(b"tial\r\n\r\ndata: [DONE]\r\n\r\n").expect("SSE suffix");
        } else {
            let body = br#"{"choices":[{"message":{"content":"complete response"}}]}"#;
            stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n").expect("JSON headers");
            stream.write_all(&body[..9]).expect("JSON prefix");
            stream.flush().expect("flush JSON prefix");
            std::thread::sleep(Duration::from_millis(250));
            stream.write_all(&body[9..]).expect("JSON suffix");
        }
    }
}
