//! #933 — serving witness authority is byte-bound and independently replayed.

use std::collections::BTreeMap;

use uor_r4_api::{
    parse_and_validate_normative_witness_replay, produce_normative_witness_replay, EngineParts,
    NormativeServingDecision, NormativeServingEngine, NormativeWitnessCandidateSource,
    NormativeWitnessReplayMaterial, NormativeWitnessReplaySpec, NormativeWitnessReplayVerdict,
    NORMATIVE_WITNESS_REPLAY_SCHEMA,
};
use uor_r4_core::transformerless::compiler::{self, STAGES};
use uor_r4_core::transformerless::{convert_r4g1, runtime};
use uor_r4_graph_compiler::induction;
use uor_r4_graph_format::{build_skipmix_table, ArtifactBuilder, GraphView, SectionId};

fn synthetic_bundle() -> (Vec<u8>, Vec<u8>) {
    let artifact_bytes = std::fs::read(format!(
        "{}/../uor-r4-core/tests/fixtures/tless_artifacts.bin",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("teacher artifact fixture");
    let artifacts = compiler::parse_artifacts(&artifact_bytes).expect("teacher artifact parses");
    let mut store: runtime::Store = (0..=STAGES).map(|_| BTreeMap::new()).collect();
    let codes: [[u8; 4]; 6] = [
        [3, 1, 4, 1],
        [3, 1, 4, 2],
        [3, 5, 9, 2],
        [7, 5, 9, 2],
        [7, 5, 8, 2],
        [11, 5, 8, 7],
    ];
    for (index, code) in codes.iter().enumerate() {
        runtime::add_evidence(&mut store, code, (index + 1) as u32, 1);
    }
    let store_bytes = runtime::store_bytes(&store);
    let graph = convert_r4g1::convert(&artifact_bytes, &artifacts, &store, &store_bytes, None)
        .expect("convert synthetic graph")
        .0;
    (graph, artifact_bytes)
}

fn corpus_bytes() -> (Vec<u8>, Vec<u8>) {
    let next = [3u16, 1, 4, 3, 1, 4, 7, 5, 8, 3, 1, 4, 7, 5, 8, 3];
    let mut meta = Vec::with_capacity(25);
    meta.extend_from_slice(&(next.len() as u64).to_le_bytes());
    meta.extend_from_slice(&1u64.to_le_bytes());
    meta.extend_from_slice(&0u64.to_le_bytes());
    meta.push(1);

    let mut records = Vec::with_capacity(next.len() * 12);
    for token in next {
        records.extend_from_slice(&0u32.to_le_bytes());
        records.extend_from_slice(&token.to_le_bytes());
        records.extend_from_slice(&token.to_le_bytes());
        records.extend_from_slice(&(-0.1f32).to_le_bytes());
    }
    (meta, records)
}

fn tokenizer_bytes(tokens: &[&[u8]]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for token in tokens {
        bytes.extend_from_slice(&(token.len() as i32).to_le_bytes());
        bytes.extend_from_slice(token);
    }
    bytes
}

fn graph_with_planted_runtime_winner(
    base: &[u8],
    teacher: &[u8],
    tokenizer: &[u8],
    corpus_meta: &[u8],
    corpus_records: &[u8],
) -> (Vec<u8>, u64, u32) {
    let corpus = compiler::load_corpus_bytes(corpus_meta, corpus_records, None)
        .expect("synthetic corpus parses");
    let mut engine = NormativeServingEngine::load_for_research(EngineParts {
        graph: base,
        signature_artifact: teacher,
        tokenizer: Some(tokenizer),
        score_report: None,
    })
    .expect("base serving engine loads");

    for position in 0..corpus.n {
        engine.reset_policy_state();
        let window = induction::context_window(&corpus, position);
        let decision = engine.predict(&window).expect("base decision");
        let NormativeServingDecision::Serve(outcome) = decision else {
            continue;
        };
        let Some(last_token) = window.last().copied() else {
            continue;
        };
        let Some(content_token) = window.iter().copied().find(|token| *token != last_token) else {
            continue;
        };
        let vocab_size = GraphView::parse(base)
            .expect("base graph parses")
            .head()
            .map_or(49_152, |head| head.vocab_size());
        let partner = (42..vocab_size)
            .find(|token| {
                outcome
                    .candidates
                    .ranked()
                    .iter()
                    .all(|candidate| candidate.token != *token)
            })
            .expect("unused planted partner");
        let skmx = build_skipmix_table(&[(content_token, last_token, vec![(partner, 2_000_000)])])
            .expect("planted SKMX row");
        let view = GraphView::parse(base).expect("base graph parses");
        let mut builder = ArtifactBuilder::new(view.header().alignment_log2);
        for section in view.sections() {
            assert_ne!(section.id, SectionId::SKMX);
            if section.id == SectionId::HEAD {
                let mut head = section.payload.to_vec();
                head[32..64].copy_from_slice(blake3::hash(tokenizer).as_bytes());
                builder.add_section(section.id, section.flags, &head);
            } else {
                builder.add_section(section.id, section.flags, section.payload);
            }
        }
        builder.add_section(SectionId::SKMX, 0, &skmx);
        return (
            builder.build().expect("lane graph builds"),
            position as u64,
            partner,
        );
    }
    panic!("synthetic corpus must contain a D4-permitted serving position");
}

#[test]
fn canonical_artifact_binds_runtime_winner_and_replays_independently() {
    let (base, teacher) = synthetic_bundle();
    let (meta, records) = corpus_bytes();
    let tokenizer = tokenizer_bytes(&[b"<unk>", b"<s>", b"</s>", b" ", b"a"]);
    let (graph, position, partner) =
        graph_with_planted_runtime_winner(&base, &teacher, &tokenizer, &meta, &records);
    let evaluated = [position];
    let spec = NormativeWitnessReplaySpec {
        material: NormativeWitnessReplayMaterial {
            graph: &graph,
            signature_artifact: &teacher,
            tokenizer: &tokenizer,
            score_report: None,
            corpus_meta: &meta,
            corpus_records: &records,
        },
        evaluated_positions: &evaluated,
        sample_size: 1,
    };

    let artifact = produce_normative_witness_replay(spec).expect("produce replay artifact");
    assert_eq!(artifact.schema, NORMATIVE_WITNESS_REPLAY_SCHEMA);
    assert_eq!(
        (artifact.requested, artifact.replayed, artifact.failures),
        (1, 1, 0)
    );
    assert_eq!(
        artifact.tokenizer_cid,
        format!("blake3:{}", blake3::hash(&tokenizer).to_hex())
    );
    let record = artifact.records.first().expect("one replay row");
    let candidate = record.candidate.expect("served runtime winner");
    assert_eq!(candidate.token, partner);
    assert_eq!(candidate.source, NormativeWitnessCandidateSource::Skipmix);
    assert!(candidate.skmx_contributed);
    assert!(!candidate.psib_contributed);
    let attribution = record.lane_attribution.expect("exact lane attribution");
    assert_eq!(attribution.promoted_token, partner);
    assert_eq!(attribution.contribution_raw, candidate.score_raw);
    assert_eq!(
        (attribution.skmx_contributed, attribution.psib_contributed),
        (candidate.skmx_contributed, candidate.psib_contributed)
    );
    assert_eq!(record.replay_verdict, NormativeWitnessReplayVerdict::Match);

    let bytes = artifact
        .deterministic_json_bytes()
        .expect("serialize canonical artifact");
    let parsed = parse_and_validate_normative_witness_replay(&bytes, spec)
        .expect("canonical artifact replays");
    assert_eq!(parsed, artifact);
    assert_eq!(
        produce_normative_witness_replay(spec)
            .expect("repeat producer")
            .deterministic_json_bytes()
            .expect("serialize repeat"),
        bytes,
        "producer bytes are deterministic"
    );

    let mut missing_provenance: serde_json::Value =
        serde_json::from_slice(&bytes).expect("parse canonical JSON");
    missing_provenance["records"][0]["candidate"]
        .as_object_mut()
        .expect("candidate object")
        .remove("skmx_contributed");
    assert!(
        parse_and_validate_normative_witness_replay(
            &serde_json::to_vec(&missing_provenance).expect("serialize missing-field control"),
            spec,
        )
        .is_err(),
        "schema-2 evidence must explicitly carry both provenance bits"
    );

    let mut old_schema = artifact;
    old_schema.schema = "uor-r4-normative-witness-replay/1".to_owned();
    let old_schema_bytes = old_schema
        .deterministic_json_bytes()
        .expect("serialize old-schema control");
    assert!(
        parse_and_validate_normative_witness_replay(&old_schema_bytes, spec).is_err(),
        "schema-1 evidence cannot inherit schema-2 candidate provenance credit"
    );
}

#[test]
fn planted_candidate_and_foreign_generation_cannot_fake_zero_failures() {
    let (base, teacher) = synthetic_bundle();
    let (meta, records) = corpus_bytes();
    let tokenizer = tokenizer_bytes(&[b"<unk>", b"<s>", b"</s>", b" ", b"a"]);
    let (graph, position, _) =
        graph_with_planted_runtime_winner(&base, &teacher, &tokenizer, &meta, &records);
    let evaluated = [position];
    let material = NormativeWitnessReplayMaterial {
        graph: &graph,
        signature_artifact: &teacher,
        tokenizer: &tokenizer,
        score_report: None,
        corpus_meta: &meta,
        corpus_records: &records,
    };
    let spec = NormativeWitnessReplaySpec {
        material,
        evaluated_positions: &evaluated,
        sample_size: 1,
    };
    let mut artifact = produce_normative_witness_replay(spec).expect("produce replay artifact");
    artifact.records[0]
        .candidate
        .as_mut()
        .expect("served candidate")
        .token ^= 1;
    assert_eq!(
        artifact.failures, 0,
        "planted artifact still claims zero failures"
    );
    let planted = artifact
        .deterministic_json_bytes()
        .expect("canonical planted bytes");
    assert!(
        parse_and_validate_normative_witness_replay(&planted, spec).is_err(),
        "validator must replay the planted token rather than accept its zero counter"
    );

    let mut provenance_tamper =
        produce_normative_witness_replay(spec).expect("produce provenance control");
    let candidate = provenance_tamper.records[0]
        .candidate
        .as_mut()
        .expect("served candidate");
    assert!(candidate.skmx_contributed);
    assert!(!candidate.psib_contributed);
    candidate.skmx_contributed = false;
    candidate.psib_contributed = true;
    assert_eq!(
        provenance_tamper.failures, 0,
        "planted artifact still claims zero failures"
    );
    let provenance_tamper_bytes = provenance_tamper
        .deterministic_json_bytes()
        .expect("serialize provenance tamper");
    assert!(
        parse_and_validate_normative_witness_replay(&provenance_tamper_bytes, spec).is_err(),
        "validator must replay SKMX/PSIB provenance rather than accept a token-identical claim"
    );

    let mut foreign_graph = graph.clone();
    let last = foreign_graph.len() - 1;
    foreign_graph[last] ^= 1;
    let foreign_spec = NormativeWitnessReplaySpec {
        material: NormativeWitnessReplayMaterial {
            graph: &foreign_graph,
            ..material
        },
        evaluated_positions: &evaluated,
        sample_size: 1,
    };
    let original = produce_normative_witness_replay(spec)
        .expect("original replay artifact")
        .deterministic_json_bytes()
        .expect("original canonical bytes");
    assert!(
        parse_and_validate_normative_witness_replay(&original, foreign_spec).is_err(),
        "artifact for another graph generation must fail before credit"
    );

    let foreign_tokenizer = tokenizer_bytes(&[b"<unk>", b"<s>", b"</s>", b" ", b"b"]);
    let foreign_tokenizer_spec = NormativeWitnessReplaySpec {
        material: NormativeWitnessReplayMaterial {
            tokenizer: &foreign_tokenizer,
            ..material
        },
        evaluated_positions: &evaluated,
        sample_size: 1,
    };
    assert!(
        parse_and_validate_normative_witness_replay(&original, foreign_tokenizer_spec).is_err(),
        "artifact for another tokenizer generation must fail before credit"
    );
    assert!(
        produce_normative_witness_replay(foreign_tokenizer_spec).is_err(),
        "schema-2 replay must not load a tokenizer that disagrees with the graph HEAD"
    );
}
