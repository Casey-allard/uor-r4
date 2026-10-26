//! Synthetic SpiralCore/FBS trunks: the finite controls of the integer FBS
//! session (claude-122, claude-127), shared with out-of-crate evaluators that
//! must reproduce `step_native` on the same trunks (claude-128's GPU
//! evaluator, through the `fbs-synthetic` feature). Test inputs, not a model.

use super::{FbsRoutes, FbsTrunkLayer, FbsTrunkSnapshot};

/// The tests' linear congruential generator.
pub struct Lcg(pub u64);

impl Lcg {
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }
}

/// The shape and value ranges of a synthetic FBS trunk (claude-127: a trained
/// trunk is wider, deeper, non-square and has larger tables than the small one).
#[derive(Clone, Copy, Debug)]
pub struct FbsSynth {
    pub vocab: usize,
    pub d: usize,
    pub mlp: usize,
    pub layers: usize,
    pub context: usize,
    pub kept: usize,
    /// RMS gains are drawn from `gain_lo + [0, gain_span)`.
    pub gain_lo: f32,
    pub gain_span: f32,
    /// Convolution taps and biases are drawn from `±table`.
    pub table: f32,
    /// Lane `i`'s decay offset is `decay_step (i + ½)`.
    pub decay_step: f32,
}

/// claude-122's small trunk.
pub const FBS_SYNTH_SMALL: FbsSynth = FbsSynth {
    vocab: 256,
    d: 32,
    mlp: 32,
    layers: 2,
    context: 8,
    kept: 2,
    gain_lo: 0.75,
    gain_span: 0.5,
    table: 0.25,
    decay_step: 0.0625,
};

/// claude-127's non-square, multi-token trunk: MLP 96 against width 64, two
/// tokens per writer cell, 8 kept cells, 3 layers and larger tables.
pub const FBS_SYNTH_NON_SQUARE: FbsSynth = FbsSynth {
    vocab: 512,
    d: 64,
    mlp: 96,
    layers: 3,
    context: 16,
    kept: 8,
    gain_lo: 0.75,
    gain_span: 1.5,
    table: 0.5,
    decay_step: 0.25,
};

/// A synthetic SpiralCore/FBS trunk of claude-122's small shape.
pub fn synthetic_fbs_trunk(seed: u64) -> FbsTrunkSnapshot {
    synthetic_fbs_trunk_with(seed, FBS_SYNTH_SMALL)
}

/// claude-127's non-square control.
pub fn fbs_synthetic_non_square(seed: u64) -> FbsTrunkSnapshot {
    synthetic_fbs_trunk_with(seed, FBS_SYNTH_NON_SQUARE)
}

/// claude-127's tiny-gain control: one layer, width 64, 8 kept cells, with the
/// final gains of the writer's 16 address channels (the query's first and
/// last 8-blocks) at about 6e-4, as a trained trunk left them.
pub fn fbs_synthetic_tiny_gain(seed: u64) -> FbsTrunkSnapshot {
    let s = FbsSynth {
        vocab: 512,
        d: 64,
        mlp: 64,
        layers: 1,
        context: 8,
        kept: 8,
        ..FBS_SYNTH_SMALL
    };
    let mut snap = synthetic_fbs_trunk_with(seed, s);
    let d = snap.width;
    for (i, g) in (0..8).chain(d - 8..d).enumerate() {
        snap.final_norm[g] = 6e-4 * (1.0 + i as f32 / 16.0);
    }
    snap
}

/// A synthetic SpiralCore/FBS trunk of shape `s`.
pub fn synthetic_fbs_trunk_with(seed: u64, s: FbsSynth) -> FbsTrunkSnapshot {
    let mut rng = Lcg(seed);
    let (vocab, d, mlp, context) = (s.vocab, s.d, s.mlp, s.context);
    let lanes = d / 4;
    let g = lanes + d;
    let rand_vec = |rng: &mut Lcg, len: usize, scale: f32| -> Vec<f32> {
        (0..len)
            .map(|_| ((rng.next() % 2001) as i32 - 1000) as f32 / 1000.0 * scale)
            .collect()
    };
    let rand_pos = |rng: &mut Lcg, len: usize| -> Vec<f32> {
        (0..len)
            .map(|_| s.gain_lo + (rng.next() % 1000) as f32 / 1000.0 * s.gain_span)
            .collect()
    };
    let routes = |rng: &mut Lcg, in_dim: usize, out_dim: usize| -> FbsRoutes {
        let (in_blocks, out_blocks) = (in_dim / 8, out_dim / 8);
        FbsRoutes {
            in_blocks,
            out_blocks,
            atoms: (0..out_blocks)
                .map(|_| (rng.next() % in_blocks as u64) as u16)
                .collect(),
            states: (0..out_blocks).map(|_| (rng.next() % 64) as u8).collect(),
        }
    };
    let layers = (0..s.layers)
        .map(|_| FbsTrunkLayer {
            rec_norm: rand_pos(&mut rng, d),
            mlp_norm: rand_pos(&mut rng, d),
            rec_in: routes(&mut rng, d, 2 * d),
            rec_gate: routes(&mut rng, d, g),
            rec_out: routes(&mut rng, d, d),
            mlp_gate: routes(&mut rng, d, mlp),
            mlp_up: routes(&mut rng, d, mlp),
            mlp_down: routes(&mut rng, mlp, d),
            conv_taps: rand_vec(&mut rng, 4 * d, s.table),
            conv_bias: rand_vec(&mut rng, d, s.table),
            gate_bias: rand_vec(&mut rng, g, s.table),
            decay: (0..lanes).map(|i| s.decay_step * (i as f32 + 0.5)).collect(),
        })
        .collect();
    FbsTrunkSnapshot {
        vocab,
        width: d,
        mlp,
        context,
        rms_eps: 1e-5,
        kept_cells: s.kept,
        embed: rand_vec(&mut rng, vocab * d, 1.5),
        final_norm: rand_pos(&mut rng, d),
        layers,
    }
}
