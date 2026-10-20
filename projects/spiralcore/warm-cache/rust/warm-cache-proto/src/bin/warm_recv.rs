//! warm_recv: receive a cold seed then a warm plan over TCP.
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;

use warm_cache_proto::wire::WarmWire;
use warm_cache_proto::*;

fn read_u32(s: &mut std::net::TcpStream) -> std::io::Result<u32> {
    let mut b = [0u8; 4]; s.read_exact(&mut b)?; Ok(u32::from_le_bytes(b))
}
fn read_bytes(s: &mut std::net::TcpStream) -> std::io::Result<Vec<u8>> {
    let n = read_u32(s)? as usize; let mut b = vec![0u8; n]; s.read_exact(&mut b)?; Ok(b)
}
fn write_u32(s: &mut std::net::TcpStream, n: u32) -> std::io::Result<()> { s.write_all(&n.to_le_bytes()) }

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 { eprintln!("usage: warm_recv <bind> <out>"); std::process::exit(2); }
    let listener = TcpListener::bind(&args[1]).unwrap();
    let (mut s, _) = listener.accept().unwrap();
    let seed_bytes = read_bytes(&mut s).unwrap();
    let seed = Seed::new(seed_bytes, 1);
    s.write_all(b"K").unwrap();
    let wire_bytes = read_bytes(&mut s).unwrap();
    let wire = WarmWire::decode(&wire_bytes).unwrap();
    let frame = wire.to_frame().unwrap();
    let mut cell = KuramotoCell::new(71, PHASE_BITS);
    for _ in 0..2000 { cell.step(0.001); }
    let header = TimeHeader::from_snapshot(0, 0, 0, &cell.snapshot());
    let mut session = WarmSession::new(seed.clone(), header);
    let result = session.decode(&wire_bytes).unwrap();
    if result == WarmResult::Committed {
        let payload = frame.plan.apply(&seed.bytes);
        fs::write(&args[2], &payload).unwrap();
        s.write_all(b"K").unwrap();
        println!("OK payload={} warm_bytes={} payload_bytes={} repairs={}",
            payload.len(), session.stats.warm_bytes, session.stats.payload_bytes, session.stats.repairs);
    } else {
        s.write_all(b"E").unwrap();
        println!("MISMATCH {:?}", result);
    }
}
