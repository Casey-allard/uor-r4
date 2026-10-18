//! objpeer: artifact replication over the native UOR-R4 object store.
//!
//! Folds a real file (a container layer, VM/disk image, model artifact or checkpoint) into the
//! content-addressed native store, serves its native manifest over TCP, and pulls only the nodes a
//! peer store is missing. The store persists between runs, so a restarted node stays warm.

use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::thread;

use sha2::{Digest, Sha256};
use uor_r4_object_store::{peer, Error, ExpectedObject, Limits, Manifest, Store};

fn limits() -> Limits {
    Limits {
        max_object_bytes: 1 << 34,
        max_records: 64_000_000,
        max_record_bytes: 1 << 34,
        max_returned_bytes: 1 << 34,
        max_visits: 1_000_000_000,
        max_occurrences: 64_000_000,
    }
}

fn die(msg: &str) -> ! {
    eprintln!("ERROR {}", msg);
    std::process::exit(2);
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex32(value: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    if value.len() != 64 {
        die("--expect values must be 64 hex characters");
    }
    for index in 0..32 {
        out[index] = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .unwrap_or_else(|_| die("--expect value is not valid hex"));
    }
    out
}

type SyncServed = (Vec<(Manifest, peer::PullStats)>, u64, String);

struct Counted<S> {
    inner: S,
    bytes: u64,
}

impl<S: Read> Read for Counted<S> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.bytes += n as u64;
        Ok(n)
    }
}

impl<S: Write> Write for Counted<S> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.bytes += n as u64;
        Ok(n)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

fn load_store(path: &str) -> Store {
    let mut store = Store::new(limits());
    if Path::new(path).exists() {
        store
            .load(path)
            .unwrap_or_else(|e| die(&format!("load store {}: {:?}", path, e)));
    }
    store
}

fn save_store(store: &Store, path: &str) {
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).ok();
        }
    }
    store
        .save(path)
        .unwrap_or_else(|e| die(&format!("save store {}: {:?}", path, e)));
}

fn fold_file(store: &mut Store, path: &str) -> Manifest {
    let bytes = fs::read(path).unwrap_or_else(|e| die(&format!("read {}: {}", path, e)));
    store
        .fold(&bytes)
        .unwrap_or_else(|e| die(&format!("fold {}: {:?}", path, e)))
        .manifest
}

fn retryable(error: &Error) -> bool {
    !matches!(
        error,
        Error::BindingMismatch | Error::UnknownObject | Error::LimitExceeded(_)
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RepairStatus {
    Current,
    Repaired,
    Failed,
    Unavailable,
}
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        die("usage: objpeer info|serve|pull|verify|join|sync|repair [--store S] [--object F ...] [--have F] [--bind A] [--addr A] [--out F] [--targets F] [--out-dir D] [--select-sha256 HEX] [--expect-sha256 HEX] [--expect-version HEX] [--expect-bytes N] [--retries N] [--json] [--batch N] [--fetch-window N] (pull/sync/repair) [--sessions N] [--concurrent] (serve)");
    }
    let mut store_path = String::from("objpeer.store");
    let mut objects: Vec<String> = Vec::new();
    let mut have = String::new();
    let mut bind = String::from("127.0.0.1:0");
    let mut addrs: Vec<String> = Vec::new();
    let mut out = String::new();
    let mut sessions: usize = 1;
    let mut concurrent = false;
    let mut sessions_given = false;
    let mut concurrent_given = false;
    let mut expect_sha: Option<[u8; 32]> = None;
    let mut expect_version: Option<[u8; 32]> = None;
    let mut expect_bytes: Option<u64> = None;
    let mut targets = String::new();
    let mut out_dir = String::new();
    let mut json = false;
    let mut select_shas: Vec<[u8; 32]> = Vec::new();
    let mut retries: usize = 0;
    let mut batch: Option<usize> = None;
    let mut fetch_window: usize = peer::DEFAULT_FETCH_WINDOW;
    let mut fetch_window_given = false;
    let mut i = 2usize;
    while i < args.len() {
        let key = args[i].clone();
        i += 1;
        if key == "--concurrent" {
            concurrent = true;
            concurrent_given = true;
            continue;
        }
        if key == "--json" {
            json = true;
            continue;
        }
        let value = if i < args.len() {
            args[i].clone()
        } else {
            die(&format!("missing value for {}", key))
        };
        i += 1;
        match key.as_str() {
            "--store" => store_path = value,
            "--object" => objects.push(value),
            "--have" => have = value,
            "--targets" => targets = value,
            "--out-dir" => out_dir = value,
            "--retries" => retries = value.parse().unwrap_or_else(|_| die("bad --retries")),
            "--bind" => bind = value,
            "--addr" => addrs.push(value),
            "--out" => out = value,
            "--expect-sha256" => expect_sha = Some(hex32(&value)),
            "--expect-version" => expect_version = Some(hex32(&value)),
            "--expect-bytes" => {
                expect_bytes = Some(value.parse().unwrap_or_else(|_| die("bad --expect-bytes")))
            }
            "--select-sha256" => select_shas.push(hex32(&value)),
            "--sessions" => {
                sessions = value.parse().unwrap_or_else(|_| die("bad --sessions"));
                sessions_given = true;
            }
            "--batch" => batch = Some(value.parse().unwrap_or_else(|_| die("bad --batch"))),
            "--fetch-window" => {
                fetch_window = value.parse().unwrap_or_else(|_| die("bad --fetch-window"));
                fetch_window_given = true;
            }
            other => die(&format!("unknown argument {}", other)),
        }
    }
    let command = args[1].as_str();
    if batch.is_some() && !matches!(command, "pull" | "sync" | "repair") {
        die("--batch applies only to pull, sync and repair");
    }
    if (sessions_given || concurrent_given) && command != "serve" {
        die("--sessions/--concurrent apply only to serve");
    }
    if fetch_window_given && !matches!(command, "pull" | "sync" | "repair") {
        die("--fetch-window applies only to pull, sync and repair");
    }
    if !matches!(fetch_window, 1..=64) {
        die("--fetch-window must be 1..=64");
    }
    let addr = addrs.first().cloned().unwrap_or_default();
    match command {
        "join" => {
            let object = objects
                .first()
                .cloned()
                .unwrap_or_else(|| die("join requires --object"));
            let expected_sha = expect_sha.unwrap_or_else(|| die("join requires --expect-sha256"));
            let started = std::time::Instant::now();
            let bytes =
                fs::read(&object).unwrap_or_else(|e| die(&format!("read {}: {}", object, e)));
            let digest: [u8; 32] = Sha256::digest(&bytes).into();
            if digest != expected_sha {
                die("join: BindingMismatch (object sha256 does not match --expect-sha256)");
            }
            if let Some(expected) = expect_bytes {
                if bytes.len() as u64 != expected {
                    die("join: BindingMismatch (object length does not match --expect-bytes)");
                }
            }
            let mut store = load_store(&store_path);
            let manifest = fold_file(&mut store, &object);
            save_store(&store, &store_path);
            let millis = started.elapsed().as_millis();
            if json {
                println!(
                    r#"{{"object_bytes":{},"roots":{},"records":{},"sha256":"{}","store":"{}","millis":{}}}"#,
                    manifest.byte_len(),
                    manifest.roots().len(),
                    store.record_count(),
                    hex(&manifest.whole_sha256()),
                    store_path,
                    millis
                );
            } else {
                println!(
                    "JOIN object_bytes={} roots={} records={} sha256={} store={} millis={}",
                    manifest.byte_len(),
                    manifest.roots().len(),
                    store.record_count(),
                    hex(&manifest.whole_sha256()),
                    store_path,
                    millis
                );
            }
        }
        "info" => {
            let object = objects
                .first()
                .cloned()
                .unwrap_or_else(|| die("info requires --object"));
            let mut store = load_store(&store_path);
            let manifest = fold_file(&mut store, &object);
            save_store(&store, &store_path);
            println!(
                "INFO object_bytes={} roots={} records={} sha256={}",
                manifest.byte_len(),
                manifest.roots().len(),
                store.record_count(),
                hex(&manifest.whole_sha256())
            );
        }
        "serve" => {
            if objects.is_empty() {
                die("serve requires at least one --object");
            }
            let mut store = load_store(&store_path);
            let mut manifests = Vec::new();
            for path in &objects {
                manifests.push(fold_file(&mut store, path));
            }
            save_store(&store, &store_path);
            let listener =
                TcpListener::bind(&bind).unwrap_or_else(|e| die(&format!("bind {}: {}", bind, e)));
            println!(
                "READY {} objects={} records={}",
                listener.local_addr().unwrap(),
                manifests.len(),
                store.record_count()
            );
            for manifest in &manifests {
                println!(
                    "OBJECT object_bytes={} roots={} sha256={} manifest_version={}",
                    manifest.byte_len(),
                    manifest.roots().len(),
                    hex(&manifest.whole_sha256()),
                    hex(&manifest.version())
                );
            }
            if concurrent {
                thread::scope(|scope| {
                    for _ in 0..sessions {
                        let (stream, _) = listener
                            .accept()
                            .unwrap_or_else(|e| die(&format!("accept: {}", e)));
                        let store_ref = &store;
                        let manifests_ref = &manifests;
                        scope.spawn(move || {
                            let mut reader = stream
                                .try_clone()
                                .unwrap_or_else(|e| die(&format!("clone: {}", e)));
                            let mut writer = stream;
                            if let Err(error) =
                                peer::serve_many(store_ref, manifests_ref, &mut reader, &mut writer)
                            {
                                eprintln!("session error: {:?}", error);
                            }
                        });
                    }
                });
                println!("SERVED {}", sessions);
            } else {
                for _ in 0..sessions {
                    let (stream, _) = listener
                        .accept()
                        .unwrap_or_else(|e| die(&format!("accept: {}", e)));
                    let mut reader = stream
                        .try_clone()
                        .unwrap_or_else(|e| die(&format!("clone: {}", e)));
                    let mut writer = stream;
                    peer::serve_many(&store, &manifests, &mut reader, &mut writer)
                        .unwrap_or_else(|e| die(&format!("serve session: {:?}", e)));
                    println!("SERVED");
                }
            }
        }
        "sync" => {
            if addrs.is_empty() || targets.is_empty() || out_dir.is_empty() {
                die("sync requires --addr, --targets and --out-dir");
            }
            let mut store = load_store(&store_path);
            if !have.is_empty() {
                fold_file(&mut store, &have);
            }
            let text = fs::read_to_string(&targets)
                .unwrap_or_else(|e| die(&format!("read {}: {}", targets, e)));
            let mut selectors: Vec<[u8; 32]> = Vec::new();
            for (number, line) in text.lines().enumerate() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if line.len() != 64 {
                    die(&format!(
                        "{}:{}: expected 64 hex characters",
                        targets,
                        number + 1
                    ));
                }
                selectors.push(hex32(line));
            }
            if selectors.is_empty() {
                die("sync targets file contained no selectors");
            }
            let requests: Vec<peer::PullRequest> = selectors
                .iter()
                .map(|selector| peer::PullRequest {
                    expected: None,
                    selector: Some(*selector),
                })
                .collect();
            let mut served: Option<SyncServed> = None;
            for address in &addrs {
                let mut attempt = 0usize;
                loop {
                    let outcome = (|| -> uor_r4_object_store::Result<(
                        Vec<(Manifest, peer::PullStats)>,
                        u64,
                    )> {
                        let stream = TcpStream::connect(address)?;
                        let reader_stream = stream.try_clone()?;
                        let mut reader = Counted {
                            inner: reader_stream,
                            bytes: 0,
                        };
                        let mut writer = Counted {
                            inner: stream,
                            bytes: 0,
                        };
                        let results = peer::pull_series_with_batch_window(
                            &mut store,
                            &requests,
                            batch,
                            fetch_window,
                            &mut reader,
                            &mut writer,
                        )?;
                        Ok((results, reader.bytes + writer.bytes))
                    })();
                    match outcome {
                        Ok((results, wire)) => {
                            served = Some((results, wire, address.clone()));
                            break;
                        }
                        Err(error) if attempt < retries && retryable(&error) => {
                            attempt += 1;
                            eprintln!("retry {}/{} after {:?}", attempt, retries, error);
                        }
                        Err(error) => {
                            eprintln!(
                                "replica {} unavailable for sync: {:?}; trying next",
                                address, error
                            );
                            break;
                        }
                    }
                }
                if served.is_some() {
                    break;
                }
            }
            let (results, wire_bytes, replica_used) =
                served.unwrap_or_else(|| die("no replica served the sync target set"));
            fs::create_dir_all(&out_dir)
                .unwrap_or_else(|e| die(&format!("create {}: {}", out_dir, e)));
            let mut nodes = 0u64;
            let mut record_bytes = 0u64;
            for (index, (manifest, stats)) in results.iter().enumerate() {
                let rebuilt = store
                    .rebuild(manifest)
                    .unwrap_or_else(|e| die(&format!("rebuild: {:?}", e)));
                let name = format!("{}.bin", hex(&selectors[index]));
                fs::write(Path::new(&out_dir).join(name), rebuilt.bytes())
                    .unwrap_or_else(|e| die(&format!("write {}: {}", out_dir, e)));
                nodes += stats.nodes_requested;
                record_bytes += stats.record_bytes;
                if !json {
                    println!(
                        "SYNC i={} nodes={} record_bytes={} object_bytes={} verified={} sha256={}",
                        index,
                        stats.nodes_requested,
                        stats.record_bytes,
                        manifest.byte_len(),
                        stats.verified,
                        hex(&manifest.whole_sha256())
                    );
                }
            }
            save_store(&store, &store_path);
            if json {
                println!(
                    "{{\"artifacts\":{},\"nodes\":{},\"record_bytes\":{},\"wire_bytes\":{},\"replica\":\"{}\",\"output_dir\":\"{}\"}}",
                    results.len(),
                    nodes,
                    record_bytes,
                    wire_bytes,
                    replica_used,
                    out_dir
                );
            } else {
                println!(
                    "SYNC_SERIES artifacts={} nodes={} record_bytes={} wire_bytes={} replica={}",
                    results.len(),
                    nodes,
                    record_bytes,
                    wire_bytes,
                    replica_used
                );
            }
        }
        "verify" => {
            if addrs.is_empty() {
                die("verify requires --addr");
            }
            let mut store = load_store(&store_path);
            if !have.is_empty() {
                fold_file(&mut store, &have);
            }
            let expected =
                if expect_sha.is_none() && expect_version.is_none() && expect_bytes.is_none() {
                    None
                } else {
                    Some(ExpectedObject {
                        bytes: expect_bytes,
                        version: expect_version,
                        whole_sha256: expect_sha,
                    })
                };
            let mut selectors: Vec<[u8; 32]> = select_shas.clone();
            if !targets.is_empty() {
                let text = fs::read_to_string(&targets)
                    .unwrap_or_else(|e| die(&format!("read {}: {}", targets, e)));
                for (number, line) in text.lines().enumerate() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if line.len() != 64 {
                        die(&format!(
                            "{}:{}: expected 64 hex characters",
                            targets,
                            number + 1
                        ));
                    }
                    selectors.push(hex32(line));
                }
            }
            let mut requests = Vec::new();
            if selectors.is_empty() {
                requests.push(peer::VerifyRequest {
                    expected,
                    selector: None,
                });
            } else {
                for selector in &selectors {
                    requests.push(peer::VerifyRequest {
                        expected,
                        selector: Some(*selector),
                    });
                }
            }
            let mut replicas: Vec<(String, Vec<peer::VerifyOutcome>, u64)> = Vec::new();
            for address in &addrs {
                let mut attempt = 0usize;
                let (outcomes, wire_bytes) = loop {
                    let outcome =
                        (|| -> uor_r4_object_store::Result<(Vec<peer::VerifyOutcome>, u64)> {
                            let stream = TcpStream::connect(address)?;
                            let reader_stream = stream.try_clone()?;
                            let mut reader = Counted {
                                inner: reader_stream,
                                bytes: 0,
                            };
                            let mut writer = Counted {
                                inner: stream,
                                bytes: 0,
                            };
                            let outcomes =
                                peer::verify_series(&store, &requests, &mut reader, &mut writer)?;
                            Ok((outcomes, reader.bytes + writer.bytes))
                        })();
                    match outcome {
                        Ok(value) => break value,
                        Err(error) if attempt < retries && retryable(&error) => {
                            attempt += 1;
                            eprintln!("retry {}/{} after {:?}", attempt, retries, error);
                        }
                        Err(error) => die(&format!("verify {}: {:?}", address, error)),
                    }
                };
                replicas.push((address.clone(), outcomes, wire_bytes));
            }
            let checks = replicas
                .first()
                .map(|(_, outcomes, _)| outcomes.len())
                .unwrap_or(0);
            let mut all_ok = vec![true; checks];
            let mut stale_targets = vec![false; checks];
            let mut unknown_targets = vec![false; checks];
            let mut mismatch_targets = vec![false; checks];
            let mut total_wire = 0u64;
            for (_, outcomes, wire) in &replicas {
                total_wire += wire;
                for (index, outcome) in outcomes.iter().enumerate() {
                    if index >= checks {
                        continue;
                    }
                    match outcome {
                        peer::VerifyOutcome::Verified { .. } => {}
                        peer::VerifyOutcome::Stale { .. } => {
                            all_ok[index] = false;
                            stale_targets[index] = true;
                        }
                        peer::VerifyOutcome::Unknown => {
                            all_ok[index] = false;
                            unknown_targets[index] = true;
                        }
                        peer::VerifyOutcome::Mismatch => {
                            all_ok[index] = false;
                            mismatch_targets[index] = true;
                        }
                    }
                }
            }
            let verified_all = all_ok.iter().filter(|value| **value).count();
            let stale = stale_targets.iter().filter(|value| **value).count();
            let unknown = unknown_targets.iter().filter(|value| **value).count();
            let mismatch = mismatch_targets.iter().filter(|value| **value).count();
            let single = replicas.len() == 1;
            if json {
                if single {
                    let (_, outcomes, _) = &replicas[0];
                    let results: Vec<String> = outcomes
                        .iter()
                        .enumerate()
                        .map(|(index, outcome)| {
                            let selector = selectors
                                .get(index)
                                .map(|value| hex(value))
                                .unwrap_or_default();
                            match outcome {
                                peer::VerifyOutcome::Verified { missing } => format!(
                                    "{{\"i\":{},\"status\":\"verified\",\"missing\":{},\"selector\":\"{}\"}}",
                                    index, missing, selector
                                ),
                                peer::VerifyOutcome::Stale { missing } => format!(
                                    "{{\"i\":{},\"status\":\"stale\",\"missing\":{},\"selector\":\"{}\"}}",
                                    index, missing, selector
                                ),
                                peer::VerifyOutcome::Unknown => format!(
                                    "{{\"i\":{},\"status\":\"unknown\",\"selector\":\"{}\"}}",
                                    index, selector
                                ),
                                peer::VerifyOutcome::Mismatch => format!(
                                    "{{\"i\":{},\"status\":\"mismatch\",\"selector\":\"{}\"}}",
                                    index, selector
                                ),
                            }
                        })
                        .collect();
                    println!(
                        "{{\"checks\":{},\"verified\":{},\"stale\":{},\"unknown\":{},\"mismatch\":{},\"wire_bytes\":{},\"results\":[{}]}}",
                        outcomes.len(),
                        verified_all,
                        stale,
                        unknown,
                        mismatch,
                        total_wire,
                        results.join(",")
                    );
                } else {
                    let targets: Vec<String> = (0..checks)
                        .map(|index| {
                            let selector = selectors
                                .get(index)
                                .map(|value| hex(value))
                                .unwrap_or_default();
                            let per_replica: Vec<String> = replicas
                                .iter()
                                .map(|(address, outcomes, _)| match &outcomes[index] {
                                    peer::VerifyOutcome::Verified { missing } => format!(
                                        "{{\"addr\":\"{}\",\"status\":\"verified\",\"missing\":{}}}",
                                        address, missing
                                    ),
                                    peer::VerifyOutcome::Stale { missing } => format!(
                                        "{{\"addr\":\"{}\",\"status\":\"stale\",\"missing\":{}}}",
                                        address, missing
                                    ),
                                    peer::VerifyOutcome::Unknown => format!(
                                        "{{\"addr\":\"{}\",\"status\":\"unknown\"}}",
                                        address
                                    ),
                                    peer::VerifyOutcome::Mismatch => format!(
                                        "{{\"addr\":\"{}\",\"status\":\"mismatch\"}}",
                                        address
                                    ),
                                })
                                .collect();
                            format!(
                                "{{\"selector\":\"{}\",\"replicas\":[{}]}}",
                                selector,
                                per_replica.join(",")
                            )
                        })
                        .collect();
                    println!(
                        "{{\"replicas\":{},\"checks\":{},\"verified_all\":{},\"stale\":{},\"unknown\":{},\"mismatch\":{},\"wire_bytes\":{},\"results\":[{}]}}",
                        replicas.len(),
                        checks,
                        verified_all,
                        stale,
                        unknown,
                        mismatch,
                        total_wire,
                        targets.join(",")
                    );
                }
            } else if single {
                let (_, outcomes, _) = &replicas[0];
                for (index, outcome) in outcomes.iter().enumerate() {
                    match outcome {
                        peer::VerifyOutcome::Verified { missing } => {
                            println!("VERIFY i={} missing={} verified=true", index, missing);
                        }
                        peer::VerifyOutcome::Stale { missing } => {
                            println!("VERIFY i={} missing={} verified=false", index, missing);
                        }
                        peer::VerifyOutcome::Unknown => {
                            eprintln!("VERIFY i={} UnknownObject", index);
                        }
                        peer::VerifyOutcome::Mismatch => {
                            eprintln!("VERIFY i={} BindingMismatch", index);
                        }
                    }
                }
                println!(
                    "VERIFY_SERIES checks={} verified={} stale={} unknown={} mismatch={} wire_bytes={}",
                    outcomes.len(),
                    verified_all,
                    stale,
                    unknown,
                    mismatch,
                    total_wire
                );
            } else {
                for (address, outcomes, wire) in &replicas {
                    let mut rep_verified = 0usize;
                    let mut rep_stale = 0usize;
                    let mut rep_unknown = 0usize;
                    let mut rep_mismatch = 0usize;
                    for outcome in outcomes {
                        match outcome {
                            peer::VerifyOutcome::Verified { .. } => rep_verified += 1,
                            peer::VerifyOutcome::Stale { .. } => rep_stale += 1,
                            peer::VerifyOutcome::Unknown => rep_unknown += 1,
                            peer::VerifyOutcome::Mismatch => rep_mismatch += 1,
                        }
                    }
                    println!(
                        "REPLICA addr={} verified={} stale={} unknown={} mismatch={} wire_bytes={}",
                        address, rep_verified, rep_stale, rep_unknown, rep_mismatch, wire
                    );
                }
                println!(
                    "VERIFY_REPLICAS replicas={} checks={} verified_all={} stale={} unknown={} mismatch={} wire_bytes={}",
                    replicas.len(),
                    checks,
                    verified_all,
                    stale,
                    unknown,
                    mismatch,
                    total_wire
                );
            }
            if unknown > 0 {
                std::process::exit(2);
            }
            if stale > 0 || mismatch > 0 {
                std::process::exit(3);
            }
        }
        "repair" => {
            if addrs.is_empty() || targets.is_empty() {
                die("repair requires at least one --addr and --targets");
            }
            let mut store = load_store(&store_path);
            if !have.is_empty() {
                fold_file(&mut store, &have);
            }
            let text = fs::read_to_string(&targets)
                .unwrap_or_else(|e| die(&format!("read {}: {}", targets, e)));
            let mut selectors: Vec<[u8; 32]> = select_shas.clone();
            for (number, line) in text.lines().enumerate() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if line.len() != 64 {
                    die(&format!(
                        "{}:{}: expected 64 hex characters",
                        targets,
                        number + 1
                    ));
                }
                selectors.push(hex32(line));
            }
            if selectors.is_empty() {
                die("repair targets file contained no selectors");
            }
            let probe_requests: Vec<peer::VerifyRequest> = selectors
                .iter()
                .map(|selector| peer::VerifyRequest {
                    expected: None,
                    selector: Some(*selector),
                })
                .collect();
            let mut states: Vec<Option<Vec<peer::VerifyOutcome>>> = Vec::with_capacity(addrs.len());
            for address in &addrs {
                let mut attempt = 0usize;
                let outcome = loop {
                    let result = (|| -> uor_r4_object_store::Result<Vec<peer::VerifyOutcome>> {
                        let stream = TcpStream::connect(address)?;
                        let reader_stream = stream.try_clone()?;
                        let mut reader = Counted {
                            inner: reader_stream,
                            bytes: 0,
                        };
                        let mut writer = Counted {
                            inner: stream,
                            bytes: 0,
                        };
                        peer::verify_series(&store, &probe_requests, &mut reader, &mut writer)
                    })();
                    match result {
                        Ok(outcomes) => break Some(outcomes),
                        Err(error) if attempt < retries && retryable(&error) => {
                            attempt += 1;
                            eprintln!("retry {}/{} after {:?}", attempt, retries, error);
                        }
                        Err(error) => {
                            eprintln!("replica {} probe failed: {:?}; skipping", address, error);
                            break None;
                        }
                    }
                };
                states.push(outcome);
            }
            let mut results: Vec<(RepairStatus, Option<usize>)> =
                vec![(RepairStatus::Unavailable, None); selectors.len()];
            let mut groups: Vec<(usize, Vec<usize>)> = Vec::new();
            let mut current = 0usize;
            let mut unavailable = 0usize;
            for (index, _) in selectors.iter().enumerate() {
                let mut chosen: Option<usize> = None;
                let mut is_current = false;
                for (replica, state) in states.iter().enumerate() {
                    let Some(outcomes) = state else {
                        continue;
                    };
                    let Some(outcome) = outcomes.get(index) else {
                        continue;
                    };
                    match outcome {
                        peer::VerifyOutcome::Verified { .. } => {
                            is_current = true;
                            chosen = Some(replica);
                            break;
                        }
                        peer::VerifyOutcome::Stale { .. } => {
                            if chosen.is_none() {
                                chosen = Some(replica);
                            }
                        }
                        peer::VerifyOutcome::Unknown | peer::VerifyOutcome::Mismatch => {}
                    }
                }
                if is_current {
                    current += 1;
                    results[index] = (RepairStatus::Current, chosen);
                } else if let Some(replica) = chosen {
                    if let Some(group) = groups.iter_mut().find(|(item, _)| *item == replica) {
                        group.1.push(index);
                    } else {
                        groups.push((replica, vec![index]));
                    }
                    results[index] = (RepairStatus::Repaired, Some(replica));
                } else {
                    unavailable += 1;
                }
            }
            let mut failed = 0usize;
            let mut repaired_wire = 0u64;
            for (replica, indexes) in &groups {
                let address = &addrs[*replica];
                let pull_requests: Vec<peer::PullRequest> = indexes
                    .iter()
                    .map(|index| peer::PullRequest {
                        expected: None,
                        selector: Some(selectors[*index]),
                    })
                    .collect();
                let mut attempt = 0usize;
                let outcome = loop {
                    let result =
                        (|| -> uor_r4_object_store::Result<Vec<(Manifest, peer::PullStats)>> {
                            let stream = TcpStream::connect(address)?;
                            let reader_stream = stream.try_clone()?;
                            let mut reader = Counted {
                                inner: reader_stream,
                                bytes: 0,
                            };
                            let mut writer = Counted {
                                inner: stream,
                                bytes: 0,
                            };
                            let results = peer::pull_series_with_batch_window(
                                &mut store,
                                &pull_requests,
                                batch,
                                fetch_window,
                                &mut reader,
                                &mut writer,
                            )?;
                            repaired_wire += reader.bytes + writer.bytes;
                            Ok(results)
                        })();
                    match result {
                        Ok(results) => break results,
                        Err(error) if attempt < retries && retryable(&error) => {
                            attempt += 1;
                            eprintln!("retry {}/{} after {:?}", attempt, retries, error);
                        }
                        Err(error) => {
                            eprintln!("repair from {} failed: {:?}", address, error);
                            for index in indexes {
                                results[*index] = (RepairStatus::Failed, Some(*replica));
                            }
                            failed += indexes.len();
                            break Vec::new();
                        }
                    }
                };
                if outcome.len() != pull_requests.len() && !outcome.is_empty() {
                    failed += indexes.len();
                }
            }
            save_store(&store, &store_path);
            let repaired = results
                .iter()
                .filter(|(status, _)| *status == RepairStatus::Repaired)
                .count();
            let status_name = |status: RepairStatus| match status {
                RepairStatus::Current => "current",
                RepairStatus::Repaired => "repaired",
                RepairStatus::Failed => "failed",
                RepairStatus::Unavailable => "unavailable",
            };
            if json {
                let entries: Vec<String> = results
                    .iter()
                    .enumerate()
                    .map(|(index, (status, replica))| {
                        let selector = hex(&selectors[index]);
                        match replica {
                            Some(replica) => format!(
                                "{{\"selector\":\"{}\",\"status\":\"{}\",\"replica\":\"{}\"}}",
                                selector,
                                status_name(*status),
                                addrs[*replica]
                            ),
                            None => format!(
                                "{{\"selector\":\"{}\",\"status\":\"{}\"}}",
                                selector,
                                status_name(*status)
                            ),
                        }
                    })
                    .collect();
                println!(
                    "{{\"replicas\":{},\"checks\":{},\"current\":{},\"repaired\":{},\"unavailable\":{},\"failed\":{},\"wire_bytes\":{},\"results\":[{}]}}",
                    addrs.len(),
                    selectors.len(),
                    current,
                    repaired,
                    unavailable,
                    failed,
                    repaired_wire,
                    entries.join(",")
                );
            } else {
                for (index, (status, replica)) in results.iter().enumerate() {
                    match replica {
                        Some(replica) => println!(
                            "REPAIR i={} status={} replica={}",
                            index,
                            status_name(*status),
                            addrs[*replica]
                        ),
                        None => println!("REPAIR i={} status={}", index, status_name(*status)),
                    }
                }
                println!(
                    "REPAIR_SUMMARY replicas={} checks={} current={} repaired={} unavailable={} failed={} wire_bytes={}",
                    addrs.len(),
                    selectors.len(),
                    current,
                    repaired,
                    unavailable,
                    failed,
                    repaired_wire
                );
            }
            if unavailable > 0 || failed > 0 {
                std::process::exit(3);
            }
        }
        "pull" => {
            if addr.is_empty() || out.is_empty() {
                die("pull requires --addr and --out");
            }
            let mut store = load_store(&store_path);
            if !have.is_empty() {
                fold_file(&mut store, &have);
            }
            save_store(&store, &store_path);
            let expected =
                if expect_sha.is_none() && expect_version.is_none() && expect_bytes.is_none() {
                    None
                } else {
                    Some(ExpectedObject {
                        bytes: expect_bytes,
                        version: expect_version,
                        whole_sha256: expect_sha,
                    })
                };
            let mut attempt = 0usize;
            let (manifest, stats, wire_bytes) = loop {
                let outcome =
                    (|| -> uor_r4_object_store::Result<(Manifest, peer::PullStats, u64)> {
                        let stream = TcpStream::connect(&addr)?;
                        let reader_stream = stream.try_clone()?;
                        let mut reader = Counted {
                            inner: reader_stream,
                            bytes: 0,
                        };
                        let mut writer = Counted {
                            inner: stream,
                            bytes: 0,
                        };
                        let selector = select_shas.first().copied();
                        let (manifest, stats) = match batch {
                            Some(size) => peer::pull_selected_with_batch_window(
                                &mut store,
                                expected,
                                selector,
                                size,
                                fetch_window,
                                &mut reader,
                                &mut writer,
                            )?,
                            None => peer::pull_selected_with_window(
                                &mut store,
                                expected,
                                selector,
                                fetch_window,
                                &mut reader,
                                &mut writer,
                            )?,
                        };
                        Ok((manifest, stats, reader.bytes + writer.bytes))
                    })();
                match outcome {
                    Ok(value) => break value,
                    Err(error) if attempt < retries && retryable(&error) => {
                        attempt += 1;
                        eprintln!("retry {}/{} after {:?}", attempt, retries, error);
                    }
                    Err(error) => die(&format!("pull: {:?}", error)),
                }
            };
            save_store(&store, &store_path);
            let verified = store
                .rebuild(&manifest)
                .unwrap_or_else(|e| die(&format!("rebuild: {:?}", e)));
            fs::write(&out, verified.bytes())
                .unwrap_or_else(|e| die(&format!("write {}: {}", out, e)));
            println!(
                "PULL_STATS object_bytes={} nodes={} record_bytes={} wire_bytes={} rounds={} verified={} sha256={}",
                manifest.byte_len(),
                stats.nodes_requested,
                stats.record_bytes,
                wire_bytes,
                stats.rounds,
                stats.verified,
                hex(&manifest.whole_sha256())
            );
        }
        other => die(&format!("unknown command {}", other)),
    }
}
