//! S2: Behavior-Sensitive Distortion Study (Attribution & Map Ranking)
//!
//! Evaluates the S2 dialogue model to localize which weight maps cause greedy
//! dialogue replies to diverge from the float model:
//!   - (i) Activation-weighted output error per map:
//!         E ||(W - \hat{W}) x||^2 = tr(E \Sigma_x E^T)
//!         via `hidden_with_capture` on heldout windows from chat-v0 tokens.u16.
//!   - (ii) Greedy-flip attribution per map group:
//!         Quantize one group at a time with the rest in float, counting greedy flips
//!         on the 58 panel turns under teacher-forced prefixes from development-requests.json.
//!   - (iii) Ranking and one-change recommendation.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::{Device, Tensor};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::report_output;
use uor_r4_lut::format::StackArtifact;
use uor_r4_tokenizer::dialogue::DialogueProtocol;
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::geometric_stack::{StackModel, StackSite};
use uor_r4_training::lut_export::dequantize_matrix;
use uor_r4_training::stack_dialogue::{load_requests, Reply};

const MAX_NEW_TOKENS: usize = 32;
const WINDOW_TIME: usize = 256;

struct ParsedArgs {
    model_dir: PathBuf,
    artifact_path: PathBuf,
    heldout_path: PathBuf,
    requests_path: PathBuf,
    tokenizer_path: PathBuf,
    windows: usize,
    out: PathBuf,
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

fn parse_cli_args() -> Result<ParsedArgs, Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if let Some((k, v)) = arg.split_once('=') {
            map.insert(k.trim_start_matches('-').to_string(), v.to_string());
            i += 1;
        } else if arg.starts_with("--") {
            let key = arg.trim_start_matches('-').to_string();
            if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                map.insert(key, args[i + 1].clone());
                i += 2;
            } else {
                map.insert(key, "true".to_string());
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    let model_dir = PathBuf::from(
        map.get("model")
            .cloned()
            .or_else(|| std::env::var("UOR_MODEL_DIR").ok())
            .unwrap_or_else(|| {
                "/Volumes/UOR-Workspace/uor-r4-lab/claude-s2-dialogue-baseline/dialogue-1/model"
                    .to_string()
            }),
    );
    let artifact_path = PathBuf::from(
        map.get("artifact")
            .cloned()
            .or_else(|| std::env::var("UOR_BASELINE_LUT").ok())
            .unwrap_or_else(|| {
                "/Volumes/UOR-Workspace/uor-r4-lab/claude-s2-dialogue-baseline/export-1/model.lut"
                    .to_string()
            }),
    );
    let heldout_path = PathBuf::from(
        map.get("heldout")
            .cloned()
            .or_else(|| std::env::var("UOR_HELDOUT").ok())
            .unwrap_or_else(|| {
                "/Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/prepared/heldout/tokens.u16"
                    .to_string()
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
    let windows: usize = match map.get("windows") {
        Some(s) => {
            let w = s
                .parse::<usize>()
                .map_err(|e| format!("invalid windows '{}': {}", s, e))?;
            if w == 0 {
                return Err("windows must be positive (> 0)".into());
            }
            w
        }
        None => 32,
    };
    let out = PathBuf::from(
        map.get("out")
            .ok_or("missing required argument 'out' (e.g. out=/path/to/report-dir)")?,
    );

    Ok(ParsedArgs {
        model_dir,
        artifact_path,
        heldout_path,
        requests_path,
        tokenizer_path,
        windows,
        out,
    })
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
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

fn read_u16_tokens(path: &Path, vocab_size: usize) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    if bytes.starts_with(b"UORT") {
        let reader = uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader::open(path)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        if reader.vocab_size() as usize > vocab_size {
            return Err(format!(
                "{} declares a larger vocabulary ({}) than model ({vocab_size})",
                path.display(),
                reader.vocab_size()
            )
            .into());
        }
        let tokens: Vec<u32> = reader.as_slice().iter().map(|&id| u32::from(id)).collect();
        if tokens.iter().any(|&id| id as usize >= vocab_size) {
            return Err(format!("{} has ids outside the vocabulary", path.display()).into());
        }
        return Ok(tokens);
    }
    if bytes.len() % 2 != 0 {
        return Err(format!("token file {} has odd byte count", path.display()).into());
    }
    let tokens: Vec<u32> = bytes
        .chunks_exact(2)
        .map(|b| u32::from(u16::from_le_bytes([b[0], b[1]])))
        .collect();
    for &id in &tokens {
        if id as usize >= vocab_size {
            return Err(format!(
                "token id {id} exceeds vocab size {vocab_size} in {}",
                path.display()
            )
            .into());
        }
    }
    Ok(tokens)
}

fn set_model_var(
    model: &StackModel,
    name: &str,
    values: &[f32],
) -> Result<(), Box<dyn std::error::Error>> {
    let var = model
        .variables()
        .get(name)
        .ok_or_else(|| format!("model has no variable {name}"))?;
    let shape = var.as_tensor().shape();
    let dims = shape.dims();
    let unpadded_values = if dims.len() == 2 && (dims[0] * dims[1] != values.len()) {
        let (rows, cols) = (dims[0], dims[1]);
        if values.len() % cols == 0 && values.len() / cols > rows {
            // Padded rows (gate / up)
            values[..rows * cols].to_vec()
        } else if values.len() % rows == 0 && values.len() / rows > cols {
            // Padded cols (down)
            let padded_cols = values.len() / rows;
            let mut unpadded = Vec::with_capacity(rows * cols);
            for r in 0..rows {
                unpadded.extend_from_slice(&values[r * padded_cols..r * padded_cols + cols]);
            }
            unpadded
        } else {
            return Err(format!(
                "shape mismatch for {name}: var is {:?}, values has {}",
                shape,
                values.len()
            )
            .into());
        }
    } else {
        values.to_vec()
    };
    var.set(&Tensor::from_vec(
        unpadded_values,
        var.as_tensor().shape(),
        var.as_tensor().device(),
    )?)?;
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

/// Fast greedy rollout using only the last hidden state for logit generation.
fn greedy_reply_with_head(
    model: &StackModel,
    head: &Tensor,
    history: &[u32],
    cap: usize,
    eos: u32,
) -> Result<Reply, Box<dyn std::error::Error>> {
    let mut window = history.to_vec();
    let mut ids = Vec::with_capacity(cap);
    for _ in 0..cap {
        if window.len() > model.config.context {
            return Err("the reply outgrew the context".into());
        }
        let hidden = model.hidden(&window, 1, window.len())?;
        let last_hidden = hidden.get(window.len() - 1)?;
        let logits = last_hidden
            .unsqueeze(0)?
            .matmul(&head.t()?)?
            .squeeze(0)?
            .to_vec1::<f32>()?;
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
        trace: Vec::new(),
        stopped_at: None,
        copy_stop: None,
    })
}

#[allow(dead_code)]
struct MapSpec {
    name: String,
    var_name: Option<String>,
    rows: usize,
    cols: usize,
    site: Option<StackSite>,
    is_head: bool,
    is_embed: bool,
    layer: Option<usize>,
    component: String,
}

#[allow(dead_code)]
struct TeacherTurn {
    turn_index: usize,
    prefix_ids: Vec<u32>,
    float_reply_ids: Vec<u32>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_cli_args()?;
    println!("=== S2: Behavior-Sensitive Distortion Study ===");
    println!("Model:      {}", args.model_dir.display());
    println!("Artifact:   {}", args.artifact_path.display());
    println!("Heldout:    {}", args.heldout_path.display());
    println!("Requests:   {}", args.requests_path.display());
    println!("Tokenizer:  {}", args.tokenizer_path.display());
    println!("Windows:    {}", args.windows);
    println!("Output Dir: {}", args.out.display());

    report_output::claim(&args.out)?;

    let execution_result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let run_started = Instant::now();

        // 1. Verify files & identities
        let model_sha = sha256_file(&args.model_dir.join("model.safetensors"))?;
        let model_config_sha = sha256_file(&args.model_dir.join("config.json"))?;
        let artifact_sha = sha256_file(&args.artifact_path)?;
        let requests_sha = sha256_file(&args.requests_path)?;
        let tokenizer_sha = sha256_file(&args.tokenizer_path)?;
        let heldout_sha = sha256_file(&args.heldout_path)?;
        println!("Model SHA256:        {model_sha}");
        println!("Model Config SHA256: {model_config_sha}");
        println!("Artifact SHA256:     {artifact_sha}");
        println!("Requests SHA256:     {requests_sha}");
        println!("Tokenizer SHA256:    {tokenizer_sha}");
        println!("Heldout SHA256:      {heldout_sha}");

        // 2. Load model and artifact
        let float_model = StackModel::load(&args.model_dir, &Device::Cpu)?;
        let artifact_bytes = fs::read(&args.artifact_path)?;
        let artifact = StackArtifact::parse(artifact_bytes)?;
        let d = float_model.config.width;
        let vocab_size = float_model.config.vocab_size;
        let mlp = artifact.header.shape.mlp;

        // 3. Inventory all 42 weight matrices
        let mut map_specs: Vec<MapSpec> = Vec::new();

        map_specs.push(MapSpec {
            name: "embed".into(),
            var_name: Some("embedding.weight".into()),
            rows: vocab_size,
            cols: d,
            site: None,
            is_head: false,
            is_embed: true,
            layer: None,
            component: "embed".into(),
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
                    site: Some(StackSite::Recurrence(l)),
                    is_head: false,
                    is_embed: false,
                    layer: Some(l),
                    component: "rec_in".into(),
                });
                let rot_rows = if float_model.config.rotation { d } else { 0 };
                map_specs.push(MapSpec {
                    name: a("rec_gate"),
                    var_name: Some(t("rec.gate.weight")),
                    rows: d / 4 + rot_rows,
                    cols: d,
                    site: Some(StackSite::Recurrence(l)),
                    is_head: false,
                    is_embed: false,
                    layer: Some(l),
                    component: "rec_gate".into(),
                });
                map_specs.push(MapSpec {
                    name: a("rec_out"),
                    var_name: Some(t("rec.out.weight")),
                    rows: d,
                    cols: d,
                    site: Some(StackSite::RecurrenceOut(l)),
                    is_head: false,
                    is_embed: false,
                    layer: Some(l),
                    component: "rec_out".into(),
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
                        site: Some(StackSite::Read(l)),
                        is_head: false,
                        is_embed: false,
                        layer: Some(l),
                        component: "read_qkv".into(),
                    });
                }
                map_specs.push(MapSpec {
                    name: a("out"),
                    var_name: Some(t("read.out.weight")),
                    rows: d,
                    cols: d,
                    site: Some(StackSite::ReadOut(l)),
                    is_head: false,
                    is_embed: false,
                    layer: Some(l),
                    component: "read_out".into(),
                });
            }
            map_specs.push(MapSpec {
                name: a("gate"),
                var_name: Some(t("mlp.gate.weight")),
                rows: mlp,
                cols: d,
                site: Some(StackSite::Mlp(l)),
                is_head: false,
                is_embed: false,
                layer: Some(l),
                component: "mlp_gate".into(),
            });
            map_specs.push(MapSpec {
                name: a("up"),
                var_name: Some(t("mlp.up.weight")),
                rows: mlp,
                cols: d,
                site: Some(StackSite::Mlp(l)),
                is_head: false,
                is_embed: false,
                layer: Some(l),
                component: "mlp_up".into(),
            });
            map_specs.push(MapSpec {
                name: a("down"),
                var_name: Some(t("mlp.down.weight")),
                rows: d,
                cols: mlp,
                site: Some(StackSite::Down(l)),
                is_head: false,
                is_embed: false,
                layer: Some(l),
                component: "mlp_down".into(),
            });
        }

        map_specs.push(MapSpec {
            name: "head".into(),
            var_name: None,
            rows: vocab_size,
            cols: d,
            site: Some(StackSite::Head),
            is_head: true,
            is_embed: false,
            layer: None,
            component: "head".into(),
        });

        println!("Identified {} weight maps to evaluate.", map_specs.len());

        // 4. Extract Float Folded Weights (W) and Dequantized Artifact Weights (W_hat)
        let mut float_folded: BTreeMap<String, Vec<f32>> = BTreeMap::new();
        let mut dequantized: BTreeMap<String, Vec<f32>> = BTreeMap::new();

        // embed
        let embed_vals = float_model.variables()["embedding.weight"]
            .as_tensor()
            .flatten_all()?
            .to_vec1::<f32>()?;
        float_folded.insert("embed".into(), embed_vals);

        // head
        let mut head_folded = float_model.variables()["embedding.weight"]
            .as_tensor()
            .flatten_all()?
            .to_vec1::<f32>()?;
        let final_gain = float_model.variables()["final_norm.weight"]
            .as_tensor()
            .flatten_all()?
            .to_vec1::<f32>()?;
        fold_columns(&mut head_folded, d, &final_gain);
        float_folded.insert("head".into(), head_folded);

        // Layer weights
        for (l, kind) in float_model.config.pattern.bytes().enumerate() {
            let t = |suffix: &str| format!("layers.{l:02}.{suffix}");
            let a = |part: &str| format!("l{l}.{part}");
            let mlp_gain = float_model.variables()[&t("mlp_norm.weight")]
                .as_tensor()
                .flatten_all()?
                .to_vec1::<f32>()?;

            if kind == b'r' {
                let rec_gain = float_model.variables()[&t("rec_norm.weight")]
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                for (part, name) in [
                    ("rec.in.weight", a("rec_in")),
                    ("rec.gate.weight", a("rec_gate")),
                ] {
                    let mut vals = float_model.variables()[&t(part)]
                        .as_tensor()
                        .flatten_all()?
                        .to_vec1::<f32>()?;
                    fold_columns(&mut vals, d, &rec_gain);
                    float_folded.insert(name, vals);
                }
                let vals = float_model.variables()[&t("rec.out.weight")]
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                float_folded.insert(a("rec_out"), vals);
            } else {
                let read_gain = float_model.variables()[&t("read_norm.weight")]
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                for (part, name) in [
                    ("read.query.weight", a("query")),
                    ("read.key.weight", a("key")),
                    ("read.value.weight", a("value")),
                    ("read.null.weight", a("null")),
                ] {
                    let mut vals = float_model.variables()[&t(part)]
                        .as_tensor()
                        .flatten_all()?
                        .to_vec1::<f32>()?;
                    fold_columns(&mut vals, d, &read_gain);
                    float_folded.insert(name, vals);
                }
                let vals = float_model.variables()[&t("read.out.weight")]
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                float_folded.insert(a("out"), vals);
            }

            for (part, name) in [("mlp.gate.weight", a("gate")), ("mlp.up.weight", a("up"))] {
                let unpadded = float_model.variables()[&t(part)]
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                let mut vals = vec![0.0f32; mlp * d];
                vals[..unpadded.len()].copy_from_slice(&unpadded);
                fold_columns(&mut vals, d, &mlp_gain);
                float_folded.insert(name, vals);
            }

            let unpadded_down = float_model.variables()[&t("mlp.down.weight")]
                .as_tensor()
                .flatten_all()?
                .to_vec1::<f32>()?;
            let mut vals_down = vec![0.0f32; d * mlp];
            let unpadded_cols = float_model.config.mlp_hidden;
            for r in 0..d {
                vals_down[r * mlp..r * mlp + unpadded_cols]
                    .copy_from_slice(&unpadded_down[r * unpadded_cols..(r + 1) * unpadded_cols]);
            }
            float_folded.insert(a("down"), vals_down);
        }

        // Dequantize from artifact
        for spec in &map_specs {
            let m = artifact.matrix(&spec.name)?;
            let vals = dequantize_matrix(
                m.rows,
                m.cols,
                m.exp_base,
                artifact.section(m.nibbles),
                artifact.section(m.scales),
            )?;
            dequantized.insert(spec.name.clone(), vals);
        }

        println!("Loaded folded float weights and dequantized artifact weights for all maps.");

        // -------------------------------------------------------------------
        // (C)(i) Activation-Weighted Output Error: E ||(W - \hat{W})x||^2 = tr(E \Sigma_x E^T)
        // -------------------------------------------------------------------
        println!("\n[1/3] Computing Activation Second Moments tr(E \\Sigma_x E^T) via hidden_with_capture...");
        let heldout_tokens = read_u16_tokens(&args.heldout_path, vocab_size)?;
        if heldout_tokens.len() < WINDOW_TIME {
            return Err(format!(
                "heldout token stream too short: {} tokens < WINDOW_TIME ({})",
                heldout_tokens.len(),
                WINDOW_TIME
            )
            .into());
        }
        let mut moments: BTreeMap<StackSite, (usize, Vec<f64>)> = BTreeMap::new();
        let total_windows = args.windows;
        let span = heldout_tokens.len() - WINDOW_TIME;

        for w in 0..total_windows {
            let start = if total_windows == 1 {
                0
            } else {
                w * span / (total_windows - 1)
            };
            let window_ids = &heldout_tokens[start..start + WINDOW_TIME];
            float_model.hidden_with_capture(window_ids, 1, WINDOW_TIME, &mut |site, x| {
                let x = x.contiguous()?;
                let cols = x.dim(1)?;
                let gram = x
                    .t()?
                    .contiguous()?
                    .matmul(&x)?
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                let entry = moments
                    .entry(site)
                    .or_insert_with(|| (cols, vec![0.0f64; cols * cols]));
                for (sum, &val) in entry.1.iter_mut().zip(&gram) {
                    *sum += f64::from(val);
                }
                Ok(())
            })?;
        }

        let total_positions = (total_windows * WINDOW_TIME) as f64;
        println!(
            "Captured moments over {total_positions} tokens across {} sites.",
            moments.len()
        );

        let mut activation_errors: BTreeMap<String, Value> = BTreeMap::new();

        for spec in &map_specs {
            let w_fl = &float_folded[&spec.name];
            let w_deq = &dequantized[&spec.name];
            let rows = spec.rows;
            let cols = spec.cols;

            // E = W - W_hat
            let mut e_mat = vec![0.0f64; rows * cols];
            for (i, e) in e_mat.iter_mut().enumerate() {
                *e = f64::from(w_fl[i]) - f64::from(w_deq[i]);
            }

            if let Some(site) = spec.site {
                if let Some((m_cols, moment_sum)) = moments.get(&site) {
                    let mut sigma = vec![0.0f64; cols * cols];
                    let width = *m_cols;
                    for r in 0..width {
                        for c in 0..width {
                            if r < cols && c < cols {
                                sigma[r * cols + c] = moment_sum[r * width + c] / total_positions;
                            }
                        }
                    }

                    // tr(E \Sigma_x E^T) = sum_r E_r \Sigma_x E_r^T
                    let mut tr_e_sigma = 0.0f64;
                    let mut tr_w_sigma = 0.0f64;
                    for r in 0..rows {
                        let e_row = &e_mat[r * cols..(r + 1) * cols];
                        let mut w_row = vec![0.0f64; cols];
                        for c in 0..cols {
                            w_row[c] = f64::from(w_fl[r * cols + c]);
                        }

                        // y = E_r * \Sigma_x
                        let mut y_e = vec![0.0f64; cols];
                        let mut y_w = vec![0.0f64; cols];
                        for c in 0..cols {
                            let mut sum_e = 0.0f64;
                            let mut sum_w = 0.0f64;
                            for k in 0..cols {
                                let sig = sigma[k * cols + c];
                                sum_e += e_row[k] * sig;
                                sum_w += w_row[k] * sig;
                            }
                            y_e[c] = sum_e;
                            y_w[c] = sum_w;
                        }

                        for c in 0..cols {
                            tr_e_sigma += y_e[c] * e_row[c];
                            tr_w_sigma += y_w[c] * w_row[c];
                        }
                    }

                    let rel_err = if tr_w_sigma > 1e-12 {
                        (tr_e_sigma / tr_w_sigma).sqrt()
                    } else {
                        0.0
                    };

                    activation_errors.insert(
                        spec.name.clone(),
                        json!({
                            "tr_e_sigma_e_t": tr_e_sigma,
                            "tr_w_sigma_w_t": tr_w_sigma,
                            "relative_error": rel_err,
                            "rows": rows,
                            "cols": cols,
                            "component": spec.component,
                            "layer": spec.layer,
                        }),
                    );
                }
            } else {
                // embed has no activation moment; report Frobenius error
                let frob_e: f64 = e_mat.iter().map(|&v| v * v).sum();
                let frob_w: f64 = w_fl.iter().map(|&v| f64::from(v) * f64::from(v)).sum();
                let rel_err = if frob_w > 1e-12 {
                    (frob_e / frob_w).sqrt()
                } else {
                    0.0
                };
                activation_errors.insert(
                    spec.name.clone(),
                    json!({
                        "tr_e_sigma_e_t": frob_e / rows as f64,
                        "tr_w_sigma_w_t": frob_w / rows as f64,
                        "relative_error": rel_err,
                        "rows": rows,
                        "cols": cols,
                        "component": spec.component,
                        "layer": spec.layer,
                    }),
                );
            }
        }

        // -------------------------------------------------------------------
        // (C)(ii) Greedy-Flip Attribution on 58 Panel Turns
        // -------------------------------------------------------------------
        println!("\n[2/3] Building 58 Teacher-Forced Prefixes from Development Requests...");
        let requests = load_requests(&args.requests_path)?;
        let tokenizer_bytes = fs::read(&args.tokenizer_path)?;
        let tokenizer =
            ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes).ok_or_else(|| {
                uor_r4_training::TrainingError::Invalid("invalid tokenizer json".into())
            })?;
        let protocol = DialogueProtocol::literal_roles_v1(&tokenizer)?;
        let encoder = protocol.bind(&tokenizer)?;
        let eos = protocol.eos_id;

        // Build working reference model with folded weights and unit norms
        let ref_model = StackModel::new(float_model.config.clone(), &Device::Cpu)?;
        let d = float_model.config.width;
        set_model_var(&ref_model, "final_norm.weight", &vec![1.0f32; d])?;
        for (l, kind) in float_model.config.pattern.bytes().enumerate() {
            let t = |suffix: &str| format!("layers.{l:02}.{suffix}");
            if kind == b'r' {
                set_model_var(&ref_model, &t("rec_norm.weight"), &vec![1.0f32; d])?;
            } else {
                set_model_var(&ref_model, &t("read_norm.weight"), &vec![1.0f32; d])?;
            }
            set_model_var(&ref_model, &t("mlp_norm.weight"), &vec![1.0f32; d])?;
        }
        for (name, var) in float_model.variables() {
            if name.contains("bias")
                || name.contains("decay")
                || name.contains("conv")
                || name.contains("age")
                || name.contains("beta")
                || name.contains("offset")
            {
                ref_model.variables()[name].set(var.as_tensor())?;
            }
        }

        // Apply all float folded weights
        for spec in &map_specs {
            if let Some(var_name) = &spec.var_name {
                set_model_var(&ref_model, var_name, &float_folded[&spec.name])?;
            }
        }
        let float_head_tensor =
            Tensor::from_vec(float_folded["head"].clone(), (vocab_size, d), &Device::Cpu)?;

        // Build teacher turns
        let mut teacher_turns: Vec<TeacherTurn> = Vec::new();
        let mut total_turns = 0usize;

        for request in &requests {
            let mut history = vec![protocol.bos_id];
            for (turn, user) in request.user_turns.iter().enumerate() {
                let prefix = encoder.encode_user_prefix(user, turn != 0);
                history.extend(&prefix.tokens);
                let reply = greedy_reply_with_head(
                    &ref_model,
                    &float_head_tensor,
                    &history,
                    MAX_NEW_TOKENS,
                    eos,
                )?;
                teacher_turns.push(TeacherTurn {
                    turn_index: total_turns,
                    prefix_ids: history.clone(),
                    float_reply_ids: reply.ids.clone(),
                });
                total_turns += 1;
                history.extend(&reply.ids);
                let closed = !reply.eos && turn + 1 < request.user_turns.len();
                if closed {
                    history.push(eos);
                }
            }
        }
        println!("Constructed {total_turns} teacher-forced turns (expected 58).");

        // Helper: evaluate model with specified quantized subset
        let evaluate_quantized_subset_with_replies = |quantized_names: &[&str]| -> Result<
            (usize, usize, Vec<Vec<u32>>),
            Box<dyn std::error::Error>,
        > {
            // Set variables
            for spec in &map_specs {
                let use_quantized = quantized_names.contains(&spec.name.as_str());
                let source_vals = if use_quantized {
                    &dequantized[&spec.name]
                } else {
                    &float_folded[&spec.name]
                };
                if let Some(var_name) = &spec.var_name {
                    set_model_var(&ref_model, var_name, source_vals)?;
                }
            }

            let head_vals = if quantized_names.contains(&"head") {
                &dequantized["head"]
            } else {
                &float_folded["head"]
            };
            let head_tensor = Tensor::from_vec(head_vals.clone(), (vocab_size, d), &Device::Cpu)?;

            let mut matches = 0usize;
            let mut replies = Vec::with_capacity(teacher_turns.len());
            for turn in &teacher_turns {
                let reply = greedy_reply_with_head(
                    &ref_model,
                    &head_tensor,
                    &turn.prefix_ids,
                    MAX_NEW_TOKENS,
                    eos,
                )?;
                if reply.ids == turn.float_reply_ids {
                    matches += 1;
                }
                replies.push(reply.ids);
            }
            let flips = total_turns - matches;
            Ok((matches, flips, replies))
        };

        let evaluate_quantized_subset =
            |quantized_names: &[&str]| -> Result<(usize, usize), Box<dyn std::error::Error>> {
                let (m, f, _) = evaluate_quantized_subset_with_replies(quantized_names)?;
                Ok((m, f))
            };

        // 1. Verify Float baseline: must be 58/58
        let (float_matches, float_flips) = evaluate_quantized_subset(&[])?;
        println!("Float Baseline:     {float_matches}/{total_turns} matches ({float_flips} flips)");

        // 2. Verify All-Quantized baseline: must match baseline (~14/58)
        let mut intervention_replies: BTreeMap<String, Vec<Vec<u32>>> = BTreeMap::new();
        let all_names: Vec<&str> = map_specs.iter().map(|s| s.name.as_str()).collect();
        let (all_q_matches, all_q_flips, all_q_replies) =
            evaluate_quantized_subset_with_replies(&all_names)?;
        println!(
            "All-Quantized Baseline: {all_q_matches}/{total_turns} matches ({all_q_flips} flips)"
        );
        intervention_replies.insert("all_quantized".to_string(), all_q_replies);

        // 3. Component Groups Attribution (quantize ONE component group at a time)
        println!("\n[3/3] Evaluating Component-wise Greedy-Flip Attribution...");
        let mut components: BTreeMap<String, Vec<&str>> = BTreeMap::new();
        for spec in &map_specs {
            components
                .entry(spec.component.clone())
                .or_default()
                .push(spec.name.as_str());
        }

        let mut component_results: Vec<Value> = Vec::new();
        for (comp_name, names) in &components {
            let (m_only, f_only, r_only) = evaluate_quantized_subset_with_replies(names)?;
            intervention_replies.insert(format!("component_only_{comp_name}"), r_only);
            // Leave-one-out: all quantized EXCEPT this group
            let leave_out_names: Vec<&str> = all_names
                .iter()
                .copied()
                .filter(|n| !names.contains(n))
                .collect();
            let (m_loo, f_loo, r_loo) = evaluate_quantized_subset_with_replies(&leave_out_names)?;
            intervention_replies.insert(format!("component_leave_out_{comp_name}"), r_loo);

            println!(
                "  Component {:<12}: {:2} flips if only quantized ({:2}/58 matches) | {:2} matches if kept float ({:2} flips)",
                comp_name, f_only, m_only, m_loo, f_loo
            );
            component_results.push(json!({
                "component": comp_name,
                "maps": names,
                "flips_if_only_quantized": f_only,
                "matches_if_only_quantized": m_only,
                "flips_if_kept_float": f_loo,
                "matches_if_kept_float": m_loo,
            }));
        }

        // 4. Layer Groups Attribution (quantize ONE layer at a time)
        println!("\nEvaluating Layer-wise Greedy-Flip Attribution...");
        let mut layer_groups: BTreeMap<String, Vec<&str>> = BTreeMap::new();
        for spec in &map_specs {
            let layer_key = match spec.layer {
                Some(l) => format!("layer_{l}"),
                None => spec.name.clone(),
            };
            layer_groups
                .entry(layer_key)
                .or_default()
                .push(spec.name.as_str());
        }

        let mut layer_results: Vec<Value> = Vec::new();
        for (layer_name, names) in &layer_groups {
            let (m_only, f_only, r_only) = evaluate_quantized_subset_with_replies(names)?;
            intervention_replies.insert(format!("layer_only_{layer_name}"), r_only);
            let leave_out_names: Vec<&str> = all_names
                .iter()
                .copied()
                .filter(|n| !names.contains(n))
                .collect();
            let (m_loo, f_loo, r_loo) = evaluate_quantized_subset_with_replies(&leave_out_names)?;
            intervention_replies.insert(format!("layer_leave_out_{layer_name}"), r_loo);

            println!(
                "  Layer {:<12}: {:2} flips if only quantized ({:2}/58 matches) | {:2} matches if kept float ({:2} flips)",
                layer_name, f_only, m_only, m_loo, f_loo
            );
            layer_results.push(json!({
                "layer_group": layer_name,
                "maps": names,
                "flips_if_only_quantized": f_only,
                "matches_if_only_quantized": m_only,
                "flips_if_kept_float": f_loo,
                "matches_if_kept_float": m_loo,
            }));
        }

        // 5. Individual Maps Ranking (evaluate each of the 42 maps)
        println!("\nEvaluating Individual Map Greedy-Flip Attribution (42 maps)...");
        let mut map_rankings: Vec<Value> = Vec::new();
        for spec in &map_specs {
            let (m_only, f_only, r_only) = evaluate_quantized_subset_with_replies(&[&spec.name])?;
            intervention_replies.insert(format!("map_only_{}", spec.name), r_only);
            let leave_out_names: Vec<&str> = all_names
                .iter()
                .copied()
                .filter(|&n| n != spec.name.as_str())
                .collect();
            let (m_loo, f_loo, r_loo) = evaluate_quantized_subset_with_replies(&leave_out_names)?;
            intervention_replies.insert(format!("map_leave_out_{}", spec.name), r_loo);

            let (tr_err, rel_err, act_status) = match activation_errors.get(&spec.name) {
                Some(v) => (
                    v.get("tr_e_sigma_e_t").and_then(|x| x.as_f64()),
                    v.get("relative_error").and_then(|x| x.as_f64()),
                    "measured",
                ),
                None => (None, None, "UNAVAILABLE_NO_CAPTURE_SITE"),
            };

            map_rankings.push(json!({
                "map": spec.name,
                "component": spec.component,
                "layer": spec.layer,
                "flips_if_only_quantized": f_only,
                "matches_if_only_quantized": m_only,
                "flips_if_kept_float": f_loo,
                "matches_if_kept_float": m_loo,
                "tr_e_sigma_e_t": tr_err,
                "relative_activation_error": rel_err,
                "activation_status": act_status,
            }));
        }

        // Sort rankings by flips_if_only_quantized descending
        map_rankings.sort_by(|a, b| {
            let f_b = b["flips_if_only_quantized"].as_u64().unwrap_or(0);
            let f_a = a["flips_if_only_quantized"].as_u64().unwrap_or(0);
            f_b.cmp(&f_a)
        });

        component_results.sort_by(|a, b| {
            let f_b = b["flips_if_only_quantized"].as_u64().unwrap_or(0);
            let f_a = a["flips_if_only_quantized"].as_u64().unwrap_or(0);
            f_b.cmp(&f_a)
        });

        println!("\n=== TOP 10 SENSITIVE MAPS BY GREEDY FLIPS ===");
        for (i, row) in map_rankings.iter().take(10).enumerate() {
            let rel_err_display = match row["relative_activation_error"].as_f64() {
                Some(err) => format!("{err:.4}"),
                None => "UNAVAILABLE".to_string(),
            };
            println!(
                "  {:2}. {:<16} | flips: {:2} | leave-out matches: {:2}/58 | rel act err: {}",
                i + 1,
                row["map"].as_str().unwrap_or(""),
                row["flips_if_only_quantized"].as_u64().unwrap_or(0),
                row["matches_if_kept_float"].as_u64().unwrap_or(0),
                rel_err_display
            );
        }

        let elapsed = run_started.elapsed().as_secs_f64();
        println!("\nCompleted S2 behavior distortion study in {elapsed:.2}s.");

        let mut row_observations = Vec::with_capacity(teacher_turns.len());
        for (i, turn) in teacher_turns.iter().enumerate() {
            let mut turn_interventions = serde_json::Map::new();
            for (interv_name, reps) in &intervention_replies {
                let rep = &reps[i];
                turn_interventions.insert(
                    interv_name.clone(),
                    json!({
                        "reply_ids": rep,
                        "matches_float": turn.float_reply_ids == *rep,
                    }),
                );
            }
            row_observations.push(json!({
                "turn_index": turn.turn_index,
                "prefix_token_count": turn.prefix_ids.len(),
                "float_reply_ids": turn.float_reply_ids,
                "all_quantized_reply_ids": intervention_replies["all_quantized"][i],
                "matches_float": turn.float_reply_ids == intervention_replies["all_quantized"][i],
                "interventions": turn_interventions,
            }));
        }
        fs::write(
            args.out.join("row_observations.json"),
            serde_json::to_vec_pretty(&row_observations)?,
        )?;

        let report = json!({
            "schema": "uor-r4.s2-behavior-distortion/1",
            "runtime_context": {
                "git_commit_unverified": current_git_commit(),
                "build_source_status": "unverified-source",
                "note": "cwd git rev-parse HEAD is runtime execution context, not verified build-source provenance"
            },
            "binary_sha256": current_binary_sha256(),
            "model_dir": args.model_dir.display().to_string(),
            "model_sha256": model_sha,
            "model_config_sha256": model_config_sha,
            "artifact_path": args.artifact_path.display().to_string(),
            "artifact_sha256": artifact_sha,
            "requests_path": args.requests_path.display().to_string(),
            "requests_sha256": requests_sha,
            "tokenizer_path": args.tokenizer_path.display().to_string(),
            "tokenizer_sha256": tokenizer_sha,
            "heldout_path": args.heldout_path.display().to_string(),
            "heldout_sha256": heldout_sha,
            "window_selection": {
                "windows": args.windows,
                "window_time": WINDOW_TIME,
                "selection_rule": "uniform_stride_span_over_tokens"
            },
            "total_turns": total_turns,
            "baselines": {
                "float_matches": float_matches,
                "float_flips": float_flips,
                "all_quantized_matches": all_q_matches,
                "all_quantized_flips": all_q_flips,
            },
            "activation_errors": activation_errors,
            "component_attribution": component_results,
            "layer_attribution": layer_results,
            "map_rankings": map_rankings,
            "row_observations_file": "row_observations.json",
            "elapsed_seconds": elapsed,
        });

        fs::write(
            args.out.join("report.json"),
            serde_json::to_vec_pretty(&report)?,
        )?;

        Ok(())
    })();

    if let Err(e) = &execution_result {
        eprintln!("Execution error: {e}");
    }

    report_output::seal(&args.out)?;
    report_output::verify(&args.out)?;
    execution_result
}
