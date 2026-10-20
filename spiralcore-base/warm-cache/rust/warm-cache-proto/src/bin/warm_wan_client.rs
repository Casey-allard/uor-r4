//! Persistent TCP WAN client: one cold seed, then repeated warm sends.
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Instant;

use warm_cache_proto::*;

fn read_u32(s: &mut TcpStream) -> std::io::Result<u32> { let mut b=[0u8;4]; s.read_exact(&mut b)?; Ok(u32::from_le_bytes(b)) }
fn read_bytes(s: &mut TcpStream) -> std::io::Result<Vec<u8>> { let n=read_u32(s)? as usize; let mut b=vec![0u8;n]; s.read_exact(&mut b)?; Ok(b) }
fn write_bytes(s: &mut TcpStream, b: &[u8]) -> std::io::Result<()> { s.write_all(&(b.len() as u32).to_le_bytes())?; s.write_all(b) }

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 { eprintln!("usage: warm_wan_client <addr> <seed> <payload> <iters>"); std::process::exit(2); }
    let iters: usize = args[4].parse().unwrap();
    let seed_bytes = fs::read(&args[2]).unwrap();
    let payload = fs::read(&args[3]).unwrap();
    let mut s = TcpStream::connect(&args[1]).unwrap();
    write_bytes(&mut s, &seed_bytes).unwrap();
    let mut ack=[0u8;1]; s.read_exact(&mut ack).unwrap(); assert_eq!(ack[0],b'K');
    let seed = Seed::new(seed_bytes,1);
    let mut cell=KuramotoCell::new(71,PHASE_BITS);
    for _ in 0..2000 { cell.step(0.001); }
    let header=TimeHeader::from_snapshot(0,0,0,&cell.snapshot());
    let mut session=WarmSession::new(seed,header);
    let mut pages=Vec::new();
    for page_id in 0..((payload.len()+PAGE-1)/PAGE) {
        let start=page_id*PAGE; let end=(start+PAGE).min(payload.len());
        if &session.seed.bytes[start..end] != &payload[start..end] {
            pages.push((page_id as u32, payload[start..end].to_vec()));
        }
    }
    let plan = if pages.is_empty() { Plan::Identity } else { Plan::PageRepair(pages) };
    let wire=session.encode(plan);
    let bytes=wire.encode();
    println!("size,warm_ms,wire_bytes,payload_bytes,repairs");
    for _ in 0..iters {
        let t0=Instant::now();
        write_bytes(&mut s,&bytes).unwrap();
        s.read_exact(&mut ack).unwrap();
        let ms=t0.elapsed().as_secs_f64()*1000.0;
        assert_eq!(ack[0],b'K');
        println!("{},{:.1},{},{},{}",payload.len(),ms,bytes.len(),wire.payload.len(),if wire.plan_kind==2 {1} else {0});
    }
}
