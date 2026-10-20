//! warm_send: send a cold seed then a warm plan over TCP.
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;

use warm_cache_proto::*;

fn write_bytes(s: &mut TcpStream, b: &[u8]) -> std::io::Result<()> {
    s.write_all(&(b.len() as u32).to_le_bytes())?; s.write_all(b)
}

fn plan_for(seed: &[u8], payload: &[u8]) -> Plan {
    if seed == payload { return Plan::Identity; }
    let mut pages = Vec::new();
    for page_id in 0..((payload.len() + PAGE - 1) / PAGE) {
        let start = page_id * PAGE;
        let end = (start + PAGE).min(payload.len());
        let a = &seed[start.min(seed.len())..end.min(seed.len())];
        let b = &payload[start..end];
        if a != b {
            pages.push((page_id as u32, b.to_vec()));
        }
    }
    Plan::PageRepair(pages)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 { eprintln!("usage: warm_send <connect> <seed> <payload>"); std::process::exit(2); }
    let seed = fs::read(&args[2]).unwrap();
    let payload = fs::read(&args[3]).unwrap();
    let mut s = TcpStream::connect(&args[1]).unwrap();
    write_bytes(&mut s, &seed).unwrap();
    let mut ack = [0u8; 1]; s.read_exact(&mut ack).unwrap(); assert_eq!(ack[0], b'K');
    let seed = Seed::new(seed, 1);
    let mut cell = KuramotoCell::new(71, PHASE_BITS);
    for _ in 0..2000 { cell.step(0.001); }
    let header = TimeHeader::from_snapshot(0, 0, 0, &cell.snapshot());
    let mut session = WarmSession::new(seed, header);
    let plan = plan_for(&session.seed.bytes, &payload);
    let wire = session.encode_bytes(plan);
    write_bytes(&mut s, &wire).unwrap();
    let mut ack = [0u8; 1]; s.read_exact(&mut ack).unwrap();
    println!("sent warm_bytes={} payload_bytes={} repairs={} ack={}",
        session.stats.warm_bytes, session.stats.payload_bytes, session.stats.repairs, ack[0] as char);
    assert_eq!(ack[0], b'K');
}
