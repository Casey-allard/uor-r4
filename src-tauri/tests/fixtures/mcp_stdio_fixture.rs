use std::io::{BufRead, Write};

fn response_id(line: &str) -> u64 {
    let Some(start) = line.find("\"id\":") else { return 0 };
    line[start + 5..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .unwrap_or(0)
}

fn write_frame(stdout: &mut impl Write, frame: &str, initialize: bool) {
    let bytes = frame.as_bytes();
    let split = bytes.len().min(7);
    stdout.write_all(&bytes[..split]).expect("partial frame");
    stdout.flush().expect("flush partial frame");
    std::thread::sleep(std::time::Duration::from_millis(5));
    stdout.write_all(&bytes[split..]).expect("remaining frame");
    stdout.write_all(b"\n").expect("frame newline");
    if initialize {
        stdout.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"server/ready\"}\n").expect("coalesced notification");
    }
    stdout.flush().expect("flush frame");
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if let Some(path) = args.iter().find_map(|arg| arg.strip_prefix("--pid-file=")) {
        std::fs::write(path, std::process::id().to_string()).expect("write fixture pid");
    }
    let fail_list = args.iter().any(|arg| arg == "--fail-list");
    let exit_with_open_pipes = args.iter().any(|arg| arg == "--exit-with-open-pipes");
    for _ in 0..2048 {
        eprintln!("mcp fixture stderr drain check");
    }
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line.expect("read request");
        if line.contains("notifications/initialized") {
            continue;
        }
        let id = response_id(&line);
        if exit_with_open_pipes && line.contains("\"method\":\"initialize\"") {
            std::process::Command::new("/bin/sh")
                .args(["-c", "trap '' HUP TERM; exec sleep 30"])
                .spawn()
                .expect("spawn pipe-holding descendant");
            std::thread::sleep(std::time::Duration::from_millis(10));
            std::process::exit(0);
        }
        if line.contains("\"method\":\"exit\"") {
            std::process::exit(0);
        }
        if line.contains("\"method\":\"tools/call\"") && line.contains("hang") {
            std::thread::sleep(std::time::Duration::from_secs(60));
            continue;
        }
        if fail_list && line.contains("\"method\":\"tools/list\"") {
            let frame = format!("{{\"jsonrpc\":\"2.0\",\"id\":{id},\"error\":{{\"message\":\"listing failed\"}}}}");
            write_frame(&mut stdout, &frame, false);
            continue;
        }
        let result = if line.contains("\"method\":\"initialize\"") {
            "{\"protocolVersion\":\"2025-03-26\",\"capabilities\":{\"tools\":{}},\"serverInfo\":{\"name\":\"stdio-fixture\",\"version\":\"1\"}}".to_string()
        } else if line.contains("\"method\":\"tools/list\"") {
            "{\"tools\":[{\"name\":\"read_file\",\"description\":\"partial/coalesced\",\"inputSchema\":{\"type\":\"object\"}}]}".to_string()
        } else if line.contains("\"method\":\"tools/call\"") {
            "{\"content\":[{\"type\":\"text\",\"text\":\"stdio result\"}],\"isError\":false}".to_string()
        } else {
            continue;
        };
        let frame = format!("{{\"jsonrpc\":\"2.0\",\"id\":{id},\"result\":{result}}}");
        write_frame(&mut stdout, &frame, line.contains("\"method\":\"initialize\""));
    }
}
