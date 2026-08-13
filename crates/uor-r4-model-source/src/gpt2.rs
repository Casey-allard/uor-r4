//! GPT-2 (`openai-community/gpt2`) source executor for #607 — the first
//! genuinely non-Llama adapter behind the architecture-neutral compiler
//! boundary.
//!
//! Every GPT-2-specific fact lives in this module: the `config.json` key
//! schema (`n_embd`/`n_head`/`n_layer`/`n_positions`), the bare
//! `wte`/`wpe`/`h.<l>.*`/`ln_f` tensor names, the fused `c_attn` QKV split,
//! the Conv1D weight orientation (`[in, out]`, applied as `x @ W` with no
//! transpose), learned absolute positions, LayerNorm-with-bias, the
//! `gelu_new` activation, and tied embeddings. Downstream crates see only
//! the [`crate::TeacherOracle`] two-surface trait, never any of these names.
//!
//! The executor is a faithful f32 reference: it reproduces an independent
//! numpy GPT-2 forward within the #599 fixture tolerances (see the
//! generated-fixture test), so "parity" is measured against a second
//! implementation, not against itself.
#![cfg(not(target_arch = "wasm32"))]

use crate::{SafetensorsSnapshot, SourceIngestKind, SourceUnavailable, TensorRequirement};

/// Parsed, validated GPT-2 geometry from `config.json`. The GPT-2 family
/// spells these fields differently from Llama; the #599 conformance gate
/// (`AdapterFeatures::huggingface_gpt2`) has already validated the raw
/// configuration against the adapter's declaration before this parse.
#[derive(Debug, Clone)]
pub struct Gpt2Config {
    /// Residual / hidden width (`n_embd`).
    pub n_embd: usize,
    /// Attention heads (`n_head`); GPT-2 is plain multi-head (kv == heads).
    pub n_head: usize,
    /// Transformer blocks (`n_layer`).
    pub n_layer: usize,
    /// Learned position-table rows (`n_positions`): the maximum context.
    pub n_positions: usize,
    /// MLP inner width. GPT-2 uses `n_inner` when present, else `4 * n_embd`.
    pub n_inner: usize,
    /// Vocabulary size (`vocab_size`).
    pub vocab: usize,
    /// LayerNorm epsilon (`layer_norm_epsilon`).
    pub layer_norm_eps: f32,
    /// Working sequence length (bounded teacher context; <= n_positions).
    pub seq_len: usize,
    /// `bos_token_id`.
    pub bos: usize,
    /// `eos_token_id`.
    pub eos: usize,
}

impl Gpt2Config {
    /// Head width (`n_embd / n_head`).
    pub fn head_size(&self) -> usize {
        self.n_embd / self.n_head
    }

    /// Parse and range-check the GPT-2 fields. Presence/geometry
    /// relationships the #599 gate already checked are re-asserted here as
    /// typed-parse errors so a direct executor construction is also safe.
    pub fn from_json(
        config: &serde_json::Value,
        sequence_length: Option<usize>,
    ) -> Result<Self, SourceUnavailable> {
        let uint = |field: &str| -> Result<usize, SourceUnavailable> {
            config
                .get(field)
                .and_then(serde_json::Value::as_u64)
                .map(|value| value as usize)
                .ok_or_else(|| {
                    SourceUnavailable::from(SourceIngestKind::Unreadable {
                        path: "config.json".to_owned(),
                        reason: format!("GPT-2 config is missing an integer `{field}`"),
                    })
                })
        };
        let n_embd = uint("n_embd")?;
        let n_head = uint("n_head")?;
        let n_layer = uint("n_layer")?;
        let n_positions = uint("n_positions")?;
        let vocab = uint("vocab_size")?;
        let n_inner = config
            .get("n_inner")
            .and_then(serde_json::Value::as_u64)
            .map(|value| value as usize)
            .filter(|inner| *inner > 0)
            .unwrap_or(4 * n_embd);
        let layer_norm_eps = config
            .get("layer_norm_epsilon")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(1e-5) as f32;
        let bos = uint("bos_token_id").unwrap_or(
            config
                .get("bos_token_id")
                .and_then(serde_json::Value::as_u64)
                .map(|value| value as usize)
                .unwrap_or(0),
        );
        let eos = config
            .get("eos_token_id")
            .and_then(serde_json::Value::as_u64)
            .map(|value| value as usize)
            .unwrap_or(bos);
        if n_head == 0 || n_embd % n_head != 0 {
            return Err(SourceIngestKind::Unreadable {
                path: "config.json".to_owned(),
                reason: format!("GPT-2 n_embd {n_embd} is not divisible by n_head {n_head}"),
            }
            .into());
        }
        let seq_len = sequence_length
            .unwrap_or(n_positions)
            .min(n_positions)
            .max(1);
        Ok(Self {
            n_embd,
            n_head,
            n_layer,
            n_positions,
            n_inner,
            vocab,
            layer_norm_eps,
            seq_len,
            bos,
            eos,
        })
    }
}

/// One transformer block's weights, in the bare GPT-2 tensor layout. Conv1D
/// weights are stored `[in, out]` exactly as the checkpoint carries them,
/// applied as `y[o] = b[o] + sum_i x[i] * w[i * out + o]` with no transpose.
pub(crate) struct Gpt2Layer {
    pub(crate) ln1_w: Vec<f32>,
    pub(crate) ln1_b: Vec<f32>,
    /// `c_attn` fused QKV projection: `[n_embd, 3 * n_embd]`.
    pub(crate) c_attn_w: Vec<f32>,
    pub(crate) c_attn_b: Vec<f32>,
    /// attention output projection `c_proj`: `[n_embd, n_embd]`.
    pub(crate) c_proj_w: Vec<f32>,
    pub(crate) c_proj_b: Vec<f32>,
    pub(crate) ln2_w: Vec<f32>,
    pub(crate) ln2_b: Vec<f32>,
    /// MLP `c_fc`: `[n_embd, n_inner]`.
    pub(crate) fc_w: Vec<f32>,
    pub(crate) fc_b: Vec<f32>,
    /// MLP `c_proj`: `[n_inner, n_embd]`.
    pub(crate) mlp_w: Vec<f32>,
    pub(crate) mlp_b: Vec<f32>,
}

/// The GPT-2 executor: token + learned-position embeddings, a stack of
/// LayerNorm/attention/MLP blocks, a final LayerNorm, and a tied lm-head.
pub struct Gpt2 {
    pub cfg: Gpt2Config,
    /// Token embeddings `wte`: `[vocab, n_embd]` (also the tied lm-head).
    pub(crate) wte: Vec<f32>,
    /// Learned position embeddings `wpe`: `[n_positions, n_embd]`.
    pub(crate) wpe: Vec<f32>,
    pub(crate) layers: Vec<Gpt2Layer>,
    pub(crate) ln_f_w: Vec<f32>,
    pub(crate) ln_f_b: Vec<f32>,
    pub(crate) kappa: String,
    pub(crate) source_bytes: usize,
}

/// The tensors the GPT-2 geometry requires, with the shapes `config.json`
/// implies. Bare names (no `transformer.` prefix), matching the pinned
/// `openai-community/gpt2` checkpoint. The causal-mask buffer
/// `h.<l>.attn.bias` is deliberately not required: masking is applied
/// directly, and a single-file snapshot ignores unrequired tensors.
pub(crate) fn required_gpt2_tensors(cfg: &Gpt2Config) -> Vec<TensorRequirement> {
    let (d, inner, three) = (cfg.n_embd, cfg.n_inner, 3 * cfg.n_embd);
    let mut req = vec![
        TensorRequirement {
            name: "wte.weight".to_owned(),
            shape: vec![cfg.vocab, d],
        },
        TensorRequirement {
            name: "wpe.weight".to_owned(),
            shape: vec![cfg.n_positions, d],
        },
    ];
    for l in 0..cfg.n_layer {
        for (suffix, shape) in [
            ("ln_1.weight", vec![d]),
            ("ln_1.bias", vec![d]),
            ("attn.c_attn.weight", vec![d, three]),
            ("attn.c_attn.bias", vec![three]),
            ("attn.c_proj.weight", vec![d, d]),
            ("attn.c_proj.bias", vec![d]),
            ("ln_2.weight", vec![d]),
            ("ln_2.bias", vec![d]),
            ("mlp.c_fc.weight", vec![d, inner]),
            ("mlp.c_fc.bias", vec![inner]),
            ("mlp.c_proj.weight", vec![inner, d]),
            ("mlp.c_proj.bias", vec![d]),
        ] {
            req.push(TensorRequirement {
                name: format!("h.{l}.{suffix}"),
                shape,
            });
        }
    }
    req.push(TensorRequirement {
        name: "ln_f.weight".to_owned(),
        shape: vec![d],
    });
    req.push(TensorRequirement {
        name: "ln_f.bias".to_owned(),
        shape: vec![d],
    });
    req
}

impl Gpt2 {
    /// Load the executor from a snapshot directory: `config.json` plus a
    /// single `model.safetensors` (or #598 indexed shards). The raw config
    /// passes the #599 GPT-2 conformance gate before the typed parse, and
    /// all shape/dtype/byte checks happen at the one
    /// [`SafetensorsSnapshot::open`] boundary before any weight is read.
    pub fn load(
        source: impl AsRef<std::path::Path>,
        sequence_length: Option<usize>,
    ) -> Result<Self, SourceUnavailable> {
        let source = source.as_ref();
        let config_bytes = std::fs::read(source.join("config.json"))?;
        let raw: serde_json::Value = serde_json::from_slice(&config_bytes)?;
        crate::conformance::AdapterFeatures::huggingface_gpt2().validate_config(&raw)?;
        let cfg = Gpt2Config::from_json(&raw, sequence_length)?;
        let required = required_gpt2_tensors(&cfg);
        let snapshot = SafetensorsSnapshot::open(source, &required)?;

        let load = |name: &str| -> Result<Vec<f32>, SourceUnavailable> {
            let mut out = Vec::new();
            snapshot.tensor_f32_into(name, &mut out)?;
            Ok(out)
        };
        let wte = load("wte.weight")?;
        let wpe = load("wpe.weight")?;
        let mut layers = Vec::with_capacity(cfg.n_layer);
        for l in 0..cfg.n_layer {
            layers.push(Gpt2Layer {
                ln1_w: load(&format!("h.{l}.ln_1.weight"))?,
                ln1_b: load(&format!("h.{l}.ln_1.bias"))?,
                c_attn_w: load(&format!("h.{l}.attn.c_attn.weight"))?,
                c_attn_b: load(&format!("h.{l}.attn.c_attn.bias"))?,
                c_proj_w: load(&format!("h.{l}.attn.c_proj.weight"))?,
                c_proj_b: load(&format!("h.{l}.attn.c_proj.bias"))?,
                ln2_w: load(&format!("h.{l}.ln_2.weight"))?,
                ln2_b: load(&format!("h.{l}.ln_2.bias"))?,
                fc_w: load(&format!("h.{l}.mlp.c_fc.weight"))?,
                fc_b: load(&format!("h.{l}.mlp.c_fc.bias"))?,
                mlp_w: load(&format!("h.{l}.mlp.c_proj.weight"))?,
                mlp_b: load(&format!("h.{l}.mlp.c_proj.bias"))?,
            });
        }
        let ln_f_w = load("ln_f.weight")?;
        let ln_f_b = load("ln_f.bias")?;
        Ok(Self {
            cfg,
            wte,
            wpe,
            layers,
            ln_f_w,
            ln_f_b,
            kappa: snapshot.kappa().to_owned(),
            source_bytes: snapshot.source_bytes(),
        })
    }
}

/// GPT-2's `gelu_new` (the tanh approximation the checkpoint was trained
/// with): `0.5 x (1 + tanh(sqrt(2/pi) (x + 0.044715 x^3)))`.
#[inline]
fn gelu_new(x: f32) -> f32 {
    const C: f32 = 0.797_884_6; // sqrt(2/pi)
    0.5 * x * (1.0 + (C * (x + 0.044_715 * x * x * x)).tanh())
}

/// LayerNorm with a learned scale and bias, into `out`:
/// `out = (x - mean) / sqrt(var + eps) * w + b`, population variance.
fn layer_norm(x: &[f32], w: &[f32], b: &[f32], eps: f32, out: &mut [f32]) {
    let n = x.len() as f32;
    let mean = x.iter().sum::<f32>() / n;
    let var = x.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / n;
    let inv = 1.0 / (var + eps).sqrt();
    for i in 0..x.len() {
        out[i] = (x[i] - mean) * inv * w[i] + b[i];
    }
}

/// Conv1D as GPT-2 stores it: weight `[in, out]` (row-major), applied
/// `out[o] = bias[o] + sum_i x[i] * w[i * out_dim + o]` — no transpose.
fn conv1d(x: &[f32], w: &[f32], bias: &[f32], out_dim: usize, out: &mut [f32]) {
    out[..out_dim].copy_from_slice(&bias[..out_dim]);
    for (i, &xi) in x.iter().enumerate() {
        if xi == 0.0 {
            continue;
        }
        let row = &w[i * out_dim..(i + 1) * out_dim];
        for o in 0..out_dim {
            out[o] += xi * row[o];
        }
    }
}

/// Recurrent decode state: per-layer key/value caches, the working
/// residual, the final hidden state, and the last step's logits.
pub struct Gpt2State {
    k_cache: Vec<f32>,
    v_cache: Vec<f32>,
    /// Logits of the last [`Gpt2::forward`] step (`vocab`).
    pub logits: Vec<f32>,
    /// Final hidden state (post `ln_f`) of the last step (`n_embd`).
    pub hidden: Vec<f32>,
    x: Vec<f32>,
}

impl Gpt2State {
    pub fn new(cfg: &Gpt2Config) -> Self {
        let cache = cfg.n_layer * cfg.seq_len * cfg.n_embd;
        Self {
            k_cache: vec![0.0; cache],
            v_cache: vec![0.0; cache],
            logits: vec![0.0; cfg.vocab],
            hidden: vec![0.0; cfg.n_embd],
            x: vec![0.0; cfg.n_embd],
        }
    }

    /// Begin a new sequence: zero the caches and working buffers.
    pub fn reset(&mut self) {
        self.k_cache.fill(0.0);
        self.v_cache.fill(0.0);
        self.logits.fill(0.0);
        self.hidden.fill(0.0);
        self.x.fill(0.0);
    }
}

impl Gpt2 {
    /// One teacher-forced forward step at `pos` (0-based), leaving logits
    /// and the final hidden state in `st`. `capture` receives the
    /// post-block residual stream for each declared layer index, in
    /// ascending order — the #599 conformance-trace tap (a no-op closure
    /// captures nothing).
    pub fn forward(
        &self,
        st: &mut Gpt2State,
        token: usize,
        pos: usize,
        capture: &[usize],
        sink: &mut dyn FnMut(usize, &[f32]),
    ) {
        let d = self.cfg.n_embd;
        // token + learned absolute position embedding.
        for i in 0..d {
            st.x[i] = self.wte[token * d + i] + self.wpe[pos * d + i];
        }
        // scratch buffers reused across layers.
        let mut normed = vec![0.0f32; d];
        let mut qkv = vec![0.0f32; 3 * d];
        let mut attn = vec![0.0f32; d];
        let mut proj = vec![0.0f32; d];
        let mut inner = vec![0.0f32; self.cfg.n_inner];
        let mut mlp_out = vec![0.0f32; d];
        for l in 0..self.cfg.n_layer {
            self.block_forward(
                st,
                l,
                pos,
                &mut normed,
                &mut qkv,
                &mut attn,
                &mut proj,
                &mut inner,
                &mut mlp_out,
                None,
                None,
            );
            if capture.contains(&l) {
                sink(l, &st.x);
            }
        }
        self.finish_forward(st);
    }

    /// Final `ln_f` LayerNorm into `st.hidden` and the tied lm-head into
    /// `st.logits`. Shared by [`Gpt2::forward`] and
    /// [`Gpt2::forward_capturing_trace`] so a traced step finishes through
    /// the exact same arithmetic.
    fn finish_forward(&self, st: &mut Gpt2State) {
        let d = self.cfg.n_embd;
        layer_norm(
            &st.x,
            &self.ln_f_w,
            &self.ln_f_b,
            self.cfg.layer_norm_eps,
            &mut st.hidden,
        );
        let hidden = st.hidden.clone();
        for v in 0..self.cfg.vocab {
            let row = &self.wte[v * d..(v + 1) * d];
            let mut acc = 0.0f32;
            for i in 0..d {
                acc += hidden[i] * row[i];
            }
            st.logits[v] = acc;
        }
    }

    /// One teacher-forced forward step at `pos` with the #603 trace lanes
    /// captured through the exact executor path: the post-block residual
    /// stream, the current-position q/k/v, and the per-head softmax
    /// attention weights, each for the layer indices `request` declares.
    /// A traced step leaves the same logits, hidden state, and k/v caches
    /// as [`Gpt2::forward`] — the taps read the executor's own
    /// intermediates, they never recompute. Sinks fire in ascending layer
    /// order (heads ascending within a layer for the attention lane).
    pub fn forward_capturing_trace(
        &self,
        st: &mut Gpt2State,
        token: usize,
        pos: usize,
        request: &crate::TraceCaptureRequest<'_>,
        sinks: &mut crate::TraceCaptureSinks<'_, '_>,
    ) {
        let d = self.cfg.n_embd;
        // token + learned absolute position embedding.
        for i in 0..d {
            st.x[i] = self.wte[token * d + i] + self.wpe[pos * d + i];
        }
        let mut normed = vec![0.0f32; d];
        let mut qkv = vec![0.0f32; 3 * d];
        let mut attn = vec![0.0f32; d];
        let mut proj = vec![0.0f32; d];
        let mut inner = vec![0.0f32; self.cfg.n_inner];
        let mut mlp_out = vec![0.0f32; d];
        for l in 0..self.cfg.n_layer {
            self.block_forward(
                st,
                l,
                pos,
                &mut normed,
                &mut qkv,
                &mut attn,
                &mut proj,
                &mut inner,
                &mut mlp_out,
                Some(request),
                Some(&mut *sinks),
            );
            if request.residual_layers.contains(&l) {
                (sinks.residual)(l, &st.x);
            }
        }
        self.finish_forward(st);
    }

    #[allow(clippy::too_many_arguments)]
    fn block_forward(
        &self,
        st: &mut Gpt2State,
        l: usize,
        pos: usize,
        normed: &mut [f32],
        qkv: &mut [f32],
        attn: &mut [f32],
        proj: &mut [f32],
        inner: &mut [f32],
        mlp_out: &mut [f32],
        request: Option<&crate::TraceCaptureRequest<'_>>,
        mut sinks: Option<&mut crate::TraceCaptureSinks<'_, '_>>,
    ) {
        let d = self.cfg.n_embd;
        let hs = self.cfg.head_size();
        let scale = 1.0 / (hs as f32).sqrt();
        let layer = &self.layers[l];
        let seq = self.cfg.seq_len;

        // --- attention ---
        layer_norm(
            &st.x,
            &layer.ln1_w,
            &layer.ln1_b,
            self.cfg.layer_norm_eps,
            normed,
        );
        conv1d(normed, &layer.c_attn_w, &layer.c_attn_b, 3 * d, qkv);
        // store this position's k, v (thirds 1 and 2 of the fused qkv).
        let base = (l * seq + pos) * d;
        st.k_cache[base..base + d].copy_from_slice(&qkv[d..2 * d]);
        st.v_cache[base..base + d].copy_from_slice(&qkv[2 * d..3 * d]);
        // #603 q/k/v tap at this position (q = all heads, then the just-cached
        // k and v rows), emitted only for a requested layer. GPT-2 is plain
        // MHA (kv heads == query heads), so every row is `d` wide.
        if let Some(request) = request {
            if request.qkv_layers.contains(&l) {
                if let Some(sinks) = sinks.as_deref_mut() {
                    (sinks.qkv)(l, &qkv[..d], &qkv[d..2 * d], &qkv[2 * d..3 * d]);
                }
            }
        }
        let mut scores = vec![0.0f32; pos + 1];
        for h in 0..self.cfg.n_head {
            let qh = &qkv[h * hs..(h + 1) * hs];
            // scaled dot-product scores against every key up to pos.
            let mut max = f32::NEG_INFINITY;
            for (t, score) in scores.iter_mut().enumerate() {
                let koff = (l * seq + t) * d + h * hs;
                let kh = &st.k_cache[koff..koff + hs];
                let mut dot = 0.0f32;
                for i in 0..hs {
                    dot += qh[i] * kh[i];
                }
                *score = dot * scale;
                if *score > max {
                    max = *score;
                }
            }
            // softmax over positions 0..=pos.
            let mut sum = 0.0f32;
            for score in scores.iter_mut() {
                *score = (*score - max).exp();
                sum += *score;
            }
            let inv = 1.0 / sum;
            // Normalize the softmax weights in place. `score * inv` is the
            // exact per-position weight the value aggregation used before,
            // so the executor arithmetic stays bit-identical — and `scores`
            // now holds the per-head weights the #603 attention tap emits.
            for score in scores.iter_mut() {
                *score *= inv;
            }
            // #603 per-head attention-weight tap over positions 0..=pos.
            if let Some(request) = request {
                if request.attention_layers.contains(&l) {
                    if let Some(sinks) = sinks.as_deref_mut() {
                        (sinks.attention)(l, h, &scores);
                    }
                }
            }
            // weighted sum of values into this head's attention output.
            let ao = &mut attn[h * hs..(h + 1) * hs];
            ao.fill(0.0);
            for (t, &weight) in scores.iter().enumerate() {
                let voff = (l * seq + t) * d + h * hs;
                let vh = &st.v_cache[voff..voff + hs];
                for i in 0..hs {
                    ao[i] += weight * vh[i];
                }
            }
        }
        conv1d(attn, &layer.c_proj_w, &layer.c_proj_b, d, proj);
        for (xi, &p) in st.x.iter_mut().zip(&proj[..d]) {
            *xi += p;
        }

        // --- MLP ---
        layer_norm(
            &st.x,
            &layer.ln2_w,
            &layer.ln2_b,
            self.cfg.layer_norm_eps,
            normed,
        );
        conv1d(normed, &layer.fc_w, &layer.fc_b, self.cfg.n_inner, inner);
        for v in inner.iter_mut() {
            *v = gelu_new(*v);
        }
        conv1d(inner, &layer.mlp_w, &layer.mlp_b, d, mlp_out);
        for (xi, &m) in st.x.iter_mut().zip(&mlp_out[..d]) {
            *xi += m;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/gpt2-tiny")
    }

    fn f32_vec(value: &serde_json::Value) -> Vec<f32> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect()
    }

    fn assert_close(got: &[f32], want: &[f32], tol: f32, ctx: &str) {
        assert_eq!(got.len(), want.len(), "{ctx}: length");
        let mut worst = 0.0f32;
        for (i, (&g, &w)) in got.iter().zip(want).enumerate() {
            let d = (g - w).abs();
            if d > worst {
                worst = d;
            }
            assert!(d <= tol, "{ctx}[{i}]: got {g}, want {w}, |Δ|={d} > {tol}");
        }
        eprintln!("{ctx}: worst |Δ| = {worst:e}");
    }

    fn argmax(xs: &[f32]) -> usize {
        let mut best = 0;
        for i in 1..xs.len() {
            if xs[i] > xs[best] {
                best = i;
            }
        }
        best
    }

    /// The Rust GPT-2 executor reproduces the INDEPENDENT numpy reference
    /// (scripts/gen_gpt2_tiny_fixture.py) within tolerance across every
    /// per-layer residual, the final hidden state, the full logit vector,
    /// and the argmax token — for multiple prompt lengths. This is the
    /// parity that makes "faithful executor" a measured claim.
    #[test]
    fn tiny_gpt2_matches_numpy_reference() {
        let dir = fixture_dir();
        let golden: serde_json::Value =
            serde_json::from_slice(&std::fs::read(dir.join("golden.json")).unwrap()).unwrap();
        let model = Gpt2::load(&dir, None).expect("load tiny gpt2 snapshot");
        let n_layer = model.cfg.n_layer;
        let mut st = Gpt2State::new(&model.cfg);
        let capture: Vec<usize> = (0..n_layer).collect();

        for case in golden["cases"].as_array().unwrap() {
            st.reset();
            let tokens: Vec<usize> = case["tokens"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap() as usize)
                .collect();
            let mut captured: Vec<Vec<f32>> = vec![Vec::new(); n_layer];
            for (pos, &tok) in tokens.iter().enumerate() {
                if pos + 1 == tokens.len() {
                    model.forward(&mut st, tok, pos, &capture, &mut |l, x| {
                        captured[l] = x.to_vec();
                    });
                } else {
                    model.forward(&mut st, tok, pos, &[], &mut |_, _| {});
                }
            }

            let per_layer = case["per_layer"].as_array().unwrap();
            for (l, layer_golden) in per_layer.iter().enumerate() {
                assert_close(
                    &captured[l],
                    &f32_vec(layer_golden),
                    2e-3,
                    &format!("tokens {tokens:?} per_layer[{l}]"),
                );
            }
            assert_close(&st.hidden, &f32_vec(&case["hidden"]), 2e-3, "hidden");
            assert_close(&st.logits, &f32_vec(&case["logits"]), 3e-3, "logits");

            let top_token = case["top_k"].as_array().unwrap()[0].as_array().unwrap()[0]
                .as_u64()
                .unwrap() as usize;
            assert_eq!(argmax(&st.logits), top_token, "top-1 token for {tokens:?}");
        }
    }

    /// #668: the GPT-2 oracle reports the truthful learned-absolute
    /// operator record, and it resolves through the versioned registry.
    /// Its positional action is NOT RoPE, so it is a distinct identity
    /// from `standard-source-attention/1` — reusing that record would be
    /// a false operator identity.
    #[test]
    fn gpt2_oracle_reports_learned_absolute_operator() {
        use crate::attention::{operator_spec, AttentionOperatorSpec};
        use crate::TeacherOracle;

        let oracle = HuggingFaceGpt2Oracle::load(fixture_dir()).expect("load tiny gpt2 oracle");
        let spec = oracle
            .attention_operator_spec()
            .expect("gpt2 oracle declares an attention operator");

        assert_eq!(
            spec,
            AttentionOperatorSpec::learned_absolute_source_attention()
        );
        assert_eq!(spec.id, AttentionOperatorSpec::LEARNED_ABSOLUTE_ID);
        assert_eq!(
            spec.version,
            AttentionOperatorSpec::LEARNED_ABSOLUTE_VERSION
        );

        // Resolvable through the registry it is registered in.
        let resolved = operator_spec(&spec.id, spec.version).expect("registered operator");
        assert_eq!(resolved, spec);

        // A distinct identity from the RoPE standard operator: the
        // positional action is the discriminating field, and the digest
        // over the declared identity differs accordingly.
        let standard = AttentionOperatorSpec::standard();
        assert_ne!(spec.positional_action, standard.positional_action);
        assert_ne!(spec.implementation_digest, standard.implementation_digest);
    }

    /// #667: a #603-traced GPT-2 step produces logits bit-identical to the
    /// plain forward, and the residual/qkv/attention taps fire only for the
    /// requested layers with the declared shapes (residual `n_embd`; q/k/v
    /// each `n_embd` wide — plain MHA; per-head attention weights over
    /// `0..=pos`, summing to 1).
    #[test]
    fn forward_capturing_trace_matches_plain_forward_and_bounds_taps() {
        let dir = fixture_dir();
        let model = Gpt2::load(&dir, None).expect("load tiny gpt2 snapshot");
        let cfg_layers = model.cfg.n_layer;
        let heads = model.cfg.n_head;
        let d = model.cfg.n_embd;
        // Safe token ids and positions for the tiny fixture.
        let steps = 5.min(model.cfg.seq_len);
        let tokens: Vec<usize> = (0..steps).map(|i| i % model.cfg.vocab).collect();

        // Reference: plain forward logits at each position.
        let mut plain = Gpt2State::new(&model.cfg);
        plain.reset();
        let mut plain_logits: Vec<Vec<f32>> = Vec::new();
        for (pos, &tok) in tokens.iter().enumerate() {
            model.forward(&mut plain, tok, pos, &[], &mut |_, _| {});
            plain_logits.push(plain.logits.clone());
        }

        // Bounded capture request.
        let residual_layers: Vec<usize> = vec![0, cfg_layers - 1];
        let qkv_layers: Vec<usize> = vec![0];
        let attn_layer = if cfg_layers > 1 { 1 } else { 0 };
        let attention_layers: Vec<usize> = vec![attn_layer];
        let request = crate::TraceCaptureRequest {
            residual_layers: &residual_layers,
            qkv_layers: &qkv_layers,
            attention_layers: &attention_layers,
        };

        let mut traced = Gpt2State::new(&model.cfg);
        traced.reset();
        for (pos, &tok) in tokens.iter().enumerate() {
            let mut residual_hits: Vec<usize> = Vec::new();
            let mut qkv_hits: Vec<(usize, usize, usize, usize)> = Vec::new();
            let mut attention_hits: Vec<(usize, usize, usize, f32)> = Vec::new();
            {
                let mut residual = |l: usize, x: &[f32]| {
                    assert_eq!(x.len(), d, "residual width is n_embd");
                    residual_hits.push(l);
                };
                let mut qkv = |l: usize, q: &[f32], k: &[f32], v: &[f32]| {
                    qkv_hits.push((l, q.len(), k.len(), v.len()));
                };
                let mut attention = |l: usize, h: usize, w: &[f32]| {
                    let sum: f32 = w.iter().sum();
                    attention_hits.push((l, h, w.len(), sum));
                };
                let mut sinks = crate::TraceCaptureSinks {
                    residual: &mut residual,
                    qkv: &mut qkv,
                    attention: &mut attention,
                };
                model.forward_capturing_trace(&mut traced, tok, pos, &request, &mut sinks);
            }

            // Logits are bit-identical to the plain forward.
            let want: Vec<u32> = plain_logits[pos].iter().map(|v| v.to_bits()).collect();
            let got: Vec<u32> = traced.logits.iter().map(|v| v.to_bits()).collect();
            assert_eq!(
                got, want,
                "traced logits at pos {pos} equal the plain forward"
            );

            // Residual tap fires exactly for the requested layers, in order.
            assert_eq!(
                residual_hits, residual_layers,
                "residual layers at pos {pos}"
            );

            // q/k/v tap: once for the one requested layer, each row n_embd wide.
            assert_eq!(qkv_hits.len(), 1, "one qkv hit at pos {pos}");
            assert_eq!(qkv_hits[0], (0, d, d, d), "qkv layer + widths at pos {pos}");

            // Attention tap: one hit per head for the requested layer, weights
            // over 0..=pos and summing to 1.
            assert_eq!(attention_hits.len(), heads, "one attention hit per head");
            for (idx, (l, h, wlen, sum)) in attention_hits.iter().enumerate() {
                assert_eq!(*l, attn_layer, "attention layer");
                assert_eq!(*h, idx, "heads ascending");
                assert_eq!(*wlen, pos + 1, "attention weights span 0..=pos");
                assert!((sum - 1.0).abs() < 1e-4, "softmax weights sum to 1: {sum}");
            }
        }
    }

    /// Loading a snapshot whose config declares a non-GPT-2 architecture is
    /// refused at the #599 gate before any weight is read.
    #[test]
    fn non_gpt2_config_is_refused() {
        let tmp = std::env::temp_dir().join(format!("uor-r4-gpt2-neg-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(
            tmp.join("config.json"),
            br#"{"model_type":"llama","n_embd":32,"n_head":4,"n_layer":2,"n_positions":16,"vocab_size":24}"#,
        )
        .unwrap();
        let err = Gpt2::load(&tmp, None)
            .err()
            .expect("a llama config must be refused by the GPT-2 adapter");
        assert!(
            matches!(err.kind, SourceIngestKind::UnsupportedConfigFeature { .. }),
            "expected UnsupportedConfigFeature, got {:?}",
            err.kind
        );
    }
}

/// The GPT-2 adapter: a [`Gpt2`] executor plus its recurrent state, exposed
/// through the architecture-neutral [`crate::TeacherOracle`] two-surface
/// trait. Nothing GPT-2-specific escapes this type; the compiler consumes
/// only the trait, so no downstream crate branches on `gpt2`, `c_attn`, or
/// `wte`. The compiled representation width it presents is
/// [`crate::geometry::COMPILED_WIDTH`] (288); `embedding` projects the
/// source-width rows down through the #600 `bucket-average/1` projection,
/// exactly as the Llama adapter does for its own source width.
///
/// Surfaces this adapter deliberately leaves absent (each scoped, not
/// silently dropped, in issue #657): the #602 attention-operator record
/// (see [`Self::attention_operator_spec`]), the #603 trace-capture surface
/// (`trace_capture_geometry`/`step_with_trace_capture` keep the trait
/// defaults, so richer trace profiles are refused rather than zero-filled),
/// a GPT-2-specific #601 tokenizer identity, and the graph-cli/compiler
/// dispatch that would let a GPT-2 source compile to an artifact.
pub struct HuggingFaceGpt2Oracle {
    model: Gpt2,
    state: Gpt2State,
    bos_token: usize,
    eos_token: usize,
}

impl HuggingFaceGpt2Oracle {
    /// Load the GPT-2 teacher from a snapshot directory.
    pub fn load(source: impl AsRef<std::path::Path>) -> Result<Self, SourceUnavailable> {
        Self::build(Gpt2::load(source, None)?)
    }

    /// Load with a bounded teacher context (short trajectories are all the
    /// compiler needs; the deployed runtime consumes an eight-token window).
    pub fn load_with_sequence_length(
        source: impl AsRef<std::path::Path>,
        sequence_length: usize,
    ) -> Result<Self, SourceUnavailable> {
        if sequence_length == 0 {
            return Err(SourceUnavailable::new(
                "teacher sequence length must be greater than zero",
            ));
        }
        Self::build(Gpt2::load(source, Some(sequence_length))?)
    }

    fn build(model: Gpt2) -> Result<Self, SourceUnavailable> {
        let state = Gpt2State::new(&model.cfg);
        let (bos_token, eos_token) = (model.cfg.bos, model.cfg.eos);
        Ok(Self {
            model,
            state,
            bos_token,
            eos_token,
        })
    }

    /// This teacher's GPT-2 configuration.
    pub fn cfg(&self) -> &Gpt2Config {
        &self.model.cfg
    }
}

impl crate::RepresentationSource for HuggingFaceGpt2Oracle {
    fn vocab_size(&self) -> usize {
        self.model.cfg.vocab
    }
    fn source_dimension(&self) -> usize {
        self.model.cfg.n_embd
    }
    fn tokenizer_address(&self) -> &str {
        "huggingface-tokenizer"
    }
    fn read_embedding_rows(&self, range: std::ops::Range<usize>, output: &mut [f32]) -> Option<()> {
        let d = self.model.cfg.n_embd;
        let count = range.end.checked_sub(range.start)?;
        if output.len() < count * d || range.end > self.model.cfg.vocab {
            return None;
        }
        output[..count * d].copy_from_slice(&self.model.wte[range.start * d..range.end * d]);
        Some(())
    }
}

impl crate::BehaviorSource for HuggingFaceGpt2Oracle {
    fn reset(&mut self) {
        self.state.reset();
    }
    fn step(&mut self, token: usize, pos: usize, logits: &mut [f32]) {
        self.model
            .forward(&mut self.state, token, pos, &[], &mut |_, _| {});
        logits.copy_from_slice(&self.state.logits);
    }
}

impl crate::TeacherOracle for HuggingFaceGpt2Oracle {
    fn vocab(&self) -> usize {
        self.model.cfg.vocab
    }
    fn dim(&self) -> usize {
        // The compiled geometry this adapter presents (D = 288), not the
        // GPT-2 source width; `embedding` projects rows down to it (#600).
        crate::geometry::COMPILED_WIDTH as usize
    }
    fn seq_len(&self) -> usize {
        self.model.cfg.seq_len
    }
    fn bos_token(&self) -> usize {
        self.bos_token
    }
    fn eos_token(&self) -> usize {
        self.eos_token
    }
    fn kappa(&self) -> String {
        self.model.kappa.clone()
    }
    fn source_bytes(&self) -> usize {
        self.model.source_bytes
    }
    fn embedding(&self, token: usize, out: &mut [f32]) {
        let d = self.model.cfg.n_embd;
        let row = &self.model.wte[token * d..(token + 1) * d];
        crate::geometry::bucket_average_project(row, out);
    }
    fn geometry_projection(&self) -> Option<crate::geometry::GeometryProjection> {
        u32::try_from(self.model.cfg.n_embd)
            .ok()
            .map(|source_width| {
                crate::geometry::GeometryProjection::bucket_average(
                    source_width,
                    crate::geometry::COMPILED_WIDTH,
                )
            })
    }
    fn attention_operator_spec(&self) -> Option<crate::attention::AttentionOperatorSpec> {
        // #668: the truthful GPT-2 operator record. `Gpt2Model::layer_forward`
        // (this module) is its implementation — a scaled dot product with the
        // standard max-subtracted softmax, but learned absolute positions (no
        // RoPE on q/k) and GPT-2's fused-`c_attn`/`c_proj` Conv1D projections.
        // Reusing `standard-source-attention/1` would misdeclare the positional
        // action, so this is its own registered `(id, version)`.
        Some(crate::attention::AttentionOperatorSpec::learned_absolute_source_attention())
    }
    fn hidden_state(&self) -> Option<&[f32]> {
        Some(&self.state.hidden)
    }
    fn top_k(&self, k: usize, out: &mut [(u32, f32)]) -> usize {
        crate::top_k_from_logits(&self.state.logits, k, out, false)
    }
    fn trace_capture_geometry(&self) -> Option<crate::TraceCaptureGeometry> {
        Some(crate::TraceCaptureGeometry {
            layers: self.model.cfg.n_layer,
            heads: self.model.cfg.n_head,
            // GPT-2 is plain multi-head: kv heads == query heads.
            kv_heads: self.model.cfg.n_head,
            // The SOURCE residual width the taps expose (n_embd), not the
            // compiled width `dim()` presents. K/v rows are the same width
            // (kv_heads == heads).
            residual_width: self.model.cfg.n_embd,
        })
    }
    fn step_with_trace_capture(
        &mut self,
        token: usize,
        pos: usize,
        logits: &mut [f32],
        request: &crate::TraceCaptureRequest<'_>,
        sinks: &mut crate::TraceCaptureSinks<'_, '_>,
    ) -> bool {
        // #603: capture through the exact executor path — a traced step
        // leaves the same logits as the plain `step`.
        self.model
            .forward_capturing_trace(&mut self.state, token, pos, request, sinks);
        logits.copy_from_slice(&self.state.logits);
        true
    }
}

#[cfg(test)]
mod real_tests {
    use super::*;
    use crate::{BehaviorSource, TeacherOracle};
    use std::path::PathBuf;

    /// Presence-gated (#599 three-state) canary over the REAL pinned
    /// openai-community/gpt2 124M snapshot: the 548 MB source is a
    /// dev/local compiler input, never a CI download, so when it is absent
    /// this reports UNAVAILABLE and is a no-op success — never a silent
    /// skip of a real failure. When present, the oracle must reproduce the
    /// independent numpy reference (scripts/gen_gpt2_real_golden.py):
    /// identical argmax and top-5 token set on every prompt, and a final
    /// hidden state within tolerance.
    #[test]
    fn real_gpt2_matches_numpy_reference() {
        let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let src = crate_dir.join("../../.uor-models/sources/gpt2-124m");
        let golden_path = crate_dir.join("tests/fixtures/gpt2-real/golden.json");
        if !src.join("model.safetensors").exists() || !golden_path.exists() {
            eprintln!(
                "UNAVAILABLE: real gpt2 snapshot absent at {} — presence-gated canary skipped",
                src.display()
            );
            return;
        }

        let golden: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&golden_path).unwrap()).unwrap();
        let mut oracle = HuggingFaceGpt2Oracle::load(&src).expect("load real gpt2 oracle");
        let vocab = oracle.vocab();
        let mut logits = vec![0.0f32; vocab];

        for case in golden["cases"].as_array().unwrap() {
            oracle.reset();
            let tokens: Vec<usize> = case["tokens"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap() as usize)
                .collect();
            for (pos, &tok) in tokens.iter().enumerate() {
                oracle.step(tok, pos, &mut logits);
            }

            // final hidden state within tolerance of the numpy reference.
            let hidden_golden: Vec<f32> = case["hidden"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap() as f32)
                .collect();
            let hidden = oracle.hidden_state().expect("hidden state");
            let mut worst = 0.0f32;
            for (&g, &w) in hidden.iter().zip(&hidden_golden) {
                worst = worst.max((g - w).abs());
            }
            eprintln!("tokens {tokens:?}: hidden worst |Δ| = {worst:e}");
            assert!(
                worst < 5e-2,
                "hidden worst |Δ| {worst} too large for {tokens:?}"
            );

            // argmax and top-5 token set must match exactly.
            let mut topk = [(0u32, 0.0f32); 10];
            let n = oracle.top_k(10, &mut topk);
            assert!(n >= 5);
            let argmax = case["argmax"].as_u64().unwrap() as u32;
            assert_eq!(topk[0].0, argmax, "argmax mismatch for {tokens:?}");

            let golden_top5: std::collections::BTreeSet<u32> = case["top_k"]
                .as_array()
                .unwrap()
                .iter()
                .take(5)
                .map(|e| e.as_array().unwrap()[0].as_u64().unwrap() as u32)
                .collect();
            let mine_top5: std::collections::BTreeSet<u32> =
                topk[..5].iter().map(|&(t, _)| t).collect();
            assert_eq!(
                mine_top5, golden_top5,
                "top-5 token set mismatch for {tokens:?}"
            );
        }
    }
}
