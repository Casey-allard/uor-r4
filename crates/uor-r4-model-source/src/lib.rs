//! The TEACHER: faithful Rust port of karpathy run.c forward pass (v0 checkpoint).
//! Arithmetic order mirrors the C exactly: sequential adds in matmul rows,
//! rmsnorm/softmax/RoPE/SwiGLU op-for-op, libm via glibc on gnu targets.
//! The Safetensors adapter also loads pinned Hugging Face SmolLM2 weights
//! into this same source-only teacher surface. The pinned legacy teacher keeps
//! the original reduction order; native Hugging Face compilation may use an
//! optimized CPU matrix-vector backend.

pub mod attention;
pub mod conformance;
pub mod geometry;
#[cfg(not(target_arch = "wasm32"))]
pub mod gpt2;
pub mod progress;
#[cfg(not(target_arch = "wasm32"))]
pub mod teacher;

#[cfg(not(target_arch = "wasm32"))]
pub use gpt2::HuggingFaceGpt2Oracle;
#[cfg(not(target_arch = "wasm32"))]
pub use teacher::Teacher;
pub struct Config {
    pub dim: usize,
    pub hidden: usize,
    pub n_layers: usize,
    pub n_heads: usize,
    pub n_kv_heads: usize,
    pub vocab: usize,
    pub seq_len: usize,
    pub rope_theta: f32,
    pub rms_norm_eps: f32,
    pub rope_interleaved: bool,
    pub r4_attention: bool,
}

pub struct Llama {
    pub cfg: Config,
    w: Vec<f32>,
    // RoPE angles depend only on the position and head dimension.  Keeping
    // them here avoids recomputing pow/cos/sin once per layer and token.
    rope_cos: Vec<f32>,
    rope_sin: Vec<f32>,
    // float offsets into w
    emb: usize,
    rms_att: usize,
    wq: usize,
    wk: usize,
    wv: usize,
    wo: usize,
    rms_ffn: usize,
    w1: usize,
    w2: usize,
    w3: usize,
    rms_final: usize,
    wcls: usize,
    /// Use the portable pure-Rust math path required by D2 canonical mode.
    canonical_math: bool,
}

pub struct State {
    pub x: Vec<f32>,
    xb: Vec<f32>,
    xb2: Vec<f32>,
    hb: Vec<f32>,
    hb2: Vec<f32>,
    q: Vec<f32>,
    att: Vec<f32>,
    key_cache: Vec<f32>,
    value_cache: Vec<f32>,
    pub logits: Vec<f32>,
}

impl State {
    pub fn new(c: &Config) -> Self {
        let kv_dim = c.dim * c.n_kv_heads / c.n_heads;
        State {
            x: vec![0.0; c.dim],
            xb: vec![0.0; c.dim],
            xb2: vec![0.0; c.dim],
            hb: vec![0.0; c.hidden],
            hb2: vec![0.0; c.hidden],
            q: vec![0.0; c.dim],
            att: vec![0.0; c.n_heads * c.seq_len],
            key_cache: vec![0.0; c.n_layers * c.seq_len * kv_dim],
            value_cache: vec![0.0; c.n_layers * c.seq_len * kv_dim],
            logits: vec![0.0; c.vocab],
        }
    }

    /// Begin a new sequence by zeroing state buffers and the KV cache.
    pub fn reset(&mut self) {
        self.x.fill(0.0);
        self.xb.fill(0.0);
        self.xb2.fill(0.0);
        self.hb.fill(0.0);
        self.hb2.fill(0.0);
        self.q.fill(0.0);
        self.att.fill(0.0);
        self.key_cache.fill(0.0);
        self.value_cache.fill(0.0);
        self.logits.fill(0.0);
    }
}

#[inline]
pub(crate) fn sqrtf(value: f32, canonical: bool) -> f32 {
    if canonical {
        libm::sqrtf(value)
    } else {
        value.sqrt()
    }
}

#[inline]
fn expf(value: f32, canonical: bool) -> f32 {
    if canonical {
        libm::expf(value)
    } else {
        value.exp()
    }
}

#[inline]
fn powf(base: f32, exponent: f32, canonical: bool) -> f32 {
    if canonical {
        libm::powf(base, exponent)
    } else {
        base.powf(exponent)
    }
}

#[inline]
fn sinf(value: f32, canonical: bool) -> f32 {
    if canonical {
        libm::sinf(value)
    } else {
        value.sin()
    }
}

#[inline]
fn cosf(value: f32, canonical: bool) -> f32 {
    if canonical {
        libm::cosf(value)
    } else {
        value.cos()
    }
}

fn rmsnorm_with_mode(o: &mut [f32], x: &[f32], weight: &[f32], canonical: bool) {
    let size = x.len();
    let mut ss = x.iter().map(|value| value * value).sum::<f32>();
    ss /= size as f32;
    ss += 1e-5f32;
    ss = 1.0f32 / sqrtf(ss, canonical);
    for ((output, value), weight) in o.iter_mut().zip(x).zip(weight) {
        *output = *weight * (ss * *value);
    }
}

/// In-place variant matching C's rmsnorm(x, x, w): C computes ss from x
/// first, then writes; identical here.
fn rmsnorm_inplace_with_mode(x: &mut [f32], weight: &[f32], canonical: bool) {
    let size = x.len();
    let mut ss = x.iter().map(|value| value * value).sum::<f32>();
    ss /= size as f32;
    ss += 1e-5f32;
    ss = 1.0f32 / sqrtf(ss, canonical);
    for (value, weight) in x.iter_mut().zip(weight) {
        *value = *weight * (ss * *value);
    }
}

pub(crate) fn softmax_with_mode(x: &mut [f32], canonical: bool) {
    let mut max_val = x[0];
    for &value in x.iter().skip(1) {
        if value > max_val {
            max_val = value;
        }
    }
    let mut sum = 0.0f32;
    for value in x.iter_mut() {
        *value = expf(*value - max_val, canonical);
        sum += *value;
    }
    for value in x.iter_mut() {
        *value /= sum;
    }
}

/// W (d,n) @ x (n,) -> xout (d,), computed by the pinned `uor-matmul` exact
/// GEMM (#655-B2). `gemm_float` accumulates every product into a complete
/// accumulator and rounds once, so the result is the correctly-rounded exact
/// dot product — byte-identical across targets (no per-machine Accelerate
/// variance), which is what teacher-side κ reproduction needs. The former
/// `fast` (Accelerate BLAS `sgemv`) / hand-rolled canonical paths are gone;
/// `_fast` is retained in the signature only so callers need not change.
fn matmul(xout: &mut [f32], x: &[f32], w: &[f32], n: usize, _fast: bool) {
    // xout[d] = W[d, n] · x[n]  ==>  C[d, 1] = A[d, n] · B[n, 1].
    let d = xout.len();
    let mut pa = vec![uor_matmul::PackedCode::default(); n];
    let mut pb = vec![uor_matmul::PackedCode::default(); n];
    uor_matmul::slice::gemm_float(d, n, 1, w, x, xout, &mut pa, &mut pb)
        .expect("teacher matrix-vector product is total over finite f32 operands");
}

/// Batched matmul: `batch` input vectors of length `n` through weight
/// `W[rows, n]` → `batch` output vectors of length `rows`, laid out
/// sequence-major (`x` is `batch * n`, `xout` is `batch * rows`). Computes
/// `C[batch, rows] = X[batch, n] · W[rows, n]ᵀ` with the pinned `uor-matmul`
/// exact GEMM (#655-B2), replacing the former Accelerate BLAS `sgemm` /
/// hand-rolled `dot_fast` reuse. Byte-identical across targets, so the batched
/// teacher path reproduces the serial [`matmul`] exactly on every machine.
fn matmul_batched(xout: &mut [f32], x: &[f32], w: &[f32], n: usize, batch: usize) {
    debug_assert!(batch > 0);
    debug_assert_eq!(xout.len() % batch, 0);
    let rows = xout.len() / batch;
    debug_assert!(w.len() >= rows * n);
    debug_assert_eq!(x.len(), batch * n);
    // gemm_float is C = A·B (no transpose), so transpose W[rows, n] → Wt[n, rows]
    // and compute X[batch, n] · Wt[n, rows] into the sequence-major `xout`.
    let mut wt = vec![0f32; n * rows];
    for r in 0..rows {
        for j in 0..n {
            wt[j * rows + r] = w[r * n + j];
        }
    }
    let mut pa = vec![uor_matmul::PackedCode::default(); n];
    let mut pb = vec![uor_matmul::PackedCode::default(); n * rows];
    uor_matmul::slice::gemm_float(batch, n, rows, x, &wt, xout, &mut pa, &mut pb)
        .expect("batched teacher product is total over finite f32 operands");
}

/// The teacher's matrix-operation backend, for the "teacher model ready"
/// diagnostic. Since #655-B2 every teacher weight matmul is the pinned,
/// portable `uor-matmul` exact GEMM — no per-machine SIMD/Accelerate path.
fn fast_matmul_backend() -> &'static str {
    "uor-matmul exact GEMM"
}

impl Llama {
    fn build_rope_cache(cfg: &Config, canonical_math: bool) -> (Vec<f32>, Vec<f32>) {
        let head_size = cfg.dim / cfg.n_heads;
        let half = head_size / 2;
        let mut cos = Vec::with_capacity(cfg.seq_len * half);
        let mut sin = Vec::with_capacity(cfg.seq_len * half);
        for pos in 0..cfg.seq_len {
            for i in 0..half {
                let freq = 1.0f32
                    / powf(
                        cfg.rope_theta,
                        (2 * i) as f32 / head_size as f32,
                        canonical_math,
                    );
                let angle = pos as f32 * freq;
                cos.push(cosf(angle, canonical_math));
                sin.push(sinf(angle, canonical_math));
            }
        }
        (cos, sin)
    }

    fn rebuild_rope_cache(&mut self) {
        (self.rope_cos, self.rope_sin) = Self::build_rope_cache(&self.cfg, self.canonical_math);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(path: &str) -> Llama {
        let raw = std::fs::read(path).expect("checkpoint");
        let i32at = |o: usize| i32::from_le_bytes(raw[o..o + 4].try_into().unwrap());
        let vocab_raw = i32at(20);
        let cfg = Config {
            dim: i32at(0) as usize,
            hidden: i32at(4) as usize,
            n_layers: i32at(8) as usize,
            n_heads: i32at(12) as usize,
            n_kv_heads: i32at(16) as usize,
            vocab: vocab_raw.unsigned_abs() as usize,
            seq_len: i32at(24) as usize,
            rope_theta: 10_000.0,
            rms_norm_eps: 1e-5,
            rope_interleaved: true,
            r4_attention: false,
        };
        let shared = vocab_raw > 0;
        let nf = (raw.len() - 28) / 4;
        let mut w = vec![0.0f32; nf];
        for (i, value) in w.iter_mut().enumerate() {
            let o = 28 + i * 4;
            *value = f32::from_le_bytes(raw[o..o + 4].try_into().unwrap());
        }
        let (dim, hid, nl, hs) = (cfg.dim, cfg.hidden, cfg.n_layers, cfg.dim / cfg.n_heads);
        let kv_dim = cfg.dim * cfg.n_kv_heads / cfg.n_heads;
        let mut p = 0usize;
        let emb = p;
        p += cfg.vocab * dim;
        let rms_att = p;
        p += nl * dim;
        let wq = p;
        p += nl * dim * dim;
        let wk = p;
        p += nl * dim * kv_dim;
        let wv = p;
        p += nl * dim * kv_dim;
        let wo = p;
        p += nl * dim * dim;
        let rms_ffn = p;
        p += nl * dim;
        let w1 = p;
        p += nl * dim * hid;
        let w2 = p;
        p += nl * hid * dim;
        let w3 = p;
        p += nl * dim * hid;
        let rms_final = p;
        p += dim;
        p += cfg.seq_len * hs / 2; // skip legacy freq_cis_real
        p += cfg.seq_len * hs / 2; // skip legacy freq_cis_imag
        let wcls = if shared { emb } else { p };
        let mut model = Llama {
            cfg,
            w,
            rope_cos: Vec::new(),
            rope_sin: Vec::new(),
            emb,
            rms_att,
            wq,
            wk,
            wv,
            wo,
            rms_ffn,
            w1,
            w2,
            w3,
            rms_final,
            wcls,
            canonical_math: false,
        };
        model.rebuild_rope_cache();
        model
    }

    fn from_flat(cfg: Config, w: Vec<f32>, shared: bool) -> Self {
        let (dim, hid, nl) = (cfg.dim, cfg.hidden, cfg.n_layers);
        let kv_dim = cfg.dim * cfg.n_kv_heads / cfg.n_heads;
        let mut p = 0usize;
        let emb = p;
        p += cfg.vocab * dim;
        let rms_att = p;
        p += nl * dim;
        let wq = p;
        p += nl * dim * dim;
        let wk = p;
        p += nl * dim * kv_dim;
        let wv = p;
        p += nl * dim * kv_dim;
        let wo = p;
        p += nl * dim * dim;
        let rms_ffn = p;
        p += nl * dim;
        let w1 = p;
        p += nl * dim * hid;
        let w2 = p;
        p += nl * hid * dim;
        let w3 = p;
        p += nl * dim * hid;
        let rms_final = p;
        p += dim;
        let wcls = if shared { emb } else { p };
        assert_eq!(w.len(), if shared { p } else { p + cfg.vocab * dim });
        let mut model = Self {
            cfg,
            w,
            rope_cos: Vec::new(),
            rope_sin: Vec::new(),
            emb,
            rms_att,
            wq,
            wk,
            wv,
            wo,
            rms_ffn,
            w1,
            w2,
            w3,
            rms_final,
            wcls,
            canonical_math: false,
        };
        model.rebuild_rope_cache();
        model
    }

    /// One forward step. After return, st.x holds the post-final-rmsnorm
    /// hidden state (the kNN-LM context vector) and st.logits the logits.
    pub fn forward(&self, st: &mut State, token: usize, pos: usize, fast_matmul: bool) {
        let dim = self.cfg.dim;
        st.x.copy_from_slice(&self.w[self.emb + token * dim..self.emb + (token + 1) * dim]);
        for l in 0..self.cfg.n_layers {
            self.layer_forward(st, l, pos, fast_matmul);
        }
        self.finish_forward(st, fast_matmul);
    }

    /// One forward step with the residual stream captured after each layer in
    /// `capture_layers` (#599 conformance trace). This IS the exact executor:
    /// embedding, per-layer body, and final norm/logits are the same
    /// [`Llama::layer_forward`]/[`Llama::finish_forward`] path `forward`
    /// takes, so a captured run and a plain run produce identical bits. The
    /// capture is bounded by construction: only the declared layer indices
    /// are copied out, once per step.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn forward_capturing(
        &self,
        st: &mut State,
        token: usize,
        pos: usize,
        fast_matmul: bool,
        capture_layers: &[usize],
        sink: &mut dyn FnMut(usize, &[f32]),
    ) {
        let dim = self.cfg.dim;
        st.x.copy_from_slice(&self.w[self.emb + token * dim..self.emb + (token + 1) * dim]);
        for l in 0..self.cfg.n_layers {
            self.layer_forward(st, l, pos, fast_matmul);
            if capture_layers.contains(&l) {
                sink(l, &st.x);
            }
        }
        self.finish_forward(st, fast_matmul);
    }

    /// One forward step with the #603 teacher-trace lanes captured at
    /// declared layer indices. This IS the exact executor — the same
    /// [`Llama::layer_forward`]/[`Llama::finish_forward`] path `forward`
    /// and `forward_capturing` take, so a traced step and a plain step
    /// produce identical bits; the taps only READ state the layer body
    /// already produced. Bounded by construction: each lane copies out
    /// only its declared layer indices, once per step.
    ///
    /// Taps, all read after `layer_forward(l)` returns:
    ///
    /// - residual: `st.x` (the post-layer residual stream, the same slice
    ///   `forward_capturing` sinks);
    /// - q/k/v: `st.q` (the current position's RoPE-rotated query) and the
    ///   layer's key/value cache rows at `pos` — the exact vectors the
    ///   #602 attention operators consumed;
    /// - attention support: per head `h`, the softmax-normalized weights
    ///   `st.att[h*seq_len .. h*seq_len+pos+1]` the #602 factored per-head
    ///   weight functions just produced for layer `l`. Read before the
    ///   next layer overwrites the shared buffer.
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn forward_capturing_trace(
        &self,
        st: &mut State,
        token: usize,
        pos: usize,
        fast_matmul: bool,
        request: &TraceCaptureRequest<'_>,
        sinks: &mut TraceCaptureSinks<'_, '_>,
    ) {
        let dim = self.cfg.dim;
        let kv_dim = self.cfg.dim * self.cfg.n_kv_heads / self.cfg.n_heads;
        st.x.copy_from_slice(&self.w[self.emb + token * dim..self.emb + (token + 1) * dim]);
        for l in 0..self.cfg.n_layers {
            self.layer_forward(st, l, pos, fast_matmul);
            if request.attention_layers.contains(&l) {
                for h in 0..self.cfg.n_heads {
                    (sinks.attention)(
                        l,
                        h,
                        &st.att[h * self.cfg.seq_len..h * self.cfg.seq_len + pos + 1],
                    );
                }
            }
            if request.qkv_layers.contains(&l) {
                let loff = l * self.cfg.seq_len * kv_dim;
                let k = &st.key_cache[loff + pos * kv_dim..loff + (pos + 1) * kv_dim];
                let v = &st.value_cache[loff + pos * kv_dim..loff + (pos + 1) * kv_dim];
                (sinks.qkv)(l, &st.q, k, v);
            }
            if request.residual_layers.contains(&l) {
                (sinks.residual)(l, &st.x);
            }
        }
        self.finish_forward(st, fast_matmul);
    }

    /// One transformer layer of the exact forward step, factored out of
    /// [`Llama::forward`] so the #599 conformance runner can observe the
    /// residual stream at declared layer indices through the very same
    /// executor path. Operation order and arithmetic are unchanged from the
    /// original in-line loop body.
    fn layer_forward(&self, st: &mut State, l: usize, pos: usize, fast_matmul: bool) {
        let c = &self.cfg;
        let (dim, hid) = (c.dim, c.hidden);
        let kv_dim = c.dim * c.n_kv_heads / c.n_heads;
        let kv_mul = c.n_heads / c.n_kv_heads;
        let head_size = dim / c.n_heads;
        let w = &self.w;
        {
            rmsnorm_with_mode(
                &mut st.xb,
                &st.x,
                &w[self.rms_att + l * dim..self.rms_att + (l + 1) * dim],
                self.canonical_math,
            );

            let loff = l * c.seq_len * kv_dim;
            matmul(
                &mut st.q,
                &st.xb,
                &w[self.wq + l * dim * dim..],
                dim,
                fast_matmul,
            );
            {
                let k = &mut st.key_cache[loff + pos * kv_dim..loff + (pos + 1) * kv_dim];
                matmul(
                    k,
                    &st.xb,
                    &w[self.wk + l * dim * kv_dim..],
                    dim,
                    fast_matmul,
                );
            }
            {
                let v = &mut st.value_cache[loff + pos * kv_dim..loff + (pos + 1) * kv_dim];
                matmul(
                    v,
                    &st.xb,
                    &w[self.wv + l * dim * kv_dim..],
                    dim,
                    fast_matmul,
                );
            }

            // RoPE: converted llama2.c checkpoints interleave pairs; native
            // Hugging Face Safetensors rotate the two head halves.
            if c.rope_interleaved {
                let k = &mut st.key_cache[loff + pos * kv_dim..loff + (pos + 1) * kv_dim];
                let rope_offset = pos * (head_size / 2);
                let mut i = 0usize;
                while i < dim {
                    let angle_index = rope_offset + (i % head_size) / 2;
                    let fcr = self.rope_cos[angle_index];
                    let fci = self.rope_sin[angle_index];
                    let rotn = if i < kv_dim { 2 } else { 1 };
                    for v in 0..rotn {
                        let vec: &mut [f32] = if v == 0 { &mut st.q } else { &mut *k };
                        let v0 = vec[i];
                        let v1 = vec[i + 1];
                        vec[i] = v0 * fcr - v1 * fci;
                        vec[i + 1] = v0 * fci + v1 * fcr;
                    }
                    i += 2;
                }
            } else {
                let k = &mut st.key_cache[loff + pos * kv_dim..loff + (pos + 1) * kv_dim];
                for vector in [&mut st.q[..], &mut k[..]] {
                    for head in vector.chunks_exact_mut(head_size) {
                        let half = head_size / 2;
                        for i in 0..half {
                            let angle_index = pos * half + i;
                            let cos = self.rope_cos[angle_index];
                            let sin = self.rope_sin[angle_index];
                            let first = head[i];
                            let second = head[i + half];
                            head[i] = first * cos - second * sin;
                            head[i + half] = second * cos + first * sin;
                        }
                    }
                }
            }

            // multihead attention (serial over heads; per-head work is
            // independent of order). The per-head weight computation and
            // value aggregation are the free #602 reference functions in
            // [`attention`], factored out of this loop unchanged; the
            // `r4_attention` switch selects between exactly the two
            // registered operators (`standard-source-attention/1` and
            // `experimental-r4-source-attention/1`, a chunked dot
            // product with the same softmax selector).
            for h in 0..c.n_heads {
                let q = &st.q[h * head_size..(h + 1) * head_size];
                let att = &mut st.att[h * c.seq_len..h * c.seq_len + pos + 1];
                let kv_head_offset = (h / kv_mul) * head_size;

                if c.r4_attention {
                    attention::experimental_r4_head_attention_weights(
                        att,
                        q,
                        &st.key_cache[loff..],
                        kv_head_offset,
                        kv_dim,
                        self.canonical_math,
                    );
                } else {
                    attention::standard_head_attention_weights(
                        att,
                        q,
                        &st.key_cache[loff..],
                        kv_head_offset,
                        kv_dim,
                        self.canonical_math,
                    );
                }

                let att = &st.att[h * c.seq_len..h * c.seq_len + pos + 1];
                let xb = &mut st.xb[h * head_size..(h + 1) * head_size];
                attention::head_attention_value_aggregate(
                    xb,
                    att,
                    &st.value_cache[loff..],
                    kv_head_offset,
                    kv_dim,
                );
            }

            matmul(
                &mut st.xb2,
                &st.xb,
                &w[self.wo + l * dim * dim..],
                dim,
                fast_matmul,
            );
            for i in 0..dim {
                st.x[i] += st.xb2[i];
            }

            rmsnorm_with_mode(
                &mut st.xb,
                &st.x,
                &w[self.rms_ffn + l * dim..self.rms_ffn + (l + 1) * dim],
                self.canonical_math,
            );
            matmul(
                &mut st.hb,
                &st.xb,
                &w[self.w1 + l * dim * hid..],
                dim,
                fast_matmul,
            );
            matmul(
                &mut st.hb2,
                &st.xb,
                &w[self.w3 + l * dim * hid..],
                dim,
                fast_matmul,
            );
            for i in 0..hid {
                let mut val = st.hb[i];
                val *= 1.0f32 / (1.0f32 + expf(-val, self.canonical_math));
                val *= st.hb2[i];
                st.hb[i] = val;
            }
            matmul(
                &mut st.xb,
                &st.hb,
                &w[self.w2 + l * hid * dim..],
                hid,
                fast_matmul,
            );
            for i in 0..dim {
                st.x[i] += st.xb[i];
            }
        }
    }

    /// The tail of the exact forward step (final in-place rmsnorm and the
    /// vocabulary matmul), factored out of [`Llama::forward`] unchanged.
    fn finish_forward(&self, st: &mut State, fast_matmul: bool) {
        let dim = self.cfg.dim;
        let w = &self.w;
        let rf = self.rms_final;
        // C: rmsnorm(x, x, w) — in-place with pre-read ss.
        {
            let (wslice, x) = (&w[rf..rf + dim], &mut st.x);
            rmsnorm_inplace_with_mode(x, wslice, self.canonical_math);
        }
        matmul(&mut st.logits, &st.x, &w[self.wcls..], dim, fast_matmul);
    }

    /// Batched forward: advance `states.len()` independent sequences by one
    /// position each — sequence `b` steps `tokens[b]` at `positions[b]` against
    /// its own KV cache in `states[b]`. The memory-bound weight matmuls (Q/K/V,
    /// output, MLP, vocab) run once over the whole batch via [`matmul_batched`]
    /// instead of once per sequence, so B sequences cost one weight sweep — the
    /// amortization that lifts the teacher off the per-token memory-bandwidth
    /// wall. Every per-sequence op (rmsnorm, RoPE, attention, SwiGLU, residual)
    /// mirrors [`Llama::forward`] exactly, and off macOS `matmul_batched` reuses
    /// the same `dot_fast`, so this is bit-identical to calling `forward` on
    /// each sequence with `fast_matmul = true`. `fast_matmul` is accepted for
    /// signature parity; the batched path always takes the amortized kernel.
    pub fn forward_batch(
        &self,
        states: &mut [State],
        tokens: &[usize],
        positions: &[usize],
        fast_matmul: bool,
    ) {
        let _ = fast_matmul;
        let c = &self.cfg;
        let (dim, hid) = (c.dim, c.hidden);
        let kv_dim = c.dim * c.n_kv_heads / c.n_heads;
        let kv_mul = c.n_heads / c.n_kv_heads;
        let head_size = dim / c.n_heads;
        let w = &self.w;
        let b = states.len();
        debug_assert_eq!(tokens.len(), b);
        debug_assert_eq!(positions.len(), b);

        // Sequence-major stacked scratch for the batched matmuls.
        let mut norm = vec![0f32; b * dim];
        let mut q = vec![0f32; b * dim];
        let mut ktmp = vec![0f32; b * kv_dim];
        let mut vtmp = vec![0f32; b * kv_dim];
        let mut attn = vec![0f32; b * dim];
        let mut o = vec![0f32; b * dim];
        let mut hb = vec![0f32; b * hid];
        let mut hb2 = vec![0f32; b * hid];
        let mut ffn = vec![0f32; b * dim];
        let mut xstack = vec![0f32; b * dim];

        for bi in 0..b {
            let token = tokens[bi];
            states[bi]
                .x
                .copy_from_slice(&w[self.emb + token * dim..self.emb + (token + 1) * dim]);
        }

        for l in 0..c.n_layers {
            let loff = l * c.seq_len * kv_dim;
            for bi in 0..b {
                rmsnorm_with_mode(
                    &mut norm[bi * dim..(bi + 1) * dim],
                    &states[bi].x,
                    &w[self.rms_att + l * dim..self.rms_att + (l + 1) * dim],
                    self.canonical_math,
                );
            }
            matmul_batched(&mut q, &norm, &w[self.wq + l * dim * dim..], dim, b);
            matmul_batched(&mut ktmp, &norm, &w[self.wk + l * dim * kv_dim..], dim, b);
            matmul_batched(&mut vtmp, &norm, &w[self.wv + l * dim * kv_dim..], dim, b);
            for bi in 0..b {
                let dst = loff + positions[bi] * kv_dim;
                states[bi].key_cache[dst..dst + kv_dim]
                    .copy_from_slice(&ktmp[bi * kv_dim..(bi + 1) * kv_dim]);
                states[bi].value_cache[dst..dst + kv_dim]
                    .copy_from_slice(&vtmp[bi * kv_dim..(bi + 1) * kv_dim]);
            }

            for bi in 0..b {
                let pos = positions[bi];
                let qb = &mut q[bi * dim..(bi + 1) * dim];
                let out = &mut attn[bi * dim..(bi + 1) * dim];
                let st = &mut states[bi];

                if c.rope_interleaved {
                    let k = &mut st.key_cache[loff + pos * kv_dim..loff + (pos + 1) * kv_dim];
                    let rope_offset = pos * (head_size / 2);
                    let mut i = 0usize;
                    while i < dim {
                        let angle_index = rope_offset + (i % head_size) / 2;
                        let fcr = self.rope_cos[angle_index];
                        let fci = self.rope_sin[angle_index];
                        let rotn = if i < kv_dim { 2 } else { 1 };
                        for v in 0..rotn {
                            let vec: &mut [f32] = if v == 0 { &mut *qb } else { &mut *k };
                            let v0 = vec[i];
                            let v1 = vec[i + 1];
                            vec[i] = v0 * fcr - v1 * fci;
                            vec[i + 1] = v0 * fci + v1 * fcr;
                        }
                        i += 2;
                    }
                } else {
                    let k = &mut st.key_cache[loff + pos * kv_dim..loff + (pos + 1) * kv_dim];
                    for vector in [&mut qb[..], &mut k[..]] {
                        for head in vector.chunks_exact_mut(head_size) {
                            let half = head_size / 2;
                            for i in 0..half {
                                let angle_index = pos * half + i;
                                let cos = self.rope_cos[angle_index];
                                let sin = self.rope_sin[angle_index];
                                let first = head[i];
                                let second = head[i + half];
                                head[i] = first * cos - second * sin;
                                head[i + half] = second * cos + first * sin;
                            }
                        }
                    }
                }

                // Same #602 reference functions as the serial
                // [`Llama::layer_forward`] path: the `r4_attention`
                // switch selects between the two registered operators.
                for h in 0..c.n_heads {
                    let qh = &qb[h * head_size..(h + 1) * head_size];
                    let att = &mut st.att[h * c.seq_len..h * c.seq_len + pos + 1];
                    let kv_head_offset = (h / kv_mul) * head_size;
                    if c.r4_attention {
                        attention::experimental_r4_head_attention_weights(
                            att,
                            qh,
                            &st.key_cache[loff..],
                            kv_head_offset,
                            kv_dim,
                            self.canonical_math,
                        );
                    } else {
                        attention::standard_head_attention_weights(
                            att,
                            qh,
                            &st.key_cache[loff..],
                            kv_head_offset,
                            kv_dim,
                            self.canonical_math,
                        );
                    }
                    let att = &st.att[h * c.seq_len..h * c.seq_len + pos + 1];
                    let outh = &mut out[h * head_size..(h + 1) * head_size];
                    attention::head_attention_value_aggregate(
                        outh,
                        att,
                        &st.value_cache[loff..],
                        kv_head_offset,
                        kv_dim,
                    );
                }
            }

            matmul_batched(&mut o, &attn, &w[self.wo + l * dim * dim..], dim, b);
            for bi in 0..b {
                for i in 0..dim {
                    states[bi].x[i] += o[bi * dim + i];
                }
            }

            for bi in 0..b {
                rmsnorm_with_mode(
                    &mut norm[bi * dim..(bi + 1) * dim],
                    &states[bi].x,
                    &w[self.rms_ffn + l * dim..self.rms_ffn + (l + 1) * dim],
                    self.canonical_math,
                );
            }
            matmul_batched(&mut hb, &norm, &w[self.w1 + l * dim * hid..], dim, b);
            matmul_batched(&mut hb2, &norm, &w[self.w3 + l * dim * hid..], dim, b);
            for idx in 0..b * hid {
                let mut val = hb[idx];
                val *= 1.0f32 / (1.0f32 + expf(-val, self.canonical_math));
                val *= hb2[idx];
                hb[idx] = val;
            }
            matmul_batched(&mut ffn, &hb, &w[self.w2 + l * hid * dim..], hid, b);
            for bi in 0..b {
                for i in 0..dim {
                    states[bi].x[i] += ffn[bi * dim + i];
                }
            }
        }

        let rf = self.rms_final;
        for bi in 0..b {
            let (wslice, x) = (&w[rf..rf + dim], &mut states[bi].x);
            rmsnorm_inplace_with_mode(x, wslice, self.canonical_math);
            xstack[bi * dim..(bi + 1) * dim].copy_from_slice(x);
        }
        let mut logits_stacked = vec![0f32; b * c.vocab];
        matmul_batched(&mut logits_stacked, &xstack, &w[self.wcls..], dim, b);
        for bi in 0..b {
            states[bi]
                .logits
                .copy_from_slice(&logits_stacked[bi * c.vocab..(bi + 1) * c.vocab]);
        }
    }
}

pub trait RepresentationSource {
    fn vocab_size(&self) -> usize;
    fn source_dimension(&self) -> usize;
    fn tokenizer_address(&self) -> &str;
    /// Copy the embedding rows in `range` into `output`. Total: returns
    /// `None` when the caller's `output` buffer cannot hold the requested
    /// rows (a property of the caller's chosen instantiation), `Some(())`
    /// once the rows are written.
    fn read_embedding_rows(&self, range: std::ops::Range<usize>, output: &mut [f32]) -> Option<()>;
}

pub trait BehaviorSource {
    fn reset(&mut self);
    fn step(&mut self, token: usize, pos: usize, logits: &mut [f32]);
}

/// The bounded dimensions a trace-capturing oracle exposes for the #603
/// teacher-trace lanes: how many layers exist (bounding the declarable
/// layer indices), how many attention heads and kv heads each layer has
/// (bounding the attention-support and k/v lane widths), and the width of
/// the residual stream the capture taps (`cfg.dim`, the SOURCE width — for
/// the Hugging Face adapter this is 576, not the compiled 288 that
/// [`TeacherOracle::dim`] presents).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceCaptureGeometry {
    /// Number of transformer layers (declared capture indices must be
    /// strictly below this).
    pub layers: usize,
    /// Attention heads per layer.
    pub heads: usize,
    /// Key/value heads per layer (grouped query).
    pub kv_heads: usize,
    /// Residual-stream width of the captured `st.x` / q vectors (the
    /// source `cfg.dim`). K/v rows are
    /// `residual_width * kv_heads / heads` wide.
    pub residual_width: usize,
}

/// Which layer indices each #603 trace lane captures during one
/// [`TeacherOracle::step_with_trace_capture`]. Empty slices capture
/// nothing for that lane; indices outside the model's layer range are
/// never matched (the caller validates them against
/// [`TraceCaptureGeometry::layers`] up front).
#[derive(Debug, Clone, Copy)]
pub struct TraceCaptureRequest<'a> {
    /// Post-layer residual-stream capture indices.
    pub residual_layers: &'a [usize],
    /// Current-position q/k/v capture indices.
    pub qkv_layers: &'a [usize],
    /// Per-head attention-weight capture indices.
    pub attention_layers: &'a [usize],
}

/// `(layer, post-layer residual stream)` sink of one traced step.
pub type ResidualSink<'a> = dyn FnMut(usize, &[f32]) + 'a;
/// `(layer, rotated q at the current position, k cache row, v cache
/// row)` sink of one traced step.
pub type QkvSink<'a> = dyn FnMut(usize, &[f32], &[f32], &[f32]) + 'a;
/// `(layer, head, softmax weights over positions 0..=pos)` sink of one
/// traced step.
pub type AttentionSupportSink<'a> = dyn FnMut(usize, usize, &[f32]) + 'a;

/// The capture sinks one traced step feeds. Each is called only for the
/// declared layer indices, in ascending layer order (heads ascending
/// within a layer for the attention sink), once per step.
pub struct TraceCaptureSinks<'a, 'b> {
    /// See [`ResidualSink`].
    pub residual: &'a mut ResidualSink<'b>,
    /// See [`QkvSink`].
    pub qkv: &'a mut QkvSink<'b>,
    /// See [`AttentionSupportSink`].
    pub attention: &'a mut AttentionSupportSink<'b>,
}

/// The TWO-SURFACE interface every source architecture must expose to the
/// compiler: the embedding table (representation) and a sequential
/// next-token forward (behavior). The compiler is written against this
/// trait and CANNOT touch anything else — the architecture-generality
/// claim (PROOF.md P4) is enforced by construction, not by inspection.
/// A qwen- or phi-class source implements this trait and nothing
/// downstream changes.
pub trait TeacherOracle: RepresentationSource + BehaviorSource {
    fn vocab(&self) -> usize;
    fn dim(&self) -> usize;
    fn seq_len(&self) -> usize;
    fn bos_token(&self) -> usize {
        1
    }
    fn eos_token(&self) -> usize {
        1
    }
    /// κ of the source artifact this oracle wraps.
    fn kappa(&self) -> String;
    /// Size in bytes of the source artifact (compression accounting).
    fn source_bytes(&self) -> usize;
    /// Copy the embedding row of `token` into `out` (len == dim).
    fn embedding(&self, token: usize, out: &mut [f32]);

    /// The #600 typed record of the source→compiled geometry projection
    /// this oracle applies inside [`TeacherOracle::embedding`], when it
    /// applies one. `None` means embedding rows pass through at the
    /// source width ([`TeacherOracle::dim`] ==
    /// [`RepresentationSource::source_dimension`]); the default keeps
    /// every existing oracle unaffected.
    fn geometry_projection(&self) -> Option<geometry::GeometryProjection> {
        None
    }

    /// The #602 typed record of the attention operator this oracle's
    /// source executor computes during [`BehaviorSource::step`], when the
    /// oracle declares one. The boolean `r4_attention` switch maps to
    /// exactly the two registered operators
    /// (`standard-source-attention/1` when off,
    /// `experimental-r4-source-attention/1` when on — see
    /// [`attention::operator_for_r4_switch`]). `None` means the oracle
    /// predates the record (the legacy interpretation documented in
    /// `docs/MODEL_LIFECYCLE.md`); the default keeps every existing
    /// oracle unaffected, mirroring
    /// [`TeacherOracle::geometry_projection`].
    fn attention_operator_spec(&self) -> Option<attention::AttentionOperatorSpec> {
        None
    }

    /// Optional compiler-only trace surface (graph-compiler plan §5 Phase
    /// 2): the final hidden state (post-final-rmsnorm activation) of the
    /// last `step`, if the oracle retains it. Defaults to `None` so
    /// existing oracles are unaffected.
    fn hidden_state(&self) -> Option<&[f32]> {
        None
    }

    /// Optional compiler-only trace surface: the top-k (token, probability)
    /// pairs of the last `step`'s softmax distribution, ordered by
    /// descending probability with a canonical tie-break (higher
    /// probability, then lower token id). Writes at most
    /// `min(k, out.len())` pairs and returns the count written. Defaults
    /// to 0 so existing oracles are unaffected.
    fn top_k(&self, k: usize, out: &mut [(u32, f32)]) -> usize {
        let _ = (k, out);
        0
    }

    /// Optional compiler-only #603 trace-capture surface: the bounded
    /// capture dimensions this oracle exposes, when it supports the
    /// traced step. `None` means the oracle has no bounded per-layer
    /// capture path (richer trace profiles must be refused, never
    /// zero-filled); the default keeps every existing oracle unaffected,
    /// mirroring [`TeacherOracle::hidden_state`].
    fn trace_capture_geometry(&self) -> Option<TraceCaptureGeometry> {
        None
    }

    /// Optional compiler-only #603 trace-capture step: one forward step
    /// with the declared lanes captured through the exact executor path
    /// (`Llama::forward_capturing_trace`, the #599 `forward_capturing`
    /// discipline extended with the q/k/v and per-head attention taps).
    /// Returns `true` when the oracle captured through its executor;
    /// the default performs a plain [`BehaviorSource::step`] and returns
    /// `false` — the caller must treat `false` as "this oracle has no
    /// capture surface" and refuse the richer profile rather than emit
    /// absent lanes as zeros.
    fn step_with_trace_capture(
        &mut self,
        token: usize,
        pos: usize,
        logits: &mut [f32],
        request: &TraceCaptureRequest<'_>,
        sinks: &mut TraceCaptureSinks<'_, '_>,
    ) -> bool {
        let _ = (request, sinks);
        self.step(token, pos, logits);
        false
    }
}

/// Shared top-k trace computation over the raw logits a llama-family
/// `State` retains after `step`. Softmax is computed in the same f32
/// max-subtracted form the corpus generator uses; ordering is canonical
/// (probability descending, token id ascending on ties).
pub(crate) fn top_k_from_logits(
    logits: &[f32],
    k: usize,
    out: &mut [(u32, f32)],
    canonical_math: bool,
) -> usize {
    let count = k.min(out.len()).min(logits.len());
    if count == 0 {
        return 0;
    }
    let mut max = logits[0];
    for &logit in &logits[1..] {
        if logit > max {
            max = logit;
        }
    }
    let mut sum = 0.0f32;
    let mut probs = vec![0.0f32; logits.len()];
    for (prob, &logit) in probs.iter_mut().zip(logits.iter()) {
        *prob = expf(logit - max, canonical_math);
        sum += *prob;
    }
    for prob in probs.iter_mut() {
        *prob /= sum;
    }
    let mut order: Vec<u32> = (0..logits.len() as u32).collect();
    order.sort_by(|a, b| {
        probs[*b as usize]
            .total_cmp(&probs[*a as usize])
            .then_with(|| a.cmp(b))
    });
    for (dest, &token) in out.iter_mut().zip(order.iter()).take(count) {
        *dest = (token, probs[token as usize]);
    }
    count
}

/// The llama-family adapter: `Llama` plus its recurrent state.
pub struct LlamaOracle {
    pub model: Llama,
    state: State,
    kappa: String,
    source_bytes: usize,
}

impl LlamaOracle {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(path: &str) -> Self {
        let bytes = std::fs::read(path).expect("source checkpoint");
        let kappa = format!("blake3:{}", blake3::hash(&bytes).to_hex());
        let source_bytes = bytes.len();
        let model = Llama::load(path);
        let state = State::new(&model.cfg);
        LlamaOracle {
            model,
            state,
            kappa,
            source_bytes,
        }
    }
}

impl RepresentationSource for LlamaOracle {
    fn vocab_size(&self) -> usize {
        self.model.cfg.vocab
    }
    fn source_dimension(&self) -> usize {
        self.model.cfg.dim
    }
    fn tokenizer_address(&self) -> &str {
        "local-llama-tokenizer"
    }
    fn read_embedding_rows(&self, range: std::ops::Range<usize>, output: &mut [f32]) -> Option<()> {
        let d = self.model.cfg.dim;
        let count = range.end - range.start;
        if output.len() < count * d {
            return None;
        }
        let start_offset = self.model.emb + range.start * d;
        let end_offset = self.model.emb + range.end * d;
        output[..count * d].copy_from_slice(&self.model.w[start_offset..end_offset]);
        Some(())
    }
}

impl BehaviorSource for LlamaOracle {
    fn reset(&mut self) {
        self.state.reset();
    }
    fn step(&mut self, token: usize, pos: usize, logits: &mut [f32]) {
        self.model.forward(&mut self.state, token, pos, false);
        logits.copy_from_slice(&self.state.logits);
    }
}

impl TeacherOracle for LlamaOracle {
    fn vocab(&self) -> usize {
        self.model.cfg.vocab
    }
    fn dim(&self) -> usize {
        self.model.cfg.dim
    }
    fn seq_len(&self) -> usize {
        self.model.cfg.seq_len
    }
    fn kappa(&self) -> String {
        self.kappa.clone()
    }
    fn source_bytes(&self) -> usize {
        self.source_bytes
    }
    fn embedding(&self, token: usize, out: &mut [f32]) {
        let d = self.model.cfg.dim;
        out.copy_from_slice(
            &self.model.w[self.model.emb + token * d..self.model.emb + (token + 1) * d],
        );
    }
    fn hidden_state(&self) -> Option<&[f32]> {
        Some(&self.state.x)
    }
    fn top_k(&self, k: usize, out: &mut [(u32, f32)]) -> usize {
        top_k_from_logits(&self.state.logits, k, out, self.model.canonical_math)
    }
    fn trace_capture_geometry(&self) -> Option<TraceCaptureGeometry> {
        Some(TraceCaptureGeometry {
            layers: self.model.cfg.n_layers,
            heads: self.model.cfg.n_heads,
            kv_heads: self.model.cfg.n_kv_heads,
            residual_width: self.model.cfg.dim,
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn step_with_trace_capture(
        &mut self,
        token: usize,
        pos: usize,
        logits: &mut [f32],
        request: &TraceCaptureRequest<'_>,
        sinks: &mut TraceCaptureSinks<'_, '_>,
    ) -> bool {
        // Same matmul selection as `step` (the legacy exact scalar path),
        // so a traced step and a plain step produce identical bits.
        self.model
            .forward_capturing_trace(&mut self.state, token, pos, false, request, sinks);
        logits.copy_from_slice(&self.state.logits);
        true
    }
}

#[derive(serde::Deserialize)]
struct HuggingFaceConfig {
    hidden_size: usize,
    intermediate_size: usize,
    num_hidden_layers: usize,
    num_attention_heads: usize,
    num_key_value_heads: usize,
    vocab_size: usize,
    max_position_embeddings: usize,
    #[serde(default = "default_rope_theta")]
    rope_theta: f32,
    #[serde(default = "default_rms_epsilon")]
    rms_norm_eps: f32,
    #[serde(default)]
    tie_word_embeddings: bool,
    #[serde(default = "default_bos_token")]
    bos_token_id: usize,
    #[serde(default = "default_eos_token")]
    eos_token_id: usize,
    #[serde(default)]
    rope_interleaved: bool,
}

fn default_rope_theta() -> f32 {
    10_000.0
}
fn default_rms_epsilon() -> f32 {
    1e-5
}
fn default_bos_token() -> usize {
    1
}
fn default_eos_token() -> usize {
    2
}

/// Offline teacher adapter for Hugging Face Llama-family Safetensors
/// (BF16/F16/F32; single-file or #598 indexed shards, ingested through the
/// [`SafetensorsSnapshot`] validation boundary).
/// The full source model executes only while compiling; deployed inference
/// continues to use the multiplication-free [`super::runtime`] tables.
pub struct HuggingFaceLlamaOracle {
    model: Llama,
    state: State,
    kappa: String,
    source_bytes: usize,
    bos_token: usize,
    eos_token: usize,
    fast_matmul: bool,
}

/// The sanctioned host-ingestion boundary error (R5).
///
/// A declared external source artifact — a teacher directory, its
/// `model.safetensors`, `config.json`, an indexed shard set, or a named
/// tensor within — could not be ingested into a valid teacher at
/// construction. This is the host-side analogue of graph-format's
/// `NotAProduct`: the only reportable condition is that the requested object
/// does not exist as a valid product, reported at construction. It carries
/// the operator-facing diagnostic (which file, which tensor, what dtype)
/// rather than discarding it, and — since #598 — the structured
/// [`SourceIngestKind`] classifying which ingestion check refused the
/// artifact, so callers and tests can assert the exact failure class without
/// this crate growing a second error surface.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct SourceUnavailable {
    /// Human-facing description of why the source could not be ingested.
    pub reason: String,
    /// The #598 ingestion failure class, when a validation-boundary check
    /// produced this error; [`SourceIngestKind::Unspecified`] otherwise
    /// (I/O, JSON, or legacy construction sites).
    pub kind: SourceIngestKind,
}

#[cfg(not(target_arch = "wasm32"))]
impl SourceUnavailable {
    /// Construct from any displayable reason (failure class
    /// [`SourceIngestKind::Unspecified`]).
    pub fn new(reason: impl std::fmt::Display) -> Self {
        Self {
            reason: reason.to_string(),
            kind: SourceIngestKind::Unspecified,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl std::fmt::Display for SourceUnavailable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "source unavailable: {}", self.reason)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl std::error::Error for SourceUnavailable {}

#[cfg(not(target_arch = "wasm32"))]
impl From<std::io::Error> for SourceUnavailable {
    fn from(error: std::io::Error) -> Self {
        Self::new(error)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<serde_json::Error> for SourceUnavailable {
    fn from(error: serde_json::Error) -> Self {
        Self::new(error)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<safetensors::SafeTensorError> for SourceUnavailable {
    fn from(error: safetensors::SafeTensorError) -> Self {
        Self::new(error)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<SourceIngestKind> for SourceUnavailable {
    fn from(kind: SourceIngestKind) -> Self {
        Self {
            reason: kind.to_string(),
            kind,
        }
    }
}

/// Exact-widening source codecs (#598): BF16, F16, and F32 → `f32`.
///
/// Every BF16 and F16 value — normals, subnormals, ±0, ±infinity, and NaN —
/// is exactly representable in `f32`, so widening is a pure bit rewrite: no
/// rounding path exists in this module. NaN widening is payload-preserving:
/// the narrow mantissa payload (including the quiet bit) is shifted into the
/// top of the `f32` mantissa (BF16 by 16 bits, F16 by 13 bits) and the sign
/// is kept, so a BF16/F16 NaN widens to an `f32` NaN carrying the same
/// payload bits. F32 is decoded little-endian, bit-identically.
pub mod codec {
    /// Widen one BF16 bit pattern to the `f32` with the identical value.
    /// BF16 is the top half of an `f32`, so this is a 16-bit left shift.
    #[inline]
    pub const fn bf16_to_f32(bits: u16) -> f32 {
        f32::from_bits((bits as u32) << 16)
    }

    /// Widen one IEEE 754 binary16 bit pattern to the `f32` with the
    /// identical value. Normals re-bias the exponent (+112), subnormals are
    /// normalized (every F16 subnormal is an `f32` normal), and ±infinity
    /// and NaN map onto the `f32` exponent 0xFF with the mantissa payload
    /// shifted left by 13.
    #[inline]
    pub const fn f16_to_f32(bits: u16) -> f32 {
        let sign = ((bits as u32) & 0x8000) << 16;
        let exponent = ((bits >> 10) & 0x1F) as u32;
        let mantissa = (bits as u32) & 0x3FF;
        let widened = if exponent == 0x1F {
            // Infinity (mantissa 0) or NaN (payload shifted, preserved).
            sign | 0x7F80_0000 | (mantissa << 13)
        } else if exponent != 0 {
            // Normal: re-bias 15 → 127.
            sign | ((exponent + 112) << 23) | (mantissa << 13)
        } else if mantissa == 0 {
            // ±0.
            sign
        } else {
            // Subnormal: value = mantissa × 2⁻²⁴; normalize the leading one
            // away. With the mantissa's most significant bit at position p,
            // the shift is 10 − p and the biased f32 exponent is 103 + p.
            let shift = mantissa.leading_zeros() - 21;
            sign | ((113 - shift) << 23) | (((mantissa << shift) & 0x3FF) << 13)
        };
        f32::from_bits(widened)
    }

    /// Decode one little-endian F32 value, bit-identically.
    #[inline]
    pub const fn f32_from_le(bytes: [u8; 4]) -> f32 {
        f32::from_le_bytes(bytes)
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Append the exact `f32` widening of `data` (one tensor's raw bytes in
    /// `dtype`) to `out`. The caller guarantees `data.len()` is a multiple
    /// of the dtype's byte size — the #598 validation boundary checks every
    /// tensor's byte length against shape × dtype size before loading.
    pub(crate) fn append_widened(dtype: super::SourceDtype, data: &[u8], out: &mut Vec<f32>) {
        match dtype {
            super::SourceDtype::Bf16 => {
                for bytes in data.chunks_exact(2) {
                    out.push(bf16_to_f32(u16::from_le_bytes([bytes[0], bytes[1]])));
                }
            }
            super::SourceDtype::F16 => {
                for bytes in data.chunks_exact(2) {
                    out.push(f16_to_f32(u16::from_le_bytes([bytes[0], bytes[1]])));
                }
            }
            super::SourceDtype::F32 => {
                for bytes in data.chunks_exact(4) {
                    out.push(f32_from_le([bytes[0], bytes[1], bytes[2], bytes[3]]));
                }
            }
        }
    }
}

/// File name of the Hugging Face Safetensors shard index.
#[cfg(not(target_arch = "wasm32"))]
pub const SAFETENSORS_INDEX_FILE_NAME: &str = "model.safetensors.index.json";
/// File name of the single-file (unsharded) Safetensors weights artifact.
#[cfg(not(target_arch = "wasm32"))]
pub const SAFETENSORS_SINGLE_FILE_NAME: &str = "model.safetensors";

/// The source dtypes this crate can ingest (#598). Each widens exactly to
/// `f32` through [`codec`]; nothing else is admitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceDtype {
    /// bfloat16 — the top half of an `f32`.
    Bf16,
    /// IEEE 754 binary16.
    F16,
    /// IEEE 754 binary32, stored little-endian.
    F32,
}

impl SourceDtype {
    /// Bytes per element in the Safetensors data section.
    pub const fn byte_size(self) -> usize {
        match self {
            Self::Bf16 | Self::F16 => 2,
            Self::F32 => 4,
        }
    }

    /// The Safetensors header spelling of this dtype.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Bf16 => "BF16",
            Self::F16 => "F16",
            Self::F32 => "F32",
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn classify(declared: &str) -> Option<Self> {
        match declared {
            "BF16" => Some(Self::Bf16),
            "F16" => Some(Self::F16),
            "F32" => Some(Self::F32),
            _ => None,
        }
    }
}

/// The #598 source-ingestion failure class: one variant per check at the
/// [`SafetensorsSnapshot::open`] validation boundary, all detected before
/// any model construction. This is NOT a second error surface — the one
/// sanctioned host-ingestion error remains [`SourceUnavailable`] (R5);
/// this enum rides inside it as the structured `kind` field, so each
/// variant names the offending object and callers/tests can still assert
/// the exact failure class.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceIngestKind {
    /// No structured ingestion class: the error came from a plain I/O,
    /// JSON, or legacy construction site ([`SourceUnavailable::new`]).
    Unspecified,
    /// A snapshot file exists but could not be read or parsed (I/O error,
    /// malformed JSON header or index, non-portable shard name).
    Unreadable {
        /// The file (or file: tensor context) that failed.
        path: String,
        /// The underlying parse or I/O diagnostic.
        reason: String,
    },
    /// A shard file referenced by the index (or the single
    /// `model.safetensors`) is absent from the snapshot directory.
    MissingShardFile {
        /// The missing shard's file name.
        shard: String,
    },
    /// A tensor is declared (by the index or by the model geometry) but the
    /// shard that should carry it does not.
    MissingTensor {
        /// The declared tensor name.
        tensor: String,
        /// Where the tensor was expected (a shard file, or the index/single
        /// file for a geometry-required tensor missing everywhere).
        shard: String,
    },
    /// A tensor is claimed more than once: two shard headers both declare
    /// it, or the raw index JSON maps the same name twice.
    DuplicateTensor {
        /// The doubly-claimed tensor name.
        tensor: String,
        /// The first claimant (shard file, or the index file itself).
        first_shard: String,
        /// The second claimant.
        second_shard: String,
    },
    /// A shard header declares a tensor the index does not assign to any
    /// shard. Only possible in indexed mode; a single-file snapshot has no
    /// index, so extra tensors there are ignored exactly as before #598.
    UnexpectedTensor {
        /// The unindexed tensor name.
        tensor: String,
        /// The shard whose header declares it.
        shard: String,
    },
    /// A geometry-required tensor's declared shape does not match the shape
    /// `config.json` implies.
    ShapeMismatch {
        /// The tensor name.
        tensor: String,
        /// The shape the model geometry requires.
        expected: Vec<usize>,
        /// The shape the shard header declares.
        actual: Vec<usize>,
    },
    /// A declared byte length is inconsistent: a tensor's data span differs
    /// from shape × dtype size, the data spans are not contiguous, or the
    /// shard file's size differs from what its header claims.
    ByteLengthMismatch {
        /// Which check failed (shard file, or shard file + tensor).
        context: String,
        /// The byte length the header/shape claims.
        expected: u64,
        /// The byte length actually found.
        actual: u64,
    },
    /// The same tensor name is declared with two different dtypes.
    DtypeInconsistency {
        /// The tensor name.
        tensor: String,
        /// First declaration, as `DTYPE (in <shard>)`.
        first: String,
        /// Conflicting declaration, as `DTYPE (in <shard>)`.
        second: String,
    },
    /// A tensor declares a dtype outside BF16/F16/F32 — e.g. quantized I8,
    /// U8, or GPTQ/AWQ-style packed formats. Rejected by name, never
    /// silently approximated (#598 non-goal).
    UnsupportedDtype {
        /// The tensor name.
        tensor: String,
        /// The declared dtype/format string from the shard header.
        dtype: String,
    },
    /// #599 adapter-conformance gate: the parsed `config.json` declares a
    /// feature outside the adapter's typed
    /// [`conformance::AdapterFeatures`] declaration — an activation, norm
    /// epsilon, RoPE mode, head geometry, bias, embedding-tying, or token
    /// policy the source executor would otherwise silently misinterpret.
    /// Detected at oracle construction, before any tensor is read and
    /// before any observation can be generated (fail-closed).
    UnsupportedConfigFeature {
        /// Which declared feature the configuration falls outside of.
        feature: conformance::AdapterFeature,
        /// What the adapter declares it supports for this feature.
        declared: String,
        /// What `config.json` actually contains.
        actual: String,
    },
    /// #600 geometry registry: a caller named a source→compiled geometry
    /// projection `(id, version)` outside the versioned registry
    /// ([`geometry::projection_implementation`]), so no implementation
    /// exists to interpret it. Refused by name, never approximated by a
    /// "closest" algorithm or version.
    UnknownGeometryProjection {
        /// The requested projection algorithm id.
        id: String,
        /// The requested projection version.
        version: u32,
    },
    /// #601 tokenizer-adapter registry: a caller named a tokenizer
    /// adapter `(family, version)` outside the versioned registry
    /// (`uor_r4_core::transformerless::hf_bpe::adapter_constructor`),
    /// so no constructor exists to interpret it. Refused by name, never
    /// approximated by a "closest" family or version. Registered entries
    /// include `hf-byte-bpe/1` plus frozen `sentencepiece-unigram/1` and
    /// current `sentencepiece-unigram/2`; any unregistered family/version pair
    /// is rejected here.
    UnknownTokenizerAdapter {
        /// The requested adapter family.
        family: String,
        /// The requested adapter version.
        version: u32,
    },
    /// #602 attention-operator registry: a caller named a source
    /// attention operator `(id, version)` outside the versioned registry
    /// ([`attention::operator_spec`]), so no specification exists to
    /// interpret it. Refused by name, never approximated by a "closest"
    /// operator or version.
    UnknownAttentionOperator {
        /// The requested operator id.
        id: String,
        /// The requested operator version.
        version: u32,
    },
}

#[cfg(not(target_arch = "wasm32"))]
impl std::fmt::Display for SourceIngestKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unspecified => write!(f, "no structured ingestion class"),
            Self::Unreadable { path, reason } => write!(f, "{path}: {reason}"),
            Self::MissingShardFile { shard } => {
                write!(
                    f,
                    "shard file {shard} is absent from the snapshot directory"
                )
            }
            Self::MissingTensor { tensor, shard } => {
                write!(f, "tensor {tensor} is declared but absent from {shard}")
            }
            Self::DuplicateTensor {
                tensor,
                first_shard,
                second_shard,
            } => write!(
                f,
                "tensor {tensor} is claimed twice ({first_shard} and {second_shard})"
            ),
            Self::UnexpectedTensor { tensor, shard } => write!(
                f,
                "tensor {tensor} appears in shard {shard} but not in the index"
            ),
            Self::ShapeMismatch {
                tensor,
                expected,
                actual,
            } => write!(
                f,
                "tensor {tensor} has shape {actual:?}, expected {expected:?}"
            ),
            Self::ByteLengthMismatch {
                context,
                expected,
                actual,
            } => write!(
                f,
                "byte-length mismatch at {context}: expected {expected} bytes, found {actual}"
            ),
            Self::DtypeInconsistency {
                tensor,
                first,
                second,
            } => write!(f, "tensor {tensor} is declared {first} and {second}"),
            Self::UnsupportedDtype { tensor, dtype } => write!(
                f,
                "tensor {tensor} is {dtype}; supported source dtypes are BF16, F16, F32 \
                 (quantized formats are rejected, not approximated)"
            ),
            Self::UnsupportedConfigFeature {
                feature,
                declared,
                actual,
            } => write!(
                f,
                "config.json feature {feature} is outside the adapter's declared support: \
                 adapter declares {declared}, configuration has {actual} \
                 (rejected before any observation)"
            ),
            Self::UnknownGeometryProjection { id, version } => write!(
                f,
                "geometry projection {id}/{version} is not in the versioned projection \
                 registry (known: {}/{})",
                geometry::GeometryProjection::BUCKET_AVERAGE_ID,
                geometry::GeometryProjection::BUCKET_AVERAGE_VERSION,
            ),
            Self::UnknownTokenizerAdapter { family, version } => write!(
                f,
                "tokenizer adapter {family}/{version} is not in the versioned adapter \
                 registry (registered entries include hf-byte-bpe/1, \
                 sentencepiece-unigram/1, and sentencepiece-unigram/2; unknown families or \
                 versions are never approximated)"
            ),
            Self::UnknownAttentionOperator { id, version } => write!(
                f,
                "attention operator {id}/{version} is not in the versioned operator \
                 registry (known: {}/{} and {}/{})",
                attention::AttentionOperatorSpec::STANDARD_ID,
                attention::AttentionOperatorSpec::STANDARD_VERSION,
                attention::AttentionOperatorSpec::EXPERIMENTAL_R4_ID,
                attention::AttentionOperatorSpec::EXPERIMENTAL_R4_VERSION,
            ),
        }
    }
}

/// One tensor the model geometry requires from a snapshot: its name and the
/// shape `config.json` implies. [`SafetensorsSnapshot::open`] verifies
/// presence, shape, and (implicitly, via the global dtype check) codec
/// support for every requirement before the model is constructed.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TensorRequirement {
    /// The Safetensors tensor name.
    pub name: String,
    /// The required shape.
    pub shape: Vec<usize>,
}

/// One tensor's location and declared metadata within a loaded shard.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
struct TensorMeta {
    dtype: SourceDtype,
    shape: Vec<usize>,
    /// Byte offsets relative to the shard's data section.
    begin: usize,
    end: usize,
}

/// One validated shard: its file name, raw bytes, data-section offset, and
/// header-declared tensors.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
struct Shard {
    name: String,
    bytes: Vec<u8>,
    data_start: usize,
    tensors: std::collections::BTreeMap<String, TensorMeta>,
}

/// JSON map entries in document order, duplicate keys preserved. serde_json
/// maps silently drop duplicate keys, so both the shard-index `weight_map`
/// and the Safetensors headers are parsed through this visitor to make
/// duplicate tensor names detectable.
#[cfg(not(target_arch = "wasm32"))]
struct MapEntries(Vec<(String, serde_json::Value)>);

#[cfg(not(target_arch = "wasm32"))]
impl<'de> serde::Deserialize<'de> for MapEntries {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EntriesVisitor;
        impl<'de> serde::de::Visitor<'de> for EntriesVisitor {
            type Value = MapEntries;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a JSON object")
            }
            fn visit_map<D>(self, mut map: D) -> Result<Self::Value, D::Error>
            where
                D: serde::de::MapAccess<'de>,
            {
                let mut entries = Vec::new();
                while let Some(entry) = map.next_entry::<String, serde_json::Value>()? {
                    entries.push(entry);
                }
                Ok(MapEntries(entries))
            }
        }
        deserializer.deserialize_map(EntriesVisitor)
    }
}

/// The raw `model.safetensors.index.json` document. Unknown fields (e.g.
/// `metadata.total_size`) are tolerated; `weight_map` keeps duplicate keys
/// visible for the duplicate-tensor check.
#[cfg(not(target_arch = "wasm32"))]
#[derive(serde::Deserialize)]
struct RawShardIndex {
    weight_map: MapEntries,
}

/// One tensor's declared header record: dtype spelling, shape, and data
/// span. Mirrors the Safetensors header schema.
#[cfg(not(target_arch = "wasm32"))]
#[derive(serde::Deserialize)]
struct RawTensorInfo {
    dtype: String,
    shape: Vec<u64>,
    data_offsets: (u64, u64),
}

/// A fully validated Safetensors snapshot: THE #598 ingestion boundary.
///
/// [`SafetensorsSnapshot::open`] is the single deterministic validation
/// entry point for both snapshot layouts:
///
/// * **Indexed shards** — `model.safetensors.index.json` is parsed and every
///   tensor is resolved to exactly one shard file.
/// * **Single file** — `model.safetensors` alone is treated as a one-shard
///   resolution with no index (the pre-#598 layout); it flows through the
///   same code path, so behavior and κ are unchanged.
///
/// Validation runs over all shard headers BEFORE any tensor data is decoded
/// or any model is constructed, and every failure class maps to one
/// [`SourceIngestKind`] variant: missing shard files and tensors,
/// duplicate and unexpected tensors, shape mismatches against the model
/// geometry, byte-length mismatches (tensor span vs shape × dtype size,
/// non-contiguous spans, shard file size vs header claim), dtype
/// inconsistencies, and unsupported (e.g. quantized) dtypes.
///
/// Seam with the #597 snapshot manifest: this boundary checks only that
/// every shard file the index references exists in the directory — the
/// byte-length checks above come from the shard headers themselves. The
/// full `source_manifest.json` digest cross-check (path/bytes/blake3 per
/// admitted file, root κ) belongs to the root crate
/// (`src/model.rs::read_source_manifest`); `uor-r4-model-source` stays free
/// of that dependency by design.
///
/// The snapshot κ is `blake3:<hex>` over the concatenated shard bytes in
/// lexicographic shard-name order. A single-file snapshot therefore keeps
/// exactly the pre-#598 κ (the hash of `model.safetensors` alone); the
/// index bytes are bound by the #597 manifest, not by this κ.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub struct SafetensorsSnapshot {
    /// Shards in lexicographic file-name order.
    shards: Vec<Shard>,
    /// Tensor name → index into `shards`.
    resolved: std::collections::BTreeMap<String, usize>,
    /// Where a geometry-required tensor is reported missing from: the index
    /// file name in indexed mode, the single file name otherwise.
    origin: String,
    kappa: String,
    source_bytes: usize,
}

#[cfg(not(target_arch = "wasm32"))]
impl SafetensorsSnapshot {
    /// Open and validate a snapshot directory. See the type-level docs for
    /// the full contract; this is the one deterministic #598 validation
    /// boundary, and it fails BEFORE any model construction.
    pub fn open(
        dir: impl AsRef<std::path::Path>,
        required: &[TensorRequirement],
    ) -> Result<Self, SourceUnavailable> {
        use std::collections::{BTreeMap, BTreeSet};
        let dir = dir.as_ref();
        let index_path = dir.join(SAFETENSORS_INDEX_FILE_NAME);

        // Resolve the shard set: parse the index, or synthesize the
        // single-file one-shard resolution.
        let (index_map, shard_names, origin) = if index_path.is_file() {
            let bytes = std::fs::read(&index_path)
                .map_err(|error| unreadable(&index_path.display().to_string(), &error))?;
            let raw: RawShardIndex = serde_json::from_slice(&bytes)
                .map_err(|error| unreadable(&index_path.display().to_string(), &error))?;
            let mut map = BTreeMap::new();
            let mut names = BTreeSet::new();
            for (tensor, value) in raw.weight_map.0 {
                let Some(shard) = value.as_str() else {
                    return Err(unreadable(
                        &index_path.display().to_string(),
                        &format!("weight_map entry {tensor} is not a string"),
                    )
                    .into());
                };
                if shard.is_empty() || shard.contains(['/', '\\']) || shard.contains("..") {
                    return Err(unreadable(
                        &index_path.display().to_string(),
                        &format!("shard name {shard:?} is not a bare file name"),
                    )
                    .into());
                }
                names.insert(shard.to_owned());
                if let Some(previous) = map.insert(tensor.clone(), shard.to_owned()) {
                    return Err(SourceIngestKind::DuplicateTensor {
                        tensor,
                        first_shard: previous,
                        second_shard: shard.to_owned(),
                    }
                    .into());
                }
            }
            (Some(map), names, SAFETENSORS_INDEX_FILE_NAME.to_owned())
        } else {
            let mut names = BTreeSet::new();
            names.insert(SAFETENSORS_SINGLE_FILE_NAME.to_owned());
            (None, names, SAFETENSORS_SINGLE_FILE_NAME.to_owned())
        };

        // Pass 1 — load every shard and validate its header in isolation.
        let mut shards = Vec::with_capacity(shard_names.len());
        for name in &shard_names {
            let path = dir.join(name);
            if !path.is_file() {
                return Err(SourceIngestKind::MissingShardFile {
                    shard: name.clone(),
                }
                .into());
            }
            let bytes = crate::progress::read_file(&path, "loading Safetensors")
                .map_err(|error| unreadable(&path.display().to_string(), &error.reason))?;
            shards.push(parse_and_validate_shard(name, bytes)?);
        }

        // Pass 2 — global resolution: every tensor to exactly one shard.
        let mut resolved: BTreeMap<String, usize> = BTreeMap::new();
        for (index, shard) in shards.iter().enumerate() {
            for (tensor, meta) in &shard.tensors {
                if let Some(&previous) = resolved.get(tensor) {
                    let previous_shard = &shards[previous];
                    let previous_meta = &previous_shard.tensors[tensor];
                    if previous_meta.dtype != meta.dtype {
                        return Err(SourceIngestKind::DtypeInconsistency {
                            tensor: tensor.clone(),
                            first: format!(
                                "{} (in {})",
                                previous_meta.dtype.name(),
                                previous_shard.name
                            ),
                            second: format!("{} (in {})", meta.dtype.name(), shard.name),
                        }
                        .into());
                    }
                    return Err(SourceIngestKind::DuplicateTensor {
                        tensor: tensor.clone(),
                        first_shard: previous_shard.name.clone(),
                        second_shard: shard.name.clone(),
                    }
                    .into());
                }
                resolved.insert(tensor.clone(), index);
            }
        }

        // Pass 3 — cross-check the index against the shard headers.
        if let Some(index_map) = &index_map {
            for (tensor, shard_name) in index_map {
                let missing = match resolved.get(tensor) {
                    Some(&index) => &shards[index].name != shard_name,
                    None => true,
                };
                if missing {
                    return Err(SourceIngestKind::MissingTensor {
                        tensor: tensor.clone(),
                        shard: shard_name.clone(),
                    }
                    .into());
                }
            }
            for shard in &shards {
                for tensor in shard.tensors.keys() {
                    if !index_map.contains_key(tensor) {
                        return Err(SourceIngestKind::UnexpectedTensor {
                            tensor: tensor.clone(),
                            shard: shard.name.clone(),
                        }
                        .into());
                    }
                }
            }
        }

        // Pass 4 — the model geometry's requirements: presence and shape.
        // (Dtype support was already validated for every declared tensor.)
        for requirement in required {
            let Some(&index) = resolved.get(&requirement.name) else {
                return Err(SourceIngestKind::MissingTensor {
                    tensor: requirement.name.clone(),
                    shard: origin.clone(),
                }
                .into());
            };
            let meta = &shards[index].tensors[&requirement.name];
            if meta.shape != requirement.shape {
                return Err(SourceIngestKind::ShapeMismatch {
                    tensor: requirement.name.clone(),
                    expected: requirement.shape.clone(),
                    actual: meta.shape.clone(),
                }
                .into());
            }
        }

        // Identity: blake3 over concatenated shard bytes in name order.
        // With one shard (the single-file layout) this is exactly the
        // pre-#598 κ of `model.safetensors`.
        let mut hasher = blake3::Hasher::new();
        let mut source_bytes = 0usize;
        for shard in &shards {
            hasher.update(&shard.bytes);
            source_bytes += shard.bytes.len();
        }
        let kappa = format!("blake3:{}", hasher.finalize().to_hex());
        Ok(Self {
            shards,
            resolved,
            origin,
            kappa,
            source_bytes,
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl SafetensorsSnapshot {
    /// The snapshot κ: `blake3:<hex>` over the concatenated shard bytes in
    /// lexicographic shard-name order (single-file: the file's own hash,
    /// unchanged from before #598).
    pub fn kappa(&self) -> &str {
        &self.kappa
    }

    /// Total bytes across all shard files.
    pub fn source_bytes(&self) -> usize {
        self.source_bytes
    }

    /// Append tensor `name`, widened exactly to `f32` through [`codec`],
    /// to `out`. Fails only for a name outside the validated resolution.
    pub fn tensor_f32_into(&self, name: &str, out: &mut Vec<f32>) -> Result<(), SourceUnavailable> {
        let Some(&index) = self.resolved.get(name) else {
            return Err(SourceIngestKind::MissingTensor {
                tensor: name.to_owned(),
                shard: self.origin.clone(),
            }
            .into());
        };
        let shard = &self.shards[index];
        let meta = &shard.tensors[name];
        let data = &shard.bytes[shard.data_start + meta.begin..shard.data_start + meta.end];
        codec::append_widened(meta.dtype, data, out);
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn unreadable(path: &str, reason: &impl std::fmt::Display) -> SourceIngestKind {
    SourceIngestKind::Unreadable {
        path: path.to_owned(),
        reason: reason.to_string(),
    }
}

/// Parse one shard's Safetensors header and validate it in isolation:
/// dtype support, tensor span vs shape × dtype size, span contiguity, and
/// file size vs header claim. Duplicate names within one header are
/// detected on the raw JSON (before map collapse).
#[cfg(not(target_arch = "wasm32"))]
fn parse_and_validate_shard(name: &str, bytes: Vec<u8>) -> Result<Shard, SourceUnavailable> {
    let file_len = bytes.len() as u64;
    if bytes.len() < 8 {
        return Err(SourceIngestKind::ByteLengthMismatch {
            context: format!("{name}: header length prefix"),
            expected: 8,
            actual: file_len,
        }
        .into());
    }
    let header_len = u64::from_le_bytes(bytes[..8].try_into().expect("eight bytes"));
    let data_start = header_len
        .checked_add(8)
        .filter(|start| *start <= file_len)
        .ok_or(SourceIngestKind::ByteLengthMismatch {
            context: format!("{name}: header"),
            expected: header_len.saturating_add(8),
            actual: file_len,
        })?;
    let header = std::str::from_utf8(&bytes[8..data_start as usize])
        .map_err(|error| unreadable(name, &error))?;
    let entries: MapEntries =
        serde_json::from_str(header).map_err(|error| unreadable(name, &error))?;

    let mut tensors: std::collections::BTreeMap<String, TensorMeta> =
        std::collections::BTreeMap::new();
    for (tensor, value) in entries.0 {
        if tensor == "__metadata__" {
            continue;
        }
        let info: RawTensorInfo = serde_json::from_value(value)
            .map_err(|error| unreadable(&format!("{name}: tensor {tensor}"), &error))?;
        let Some(dtype) = SourceDtype::classify(&info.dtype) else {
            return Err(SourceIngestKind::UnsupportedDtype {
                tensor,
                dtype: info.dtype,
            }
            .into());
        };
        let mut shape = Vec::with_capacity(info.shape.len());
        for &extent in &info.shape {
            let extent = usize::try_from(extent)
                .map_err(|error| unreadable(&format!("{name}: tensor {tensor}"), &error))?;
            shape.push(extent);
        }
        let elements = shape
            .iter()
            .copied()
            .try_fold(1usize, usize::checked_mul)
            .ok_or_else(|| {
                unreadable(
                    &format!("{name}: tensor {tensor}"),
                    &"shape element count overflows",
                )
            })?;
        let claimed = elements as u64 * dtype.byte_size() as u64;
        let (begin, end) = info.data_offsets;
        if end < begin || end - begin != claimed {
            return Err(SourceIngestKind::ByteLengthMismatch {
                context: format!("{name}: tensor {tensor} data length vs shape×dtype size"),
                expected: claimed,
                actual: end.saturating_sub(begin),
            }
            .into());
        }
        let meta = TensorMeta {
            dtype,
            shape,
            begin: usize::try_from(begin)
                .map_err(|error| unreadable(&format!("{name}: tensor {tensor}"), &error))?,
            end: usize::try_from(end)
                .map_err(|error| unreadable(&format!("{name}: tensor {tensor}"), &error))?,
        };
        if tensors.insert(tensor.clone(), meta).is_some() {
            return Err(SourceIngestKind::DuplicateTensor {
                tensor,
                first_shard: name.to_owned(),
                second_shard: name.to_owned(),
            }
            .into());
        }
    }

    // Spans must tile the data section contiguously from zero.
    let mut spans: Vec<(usize, usize, &String)> = tensors
        .iter()
        .map(|(tensor, meta)| (meta.begin, meta.end, tensor))
        .collect();
    spans.sort_unstable();
    let mut cursor = 0usize;
    for (begin, end, tensor) in spans {
        if begin != cursor {
            return Err(SourceIngestKind::ByteLengthMismatch {
                context: format!("{name}: tensor {tensor} data offsets are not contiguous"),
                expected: cursor as u64,
                actual: begin as u64,
            }
            .into());
        }
        cursor = end;
    }
    let claimed_file_len = data_start + cursor as u64;
    if claimed_file_len != file_len {
        return Err(SourceIngestKind::ByteLengthMismatch {
            context: format!("{name}: file size vs header claim"),
            expected: claimed_file_len,
            actual: file_len,
        }
        .into());
    }
    Ok(Shard {
        name: name.to_owned(),
        bytes,
        data_start: data_start as usize,
        tensors,
    })
}

/// The tensors (name + shape) a Llama-family teacher requires, in the
/// flattened append order `load_inner` uses. Derived entirely from the
/// `config.json` geometry, so shape validation happens at the #598 boundary
/// before the model is constructed.
#[cfg(not(target_arch = "wasm32"))]
fn required_llama_tensors(cfg: &Config, tie_word_embeddings: bool) -> Vec<TensorRequirement> {
    let (dim, hidden, vocab) = (cfg.dim, cfg.hidden, cfg.vocab);
    let kv_dim = cfg.dim * cfg.n_kv_heads / cfg.n_heads;
    let mut required = Vec::new();
    let mut push = |name: String, shape: Vec<usize>| {
        required.push(TensorRequirement { name, shape });
    };
    push("model.embed_tokens.weight".to_owned(), vec![vocab, dim]);
    for (suffix, shape) in [
        ("input_layernorm.weight", vec![dim]),
        ("self_attn.q_proj.weight", vec![dim, dim]),
        ("self_attn.k_proj.weight", vec![kv_dim, dim]),
        ("self_attn.v_proj.weight", vec![kv_dim, dim]),
        ("self_attn.o_proj.weight", vec![dim, dim]),
        ("post_attention_layernorm.weight", vec![dim]),
        ("mlp.gate_proj.weight", vec![hidden, dim]),
        ("mlp.down_proj.weight", vec![dim, hidden]),
        ("mlp.up_proj.weight", vec![hidden, dim]),
    ] {
        for layer in 0..cfg.n_layers {
            push(format!("model.layers.{layer}.{suffix}"), shape.clone());
        }
    }
    push("model.norm.weight".to_owned(), vec![dim]);
    if !tie_word_embeddings {
        push("lm_head.weight".to_owned(), vec![vocab, dim]);
    }
    required
}

/// A teacher that can advance a batch of independent sequences one position
/// each through a single memory-amortized forward. Implemented by the HF oracle
/// (the deployed teacher) and mockable for tests, so the batched observe driver
/// need not name a concrete oracle.
pub trait BatchedTeacher {
    /// The per-sequence decode state this teacher advances. Generic so a
    /// non-Llama architecture (GPT-2) can carry its own state through the
    /// same batched observe driver instead of the Llama `State`.
    type State;
    /// A fresh per-sequence state for this teacher.
    fn new_state(&self) -> Self::State;
    /// Reset a state to begin a new sequence (zero its caches/buffers).
    fn reset_state(&self, state: &mut Self::State);
    /// Mutable view of the logits the last
    /// [`BatchedTeacher::forward_batch_into`] left in `state` — the observe
    /// driver encodes each position in place.
    fn logits_mut<'a>(&self, state: &'a mut Self::State) -> &'a mut [f32];
    /// Maximum context length (teacher-forced positions per article).
    fn seq_len(&self) -> usize;
    /// Vocabulary size (logits length).
    fn vocab(&self) -> usize;
    /// Advance `states.len()` sequences one position each: sequence `b` steps
    /// `tokens[b]` at `positions[b]`, leaving its logits reachable through
    /// [`BatchedTeacher::logits_mut`].
    fn forward_batch_into(&self, states: &mut [Self::State], tokens: &[usize], positions: &[usize]);
}

impl BatchedTeacher for HuggingFaceLlamaOracle {
    type State = State;
    fn new_state(&self) -> State {
        State::new(&self.model.cfg)
    }
    fn reset_state(&self, state: &mut State) {
        state.reset();
    }
    fn logits_mut<'a>(&self, state: &'a mut State) -> &'a mut [f32] {
        &mut state.logits
    }
    fn seq_len(&self) -> usize {
        self.model.cfg.seq_len
    }
    fn vocab(&self) -> usize {
        self.model.cfg.vocab
    }
    fn forward_batch_into(&self, states: &mut [State], tokens: &[usize], positions: &[usize]) {
        self.model
            .forward_batch(states, tokens, positions, self.fast_matmul);
    }
}

impl HuggingFaceLlamaOracle {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(source: impl AsRef<std::path::Path>) -> Result<Self, SourceUnavailable> {
        Self::load_inner(source, None)
    }

    /// This teacher's configuration (dims, heads, vocab, sequence length).
    pub fn cfg(&self) -> &Config {
        &self.model.cfg
    }

    /// Load an offline teacher with a bounded context allocation. Compilation
    /// only needs short trajectories because the deployed runtime consumes an
    /// eight-token window; bounding teacher stories avoids quadratic attention
    /// work at source-model maximum context lengths.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_with_sequence_length(
        source: impl AsRef<std::path::Path>,
        sequence_length: usize,
    ) -> Result<Self, SourceUnavailable> {
        if sequence_length == 0 {
            return Err(SourceUnavailable::new(
                "teacher sequence length must be greater than zero",
            ));
        }
        Self::load_inner(source, Some(sequence_length))
    }

    /// Enable or disable the experimental attention variant
    /// (`experimental-r4-source-attention/1`, #602): a 4-wide-chunked
    /// dot product — truncating the trailing `head_size mod 4`
    /// dimensions from the score — followed by the same softmax the
    /// standard operator applies. Despite the flag's historical name it
    /// is neither quaternionic nor softmax-bypassing (#515 audit).
    pub fn set_r4_attention(&mut self, enable: bool) {
        self.model.cfg.r4_attention = enable;
    }

    /// Check if the experimental attention variant
    /// (`experimental-r4-source-attention/1`) is enabled.
    pub fn r4_attention(&self) -> bool {
        self.model.cfg.r4_attention
    }

    /// One teacher step with the residual stream captured after each layer
    /// in `capture_layers` (#599 conformance trace). Delegates to the exact
    /// executor path [`Llama::forward_capturing`] with this oracle's own
    /// matmul selection, so a traced step and a plain [`BehaviorSource::step`]
    /// produce identical state, logits, and top-k.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn step_with_layer_capture(
        &mut self,
        token: usize,
        pos: usize,
        capture_layers: &[usize],
        sink: &mut dyn FnMut(usize, &[f32]),
    ) {
        self.model.forward_capturing(
            &mut self.state,
            token,
            pos,
            self.fast_matmul,
            capture_layers,
            sink,
        );
    }

    /// The logits the last step (plain or capturing) left in this oracle's
    /// state (#599 conformance trace surface).
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn last_logits(&self) -> &[f32] {
        &self.state.logits
    }

    /// Load the teacher from a snapshot directory: `config.json` plus the
    /// weights as either a single `model.safetensors` or #598 indexed
    /// shards (`model.safetensors.index.json` + shard files). Both layouts
    /// flow through the one [`SafetensorsSnapshot::open`] validation
    /// boundary — resolution, shape, byte-length, and dtype checks all fail
    /// there, before any model construction — and tensors are widened
    /// exactly from BF16/F16/F32 to `f32` through [`codec`]. κ is blake3
    /// over the shard bytes in shard-name order, so single-file snapshots
    /// keep their pre-#598 κ unchanged.
    #[cfg(not(target_arch = "wasm32"))]
    fn load_inner(
        source: impl AsRef<std::path::Path>,
        sequence_length: Option<usize>,
    ) -> Result<Self, SourceUnavailable> {
        let source = source.as_ref();
        let config_bytes = std::fs::read(source.join("config.json"))?;
        // #599: fail-closed adapter-conformance gate. The raw configuration
        // is validated against this adapter's typed feature declaration
        // BEFORE the typed parse, before the #598 snapshot boundary, and
        // therefore before any tensor is read or observation generated. Any
        // feature outside the declaration is a focused
        // [`SourceIngestKind::UnsupportedConfigFeature`] failure.
        let raw_config: serde_json::Value = serde_json::from_slice(&config_bytes)?;
        conformance::AdapterFeatures::huggingface_llama().validate_config(&raw_config)?;
        let config: HuggingFaceConfig = serde_json::from_slice(&config_bytes)?;
        let cfg = Config {
            dim: config.hidden_size,
            hidden: config.intermediate_size,
            n_layers: config.num_hidden_layers,
            n_heads: config.num_attention_heads,
            n_kv_heads: config.num_key_value_heads,
            vocab: config.vocab_size,
            seq_len: sequence_length
                .unwrap_or(config.max_position_embeddings)
                .min(config.max_position_embeddings),
            rope_theta: config.rope_theta,
            rms_norm_eps: config.rms_norm_eps,
            rope_interleaved: config.rope_interleaved,
            r4_attention: false,
        };
        eprintln!(
            "model geometry: vocab={} hidden={} layers={} heads={} kv_heads={}",
            cfg.vocab, cfg.dim, cfg.n_layers, cfg.n_heads, cfg.n_kv_heads
        );
        // #598: one validation boundary for single-file and indexed sharded
        // snapshots; every failure class errors here, before construction.
        let required = required_llama_tensors(&cfg, config.tie_word_embeddings);
        let snapshot = SafetensorsSnapshot::open(source, &required)?;
        eprintln!("widening source tensors (BF16/F16/F32) to the compiler teacher layout...");
        let flattened_len = required
            .iter()
            .map(|requirement| requirement.shape.iter().product::<usize>())
            .sum();
        let mut weights = Vec::with_capacity(flattened_len);
        append_tensor(&snapshot, "model.embed_tokens.weight", &mut weights)?;
        append_layers(
            &snapshot,
            cfg.n_layers,
            "input_layernorm.weight",
            &mut weights,
        )?;
        append_layers(
            &snapshot,
            cfg.n_layers,
            "self_attn.q_proj.weight",
            &mut weights,
        )?;
        append_layers(
            &snapshot,
            cfg.n_layers,
            "self_attn.k_proj.weight",
            &mut weights,
        )?;
        append_layers(
            &snapshot,
            cfg.n_layers,
            "self_attn.v_proj.weight",
            &mut weights,
        )?;
        append_layers(
            &snapshot,
            cfg.n_layers,
            "self_attn.o_proj.weight",
            &mut weights,
        )?;
        append_layers(
            &snapshot,
            cfg.n_layers,
            "post_attention_layernorm.weight",
            &mut weights,
        )?;
        append_layers(
            &snapshot,
            cfg.n_layers,
            "mlp.gate_proj.weight",
            &mut weights,
        )?;
        append_layers(
            &snapshot,
            cfg.n_layers,
            "mlp.down_proj.weight",
            &mut weights,
        )?;
        append_layers(&snapshot, cfg.n_layers, "mlp.up_proj.weight", &mut weights)?;
        append_tensor(&snapshot, "model.norm.weight", &mut weights)?;
        if !config.tie_word_embeddings {
            append_tensor(&snapshot, "lm_head.weight", &mut weights)?;
        }
        let kappa = snapshot.kappa().to_owned();
        let source_bytes = snapshot.source_bytes();
        let canonical_math =
            std::env::var("TLESS_CANONICAL_DETERMINISTIC").is_ok_and(|value| value != "0");
        let mut model = Llama::from_flat(cfg, weights, config.tie_word_embeddings);
        model.canonical_math = canonical_math;
        // `from_flat` builds the fast native-math cache first; rebuild it if
        // D2 canonical math was requested so the cache uses the same libm
        // implementation as the rest of the forward pass.
        if canonical_math {
            model.rebuild_rope_cache();
        }
        let state = State::new(&model.cfg);
        let fast_matmul = !canonical_math && std::env::var("TLESS_EXACT_SCALAR").is_err();
        let backend = if canonical_math {
            "canonical libm scalar (D2)"
        } else if fast_matmul {
            fast_matmul_backend()
        } else {
            "exact scalar (deterministic)"
        };
        eprintln!("teacher model ready (κ {kappa}, matmul={backend})");
        Ok(Self {
            model,
            state,
            kappa,
            source_bytes,
            bos_token: config.bos_token_id,
            eos_token: config.eos_token_id,
            fast_matmul,
        })
    }
}

/// Append one tensor per layer (`model.layers.<n>.<suffix>`) from the
/// validated snapshot, in layer order.
#[cfg(not(target_arch = "wasm32"))]
fn append_layers(
    snapshot: &SafetensorsSnapshot,
    layers: usize,
    suffix: &str,
    out: &mut Vec<f32>,
) -> Result<(), SourceUnavailable> {
    for layer in 0..layers {
        append_tensor(snapshot, &format!("model.layers.{layer}.{suffix}"), out)?;
    }
    Ok(())
}

/// Append tensor `name` from the validated snapshot, widened exactly from
/// its declared source dtype (BF16, F16, or F32) to `f32` through
/// [`codec`]. Dtype support, shape, and byte lengths were already checked
/// at the [`SafetensorsSnapshot::open`] boundary (#598), so this only fails
/// for a name outside the validated resolution.
#[cfg(not(target_arch = "wasm32"))]
fn append_tensor(
    snapshot: &SafetensorsSnapshot,
    name: &str,
    out: &mut Vec<f32>,
) -> Result<(), SourceUnavailable> {
    snapshot.tensor_f32_into(name, out)
}

impl RepresentationSource for HuggingFaceLlamaOracle {
    fn vocab_size(&self) -> usize {
        self.model.cfg.vocab
    }
    fn source_dimension(&self) -> usize {
        self.model.cfg.dim
    }
    fn tokenizer_address(&self) -> &str {
        "huggingface-tokenizer"
    }
    fn read_embedding_rows(&self, range: std::ops::Range<usize>, output: &mut [f32]) -> Option<()> {
        let d = self.model.cfg.dim;
        let count = range.end - range.start;
        if output.len() < count * d {
            return None;
        }
        let start_offset = self.model.emb + range.start * d;
        let end_offset = self.model.emb + range.end * d;
        output[..count * d].copy_from_slice(&self.model.w[start_offset..end_offset]);
        Some(())
    }
}

impl BehaviorSource for HuggingFaceLlamaOracle {
    fn reset(&mut self) {
        self.state.reset();
    }
    fn step(&mut self, token: usize, pos: usize, logits: &mut [f32]) {
        self.model
            .forward(&mut self.state, token, pos, self.fast_matmul);
        logits.copy_from_slice(&self.state.logits);
    }
}

impl TeacherOracle for HuggingFaceLlamaOracle {
    fn vocab(&self) -> usize {
        self.model.cfg.vocab
    }
    fn dim(&self) -> usize {
        // The compiled geometry this adapter presents (D = 288), not the
        // source width; `embedding` projects rows down to it (#600).
        geometry::COMPILED_WIDTH as usize
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
        self.kappa.clone()
    }
    fn source_bytes(&self) -> usize {
        self.source_bytes
    }
    fn embedding(&self, token: usize, out: &mut [f32]) {
        let dim = self.model.cfg.dim;
        let row = &self.model.w[self.model.emb + token * dim..self.model.emb + (token + 1) * dim];
        // #600: the explicitly named `bucket-average/1` projection — the
        // exact arithmetic this method always performed, factored into
        // the versioned geometry module (see `geometry_projection`).
        geometry::bucket_average_project(row, out);
    }
    fn geometry_projection(&self) -> Option<geometry::GeometryProjection> {
        u32::try_from(self.model.cfg.dim).ok().map(|source_width| {
            geometry::GeometryProjection::bucket_average(source_width, geometry::COMPILED_WIDTH)
        })
    }
    fn attention_operator_spec(&self) -> Option<attention::AttentionOperatorSpec> {
        // #602: the boolean switch maps to exactly the two registered
        // operators — this is the one boundary where the legacy flag
        // becomes a versioned identity.
        Some(attention::operator_for_r4_switch(
            self.model.cfg.r4_attention,
        ))
    }
    fn hidden_state(&self) -> Option<&[f32]> {
        Some(&self.state.x)
    }
    fn top_k(&self, k: usize, out: &mut [(u32, f32)]) -> usize {
        top_k_from_logits(&self.state.logits, k, out, self.model.canonical_math)
    }
    fn trace_capture_geometry(&self) -> Option<TraceCaptureGeometry> {
        Some(TraceCaptureGeometry {
            layers: self.model.cfg.n_layers,
            heads: self.model.cfg.n_heads,
            kv_heads: self.model.cfg.n_kv_heads,
            residual_width: self.model.cfg.dim,
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn step_with_trace_capture(
        &mut self,
        token: usize,
        pos: usize,
        logits: &mut [f32],
        request: &TraceCaptureRequest<'_>,
        sinks: &mut TraceCaptureSinks<'_, '_>,
    ) -> bool {
        // This oracle's own matmul selection, exactly as
        // `step_with_layer_capture` (#599) delegates it, so a traced
        // step and a plain step produce identical bits.
        self.model.forward_capturing_trace(
            &mut self.state,
            token,
            pos,
            self.fast_matmul,
            request,
            sinks,
        );
        logits.copy_from_slice(&self.state.logits);
        true
    }
}

/// Backward-compatible name for the first supported Hugging Face model.
pub type SmolLm2Oracle = HuggingFaceLlamaOracle;

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn unknown_tokenizer_adapter_diagnostic_names_current_registry() {
        let error: SourceUnavailable = SourceIngestKind::UnknownTokenizerAdapter {
            family: "unknown-family".to_owned(),
            version: 9,
        }
        .into();
        assert!(matches!(
            error.kind,
            SourceIngestKind::UnknownTokenizerAdapter {
                ref family,
                version: 9,
            } if family == "unknown-family"
        ));
        assert!(error.reason.contains("hf-byte-bpe/1"));
        assert!(error.reason.contains("sentencepiece-unigram/1"));
        assert!(error.reason.contains("sentencepiece-unigram/2"));
        assert!(!error.reason.contains("stays rejected"));
    }

    #[test]
    fn canonical_math_delegates_to_portable_libm() {
        let values = [-7.25f32, -1.0, 0.125, 1.0, 7.25];
        for &value in &values {
            assert_eq!(
                sqrtf(value.abs() + 0.5, true).to_bits(),
                libm::sqrtf(value.abs() + 0.5).to_bits()
            );
            assert_eq!(expf(value, true).to_bits(), libm::expf(value).to_bits());
            assert_eq!(sinf(value, true).to_bits(), libm::sinf(value).to_bits());
            assert_eq!(cosf(value, true).to_bits(), libm::cosf(value).to_bits());
            assert_eq!(
                powf(10_000.0, value / 8.0, true).to_bits(),
                libm::powf(10_000.0, value / 8.0).to_bits()
            );
        }
    }

    #[test]
    fn rope_cache_preserves_angle_bits() {
        let cfg = Config {
            dim: 12,
            hidden: 16,
            n_layers: 1,
            n_heads: 3,
            n_kv_heads: 1,
            vocab: 8,
            seq_len: 5,
            rope_theta: 10_000.0,
            rms_norm_eps: 1e-5,
            rope_interleaved: false,
            r4_attention: false,
        };
        let (cos, sin) = Llama::build_rope_cache(&cfg, false);
        let half = cfg.dim / cfg.n_heads / 2;
        for pos in 0..cfg.seq_len {
            for i in 0..half {
                let freq = 1.0f32 / cfg.rope_theta.powf((2 * i) as f32 / (2 * half) as f32);
                let angle = pos as f32 * freq;
                assert_eq!(cos[pos * half + i].to_bits(), angle.cos().to_bits());
                assert_eq!(sin[pos * half + i].to_bits(), angle.sin().to_bits());
            }
        }
    }

    #[test]
    fn canonical_softmax_is_repeatable() {
        let input = [3.0f32, -1.0, 0.25, 7.0];
        let mut first = input;
        let mut second = input;
        softmax_with_mode(&mut first, true);
        softmax_with_mode(&mut second, true);
        assert_eq!(first, second);
        assert!((first.iter().sum::<f32>() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn fast_matmul_tracks_exact_cpu_result() {
        const ROWS: usize = 67;
        const COLUMNS: usize = 73;
        let input: Vec<f32> = (0..COLUMNS)
            .map(|index| ((index * 17 % 31) as f32 - 15.0) / 16.0)
            .collect();
        let weights: Vec<f32> = (0..ROWS * COLUMNS)
            .map(|index| ((index * 29 % 43) as f32 - 21.0) / 32.0)
            .collect();
        let mut exact = [0.0f32; ROWS];
        let mut fast = [0.0f32; ROWS];
        matmul(&mut exact, &input, &weights, COLUMNS, false);
        matmul(&mut fast, &input, &weights, COLUMNS, true);
        for (expected, actual) in exact.into_iter().zip(fast) {
            let tolerance = 1e-5f32.max(expected.abs() * 1e-5);
            assert!((expected - actual).abs() <= tolerance);
        }
    }

    #[test]
    fn matmul_batched_matches_serial_fast() {
        const N: usize = 73;
        const ROWS: usize = 40;
        const BATCH: usize = 6;
        let weights: Vec<f32> = (0..ROWS * N)
            .map(|index| ((index * 29 % 43) as f32 - 21.0) / 32.0)
            .collect();
        let x: Vec<f32> = (0..BATCH * N)
            .map(|index| ((index * 17 % 31) as f32 - 15.0) / 16.0)
            .collect();
        let mut batched = vec![0.0f32; BATCH * ROWS];
        matmul_batched(&mut batched, &x, &weights, N, BATCH);
        for bi in 0..BATCH {
            let mut serial = [0.0f32; ROWS];
            matmul(&mut serial, &x[bi * N..(bi + 1) * N], &weights, N, true);
            for row in 0..ROWS {
                let want = serial[row];
                let got = batched[bi * ROWS + row];
                // Off macOS the batched kernel reuses the exact same dot_fast as
                // the serial fast path, so it is bit-identical. On macOS it is
                // Accelerate sgemm vs sgemv — deterministic, within tolerance.
                #[cfg(not(target_os = "macos"))]
                assert_eq!(got, want, "batched != serial at b{bi} row{row}");
                #[cfg(target_os = "macos")]
                {
                    let tolerance = 1e-4f32.max(want.abs() * 1e-4);
                    assert!(
                        (got - want).abs() <= tolerance,
                        "batched vs serial b{bi} row{row}"
                    );
                }
            }
        }
    }

    /// A tiny synthetic Llama with deterministic weights, for exercising the
    /// forward path without loading a real checkpoint.
    fn tiny_llama() -> Llama {
        let (dim, hid, nl, kv_dim, vocab, seq_len) =
            (8usize, 16usize, 2usize, 8usize, 10usize, 8usize);
        let cfg = Config {
            dim,
            hidden: hid,
            n_layers: nl,
            n_heads: 2,
            n_kv_heads: 2,
            vocab,
            seq_len,
            rope_theta: 10_000.0,
            rms_norm_eps: 1e-5,
            rope_interleaved: true,
            r4_attention: false,
        };
        let emb = 0;
        let rms_att = emb + vocab * dim;
        let wq = rms_att + nl * dim;
        let wk = wq + nl * dim * dim;
        let wv = wk + nl * dim * kv_dim;
        let wo = wv + nl * dim * kv_dim;
        let rms_ffn = wo + nl * dim * dim;
        let w1 = rms_ffn + nl * dim;
        let w2 = w1 + nl * dim * hid;
        let w3 = w2 + nl * hid * dim;
        let rms_final = w3 + nl * dim * hid;
        let total = rms_final + dim;
        let w: Vec<f32> = (0..total)
            .map(|i| (((i * 131 + 7) % 251) as f32 / 251.0 - 0.5) * 0.2)
            .collect();
        let mut model = Llama {
            cfg,
            w,
            rope_cos: Vec::new(),
            rope_sin: Vec::new(),
            emb,
            rms_att,
            wq,
            wk,
            wv,
            wo,
            rms_ffn,
            w1,
            w2,
            w3,
            rms_final,
            wcls: emb,
            canonical_math: false,
        };
        model.rebuild_rope_cache();
        model
    }

    /// #603: the traced forward IS the exact executor — a
    /// `forward_capturing_trace` stream produces bit-identical logits and
    /// hidden state to a plain `forward` stream, and its taps are bounded
    /// to the declared layer indices: residuals once per declared layer,
    /// q/k/v rows at the current position, and per-head attention weights
    /// that are softmax-normalized over the prefix (each head's captured
    /// row sums to 1 and has `pos + 1` entries).
    #[test]
    fn forward_capturing_trace_matches_plain_forward_with_bounded_taps() {
        let model = tiny_llama();
        let tokens = [1usize, 3, 5, 2, 7];
        let kv_dim = model.cfg.dim * model.cfg.n_kv_heads / model.cfg.n_heads;

        let mut plain = State::new(&model.cfg);
        plain.reset();
        let mut traced = State::new(&model.cfg);
        traced.reset();
        let request = TraceCaptureRequest {
            residual_layers: &[1],
            qkv_layers: &[0],
            attention_layers: &[1],
        };
        for (pos, &token) in tokens.iter().enumerate() {
            model.forward(&mut plain, token, pos, false);

            let mut residuals: Vec<(usize, Vec<f32>)> = Vec::new();
            let mut qkv: Vec<(usize, usize, usize, usize)> = Vec::new();
            let mut attention: Vec<(usize, usize, Vec<f32>)> = Vec::new();
            let mut residual_sink = |layer: usize, x: &[f32]| residuals.push((layer, x.to_vec()));
            let mut qkv_sink = |layer: usize, q: &[f32], k: &[f32], v: &[f32]| {
                qkv.push((layer, q.len(), k.len(), v.len()));
            };
            let mut attention_sink = |layer: usize, head: usize, att: &[f32]| {
                attention.push((layer, head, att.to_vec()))
            };
            model.forward_capturing_trace(
                &mut traced,
                token,
                pos,
                false,
                &request,
                &mut TraceCaptureSinks {
                    residual: &mut residual_sink,
                    qkv: &mut qkv_sink,
                    attention: &mut attention_sink,
                },
            );

            let plain_bits: Vec<u32> = plain.logits.iter().map(|v| v.to_bits()).collect();
            let traced_bits: Vec<u32> = traced.logits.iter().map(|v| v.to_bits()).collect();
            assert_eq!(plain_bits, traced_bits, "logits diverged at pos {pos}");
            let plain_x: Vec<u32> = plain.x.iter().map(|v| v.to_bits()).collect();
            let traced_x: Vec<u32> = traced.x.iter().map(|v| v.to_bits()).collect();
            assert_eq!(plain_x, traced_x, "hidden state diverged at pos {pos}");

            // Bounded taps: exactly the declared layers, once per step.
            assert_eq!(residuals.len(), 1);
            assert_eq!(residuals[0].0, 1);
            assert_eq!(residuals[0].1.len(), model.cfg.dim);
            assert_eq!(qkv, vec![(0, model.cfg.dim, kv_dim, kv_dim)]);
            assert_eq!(attention.len(), model.cfg.n_heads);
            for (head, (layer, got_head, att)) in attention.iter().enumerate() {
                assert_eq!(*layer, 1);
                assert_eq!(*got_head, head);
                assert_eq!(att.len(), pos + 1, "prefix-bounded weights");
                let total: f32 = att.iter().sum();
                assert!((total - 1.0).abs() < 1e-5, "head {head} not normalized");
            }
        }
    }

    /// forward_batch over B sequences, stepped position by position, must
    /// produce the exact logits of B independent forward() streams. Off macOS
    /// the batched matmul reuses the serial dot_fast, so this is bit-identical;
    /// guards the batched teacher path added for #531.
    #[cfg(not(target_os = "macos"))]
    #[test]
    #[allow(clippy::needless_range_loop)]
    fn forward_batch_matches_serial_forward() {
        let model = tiny_llama();
        let seqs: [[usize; 4]; 3] = [[1, 3, 5, 2], [4, 0, 7, 8], [2, 9, 1, 6]];
        let len = 4;

        // Serial reference: one State per sequence, stepped token by token.
        let mut serial: Vec<Vec<f32>> = Vec::new();
        let mut sstates: Vec<State> = (0..3).map(|_| State::new(&model.cfg)).collect();
        sstates.iter_mut().for_each(State::reset);
        for pos in 0..len {
            for (b, st) in sstates.iter_mut().enumerate() {
                model.forward(st, seqs[b][pos], pos, true);
                serial.push(st.logits.clone());
            }
        }

        // Batched: all three sequences advanced together each step.
        let mut bstates: Vec<State> = (0..3).map(|_| State::new(&model.cfg)).collect();
        bstates.iter_mut().for_each(State::reset);
        for pos in 0..len {
            let tokens: Vec<usize> = (0..3).map(|b| seqs[b][pos]).collect();
            let positions = vec![pos; 3];
            model.forward_batch(&mut bstates, &tokens, &positions, true);
            for (b, st) in bstates.iter().enumerate() {
                assert_eq!(
                    st.logits,
                    serial[pos * 3 + b],
                    "logits differ at pos {pos} seq {b}"
                );
            }
        }
    }

    /// Raw teacher throughput: serial `step` vs batched `forward_batch_into` on
    /// a real teacher. Ignored by default; run against a model directory:
    ///   TLESS_BENCH_MODEL=.uor-models/sources/smollm2-360m-instruct \
    ///   cargo test -p uor-r4-model-source --release bench_forward_batch \
    ///     -- --ignored --nocapture
    /// Optionally TLESS_BENCH_B=16 (batch), TLESS_BENCH_STEPS=128 (positions).
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    #[ignore]
    fn bench_forward_batch() {
        use std::time::Instant;
        let dir = std::env::var("TLESS_BENCH_MODEL")
            .expect("set TLESS_BENCH_MODEL to a teacher source directory");
        let steps: usize = std::env::var("TLESS_BENCH_STEPS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(128);
        let batch: usize = std::env::var("TLESS_BENCH_B")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(16);
        let mut oracle =
            HuggingFaceLlamaOracle::load_with_sequence_length(&dir, 128).expect("load teacher");
        let vocab = oracle.cfg().vocab;

        // Serial baseline: one sequence, `steps` positions via step().
        let mut logits = vec![0f32; vocab];
        oracle.reset();
        let t = Instant::now();
        for pos in 0..steps {
            oracle.step((pos % vocab).max(1), pos, &mut logits);
        }
        let serial_s = t.elapsed().as_secs_f64();
        let serial_tps = steps as f64 / serial_s;

        // Batched: `batch` sequences, `steps` positions each.
        let mut states: Vec<State> = (0..batch).map(|_| oracle.new_state()).collect();
        states.iter_mut().for_each(State::reset);
        let t = Instant::now();
        for pos in 0..steps {
            let tokens: Vec<usize> = (0..batch).map(|b| ((pos + b) % vocab).max(1)).collect();
            let positions = vec![pos; batch];
            oracle.forward_batch_into(&mut states, &tokens, &positions);
        }
        let batched_s = t.elapsed().as_secs_f64();
        let batched_tps = (steps * batch) as f64 / batched_s;

        eprintln!(
            "BENCH serial: {steps} tok in {serial_s:.3}s = {serial_tps:.1} tok/s | \
             batched B={batch}: {} tok in {batched_s:.3}s = {batched_tps:.1} tok/s | \
             speedup {:.2}x",
            steps * batch,
            batched_tps / serial_tps
        );
    }

    /// #598 source-codec bit-pattern tables: exact widening of BF16, F16,
    /// and F32 against hand-written hex constants — normals, subnormals,
    /// ±0, ±infinity, and payload-carrying NaNs. Equality is on raw bits.
    mod codec_bits {
        use crate::codec;

        /// (bf16 bits, expected f32 bits). BF16 → f32 is a 16-bit shift, so
        /// every class widens exactly, including NaN payloads.
        const BF16_WIDENING: &[(u16, u32)] = &[
            (0x0000, 0x0000_0000), // +0
            (0x8000, 0x8000_0000), // -0
            (0x3F80, 0x3F80_0000), // 1.0
            (0xBF80, 0xBF80_0000), // -1.0
            (0x4000, 0x4000_0000), // 2.0
            (0x3FC0, 0x3FC0_0000), // 1.5
            (0xC2F7, 0xC2F7_0000), // -123.5
            (0x0001, 0x0001_0000), // min subnormal (widens to an f32 subnormal)
            (0x007F, 0x007F_0000), // max subnormal
            (0x0080, 0x0080_0000), // min normal 2^-126
            (0x7F7F, 0x7F7F_0000), // max finite
            (0x7F80, 0x7F80_0000), // +inf
            (0xFF80, 0xFF80_0000), // -inf
            (0x7FC0, 0x7FC0_0000), // quiet NaN
            (0x7F81, 0x7F81_0000), // NaN, payload 0x01 preserved
            (0xFFC1, 0xFFC1_0000), // negative NaN, payload preserved
        ];

        /// (f16 bits, expected f32 bits). Normals re-bias (+112),
        /// subnormals normalize to f32 normals, NaN payloads shift by 13.
        const F16_WIDENING: &[(u16, u32)] = &[
            (0x0000, 0x0000_0000), // +0
            (0x8000, 0x8000_0000), // -0
            (0x3C00, 0x3F80_0000), // 1.0
            (0xBC00, 0xBF80_0000), // -1.0
            (0x4000, 0x4000_0000), // 2.0
            (0x3E00, 0x3FC0_0000), // 1.5
            (0x3555, 0x3EAA_A000), // 0.333251953125
            (0x7BFF, 0x477F_E000), // 65504.0 (max finite)
            (0xFBFF, 0xC77F_E000), // -65504.0
            (0x0400, 0x3880_0000), // min normal 2^-14
            (0x0001, 0x3380_0000), // min subnormal 2^-24
            (0x8001, 0xB380_0000), // -min subnormal
            (0x03FF, 0x387F_C000), // max subnormal 1023×2^-24
            (0x7C00, 0x7F80_0000), // +inf
            (0xFC00, 0xFF80_0000), // -inf
            (0x7E00, 0x7FC0_0000), // quiet NaN
            (0x7C01, 0x7F80_2000), // NaN, payload 0x001 → mantissa 0x001<<13
            (0xFE2A, 0xFFC5_4000), // negative NaN, payload 0x22A preserved
        ];

        /// F32 little-endian decode is bit-identity.
        const F32_IDENTITY: &[u32] = &[
            0x0000_0000, // +0
            0x8000_0000, // -0
            0x3F80_0000, // 1.0
            0xC2F6_E979, // -123.456...
            0x0000_0001, // min subnormal
            0x7F80_0000, // +inf
            0xFF80_0000, // -inf
            0x7FC0_0001, // quiet NaN with payload
            0x7F80_0001, // signaling NaN bit pattern
        ];

        #[test]
        fn bf16_widening_bit_patterns() {
            for &(bits, expected) in BF16_WIDENING {
                assert_eq!(
                    codec::bf16_to_f32(bits).to_bits(),
                    expected,
                    "bf16 {bits:#06x}"
                );
            }
        }

        #[test]
        fn f16_widening_bit_patterns() {
            for &(bits, expected) in F16_WIDENING {
                assert_eq!(
                    codec::f16_to_f32(bits).to_bits(),
                    expected,
                    "f16 {bits:#06x}"
                );
            }
        }

        #[test]
        fn f32_decode_is_bit_identity() {
            for &bits in F32_IDENTITY {
                assert_eq!(
                    codec::f32_from_le(bits.to_le_bytes()).to_bits(),
                    bits,
                    "f32 {bits:#010x}"
                );
            }
        }

        /// Exact round-trip: an f32 value representable in the narrow type
        /// survives narrow → widen unchanged, bit for bit.
        #[test]
        fn bf16_round_trip_values_are_exact() {
            for value in [1.0f32, -2.5, 0.156_25, 3.875, -0.007_812_5] {
                let narrow = (value.to_bits() >> 16) as u16;
                assert_eq!(codec::bf16_to_f32(narrow).to_bits(), value.to_bits());
            }
        }

        #[test]
        fn f16_round_trip_values_are_exact() {
            for (narrow, value) in [
                (0x3C00u16, 1.0f32),
                (0xB800, -0.5),
                (0x7BFF, 65504.0),
                (0x0400, f32::from_bits(0x3880_0000)), // 2^-14
            ] {
                assert_eq!(codec::f16_to_f32(narrow).to_bits(), value.to_bits());
            }
        }

        /// The byte-stream widening path used by tensor loading.
        #[cfg(not(target_arch = "wasm32"))]
        #[test]
        fn append_widened_decodes_each_dtype() {
            let mut out = Vec::new();
            codec::append_widened(
                crate::SourceDtype::Bf16,
                &[0x80, 0x3F, 0x00, 0xC0], // 1.0, -2.0
                &mut out,
            );
            codec::append_widened(
                crate::SourceDtype::F16,
                &[0x00, 0x3C, 0x00, 0xB8], // 1.0, -0.5
                &mut out,
            );
            codec::append_widened(crate::SourceDtype::F32, &1.5f32.to_le_bytes(), &mut out);
            assert_eq!(out, [1.0, -2.0, 1.0, -0.5, 1.5]);
        }
    }

    /// #598 ingestion-boundary tests: synthetic Safetensors shards and
    /// indexes built in-test (std only), one test per failure class, plus
    /// the sharded-equals-single-file round trip.
    #[cfg(not(target_arch = "wasm32"))]
    mod shard_ingestion {
        use crate::{
            HuggingFaceLlamaOracle, SafetensorsSnapshot, SourceIngestKind, TensorRequirement,
        };
        use std::path::{Path, PathBuf};
        use std::sync::atomic::{AtomicUsize, Ordering};

        const SHARD_1: &str = "model-00001-of-00002.safetensors";
        const SHARD_2: &str = "model-00002-of-00002.safetensors";

        fn temp_snapshot_dir(tag: &str) -> PathBuf {
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let dir = std::env::temp_dir().join(format!(
                "uor-r4-i598-{tag}-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(&dir).expect("create synthetic snapshot dir");
            dir
        }

        fn raw_shard(header: &str, data: &[u8]) -> Vec<u8> {
            let mut out = (header.len() as u64).to_le_bytes().to_vec();
            out.extend_from_slice(header.as_bytes());
            out.extend_from_slice(data);
            out
        }

        /// Build a well-formed shard: contiguous offsets in entry order.
        fn shard_bytes(tensors: &[(&str, &str, &[usize], &[u8])]) -> Vec<u8> {
            let mut entries = Vec::new();
            let mut data = Vec::new();
            let mut offset = 0usize;
            for (name, dtype, shape, bytes) in tensors {
                let end = offset + bytes.len();
                let dims = shape
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                entries.push(format!(
                    "\"{name}\":{{\"dtype\":\"{dtype}\",\"shape\":[{dims}],\
                     \"data_offsets\":[{offset},{end}]}}"
                ));
                data.extend_from_slice(bytes);
                offset = end;
            }
            raw_shard(&format!("{{{}}}", entries.join(",")), &data)
        }

        fn index_bytes(entries: &[(&str, &str)]) -> Vec<u8> {
            let body = entries
                .iter()
                .map(|(tensor, shard)| format!("\"{tensor}\":\"{shard}\""))
                .collect::<Vec<_>>()
                .join(",");
            format!("{{\"metadata\":{{\"total_size\":0}},\"weight_map\":{{{body}}}}}").into_bytes()
        }

        fn write(dir: &Path, name: &str, bytes: &[u8]) {
            std::fs::write(dir.join(name), bytes).expect("write synthetic snapshot file");
        }

        fn req(name: &str, shape: &[usize]) -> TensorRequirement {
            TensorRequirement {
                name: name.to_owned(),
                shape: shape.to_vec(),
            }
        }

        /// Deterministic finite BF16 payload bytes for `n` elements.
        fn bf16_data(n: usize, salt: u16) -> Vec<u8> {
            (0..n)
                .flat_map(|i| {
                    (0x3F00u16 | ((i as u16).wrapping_mul(31).wrapping_add(salt) & 0x7F))
                        .to_le_bytes()
                })
                .collect()
        }

        #[test]
        fn missing_shard_file_is_detected() {
            let dir = temp_snapshot_dir("missing-shard");
            write(
                &dir,
                "model.safetensors.index.json",
                &index_bytes(&[("a", SHARD_1), ("b", SHARD_2)]),
            );
            write(
                &dir,
                SHARD_1,
                &shard_bytes(&[("a", "BF16", &[2], &bf16_data(2, 1))]),
            );
            let error = SafetensorsSnapshot::open(&dir, &[]).unwrap_err();
            assert_eq!(
                error.kind,
                SourceIngestKind::MissingShardFile {
                    shard: SHARD_2.to_owned()
                }
            );
        }

        #[test]
        fn tensor_missing_from_its_mapped_shard_is_detected() {
            let dir = temp_snapshot_dir("missing-tensor");
            write(
                &dir,
                "model.safetensors.index.json",
                &index_bytes(&[("a", SHARD_1), ("b", SHARD_1)]),
            );
            write(
                &dir,
                SHARD_1,
                &shard_bytes(&[("a", "BF16", &[2], &bf16_data(2, 2))]),
            );
            let error = SafetensorsSnapshot::open(&dir, &[]).unwrap_err();
            assert_eq!(
                error.kind,
                SourceIngestKind::MissingTensor {
                    tensor: "b".to_owned(),
                    shard: SHARD_1.to_owned()
                }
            );
        }

        #[test]
        fn required_tensor_absent_everywhere_is_detected() {
            let dir = temp_snapshot_dir("missing-required");
            write(
                &dir,
                "model.safetensors",
                &shard_bytes(&[("a", "BF16", &[2], &bf16_data(2, 3))]),
            );
            let error = SafetensorsSnapshot::open(&dir, &[req("b", &[2])]).unwrap_err();
            assert_eq!(
                error.kind,
                SourceIngestKind::MissingTensor {
                    tensor: "b".to_owned(),
                    shard: "model.safetensors".to_owned()
                }
            );
        }

        #[test]
        fn duplicate_tensor_across_shards_is_detected() {
            let dir = temp_snapshot_dir("dup-shards");
            write(
                &dir,
                "model.safetensors.index.json",
                &index_bytes(&[("a", SHARD_1), ("b", SHARD_2)]),
            );
            write(
                &dir,
                SHARD_1,
                &shard_bytes(&[("a", "BF16", &[2], &bf16_data(2, 4))]),
            );
            write(
                &dir,
                SHARD_2,
                &shard_bytes(&[
                    ("a", "BF16", &[2], &bf16_data(2, 4)),
                    ("b", "BF16", &[2], &bf16_data(2, 12)),
                ]),
            );
            let error = SafetensorsSnapshot::open(&dir, &[]).unwrap_err();
            assert_eq!(
                error.kind,
                SourceIngestKind::DuplicateTensor {
                    tensor: "a".to_owned(),
                    first_shard: SHARD_1.to_owned(),
                    second_shard: SHARD_2.to_owned()
                }
            );
        }

        #[test]
        fn duplicate_index_mapping_is_detected() {
            let dir = temp_snapshot_dir("dup-index");
            // Hand-written raw JSON: the same key appears twice in
            // weight_map; a plain serde_json map would silently drop one.
            let index = format!("{{\"weight_map\":{{\"a\":\"{SHARD_1}\",\"a\":\"{SHARD_1}\"}}}}");
            write(&dir, "model.safetensors.index.json", index.as_bytes());
            write(
                &dir,
                SHARD_1,
                &shard_bytes(&[("a", "BF16", &[2], &bf16_data(2, 5))]),
            );
            let error = SafetensorsSnapshot::open(&dir, &[]).unwrap_err();
            assert_eq!(
                error.kind,
                SourceIngestKind::DuplicateTensor {
                    tensor: "a".to_owned(),
                    first_shard: SHARD_1.to_owned(),
                    second_shard: SHARD_1.to_owned()
                }
            );
        }

        #[test]
        fn unexpected_tensor_outside_index_is_detected() {
            let dir = temp_snapshot_dir("unexpected");
            write(
                &dir,
                "model.safetensors.index.json",
                &index_bytes(&[("a", SHARD_1)]),
            );
            write(
                &dir,
                SHARD_1,
                &shard_bytes(&[
                    ("a", "BF16", &[2], &bf16_data(2, 6)),
                    ("b", "BF16", &[2], &bf16_data(2, 7)),
                ]),
            );
            let error = SafetensorsSnapshot::open(&dir, &[]).unwrap_err();
            assert_eq!(
                error.kind,
                SourceIngestKind::UnexpectedTensor {
                    tensor: "b".to_owned(),
                    shard: SHARD_1.to_owned()
                }
            );
        }

        #[test]
        fn shape_mismatch_against_geometry_is_detected() {
            let dir = temp_snapshot_dir("shape");
            write(
                &dir,
                "model.safetensors",
                &shard_bytes(&[("a", "BF16", &[4], &bf16_data(4, 8))]),
            );
            let error = SafetensorsSnapshot::open(&dir, &[req("a", &[2, 2])]).unwrap_err();
            assert_eq!(
                error.kind,
                SourceIngestKind::ShapeMismatch {
                    tensor: "a".to_owned(),
                    expected: vec![2, 2],
                    actual: vec![4]
                }
            );
        }

        #[test]
        fn tensor_span_shorter_than_shape_dtype_size_is_detected() {
            let dir = temp_snapshot_dir("span");
            // Shape [2,2] BF16 claims 8 bytes but the span declares 6.
            let header = "{\"a\":{\"dtype\":\"BF16\",\"shape\":[2,2],\"data_offsets\":[0,6]}}";
            write(&dir, "model.safetensors", &raw_shard(header, &[0u8; 6]));
            let error = SafetensorsSnapshot::open(&dir, &[]).unwrap_err();
            match error.kind {
                SourceIngestKind::ByteLengthMismatch {
                    context,
                    expected: 8,
                    actual: 6,
                } => assert!(context.contains("shape×dtype"), "{context}"),
                other => panic!("expected tensor-span ByteLengthMismatch, got {other:?}"),
            }
        }

        #[test]
        fn shard_file_size_disagreeing_with_header_is_detected() {
            let dir = temp_snapshot_dir("filesize");
            let mut shard = shard_bytes(&[("a", "BF16", &[2], &bf16_data(2, 9))]);
            shard.extend_from_slice(&[0u8; 4]); // trailing bytes the header never claimed
            write(&dir, "model.safetensors", &shard);
            let error = SafetensorsSnapshot::open(&dir, &[]).unwrap_err();
            match error.kind {
                SourceIngestKind::ByteLengthMismatch { context, .. } => {
                    assert!(context.contains("file size"), "{context}");
                }
                other => panic!("expected file-size ByteLengthMismatch, got {other:?}"),
            }
        }

        #[test]
        fn non_contiguous_data_offsets_are_detected() {
            let dir = temp_snapshot_dir("gap");
            let header = "{\"a\":{\"dtype\":\"BF16\",\"shape\":[2],\"data_offsets\":[4,8]}}";
            write(&dir, "model.safetensors", &raw_shard(header, &[0u8; 8]));
            let error = SafetensorsSnapshot::open(&dir, &[]).unwrap_err();
            match error.kind {
                SourceIngestKind::ByteLengthMismatch { context, .. } => {
                    assert!(context.contains("contiguous"), "{context}");
                }
                other => panic!("expected contiguity ByteLengthMismatch, got {other:?}"),
            }
        }

        #[test]
        fn dtype_inconsistency_across_shards_is_detected() {
            let dir = temp_snapshot_dir("dtype-mix");
            write(
                &dir,
                "model.safetensors.index.json",
                &index_bytes(&[("a", SHARD_1), ("b", SHARD_2)]),
            );
            write(
                &dir,
                SHARD_1,
                &shard_bytes(&[("a", "BF16", &[2], &bf16_data(2, 10))]),
            );
            write(
                &dir,
                SHARD_2,
                &shard_bytes(&[
                    ("a", "F16", &[2], &[0u8; 4]),
                    ("b", "BF16", &[2], &bf16_data(2, 13)),
                ]),
            );
            let error = SafetensorsSnapshot::open(&dir, &[]).unwrap_err();
            match error.kind {
                SourceIngestKind::DtypeInconsistency {
                    tensor,
                    first,
                    second,
                } => {
                    assert_eq!(tensor, "a");
                    assert!(first.contains("BF16"), "{first}");
                    assert!(second.contains("F16"), "{second}");
                }
                other => panic!("expected DtypeInconsistency, got {other:?}"),
            }
        }

        #[test]
        fn quantized_and_unknown_dtypes_are_rejected_by_name() {
            for dtype in ["I8", "U8", "Q4_GPTQ"] {
                let dir = temp_snapshot_dir("unsupported");
                let header = format!(
                    "{{\"a\":{{\"dtype\":\"{dtype}\",\"shape\":[4],\"data_offsets\":[0,4]}}}}"
                );
                write(&dir, "model.safetensors", &raw_shard(&header, &[0u8; 4]));
                let error = SafetensorsSnapshot::open(&dir, &[]).unwrap_err();
                assert_eq!(
                    error.kind,
                    SourceIngestKind::UnsupportedDtype {
                        tensor: "a".to_owned(),
                        dtype: dtype.to_owned()
                    }
                );
                assert!(error.to_string().contains(dtype));
            }
        }

        #[test]
        fn single_file_snapshot_keeps_its_kappa_and_widens_exactly() {
            let dir = temp_snapshot_dir("single");
            let shard = shard_bytes(&[(
                "a",
                "BF16",
                &[2],
                &[0x80, 0x3F, 0x00, 0xC0], // 1.0, -2.0
            )]);
            write(&dir, "model.safetensors", &shard);
            let snapshot = SafetensorsSnapshot::open(&dir, &[req("a", &[2])]).expect("open");
            // The single-file κ is the pre-#598 hash of model.safetensors.
            assert_eq!(
                snapshot.kappa(),
                format!("blake3:{}", blake3::hash(&shard).to_hex())
            );
            assert_eq!(snapshot.source_bytes(), shard.len());
            let mut out = Vec::new();
            snapshot.tensor_f32_into("a", &mut out).expect("tensor");
            assert_eq!(out, [1.0, -2.0]);
        }

        #[test]
        fn extra_tensor_without_index_is_tolerated() {
            let dir = temp_snapshot_dir("extra");
            write(
                &dir,
                "model.safetensors",
                &shard_bytes(&[
                    ("a", "BF16", &[2], &bf16_data(2, 11)),
                    ("rotary.inv_freq", "F32", &[1], &1.0f32.to_le_bytes()),
                ]),
            );
            // No index: extra tensors are ignored exactly as before #598.
            SafetensorsSnapshot::open(&dir, &[req("a", &[2])]).expect("open single-file");
        }

        /// The tiny synthetic Llama snapshot used by the round-trip test:
        /// mixed BF16/F16/F32 tensors matching `tiny_config_json`.
        fn tiny_model_tensors() -> Vec<(String, &'static str, Vec<usize>, Vec<u8>)> {
            let f16_data = |n: usize| -> Vec<u8> {
                (0..n)
                    .flat_map(|i| (0x3400u16 | ((i as u16).wrapping_mul(17) & 0xFF)).to_le_bytes())
                    .collect()
            };
            let f32_data = |n: usize| -> Vec<u8> {
                (0..n)
                    .flat_map(|i| (i as f32 * 0.031_25 - 1.0).to_le_bytes())
                    .collect()
            };
            let mut tensors = Vec::new();
            let mut push = |name: &str, dtype: &'static str, shape: &[usize], data: Vec<u8>| {
                tensors.push((name.to_owned(), dtype, shape.to_vec(), data));
            };
            push(
                "model.embed_tokens.weight",
                "BF16",
                &[10, 8],
                bf16_data(80, 20),
            );
            push(
                "model.layers.0.input_layernorm.weight",
                "F16",
                &[8],
                f16_data(8),
            );
            push(
                "model.layers.0.self_attn.q_proj.weight",
                "BF16",
                &[8, 8],
                bf16_data(64, 21),
            );
            push(
                "model.layers.0.self_attn.k_proj.weight",
                "BF16",
                &[8, 8],
                bf16_data(64, 22),
            );
            push(
                "model.layers.0.self_attn.v_proj.weight",
                "BF16",
                &[8, 8],
                bf16_data(64, 23),
            );
            push(
                "model.layers.0.self_attn.o_proj.weight",
                "BF16",
                &[8, 8],
                bf16_data(64, 24),
            );
            push(
                "model.layers.0.post_attention_layernorm.weight",
                "BF16",
                &[8],
                bf16_data(8, 25),
            );
            push(
                "model.layers.0.mlp.gate_proj.weight",
                "BF16",
                &[16, 8],
                bf16_data(128, 26),
            );
            push(
                "model.layers.0.mlp.down_proj.weight",
                "BF16",
                &[8, 16],
                bf16_data(128, 27),
            );
            push(
                "model.layers.0.mlp.up_proj.weight",
                "BF16",
                &[16, 8],
                bf16_data(128, 28),
            );
            push("model.norm.weight", "F32", &[8], f32_data(8));
            tensors
        }

        const TINY_CONFIG: &str = r#"{
            "hidden_size": 8,
            "intermediate_size": 16,
            "num_hidden_layers": 1,
            "num_attention_heads": 2,
            "num_key_value_heads": 2,
            "vocab_size": 10,
            "max_position_embeddings": 8,
            "tie_word_embeddings": true
        }"#;

        fn as_entries<'a>(
            tensors: &'a [(String, &'static str, Vec<usize>, Vec<u8>)],
        ) -> Vec<(&'a str, &'a str, &'a [usize], &'a [u8])> {
            tensors
                .iter()
                .map(|(name, dtype, shape, data)| {
                    (name.as_str(), *dtype, shape.as_slice(), data.as_slice())
                })
                .collect()
        }

        /// The #598 round trip: a valid 2-shard indexed snapshot loads to
        /// the same flattened f32 weights — and the same step logits — as
        /// the equivalent single-file snapshot, through one code path.
        #[test]
        fn two_shard_index_round_trip_equals_single_file_load() {
            let tensors = tiny_model_tensors();

            let single = temp_snapshot_dir("roundtrip-single");
            write(&single, "config.json", TINY_CONFIG.as_bytes());
            write(
                &single,
                "model.safetensors",
                &shard_bytes(&as_entries(&tensors)),
            );

            let sharded = temp_snapshot_dir("roundtrip-sharded");
            write(&sharded, "config.json", TINY_CONFIG.as_bytes());
            let (first, second) = tensors.split_at(5);
            write(&sharded, SHARD_1, &shard_bytes(&as_entries(first)));
            write(&sharded, SHARD_2, &shard_bytes(&as_entries(second)));
            let index: Vec<(&str, &str)> = first
                .iter()
                .map(|(name, ..)| (name.as_str(), SHARD_1))
                .chain(second.iter().map(|(name, ..)| (name.as_str(), SHARD_2)))
                .collect();
            write(
                &sharded,
                "model.safetensors.index.json",
                &index_bytes(&index),
            );

            let mut from_single =
                HuggingFaceLlamaOracle::load(&single).expect("load single-file snapshot");
            let mut from_shards =
                HuggingFaceLlamaOracle::load(&sharded).expect("load sharded snapshot");
            assert_eq!(
                from_single.model.w, from_shards.model.w,
                "flattened f32 weights must be identical"
            );

            use crate::BehaviorSource;
            let vocab = from_single.model.cfg.vocab;
            let mut single_logits = vec![0.0f32; vocab];
            let mut sharded_logits = vec![0.0f32; vocab];
            from_single.reset();
            from_shards.reset();
            from_single.step(1, 0, &mut single_logits);
            from_shards.step(1, 0, &mut sharded_logits);
            assert!(single_logits.iter().all(|logit| logit.is_finite()));
            assert_eq!(single_logits, sharded_logits);
        }
    }
}
