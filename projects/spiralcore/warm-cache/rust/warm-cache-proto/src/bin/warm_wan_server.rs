//! Persistent TCP WAN receiver: one cold seed, then many warm frames.
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;

use warm_cache_proto::wire::WarmWire;
use warm_cache_proto::*;

fn read_u32(s: &mut std::net::TcpStream) -> std::io::Result<u32> { let mut b=[0u8;4]; s.read_exact(&mut b)?; Ok(u32::from_le_bytes(b)) }
fn read_bytes(s: &mut std::net::TcpStream) -> std::io::Result<Vec<u8>> { let n=read_u32(s)? as usize; let mut b=vec![0u8;n]; s.read_exact(&mut b)?; Ok(b) }
fn write_u32(s: &mut std::net::TcpStream, n: u32) -> std::io::Result<()> { s.write_all(&n.to_le_bytes()) }

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 { eprintln!("usage: warm_wan_server <bind> <out> <count>"); std::process::exit(2); }
    let count: usize = args[3].parse().unwrap();
    let listener = TcpListener::bind(&args[1]).unwrap();
    let (mut s, _) = listener.accept().unwrap();
    let seed = Seed::new(read_bytes(&mut s).unwrap(), 1);
    s.write_all(b"K").unwrap();
    for _ in 0..count {
        let wire_bytes = read_bytes(&mut s).unwrap();
        let wire = WarmWire::decode(&wire_bytes).unwrap();
        let frame = wire.to_frame().unwrap();
        let result = receive_warm(&seed, &frame.header, &frame);
        if result == WarmResult::Committed {
            let payload = frame.plan.apply(&seed.bytes);
            fs::write(&args[2], &payload).unwrap();
        }
        s.write_all(&[if result == WarmResult::Committed { b'K' } else { b'E' }]).unwrap();
    }
    println!("done count={}", count);
}
