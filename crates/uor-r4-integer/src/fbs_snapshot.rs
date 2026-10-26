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

/// The float GPU trunk's single-token outputs, for comparison with its
/// converted snapshot's reference and integer session.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsFloatReadout {
    pub vocab: usize,
    pub width: usize,
    pub positions: Vec<FbsFloatPosition>,
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
