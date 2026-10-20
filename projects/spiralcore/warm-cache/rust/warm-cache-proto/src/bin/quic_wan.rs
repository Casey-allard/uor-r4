//! QUIC WAN harness: run 'quic_wan server <bind>' and 'quic_wan client <addr>'.
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use quinn::{ClientConfig, Endpoint, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};

use warm_cache_proto::certs;
use warm_cache_proto::wire::WarmWire;
use warm_cache_proto::*;

async fn exchange(conn: &quinn::Connection, kind: u8, data: &[u8]) -> Vec<u8> {
    let (mut send, mut recv) = conn.open_bi().await.unwrap();
    let mut msg = Vec::with_capacity(1 + data.len());
    msg.push(kind);
    msg.extend_from_slice(data);
    send.write_all(&msg).await.unwrap();
    send.finish().unwrap();
    recv.read_to_end(1 << 26).await.unwrap()
}

async fn serve(bind: &str) {
    let cert = certs::cert("server-cert.pem");
    let ca = certs::cert("ca-cert.pem");
    let key = certs::key("server-key.pem");
    let cfg = ServerConfig::with_single_cert(vec![cert, ca], key).unwrap();
    let endpoint = Endpoint::server(cfg, bind.parse::<SocketAddr>().unwrap()).unwrap();
    println!("listening {}", endpoint.local_addr().unwrap());
    while let Some(incoming) = endpoint.accept().await {
        tokio::spawn(async move {
            let conn = incoming.await.unwrap();
            let mut seed: Option<Seed> = None;
            while let Ok((mut send, mut recv)) = conn.accept_bi().await {
                let msg = recv.read_to_end(1 << 26).await.unwrap();
                let ok = if msg.is_empty() { false } else {
                    match msg[0] {
                        0 => { seed = Some(Seed::new(msg[1..].to_vec(), 1)); true }
                        1 => if let Some(s) = &seed {
                            match WarmWire::decode(&msg[1..]) {
                                Ok(w) => match w.to_frame() { Ok(f) => receive_warm(s, &f.header, &f) == WarmResult::Committed, Err(_) => false },
                                Err(_) => false,
                            }
                        } else { false },
                        _ => false,
                    }
                };
                send.write_all(&[if ok { b'K' } else { b'E' }]).await.unwrap();
                send.finish().unwrap();
            }
        });
    }
}

async fn client(addr: &str, sizes: &[usize]) {
    let ca = certs::cert("ca-cert.pem");
    let mut roots = rustls::RootCertStore::empty();
    roots.add(ca).unwrap();
    let crypto = rustls::ClientConfig::builder().with_root_certificates(roots).with_no_client_auth();
    let qcc = quinn::crypto::rustls::QuicClientConfig::try_from(crypto).unwrap();
    let mut endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap()).unwrap();
    endpoint.set_default_client_config(ClientConfig::new(Arc::new(qcc)));
    let conn = endpoint.connect(addr.parse().unwrap(), "localhost").unwrap().await.unwrap();
    let mut cell = KuramotoCell::new(71, PHASE_BITS);
    for _ in 0..2000 { cell.step(0.001); }
    let h = TimeHeader::from_snapshot(0, 0, 0, &cell.snapshot());
    println!("size,cold_ms,warm_ms,repair_ms,warm_bytes,repair_bytes");
    for &size in sizes {
        let bytes: Vec<u8> = (0..size).map(|i| ((i * 17 + 3) & 0xFF) as u8).collect();
        let seed = Seed::new(bytes, 1);
        let wi = WarmWire::from_frame(&make_warm(&seed, Plan::Identity, h.clone()));
        let mut page = seed.bytes[0..PAGE].to_vec(); page[10] ^= 0xAA;
        let wr = WarmWire::from_frame(&make_warm(&seed, Plan::PageRepair(vec![(0, page)]), h.clone()));
        let t0 = Instant::now(); assert_eq!(exchange(&conn, 0, &seed.bytes).await, b"K");
        let cold = t0.elapsed().as_secs_f64() * 1000.0;
        let t1 = Instant::now(); assert_eq!(exchange(&conn, 1, &wi.encode()).await, b"K");
        let warm = t1.elapsed().as_secs_f64() * 1000.0;
        let t2 = Instant::now(); assert_eq!(exchange(&conn, 1, &wr.encode()).await, b"K");
        let repair = t2.elapsed().as_secs_f64() * 1000.0;
        println!("{},{:.1},{:.1},{:.1},{},{}", size, cold, warm, repair, wi.wire_len(), wr.wire_len());
    }
}

#[tokio::main]
async fn main() {
    rustls::crypto::ring::default_provider().install_default().ok();
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 { eprintln!("usage: quic_wan server <bind> | client <addr> [sizes]"); std::process::exit(2); }
    match args[1].as_str() {
        "server" => serve(&args[2]).await,
        "client" => {
            let sizes: Vec<usize> = args[3..].iter().filter_map(|s| s.parse().ok()).collect();
            let sizes = if sizes.is_empty() { vec![4 * 1024, 64 * 1024, 1 << 20] } else { sizes };
            client(&args[2], &sizes).await
        }
        _ => { eprintln!("role must be server or client"); std::process::exit(2); }
    }
}
