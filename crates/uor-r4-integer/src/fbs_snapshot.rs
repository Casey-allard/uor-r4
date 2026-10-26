//! The offline SpiralCore/FBS float-trunk snapshot: plain data, with no serving
//! code, so a converter in the native graph can build it without the
//! `serving-wasm` feature (d11-exporter-float-trunk, claude-123).
//!
//! The serving build exports it with `stack::export_fbs_trunk_uorlut01`, and
//! `stack::format` re-exports these types. The snapshot serializes to JSON for
//! the handoff between the two builds; `serde_json` writes each `f32` so that
//! it reads back exactly.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The fixed-point exponent the exporter stores biases and decay offsets at
/// (`stack::kernels::RESIDUAL_EXP`, held equal by a stack test).
pub const FBS_FIXED_EXP: i32 = -16;

/// Encode a finite scalar as a grid code `±(16 + m) 2^(e - 4)` with
/// `1 <= e + 64 <= 127`: the exporter's form for gains and convolution taps.
/// `stack::encode_grid_code` wraps it.
pub fn grid_code(value: f64, positive: bool) -> Result<i16, String> {
    if !value.is_finite() || (positive && value <= 0.0) {
        return Err("nonfinite or non-positive scalar".to_owned());
    }
    if value.abs() < 1e-18 {
        if positive {
            return Err("zero positive grid scalar".to_owned());
        }
        return Ok(0);
    }
    let abs = value.abs();
    let mut best_code = 0i16;
    let mut best_err = f64::INFINITY;
    for field in 1i16..=127 {
        let e = i32::from(field) - 64;
        let step = 2f64.powi(e - 4);
        for m in 0i16..16 {
            let cand = f64::from(16 + m) * step;
            let err = (abs - cand).abs();
            if err < best_err {
                best_err = err;
                best_code = (field << 4) | m;
            }
        }
    }
    Ok(if value < 0.0 { -best_code } else { best_code })
}

/// Decode a grid code `±(16 + m) 2^(e - 4)` to `f64`.
pub fn grid_value(code: i16) -> f64 {
    if code == 0 {
        return 0.0;
    }
    let c = code.unsigned_abs();
    let e = i32::from(c >> 4) - 64;
    let mag = f64::from(16 + (c & 15)) * 2f64.powi(e - 4);
    if code < 0 {
        -mag
    } else {
        mag
    }
}

/// `v` rounded to fixed point at `2^exp`, as the exporter stores it.
pub fn fixed_value(v: f32, exp: i32) -> f64 {
    (f64::from(v) * 2f64.powi(-exp)).round() * 2f64.powi(exp)
}

/// `v` at Q12 in `i16`, as the exporter writes the writer table, or `None`
/// when it is not finite or does not fit (the exporter refuses both).
pub fn q12_value(v: f32) -> Option<f64> {
    if !v.is_finite() {
        return None;
    }
    i16::try_from((f64::from(v) * 4096.0).round() as i64)
        .ok()
        .map(|k| f64::from(k) / 4096.0)
}

/// Write `bytes` to a new file at `path`, never leaving a partial file there
/// and never replacing one. The bytes go to `<path>.partial` first, which is
/// linked into place only when complete (the link refuses an existing
/// `path`); the partial file is removed either way. For the handoff bins.
pub fn write_new_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut partial = path.as_os_str().to_owned();
    partial.push(".partial");
    let partial = PathBuf::from(partial);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&partial)?;
    let written = file.write_all(bytes).and_then(|()| file.sync_all());
    drop(file);
    let linked = written.and_then(|()| std::fs::hard_link(&partial, path));
    let removed = std::fs::remove_file(&partial);
    linked.and(removed)
}

/// SpiralCore/FBS routes of one projection: one `(atom, state)` per output 8-block.
///
/// Output block `o` reads input block `atoms[o]`: its channel `j` is
/// `y[o*8 + j] = ±x[atoms[o]*8 + (code & 7)]`, negated when `code & 8`, where
/// `code` is the `j`-th code of the 64-state Cl(0,6) signed-axis table at
/// `states[o]` (`discrete_d11::spiralcore_signed_axes`). The integer session
/// serves it as a gather and a sign (`stack_fbs_gather`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsRoutes {
    /// Input 8-blocks the atoms index.
    pub in_blocks: usize,
    /// Output 8-blocks, one route each.
    pub out_blocks: usize,
    /// `out_blocks` input-block indices, each below `in_blocks`.
    pub atoms: Vec<u16>,
    /// `out_blocks` signed-axis states, each below 64.
    pub states: Vec<u8>,
}

/// One recurrence layer of a SpiralCore/FBS trunk snapshot (`d = width`,
/// `m = mlp`, `lanes = d / 4`, `g = lanes + d`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsTrunkLayer {
    /// `d` RMSNorm gain before the recurrence.
    pub rec_norm: Vec<f32>,
    /// `d` RMSNorm gain before the Hamilton feed-forward block.
    pub mlp_norm: Vec<f32>,
    /// `d/8 -> 2d/8` drive and output-gate routes.
    pub rec_in: FbsRoutes,
    /// `d/8 -> g/8` decay-gate and rotation routes.
    pub rec_gate: FbsRoutes,
    /// `d/8 -> d/8` output routes.
    pub rec_out: FbsRoutes,
    /// `d/8 -> m/8` feed-forward gate routes.
    pub mlp_gate: FbsRoutes,
    /// `d/8 -> m/8` feed-forward up routes.
    pub mlp_up: FbsRoutes,
    /// `m/8 -> d/8` feed-forward down routes.
    pub mlp_down: FbsRoutes,
    /// `4 * d` causal convolution taps `[tap][channel]`, tap 0 on the current
    /// position, as the dense trunk's `FloatTrunkRecurrence::conv_taps`.
    pub conv_taps: Vec<f32>,
    /// `d` causal convolution biases.
    pub conv_bias: Vec<f32>,
    /// `g` gate biases (decay-gate bias then rotation bias).
    pub gate_bias: Vec<f32>,
    /// `lanes` learned per-lane decay offsets.
    pub decay: Vec<f32>,
}

/// The float GPU trunk's own output at one single-token position: the
/// comparison side of `fbs-compare`, written by the float converter.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsFloatPosition {
    /// The input token.
    pub token: u32,
    /// The float native writer's next token.
    pub next: u32,
    /// `width` final hidden values, after the final RMSNorm gain.
    pub hidden: Vec<f32>,
}

/// The float GPU trunk's outputs at every position of one token sequence: the
/// recurrence's transition, its later convolution taps and the carried state
/// act only after the first position (claude-124).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsFloatSequence {
    /// The input tokens, in order.
    pub tokens: Vec<u32>,
    /// The float native writer's next token at each position.
    pub next: Vec<u32>,
    /// `width` final hidden values at each position.
    pub hidden: Vec<Vec<f32>>,
}

/// How the float GPU trunk was read (claude-124, claude-125).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FbsReadoutMode {
    /// Plain float: nothing snapped, tables at full precision.
    Plain,
    /// The float sheets' D11 export mode: Q12-rounded, icosian-snapped
    /// Hamilton gates; the transition and the tables as in `Plain`.
    Export,
    /// The D11 mode: the gates and the transition snapped to the 120 icosians
    /// directly (no Q12 rounding of the gates, as the integer path), the
    /// tables rounded as exported and the final query rounded to Q12, so the
    /// float trunk computes what the artifact computes.
    D11,
}

/// The float GPU trunk's single-token and sequence outputs, for comparison
/// with its converted snapshot's reference and integer session.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsFloatReadout {
    pub vocab: usize,
    pub width: usize,
    /// The mode the trunk was read in.
    pub mode: FbsReadoutMode,
    /// One position per token, in token order.
    pub positions: Vec<FbsFloatPosition>,
    /// Token sequences read position by position.
    pub sequences: Vec<FbsFloatSequence>,
}

impl FbsFloatReadout {
    /// Refuse a readout that does not hold every token of a `vocab`-token,
    /// `width`-wide trunk once, in token order, with a full hidden vector,
    /// that holds a sequence that is empty, longer than `context`, out of the
    /// vocabulary or not fully read, or that was read in the other mode.
    pub fn check(
        &self,
        vocab: usize,
        width: usize,
        context: usize,
        mode: FbsReadoutMode,
    ) -> Result<(), String> {
        if self.sequences.is_empty() {
            return Err("the readout holds no sequences".to_owned());
        }
        for (s, q) in self.sequences.iter().enumerate() {
            let n = q.tokens.len();
            if n == 0
                || n > context
                || q.tokens.iter().any(|&t| t as usize >= vocab)
                || q.next.len() != n
                || q.hidden.len() != n
                || q.hidden.iter().any(|h| h.len() != width)
            {
                return Err(format!(
                    "sequence {s} is not 1 to {context} tokens with an output at each"
                ));
            }
        }
        if self.vocab != vocab || self.width != width {
            return Err("the readout's vocabulary or width differs from the trunk's".to_owned());
        }
        if self.mode != mode {
            return Err(format!("the readout was read in mode {:?}", self.mode));
        }
        if self.positions.len() != vocab {
            return Err(format!(
                "{} positions for a vocabulary of {vocab}",
                self.positions.len()
            ));
        }
        match self
            .positions
            .iter()
            .enumerate()
            .find(|(i, p)| p.token as usize != *i || p.hidden.len() != width)
        {
            Some((i, _)) => Err(format!("position {i} is not token {i} with {width} values")),
            None => Ok(()),
        }
    }
}

/// Offline SpiralCore/FBS trunk snapshot handed to `stack::export_fbs_trunk_uorlut01`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsTrunkSnapshot {
    pub vocab: usize,
    pub width: usize,
    pub mlp: usize,
    pub context: usize,
    pub rms_eps: f64,
    /// Cells the native vector head keeps (1 to 16).
    pub kept_cells: usize,
    /// `vocab * width` row-major: the input embedding and the writer table.
    pub embed: Vec<f32>,
    /// `width` final RMSNorm gain.
    pub final_norm: Vec<f32>,
    pub layers: Vec<FbsTrunkLayer>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_new_file_never_replaces_and_never_leaves_a_partial_file() {
        let dir = std::env::temp_dir().join(format!("uor-fbs-write-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("out.json");
        let partial = dir.join("out.json.partial");
        let _ = std::fs::remove_file(&path);
        write_new_file(&path, b"first").expect("a new file is written");
        assert_eq!(std::fs::read(&path).expect("read"), b"first");
        assert!(!partial.exists());
        // An existing file is refused and kept as it was.
        assert!(write_new_file(&path, b"second").is_err());
        assert_eq!(std::fs::read(&path).expect("read"), b"first");
        assert!(!partial.exists());
        std::fs::remove_dir_all(&dir).expect("clean up");
    }

    #[test]
    fn float_readout_check_refuses_an_incomplete_or_misordered_readout() {
        let readout = |tokens: &[u32], mode: FbsReadoutMode| FbsFloatReadout {
            vocab: 3,
            width: 2,
            mode,
            positions: tokens
                .iter()
                .map(|&token| FbsFloatPosition {
                    token,
                    next: 0,
                    hidden: vec![0.0; 2],
                })
                .collect(),
            sequences: vec![FbsFloatSequence {
                tokens: vec![2, 0],
                next: vec![1, 1],
                hidden: vec![vec![0.0; 2]; 2],
            }],
        };
        assert!(readout(&[0, 1, 2], FbsReadoutMode::Plain)
            .check(3, 2, 4, FbsReadoutMode::Plain)
            .is_ok());
        assert!(readout(&[0, 1, 2], FbsReadoutMode::Export)
            .check(3, 2, 4, FbsReadoutMode::Export)
            .is_ok());
        for (what, r, reason) in [
            ("empty", readout(&[], FbsReadoutMode::Plain), "0 positions"),
            (
                "a token missing",
                readout(&[0, 1], FbsReadoutMode::Plain),
                "2 positions",
            ),
            (
                "a token repeated",
                readout(&[0, 1, 1], FbsReadoutMode::Plain),
                "position 2 is not token 2",
            ),
            (
                "out of order",
                readout(&[1, 0, 2], FbsReadoutMode::Plain),
                "position 0 is not token 0",
            ),
            (
                "the other mode",
                readout(&[0, 1, 2], FbsReadoutMode::Export),
                "mode Export",
            ),
        ] {
            match r.check(3, 2, 4, FbsReadoutMode::Plain) {
                Err(e) => assert!(e.contains(reason), "{what}: {e}"),
                Ok(()) => panic!("{what} must be refused"),
            }
        }
        let mut short = readout(&[0, 1, 2], FbsReadoutMode::Plain);
        short.positions[1].hidden.pop();
        assert!(short
            .check(3, 2, 4, FbsReadoutMode::Plain)
            .is_err_and(|e| e.contains("position 1")));
        assert!(readout(&[0, 1, 2], FbsReadoutMode::Plain)
            .check(4, 2, 4, FbsReadoutMode::Plain)
            .is_err());
        let mut none = readout(&[0, 1, 2], FbsReadoutMode::Plain);
        none.sequences.clear();
        assert!(none
            .check(3, 2, 4, FbsReadoutMode::Plain)
            .is_err_and(|e| e.contains("no sequences")));
        // A sequence must be 1 to `context` in-vocabulary tokens, each read.
        type Seq = fn(&mut FbsFloatSequence);
        let bad: [(&str, Seq); 5] = [
            ("empty", |q| {
                q.tokens.clear();
                q.next.clear();
                q.hidden.clear();
            }),
            ("past the context", |q| {
                q.tokens = vec![0; 5];
                q.next = vec![0; 5];
                q.hidden = vec![vec![0.0; 2]; 5];
            }),
            ("a token out of the vocabulary", |q| q.tokens[1] = 3),
            ("a position not read", |q| {
                q.next.pop();
            }),
            ("a short hidden vector", |q| {
                q.hidden[1].pop();
            }),
        ];
        for (what, mutate) in bad {
            let mut r = readout(&[0, 1, 2], FbsReadoutMode::Plain);
            mutate(&mut r.sequences[0]);
            match r.check(3, 2, 4, FbsReadoutMode::Plain) {
                Err(e) => assert!(e.contains("sequence 0"), "{what}: {e}"),
                Ok(()) => panic!("{what} must be refused"),
            }
        }
    }

    #[test]
    fn json_f32_round_trips_bit_exactly() {
        // The snapshot hands every f32 over as JSON; each must read back to
        // the same bits, signed zeros, subnormals and the extremes included.
        let mut values = vec![
            0.0f32,
            -0.0,
            f32::MIN_POSITIVE,
            -f32::MIN_POSITIVE,
            f32::from_bits(1),
            f32::from_bits(0x8000_0001),
            f32::from_bits(0x007f_ffff),
            f32::MAX,
            f32::MIN,
            f32::EPSILON,
            1.0 / 3.0,
            0.1,
        ];
        let mut state = 0x2026_1026_u64;
        while values.len() < 200_000 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let v = f32::from_bits((state >> 32) as u32);
            if v.is_finite() {
                values.push(v);
            }
        }
        let json = serde_json::to_vec(&values).expect("serialize");
        let back: Vec<f32> = serde_json::from_slice(&json).expect("deserialize");
        let changed = values
            .iter()
            .zip(&back)
            .filter(|(a, b)| a.to_bits() != b.to_bits())
            .count();
        assert_eq!(
            changed,
            0,
            "{changed} of {} values changed bits",
            values.len()
        );
    }
}
