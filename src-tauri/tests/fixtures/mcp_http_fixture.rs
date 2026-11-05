use std::io::{Read, Write};
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:9000").expect("bind MCP HTTP fixture");
    println!("MCP HTTP fixture listening on http://127.0.0.1:9000/mcp");
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        let mut raw = Vec::new();
        let mut buffer = [0u8; 4096];
        let mut header_end = 0;
        let mut content_length = 0;
        for _ in 0..64 {
            let Ok(read) = stream.read(&mut buffer) else { break };
            if read == 0 { break; }
            raw.extend_from_slice(&buffer[..read]);
            if let Some(position) = raw.windows(4).position(|window| window == b"\r\n\r\n") {
                header_end = position + 4;
                let head = String::from_utf8_lossy(&raw[..position]);
                content_length = head.lines().find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length").then(|| value.trim().parse().ok()).flatten()
                }).unwrap_or(0);
                if raw.len() >= header_end + content_length { break; }
            }
        }
        let head = String::from_utf8_lossy(&raw[..header_end]);
        let body = String::from_utf8_lossy(&raw[header_end..]);
        let response = if head.starts_with("OPTIONS") {
            "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: Content-Type, Accept, Authorization, Mcp-Session-Id, MCP-Protocol-Version\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string()
        } else if !head.to_ascii_lowercase().contains("authorization: bearer secret") {
            "HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nContent-Length: 38\r\nConnection: close\r\n\r\n{\"error\":{\"message\":\"bad credential\"}}".to_string()
        } else if body.contains("notifications/initialized") {
            "HTTP/1.1 202 Accepted\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string()
        } else {
            let (id, result) = if body.contains("\"method\":\"initialize\"") {
                (1, "{\"protocolVersion\":\"2025-03-26\",\"capabilities\":{\"tools\":{}},\"serverInfo\":{\"name\":\"gui-http\",\"version\":\"1\"}}")
            } else if body.contains("\"method\":\"tools/list\"") {
                (2, "{\"tools\":[{\"name\":\"read_file\",\"description\":\"native remote GUI\",\"inputSchema\":{\"type\":\"object\"}}]}")
            } else if body.contains("\"method\":\"tools/call\"") {
                (3, "{\"content\":[{\"type\":\"text\",\"text\":\"native remote result\"}],\"isError\":false}")
            } else {
                (4, "{\"code\":-32601,\"message\":\"unknown\"}")
            };
            let payload = if id == 4 {
                format!("{{\"jsonrpc\":\"2.0\",\"id\":{id},\"error\":{result}}}")
            } else {
                format!("{{\"jsonrpc\":\"2.0\",\"id\":{id},\"result\":{result}}}")
            };
            format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nMcp-Session-Id: gui-session\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", payload.len(), payload)
        };
        stream.write_all(response.as_bytes()).expect("write MCP response");
    }
}
