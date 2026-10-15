//! Byte/data-structure known answers from the owner-supplied Showcase archive.
//! No tokenization, model, reference model, GPU or learning is executed here.
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use uor_r4_object_store::{Limits, Manifest, NodeId, Record, Store};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

const VECTORS: &[u8] = include_bytes!("fixtures/python_vectors.json");
const CONSTRUCTION: &[u8] = include_bytes!("fixtures/construction.json");

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn sha256(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

fn records(bytes: &[u8]) -> TestResult<BTreeMap<NodeId, Record>> {
    let mut parsed = BTreeMap::new();
    let mut offset = 0;
    let mut last = None;
    while offset < bytes.len() {
        let length = if bytes[offset] == 0 { 45 } else { 49 };
        let wire = bytes
            .get(offset..offset + length)
            .ok_or("truncated export")?;
        let record = Record::decode(wire)?;
        assert_eq!(record.encode(), wire);
        let id = record.id();
        if let Some(previous) = last {
            assert!(previous < id, "export must be strictly identity-sorted");
        }
        last = Some(id);
        assert!(parsed.insert(id, record).is_none());
        offset += length;
    }
    Ok(parsed)
}

fn original_record_order(
    manifest: &Manifest,
    parsed: &BTreeMap<NodeId, Record>,
) -> TestResult<Vec<u8>> {
    fn visit(
        id: NodeId,
        scale: u8,
        parsed: &BTreeMap<NodeId, Record>,
        seen: &mut BTreeSet<NodeId>,
        output: &mut Vec<u8>,
    ) -> TestResult {
        let record = parsed.get(&id).ok_or("missing record")?;
        assert_eq!(record.scale(), scale);
        if !seen.insert(id) {
            return Ok(());
        }
        if scale > 0 {
            assert_eq!(record.payload().len(), 36);
            for child in record.payload().chunks_exact(12) {
                visit(child.try_into()?, scale - 1, parsed, seen, output)?;
            }
        }
        output.extend_from_slice(&record.encode());
        Ok(())
    }
    let mut seen = BTreeSet::new();
    let mut output = Vec::new();
    for &(id, scale) in manifest.roots() {
        visit(id, scale, parsed, &mut seen, &mut output)?;
    }
    assert_eq!(seen.len(), parsed.len(), "fresh fold exports only this DAG");
    Ok(output)
}

fn original_input(name: &str) -> TestResult<Vec<u8>> {
    Ok(match name {
        "unique" => (0..1024)
            .flat_map(|index| Sha256::digest(format!("CS_RANGE_ATOM_{index}").as_bytes()))
            .collect(),
        "repeated" => (0..243)
            .flat_map(|index| [b'A' + (index % 3) as u8; 32])
            .collect(),
        "utf8" => format!(
            "{}🌀{}音楽{}",
            "a".repeat(31),
            "b".repeat(28),
            "é\0tail\0".repeat(67)
        )
        .into_bytes(),
        "empty" => Vec::new(),
        _ => return Err("unknown original fixture name".into()),
    })
}

#[test]
fn retained_known_answers_are_hash_pinned() -> TestResult {
    assert_eq!(
        sha256(VECTORS),
        "0930044db928f0389dfa35e603812e4c765e0c8c7ced3182f6616adf31299031"
    );
    assert_eq!(
        sha256(CONSTRUCTION),
        "644351faf3eb22b91f742bf30bcdb650793d2ef477ddf4f5c0b2669e43c1bcce"
    );
    let reference: Value = serde_json::from_slice(CONSTRUCTION)?;
    assert_eq!(
        reference["sourceSha256"],
        "7ca3dcfb552ca6ff3fc8696734265bcd9b993aa8b8e6242984060ace8b35bc22"
    );
    assert_eq!(
        reference["transportSha256"],
        "1d254a75dfe497f91b302ada0d581eab3407c7ea8f3994920a58d9a2652dc0fe"
    );
    Ok(())
}

#[test]
fn all_four_original_fixture_records_match_the_historical_wire_hashes() -> TestResult {
    let reference: Value = serde_json::from_slice(CONSTRUCTION)?;
    let cases = reference["cases"].as_array().ok_or("missing cases")?;
    assert_eq!(cases.len(), 4);
    for case in cases {
        let name = case["name"].as_str().ok_or("missing name")?;
        let bytes = original_input(name)?;
        assert_eq!(sha256(&bytes), case["inputSha256"], "{name}");
        assert_eq!(bytes.len() as u64, case["inputBytes"], "{name}");
        let mut store = Store::new(Limits::default());
        let folded = store.fold(&bytes)?;
        assert_eq!(folded.manifest.byte_len(), bytes.len() as u64);
        assert_eq!(hex(&folded.manifest.whole_sha256()), case["inputSha256"]);
        assert_eq!(
            folded.manifest.roots().len() as u64,
            case["rootOccurrences"],
            "{name}"
        );
        assert_eq!(store.rebuild(&folded.manifest)?.bytes(), bytes, "{name}");
        let exported = store.export_records()?;
        let parsed = records(&exported)?;
        assert_eq!(parsed.len() as u64, case["uniqueNodes"], "{name}");
        assert_eq!(exported.len() as u64, case["recordBytes"], "{name}");
        let original_order = original_record_order(&folded.manifest, &parsed)?;
        assert_eq!(sha256(&original_order), case["recordSha256"], "{name}");

        // Sender/receiver reconstruction also accepts the original wire order.
        let mut receiver = Store::new(Limits::default());
        receiver.import_records(&original_order)?;
        assert_eq!(receiver.rebuild(&folded.manifest)?.bytes(), bytes, "{name}");
        assert_eq!(receiver.export_records()?, exported, "{name}");
        receiver.import_records(&original_order)?;
        assert_eq!(receiver.export_records()?, exported, "idempotence: {name}");
        let mut independent_fold = Store::new(Limits::default());
        independent_fold.fold(&bytes)?;
        assert_eq!(independent_fold.export_records()?, exported, "{name}");
    }
    Ok(())
}

fn assert_fold_known_answer(bytes: &[u8], vector: &Value) -> TestResult {
    assert_eq!(sha256(bytes), vector["sha256"]);
    let mut store = Store::new(Limits::default());
    let folded = store.fold(bytes)?;
    let roots: Vec<String> = folded
        .manifest
        .roots()
        .iter()
        .map(|(id, _)| hex(id))
        .collect();
    assert_eq!(serde_json::to_value(roots)?, vector["roots"]);
    assert_eq!(hex(&folded.manifest.whole_sha256()), vector["sha256"]);
    assert_eq!(folded.manifest.byte_len(), bytes.len() as u64);
    assert_eq!(
        records(&store.export_records()?)?.len() as u64,
        vector["uniqueNodes"]
    );
    assert_eq!(store.rebuild(&folded.manifest)?.bytes(), bytes);
    Ok(())
}

#[test]
fn supplied_fold_known_answers_cover_padding_and_ternary_carries() -> TestResult {
    let vectors: Value = serde_json::from_slice(VECTORS)?;
    let cases = vectors["fold"].as_array().ok_or("missing fold vectors")?;
    assert_eq!(cases.len(), 15);
    for vector in cases {
        let size = vector["size"].as_u64().ok_or("missing size")? as usize;
        let bytes: Vec<u8> = (0..size).map(|index| (index % 256) as u8).collect();
        assert_fold_known_answer(&bytes, vector)?;
    }
    Ok(())
}

#[test]
fn supplied_reference_and_novel_mebibyte_roots_match() -> TestResult {
    let vectors: Value = serde_json::from_slice(VECTORS)?;
    let cases = vectors["fixtures"].as_array().ok_or("missing fixtures")?;
    assert_eq!(cases.len(), 2);
    let pool: Vec<[u8; 32]> = (0..512)
        .map(|index| Sha256::digest(format!("SPIRALCORE_ATOM_{index}").as_bytes()).into())
        .collect();
    for vector in cases {
        let kind = vector["kind"].as_str().ok_or("missing fixture kind")?;
        let mut bytes = Vec::with_capacity(1_048_576);
        for index in 0..32768 {
            match kind {
                "reference" => bytes.extend_from_slice(&pool[(index * 131 + index / 27) % 512]),
                "novel" => bytes
                    .extend_from_slice(&Sha256::digest(format!("NOVEL_BLOCK_{index}").as_bytes())),
                _ => return Err("unknown fixture kind".into()),
            }
        }
        assert_fold_known_answer(&bytes, vector)?;
    }
    Ok(())
}

#[test]
fn blake2b_96_record_identity_matches_supplied_leaf_and_parent_answers() -> TestResult {
    // These IDs are retained in the supplied 32-byte and 96-byte fold vectors.
    // They pin BLAKE2b configured for 12 output bytes, not truncated BLAKE2b-512.
    let mut store = Store::new(Limits::default());
    let bytes: Vec<u8> = (0..96).collect();
    let folded = store.fold(&bytes)?;
    assert_eq!(folded.manifest.roots().len(), 1);
    let (parent_id, parent_scale) = folded.manifest.roots()[0];
    assert_eq!(hex(&parent_id), "042381f84352bd615419c232");
    assert_eq!(parent_scale, 1);
    let parsed = records(&store.export_records()?)?;
    let parent = parsed.get(&parent_id).ok_or("missing parent")?;
    let first_child: NodeId = parent.payload()[..12].try_into()?;
    assert_eq!(hex(&first_child), "3fa41fa2442344c326742351");
    let leaf = parsed.get(&first_child).ok_or("missing leaf")?;
    assert_eq!(leaf.scale(), 0);
    assert_eq!(leaf.payload(), &bytes[..32]);
    let mut corrupted_leaf = leaf.encode();
    corrupted_leaf[13] ^= 1;
    assert!(Record::decode(&corrupted_leaf).is_err());
    let mut corrupted_parent = parent.encode();
    corrupted_parent[13] ^= 1;
    assert!(Record::decode(&corrupted_parent).is_err());
    Ok(())
}
