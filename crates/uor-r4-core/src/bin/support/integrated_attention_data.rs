//! Small, source-separated development inputs for the integrated attention experiment.
//!
//! These documents and authored correction episodes are exposed development material. Source
//! pointers below are training-only metadata inferred from already observed bytes; serving never
//! receives a gold role, scope, relation, or source pointer from this loader.

use std::path::Path;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::learner::integrated_attention::training::TrainingEpisode;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

const VOCAB: u32 = 4_096;
const MAX_SOURCE_BYTES: usize = 1 << 20;
const MAX_DOCUMENT_TOKENS: usize = 2_048;
const CHUNK_TOKENS: usize = 512;
const MIN_SOURCE_LAG: usize = 8;
const MIN_SYNTHETIC_LAG: usize = 32;
const DERIVED_TOKENIZER_SHA256: &str =
    "a7ac75b68aa997fe7cc338d25d843157f2dc9d4265fb2f958ee779bb04828d6f";

pub struct DataSet {
    pub fit: Vec<TrainingEpisode>,
    pub dev: Vec<TrainingEpisode>,
    pub manifest: Value,
}

#[derive(Clone, Copy)]
struct SourceSpec {
    path: &'static str,
    sha256: &'static str,
}

const FIT_SOURCES: &[SourceSpec] = &[
    SourceSpec {
        path: "docs/RELEASE_PIPELINE.md",
        sha256: "14f9bf9eed2560fadc74a8474679c6c6315a357e71de279118a7958530535295",
    },
    SourceSpec {
        path: "docs/addr_prism_correspondence.md",
        sha256: "fccf9b4251ce57fe710269f12f578fa9fc1a2d788e2a1af65bba0d24108bac58",
    },
    SourceSpec {
        path: "docs/codebook_fit_460.md",
        sha256: "8ee5714ed2a866432035e042df967711abf0157aeaac923fd24e0411701eaf79",
    },
    SourceSpec {
        path: "docs/compiler_concurrency_config.md",
        sha256: "18cbb6a83301e4934a48e997fa86a426726316cd2b74982a8e495a1c1d8d9737",
    },
    SourceSpec {
        path: "docs/compiler_memory_budget.md",
        sha256: "fc0541462077ba829802fcf8f8990a6c09e76cc1e946dafe869e8392923860d7",
    },
    SourceSpec {
        path: "docs/compositional_planning_certification_spec_846.md",
        sha256: "ffabb86d6fb2e222a3d98ceffd3d4fd4191c81b89f139d9a08fad65adabb4857",
    },
    SourceSpec {
        path: "docs/cover_scaling_460.md",
        sha256: "68fd1ea74c51ae308b8dffb729aaf1b8efdaac2fb9be3f1ac03493a95f1b1e95",
    },
    SourceSpec {
        path: "docs/explainers/ELI5.md",
        sha256: "036b9dcef8ea6f9a02b4b34106844b11e7640dbbdd3ac7adbf153a24555471b2",
    },
    SourceSpec {
        path: "docs/inference_contract.md",
        sha256: "8856b1381a7e6341b06e09c397da16cbc1371aaf86fbc3de56b2582f5fe58194",
    },
    SourceSpec {
        path: "docs/minimal_client.md",
        sha256: "cd0f53d82a8dd953785f1f474104b5b8082dd8e1d1e97a62d79c3719716860f8",
    },
    SourceSpec {
        path: "crates/uor-r4-router/src/decoder_memory.rs",
        sha256: "2a0f899c361d42d63f1dba51cf34db9621941ca2b5429a3049133f3d6eb21565",
    },
    SourceSpec {
        path: "crates/uor-r4-router/src/benchmark.rs",
        sha256: "4785e867fa8b9693f669a2c991c6c1e5a77be1cccc016814000fa8052ee4a7fc",
    },
];

const DEV_SOURCES: &[SourceSpec] = &[
    SourceSpec {
        path: "docs/prime_router_geometric_context_evidence.md",
        sha256: "d197972c903dd61464dc604fe53ae2357a0ae732e3ec4b827f9cbdb89da04195",
    },
    SourceSpec {
        path: "docs/transformerless/LOCAL_ONLY.md",
        sha256: "8b869a372f799d3e22a3c12af8bf29597b5b3a8d7ac531ece63af2c88e407df9",
    },
    SourceSpec {
        path: "crates/uor-r4-graph-runtime/src/route_attention.rs",
        sha256: "6ca45380a9404661ef1bb721706c5a057e6da8b85f4250441e3fd9b9305e2637",
    },
    SourceSpec {
        path: "crates/uor-r4-graph-runtime/src/patch_chain.rs",
        sha256: "3c5a0df6937db04579cec8caffb08d8ce4883d87058028416743ab6be1b682d3",
    },
];

struct AlignedTokens {
    ids: Vec<u16>,
    bytes: Vec<Vec<u8>>,
    starts: Vec<usize>,
    ends: Vec<usize>,
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// Encode once over the whole source. Token-level decoding must reconstitute each exact byte
/// prefix, so a BPE boundary can never be inferred by independently encoding a cited span.
fn tokenize_aligned(tokenizer: &HfBpeTokenizer, text: &str) -> Result<AlignedTokens, String> {
    if text.len() > MAX_SOURCE_BYTES {
        return Err(format!("source exceeds {MAX_SOURCE_BYTES} bytes"));
    }
    let source = text.as_bytes();
    let mut result = AlignedTokens {
        ids: Vec::new(),
        bytes: Vec::new(),
        starts: Vec::new(),
        ends: Vec::new(),
    };
    let mut offset = 0usize;
    for id in tokenizer.encode(text) {
        if id >= VOCAB {
            return Err(format!(
                "token id {id} is outside pinned vocabulary {VOCAB}"
            ));
        }
        let piece = tokenizer.decode_bytes(&[id]);
        let end = offset
            .checked_add(piece.len())
            .ok_or_else(|| "decoded byte offset overflow".to_string())?;
        if piece.is_empty() || end > source.len() || source[offset..end] != piece {
            return Err(format!(
                "whole-stream BPE byte mismatch at input offset {offset}, token {id}"
            ));
        }
        result.ids.push(id as u16);
        result.bytes.push(piece);
        result.starts.push(offset);
        result.ends.push(end);
        offset = end;
    }
    if offset != source.len() {
        return Err(format!(
            "whole-stream BPE ended at {offset} of {} bytes",
            source.len()
        ));
    }
    Ok(result)
}

/// Weak training-only recurrence pointer: target token and its immediate predecessor appeared
/// earlier in this same causal chunk, at least eight tokens back. It is not a semantic truth label.
fn recurrence_targets(tokens: &[u16]) -> Vec<Option<usize>> {
    let mut targets = vec![None; tokens.len()];
    for target in MIN_SOURCE_LAG + 1..tokens.len() {
        for source in (1..=target - MIN_SOURCE_LAG).rev() {
            if tokens[source - 1] == tokens[target - 1] && tokens[source] == tokens[target] {
                targets[target] = Some(source);
                break;
            }
        }
    }
    targets
}

fn load_sources(
    repo: &Path,
    tokenizer: &HfBpeTokenizer,
    specs: &[SourceSpec],
    split: &str,
    next_source_id: &mut u64,
    episodes: &mut Vec<TrainingEpisode>,
    manifest: &mut Vec<Value>,
) -> Result<(), String> {
    for spec in specs {
        let path = repo.join(spec.path);
        let raw = std::fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        if raw.len() > MAX_SOURCE_BYTES {
            return Err(format!("{} exceeds source byte cap", spec.path));
        }
        let actual_sha256 = sha256_hex(&raw);
        if actual_sha256 != spec.sha256 {
            return Err(format!(
                "pinned source mismatch for {}: expected {}, got {}",
                spec.path, spec.sha256, actual_sha256
            ));
        }
        let text = String::from_utf8(raw)
            .map_err(|e| format!("{} is not exact UTF-8 text: {e}", spec.path))?;
        let aligned = tokenize_aligned(tokenizer, &text)
            .map_err(|e| format!("tokenize {}: {e}", spec.path))?;
        let source_id = *next_source_id;
        *next_source_id = next_source_id
            .checked_add(1)
            .ok_or_else(|| "source id overflow".to_string())?;
        let used = aligned.ids.len().min(MAX_DOCUMENT_TOKENS);
        let mut chunk_count = 0usize;
        let mut pointer_count = 0usize;
        for start in (0..used).step_by(CHUNK_TOKENS) {
            let end = (start + CHUNK_TOKENS).min(used);
            if end - start < 2 {
                continue;
            }
            let tokens = aligned.ids[start..end].to_vec();
            let source_targets = recurrence_targets(&tokens);
            pointer_count += source_targets.iter().filter(|p| p.is_some()).count();
            let token_bytes = aligned.bytes[start..end].to_vec();
            episodes.push(TrainingEpisode {
                name: format!("{split}:{}:chunk-{chunk_count}", spec.path),
                source_id,
                tokens,
                token_bytes,
                source_targets,
                prompt_len: None,
            });
            chunk_count += 1;
        }
        manifest.push(json!({
            "kind": "pinned_repository_text",
            "split": split,
            "path": spec.path,
            "sha256": actual_sha256,
            "bytes": text.len(),
            "whole_stream_tokens": aligned.ids.len(),
            "used_tokens": used,
            "chunks": chunk_count,
            "source_id": source_id,
            "weak_recurrence_pointers": pointer_count,
            "exposure": "previously used local development source; not final holdout"
        }));
    }
    Ok(())
}

struct CorrectionCase {
    name: &'static str,
    initial: &'static str,
    correction: &'static str,
    corrected_value: &'static str,
    distractor: &'static str,
    question: &'static str,
    answer: &'static str,
}

const FIT_CORRECTIONS: &[CorrectionCase] = &[
    CorrectionCase {
        name: "cedar-denied",
        initial: "The earlier rule said Project Cedar was allowed to use stale entries.\n",
        correction: "A later correction states that Project Cedar is denied access to stale entries.\n",
        corrected_value: "denied",
        distractor: "Project Juniper is allowed to inspect its own archive. The Juniper setting concerns another project and does not revise Cedar. The log also lists a green queue, a blue queue, and an unrelated staging server.\n",
        question: "Question: Can Project Cedar use a stale entry now? Explain briefly.\nAnswer:",
        answer: " No, Cedar cannot use a stale entry now.",
    },
    CorrectionCase {
        name: "aster-allowed",
        initial: "The first note said Project Aster was denied access to stale entries.\n",
        correction: "The current correction says Project Aster is allowed to use stale entries.\n",
        corrected_value: "allowed",
        distractor: "Project Birch remains denied access to its own old records. Birch has a separate owner and its note does not change Aster. A release summary mentions a red queue, a quiet cache, and a test that was postponed.\n",
        question: "Question: Can Project Aster use a stale entry now? Explain briefly.\nAnswer:",
        answer: " Yes, Aster can use a stale entry now.",
    },
    CorrectionCase {
        name: "maple-denied",
        initial: "For Project Maple, an old instruction allowed stale data in the read path.\n",
        correction: "The final update denied Project Maple the use of stale data.\n",
        corrected_value: "denied",
        distractor: "Project Elm can still read a different cache. Elm's source is separate from Maple's source. The operations page lists three unrelated ports, two build hosts, and a closed maintenance window.\n",
        question: "Question: Is a stale read permitted for Project Maple now? Give a short answer.\nAnswer:",
        answer: " No, Maple cannot perform a stale read now.",
    },
    CorrectionCase {
        name: "willow-allowed",
        initial: "An early warning denied Project Willow use of an old cache entry.\n",
        correction: "A signed revision allowed Project Willow to use an old cache entry.\n",
        corrected_value: "allowed",
        distractor: "Project Oak is denied access to its unrelated queue. The Oak rule belongs to a different team. This paragraph also records an empty staging slot and a repaired display panel.\n",
        question: "Question: May Project Willow read an old cache entry now? Explain briefly.\nAnswer:",
        answer: " Yes, Willow can read an old cache entry now.",
    },
];

const DEV_CORRECTIONS: &[CorrectionCase] = &[
    CorrectionCase {
        name: "orion-allowed",
        initial: "The first operations memo denied Project Orion the use of an archived response.\n",
        correction: "The later policy revision allowed Project Orion to use an archived response.\n",
        corrected_value: "allowed",
        distractor: "Project Vega is denied access to its archive, but Vega is not Orion. A separate report describes an amber status light, a rotated service key, and a completed network inspection.\n",
        question: "Question: Can Orion reuse an archived response now? Give the consequence.\nAnswer:",
        answer: " Yes, Orion can reuse an archived response now.",
    },
    CorrectionCase {
        name: "larch-denied",
        initial: "The old operator note allowed Project Larch to rely on an expired record.\n",
        correction: "The corrected operator note denied Project Larch use of an expired record.\n",
        corrected_value: "denied",
        distractor: "Project Hazel remains allowed to inspect its separate inventory. Hazel has no authority over Larch. Elsewhere the note describes a cold storage shelf, a queued audit, and an empty request log.\n",
        question: "Question: Should Larch rely on an expired record now? Give the consequence.\nAnswer:",
        answer: " No, Larch cannot rely on an expired record now.",
    },
];

fn load_corrections(
    tokenizer: &HfBpeTokenizer,
    cases: &[CorrectionCase],
    split: &str,
    next_source_id: &mut u64,
    episodes: &mut Vec<TrainingEpisode>,
    manifest: &mut Vec<Value>,
) -> Result<(), String> {
    for case in cases {
        let value_in_correction = case
            .correction
            .find(case.corrected_value)
            .ok_or_else(|| format!("{} missing corrected value", case.name))?;
        let source_value_start = case.initial.len() + value_in_correction;
        let source_value_end = source_value_start + case.corrected_value.len();
        let prompt = format!(
            "{}{}{}{}",
            case.initial, case.correction, case.distractor, case.question
        );
        let full = format!("{}{}", prompt, case.answer);
        let aligned = tokenize_aligned(tokenizer, &full)
            .map_err(|e| format!("correction {}: {e}", case.name))?;
        let prompt_ids = tokenizer.encode(&prompt);
        if aligned.ids.len() > MAX_DOCUMENT_TOKENS
            || prompt_ids.len() >= aligned.ids.len()
            || !aligned
                .ids
                .iter()
                .map(|&id| u32::from(id))
                .take(prompt_ids.len())
                .eq(prompt_ids.iter().copied())
            || aligned.starts[prompt_ids.len()] != prompt.len()
        {
            return Err(format!("{} has unstable prompt/BPE boundary", case.name));
        }
        let source_token = aligned
            .starts
            .iter()
            .zip(&aligned.ends)
            .enumerate()
            .find_map(|(i, (&start, &end))| {
                (start < source_value_end && end > source_value_start).then_some(i)
            })
            .ok_or_else(|| format!("{} correction span has no token", case.name))?;
        let prompt_len = prompt_ids.len();
        if prompt_len.saturating_sub(source_token) < MIN_SYNTHETIC_LAG {
            return Err(format!("{} source gap is too short", case.name));
        }
        let mut targets = vec![None; aligned.ids.len()];
        for target in prompt_len..targets.len() {
            targets[target] = Some(source_token);
        }
        let source_id = *next_source_id;
        *next_source_id = next_source_id
            .checked_add(1)
            .ok_or_else(|| "source id overflow".to_string())?;
        manifest.push(json!({
            "kind": "authored_correction_consequence",
            "split": split,
            "name": case.name,
            "source_id": source_id,
            "source_text": format!("{}{}{}", case.initial, case.correction, case.distractor),
            "question": case.question,
            "answer": case.answer,
            "sha256_full_text": sha256_hex(full.as_bytes()),
            "tokens": aligned.ids.len(),
            "prompt_tokens": prompt_len,
            "corrected_value": case.corrected_value,
            "corrected_value_byte_span": [source_value_start, source_value_end],
            "source_token": source_token,
            "answer_grounding_targets": targets.len() - prompt_len,
            "exposure": "authored and visible development example; not a fresh holdout"
        }));
        episodes.push(TrainingEpisode {
            name: format!("{split}:correction:{}", case.name),
            source_id,
            tokens: aligned.ids,
            token_bytes: aligned.bytes,
            source_targets: targets,
            prompt_len: Some(prompt_len),
        });
    }
    Ok(())
}

pub fn load_development_data(repo: &Path, tokenizer: &HfBpeTokenizer) -> Result<DataSet, String> {
    if tokenizer.vocab_size() != VOCAB as usize {
        return Err(format!(
            "expected pinned vocabulary {VOCAB}, got {}",
            tokenizer.vocab_size()
        ));
    }
    let mut fit = Vec::new();
    let mut dev = Vec::new();
    let mut documents = Vec::new();
    let mut synthetic_cases = Vec::new();
    let mut source_id = 1u64;
    load_sources(
        repo,
        tokenizer,
        FIT_SOURCES,
        "fit",
        &mut source_id,
        &mut fit,
        &mut documents,
    )?;
    load_sources(
        repo,
        tokenizer,
        DEV_SOURCES,
        "dev",
        &mut source_id,
        &mut dev,
        &mut documents,
    )?;
    load_corrections(
        tokenizer,
        FIT_CORRECTIONS,
        "fit",
        &mut source_id,
        &mut fit,
        &mut synthetic_cases,
    )?;
    load_corrections(
        tokenizer,
        DEV_CORRECTIONS,
        "dev",
        &mut source_id,
        &mut dev,
        &mut synthetic_cases,
    )?;
    let manifest = json!({
        "schema": "uor-r4.integrated-attention-development-data/1",
        "source_snapshot": "3101c06061726d9438a22649a25b98558525e8d0",
        "preserved_markdown_corpus_commit": "e9c04e80",
        "tokenizer": {
            "derived_sha256_expected": DERIVED_TOKENIZER_SHA256,
            "runtime_address": tokenizer.address(),
            "vocab": VOCAB,
        },
        "limits": {
            "max_source_bytes": MAX_SOURCE_BYTES,
            "max_document_tokens": MAX_DOCUMENT_TOKENS,
            "chunk_tokens": CHUNK_TOKENS,
            "minimum_causal_source_lag": MIN_SOURCE_LAG,
            "minimum_correction_source_lag": MIN_SYNTHETIC_LAG,
        },
        "documents": documents,
        "synthetic_cases": synthetic_cases,
        "fit_episodes": fit.len(),
        "dev_episodes": dev.len(),
        "claim_boundary": "open development data with training-only weak source pointers; no semantic or coding qualification"
    });
    Ok(DataSet { fit, dev, manifest })
}
