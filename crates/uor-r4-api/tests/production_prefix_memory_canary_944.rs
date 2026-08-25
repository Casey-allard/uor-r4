//! Bounded S3 production canary (#944).
//!
//! This is deliberately not a coherence benchmark. It admits the exact
//! schema-2 production envelope, then asks whether two token histories with
//! the same newest compiler window but different earlier prefixes can change
//! a normative candidate list or served token through an actually admitted
//! secondary session probe.
//!
//! Run:
//!   R4_S3_CANARY_BUNDLE=/path/to/schema-2-bundle \
//!     cargo test -p uor-r4-api --test production_prefix_memory_canary_944 \
//!       production_prefix_memory_canary_944 -- --ignored --nocapture

use std::path::{Path, PathBuf};

use serde::Serialize;
use uor_r4_api::{
    verify_production_envelope, EngineParts, NormativeServingDecision, NormativeServingEngine,
    ProductionEnvelopeParts, ProductionServingParts,
};
use uor_r4_core::transformerless::compiler::{self, WINDOW};
use uor_r4_graph_runtime::{ServedCandidateSource, SignatureRoutingSource, SignatureRoutingTrace};
use uor_r4_router::TokenHistorySignature;

const MAX_CASES: usize = 512;
const RESULT_SCHEMA: &str = "uor-r4-production-prefix-memory-canary/1";
const DEFAULT_ISSUE: u32 = 944;
const DEFAULT_REPORT: &str = "docs/production_prefix_memory_canary_944_result.json";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("api crate has repository parent")
        .to_path_buf()
}

fn bundle_root() -> Result<PathBuf, String> {
    std::env::var_os("R4_S3_CANARY_BUNDLE")
        .map(PathBuf::from)
        .ok_or_else(|| "R4_S3_CANARY_BUNDLE is required; no ambient bundle fallback".to_owned())
}

fn read(root: &Path, relative: &str) -> Result<Vec<u8>, String> {
    std::fs::read(root.join(relative))
        .map_err(|error| format!("read {relative} from {}: {error}", root.display()))
}

fn cid(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct CandidateRecord {
    token: u32,
    score_raw: i32,
    source: &'static str,
    skmx_contributed: bool,
    psib_contributed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct DecisionRecord {
    disposition: &'static str,
    token: Option<u32>,
    candidates: Vec<CandidateRecord>,
}

fn decision_record(decision: NormativeServingDecision) -> DecisionRecord {
    match decision {
        NormativeServingDecision::Serve(served) => DecisionRecord {
            disposition: "serve",
            token: Some(served.token),
            candidates: served
                .candidates
                .ranked()
                .iter()
                .map(|candidate| CandidateRecord {
                    token: candidate.token,
                    score_raw: candidate.score.raw(),
                    source: match candidate.source {
                        ServedCandidateSource::Base => "base",
                        ServedCandidateSource::Skipmix => "skipmix",
                    },
                    skmx_contributed: candidate.skmx_contributed,
                    psib_contributed: candidate.psib_contributed,
                })
                .collect(),
        },
        NormativeServingDecision::Abstain(_) => DecisionRecord {
            disposition: "abstain",
            token: None,
            candidates: Vec::new(),
        },
        NormativeServingDecision::Decline(_) => DecisionRecord {
            disposition: "decline",
            token: None,
            candidates: Vec::new(),
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
struct RoutingRecord {
    context_row_hit: bool,
    suffix_dfa_nodes: u8,
    context_probe_attempted: bool,
    context_admitted_nodes: u8,
    session_probe_attempted: bool,
    session_admitted_nodes: u8,
    selected_source: &'static str,
}

fn routing_record(trace: SignatureRoutingTrace) -> RoutingRecord {
    RoutingRecord {
        context_row_hit: trace.context_row_hit,
        suffix_dfa_nodes: trace.suffix_dfa_nodes,
        context_probe_attempted: trace.context_probe_attempted,
        context_admitted_nodes: trace.context_admitted_nodes,
        session_probe_attempted: trace.session_probe_attempted,
        session_admitted_nodes: trace.session_admitted_nodes,
        selected_source: match trace.selected_source {
            SignatureRoutingSource::None => "none",
            SignatureRoutingSource::ContextRow => "context-row",
            SignatureRoutingSource::SuffixDfa => "suffix-dfa",
            SignatureRoutingSource::ContextSignature => "context-signature",
            SignatureRoutingSource::SessionSignature => "session-signature",
            SignatureRoutingSource::ComposedSignatures => "composed-signatures",
            SignatureRoutingSource::NearestContextSignature => "nearest-context-signature",
            SignatureRoutingSource::NearestSessionSignature => "nearest-session-signature",
            SignatureRoutingSource::DefaultNode => "default-node",
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct EffectRecord {
    position: u64,
    story: u32,
    prefix_len: usize,
    window: Vec<u32>,
    full_signature_cid: String,
    suffix_signature_cid: String,
    full_routing: RoutingRecord,
    suffix_routing: RoutingRecord,
    full_decision: DecisionRecord,
    suffix_decision: DecisionRecord,
    no_session_decision: DecisionRecord,
    candidate_list_changed: bool,
    served_token_changed: bool,
}

fn is_effect(record: &EffectRecord) -> bool {
    record.full_routing.context_probe_attempted
        && record.full_routing.session_probe_attempted
        && record.full_routing.session_admitted_nodes > 0
        && matches!(
            record.full_routing.selected_source,
            "session-signature" | "composed-signatures"
        )
        && (record.candidate_list_changed || record.served_token_changed)
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
struct Counts {
    inspected: u64,
    context_row_hits: u64,
    suffix_dfa_routes: u64,
    context_probe_attempts: u64,
    context_probe_misses: u64,
    session_probe_attempts: u64,
    session_probe_admissions: u64,
    candidate_list_changes: u64,
    served_token_changes: u64,
}

impl Counts {
    fn observe(&mut self, record: &EffectRecord) {
        self.inspected += 1;
        self.context_row_hits += u64::from(record.full_routing.context_row_hit);
        self.suffix_dfa_routes += u64::from(record.full_routing.selected_source == "suffix-dfa");
        self.context_probe_attempts += u64::from(record.full_routing.context_probe_attempted);
        self.context_probe_misses += u64::from(
            record.full_routing.context_probe_attempted
                && record.full_routing.context_admitted_nodes == 0,
        );
        self.session_probe_attempts += u64::from(record.full_routing.session_probe_attempted);
        self.session_probe_admissions += u64::from(record.full_routing.session_admitted_nodes > 0);
        self.candidate_list_changes += u64::from(record.candidate_list_changed);
        self.served_token_changes += u64::from(record.served_token_changed);
    }
}

#[derive(Debug, Serialize)]
struct CanaryReport {
    schema: &'static str,
    issue: u32,
    verdict: &'static str,
    claim_scope: &'static str,
    max_cases: usize,
    graph_cid: String,
    release_manifest_cid: String,
    corpus_meta_cid: String,
    corpus_records_cid: String,
    selection_rule: &'static str,
    tested_count: usize,
    first_tested_position: u64,
    last_tested_position: u64,
    tested_positions_cid: String,
    observations_cid: String,
    counts: Counts,
    first_effect: Option<EffectRecord>,
}

fn select(
    engine: &mut NormativeServingEngine<'_>,
    window: &[u32],
    signature: Option<&[u8]>,
) -> Result<DecisionRecord, String> {
    engine.reset_policy_state();
    engine
        .predict_with_session_signature(window, signature)
        .map(decision_record)
        .map_err(|error| format!("normative selection rejected canary input: {error:?}"))
}

fn run_canary(root: &Path, issue: u32) -> Result<CanaryReport, String> {
    let graph = read(root, "graph/score.r4g1")?;
    let sections_absent_graph = read(root, "graph/score_sections_absent.r4g1")?;
    let label_shuffled_graph = read(root, "graph/score_label_shuffled.r4g1")?;
    let signature_artifact = read(root, "tless_artifacts.bin")?;
    let tla_comparator_store = read(root, "tless_store.bin")?;
    let tokenizer = read(root, "tokenizer.bin")?;
    let score_report = read(root, "graph/score_report.json")?;
    let compile_report = read(root, "graph-cover/cover_report.json")?;
    let deployed_quality_report = read(root, "graph/deployed_quality_report.json")?;
    let cross_surface_parity = read(root, "graph/cross_surface_parity.json")?;
    let witness_replay = read(root, "graph/witness_replay.json")?;
    let corpus_meta = read(root, "corpus.meta")?;
    let corpus_records = read(root, "corpus.records")?;
    let tokenizer_adapter = read(root, "tokenizer_adapter.json")?;
    let release_manifest = read(root, "release-bundle.json")?;

    let verified = verify_production_envelope(ProductionEnvelopeParts {
        graph: &graph,
        sections_absent_graph: &sections_absent_graph,
        label_shuffled_graph: &label_shuffled_graph,
        signature_artifact: &signature_artifact,
        tla_comparator_store: &tla_comparator_store,
        tokenizer: &tokenizer,
        score_report: &score_report,
        compile_report: &compile_report,
        deployed_quality_report: &deployed_quality_report,
        cross_surface_parity: &cross_surface_parity,
        witness_replay: &witness_replay,
        corpus_meta: &corpus_meta,
        corpus_records: &corpus_records,
        tokenizer_adapter: &tokenizer_adapter,
        release_manifest: &release_manifest,
    })
    .map_err(|error| format!("production envelope unavailable: {error}"))?;
    let mut engine = NormativeServingEngine::load(ProductionServingParts {
        engine: EngineParts {
            graph: &graph,
            signature_artifact: &signature_artifact,
            tokenizer: Some(&tokenizer),
            score_report: Some(&score_report),
        },
        deployed_quality_report: &deployed_quality_report,
        verified_envelope: &verified,
    })
    .map_err(|error| format!("normative production engine unavailable: {error}"))?;
    let corpus = compiler::load_corpus_bytes(&corpus_meta, &corpus_records, None)
        .ok_or_else(|| "production corpus bytes do not parse".to_owned())?;
    let (_, held_out) = compiler::split_positions(&corpus);

    let mut tested_positions = Vec::with_capacity(MAX_CASES);
    let mut observation_bytes = Vec::new();
    let mut counts = Counts::default();
    let mut first_effect = None;

    for position in held_out {
        if tested_positions.len() == MAX_CASES || first_effect.is_some() {
            break;
        }
        let window = compiler::context_window(&corpus, position);
        if window.len() != WINDOW {
            continue;
        }
        let story = corpus.story[position];
        let mut story_start = position;
        while story_start > 0 && corpus.story[story_start - 1] == story {
            story_start -= 1;
        }
        let full_prefix = &corpus.input[story_start..=position];
        if full_prefix.len() <= WINDOW {
            continue;
        }
        let full_signature = TokenHistorySignature::from_tokens(full_prefix).signature();
        let suffix_signature = TokenHistorySignature::from_tokens(&window).signature();
        if full_signature == suffix_signature {
            return Err(format!(
                "distinct complete and suffix histories collided at position {position}"
            ));
        }

        let full_routing = engine
            .inspect_signature_routing(&window, Some(&full_signature))
            .map(routing_record)
            .map_err(|error| format!("inspect full-prefix route: {error:?}"))?;
        let suffix_routing = engine
            .inspect_signature_routing(&window, Some(&suffix_signature))
            .map(routing_record)
            .map_err(|error| format!("inspect suffix route: {error:?}"))?;
        let full_decision = select(&mut engine, &window, Some(&full_signature))?;
        let suffix_decision = select(&mut engine, &window, Some(&suffix_signature))?;
        let no_session_decision = select(&mut engine, &window, None)?;
        let candidate_list_changed = full_decision.candidates != suffix_decision.candidates;
        let served_token_changed = full_decision.token != suffix_decision.token;
        let record = EffectRecord {
            position: position as u64,
            story,
            prefix_len: full_prefix.len(),
            window,
            full_signature_cid: cid(&full_signature),
            suffix_signature_cid: cid(&suffix_signature),
            full_routing,
            suffix_routing,
            full_decision,
            suffix_decision,
            no_session_decision,
            candidate_list_changed,
            served_token_changed,
        };
        tested_positions.push(position as u64);
        counts.observe(&record);
        let bytes = serde_json::to_vec(&record)
            .map_err(|error| format!("serialize canary observation: {error}"))?;
        observation_bytes.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
        observation_bytes.extend_from_slice(&bytes);
        if is_effect(&record) {
            first_effect = Some(record);
        }
    }

    if tested_positions.is_empty() {
        return Err(
            "no held-out position had a complete prefix beyond the fixed window".to_owned(),
        );
    }
    let position_bytes: Vec<u8> = tested_positions
        .iter()
        .flat_map(|position| position.to_le_bytes())
        .collect();
    Ok(CanaryReport {
        schema: RESULT_SCHEMA,
        issue,
        verdict: if first_effect.is_some() {
            "EFFECT_ESTABLISHED"
        } else {
            "INERT"
        },
        claim_scope: "one-step normative R4G1 production selection on the exact admitted artifact; no coherence claim",
        max_cases: MAX_CASES,
        graph_cid: cid(&graph),
        release_manifest_cid: cid(&release_manifest),
        corpus_meta_cid: cid(&corpus_meta),
        corpus_records_cid: cid(&corpus_records),
        selection_rule: "first held-out corpus positions with a complete in-story prefix longer than the newest eight-token window",
        tested_count: tested_positions.len(),
        first_tested_position: tested_positions[0],
        last_tested_position: *tested_positions.last().expect("non-empty positions"),
        tested_positions_cid: cid(&position_bytes),
        observations_cid: cid(&observation_bytes),
        counts,
        first_effect,
    })
}

fn write_report(report: &CanaryReport) -> Result<PathBuf, String> {
    let mut bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| format!("serialize canary report: {error}"))?;
    bytes.push(b'\n');
    let output = std::env::var_os("R4_S3_CANARY_REPORT")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root().join(DEFAULT_REPORT));
    std::fs::write(&output, bytes)
        .map_err(|error| format!("write {}: {error}", output.display()))?;
    Ok(output)
}

#[test]
fn canary_verdict_requires_behavior_and_session_admission() {
    let decision = DecisionRecord {
        disposition: "serve",
        token: Some(7),
        candidates: vec![CandidateRecord {
            token: 7,
            score_raw: 10,
            source: "base",
            skmx_contributed: false,
            psib_contributed: false,
        }],
    };
    let mut record = EffectRecord {
        position: 9,
        story: 1,
        prefix_len: WINDOW + 1,
        window: vec![1; WINDOW],
        full_signature_cid: "blake3:full".to_owned(),
        suffix_signature_cid: "blake3:suffix".to_owned(),
        full_routing: RoutingRecord {
            context_row_hit: false,
            suffix_dfa_nodes: 0,
            context_probe_attempted: true,
            context_admitted_nodes: 0,
            session_probe_attempted: true,
            session_admitted_nodes: 1,
            selected_source: "session-signature",
        },
        suffix_routing: RoutingRecord {
            context_row_hit: false,
            suffix_dfa_nodes: 0,
            context_probe_attempted: true,
            context_admitted_nodes: 0,
            session_probe_attempted: true,
            session_admitted_nodes: 0,
            selected_source: "nearest-context-signature",
        },
        full_decision: decision.clone(),
        suffix_decision: decision.clone(),
        no_session_decision: decision,
        candidate_list_changed: false,
        served_token_changed: false,
    };
    assert!(
        !is_effect(&record),
        "routing alone is not a behavioral effect"
    );
    record.served_token_changed = true;
    assert!(
        is_effect(&record),
        "admitted session routing plus a changed token qualifies"
    );
    record.full_routing.context_admitted_nodes = 4;
    record.full_routing.selected_source = "composed-signatures";
    assert!(
        is_effect(&record),
        "composed context and trajectory admissions qualify"
    );
    record.full_routing.session_admitted_nodes = 0;
    assert!(
        !is_effect(&record),
        "a changed token without session admission is inert"
    );
}

#[test]
#[ignore = "bounded local evidence run: requires the exact schema-2 #933 envelope"]
fn production_prefix_memory_canary_944() {
    let root = bundle_root().unwrap_or_else(|reason| panic!("UNAVAILABLE: {reason}"));
    let issue = std::env::var("R4_S3_CANARY_ISSUE")
        .ok()
        .map(|value| value.parse::<u32>())
        .transpose()
        .unwrap_or_else(|error| panic!("UNAVAILABLE: invalid R4_S3_CANARY_ISSUE: {error}"))
        .unwrap_or(DEFAULT_ISSUE);
    let report = run_canary(&root, issue).unwrap_or_else(|reason| panic!("UNAVAILABLE: {reason}"));
    let output = write_report(&report).expect("retain deterministic canary report");
    println!("verdict                 : {}", report.verdict);
    println!("inspected               : {}", report.counts.inspected);
    println!(
        "context probe misses    : {}",
        report.counts.context_probe_misses
    );
    println!(
        "session probe admissions: {}",
        report.counts.session_probe_admissions
    );
    println!(
        "candidate list changes  : {}",
        report.counts.candidate_list_changes
    );
    println!(
        "served token changes    : {}",
        report.counts.served_token_changes
    );
    println!("observations CID        : {}", report.observations_cid);
    println!("wrote                   : {}", output.display());
}
