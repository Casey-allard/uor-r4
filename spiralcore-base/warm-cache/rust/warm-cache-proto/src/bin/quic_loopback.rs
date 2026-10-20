//! QUIC loopback transport for the warm-cache prototype (quinn + rustls).
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

#[tokio::main]
async fn main() {
    rustls::crypto::ring::default_provider().install_default().ok();
    let server_cert = certs::cert("server-cert.pem");
    let ca_cert = certs::cert("ca-cert.pem");
    let key = certs::key("server-key.pem");
    let server_config = ServerConfig::with_single_cert(vec![server_cert, ca_cert.clone()], key).unwrap();
    let bind: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let endpoint = Endpoint::server(server_config, bind).unwrap();
    let addr = endpoint.local_addr().unwrap();
    let server = tokio::spawn(async move {
        while let Some(incoming) = endpoint.accept().await {
            tokio::spawn(async move {
                let conn = incoming.await.unwrap();
                let mut seed: Option<Seed> = None;
                while let Ok((mut send, mut recv)) = conn.accept_bi().await {
                    let msg = recv.read_to_end(1 << 26).await.unwrap();
                    let ok = if msg.is_empty() { false } else {
                        match msg[0] {
                            0 => { seed = Some(Seed::new(msg[1..].to_vec(), 1)); true }
                            1 => {
                                if let Some(s) = &seed {
                                    match WarmWire::decode(&msg[1..]) {
                                        Ok(w) => match w.to_frame() {
                                            Ok(f) => receive_warm(s, &f.header, &f) == WarmResult::Committed,
                                            Err(_) => false,
                                        },
                                        Err(_) => false,
                                    }
                                } else { false }
                            }
                            _ => false,
                        }
                    };
                    send.write_all(&[if ok { b'K' } else { b'E' }]).await.unwrap();
                    send.finish().unwrap();
                }
            });
        }
    });

    let mut roots = rustls::RootCertStore::empty();
    roots.add(ca_cert).unwrap();
    let client_crypto = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let qcc = quinn::crypto::rustls::QuicClientConfig::try_from(client_crypto).unwrap();
    let client_config = ClientConfig::new(Arc::new(qcc));
    let mut client = Endpoint::client("127.0.0.1:0".parse().unwrap()).unwrap();
    client.set_default_client_config(client_config);
    let conn = client.connect(addr, "localhost").unwrap().await.unwrap();

    let mut cell = KuramotoCell::new(71, PHASE_BITS);
    for _ in 0..2000 { cell.step(0.001); }
    let h = TimeHeader::from_snapshot(0, 0, 0, &cell.snapshot());
    let sizes = [4usize * 1024, 64 * 1024, 1 << 20, 4 << 20];
    println!("size,cold_bytes,warm_bytes,warm_payload,repair_bytes,repair_payload,elapsed_ms");
    for &size in &sizes {
        let bytes: Vec<u8> = (0..size).map(|i| ((i * 17 + 3) & 0xFF) as u8).collect();
        let seed = Seed::new(bytes, 1);
        let identity = make_warm(&seed, Plan::Identity, h.clone());
        let mut page = seed.bytes[0..PAGE].to_vec();
        page[10] ^= 0xAA;
        let repair = make_warm(&seed, Plan::PageRepair(vec![(0, page)]), h.clone());
        let wi = WarmWire::from_frame(&identity);
        let wr = WarmWire::from_frame(&repair);
        let t0 = Instant::now();
        let cold = exchange(&conn, 0, &seed.bytes).await;
        assert_eq!(cold, b"K");
        let warm = exchange(&conn, 1, &wi.encode()).await;
        assert_eq!(warm, b"K");
        let rep = exchange(&conn, 1, &wr.encode()).await;
        assert_eq!(rep, b"K");
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        println!("{},{},{},{},{},{},{:.3}",
            size, seed.bytes.len(), wi.wire_len(), wi.payload.len(),
            wr.wire_len(), wr.payload.len(), ms);
    }
    server.abort();
}
