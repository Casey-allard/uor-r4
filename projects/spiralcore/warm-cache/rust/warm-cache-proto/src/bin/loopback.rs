//! Multi-size TCP loopback for the warm-cache prototype.
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Instant;
use warm_cache_proto::wire::WarmWire;
use warm_cache_proto::*;

fn read_u32(s: &mut TcpStream) -> std::io::Result<u32> {
    let mut b = [0u8; 4];
    s.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}
fn write_u32(s: &mut TcpStream, n: u32) -> std::io::Result<()> {
    s.write_all(&n.to_le_bytes())
}
fn read_bytes(s: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    let n = read_u32(s)? as usize;
    let mut b = vec![0u8; n]; s.read_exact(&mut b)?; Ok(b)
}
fn write_bytes(s: &mut TcpStream, b: &[u8]) -> std::io::Result<()> {
    write_u32(s, b.len() as u32)?; s.write_all(b)
}

fn main() {
    let sizes = [4usize * 1024, 64 * 1024, 1 << 20, 4 << 20];
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        for _ in &sizes {
            let seed_bytes = read_bytes(&mut stream).unwrap();
            let seed = Seed::new(seed_bytes, 1);
            let cold_ack = seed.id[0];
            stream.write_all(&[cold_ack]).unwrap();
            // two warm frames: identity, then one-page repair
            for _kind in 0..2 {
                let wire_bytes = read_bytes(&mut stream).unwrap();
                let wire = WarmWire::decode(&wire_bytes).unwrap();
                let frame = wire.to_frame().unwrap();
                let res = receive_warm(&seed, &frame.header, &frame);
                stream.write_all(&[if res == WarmResult::Committed { b'K' } else { b'E' }]).unwrap();
            }
        }
    });

    let mut stream = TcpStream::connect(addr).unwrap();
    let mut cell = KuramotoCell::new(71, PHASE_BITS);
    for _ in 0..2000 { cell.step(0.001); }
    let h = TimeHeader::from_snapshot(0, 0, 0, &cell.snapshot());
    println!("size,cold_bytes,warm_identity_bytes,warm_payload,repair_bytes,repair_payload,elapsed_ms");
    for &size in &sizes {
        let bytes: Vec<u8> = (0..size).map(|i| ((i * 17 + 3) & 0xFF) as u8).collect();
        let seed = Seed::new(bytes, 1);
        let identity = make_warm(&seed, Plan::Identity, h.clone());
        let mut target_page = seed.bytes[0..PAGE].to_vec();
        target_page[10] ^= 0xAA;
        let repair = make_warm(&seed, Plan::PageRepair(vec![(0u32, target_page)]), h.clone());
        let wi = WarmWire::from_frame(&identity);
        let wr = WarmWire::from_frame(&repair);
        let t0 = Instant::now();
        write_bytes(&mut stream, &seed.bytes).unwrap();
        let mut ack = [0u8; 1]; stream.read_exact(&mut ack).unwrap();
        assert_eq!(ack[0], seed.id[0]);
        write_bytes(&mut stream, &wi.encode()).unwrap();
        stream.read_exact(&mut ack).unwrap(); assert_eq!(ack[0], b'K');
        write_bytes(&mut stream, &wr.encode()).unwrap();
        stream.read_exact(&mut ack).unwrap(); assert_eq!(ack[0], b'K');
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        let cold = 4 + seed.bytes.len() as u64;
        let warm = 4 + wi.wire_len();
        let rep = 4 + wr.wire_len();
        println!("{},{},{},{},{},{},{:.3}", size, cold, warm, wi.payload.len(), rep, wr.payload.len(), ms);
    }
    server.join().unwrap();
}
