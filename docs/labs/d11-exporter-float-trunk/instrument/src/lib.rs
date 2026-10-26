//! A GPU evaluator of a `UORLUT01` SpiralCore/FBS stack artifact
//! (d11-exporter-float-trunk, claude-128).
//!
//! `src/fbs_core.h` is an integer-only port of the Rust session's FBS step,
//! compiled by nvcc into the evaluator (`cuda` feature) and by the host C++
//! compiler into a twin that only qualifies the port against
//! [`IntegerStackSession::step_native`](uor_r4_integer::stack::IntegerStackSession::step_native).
//! Both read the artifact through `uor-r4-integer`'s [`FbsImage`]; there is no
//! second parser.

use uor_r4_integer::h4_classifier::H4_ROOT_COEFFICIENTS;
use uor_r4_integer::stack::{FbsImage, FbsMapImage, IntegerStackModel};

/// `fbs_map` of the core.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CMap {
    pub atoms: *const u16,
    pub codes: *const u8,
    pub blocks: u32,
}

/// `fbs_layer` of the core.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CLayer {
    pub rec_gain: *const i16,
    pub mlp_gain: *const i16,
    pub rec_in: CMap,
    pub rec_gate: CMap,
    pub rec_out: CMap,
    pub mlp_gate: CMap,
    pub mlp_up: CMap,
    pub mlp_down: CMap,
    pub taps: *const i16,
    pub conv_bias: *const i32,
    pub gate_bias: *const i32,
    pub decay_offset: *const i32,
}

/// `fbs_model` of the core.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CModel {
    pub vocab: u32,
    pub width: u32,
    pub mlp: u32,
    pub context: u32,
    pub kept: u32,
    pub layer_count: u32,
    pub eps_mantissa: i64,
    pub eps_exp: i32,
    pub layers: *const CLayer,
    pub final_gain: *const i16,
    pub writer_table: *const i16,
    pub roots_q12: *const i32,
    pub icosian: *const i8,
    pub keep_q31: *const u64,
}

static KEEP_Q31: [u64; 16] = FbsImage::GEOMETRIC_KEEP_Q31;
static ICOSIAN: [[[i8; 2]; 4]; 120] = H4_ROOT_COEFFICIENTS;

#[cfg(feature = "cuda")]
pub mod device;

extern "C" {
    fn fbs_host_sizeof_model() -> i64;
    fn fbs_host_sizeof_layer() -> i64;
    fn fbs_host_work_bytes(m: *const CModel) -> u64;
    fn fbs_host_sequences(
        m: *const CModel,
        tokens: *const u32,
        sequences: u32,
        len: u32,
        out_tokens: *mut u32,
        out_kept: *mut u32,
        out_query: *mut i32,
    ) -> i32;
    fn fbs_host_trace_stride(m: *const CModel) -> u64;
    fn fbs_host_trace_sequences(
        m: *const CModel,
        tokens: *const u32,
        sequences: u32,
        len: u32,
        out_tokens: *mut u32,
        out_kept: *mut u32,
        out_query: *mut i32,
        out_trace: *mut u8,
    ) -> i32;
}

/// The evaluator's per-position output for `sequences` sequences of `len`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outputs {
    pub sequences: usize,
    pub len: usize,
    pub kept: usize,
    pub width: usize,
    /// `sequences * len` chosen tokens.
    pub tokens: Vec<u32>,
    /// `sequences * len * kept` kept cells, in selection order.
    pub kept_cells: Vec<u32>,
    /// `sequences * len * width` Q12 queries.
    pub queries: Vec<i32>,
}

impl Outputs {
    fn zeroed(sequences: usize, len: usize, kept: usize, width: usize) -> Self {
        let n = sequences * len;
        Self {
            sequences,
            len,
            kept,
            width,
            tokens: vec![0; n],
            kept_cells: vec![0; n * kept],
            queries: vec![0; n * width],
        }
    }
}

/// The core's view of a parsed FBS artifact. The pointers borrow `model`;
/// [`CModelHost::model`] is valid while both live.
pub struct CModelHost<'a> {
    _image: FbsImage<'a>,
    layers: Vec<CLayer>,
    model: CModel,
}

fn cmap(map: &FbsMapImage<'_>) -> CMap {
    CMap {
        atoms: map.atoms.as_ptr(),
        codes: map.codes.as_ptr().cast::<u8>(),
        blocks: map.atoms.len() as u32,
    }
}

impl<'a> CModelHost<'a> {
    /// The core's model over `model`'s FBS image; an error for a dense stack.
    pub fn new(model: &'a IntegerStackModel) -> Result<Self, String> {
        let image = model
            .fbs_image()
            .ok_or("the artifact is not a SpiralCore/FBS stack")?;
        let shape = model.shape();
        let eps = model.rms_epsilon();
        let layers: Vec<CLayer> = image
            .layers
            .iter()
            .map(|l| CLayer {
                rec_gain: l.rec_gain.as_ptr(),
                mlp_gain: l.mlp_gain.as_ptr(),
                rec_in: cmap(&l.rec_in),
                rec_gate: cmap(&l.rec_gate),
                rec_out: cmap(&l.rec_out),
                mlp_gate: cmap(&l.mlp_gate),
                mlp_up: cmap(&l.mlp_up),
                mlp_down: cmap(&l.mlp_down),
                taps: l.taps.as_ptr(),
                conv_bias: l.conv_bias.as_ptr(),
                gate_bias: l.gate_bias.as_ptr(),
                decay_offset: l.decay_offset.as_ptr(),
            })
            .collect();
        let model = CModel {
            vocab: shape.vocab as u32,
            width: shape.width as u32,
            mlp: shape.mlp as u32,
            context: shape.context as u32,
            kept: image.kept_cells as u32,
            layer_count: layers.len() as u32,
            eps_mantissa: eps.mantissa,
            eps_exp: eps.exp,
            layers: layers.as_ptr(),
            final_gain: image.final_gain.as_ptr(),
            writer_table: image.writer_table.as_ptr(),
            roots_q12: image.roots_q12.as_ptr().cast::<i32>(),
            icosian: ICOSIAN.as_ptr().cast::<i8>(),
            keep_q31: KEEP_Q31.as_ptr(),
        };
        Ok(Self {
            _image: image,
            layers,
            model,
        })
    }

    pub fn model(&self) -> &CModel {
        &self.model
    }

    pub fn layers(&self) -> &[CLayer] {
        &self.layers
    }
}

/// The bytes of one sequence's work in the core's layout (`fbs_work_bytes`),
/// read from `m`'s scalars only.
pub fn work_bytes(m: &CModel) -> u64 {
    // SAFETY: `fbs_work_bytes` reads only `m`'s integer fields.
    unsafe { fbs_host_work_bytes(m) }
}

/// The host trace's prefill (claude-134): every byte the core writes is a root below 120 or 255
/// (a zero raw), so a byte left at 0xFE is a slot the core never wrote.
pub const TRACE_UNWRITTEN: u8 = 0xFE;

/// The bytes of one position's snap trace (`fbs_trace_stride`, claude-134):
/// per layer, the transition's icosian root per lane, the output gate's root
/// per lane, then the feed-forward gate's root per 4 MLP channels; 255 marks a
/// zero raw, which selects root 0 on the tie.
pub fn trace_stride(m: &CModel) -> u64 {
    // SAFETY: `fbs_trace_stride` reads only `m`'s integer fields.
    unsafe { fbs_host_trace_stride(m) }
}

/// SHA-256 of `bytes`, hex.
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Whether the Rust and C layouts of the core's structs agree.
pub fn layouts_agree() -> bool {
    // SAFETY: the two functions only return `sizeof` constants.
    let (model, layer) = unsafe { (fbs_host_sizeof_model(), fbs_host_sizeof_layer()) };
    model == std::mem::size_of::<CModel>() as i64 && layer == std::mem::size_of::<CLayer>() as i64
}

/// The host twin over `sequences` (each of the same length, from a fresh
/// session). Qualification only: it never scores a panel.
pub fn host_sequences(
    model: &IntegerStackModel,
    sequences: &[Vec<u32>],
) -> Result<Outputs, String> {
    if !layouts_agree() {
        return Err("the Rust and C layouts of the FBS core disagree".to_owned());
    }
    let host = CModelHost::new(model)?;
    let len = sequences.first().map_or(0, Vec::len);
    if sequences.iter().any(|s| s.len() != len) {
        return Err("every sequence must have the same length".to_owned());
    }
    let m = host.model();
    let flat: Vec<u32> = sequences.iter().flatten().copied().collect();
    let mut out = Outputs::zeroed(sequences.len(), len, m.kept as usize, m.width as usize);
    // SAFETY: `m` points into `model`'s image and `host`'s layers, both alive
    // for the call; every output buffer has the length the core writes.
    let err = unsafe {
        fbs_host_sequences(
            m,
            flat.as_ptr(),
            sequences.len() as u32,
            len as u32,
            out.tokens.as_mut_ptr(),
            out.kept_cells.as_mut_ptr(),
            out.queries.as_mut_ptr(),
        )
    };
    match err {
        0 => Ok(out),
        1 => Err("a token lies outside the vocabulary".to_owned()),
        2 => Err("a sequence is longer than the context".to_owned()),
        e => Err(format!("the FBS core failed with code {e}")),
    }
}

/// [`host_sequences`] with every position's snap trace, [`trace_stride`] bytes
/// each (claude-134). Qualification only, like the host twin itself.
pub fn host_trace_sequences(
    model: &IntegerStackModel,
    sequences: &[Vec<u32>],
) -> Result<(Outputs, Vec<u8>), String> {
    if !layouts_agree() {
        return Err("the Rust and C layouts of the FBS core disagree".to_owned());
    }
    let host = CModelHost::new(model)?;
    let len = sequences.first().map_or(0, Vec::len);
    if sequences.iter().any(|s| s.len() != len) {
        return Err("every sequence must have the same length".to_owned());
    }
    let m = host.model();
    let flat: Vec<u32> = sequences.iter().flatten().copied().collect();
    let mut out = Outputs::zeroed(sequences.len(), len, m.kept as usize, m.width as usize);
    // Prefilled with a byte the core never writes, so a slot it misses cannot pass for root 0
    // (and differs from the GPU's zeroed buffer, failing G = H).
    let mut trace = vec![TRACE_UNWRITTEN; sequences.len() * len * trace_stride(m) as usize];
    // SAFETY: as in `host_sequences`; `trace` has `trace_stride` bytes per position.
    let err = unsafe {
        fbs_host_trace_sequences(
            m,
            flat.as_ptr(),
            sequences.len() as u32,
            len as u32,
            out.tokens.as_mut_ptr(),
            out.kept_cells.as_mut_ptr(),
            out.queries.as_mut_ptr(),
            trace.as_mut_ptr(),
        )
    };
    match err {
        0 => Ok((out, trace)),
        1 => Err("a token lies outside the vocabulary".to_owned()),
        2 => Err("a sequence is longer than the context".to_owned()),
        e => Err(format!("the FBS core failed with code {e}")),
    }
}

/// The Rust integer session's outputs on the same sequences: the reference
/// the evaluator must equal bit for bit.
pub fn session_sequences(
    model: &IntegerStackModel,
    sequences: &[Vec<u32>],
) -> Result<Outputs, String> {
    let shape = model.shape();
    let kept = model.fbs_image().ok_or("not an FBS stack")?.kept_cells;
    let len = sequences.first().map_or(0, Vec::len);
    if sequences.iter().any(|s| s.len() != len) {
        return Err("every sequence must have the same length".to_owned());
    }
    let mut out = Outputs::zeroed(sequences.len(), len, kept, shape.width);
    for (s, seq) in sequences.iter().enumerate() {
        let mut session = model.session();
        for (t, &token) in seq.iter().enumerate() {
            let step = session.step_native(token).map_err(|e| e.to_string())?;
            let at = s * len + t;
            out.tokens[at] = step.token;
            out.kept_cells[at * kept..(at + 1) * kept].copy_from_slice(&step.kept_cells);
            out.queries[at * shape.width..(at + 1) * shape.width].copy_from_slice(&step.query_q12);
        }
    }
    Ok(out)
}

/// The first position where two outputs differ, as `(sequence, position)`;
/// `(0, 0)` when their dimensions differ.
pub fn first_difference(a: &Outputs, b: &Outputs) -> Option<(usize, usize)> {
    if (a.sequences, a.len, a.kept, a.width) != (b.sequences, b.len, b.kept, b.width) {
        return Some((0, 0));
    }
    let n = a.sequences * a.len;
    (0..n)
        .find(|&i| {
            a.tokens[i] != b.tokens[i]
                || a.kept_cells[i * a.kept..(i + 1) * a.kept]
                    != b.kept_cells[i * b.kept..(i + 1) * b.kept]
                || a.queries[i * a.width..(i + 1) * a.width]
                    != b.queries[i * b.width..(i + 1) * b.width]
        })
        .map(|i| (i / a.len.max(1), i % a.len.max(1)))
}
