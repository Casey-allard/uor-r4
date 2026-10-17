//! Peer replication: trusted identity, cold pull, delta correlation, atomic commit, closure scoping.

use std::net::{TcpListener, TcpStream};
use std::thread;

use uor_r4_object_store::peer::{self, ExpectedObject};
use uor_r4_object_store::{Error, Limits, Manifest, Store};

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

fn atoms(n: usize, salt: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(n * 32);
    for i in 0..n {
        for j in 0..32 {
            let v = if j < 8 {
                i.to_le_bytes()[j]
            } else {
                ((i * 37 + j * 11 + salt) & 0xFF) as u8
            };
            out.push(v);
        }
    }
    out
}

fn pull_from(
    sender: Store,
    manifest: Manifest,
    receiver: &mut Store,
    expected: Option<ExpectedObject>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    let sender_manifest = manifest.clone();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("accept");
        let mut reader = stream.try_clone().expect("clone");
        let mut writer = stream;
        peer::serve(&sender, &sender_manifest, &mut reader, &mut writer).expect("serve");
    });
    let stream = TcpStream::connect(addr).expect("connect");
    let mut reader = stream.try_clone().expect("clone");
    let mut writer = stream;
    let result = peer::pull_expected(receiver, expected, &mut reader, &mut writer);
    server.join().expect("server");
    result.expect("pull");
}

#[test]
fn partial_store_pulls_only_novel_records_and_rebuilds_exactly() {
    let new = atoms(32768, 0);
    let old = atoms(32752, 0);
    let mut sender = Store::new(limits());
    let manifest = sender.fold(&new).expect("fold new").manifest;
    let mut receiver = Store::new(limits());
    receiver.fold(&old).expect("fold old");
    let expected = ExpectedObject::from_manifest(&manifest);
    pull_from(sender, manifest.clone(), &mut receiver, Some(expected));
    let rebuilt = receiver.rebuild(&manifest).expect("rebuild");
    assert_eq!(rebuilt.bytes(), &new[..]);
}

#[test]
fn identical_store_pulls_no_records() {
    let payload = atoms(4096, 1);
    let mut sender = Store::new(limits());
    let manifest = sender.fold(&payload).expect("fold").manifest;
    let mut receiver = Store::new(limits());
    receiver.fold(&payload).expect("fold local");
    let expected = ExpectedObject::from_manifest(&manifest);
    pull_from(sender, manifest.clone(), &mut receiver, Some(expected));
    assert_eq!(
        receiver.rebuild(&manifest).expect("rebuild").bytes(),
        &payload[..]
    );
}

#[test]
fn cold_pull_completes_a_one_mib_object() {
    let payload = atoms(32768, 2);
    let mut sender = Store::new(limits());
    let manifest = sender.fold(&payload).expect("fold").manifest;
    let mut receiver = Store::new(limits());
    assert_eq!(receiver.record_count(), 0);
    let expected = ExpectedObject::from_manifest(&manifest);
    pull_from(sender, manifest.clone(), &mut receiver, Some(expected));
    assert!(
        receiver.record_count() >= 32768,
        "a cold pull must transfer the whole object, got {} records",
        receiver.record_count()
    );
    let rebuilt = receiver.rebuild(&manifest).expect("rebuild");
    assert_eq!(rebuilt.bytes(), &payload[..]);
    assert!(rebuilt.whole_object_digest_verified());
}

#[test]
fn scattered_edits_cost_paths_not_payload() {
    let old = atoms(32768, 3);
    let mut new = old.clone();
    for edit in 0..16usize {
        let atom = (edit * 2048 + 37) % 32768;
        let base = atom * 32;
        for j in 0..32 {
            new[base + j] = ((edit * 131 + j * 37 + 7) & 0xFF) as u8;
        }
    }
    let mut sender = Store::new(limits());
    let manifest = sender.fold(&new).expect("fold new").manifest;
    let mut receiver = Store::new(limits());
    receiver.fold(&old).expect("fold old");
    let before = receiver.record_count();
    let expected = ExpectedObject::from_manifest(&manifest);
    pull_from(sender, manifest.clone(), &mut receiver, Some(expected));
    let rebuilt = receiver.rebuild(&manifest).expect("rebuild");
    assert_eq!(rebuilt.bytes(), &new[..]);
    let added = receiver.record_count() - before;
    assert!(
        added <= 16 * 12 + 16,
        "scattered edits must add paths, not payload: {} added records",
        added
    );
    println!("scattered 16-edit delta added {} records", added);
}

#[test]
fn a_wrong_expected_identity_is_rejected_before_import() {
    let payload = atoms(4096, 4);
    let mut sender = Store::new(limits());
    let manifest = sender.fold(&payload).expect("fold").manifest;
    let mut receiver = Store::new(limits());
    let wrong = ExpectedObject {
        bytes: manifest.byte_len(),
        version: [0u8; 32],
    };
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    let sender_manifest = manifest.clone();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("accept");
        let mut reader = stream.try_clone().expect("clone");
        let mut writer = stream;
        let _ = peer::serve(&sender, &sender_manifest, &mut reader, &mut writer);
    });
    let stream = TcpStream::connect(addr).expect("connect");
    let mut reader = stream.try_clone().expect("clone");
    let mut writer = stream;
    let result = peer::pull_expected(&mut receiver, Some(wrong), &mut reader, &mut writer);
    assert!(matches!(result, Err(Error::BindingMismatch)));
    assert_eq!(
        receiver.record_count(),
        0,
        "no record may be imported before verification"
    );
    drop(writer);
    drop(reader);
    server.join().ok();
}

#[test]
fn record_accessors_expose_native_record_bytes() {
    let payload = atoms(64, 5);
    let mut store = Store::new(limits());
    let manifest = store.fold(&payload).expect("fold").manifest;
    let (root, scale) = manifest.roots()[0];
    assert!(store.has_record(root));
    let bytes = store.record_bytes(root).expect("record bytes");
    let record = uor_r4_object_store::Record::decode(&bytes).expect("native decode");
    assert_eq!(record.id(), root);
    assert_eq!(record.scale(), scale);
}

#[test]
fn deduplicated_dag_nodes_are_requested_once() {
    // Repeated atom content creates identical subtrees, so the FBS tree is a DAG. A delta pull must
    // request each missing node exactly once and complete.
    fn periodic(n: usize, salt: u8) -> Vec<u8> {
        let mut out = Vec::with_capacity(n * 32);
        for i in 0..n {
            for j in 0..32 {
                out.push((((i % 8) * 37 + j * 11 + salt as usize) & 0xFF) as u8);
            }
        }
        out
    }
    let old = periodic(65536, 1);
    let mut new = old.clone();
    for edit in 0..128usize {
        let base = 32768 + edit * 32;
        for j in 0..32 {
            new[base + j] = (((edit % 8) * 53 + j * 7 + 3) & 0xFF) as u8;
        }
    }
    let mut sender = Store::new(limits());
    let manifest = sender.fold(&new).expect("fold new").manifest;
    let mut receiver = Store::new(limits());
    receiver.fold(&old).expect("fold old");
    let expected = ExpectedObject::from_manifest(&manifest);
    pull_from(sender, manifest.clone(), &mut receiver, Some(expected));
    let rebuilt = receiver.rebuild(&manifest).expect("rebuild");
    assert_eq!(rebuilt.bytes(), &new[..]);
}
