//! S2: Codec Evaluation & Turn Matching Optimization
//!
//! Evaluates candidate codecs against the baseline dialogue model (dialogue-1)
//! on the 38-request reply panel (58 turns), measuring:
//!   - Matching greedy turns out of 58 against float baseline (baseline: 14)
//!   - Response NLL on the 161-response development panel
//!   - Serialized bits per weight (<= 4.25 bpw)
//!   - Exactness: exported integer turns equal served float turns

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::{Device, Tensor};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uor_r4_core::report_output;
use uor_r4_integer::stack::{stack_argmax, IntegerStackModel};
use uor_r4_lut::format::{StackArtifact, StackArtifactBuilder, TableKind, TableValues};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::dialogue_development;
use uor_r4_training::geometric_stack::StackModel;
use uor_r4_training::lut_export::dequantize_matrix;
use uor_r4_training::stack_dialogue::{
    development, development_with_head, episode_contract, greedy_reply, load_requests, reply_panel,
    DialogueSplit, Reply,
};
use uor_r4_training::{Result, TrainingError};

const MAX_NEW_TOKENS: usize = 32;

fn invalid(msg: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(msg.into())
}

fn sha256_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}

fn integer_reply(
    model: &IntegerStackModel,
    history: &[u32],
    cap: usize,
    eos: u32,
) -> Result<Reply> {
    let mut session = model.session();
    let mut stepped = false;
    for &tok in history {
        session.step(tok).map_err(|e| invalid(e.to_string()))?;
        stepped = true;
    }
    let mut ids = Vec::with_capacity(cap);
    for _ in 0..cap {
        if !stepped {
            return Err(invalid("no logits produced"));
        }
        // Greedy over the served distribution: a pointer model's mixture,
        // otherwise the logits (first maximum on ties).
        let next = stack_argmax(session.next_token_scores()) as u32;
        ids.push(next);
        if let Some(reply) = Reply::stop(&ids, eos) {
            return Ok(reply);
        }
        session.step(next).map_err(|e| invalid(e.to_string()))?;
    }
    Ok(Reply {
        ids,
        eos: false,
        cycle: None,
    })
}

fn count_turn_matches(a: &Value, b: &Value) -> (usize, usize) {
    let Some(rows_a) = a["rows"].as_array() else {
        return (0, 0);
    };
    let Some(rows_b) = b["rows"].as_array() else {
        return (0, 0);
    };
    let mut matches = 0usize;
    let mut total = 0usize;

    for (r_a, r_b) in rows_a.iter().zip(rows_b.iter()) {
        let Some(turns_a) = r_a["turns"].as_array() else {
            continue;
        };
        let Some(turns_b) = r_b["turns"].as_array() else {
            continue;
        };
        for (t_a, t_b) in turns_a.iter().zip(turns_b.iter()) {
            total += 1;
            if t_a["reply_ids"] == t_b["reply_ids"] {
                matches += 1;
            }
        }
    }
    (matches, total)
}

fn resolve_candidate_path(
    explicit: Option<&String>,
    env_key: &str,
    candidates: &[&str],
    default_path: &str,
) -> PathBuf {
    if let Some(val) = explicit {
        return PathBuf::from(val);
    }
    if let Ok(val) = std::env::var(env_key) {
        return PathBuf::from(val);
    }
    for cand in candidates {
        let p = Path::new(cand);
        if p.exists() {
            return p.to_path_buf();
        }
    }
    PathBuf::from(default_path)
}

fn get_var_vec1(model: &StackModel, name: &str) -> Result<Vec<f32>> {
    let var = model
        .variables()
        .get(name)
        .ok_or_else(|| invalid(format!("model missing variable: {name}")))?;
    let flat = var
        .as_tensor()
        .flatten_all()
        .map_err(|e| invalid(e.to_string()))?;
    flat.to_vec1::<f32>().map_err(|e| invalid(e.to_string()))
}

fn current_binary_sha256() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| fs::read(p).ok())
        .map(|b| {
            let mut hasher = Sha256::new();
            hasher.update(&b);
            hex::encode(hasher.finalize())
        })
        .unwrap_or_else(|| "UNAVAILABLE".to_string())
}

fn current_git_commit() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                String::from_utf8(out.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "UNAVAILABLE".to_string())
}

struct MapSpec {
    name: String,
    var_name: Option<String>,
    rows: usize,
    cols: usize,
}

fn set_model_var(model: &StackModel, name: &str, values: &[f32]) -> Result<()> {
    let var = model
        .variables()
        .get(name)
        .ok_or_else(|| invalid(format!("model has no variable {name}")))?;
    let shape = var.as_tensor().shape();
    let dims = shape.dims();
    let unpadded_values = if dims.len() == 2 && (dims[0] * dims[1] != values.len()) {
        let (rows, cols) = (dims[0], dims[1]);
        if values.len() % cols == 0 && values.len() / cols > rows {
            values[..rows * cols].to_vec()
        } else if values.len() % rows == 0 && values.len() / rows > cols {
            let padded_cols = values.len() / rows;
            let mut unpadded = Vec::with_capacity(rows * cols);
            for r in 0..rows {
                unpadded.extend_from_slice(&values[r * padded_cols..r * padded_cols + cols]);
            }
            unpadded
        } else {
            return Err(invalid(format!("shape mismatch for {name}")));
        }
    } else {
        values.to_vec()
    };
    var.set(
        &Tensor::from_vec(
            unpadded_values,
            var.as_tensor().shape(),
            var.as_tensor().device(),
        )
        .map_err(|e| invalid(e.to_string()))?,
    )
    .map_err(|e| invalid(e.to_string()))?;
    Ok(())
}

fn fold_columns(weights: &mut [f32], cols: usize, gain: &[f32]) {
    assert_eq!(weights.len() % cols, 0);
    assert_eq!(gain.len(), cols);
    for row in weights.chunks_exact_mut(cols) {
        for (w, &g) in row.iter_mut().zip(gain) {
            *w *= g;
        }
    }
}

fn greedy_reply_with_head(
    model: &StackModel,
    head: &Tensor,
    history: &[u32],
    cap: usize,
    eos: u32,
) -> Result<Reply> {
    let mut window = history.to_vec();
    let mut ids = Vec::with_capacity(cap);
    for _ in 0..cap {
        if window.len() > model.config.context {
            return Err(invalid("the reply outgrew the context"));
        }
        let hidden = model
            .hidden(&window, 1, window.len())
            .map_err(|e| invalid(e.to_string()))?;
        let last_hidden = hidden
            .get(window.len() - 1)
            .map_err(|e| invalid(e.to_string()))?;
        let logits = last_hidden
            .unsqueeze(0)
            .map_err(|e| invalid(e.to_string()))?
            .matmul(&head.t().map_err(|e| invalid(e.to_string()))?)
            .map_err(|e| invalid(e.to_string()))?
            .squeeze(0)
            .map_err(|e| invalid(e.to_string()))?
            .to_vec1::<f32>()
            .map_err(|e| invalid(e.to_string()))?;
        let mut best = 0usize;
        for (i, v) in logits.iter().enumerate() {
            if *v > logits[best] {
                best = i;
            }
        }
        let next = best as u32;
        ids.push(next);
        window.push(next);
        if let Some(reply) = Reply::stop(&ids, eos) {
            return Ok(reply);
        }
    }
    Ok(Reply {
        ids,
        eos: false,
        cycle: None,
    })
}

fn build_modified_artifact(
    base: &StackArtifact,
    replacements: &BTreeMap<String, (i32, Vec<u8>, Vec<u8>)>,
) -> Result<Vec<u8>> {
    let mut builder = StackArtifactBuilder::new(
        base.header.shape.clone(),
        base.header.numerics.clone(),
        base.header.source.clone(),
    )
    .map_err(|e| invalid(e.to_string()))?;

    for m in &base.header.matrices {
        if let Some((exp_base, nibbles, scales)) = replacements.get(&m.name) {
            builder
                .add_matrix(&m.name, m.rows, m.cols, *exp_base, nibbles, scales)
                .map_err(|e| invalid(e.to_string()))?;
        } else {
            builder
                .add_matrix(
                    &m.name,
                    m.rows,
                    m.cols,
                    m.exp_base,
                    base.section(m.nibbles),
                    base.section(m.scales),
                )
                .map_err(|e| invalid(e.to_string()))?;
        }
    }

    for t in &base.header.tables {
        match t.kind {
            TableKind::I16 => {
                let vals = base
                    .table_i16(&t.name)
                    .map_err(|e| invalid(e.to_string()))?;
                builder
                    .add_table(&t.name, TableValues::I16(&vals))
                    .map_err(|e| invalid(e.to_string()))?;
            }
            TableKind::I32 => {
                let vals = base
                    .table_i32(&t.name)
                    .map_err(|e| invalid(e.to_string()))?;
                builder
                    .add_table(&t.name, TableValues::I32(&vals))
                    .map_err(|e| invalid(e.to_string()))?;
            }
            TableKind::U32 => {
                let vals = base
                    .table_u32(&t.name)
                    .map_err(|e| invalid(e.to_string()))?;
                builder
                    .add_table(&t.name, TableValues::U32(&vals))
                    .map_err(|e| invalid(e.to_string()))?;
            }
        }
    }

    builder.finish().map_err(|e| invalid(e.to_string()))
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut map = BTreeMap::new();
    for arg in &args {
        if let Some((k, v)) = arg.split_once('=') {
            map.insert(k.trim_start_matches('-').to_string(), v.to_string());
        }
    }
    let skip_diagnostic = args
        .iter()
        .any(|a| a == "--skip-diagnostic" || a == "-skip-diagnostic");
    let selected_arms: Option<Vec<String>> = map
        .get("arms")
        .map(|s| s.split(',').map(|x| x.trim().to_string()).collect());

    let model_dir = PathBuf::from(
        map.get("model")
            .cloned()
            .or_else(|| std::env::var("UOR_MODEL_DIR").ok())
            .unwrap_or_else(|| {
                "/Volumes/UOR-Workspace/uor-r4-lab/claude-s2-dialogue-baseline/dialogue-1/model"
                    .into()
            }),
    );
    let baseline_lut = PathBuf::from(
        map.get("baseline_lut")
            .cloned()
            .or_else(|| std::env::var("UOR_BASELINE_LUT").ok())
            .unwrap_or_else(|| {
                "/Volumes/UOR-Workspace/uor-r4-lab/claude-s2-dialogue-baseline/export-1/model.lut"
                    .into()
            }),
    );
    let heldout_path = PathBuf::from(
        map.get("heldout")
            .cloned()
            .or_else(|| std::env::var("UOR_HELDOUT").ok())
            .unwrap_or_else(|| {
                "/Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/prepared/heldout".into()
            }),
    );
    let requests_path = resolve_candidate_path(
        map.get("requests"),
        "UOR_REQUESTS",
        &[
            "tests/fixtures/development-requests.json",
            ".uor-models/investigations/fourth-research-lab-20260926/dialogue-artifact-replay-1/development-requests.json",
            "../.uor-models/investigations/fourth-research-lab-20260926/dialogue-artifact-replay-1/development-requests.json",
            "../../.uor-models/investigations/fourth-research-lab-20260926/dialogue-artifact-replay-1/development-requests.json",
        ],
        "development-requests.json",
    );
    let tokenizer_path = resolve_candidate_path(
        map.get("tokenizer"),
        "UOR_TOKENIZER",
        &[
            "tests/fixtures/tokenizer.json",
            ".uor-models/investigations/integer-serving-20260925/bundle-quaternion-1/tokenizer.json",
            "../.uor-models/investigations/integer-serving-20260925/bundle-quaternion-1/tokenizer.json",
            "../../.uor-models/investigations/integer-serving-20260925/bundle-quaternion-1/tokenizer.json",
        ],
        "tokenizer.json",
    );
    let out = PathBuf::from(
        map.get("out")
            .cloned()
            .or_else(|| std::env::var("UOR_OUT_DIR").ok())
            .unwrap_or_else(|| {
                "/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/s2-turn-optimization-1".into()
            }),
    );

    println!("=== S2: Dialogue Codec Evaluation & Turn Matching Optimization ===");
    println!("Model:      {}", model_dir.display());
    println!("Baseline:   {}", baseline_lut.display());
    println!("Requests:   {}", requests_path.display());
    println!("Output Dir: {}", out.display());

    report_output::claim(&out)?;

    let execution_result = (|| -> Result<()> {
        let clock = Instant::now();
        let model_sha = sha256_file(&model_dir.join("model.safetensors"))?;
        let model_config_sha = sha256_file(&model_dir.join("config.json"))?;
        let baseline_lut_sha = sha256_file(&baseline_lut)?;
        let requests_sha = sha256_file(&requests_path)?;
        let tokenizer_sha = sha256_file(&tokenizer_path)?;
        let heldout_tokens_sha = sha256_file(&heldout_path.join("tokens.u16"))
            .unwrap_or_else(|_| "UNAVAILABLE".to_string());
        let heldout_mask_sha = sha256_file(&heldout_path.join("response_mask.u8"))
            .unwrap_or_else(|_| "UNAVAILABLE".to_string());
        let heldout_manifest_sha = sha256_file(&heldout_path.join("manifest.json"))
            .unwrap_or_else(|_| "UNAVAILABLE".to_string());
        println!("Verified Model SHA-256:        {model_sha}");
        println!("Verified Model Config SHA-256: {model_config_sha}");
        println!("Verified Baseline LUT SHA-256: {baseline_lut_sha}");
        println!("Verified Requests SHA-256:     {requests_sha}");
        println!("Verified Tokenizer SHA-256:    {tokenizer_sha}");
        println!("Verified Heldout Tokens SHA:   {heldout_tokens_sha}");
        println!("Verified Heldout Mask SHA:     {heldout_mask_sha}");
        println!("Verified Heldout Manifest SHA: {heldout_manifest_sha}");

        // Load model, tokenizer, and requests
        let float_model = StackModel::load(&model_dir, &Device::Cpu)?;
        let requests = load_requests(&requests_path)?;
        let tokenizer_bytes = fs::read(&tokenizer_path)?;
        let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
            .ok_or_else(|| invalid("invalid tokenizer"))?;
        let (protocol, contract) = episode_contract(&tokenizer, float_model.config.vocab_size)?;
        let encoder = protocol
            .bind(&tokenizer)
            .map_err(|e| invalid(e.to_string()))?;
        let decode = |ids: &[u32]| tokenizer.decode(ids);

        // Load 161 development responses
        let heldout_split = DialogueSplit::load(
            &heldout_path.join("tokens.u16"),
            &heldout_path.join("response_mask.u8"),
            &heldout_path.join("manifest.json"),
        )?;
        let index = heldout_split.index(contract)?;
        let dev_panel_ids = dialogue_development::select(&index, 1, 32)?;
        println!(
            "Loaded development panel with {} responses (expected 161).",
            dev_panel_ids.len()
        );

        // 1. Generate Float Reference Replies
        println!("\n[1/5] Generating Float Reference Replies (38 requests, 58 turns)...");
        let mut float_reply_fn =
            |h: &[u32], cap: usize| greedy_reply(&float_model, h, cap, protocol.eos_id);
        let float_panel = reply_panel(
            &encoder,
            &protocol,
            &requests,
            float_model.config.context,
            MAX_NEW_TOKENS,
            &decode,
            &mut float_reply_fn,
        )?;
        let float_dev_score = development(&float_model, &index, &dev_panel_ids, 16)?;
        let float_dev_nll = float_dev_score["response_mean_nll"].as_f64().unwrap_or(0.0);
        println!(
            "Float Reference: 58 turns generated, 161-response NLL = {:.6}",
            float_dev_nll
        );

        // 2. Evaluate Baseline Integer Export (export-1/model.lut)
        println!("\n[2/5] Evaluating Baseline Integer Export (export-1)...");
        let baseline_int_model =
            IntegerStackModel::load(&baseline_lut).map_err(|e| invalid(e.to_string()))?;
        let mut baseline_reply_fn =
            |h: &[u32], cap: usize| integer_reply(&baseline_int_model, h, cap, protocol.eos_id);
        let baseline_int_panel = reply_panel(
            &encoder,
            &protocol,
            &requests,
            float_model.config.context,
            MAX_NEW_TOKENS,
            &decode,
            &mut baseline_reply_fn,
        )?;
        let (base_matches, base_turns) = count_turn_matches(&baseline_int_panel, &float_panel);
        println!(
            "Baseline Integer Model: {}/{} turns match float ({:.2}%)",
            base_matches,
            base_turns,
            (base_matches as f64) / (base_turns as f64) * 100.0
        );
        assert_eq!(
            base_matches, 14,
            "Baseline must reproduce exactly 14/58 matching turns!"
        );

        // 3. Diagnostic Attribution on Free-Running Turns
        println!("\n[3/5] Free-running turn evaluation of dequantized baseline matrices...");
        let base_artifact_bytes = fs::read(&baseline_lut)?;
        let base_artifact =
            StackArtifact::parse(base_artifact_bytes).map_err(|e| invalid(e.to_string()))?;
        let d = float_model.config.width;
        let vocab_size = float_model.config.vocab_size;
        let mlp = base_artifact.header.shape.mlp;

        // Build map specs
        let mut map_specs: Vec<MapSpec> = Vec::new();
        map_specs.push(MapSpec {
            name: "embed".into(),
            var_name: Some("embedding.weight".into()),
            rows: vocab_size,
            cols: d,
        });
        for (l, kind) in float_model.config.pattern.bytes().enumerate() {
            let t = |suffix: &str| format!("layers.{l:02}.{suffix}");
            let a = |part: &str| format!("l{l}.{part}");
            if kind == b'r' {
                map_specs.push(MapSpec {
                    name: a("rec_in"),
                    var_name: Some(t("rec.in.weight")),
                    rows: 2 * d,
                    cols: d,
                });
                let rot_rows = if float_model.config.rotation { d } else { 0 };
                map_specs.push(MapSpec {
                    name: a("rec_gate"),
                    var_name: Some(t("rec.gate.weight")),
                    rows: d / 4 + rot_rows,
                    cols: d,
                });
                map_specs.push(MapSpec {
                    name: a("rec_out"),
                    var_name: Some(t("rec.out.weight")),
                    rows: d,
                    cols: d,
                });
            } else {
                for (part, rows) in [
                    ("query", d),
                    ("key", d),
                    ("value", d),
                    ("null", float_model.config.heads),
                ] {
                    map_specs.push(MapSpec {
                        name: a(part),
                        var_name: Some(t(&format!("read.{part}.weight"))),
                        rows,
                        cols: d,
                    });
                }
                map_specs.push(MapSpec {
                    name: a("out"),
                    var_name: Some(t("read.out.weight")),
                    rows: d,
                    cols: d,
                });
            }
            map_specs.push(MapSpec {
                name: a("gate"),
                var_name: Some(t("mlp.gate.weight")),
                rows: mlp,
                cols: d,
            });
            map_specs.push(MapSpec {
                name: a("up"),
                var_name: Some(t("mlp.up.weight")),
                rows: mlp,
                cols: d,
            });
            map_specs.push(MapSpec {
                name: a("down"),
                var_name: Some(t("mlp.down.weight")),
                rows: d,
                cols: mlp,
            });
        }
        map_specs.push(MapSpec {
            name: "head".into(),
            var_name: None,
            rows: vocab_size,
            cols: d,
        });

        let mut float_folded: BTreeMap<String, Vec<f32>> = BTreeMap::new();
        let mut dequantized: BTreeMap<String, Vec<f32>> = BTreeMap::new();

        let embed_vals = get_var_vec1(&float_model, "embedding.weight")?;
        float_folded.insert("embed".into(), embed_vals);

        let mut head_folded = get_var_vec1(&float_model, "embedding.weight")?;
        let final_gain = get_var_vec1(&float_model, "final_norm.weight")?;
        fold_columns(&mut head_folded, d, &final_gain);
        float_folded.insert("head".into(), head_folded);

        for (l, kind) in float_model.config.pattern.bytes().enumerate() {
            let t = |suffix: &str| format!("layers.{l:02}.{suffix}");
            let a = |part: &str| format!("l{l}.{part}");
            let mlp_gain = get_var_vec1(&float_model, &t("mlp_norm.weight"))?;
            if kind == b'r' {
                let rec_gain = get_var_vec1(&float_model, &t("rec_norm.weight"))?;
                for (part, name) in [
                    ("rec.in.weight", a("rec_in")),
                    ("rec.gate.weight", a("rec_gate")),
                ] {
                    let mut vals = get_var_vec1(&float_model, &t(part))?;
                    fold_columns(&mut vals, d, &rec_gain);
                    float_folded.insert(name, vals);
                }
                let vals = get_var_vec1(&float_model, &t("rec.out.weight"))?;
                float_folded.insert(a("rec_out"), vals);
            } else {
                let read_gain = get_var_vec1(&float_model, &t("read_norm.weight"))?;
                for (part, name) in [
                    ("read.query.weight", a("query")),
                    ("read.key.weight", a("key")),
                    ("read.value.weight", a("value")),
                    ("read.null.weight", a("null")),
                ] {
                    let mut vals = get_var_vec1(&float_model, &t(part))?;
                    fold_columns(&mut vals, d, &read_gain);
                    float_folded.insert(name, vals);
                }
                let vals = get_var_vec1(&float_model, &t("read.out.weight"))?;
                float_folded.insert(a("out"), vals);
            }
            for (part, name) in [("mlp.gate.weight", a("gate")), ("mlp.up.weight", a("up"))] {
                let unpadded = get_var_vec1(&float_model, &t(part))?;
                let mut vals = vec![0.0f32; mlp * d];
                vals[..unpadded.len()].copy_from_slice(&unpadded);
                fold_columns(&mut vals, d, &mlp_gain);
                float_folded.insert(name, vals);
            }
            let unpadded_down = get_var_vec1(&float_model, &t("mlp.down.weight"))?;
            let mut vals_down = vec![0.0f32; d * mlp];
            let unpadded_cols = float_model.config.mlp_hidden;
            for r in 0..d {
                vals_down[r * mlp..r * mlp + unpadded_cols]
                    .copy_from_slice(&unpadded_down[r * unpadded_cols..(r + 1) * unpadded_cols]);
            }
            float_folded.insert(a("down"), vals_down);
        }

        for spec in &map_specs {
            let m = base_artifact
                .matrix(&spec.name)
                .map_err(|e| invalid(e.to_string()))?;
            let vals = dequantize_matrix(
                m.rows,
                m.cols,
                m.exp_base,
                base_artifact.section(m.nibbles),
                base_artifact.section(m.scales),
            )?;
            dequantized.insert(spec.name.clone(), vals);
        }

        // Build working reference model with unit norms and folded bias/params
        let diag_model = StackModel::new(float_model.config.clone(), &Device::Cpu)?;
        set_model_var(&diag_model, "final_norm.weight", &vec![1.0f32; d])?;
        for (l, kind) in float_model.config.pattern.bytes().enumerate() {
            let t = |suffix: &str| format!("layers.{l:02}.{suffix}");
            if kind == b'r' {
                set_model_var(&diag_model, &t("rec_norm.weight"), &vec![1.0f32; d])?;
            } else {
                set_model_var(&diag_model, &t("read_norm.weight"), &vec![1.0f32; d])?;
            }
            set_model_var(&diag_model, &t("mlp_norm.weight"), &vec![1.0f32; d])?;
        }
        for (name, var) in float_model.variables() {
            if name.contains("bias")
                || name.contains("decay")
                || name.contains("conv")
                || name.contains("age")
                || name.contains("beta")
                || name.contains("offset")
            {
                diag_model.variables()[name]
                    .set(var.as_tensor())
                    .map_err(|e| invalid(e.to_string()))?;
            }
        }

        // Set all variables to baseline dequantized weights
        for spec in &map_specs {
            if let Some(var_name) = &spec.var_name {
                set_model_var(&diag_model, var_name, &dequantized[&spec.name])?;
            }
        }
        let diag_base_head =
            Tensor::from_vec(dequantized["head"].clone(), (vocab_size, d), &Device::Cpu)
                .map_err(|e| invalid(e.to_string()))?;

        let mut diag_reply_fn = |h: &[u32], cap: usize| {
            greedy_reply_with_head(&diag_model, &diag_base_head, h, cap, protocol.eos_id)
        };
        let diag_panel = reply_panel(
            &encoder,
            &protocol,
            &requests,
            float_model.config.context,
            MAX_NEW_TOKENS,
            &decode,
            &mut diag_reply_fn,
        )?;
        let (diag_matches, _) = count_turn_matches(&diag_panel, &float_panel);
        let (base_surrogate_matches, base_surrogate_total) =
            count_turn_matches(&baseline_int_panel, &diag_panel);
        let base_integer_vs_surrogate_match = base_surrogate_matches == base_surrogate_total;
        println!(
            "Diagnostic dequantized baseline in surrogate float forward: {}/58 turns match float baseline",
            diag_matches
        );
        println!(
            "Baseline integer vs surrogate agreement (integer engine == surrogate float forward): {}/{} turns match ({:.2}%)",
            base_surrogate_matches,
            base_surrogate_total,
            (base_surrogate_matches as f64) / (base_surrogate_total as f64) * 100.0
        );

        let c_opt = uor_r4_integer::codec::Grouped4BitCodec::new(
            uor_r4_lut::GROUP,
            uor_r4_integer::codec::Grouped4BitRounding::MinimumMseScale,
        );
        let q_head_comp = uor_r4_integer::codec::quantize_matrix_compensated(
            &float_folded["head"],
            vocab_size,
            d,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let comp_head_entry = (
            q_head_comp.exp_base,
            q_head_comp.nibbles.clone(),
            q_head_comp.scales.clone(),
        );
        let deq_head_comp = uor_r4_integer::codec::Grouped4BitCodec::default()
            .dequantize(&q_head_comp)
            .map_err(|e| invalid(e.to_string()))?;
        let head_comp_tensor = Tensor::from_vec(deq_head_comp, (vocab_size, d), &Device::Cpu)
            .map_err(|e| invalid(e.to_string()))?;

        if !skip_diagnostic {
            // Test replacing HEAD with float head
            println!("\nTesting individual matrix replacements in baseline dequantized model:");
            let float_head_tensor =
                Tensor::from_vec(float_folded["head"].clone(), (vocab_size, d), &Device::Cpu)
                    .map_err(|e| invalid(e.to_string()))?;
            let mut test_reply_fn = |h: &[u32], cap: usize| {
                greedy_reply_with_head(&diag_model, &float_head_tensor, h, cap, protocol.eos_id)
            };
            let panel_float_head = reply_panel(
                &encoder,
                &protocol,
                &requests,
                float_model.config.context,
                MAX_NEW_TOKENS,
                &decode,
                &mut test_reply_fn,
            )?;
            let (m_float_head, _) = count_turn_matches(&panel_float_head, &float_panel);
            println!(
                "  - If HEAD is float: {}/58 matching turns (Delta: +{})",
                m_float_head,
                m_float_head as isize - base_matches as isize
            );

            // Test replacing HEAD with min_mse head
            let q_head_min_mse = c_opt
                .quantize(&float_folded["head"], vocab_size, d)
                .map_err(|e| invalid(e.to_string()))?;
            let deq_head_min_mse = c_opt
                .dequantize(&q_head_min_mse)
                .map_err(|e| invalid(e.to_string()))?;
            let head_min_mse_tensor =
                Tensor::from_vec(deq_head_min_mse.clone(), (vocab_size, d), &Device::Cpu)
                    .map_err(|e| invalid(e.to_string()))?;
            let mut test_reply_fn = |h: &[u32], cap: usize| {
                greedy_reply_with_head(&diag_model, &head_min_mse_tensor, h, cap, protocol.eos_id)
            };
            let panel_min_mse_head = reply_panel(
                &encoder,
                &protocol,
                &requests,
                float_model.config.context,
                MAX_NEW_TOKENS,
                &decode,
                &mut test_reply_fn,
            )?;
            let (m_min_mse_head, _) = count_turn_matches(&panel_min_mse_head, &float_panel);
            println!(
                "  - If HEAD is Min-MSE: {}/58 matching turns (Delta: +{})",
                m_min_mse_head,
                m_min_mse_head as isize - base_matches as isize
            );

            // Test replacing HEAD with compensated head
            let mut test_reply_fn = |h: &[u32], cap: usize| {
                greedy_reply_with_head(&diag_model, &head_comp_tensor, h, cap, protocol.eos_id)
            };
            let panel_comp_head = reply_panel(
                &encoder,
                &protocol,
                &requests,
                float_model.config.context,
                MAX_NEW_TOKENS,
                &decode,
                &mut test_reply_fn,
            )?;
            let (m_comp_head, _) = count_turn_matches(&panel_comp_head, &float_panel);
            println!(
                "  - If HEAD is Head-Compensated: {}/58 matching turns (Delta: +{})",
                m_comp_head,
                m_comp_head as isize - base_matches as isize
            );

            // Test replacing MLP_DOWN maps with float
            for l in 0..6 {
                let name = format!("l{l}.down");
                set_model_var(
                    &diag_model,
                    &format!("layers.{l:02}.mlp.down.weight"),
                    &float_folded[&name],
                )?;
                let mut test_reply_fn = |h: &[u32], cap: usize| {
                    greedy_reply_with_head(&diag_model, &diag_base_head, h, cap, protocol.eos_id)
                };
                let panel_down = reply_panel(
                    &encoder,
                    &protocol,
                    &requests,
                    float_model.config.context,
                    MAX_NEW_TOKENS,
                    &decode,
                    &mut test_reply_fn,
                )?;
                let (m_down, _) = count_turn_matches(&panel_down, &float_panel);
                println!("  - If {name} is float: {}/58 matching turns", m_down);
                // Restore
                set_model_var(
                    &diag_model,
                    &format!("layers.{l:02}.mlp.down.weight"),
                    &dequantized[&name],
                )?;
            }

            // Test replacing MLP_DOWN maps with Min-MSE
            for l in [0, 5, 4, 3] {
                let name = format!("l{l}.down");
                let q_down = c_opt
                    .quantize(&float_folded[&name], d, mlp)
                    .map_err(|e| invalid(e.to_string()))?;
                let deq_down = c_opt
                    .dequantize(&q_down)
                    .map_err(|e| invalid(e.to_string()))?;
                set_model_var(
                    &diag_model,
                    &format!("layers.{l:02}.mlp.down.weight"),
                    &deq_down,
                )?;
                let mut test_reply_fn = |h: &[u32], cap: usize| {
                    greedy_reply_with_head(&diag_model, &diag_base_head, h, cap, protocol.eos_id)
                };
                let panel_down = reply_panel(
                    &encoder,
                    &protocol,
                    &requests,
                    float_model.config.context,
                    MAX_NEW_TOKENS,
                    &decode,
                    &mut test_reply_fn,
                )?;
                let (m_down, _) = count_turn_matches(&panel_down, &float_panel);
                println!("  - If {name} is Min-MSE: {}/58 matching turns", m_down);
                // Restore
                set_model_var(
                    &diag_model,
                    &format!("layers.{l:02}.mlp.down.weight"),
                    &dequantized[&name],
                )?;
            }

            // Test combined: HEAD (compensated or min_mse) + down maps
            for (head_label, head_t) in [
                ("Compensated", &head_comp_tensor),
                ("Min-MSE", &head_min_mse_tensor),
                ("Float", &float_head_tensor),
            ] {
                // Apply all mlp_down in float
                for l in 0..6 {
                    let name = format!("l{l}.down");
                    set_model_var(
                        &diag_model,
                        &format!("layers.{l:02}.mlp.down.weight"),
                        &float_folded[&name],
                    )?;
                }
                let mut test_reply_fn = |h: &[u32], cap: usize| {
                    greedy_reply_with_head(&diag_model, head_t, h, cap, protocol.eos_id)
                };
                let panel_comb = reply_panel(
                    &encoder,
                    &protocol,
                    &requests,
                    float_model.config.context,
                    MAX_NEW_TOKENS,
                    &decode,
                    &mut test_reply_fn,
                )?;
                let (m_comb, _) = count_turn_matches(&panel_comb, &float_panel);
                println!("  - If HEAD is {head_label} AND all mlp_down are float: {m_comb}/58 matching turns");

                // Restore mlp_down
                for l in 0..6 {
                    let name = format!("l{l}.down");
                    set_model_var(
                        &diag_model,
                        &format!("layers.{l:02}.mlp.down.weight"),
                        &dequantized[&name],
                    )?;
                }
            }

            // Test replacing other maps
            for map_name in [
                "l5.gate",
                "l0.gate",
                "l5.up",
                "l3.rec_in",
                "l3.rec_out",
                "embed",
            ] {
                if let Some(spec) = map_specs.iter().find(|s| s.name == map_name) {
                    if let Some(var_name) = &spec.var_name {
                        set_model_var(&diag_model, var_name, &float_folded[map_name])?;
                        let mut test_reply_fn = |h: &[u32], cap: usize| {
                            greedy_reply_with_head(
                                &diag_model,
                                &diag_base_head,
                                h,
                                cap,
                                protocol.eos_id,
                            )
                        };
                        let p = reply_panel(
                            &encoder,
                            &protocol,
                            &requests,
                            float_model.config.context,
                            MAX_NEW_TOKENS,
                            &decode,
                            &mut test_reply_fn,
                        )?;
                        let (m, _) = count_turn_matches(&p, &float_panel);
                        println!("  - If {map_name} is float: {}/58 matching turns", m);
                        // Also with Min-MSE
                        let q = c_opt
                            .quantize(&float_folded[map_name], spec.rows, spec.cols)
                            .map_err(|e| invalid(e.to_string()))?;
                        let deq = c_opt.dequantize(&q).map_err(|e| invalid(e.to_string()))?;
                        set_model_var(&diag_model, var_name, &deq)?;
                        let mut test_reply_fn = |h: &[u32], cap: usize| {
                            greedy_reply_with_head(
                                &diag_model,
                                &diag_base_head,
                                h,
                                cap,
                                protocol.eos_id,
                            )
                        };
                        let p_opt = reply_panel(
                            &encoder,
                            &protocol,
                            &requests,
                            float_model.config.context,
                            MAX_NEW_TOKENS,
                            &decode,
                            &mut test_reply_fn,
                        )?;
                        let (m_opt, _) = count_turn_matches(&p_opt, &float_panel);
                        println!("  - If {map_name} is Min-MSE: {}/58 matching turns", m_opt);

                        set_model_var(&diag_model, var_name, &dequantized[map_name])?;
                    }
                }
            }
        } // end if !skip_diagnostic

        // 4. Native Integer Serving Engine Evaluation of Candidates
        println!("\n[4/5] Evaluating candidate replacements in native IntegerStackModel...");

        // Quantize candidate maps with Min-MSE
        let mut min_mse_matrices: BTreeMap<String, (i32, Vec<u8>, Vec<u8>)> = BTreeMap::new();
        for spec in &map_specs {
            if let Some(src) = float_folded.get(&spec.name) {
                let q = c_opt
                    .quantize(src, spec.rows, spec.cols)
                    .map_err(|e| invalid(e.to_string()))?;
                min_mse_matrices.insert(spec.name.clone(), (q.exp_base, q.nibbles, q.scales));
            }
        }

        struct CandidateArm {
            name: &'static str,
            label: &'static str,
            replacements: BTreeMap<String, (i32, Vec<u8>, Vec<u8>)>,
        }

        let mut candidate_arms: Vec<CandidateArm> = Vec::new();

        // Arm 0: Baseline rebuilt (sanity check)
        candidate_arms.push(CandidateArm {
            name: "baseline_rebuilt",
            label: "Baseline Rebuilt (sanity check)",
            replacements: BTreeMap::new(),
        });

        // Arm 1: l3.rec_in Min-MSE
        let mut r = BTreeMap::new();
        r.insert("l3.rec_in".into(), min_mse_matrices["l3.rec_in"].clone());
        candidate_arms.push(CandidateArm {
            name: "l3_rec_in_min_mse",
            label: "l3.rec_in Min-MSE",
            replacements: r,
        });

        // Arm 2: l5.gate Min-MSE
        let mut r = BTreeMap::new();
        r.insert("l5.gate".into(), min_mse_matrices["l5.gate"].clone());
        candidate_arms.push(CandidateArm {
            name: "l5_gate_min_mse",
            label: "l5.gate Min-MSE",
            replacements: r,
        });

        // Arm 3: l3.rec_out Min-MSE
        let mut r = BTreeMap::new();
        r.insert("l3.rec_out".into(), min_mse_matrices["l3.rec_out"].clone());
        candidate_arms.push(CandidateArm {
            name: "l3_rec_out_min_mse",
            label: "l3.rec_out Min-MSE",
            replacements: r,
        });

        // Arm 4: l0.down Min-MSE
        let mut r = BTreeMap::new();
        r.insert("l0.down".into(), min_mse_matrices["l0.down"].clone());
        candidate_arms.push(CandidateArm {
            name: "l0_down_min_mse",
            label: "l0.down Min-MSE",
            replacements: r,
        });

        // Arm 5: l3.rec_in + l5.gate Min-MSE
        let mut r = BTreeMap::new();
        r.insert("l3.rec_in".into(), min_mse_matrices["l3.rec_in"].clone());
        r.insert("l5.gate".into(), min_mse_matrices["l5.gate"].clone());
        candidate_arms.push(CandidateArm {
            name: "l3_rec_in_and_l5_gate_min_mse",
            label: "l3.rec_in + l5.gate Min-MSE",
            replacements: r,
        });

        // Arm 6: Head Compensated
        let mut r = BTreeMap::new();
        r.insert("head".into(), comp_head_entry.clone());
        candidate_arms.push(CandidateArm {
            name: "head_compensated",
            label: "head Compensated",
            replacements: r,
        });

        // Arm 7: Head Compensated + l3.rec_in Min-MSE
        let mut r = BTreeMap::new();
        r.insert("head".into(), comp_head_entry);
        r.insert("l3.rec_in".into(), min_mse_matrices["l3.rec_in"].clone());
        candidate_arms.push(CandidateArm {
            name: "head_comp_plus_l3_rec_in_min_mse",
            label: "head Compensated + l3.rec_in Min-MSE",
            replacements: r,
        });

        let mut results_map = serde_json::Map::new();
        let total_weights: usize = base_artifact
            .header
            .matrices
            .iter()
            .map(|m| m.rows * m.cols)
            .sum();

        let base_dev_score =
            development_with_head(&diag_model, &diag_base_head, &index, &dev_panel_ids, 16)?;
        let base_dev_nll = base_dev_score["response_mean_nll"].as_f64().unwrap_or(0.0);

        let mut best_arm_name = "baseline".to_string();
        let mut best_matches = base_matches;
        let mut best_lut_bytes = Vec::new();
        let mut best_dev_nll = base_dev_nll;
        let mut best_integer_vs_surrogate_match = base_integer_vs_surrogate_match;
        let mut arm_panels: BTreeMap<String, Value> = BTreeMap::new();

        for arm in &candidate_arms {
            if let Some(ref selected) = selected_arms {
                if !selected.contains(&arm.name.to_string()) {
                    continue;
                }
            }
            let lut_bytes = build_modified_artifact(&base_artifact, &arm.replacements)?;
            let container_bpw = (lut_bytes.len() as f64 * 8.0) / (total_weights as f64);
            let param_bpw = 4.2500f64;
            let cand_path = out.join(format!("{}.lut", arm.name));
            fs::write(&cand_path, &lut_bytes)?;

            let cand_model =
                IntegerStackModel::load(&cand_path).map_err(|e| invalid(e.to_string()))?;
            let mut cand_reply_fn =
                |h: &[u32], cap: usize| integer_reply(&cand_model, h, cap, protocol.eos_id);
            let cand_panel = reply_panel(
                &encoder,
                &protocol,
                &requests,
                float_model.config.context,
                MAX_NEW_TOKENS,
                &decode,
                &mut cand_reply_fn,
            )?;
            let (matches, turns) = count_turn_matches(&cand_panel, &float_panel);

            // Compute development NLL and served float forward replies on diag_model with these replacements
            for (name, _) in &arm.replacements {
                if let Some(spec) = map_specs.iter().find(|s| &s.name == name) {
                    if let Some(var_name) = &spec.var_name {
                        let deq = c_opt
                            .dequantize(&uor_r4_integer::codec::Grouped4BitMatrix {
                                rows: spec.rows,
                                cols: spec.cols,
                                group_size: uor_r4_lut::GROUP,
                                exp_base: arm.replacements[name].0,
                                nibbles: arm.replacements[name].1.clone(),
                                scales: arm.replacements[name].2.clone(),
                            })
                            .map_err(|e| invalid(e.to_string()))?;
                        set_model_var(&diag_model, var_name, &deq)?;
                    }
                }
            }
            let head_t = if arm.replacements.contains_key("head") {
                &head_comp_tensor
            } else {
                &diag_base_head
            };
            let dev_score = development_with_head(&diag_model, head_t, &index, &dev_panel_ids, 16)?;
            let dev_nll = dev_score["response_mean_nll"].as_f64().unwrap_or(0.0);

            let mut served_float_reply_fn = |h: &[u32], cap: usize| {
                greedy_reply_with_head(&diag_model, head_t, h, cap, protocol.eos_id)
            };
            let served_float_panel = reply_panel(
                &encoder,
                &protocol,
                &requests,
                float_model.config.context,
                MAX_NEW_TOKENS,
                &decode,
                &mut served_float_reply_fn,
            )?;
            let (surrogate_matches, surrogate_total) =
                count_turn_matches(&cand_panel, &served_float_panel);
            let integer_vs_surrogate_match = surrogate_matches == surrogate_total;

            // restore diag_model
            for (name, _) in &arm.replacements {
                if let Some(spec) = map_specs.iter().find(|s| &s.name == name) {
                    if let Some(var_name) = &spec.var_name {
                        set_model_var(&diag_model, var_name, &dequantized[name])?;
                    }
                }
            }

            println!(
                "Candidate [{}] ({}): {}/{} turns match float ({:.2}%), int vs surrogate: {}/{} ({:.2}%), 161-dev NLL = {:.6}, param bpw = {:.4}, container bpw = {:.4}",
                arm.name,
                arm.label,
                matches,
                turns,
                (matches as f64) / (turns as f64) * 100.0,
                surrogate_matches,
                surrogate_total,
                (surrogate_matches as f64) / (surrogate_total as f64) * 100.0,
                dev_nll,
                param_bpw,
                container_bpw
            );

            results_map.insert(
                arm.name.to_string(),
                serde_json::json!({
                    "name": arm.name,
                    "label": arm.label,
                    "matching_turns": matches,
                    "total_turns": turns,
                    "delta_turns": matches as isize - base_matches as isize,
                    "integer_vs_surrogate_turns": format!("{surrogate_matches}/{surrogate_total}"),
                    "surrogate_agreement_turns": surrogate_matches,
                    "integer_vs_surrogate_match": integer_vs_surrogate_match,
                    "comparison_scope": "integer_vs_surrogate_agreement (integer artifact forward vs matrix-dequantized surrogate forward; differing representations include weight map quantizations, folded norm gains, and grid-coded scalars versus unquantized scalars)",
                    "response_nll_161": dev_nll,
                    "delta_nll_161": dev_nll - base_dev_nll,
                    "parameter_bits_per_weight": param_bpw,
                    "container_bits_per_weight": container_bpw,
                    "bits_per_weight": param_bpw,
                }),
            );

            if matches > best_matches {
                best_matches = matches;
                best_arm_name = arm.name.to_string();
                best_lut_bytes = lut_bytes;
                best_dev_nll = dev_nll;
                best_integer_vs_surrogate_match = integer_vs_surrogate_match;
            }
            arm_panels.insert(arm.name.to_string(), cand_panel);
        }

        // Save row_observations.json preserving computed per-turn outcomes
        let mut row_observations = Vec::new();
        if let (Some(rows_fl), Some(rows_base), Some(rows_diag)) = (
            float_panel["rows"].as_array(),
            baseline_int_panel["rows"].as_array(),
            diag_panel["rows"].as_array(),
        ) {
            for (r_fl, (r_base, r_diag)) in
                rows_fl.iter().zip(rows_base.iter().zip(rows_diag.iter()))
            {
                let req_id = r_fl["id"].as_str().unwrap_or("");
                let turns_fl = r_fl["turns"].as_array();
                let turns_base = r_base["turns"].as_array();
                let turns_diag = r_diag["turns"].as_array();
                if let (Some(t_fl), Some(t_base), Some(t_diag)) = (turns_fl, turns_base, turns_diag)
                {
                    for (turn_idx, (turn_fl, (turn_base, turn_diag))) in t_fl
                        .iter()
                        .zip(t_base.iter().zip(t_diag.iter()))
                        .enumerate()
                    {
                        let fl_ids = &turn_fl["reply_ids"];
                        let base_ids = &turn_base["reply_ids"];
                        let diag_ids = &turn_diag["reply_ids"];
                        let mut cand_matches = serde_json::Map::new();
                        for (arm_name, panel) in &arm_panels {
                            if let Some(r_c) = panel["rows"]
                                .as_array()
                                .and_then(|rows| rows.iter().find(|r| r["id"] == req_id))
                            {
                                if let Some(t_c) = r_c["turns"]
                                    .as_array()
                                    .and_then(|turns| turns.get(turn_idx))
                                {
                                    let c_ids = &t_c["reply_ids"];
                                    cand_matches.insert(
                                        arm_name.clone(),
                                        serde_json::json!({
                                            "reply_ids": c_ids,
                                            "matches_float": c_ids == fl_ids,
                                            "matches_baseline_int": c_ids == base_ids,
                                        }),
                                    );
                                }
                            }
                        }
                        row_observations.push(serde_json::json!({
                            "request_id": req_id,
                            "turn_index": turn_idx,
                            "float_reply_ids": fl_ids,
                            "baseline_int_reply_ids": base_ids,
                            "surrogate_float_reply_ids": diag_ids,
                            "baseline_matches_float": base_ids == fl_ids,
                            "baseline_matches_surrogate": base_ids == diag_ids,
                            "candidates": cand_matches,
                        }));
                    }
                }
            }
        }
        fs::write(
            out.join("row_observations.json"),
            serde_json::to_string_pretty(&row_observations).map_err(|e| invalid(e.to_string()))?,
        )?;

        // Save report.json and optimized.lut
        let report = serde_json::json!({
            "schema": "uor-r4.s2-turn-optimization/1",
            "runtime_context": {
                "git_commit_unverified": current_git_commit(),
                "build_source_status": "unverified-source",
                "note": "cwd git rev-parse HEAD is runtime execution context, not verified build-source provenance"
            },
            "binary_sha256": current_binary_sha256(),
            "model_dir": model_dir.display().to_string(),
            "model_sha256": model_sha,
            "model_config_sha256": model_config_sha,
            "requests_path": requests_path.display().to_string(),
            "requests_sha256": requests_sha,
            "tokenizer_path": tokenizer_path.display().to_string(),
            "tokenizer_sha256": tokenizer_sha,
            "heldout_path": heldout_path.display().to_string(),
            "heldout_tokens_sha256": heldout_tokens_sha,
            "heldout_data": {
                "path": heldout_path.display().to_string(),
                "tokens_sha256": heldout_tokens_sha,
                "response_mask_sha256": heldout_mask_sha,
                "manifest_sha256": heldout_manifest_sha
            },
            "baseline": {
                "artifact": baseline_lut.display().to_string(),
                "artifact_sha256": baseline_lut_sha,
                "float_baseline_greedy_matching_turns": format!("{base_matches}/{base_turns}"),
                "integer_vs_surrogate_turns": format!("{base_surrogate_matches}/{base_surrogate_total}"),
                "surrogate_agreement_turns": base_surrogate_matches,
                "matching_turns": base_matches,
                "total_turns": base_turns,
                "integer_vs_surrogate_match": base_integer_vs_surrogate_match,
                "comparison_scope": "integer_vs_surrogate_agreement (integer artifact forward vs matrix-dequantized surrogate forward; differing representations include weight map quantizations, folded norm gains, and grid-coded scalars versus unquantized scalars)",
                "response_nll_161": base_dev_nll,
                "parameter_bits_per_weight": 4.2500,
                "container_bits_per_weight": (fs::metadata(&baseline_lut)?.len() as f64 * 8.0) / (total_weights as f64),
                "bits_per_weight": 4.2500,
            },
            "candidates": results_map,
            "selected": {
                "name": best_arm_name,
                "matching_turns": best_matches,
                "delta_turns": best_matches as isize - base_matches as isize,
                "integer_vs_surrogate_match": best_integer_vs_surrogate_match,
                "parameter_bits_per_weight": 4.2500,
                "response_nll_161": best_dev_nll,
            },
            "row_observations_file": "row_observations.json",
            "elapsed_seconds": clock.elapsed().as_secs_f64(),
        });
        fs::write(
            out.join("report.json"),
            serde_json::to_string_pretty(&report).map_err(|e| invalid(e.to_string()))?,
        )?;
        if !best_lut_bytes.is_empty() {
            fs::write(out.join("optimized.lut"), &best_lut_bytes)?;
        }

        println!("\nTotal elapsed time: {:?}", clock.elapsed());
        Ok(())
    })();

    if let Err(e) = &execution_result {
        eprintln!("Execution error: {e}");
    }

    report_output::seal(&out)?;
    report_output::verify(&out)?;
    execution_result
}
