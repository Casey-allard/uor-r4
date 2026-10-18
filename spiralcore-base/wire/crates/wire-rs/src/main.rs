use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use std::collections::HashMap;
use std::env;
use std::io::{self, Read, Write};

#[cfg(feature = "backend-zlib-rs")]
const BACKEND: &str = "zlib-rs";
#[cfg(not(feature = "backend-zlib-rs"))]
const BACKEND: &str = "miniz_oxide";
use std::net::{TcpListener, TcpStream};
use std::process::exit;
use std::sync::Arc;
use std::time::Instant;
use wire_rs::mux::{MuxClient, MuxGateway};
use wire_rs::{WireListener, WireSocket};

fn usage() -> ! {
    eprintln!(
        "usage: wire-rs echo-ping --connect ADDR [--route NAME] [--bytes N] [--reps K] [--plain] | echo-server --bind ADDR [--plain] | echo-client --connect ADDR [--route NAME] [--bytes N] [--pattern text|random] [--plain] | hub --listen ADDR --route NAME=HOST:PORT | edge --listen ADDR --remote GATEWAY [--route NAME] | gateway --listen ADDR --target HOST:PORT | bench-compress [--bytes N|--file PATH] [--pattern text|random] [--reps K]"
    );
    exit(2)
}

fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    let mut index = 0;
    while index < args.len() {
        if args[index] == name && index + 1 < args.len() {
            return Some(&args[index + 1]);
        }
        index += 1;
    }
    None
}

fn payload(pattern: &str, size: usize) -> Vec<u8> {
    let mut data = vec![0u8; size];
    if pattern == "random" {
        let mut state: u64 = 0x20261017;
        for byte in data.iter_mut() {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            *byte = (state >> 33) as u8;
        }
    } else {
        let line = b"{\"kind\":\"rust-client\",\"payload\":\"abcdef0123456789\"}\n";
        for (index, byte) in data.iter_mut().enumerate() {
            *byte = line[index % line.len()];
        }
    }
    data
}

fn echo_server(args: &[String]) -> io::Result<()> {
    let bind = flag(args, "--bind").unwrap_or_else(|| usage());
    if args.iter().any(|arg| arg == "--plain") {
        return plain_server(bind);
    }
    let listener = WireListener::bind(bind)?;
    let addr = listener.local_addr()?;
    println!("READY {}", addr);
    io::stdout().flush().ok();
    let (mut socket, route) = listener.accept()?;
    let mut total = 0usize;
    let mut buffer = vec![0u8; 262144];
    loop {
        let read = socket.recv_into(&mut buffer)?;
        if read == 0 {
            break;
        }
        socket.send_all(&buffer[..read])?;
        total += read;
    }
    socket.shutdown()?;
    let stats = socket.stats();
    println!(
        "RUST_SERVER_OK route={} bytes={} ratio={:.4}",
        route,
        total,
        stats.ratio()
    );
    Ok(())
}

fn plain_server(bind: &str) -> io::Result<()> {
    let listener = TcpListener::bind(bind)?;
    println!("READY {}", listener.local_addr()?);
    io::stdout().flush().ok();
    let (mut stream, _) = listener.accept()?;
    stream.set_nodelay(true)?;
    let mut total = 0usize;
    let mut buffer = vec![0u8; 262144];
    loop {
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        stream.write_all(&buffer[..read])?;
        total += read;
    }
    stream.shutdown(std::net::Shutdown::Write).ok();
    println!("PLAIN_SERVER_OK bytes={}", total);
    Ok(())
}

fn echo_client(args: &[String]) -> io::Result<()> {
    let connect = flag(args, "--connect").unwrap_or_else(|| usage());
    let route = flag(args, "--route").unwrap_or("");
    let size: usize = flag(args, "--bytes")
        .unwrap_or("262144")
        .parse()
        .unwrap_or(262144);
    let pattern = flag(args, "--pattern").unwrap_or("text");
    if args.iter().any(|arg| arg == "--plain") {
        return plain_client(connect, pattern, size);
    }
    let data = payload(pattern, size);
    let mut socket = WireSocket::connect(connect, route)?;
    let started = Instant::now();
    socket.send_all(&data)?;
    socket.shutdown()?;
    let received = socket.recv_all()?;
    let seconds = started.elapsed().as_secs_f64();
    let stats = socket.stats();
    if received != data {
        println!(
            "RUST_CLIENT_FAIL bytes={} received={}",
            data.len(),
            received.len()
        );
        exit(1);
    }
    println!(
        "RUST_CLIENT_OK bytes={} seconds={:.4} mibs={:.1} ratio={:.4}",
        data.len(),
        seconds,
        data.len() as f64 / seconds / (1024.0 * 1024.0),
        stats.ratio()
    );
    Ok(())
}

fn serve_connection(
    mut socket: WireSocket,
    route: String,
    routes: Arc<HashMap<String, String>>,
) -> io::Result<()> {
    let target_addr = match routes.get(&route) {
        Some(value) => value.clone(),
        None => {
            let _ = socket.shutdown();
            return Ok(());
        }
    };
    let target = TcpStream::connect(&target_addr)?;
    let (mut reader, mut writer) = wire_rs::split(socket)?;
    let mut to_target = target.try_clone()?;
    let reader_thread = std::thread::spawn(move || {
        let mut buffer = vec![0u8; 262144];
        loop {
            match reader.recv_into(&mut buffer) {
                Ok(0) => {
                    let _ = to_target.shutdown(std::net::Shutdown::Write);
                    break;
                }
                Ok(read) => {
                    if to_target.write_all(&buffer[..read]).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    let mut from_target = target.try_clone()?;
    let writer_thread = std::thread::spawn(move || {
        let mut buffer = vec![0u8; 262144];
        loop {
            match from_target.read(&mut buffer) {
                Ok(0) => {
                    let _ = writer.shutdown();
                    break;
                }
                Ok(size) => {
                    if writer.send_all(&buffer[..size]).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    reader_thread.join().ok();
    writer_thread.join().ok();
    Ok(())
}

fn hub(args: &[String]) -> io::Result<()> {
    let bind = flag(args, "--listen").unwrap_or_else(|| usage());
    let mut routes = HashMap::new();
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--route" && index + 1 < args.len() {
            if let Some((name, target)) = args[index + 1].split_once('=') {
                routes.insert(name.to_string(), target.to_string());
            }
            index += 2;
        } else {
            index += 1;
        }
    }
    if routes.is_empty() {
        eprintln!("hub needs at least one --route NAME=HOST:PORT");
        exit(2);
    }
    let listener = WireListener::bind(bind)?;
    println!("READY {}", listener.local_addr()?);
    io::stdout().flush().ok();
    let routes = Arc::new(routes);
    loop {
        let (socket, route) = match listener.accept() {
            Ok(value) => value,
            Err(error) => {
                eprintln!("accept error: {}", error);
                continue;
            }
        };
        let routes = Arc::clone(&routes);
        std::thread::spawn(move || {
            if let Err(error) = serve_connection(socket, route, routes) {
                eprintln!("connection error: {}", error);
            }
        });
    }
}

fn plain_client(connect: &str, pattern: &str, size: usize) -> io::Result<()> {
    let data = payload(pattern, size);
    let mut stream = TcpStream::connect(connect)?;
    stream.set_nodelay(true)?;
    let started = Instant::now();
    stream.write_all(&data)?;
    stream.shutdown(std::net::Shutdown::Write)?;
    let mut received = Vec::with_capacity(data.len());
    stream.read_to_end(&mut received)?;
    let seconds = started.elapsed().as_secs_f64();
    if received != data {
        println!(
            "PLAIN_CLIENT_FAIL bytes={} received={}",
            data.len(),
            received.len()
        );
        exit(1);
    }
    println!(
        "PLAIN_CLIENT_OK bytes={} seconds={:.5} mibs={:.1}",
        data.len(),
        seconds,
        data.len() as f64 / seconds / (1024.0 * 1024.0)
    );
    Ok(())
}

fn compress_once(data: &[u8], level: u32) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(level));
    encoder.write_all(data).expect("compress write");
    encoder.finish().expect("compress finish")
}

fn decompress_once(data: &[u8]) -> Vec<u8> {
    let mut decoder = ZlibDecoder::new(data);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out).expect("decompress");
    out
}

fn bench_compress(args: &[String]) -> io::Result<()> {
    let size: usize = flag(args, "--bytes")
        .unwrap_or("1310720")
        .parse()
        .unwrap_or(1310720);
    let pattern = flag(args, "--pattern").unwrap_or("text");
    let reps: usize = flag(args, "--reps").unwrap_or("10").parse().unwrap_or(10);
    let level: u32 = flag(args, "--level").unwrap_or("6").parse().unwrap_or(6);
    let data = match flag(args, "--file") {
        Some(path) => std::fs::read(path)?,
        None => payload(pattern, size),
    };
    let size = data.len();
    let _ = compress_once(&data, level); // warm up allocator and backend tables
    let started = Instant::now();
    let mut wire = 0usize;
    for _ in 0..reps {
        wire += compress_once(&data, level).len();
    }
    let compress_seconds = started.elapsed().as_secs_f64();
    let compressed = compress_once(&data, level);
    let started = Instant::now();
    for _ in 0..reps {
        let out = decompress_once(&compressed);
        assert_eq!(out.len(), data.len());
    }
    let decompress_seconds = started.elapsed().as_secs_f64();
    let mebibytes = (size * reps) as f64 / (1024.0 * 1024.0);
    println!(
        "BENCH_COMPRESS_OK backend={} pattern={} bytes={} reps={} level={} compress_mibs={:.1} decompress_mibs={:.1} ratio={:.5}",
        BACKEND,
        pattern,
        size,
        reps,
        level,
        mebibytes / compress_seconds,
        mebibytes / decompress_seconds,
        wire as f64 / (size * reps) as f64
    );
    Ok(())
}

fn relay_plain_wire(plain: TcpStream, socket: WireSocket) -> io::Result<()> {
    let (mut reader, mut writer) = wire_rs::split(socket)?;
    let mut plain_write = plain.try_clone()?;
    let wire_to_plain = std::thread::spawn(move || {
        let mut buffer = vec![0u8; 262144];
        loop {
            match reader.recv_into(&mut buffer) {
                Ok(0) => {
                    let _ = plain_write.shutdown(std::net::Shutdown::Write);
                    break;
                }
                Ok(read) => {
                    if plain_write.write_all(&buffer[..read]).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    let mut plain_read = plain.try_clone()?;
    let plain_to_wire = std::thread::spawn(move || {
        let mut buffer = vec![0u8; 262144];
        loop {
            match plain_read.read(&mut buffer) {
                Ok(0) => {
                    let _ = writer.shutdown();
                    break;
                }
                Ok(size) => {
                    if writer.send_all(&buffer[..size]).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    wire_to_plain.join().ok();
    plain_to_wire.join().ok();
    Ok(())
}

fn adapter_edge(args: &[String]) -> io::Result<()> {
    let listen = flag(args, "--listen").unwrap_or_else(|| usage());
    let remote = flag(args, "--remote").unwrap_or_else(|| usage());
    let route = flag(args, "--route").unwrap_or("").to_string();
    let listener = TcpListener::bind(listen)?;
    println!("READY {}", listener.local_addr()?);
    io::stdout().flush().ok();
    for incoming in listener.incoming() {
        let stream = match incoming {
            Ok(stream) => stream,
            Err(error) => {
                eprintln!("accept error: {}", error);
                continue;
            }
        };
        let remote = remote.to_string();
        let route = route.clone();
        std::thread::spawn(move || {
            stream.set_nodelay(true).ok();
            match WireSocket::connect(&remote, &route) {
                Ok(socket) => {
                    if let Err(error) = relay_plain_wire(stream, socket) {
                        eprintln!("relay error: {}", error);
                    }
                }
                Err(error) => eprintln!("wire connect error: {}", error),
            }
        });
    }
    Ok(())
}

fn adapter_gateway(args: &[String]) -> io::Result<()> {
    let listen = flag(args, "--listen").unwrap_or_else(|| usage());
    let target = flag(args, "--target")
        .unwrap_or_else(|| usage())
        .to_string();
    let listener = WireListener::bind(listen)?;
    println!("READY {}", listener.local_addr()?);
    io::stdout().flush().ok();
    loop {
        let (socket, route) = match listener.accept() {
            Ok(value) => value,
            Err(error) => {
                eprintln!("wire accept error: {}", error);
                continue;
            }
        };
        let target = target.clone();
        std::thread::spawn(move || match TcpStream::connect(&target) {
            Ok(stream) => {
                stream.set_nodelay(true).ok();
                if let Err(error) = relay_plain_wire(stream, socket) {
                    eprintln!("route {} relay error: {}", route, error);
                }
            }
            Err(error) => eprintln!("target connect error: {}", error),
        });
    }
}

fn mux_gateway(args: &[String]) -> io::Result<()> {
    let listen = flag(args, "--listen").unwrap_or_else(|| usage());
    let mut routes = std::collections::HashMap::new();
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--route" && index + 1 < args.len() {
            if let Some((name, target)) = args[index + 1].split_once('=') {
                routes.insert(name.to_string(), target.to_string());
            }
            index += 2;
        } else {
            index += 1;
        }
    }
    if routes.is_empty() {
        eprintln!("mux-gateway needs at least one --route NAME=HOST:PORT");
        exit(2);
    }
    let gateway = MuxGateway::bind(listen, routes)?;
    println!("READY {}", gateway.local_addr()?);
    io::stdout().flush().ok();
    gateway.serve_forever()
}

fn mux_client(args: &[String]) -> io::Result<()> {
    let connect = flag(args, "--connect").unwrap_or_else(|| usage());
    let route = flag(args, "--route").unwrap_or("").to_string();
    let streams: usize = flag(args, "--streams").unwrap_or("8").parse().unwrap_or(8);
    let size: usize = flag(args, "--bytes")
        .unwrap_or("131072")
        .parse()
        .unwrap_or(131072);
    let pattern = flag(args, "--pattern").unwrap_or("text");
    let payloads: Vec<Vec<u8>> = (0..streams).map(|_| payload(pattern, size)).collect();
    let client = MuxClient::connect(connect, &route)?;
    let started = Instant::now();
    let mut handles = Vec::new();
    for payload in payloads {
        let mut stream = client.open_stream("")?;
        handles.push(std::thread::spawn(move || {
            stream.send_all(&payload)?;
            stream.shutdown()?;
            let mut received = 0usize;
            loop {
                let data = stream.recv(65536)?;
                if data.is_empty() {
                    break;
                }
                received += data.len();
            }
            stream.close();
            Ok::<usize, std::io::Error>(received)
        }));
    }
    let mut total = 0usize;
    for handle in handles {
        let received = handle.join().unwrap()?;
        total += received;
    }
    let seconds = started.elapsed().as_secs_f64();
    let expected = streams * size;
    if total != expected {
        println!("MUX_CLIENT_FAIL bytes={} expected={}", total, expected);
        exit(1);
    }
    println!(
        "MUX_CLIENT_OK streams={} bytes={} seconds={:.5} mibs={:.1}",
        streams,
        total,
        seconds,
        total as f64 / seconds / (1024.0 * 1024.0)
    );
    Ok(())
}

fn echo_ping(args: &[String]) -> io::Result<()> {
    let connect = flag(args, "--connect").unwrap_or_else(|| usage());
    let route = flag(args, "--route").unwrap_or("");
    let size: usize = flag(args, "--bytes")
        .unwrap_or("1024")
        .parse()
        .unwrap_or(1024);
    let reps: usize = flag(args, "--reps").unwrap_or("200").parse().unwrap_or(200);
    let payload = vec![b'x'; size];
    let mut buffer = vec![0u8; size];
    let plain = args.iter().any(|arg| arg == "--plain");
    let started;
    let mut ratio = 0.0;
    if plain {
        let mut stream = TcpStream::connect(connect)?;
        stream.set_nodelay(true)?;
        started = Instant::now();
        for _ in 0..reps {
            stream.write_all(&payload)?;
            stream.read_exact(&mut buffer)?;
        }
    } else {
        let mut socket = WireSocket::connect(connect, route)?;
        started = Instant::now();
        for _ in 0..reps {
            socket.send_all(&payload)?;
            let mut got = 0usize;
            while got < size {
                let read = socket.recv_into(&mut buffer[got..])?;
                if read == 0 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "ping eof",
                    ));
                }
                got += read;
            }
        }
        ratio = socket.stats().ratio();
    }
    let seconds = started.elapsed().as_secs_f64();
    println!(
        "PING_OK mode={} bytes={} reps={} mean_us={:.1} ratio={:.4}",
        if plain { "plain" } else { "wire" },
        size,
        reps,
        seconds / reps as f64 * 1e6,
        ratio
    );
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        usage();
    }
    match args[1].as_str() {
        "echo-server" => echo_server(&args),
        "edge" => adapter_edge(&args),
        "gateway" => adapter_gateway(&args),
        "mux-gateway" => mux_gateway(&args),
        "mux-client" => mux_client(&args),
        "echo-ping" => echo_ping(&args),
        "echo-client" => echo_client(&args),
        "hub" => hub(&args),
        "bench-compress" => bench_compress(&args),
        _ => usage(),
    }
}
