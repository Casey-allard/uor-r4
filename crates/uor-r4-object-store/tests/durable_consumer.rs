//! Finite CPU controls for the M3 object-store durable-memory consumer (#2873).
//!
//! Covers:
//! 1. Write then read back byte-identical and token-occurrence verified.
//! 2. Process restart then restore byte-identical (`UORMEM01` + `UR4SESS1`).
//! 3. Retracted or unstated fact refused rather than guessed (zero backend reads on unstated).
//! 4. Non-memory prose turns folded into the store without overwriting stored facts (#1718).
//! 5. Explicit reset and strict `IdentityScope` partition isolation.
//! 6. Hostile or corrupt records, manifests, `UORMEM01` envelopes, and `UR4SESS1` snapshots refused.

use std::sync::atomic::{AtomicUsize, Ordering};
use uor_r4_object_store::{
    bindings::PinnedTokenizer,
    durable_consumer::{
        DurableObjectMemoryStore, FactAction, FactReadOutcome, IdentityScope, Ur4Sess1HeaderView,
        DURABLE_CONSUMER_MAGIC,
    },
    Error, Limits, Manifest, NodeId, Record, RecordBackend, Store,
};
use uor_r4_tokenizer::ByteBpeTokenizer;

const FIXTURE_TOKENIZER_JSON: &str = r#"{
    "pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},
    "added_tokens":[{"id":30,"content":"<eos>"}],
    "model":{
        "type":"BPE",
        "vocab":{
            "A":0,"l":1,"i":2,"c":3,"e":4,"Ġ":5,"L":6,"s":7,"b":8,"o":9,"n":10,"p":11,
            "r":12,"t":13,"8":14,"0":15,"9":16,"B":17,
            "Al":18,"Ali":19,"Alic":20,"Alice":21,
            "Li":22,"Lis":23,"Lisb":24,"Lisbo":25,"Lisbon":26,
            "80":27,"8080":28
        },
        "merges":["A l","Al i","Ali c","Alic e","L i","Li s","Lis b","Lisb o","Lisbo n","8 0","80 80"]
    }
}"#;

fn fixture_tokenizer() -> ByteBpeTokenizer {
    ByteBpeTokenizer::from_tokenizer_json_bytes(FIXTURE_TOKENIZER_JSON.as_bytes())
        .expect("valid fixture tokenizer")
}

fn sample_ur4sess1_bytes(position: u64, tokens: &[u32], logits: &[i32]) -> Vec<u8> {
    fn put_u32(out: &mut Vec<u8>, v: u32) {
        out.extend_from_slice(&v.to_le_bytes());
    }
    fn put_i32(out: &mut Vec<u8>, v: i32) {
        out.extend_from_slice(&v.to_le_bytes());
    }
    fn put_u64(out: &mut Vec<u8>, v: u64) {
        out.extend_from_slice(&v.to_le_bytes());
    }
    fn put_v64(out: &mut Vec<u8>, mut v: u64) {
        loop {
            let b = (v & 0x7f) as u8;
            v >>= 7;
            if v == 0 {
                out.push(b);
                return;
            }
            out.push(b | 0x80);
        }
    }
    fn put_str(out: &mut Vec<u8>, s: &str) {
        put_u32(out, s.len() as u32);
        out.extend_from_slice(s.as_bytes());
    }

    let mut out = Vec::new();
    out.extend_from_slice(b"UR4SESS1");
    put_str(&mut out, "uor-r4.stack-session/1");
    put_u32(&mut out, 1);
    put_str(&mut out, &"ab".repeat(32));
    put_u64(&mut out, position);
    put_u64(&mut out, position * 288);
    put_u64(&mut out, position * 6);
    put_i32(&mut out, 65_536);
    put_v64(&mut out, logits.len() as u64);
    for &x in logits {
        put_i32(&mut out, x);
    }
    put_v64(&mut out, tokens.len() as u64);
    for &t in tokens {
        put_u32(&mut out, t);
    }
    out.push(0); // snap_trace = None
    put_v64(&mut out, 0); // pointer_keys
    put_v64(&mut out, 0); // pointer_lifts
    put_v64(&mut out, 0); // mixture
    put_v64(&mut out, 0); // layers
    out
}

#[test]
fn write_then_read_back_byte_identical_and_occurrence_bound() {
    let tok = fixture_tokenizer();
    let pinned = PinnedTokenizer::new(&tok).expect("pinned tokenizer");
    let mut store = Store::new(Limits::default());
    let mut memory = DurableObjectMemoryStore::new();
    let scope = IdentityScope::new("user-1", "proj-m3", "sess-a").expect("valid scope");

    let v1 = memory
        .assert_fact(
            &mut store,
            &pinned,
            &scope,
            "Alice",
            "city",
            b"Alice Lisbon",
            6..12,
        )
        .expect("assert fact v1");
    assert_eq!(v1, 1);

    // Read back active fact via Store and Bindings.
    let outcome = memory
        .read_fact(&store, &pinned, &scope, "Alice", "city")
        .expect("read active fact");
    match outcome {
        FactReadOutcome::Quoted {
            owner,
            relation,
            value_bytes,
            token_ids,
            version,
            action,
            conflict,
            previous_value_bytes,
            ..
        } => {
            assert_eq!(owner, "Alice");
            assert_eq!(relation, "city");
            assert_eq!(value_bytes, b"Lisbon");
            assert!(!token_ids.is_empty());
            assert_eq!(tok.decode_bytes(&token_ids), b"Lisbon");
            assert_eq!(version, 1);
            assert_eq!(action, FactAction::Assert);
            assert!(!conflict);
            assert!(previous_value_bytes.is_none());
        }
        other => panic!("expected Quoted, got {other:?}"),
    }

    // Asserting a different value without explicit revision flags a conflict while preserving lineage.
    let v2_conflict = memory
        .assert_fact(
            &mut store,
            &pinned,
            &scope,
            "Alice",
            "city",
            b"Alice Bono",
            6..10,
        )
        .expect("assert conflicting fact");
    assert_eq!(v2_conflict, 2);

    // Explicit revision resolves the conflict and keeps the previous distinct value recoverable.
    let v3_revised = memory
        .revise_fact(
            &mut store,
            &pinned,
            &scope,
            "Alice",
            "city",
            b"Alice Bobo",
            6..10,
        )
        .expect("revise fact");
    assert_eq!(v3_revised, 3);

    // Pronoun resolution ("she") resolves to the most recent active entity ("Alice").
    let revised = memory
        .read_fact(&store, &pinned, &scope, "she", "city")
        .expect("read revised via pronoun");
    match revised {
        FactReadOutcome::Quoted {
            owner,
            value_bytes,
            version,
            action,
            conflict,
            previous_value_bytes,
            ..
        } => {
            assert_eq!(owner, "Alice");
            assert_eq!(value_bytes, b"Bobo");
            assert_eq!(version, 3);
            assert_eq!(action, FactAction::Revise);
            assert!(!conflict);
            assert_eq!(previous_value_bytes.as_deref(), Some(&b"Bono"[..]));
        }
        other => panic!("expected Quoted after revision, got {other:?}"),
    }
}

#[test]
fn restart_then_restore_byte_identical_and_ur4sess1_round_trip() {
    let tok = fixture_tokenizer();
    let pinned = PinnedTokenizer::new(&tok).expect("pinned tokenizer");
    let scope = IdentityScope::new("user-restart", "proj-m3", "sess-restart").expect("scope");

    let dir = std::env::temp_dir().join(format!(
        "uor-object-store-durable-consumer-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create temp dir");

    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let records_path = dir.join("store.records");
        let manifest_path = dir.join("scope.manifest");

        let mut store = Store::new(Limits::default());
        let mut memory = DurableObjectMemoryStore::new();

        memory.assert_fact(
            &mut store,
            &pinned,
            &scope,
            "port",
            "value",
            b"port 8080",
            5..9,
        )?;
        memory.revise_fact(
            &mut store,
            &pinned,
            &scope,
            "port",
            "value",
            b"port 9080",
            5..9,
        )?;
        memory.ingest_non_memory_prose(&mut store, &pinned, &scope, b"Alice Lisbon")?;

        let sess_bytes = sample_ur4sess1_bytes(4, &[0, 5, 6, 12], &[10, 20, -5, 99]);
        memory.set_ur4sess1_snapshot(&scope, &sess_bytes)?;

        // Export scope to the object store, persist records and manifest to disk.
        let exported_manifest = memory.export_scope(&mut store, &scope)?;
        store.save(&records_path)?;
        std::fs::write(&manifest_path, exported_manifest.encode())?;

        // Simulate process restart: load store records into a fresh Store and import_scope.
        let mut restarted_store = Store::new(Limits::default());
        restarted_store.load(&records_path)?;
        let loaded_manifest_bytes = std::fs::read(&manifest_path)?;
        let loaded_manifest = Manifest::decode(&loaded_manifest_bytes, restarted_store.limits())?;
        assert_eq!(loaded_manifest, exported_manifest);

        let mut restarted_memory = DurableObjectMemoryStore::new();
        let restored_scope = restarted_memory.import_scope(&restarted_store, &loaded_manifest)?;
        assert_eq!(restored_scope, scope);

        // Verify UR4SESS1 snapshot bytes and parsed header match bit-for-bit.
        let restored_sess = restarted_memory
            .ur4sess1_snapshot(&scope)
            .expect("restored UR4SESS1 snapshot present");
        assert_eq!(restored_sess, sess_bytes.as_slice());
        let header = Ur4Sess1HeaderView::parse(restored_sess)?;
        assert_eq!(header.position, 4);
        assert_eq!(header.tokens, vec![0, 5, 6, 12]);
        assert_eq!(header.logits, vec![10, 20, -5, 99]);

        // Verify fact state, previous version, and non-memory prose count match byte-for-byte.
        let read_after =
            restarted_memory.read_fact(&restarted_store, &pinned, &scope, "port", "value")?;
        match read_after {
            FactReadOutcome::Quoted {
                value_bytes,
                version,
                action,
                previous_value_bytes,
                ..
            } => {
                assert_eq!(value_bytes, b"9080");
                assert_eq!(version, 2);
                assert_eq!(action, FactAction::Revise);
                assert_eq!(previous_value_bytes.as_deref(), Some(&b"8080"[..]));
            }
            other => panic!("expected Quoted after restart, got {other:?}"),
        }
        assert_eq!(restarted_memory.non_memory_turn_count(&scope), 1);

        // Re-exporting from the restored store produces the exact same Manifest & ObjectVersion.
        let re_exported = restarted_memory.export_scope(&mut restarted_store, &scope)?;
        assert_eq!(re_exported.version(), exported_manifest.version());
        assert_eq!(re_exported.encode(), exported_manifest.encode());
        Ok(())
    })();

    let _ = std::fs::remove_dir_all(&dir);
    result.expect("restart and restore control passed");
}

#[test]
fn retracted_or_unstated_fact_refused_rather_than_guessed() {
    let tok = fixture_tokenizer();
    let pinned = PinnedTokenizer::new(&tok).expect("pinned tokenizer");
    let mut store = Store::new(Limits::default());
    let mut memory = DurableObjectMemoryStore::new();
    let scope = IdentityScope::new("user-abstain", "proj-m3", "sess-1").expect("scope");

    memory
        .assert_fact(
            &mut store,
            &pinned,
            &scope,
            "Alice",
            "city",
            b"Alice Lisbon",
            6..12,
        )
        .expect("assert Alice city");

    // Counting backend to prove unstated and retracted queries perform zero store fetches.
    struct CountingBackend<'a> {
        inner: &'a Store,
        fetches: AtomicUsize,
    }
    impl RecordBackend for CountingBackend<'_> {
        fn fetch(&self, id: NodeId) -> uor_r4_object_store::Result<Option<Record>> {
            self.fetches.fetch_add(1, Ordering::Relaxed);
            self.inner.fetch(id)
        }
    }
    let counting = CountingBackend {
        inner: &store,
        fetches: AtomicUsize::new(0),
    };

    // Unstated entity or relation is refused with zero backend reads.
    let unstated = memory
        .read_fact_from(&counting, store.limits(), &pinned, &scope, "port", "value")
        .expect("unstated query");
    assert_eq!(unstated, FactReadOutcome::RefusedUnstated);
    assert_eq!(counting.fetches.load(Ordering::Relaxed), 0);

    // Explicit retraction marks the slot retracted and refuses subsequent reads without guessing.
    let retract_ver = memory
        .retract_fact(&scope, "Alice", "city")
        .expect("retract Alice city");
    assert_eq!(retract_ver, 2);

    let retracted = memory
        .read_fact_from(&counting, store.limits(), &pinned, &scope, "Alice", "city")
        .expect("retracted query");
    assert_eq!(
        retracted,
        FactReadOutcome::RefusedRetracted {
            owner: "Alice".to_string(),
            relation: "city".to_string(),
            retracted_at_version: 2,
        }
    );
    assert_eq!(counting.fetches.load(Ordering::Relaxed), 0);
}

#[test]
fn non_memory_prose_never_silently_overwrites_stored_facts() {
    let tok = fixture_tokenizer();
    let pinned = PinnedTokenizer::new(&tok).expect("pinned tokenizer");
    let mut store = Store::new(Limits::default());
    let mut memory = DurableObjectMemoryStore::new();
    let scope = IdentityScope::new("user-prose", "proj-m3", "sess-1718").expect("scope");

    memory
        .assert_fact(
            &mut store,
            &pinned,
            &scope,
            "Alice",
            "city",
            b"Alice Lisbon",
            6..12,
        )
        .expect("assert fact");
    memory
        .assert_fact(
            &mut store,
            &pinned,
            &scope,
            "port",
            "value",
            b"port 8080",
            5..9,
        )
        .expect("assert port");

    // Ingest non-memory prose turns that mention the same entity tokens and distractors (#1718).
    let prose_1 = memory
        .ingest_non_memory_prose(&mut store, &pinned, &scope, b"Alice Bono port 9080")
        .expect("ingest prose 1");
    let prose_2 = memory
        .ingest_non_memory_prose(&mut store, &pinned, &scope, b"Alice Bobo port 8090")
        .expect("ingest prose 2");
    assert_eq!(
        store.rebuild(&prose_1).expect("rebuild prose 1").bytes(),
        b"Alice Bono port 9080"
    );
    assert_eq!(
        store.rebuild(&prose_2).expect("rebuild prose 2").bytes(),
        b"Alice Bobo port 8090"
    );
    assert_eq!(memory.non_memory_turn_count(&scope), 2);

    // Stored facts remain untouched at version 1 with zero conflict.
    match memory
        .read_fact(&store, &pinned, &scope, "Alice", "city")
        .expect("read Alice")
    {
        FactReadOutcome::Quoted {
            value_bytes,
            version,
            conflict,
            ..
        } => {
            assert_eq!(value_bytes, b"Lisbon");
            assert_eq!(version, 1);
            assert!(!conflict);
        }
        other => panic!("expected Quoted Lisbon, got {other:?}"),
    }
    match memory
        .read_fact(&store, &pinned, &scope, "port", "value")
        .expect("read port")
    {
        FactReadOutcome::Quoted {
            value_bytes,
            version,
            conflict,
            ..
        } => {
            assert_eq!(value_bytes, b"8080");
            assert_eq!(version, 1);
            assert!(!conflict);
        }
        other => panic!("expected Quoted 8080, got {other:?}"),
    }
}

#[test]
fn explicit_reset_and_strict_identity_scope_isolation() {
    assert!(IdentityScope::new("", "proj", "sess").is_err());
    assert!(IdentityScope::new("user", "  ", "sess").is_err());
    assert!(IdentityScope::new("user", "proj", "").is_err());

    let tok = fixture_tokenizer();
    let pinned = PinnedTokenizer::new(&tok).expect("pinned tokenizer");
    let mut store = Store::new(Limits::default());
    let mut memory = DurableObjectMemoryStore::new();

    let scope_a = IdentityScope::new("user-a", "proj-1", "sess-1").expect("scope a");
    let scope_b = IdentityScope::new("user-b", "proj-1", "sess-1").expect("scope b");

    memory
        .assert_fact(
            &mut store,
            &pinned,
            &scope_a,
            "Alice",
            "city",
            b"Alice Lisbon",
            6..12,
        )
        .expect("assert in scope a");
    memory
        .assert_fact(
            &mut store,
            &pinned,
            &scope_b,
            "port",
            "value",
            b"port 8080",
            5..9,
        )
        .expect("assert in scope b");
    memory
        .set_ur4sess1_snapshot(&scope_a, &sample_ur4sess1_bytes(2, &[0, 1], &[5, 6]))
        .expect("set snapshot a");

    // Scope B cannot read Scope A's fact, and vice versa.
    assert_eq!(
        memory
            .read_fact(&store, &pinned, &scope_b, "Alice", "city")
            .expect("cross-scope read"),
        FactReadOutcome::RefusedUnstated
    );
    assert_eq!(
        memory
            .read_fact(&store, &pinned, &scope_a, "port", "value")
            .expect("cross-scope read"),
        FactReadOutcome::RefusedUnstated
    );

    // Explicit reset of Scope A wipes its facts, pronoun antecedent, and UR4SESS1 snapshot.
    memory.reset_scope(&scope_a).expect("reset scope a");
    assert_eq!(
        memory
            .read_fact(&store, &pinned, &scope_a, "Alice", "city")
            .expect("post-reset read"),
        FactReadOutcome::RefusedUnstated
    );
    assert!(memory.ur4sess1_snapshot(&scope_a).is_none());

    // Scope B remains intact.
    match memory
        .read_fact(&store, &pinned, &scope_b, "port", "value")
        .expect("scope b intact")
    {
        FactReadOutcome::Quoted { value_bytes, .. } => assert_eq!(value_bytes, b"8080"),
        other => panic!("expected Scope B fact intact, got {other:?}"),
    }
}

#[test]
fn hostile_or_corrupt_record_and_envelope_refused_fail_closed() {
    let tok = fixture_tokenizer();
    let pinned = PinnedTokenizer::new(&tok).expect("pinned tokenizer");
    let mut store = Store::new(Limits::default());
    let mut memory = DurableObjectMemoryStore::new();
    let scope = IdentityScope::new("user-sec", "proj-m3", "sess-sec").expect("scope");

    memory
        .assert_fact(
            &mut store,
            &pinned,
            &scope,
            "port",
            "value",
            b"port 8080",
            5..9,
        )
        .expect("assert port");
    let valid_sess = sample_ur4sess1_bytes(2, &[0, 5], &[1, 2]);
    memory
        .set_ur4sess1_snapshot(&scope, &valid_sess)
        .expect("set valid ur4sess1");

    // 1. Corrupt UR4SESS1 magic or trailing bytes or huge varint is rejected before storage.
    let mut bad_sess_magic = valid_sess.clone();
    bad_sess_magic[0] ^= 0xff;
    assert_eq!(
        memory.set_ur4sess1_snapshot(&scope, &bad_sess_magic),
        Err(Error::SchemaMismatch)
    );
    let mut bad_sess_trailing = valid_sess.clone();
    bad_sess_trailing.push(0x99);
    assert_eq!(
        memory.set_ur4sess1_snapshot(&scope, &bad_sess_trailing),
        Err(Error::SchemaMismatch)
    );

    // 2. Export valid envelope and test corruptions of the envelope in the store.
    let manifest = memory
        .export_scope(&mut store, &scope)
        .expect("export valid scope");
    let raw_envelope = store
        .rebuild(&manifest)
        .expect("rebuild envelope")
        .bytes()
        .to_vec();
    assert_eq!(&raw_envelope[..8], DURABLE_CONSUMER_MAGIC);

    // Tampered envelope magic folded as a hostile object is refused on import_scope.
    let mut bad_env_magic = raw_envelope.clone();
    bad_env_magic[0] ^= 1;
    let bad_folded = store.fold(&bad_env_magic).expect("fold hostile bytes");
    let mut target_memory = DurableObjectMemoryStore::new();
    assert_eq!(
        target_memory.import_scope(&store, &bad_folded.manifest),
        Err(Error::SchemaMismatch)
    );
    assert!(target_memory.scopes().is_empty());

    // Truncated envelope folded as a hostile object is refused on import_scope.
    let truncated_folded = store
        .fold(&raw_envelope[..raw_envelope.len() - 3])
        .expect("fold truncated");
    assert!(target_memory
        .import_scope(&store, &truncated_folded.manifest)
        .is_err());
    assert!(target_memory.scopes().is_empty());

    // Forged manifest with wrong whole-object SHA-256 is refused with IntegrityFailure.
    let mut forged_manifest_wire = manifest.encode();
    forged_manifest_wire[20] ^= 0x40;
    let forged_manifest =
        Manifest::decode(&forged_manifest_wire, store.limits()).expect("decode forged manifest");
    assert_eq!(
        target_memory.import_scope(&store, &forged_manifest),
        Err(Error::IntegrityFailure)
    );

    // Expected-scope verification refuses a valid envelope belonging to a different IdentityScope.
    let wrong_scope = IdentityScope::new("attacker", "proj-m3", "sess-sec").expect("wrong scope");
    assert_eq!(
        target_memory.import_scope_expecting(&store, &manifest, &wrong_scope),
        Err(Error::BindingMismatch)
    );
    assert!(target_memory.scopes().is_empty());
}
