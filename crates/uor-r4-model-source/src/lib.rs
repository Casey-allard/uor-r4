//! The TEACHER: faithful Rust port of karpathy run.c forward pass (v0 checkpoint).
//! Arithmetic order mirrors the C exactly: sequential adds in matmul rows,
//! rmsnorm/softmax/RoPE/SwiGLU op-for-op, libm via glibc on gnu targets.
//! The Safetensors adapter also loads pinned Hugging Face SmolLM2 weights
//! into this same source-only teacher surface. The pinned legacy teacher keeps
//! the original reduction order; native Hugging Face compilation may use an
//! optimized CPU matrix-vector backend.

pub mod progress;
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
fn sqrtf(value: f32, canonical: bool) -> f32 {
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

fn softmax_with_mode(x: &mut [f32], canonical: bool) {
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

/// W (d,n) @ x (n,) -> xout (d,). The exact path preserves the original C
/// reduction order for certificate reproduction. Hugging Face compilation
/// may select the optimized CPU path because those source logits are teacher
/// data rather than part of the pinned legacy proof.
///
/// Note: a row-parallel (rayon fork-join) variant of this exact path was
/// measured 2026-07-31 and REVERTED — per-matmul fork-join costs ~1.5 ms of
/// worker-scheduling latency on a load-average->100 development machine,
/// ~10× slower end-to-end than the serial loop below (recorded negative
/// result, issue #310). Parallelism in corpus generation must be
/// story-level (a new corpus era) or not at all.
fn matmul(xout: &mut [f32], x: &[f32], w: &[f32], n: usize, fast: bool) {
    if fast {
        return matmul_fast(xout, x, w, n);
    }
    let d = xout.len();
    // Four rows in flight hide FP add latency while retaining each row's
    // strictly sequential accumulation chain and therefore its exact bits.
    let mut i = 0usize;
    while i + 4 <= d {
        let r0 = &w[i * n..i * n + n];
        let r1 = &w[(i + 1) * n..(i + 1) * n + n];
        let r2 = &w[(i + 2) * n..(i + 2) * n + n];
        let r3 = &w[(i + 3) * n..(i + 3) * n + n];
        let (mut v0, mut v1, mut v2, mut v3) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        for j in 0..n {
            let xj = x[j];
            v0 += r0[j] * xj;
            v1 += r1[j] * xj;
            v2 += r2[j] * xj;
            v3 += r3[j] * xj;
        }
        xout[i] = v0;
        xout[i + 1] = v1;
        xout[i + 2] = v2;
        xout[i + 3] = v3;
        i += 4;
    }
    while i < d {
        let mut value = 0.0f32;
        let row = &w[i * n..i * n + n];
        for j in 0..n {
            value += row[j] * x[j];
        }
        xout[i] = value;
        i += 1;
    }
}

#[cfg(target_os = "macos")]
fn matmul_fast(xout: &mut [f32], x: &[f32], w: &[f32], n: usize) {
    const CBLAS_ROW_MAJOR: i32 = 101;
    const CBLAS_NO_TRANSPOSE: i32 = 111;
    debug_assert!(w.len() >= xout.len() * n);
    // SAFETY: all pointers refer to initialized, non-overlapping f32 slices;
    // their dimensions and strides describe W[xout.len(), n] and x[n].
    unsafe {
        cblas_sgemv(
            CBLAS_ROW_MAJOR,
            CBLAS_NO_TRANSPOSE,
            i32::try_from(xout.len()).expect("teacher output dimension exceeds CBLAS i32"),
            i32::try_from(n).expect("teacher input dimension exceeds CBLAS i32"),
            1.0,
            w.as_ptr(),
            i32::try_from(n).expect("teacher stride exceeds CBLAS i32"),
            x.as_ptr(),
            1,
            0.0,
            xout.as_mut_ptr(),
            1,
        );
    }
}

#[cfg(target_os = "macos")]
#[link(name = "Accelerate", kind = "framework")]
unsafe extern "C" {
    fn cblas_sgemv(
        order: i32,
        transpose: i32,
        rows: i32,
        columns: i32,
        alpha: f32,
        matrix: *const f32,
        leading_dimension: i32,
        vector: *const f32,
        vector_stride: i32,
        beta: f32,
        output: *mut f32,
        output_stride: i32,
    );

    fn cblas_sgemm(
        order: i32,
        transpose_a: i32,
        transpose_b: i32,
        m: i32,
        n: i32,
        k: i32,
        alpha: f32,
        a: *const f32,
        lda: i32,
        b: *const f32,
        ldb: i32,
        beta: f32,
        c: *mut f32,
        ldc: i32,
    );
}

#[cfg(not(target_os = "macos"))]
fn matmul_fast(xout: &mut [f32], x: &[f32], w: &[f32], n: usize) {
    debug_assert_eq!(x.len(), n);
    debug_assert!(w.len() >= xout.len() * n);
    for (output, row) in xout.iter_mut().zip(w.chunks_exact(n)) {
        *output = dot_fast(row, x);
    }
}

/// Batched matmul: `batch` input vectors of length `n` through weight
/// `W[rows, n]` → `batch` output vectors of length `rows`. `x` is the `batch`
/// input vectors laid out contiguously (`batch * n`); `xout` is the `batch`
/// output vectors (`batch * rows`), same sequence-major layout.
///
/// This is the memory-amortized core of batched teacher inference: the serial
/// [`matmul`] re-reads the entire weight `W` for every single token, which
/// makes autoregressive inference of a large teacher memory-bandwidth bound.
/// Here each weight is read once and reused across all `batch` inputs, so `B`
/// tokens cost one weight sweep instead of `B` — turning the per-token
/// matrix-vector products into one matrix-matrix product.
///
/// Off macOS this reuses [`dot_fast`] per (row, input) with each weight row
/// held hot across the batch, so it is bit-identical to `batch` separate
/// [`matmul`]`(.., fast = true)` calls. On macOS it dispatches to Accelerate's
/// `cblas_sgemm`; like the serial `cblas_sgemv` fast path it is teacher data,
/// not part of the pinned legacy proof, and is deterministic per machine.
#[cfg(not(target_os = "macos"))]
fn matmul_batched(xout: &mut [f32], x: &[f32], w: &[f32], n: usize, batch: usize) {
    debug_assert!(batch > 0);
    debug_assert_eq!(xout.len() % batch, 0);
    let rows = xout.len() / batch;
    debug_assert!(w.len() >= rows * n);
    debug_assert_eq!(x.len(), batch * n);
    for row in 0..rows {
        let wr = &w[row * n..row * n + n];
        for bi in 0..batch {
            let xi = &x[bi * n..bi * n + n];
            xout[bi * rows + row] = dot_fast(wr, xi);
        }
    }
}

#[cfg(target_os = "macos")]
fn matmul_batched(xout: &mut [f32], x: &[f32], w: &[f32], n: usize, batch: usize) {
    debug_assert!(batch > 0);
    debug_assert_eq!(xout.len() % batch, 0);
    let rows = xout.len() / batch;
    debug_assert!(w.len() >= rows * n);
    debug_assert_eq!(x.len(), batch * n);
    const CBLAS_ROW_MAJOR: i32 = 101;
    const CBLAS_NO_TRANSPOSE: i32 = 111;
    const CBLAS_TRANSPOSE: i32 = 112;
    // C[batch, rows] = X[batch, n] · W[rows, n]^T
    // SAFETY: all pointers refer to initialized, non-overlapping f32 slices
    // whose dimensions/strides describe X[batch, n], W[rows, n], C[batch, rows].
    unsafe {
        cblas_sgemm(
            CBLAS_ROW_MAJOR,
            CBLAS_NO_TRANSPOSE,
            CBLAS_TRANSPOSE,
            i32::try_from(batch).expect("teacher batch exceeds CBLAS i32"),
            i32::try_from(rows).expect("teacher output dimension exceeds CBLAS i32"),
            i32::try_from(n).expect("teacher input dimension exceeds CBLAS i32"),
            1.0,
            x.as_ptr(),
            i32::try_from(n).expect("teacher input stride exceeds CBLAS i32"),
            w.as_ptr(),
            i32::try_from(n).expect("teacher weight stride exceeds CBLAS i32"),
            0.0,
            xout.as_mut_ptr(),
            i32::try_from(rows).expect("teacher output stride exceeds CBLAS i32"),
        );
    }
}

#[cfg(all(not(target_os = "macos"), target_arch = "aarch64"))]
fn dot_fast(weights: &[f32], values: &[f32]) -> f32 {
    // NEON is part of the AArch64 architecture baseline.
    // SAFETY: the helper only performs unaligned loads within these equal-size
    // slices, and its required target feature is always present on AArch64.
    unsafe { dot_neon(weights, values) }
}

#[cfg(all(target_arch = "aarch64", any(not(target_os = "macos"), test)))]
#[target_feature(enable = "neon")]
unsafe fn dot_neon(weights: &[f32], values: &[f32]) -> f32 {
    use core::arch::aarch64::{vaddq_f32, vaddvq_f32, vdupq_n_f32, vfmaq_f32, vld1q_f32};

    debug_assert_eq!(weights.len(), values.len());
    let mut sums = [vdupq_n_f32(0.0); 4];
    let mut index = 0usize;
    while index + 16 <= weights.len() {
        for (lane, sum) in sums.iter_mut().enumerate() {
            let offset = index + lane * 4;
            // SAFETY: the loop condition guarantees four readable values from
            // each pointer, and NEON's vld1q instruction permits unaligned data.
            let (weight, value) = unsafe {
                (
                    vld1q_f32(weights.as_ptr().add(offset)),
                    vld1q_f32(values.as_ptr().add(offset)),
                )
            };
            *sum = vfmaq_f32(*sum, weight, value);
        }
        index += 16;
    }
    let combined = vaddq_f32(vaddq_f32(sums[0], sums[1]), vaddq_f32(sums[2], sums[3]));
    let mut result = vaddvq_f32(combined);
    for (&weight, &value) in weights[index..].iter().zip(&values[index..]) {
        result += weight * value;
    }
    result
}

#[cfg(all(not(target_os = "macos"), target_arch = "x86_64"))]
fn dot_fast(weights: &[f32], values: &[f32]) -> f32 {
    if std::arch::is_x86_feature_detected!("avx2") && std::arch::is_x86_feature_detected!("fma") {
        // SAFETY: runtime detection above establishes both target features;
        // the helper bounds every unaligned load to the input slices.
        unsafe { dot_avx2_fma(weights, values) }
    } else {
        dot_portable(weights, values)
    }
}

#[cfg(all(not(target_os = "macos"), target_arch = "x86_64"))]
#[target_feature(enable = "avx2,fma")]
unsafe fn dot_avx2_fma(weights: &[f32], values: &[f32]) -> f32 {
    use core::arch::x86_64::{
        _mm256_add_ps, _mm256_fmadd_ps, _mm256_loadu_ps, _mm256_setzero_ps, _mm256_storeu_ps,
    };

    debug_assert_eq!(weights.len(), values.len());
    let mut sums = [_mm256_setzero_ps(); 4];
    let mut index = 0usize;
    while index + 32 <= weights.len() {
        for (lane, sum) in sums.iter_mut().enumerate() {
            let offset = index + lane * 8;
            // SAFETY: the loop condition guarantees eight readable values
            // from each pointer; loadu explicitly supports unaligned data.
            let (weight, value) = unsafe {
                (
                    _mm256_loadu_ps(weights.as_ptr().add(offset)),
                    _mm256_loadu_ps(values.as_ptr().add(offset)),
                )
            };
            *sum = _mm256_fmadd_ps(weight, value, *sum);
        }
        index += 32;
    }
    let combined = _mm256_add_ps(
        _mm256_add_ps(sums[0], sums[1]),
        _mm256_add_ps(sums[2], sums[3]),
    );
    let mut lanes = [0.0f32; 8];
    // SAFETY: `lanes` has room for all eight values written by the intrinsic.
    unsafe { _mm256_storeu_ps(lanes.as_mut_ptr(), combined) };
    let mut result = lanes.into_iter().sum::<f32>();
    for (&weight, &value) in weights[index..].iter().zip(&values[index..]) {
        result += weight * value;
    }
    result
}

#[cfg(all(
    not(target_os = "macos"),
    not(any(target_arch = "aarch64", target_arch = "x86_64"))
))]
fn dot_fast(weights: &[f32], values: &[f32]) -> f32 {
    dot_portable(weights, values)
}

#[cfg(not(target_os = "macos"))]
fn dot_portable(weights: &[f32], values: &[f32]) -> f32 {
    debug_assert_eq!(weights.len(), values.len());
    let mut partial = [0.0f32; 8];
    let mut weight_chunks = weights.chunks_exact(8);
    let mut input_chunks = values.chunks_exact(8);
    for (weight_chunk, value_chunk) in weight_chunks.by_ref().zip(input_chunks.by_ref()) {
        for lane in 0..8 {
            partial[lane] += weight_chunk[lane] * value_chunk[lane];
        }
    }
    let mut result = partial.into_iter().sum::<f32>();
    for (&weight, &value) in weight_chunks
        .remainder()
        .iter()
        .zip(input_chunks.remainder())
    {
        result += weight * value;
    }
    result
}

#[allow(dead_code)]
#[cfg(target_os = "macos")]
fn fast_matmul_backend() -> &'static str {
    "Apple Accelerate CPU SIMD"
}

#[allow(dead_code)]
#[cfg(all(not(target_os = "macos"), target_arch = "aarch64"))]
fn fast_matmul_backend() -> &'static str {
    "AArch64 NEON CPU"
}

#[allow(dead_code)]
#[cfg(all(not(target_os = "macos"), target_arch = "x86_64"))]
fn fast_matmul_backend() -> &'static str {
    if std::arch::is_x86_feature_detected!("avx2") && std::arch::is_x86_feature_detected!("fma") {
        "x86-64 AVX2/FMA CPU"
    } else {
        "portable CPU"
    }
}

#[allow(dead_code)]
#[cfg(all(
    not(target_os = "macos"),
    not(any(target_arch = "aarch64", target_arch = "x86_64"))
))]
fn fast_matmul_backend() -> &'static str {
    "portable CPU"
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
        let c = &self.cfg;
        let (dim, hid) = (c.dim, c.hidden);
        let kv_dim = c.dim * c.n_kv_heads / c.n_heads;
        let kv_mul = c.n_heads / c.n_kv_heads;
        let head_size = dim / c.n_heads;
        let w = &self.w;

        st.x.copy_from_slice(&w[self.emb + token * dim..self.emb + (token + 1) * dim]);

        for l in 0..c.n_layers {
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
            // independent of order).
            for h in 0..c.n_heads {
                let q = &st.q[h * head_size..(h + 1) * head_size];
                let att = &mut st.att[h * c.seq_len..h * c.seq_len + pos + 1];

                if c.r4_attention {
                    // Compute R4 4D Spin(4) quaternionic alignment
                    for (t, attention) in att.iter_mut().enumerate() {
                        let k = &st.key_cache[loff + t * kv_dim + (h / kv_mul) * head_size..]
                            [..head_size];

                        let mut head_score = 0.0f32;
                        let chunks = head_size / 4;
                        for chunk_idx in 0..chunks {
                            let q_chunk = &q[chunk_idx * 4..(chunk_idx + 1) * 4];
                            let k_chunk = &k[chunk_idx * 4..(chunk_idx + 1) * 4];
                            let dot_4d = q_chunk[0] * k_chunk[0]
                                + q_chunk[1] * k_chunk[1]
                                + q_chunk[2] * k_chunk[2]
                                + q_chunk[3] * k_chunk[3];
                            head_score += dot_4d;
                        }
                        head_score /= sqrtf(head_size as f32, self.canonical_math);
                        *attention = head_score;
                    }
                    softmax_with_mode(att, self.canonical_math);
                } else {
                    // Standard Llama scaled dot-product attention
                    for (t, attention) in att.iter_mut().enumerate() {
                        let k = &st.key_cache[loff + t * kv_dim + (h / kv_mul) * head_size..]
                            [..head_size];
                        let mut score = 0.0f32;
                        for i in 0..head_size {
                            score += q[i] * k[i];
                        }
                        score /= sqrtf(head_size as f32, self.canonical_math);
                        *attention = score;
                    }
                    softmax_with_mode(att, self.canonical_math);
                }

                let xb = &mut st.xb[h * head_size..(h + 1) * head_size];
                xb.iter_mut().for_each(|v| *v = 0.0);
                for (t, &attention) in att.iter().enumerate() {
                    let v = &st.value_cache[loff + t * kv_dim + (h / kv_mul) * head_size..]
                        [..head_size];
                    let a = attention;
                    for i in 0..head_size {
                        xb[i] += a * v[i];
                    }
                }
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

                for h in 0..c.n_heads {
                    let qh = &qb[h * head_size..(h + 1) * head_size];
                    let att = &mut st.att[h * c.seq_len..h * c.seq_len + pos + 1];
                    if c.r4_attention {
                        for (t, attention) in att.iter_mut().enumerate() {
                            let k = &st.key_cache[loff + t * kv_dim + (h / kv_mul) * head_size..]
                                [..head_size];
                            let mut head_score = 0.0f32;
                            let chunks = head_size / 4;
                            for chunk_idx in 0..chunks {
                                let q_chunk = &qh[chunk_idx * 4..(chunk_idx + 1) * 4];
                                let k_chunk = &k[chunk_idx * 4..(chunk_idx + 1) * 4];
                                head_score += q_chunk[0] * k_chunk[0]
                                    + q_chunk[1] * k_chunk[1]
                                    + q_chunk[2] * k_chunk[2]
                                    + q_chunk[3] * k_chunk[3];
                            }
                            head_score /= sqrtf(head_size as f32, self.canonical_math);
                            *attention = head_score;
                        }
                        softmax_with_mode(att, self.canonical_math);
                    } else {
                        for (t, attention) in att.iter_mut().enumerate() {
                            let k = &st.key_cache[loff + t * kv_dim + (h / kv_mul) * head_size..]
                                [..head_size];
                            let mut score = 0.0f32;
                            for i in 0..head_size {
                                score += qh[i] * k[i];
                            }
                            score /= sqrtf(head_size as f32, self.canonical_math);
                            *attention = score;
                        }
                        softmax_with_mode(att, self.canonical_math);
                    }
                    let outh = &mut out[h * head_size..(h + 1) * head_size];
                    outh.iter_mut().for_each(|v| *v = 0.0);
                    for (t, &attention) in att.iter().enumerate() {
                        let v = &st.value_cache[loff + t * kv_dim + (h / kv_mul) * head_size..]
                            [..head_size];
                        for i in 0..head_size {
                            outh[i] += attention * v[i];
                        }
                    }
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
}

/// Shared top-k trace computation over the raw logits a llama-family
/// `State` retains after `step`. Softmax is computed in the same f32
/// max-subtracted form the corpus generator uses; ordering is canonical
/// (probability descending, token id ascending on ties).
fn top_k_from_logits(
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

/// Offline teacher adapter for Hugging Face Llama-family BF16 Safetensors.
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
/// `model.safetensors`, `config.json`, or a named tensor within — could not be
/// ingested into a valid teacher at construction. This is the host-side
/// analogue of graph-format's `NotAProduct`: the only reportable condition is
/// that the requested object does not exist as a valid product, reported at
/// construction. It carries the operator-facing diagnostic (which file, which
/// tensor, what dtype) rather than discarding it.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct SourceUnavailable {
    /// Human-facing description of why the source could not be ingested.
    pub reason: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl SourceUnavailable {
    /// Construct from any displayable reason.
    pub fn new(reason: impl std::fmt::Display) -> Self {
        Self {
            reason: reason.to_string(),
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

/// A teacher that can advance a batch of independent sequences one position
/// each through a single memory-amortized forward. Implemented by the HF oracle
/// (the deployed teacher) and mockable for tests, so the batched observe driver
/// need not name a concrete oracle.
pub trait BatchedTeacher {
    /// A fresh per-sequence state for this teacher.
    fn new_state(&self) -> State;
    /// Maximum context length (teacher-forced positions per article).
    fn seq_len(&self) -> usize;
    /// Vocabulary size (logits length).
    fn vocab(&self) -> usize;
    /// Advance `states.len()` sequences one position each: sequence `b` steps
    /// `tokens[b]` at `positions[b]`, leaving its logits in `states[b].logits`.
    fn forward_batch_into(&self, states: &mut [State], tokens: &[usize], positions: &[usize]);
}

impl BatchedTeacher for HuggingFaceLlamaOracle {
    fn new_state(&self) -> State {
        State::new(&self.model.cfg)
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

    /// Enable or disable experimental R4 Spin(4) softmax-free attention calculation.
    pub fn set_r4_attention(&mut self, enable: bool) {
        self.model.cfg.r4_attention = enable;
    }

    /// Check if experimental R4 Spin(4) attention is enabled.
    pub fn r4_attention(&self) -> bool {
        self.model.cfg.r4_attention
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_inner(
        source: impl AsRef<std::path::Path>,
        sequence_length: Option<usize>,
    ) -> Result<Self, SourceUnavailable> {
        let source = source.as_ref();
        let config: HuggingFaceConfig =
            serde_json::from_slice(&std::fs::read(source.join("config.json"))?)?;
        let model_bytes =
            crate::progress::read_file(source.join("model.safetensors"), "loading Safetensors")?;
        let tensors = safetensors::SafeTensors::deserialize(&model_bytes)?;
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
        eprintln!("converting BF16 tensors to the compiler teacher layout...");
        let mut weights = Vec::with_capacity(model_bytes.len() / 2);
        append_tensor(&tensors, "model.embed_tokens.weight", &mut weights)?;
        append_layers(
            &tensors,
            cfg.n_layers,
            "input_layernorm.weight",
            &mut weights,
        )?;
        append_layers(
            &tensors,
            cfg.n_layers,
            "self_attn.q_proj.weight",
            &mut weights,
        )?;
        append_layers(
            &tensors,
            cfg.n_layers,
            "self_attn.k_proj.weight",
            &mut weights,
        )?;
        append_layers(
            &tensors,
            cfg.n_layers,
            "self_attn.v_proj.weight",
            &mut weights,
        )?;
        append_layers(
            &tensors,
            cfg.n_layers,
            "self_attn.o_proj.weight",
            &mut weights,
        )?;
        append_layers(
            &tensors,
            cfg.n_layers,
            "post_attention_layernorm.weight",
            &mut weights,
        )?;
        append_layers(&tensors, cfg.n_layers, "mlp.gate_proj.weight", &mut weights)?;
        append_layers(&tensors, cfg.n_layers, "mlp.down_proj.weight", &mut weights)?;
        append_layers(&tensors, cfg.n_layers, "mlp.up_proj.weight", &mut weights)?;
        append_tensor(&tensors, "model.norm.weight", &mut weights)?;
        if !config.tie_word_embeddings {
            append_tensor(&tensors, "lm_head.weight", &mut weights)?;
        }
        let kappa = format!("blake3:{}", blake3::hash(&model_bytes).to_hex());
        let source_bytes = model_bytes.len();
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

#[cfg(not(target_arch = "wasm32"))]
fn append_layers(
    tensors: &safetensors::SafeTensors<'_>,
    layers: usize,
    suffix: &str,
    out: &mut Vec<f32>,
) -> Result<(), SourceUnavailable> {
    for layer in 0..layers {
        append_tensor(tensors, &format!("model.layers.{layer}.{suffix}"), out)?;
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn append_tensor(
    tensors: &safetensors::SafeTensors<'_>,
    name: &str,
    out: &mut Vec<f32>,
) -> Result<(), SourceUnavailable> {
    let tensor = tensors.tensor(name)?;
    if tensor.dtype() != safetensors::Dtype::BF16 {
        return Err(SourceUnavailable::new(format!(
            "tensor {name} is {:?}, expected BF16",
            tensor.dtype()
        )));
    }
    for bytes in tensor.data().chunks_exact(2) {
        let bits = u16::from_le_bytes([bytes[0], bytes[1]]);
        out.push(f32::from_bits(u32::from(bits) << 16));
    }
    Ok(())
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
        288
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
        assert!(
            dim >= out.len(),
            "source dimension is smaller than runtime geometry"
        );
        let output_dimensions = out.len();
        for (index, value) in out.iter_mut().enumerate() {
            let start = index * dim / output_dimensions;
            let end = (index + 1) * dim / output_dimensions;
            let bucket = &row[start..end];
            *value = bucket.iter().sum::<f32>() / bucket.len() as f32;
        }
    }
    fn hidden_state(&self) -> Option<&[f32]> {
        Some(&self.state.x)
    }
    fn top_k(&self, k: usize, out: &mut [(u32, f32)]) -> usize {
        top_k_from_logits(&self.state.logits, k, out, self.model.canonical_math)
    }
}

/// Backward-compatible name for the first supported Hugging Face model.
pub type SmolLm2Oracle = HuggingFaceLlamaOracle;

#[cfg(test)]
mod tests {
    use super::*;

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

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn neon_dot_tracks_scalar_result() {
        const COLUMNS: usize = 73;
        let values: Vec<f32> = (0..COLUMNS)
            .map(|index| ((index * 17 % 31) as f32 - 15.0) / 16.0)
            .collect();
        let weights: Vec<f32> = (0..COLUMNS)
            .map(|index| ((index * 29 % 43) as f32 - 21.0) / 32.0)
            .collect();
        let expected = weights
            .iter()
            .zip(&values)
            .map(|(weight, value)| weight * value)
            .sum::<f32>();
        // SAFETY: NEON is part of the AArch64 baseline and both slices have
        // identical lengths.
        let actual = unsafe { dot_neon(&weights, &values) };
        let tolerance = 1e-5f32.max(expected.abs() * 1e-5);
        assert!((expected - actual).abs() <= tolerance);
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
}
