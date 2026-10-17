//! Peer replication: a partial store pulls only the missing nodes and rebuilds the exact object.

use std::net::{TcpListener, TcpStream};
use std::thread;

use uor_r4_object_store::peer;
use uor_r4_object_store::{Limits, Store};

fn limits() -> Limits {
    Limits {
        max_object_bytes: 1 << 30,
        max_records: 10_000_000,
        max_record_bytes: 1 << 30,
        max_returned_bytes: 1 << 30,
        max_visits: 100_000_000,
        max_occurrences: 10_000_000,
    }
}

fn atoms(n: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(n * 32);
    for i in 0..n {
        for j in 0..32 {
            let v = if j < 8 {
                i.to_le_bytes()[j]
            } else {
                ((i * 37 + j * 11 + 5) & 0xFF) as u8
            };
            out.push(v);
        }
    }
    out
}

fn serve_once(sender: Store, manifest: uor_r4_object_store::Manifest) -> std::net::SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    thread::spawn(move || {
        let (stream, _) = listener.accept().expect("accept");
        let mut reader = stream.try_clone().expect("clone");
        let mut writer = stream;
        peer::serve(&sender, &manifest, &mut reader, &mut writer).expect("serve");
    });
    addr
}

#[test]
fn partial_store_pulls_only_novel_records_and_rebuilds_exactly() {
    let new = atoms(32768);
    let old = atoms(32752);
    let mut sender = Store::new(limits());
    let manifest = sender.fold(&new).expect("fold new").manifest;
    let mut receiver = Store::new(limits());
    receiver.fold(&old).expect("fold old");

    let addr = serve_once(sender, manifest.clone());
    let stream = TcpStream::connect(addr).expect("connect");
    let mut reader = stream.try_clone().expect("clone");
    let mut writer = stream;
    let (pulled, stats) = peer::pull(&mut receiver, &mut reader, &mut writer).expect("pull");
    println!(
        "peer delta: rounds={} nodes={} record_bytes={} verified={}",
        stats.rounds, stats.nodes_requested, stats.record_bytes, stats.verified
    );

    assert!(stats.verified, "receiver must verify the native digest");
    assert!(
        stats.nodes_requested > 0,
        "a delta must transfer novel nodes"
    );
    assert!(
        stats.record_bytes < 4096,
        "delta must be small (got {} bytes for {} payload bytes)",
        stats.record_bytes,
        new.len()
    );
    assert_eq!(pulled.whole_sha256(), manifest.whole_sha256());
    let rebuilt = receiver.rebuild(&pulled).expect("rebuild");
    assert_eq!(rebuilt.bytes(), &new[..]);
}

#[test]
fn identical_store_pulls_no_records() {
    let payload = atoms(4096);
    let mut sender = Store::new(limits());
    let manifest = sender.fold(&payload).expect("fold").manifest;
    let mut receiver = Store::new(limits());
    receiver.fold(&payload).expect("fold local");

    let addr = serve_once(sender, manifest.clone());
    let stream = TcpStream::connect(addr).expect("connect");
    let mut reader = stream.try_clone().expect("clone");
    let mut writer = stream;
    let (pulled, stats) = peer::pull(&mut receiver, &mut reader, &mut writer).expect("pull");
    assert_eq!(stats.nodes_requested, 0);
    assert_eq!(stats.record_bytes, 0);
    assert!(stats.verified);
    assert_eq!(
        receiver.rebuild(&pulled).expect("rebuild").bytes(),
        &payload[..]
    );
}

#[test]
fn record_accessors_expose_native_record_bytes() {
    let payload = atoms(64);
    let mut store = Store::new(limits());
    let manifest = store.fold(&payload).expect("fold").manifest;
    let (root, scale) = manifest.roots()[0];
    assert!(store.has_record(root));
    let bytes = store.record_bytes(root).expect("record bytes");
    let record = uor_r4_object_store::Record::decode(&bytes).expect("native decode");
    assert_eq!(record.id(), root);
    assert_eq!(record.scale(), scale);
}
