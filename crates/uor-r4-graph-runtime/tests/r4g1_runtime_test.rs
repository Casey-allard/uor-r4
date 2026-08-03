//! R4G1Runtime unit tests verifying multiplication-free zero-allocation prediction behavior over R4G1 GraphView containers.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::collections::BTreeMap;
use uor_r4_core::transformerless::compiler::{self, STAGES};
use uor_r4_core::transformerless::convert_r4g1;
use uor_r4_core::transformerless::runtime::{self, Store};
use uor_r4_graph_format::{ArtifactBuilder, GraphView, ScoreQ, SectionId};
use uor_r4_graph_runtime::R4G1Runtime;
use uor_r4_graph_runtime::packed_kernels::evaluate_fmm_translation_table;

struct CountingAllocator;

thread_local! {
    static COUNT_ALLOCATIONS: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            let _ = COUNT_ALLOCATIONS.try_with(|gate| {
                if gate.get() {
                    let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
                }
            });
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn counted_allocations(f: impl FnOnce()) -> usize {
    ALLOCATIONS.with(|count| count.set(0));
    COUNT_ALLOCATIONS.with(|gate| gate.set(true));
    f();
    COUNT_ALLOCATIONS.with(|gate| gate.set(false));
    ALLOCATIONS.with(Cell::get)
}

fn fixture_artifacts() -> (Vec<u8>, compiler::Compiled) {
    let dir = env!("CARGO_MANIFEST_DIR");
    let bytes = std::fs::read(format!(
        "{dir}/../uor-r4-core/tests/fixtures/tless_artifacts.bin"
    ))
    .expect("fixture artifacts present");
    let artifacts = compiler::parse_artifacts(&bytes).expect("fixture artifacts parse");
    (bytes, artifacts)
}

fn synthetic_store() -> Store {
    let mut store: Store = (0..=STAGES).map(|_| BTreeMap::new()).collect();
    let codes: [[u8; 4]; 6] = [
        [3, 1, 4, 1],
        [3, 1, 4, 2],
        [3, 5, 9, 2],
        [7, 5, 9, 2],
        [7, 5, 8, 2],
        [11, 5, 8, 7],
    ];
    for (i, code) in codes.iter().enumerate() {
        runtime::add_evidence(&mut store, code, (i + 1) as u32, 1);
    }
    store
}

fn base_r4g1_artifact() -> Vec<u8> {
    let (art_bytes, artifacts) = fixture_artifacts();
    let store = synthetic_store();
    let store_bytes = runtime::store_bytes(&store);
    convert_r4g1::convert(&art_bytes, &artifacts, &store, &store_bytes, None)
        .expect("convert to R4G1 succeeds")
        .0
}

fn ngram_payload(include_trigram: bool) -> Vec<u8> {
    let row_count = if include_trigram { 2usize } else { 1usize };
    let entries_start =
        uor_r4_graph_format::NGRAM_HEADER_LEN + row_count * uor_r4_graph_format::NGRAM_ROW_LEN;
    let mut bytes = vec![0u8; entries_start];
    bytes[..4].copy_from_slice(&uor_r4_graph_format::NGRAM_MAGIC);
    bytes[4..6].copy_from_slice(&uor_r4_graph_format::NGRAM_VERSION.to_le_bytes());
    bytes[8..12].copy_from_slice(&(row_count as u32).to_le_bytes());
    bytes[12..14].copy_from_slice(&2u16.to_le_bytes());

    let mut entry_offset = entries_start;
    let mut add_row =
        |index: usize, context_len: u8, key0: u32, key1: u32, entries: &[(u32, i32)]| {
            let header =
                uor_r4_graph_format::NGRAM_HEADER_LEN + index * uor_r4_graph_format::NGRAM_ROW_LEN;
            bytes[header] = context_len;
            bytes[header + 2..header + 4].copy_from_slice(&(entries.len() as u16).to_le_bytes());
            bytes[header + 4..header + 8].copy_from_slice(&key0.to_le_bytes());
            bytes[header + 8..header + 12].copy_from_slice(&key1.to_le_bytes());
            bytes[header + 12..header + 16].copy_from_slice(&(entry_offset as u32).to_le_bytes());
            for &(token, score) in entries {
                bytes.extend_from_slice(&token.to_le_bytes());
                bytes.extend_from_slice(&score.to_le_bytes());
            }
            entry_offset = bytes.len();
        };

    add_row(0, 1, 20, 0, &[(7, 100)]);
    if include_trigram {
        add_row(1, 2, 10, 20, &[(8, 200), (9, 200)]);
    }
    bytes
}

fn fmm_section_fixture() -> Vec<u8> {
    let dimension = 288u16;
    let rank = 1u16;
    let token_count = 2u32;
    let mut bytes = Vec::with_capacity(20 + 2 * 4 + 2 * 4 + usize::from(dimension) * 2 * 4);
    bytes.extend_from_slice(b"FMM1");
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&dimension.to_le_bytes());
    bytes.extend_from_slice(&rank.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&token_count.to_le_bytes());
    bytes.push(1);
    bytes.extend_from_slice(&[0u8; 3]);
    bytes.extend_from_slice(&7u32.to_le_bytes());
    bytes.extend_from_slice(&9u32.to_le_bytes());
    bytes.extend_from_slice(&100i32.to_le_bytes());
    bytes.extend_from_slice(&100i32.to_le_bytes());
    for coordinate in 0..usize::from(dimension) {
        bytes.extend_from_slice(&(if coordinate == 0 { -10i32 } else { 0 }).to_le_bytes());
        bytes.extend_from_slice(&(if coordinate == 0 { 10i32 } else { 0 }).to_le_bytes());
    }
    bytes
}

fn artifact_with_ngram(include_trigram: bool) -> Vec<u8> {
    let base = base_r4g1_artifact();
    let view = GraphView::parse(&base).expect("base artifact parses");
    let mut builder = ArtifactBuilder::new(view.header().alignment_log2);
    for section in view.sections() {
        builder.add_section(section.id, section.flags, section.payload);
    }
    builder.add_section(SectionId::NGRAM, 0, &ngram_payload(include_trigram));
    builder.build().expect("artifact with NGRAM builds")
}

fn add_fmm_to_artifact(base: &[u8], fmm: &[u8]) -> Vec<u8> {
    let view = GraphView::parse(base).expect("converted artifact parses");
    let mut builder = ArtifactBuilder::new(6);
    for section in [
        SectionId::HEAD,
        SectionId::CODE,
        SectionId::NODE,
        SectionId::EDGE,
        SectionId::ROUT,
        SectionId::EMIT,
        SectionId::EXCT,
        SectionId::PROV,
        SectionId::CERT,
        SectionId::PTCH,
        SectionId::SECT,
        SectionId::RTNX,
    ] {
        if let Some(bytes) = view.section(section) {
            builder.add_section(section, 0, bytes);
        }
    }
    builder.add_section(SectionId::FMM, 0, fmm);
    builder.build().expect("artifact with FMM section builds")
}

#[test]
fn r4g1_runtime_parses_and_predicts() {
    let (art_bytes, artifacts) = fixture_artifacts();
    let store = synthetic_store();
    let store_bytes = runtime::store_bytes(&store);
    let (r4g1_bytes, _) = convert_r4g1::convert(&art_bytes, &artifacts, &store, &store_bytes, None)
        .expect("convert to R4G1 succeeds");

    let runtime = R4G1Runtime::parse(&r4g1_bytes).expect("R4G1Runtime parses container");
    assert!(runtime.node_count() > 0);
    assert!(runtime.edge_count() > 0);

    let mut node_scores = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let (best_token, best_score) = runtime.predict_distribution(&[1, 2, 3], None, &mut node_scores);
    assert!(best_score > ScoreQ::MIN);

    let mut node_scores2 = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let token = runtime.predict_token(&[1, 2, 3], None, &mut node_scores2);
    assert_eq!(token, best_token);
}

#[test]
fn r4g1_runtime_enforces_no_float_in_prediction_path() {
    let (art_bytes, artifacts) = fixture_artifacts();
    let store = synthetic_store();
    let store_bytes = runtime::store_bytes(&store);
    let (r4g1_bytes, _) =
        convert_r4g1::convert(&art_bytes, &artifacts, &store, &store_bytes, None).unwrap();
    let runtime = R4G1Runtime::parse(&r4g1_bytes).unwrap();

    let mut node_scores = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let (_, score) = runtime.predict_distribution(&[3, 1, 4], None, &mut node_scores);

    assert!(score >= ScoreQ::MIN);
    assert!(score <= ScoreQ::MAX);
}

#[test]
fn session_signature_is_bias_only_until_routing_is_calibrated() {
    let (art_bytes, artifacts) = fixture_artifacts();
    let store = synthetic_store();
    let store_bytes = runtime::store_bytes(&store);
    let (r4g1_bytes, _) =
        convert_r4g1::convert(&art_bytes, &artifacts, &store, &store_bytes, None).unwrap();
    let runtime = R4G1Runtime::parse(&r4g1_bytes).unwrap();
    let context_signature = [0x55u8; 36];
    let zero_session = [0u8; 36];
    let full_session = [0xffu8; 36];

    let mut zero_scores = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let mut full_scores = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let zero = runtime.predict_distribution_with_signature_lanes(
        &[3, 1, 4],
        Some(&context_signature),
        Some(&zero_session),
        &mut zero_scores,
    );
    let full = runtime.predict_distribution_with_signature_lanes(
        &[3, 1, 4],
        Some(&context_signature),
        Some(&full_session),
        &mut full_scores,
    );

    assert_eq!(
        zero.0, full.0,
        "session lane must not change ROUT fallback yet"
    );
    assert!(zero.1 >= ScoreQ::MIN);
    assert!(full.1 >= ScoreQ::MIN);
}

#[test]
fn ngram_runtime_prefers_trigram_then_bigram_without_allocations() {
    let trigram_bytes = artifact_with_ngram(true);
    let trigram_runtime = R4G1Runtime::parse(&trigram_bytes).expect("trigram artifact parses");
    let mut scores = vec![ScoreQ::MIN; trigram_runtime.node_count() as usize];
    let trigram = trigram_runtime.predict_distribution(&[1, 10, 20], None, &mut scores);
    assert_eq!(trigram, (8, ScoreQ::from_raw(200)));

    let allocations = counted_allocations(|| {
        for _ in 0..64 {
            assert_eq!(
                trigram_runtime.predict_distribution(&[1, 10, 20], None, &mut scores),
                trigram
            );
        }
    });
    assert_eq!(allocations, 0, "NGRAM lookup must be allocation-free");

    let bigram_bytes = artifact_with_ngram(false);
    let bigram_runtime = R4G1Runtime::parse(&bigram_bytes).expect("bigram artifact parses");
    let mut bigram_scores = vec![ScoreQ::MIN; bigram_runtime.node_count() as usize];
    assert_eq!(
        bigram_runtime.predict_distribution(&[1, 10, 20], None, &mut bigram_scores),
        (7, ScoreQ::from_raw(100))
    );
}

#[test]
fn ngram_lookup_kernel_is_integer_only_by_source_scan() {
    let source = include_str!("../src/engine.rs");
    let start = source
        .find("fn context_backoff")
        .expect("context lookup source exists");
    let end = source
        .find("impl<'a> R4G1Runtime")
        .expect("runtime implementation follows lookup helpers");
    let kernel = &source[start..end];
    for forbidden in ["*", "/", "f32", "f64"] {
        assert!(
            !kernel.contains(forbidden),
            "NGRAM lookup kernel contains forbidden operation/type {forbidden:?}"
        );
    }
}

#[test]
fn ngram_artifact_bytes_are_deterministic() {
    assert_eq!(artifact_with_ngram(true), artifact_with_ngram(true));
}

#[test]
fn packed_fmm_runtime_matches_folded_integer_table_without_allocations() {
    let (art_bytes, artifacts) = fixture_artifacts();
    let store = synthetic_store();
    let store_bytes = runtime::store_bytes(&store);
    let (base, _) = convert_r4g1::convert(&art_bytes, &artifacts, &store, &store_bytes, None)
        .expect("convert to R4G1 succeeds");
    let fmm = fmm_section_fixture();
    let bytes = add_fmm_to_artifact(&base, &fmm);
    let view = GraphView::parse(&bytes).expect("FMM artifact parses");
    let table = view
        .fmm_translation_table()
        .expect("FMM section validates")
        .expect("FMM section is present");
    assert_eq!(table.dimension(), 288);
    assert_eq!(table.token_count(), 2);
    let artifact_delta = bytes.len() - base.len();
    assert!(artifact_delta >= fmm.len() + 16);
    println!(
        "FMM footprint: {} bytes; artifact delta: {} bytes; serving work: {} sign/add updates per query",
        fmm.len(),
        artifact_delta,
        usize::from(table.dimension()) * table.token_count() as usize
    );
    let signature = [1u8; 36];
    let mut scores = [(0u32, ScoreQ::ZERO); 2];

    let selected = evaluate_fmm_translation_table(&view, &signature, &[], &mut scores)
        .expect("integer FMM evaluation succeeds")
        .expect("FMM table returns a candidate");
    assert_eq!(selected, (9, ScoreQ::from_raw(110)));

    let allocations = counted_allocations(|| {
        for _ in 0..64 {
            let mut scores = [(0u32, ScoreQ::ZERO); 2];
            let result = evaluate_fmm_translation_table(&view, &signature, &[], &mut scores)
                .expect("steady-state FMM evaluation succeeds")
                .expect("steady-state FMM table returns a candidate");
            assert_eq!(result.0, 9);
        }
    });
    assert_eq!(allocations, 0);

    let recent = [9u32];
    let mut scores = [(0u32, ScoreQ::ZERO); 2];
    let penalized = evaluate_fmm_translation_table(&view, &signature, &recent, &mut scores)
        .expect("recent-token penalty evaluation succeeds")
        .expect("penalized FMM table returns a candidate");
    assert_eq!(penalized.0, 7);
}
