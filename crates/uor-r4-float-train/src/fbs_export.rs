//! D11 export of a SpiralCore/FBS float trunk (d11-exporter-float-trunk,
//! claude-123): the converter from a [`StackModel`], as Stage 3 trains and
//! saves it, to the integer crate's [`FbsTrunkSnapshot`].
//!
//! A float FBS projection routes each output 8-block by a GPU argmin over
//! learned query points, so its routes are read back through the model's own
//! GPU projection: one coded carrier, whose input slot `i = atom*8 + c` holds
//! `i + 1`, comes out as `±(atom*8 + (code & 7) + 1)` in every output channel.
//! The atom, axis and sign of each route are then read exactly, and no host
//! argmin is computed. Everything else is copied by name.
//!
//! The snapshot is plain data (`uor_r4_integer::fbs_snapshot`), built without
//! the integer crate's serving build; `fbs-export` turns its JSON into the
//! `UORLUT01` artifact.

use candle_core::Tensor;
use uor_r4_integer::discrete_d11::{spiralcore_orbit_roots, spiralcore_signed_axes};
use uor_r4_integer::fbs_snapshot::{
    FbsFloatPosition, FbsFloatReadout, FbsFloatSequence, FbsRoutes, FbsTrunkLayer, FbsTrunkSnapshot,
};

use crate::geometric_stack::{
    Gate, QuaternionWeights, RecGates, RotationGroup, StackArch, StackConfig, StackModel,
    RMS_EPSILON,
};
use crate::native_vector_writer::WriterSpec;
use crate::{invalid, Result};

/// The six projections of one recurrence layer, in [`FbsTrunkLayer`] order.
pub const FBS_PROJECTIONS: [&str; 6] = [
    "rec.in", "rec.gate", "rec.out", "mlp.gate", "mlp.up", "mlp.down",
];

/// Refuse a trunk the integer FBS session cannot serve, each for its own reason.
pub fn check_fbs_export_config(
    config: &StackConfig,
    weights: QuaternionWeights,
    head: Option<&WriterSpec>,
) -> Result<()> {
    let refusal = if config.arch != StackArch::Geometric {
        Some("the trunk is not geometric")
    } else if config.pattern.is_empty() || config.pattern.bytes().any(|c| c != b'r') {
        Some("the integer FBS trunk serves recurrence layers only")
    } else if config.rec_gates != RecGates::Geometric {
        Some("the recurrence gates are not geometric")
    } else if !config.rotation {
        Some("the recurrence has no learned rotations")
    } else if config.rotation_group != RotationGroup::Quaternion {
        Some("the U(1) rotation control is not served")
    } else if config.gate != Gate::Hamilton {
        Some("the feed-forward gate is not Hamilton")
    } else if weights != QuaternionWeights::SpiralcoreFbs {
        Some("the projections are not SpiralCore/FBS")
    } else if config.pointer.is_some() {
        Some("a pointer head is not served")
    } else if config.memory.is_some() || config.select.is_some() || config.read_router.is_some() {
        Some("a memory, read selection or read router is not served")
    } else if head.is_none_or(|h| h.vocabulary != config.vocab_size || h.width != config.width) {
        Some("the native vector head is absent or does not match the trunk")
    } else {
        None
    };
    refusal.map_or(Ok(()), |r| Err(invalid(r)))
}

/// Compare a float geometry's signed axes (64 states × 8, low three bits an
/// axis, bit 3 a negation) and orbit roots with the integer crate's, in order.
/// The integer session applies the integer table's codes, and the writer's
/// cell `a*16 + b` names roots by their order.
pub fn compare_geometry_tables(axes: &[u32], roots: &[[i8; 8]; 16]) -> Result<()> {
    let integer_axes = spiralcore_signed_axes();
    if axes.len() != integer_axes.len()
        || axes
            .iter()
            .zip(integer_axes)
            .any(|(&f, &i)| f != u32::from(i))
    {
        return Err(invalid(
            "the float and integer Cl(0,6) signed-axis tables differ",
        ));
    }
    if roots != spiralcore_orbit_roots() {
        return Err(invalid("the float and integer orbit roots differ"));
    }
    Ok(())
}

/// [`compare_geometry_tables`] on this crate's own SpiralCore/FBS geometry.
pub fn check_geometry_tables() -> Result<()> {
    let plan = crate::spiralcore_fbs::ProjectionPlan::new(8, 8)?;
    compare_geometry_tables(&plan.geometry.axes, &plan.geometry.roots)
}

/// The coded carrier of a projection with `in_blocks` input 8-blocks: slot `i`
/// holds `i + 1`, an exact `f32` integer.
pub fn coded_carrier(in_blocks: usize) -> Vec<f32> {
    (1..=in_blocks * 8).map(|i| i as f32).collect()
}

/// Read the routes of a projection back from its output on the coded carrier.
///
/// Each output block must read one input atom through eight whole signed
/// axes, and the eight codes must be a state of the integer Cl(0,6) table;
/// anything else (a mix of atoms, a scaled or non-integer value, a code row
/// outside the group) is refused, because the integer session serves only a
/// gather and a sign.
pub fn decode_coded_routes(
    projected: &[f32],
    in_blocks: usize,
    out_blocks: usize,
) -> Result<FbsRoutes> {
    if in_blocks == 0 || in_blocks > usize::from(u16::MAX) + 1 || projected.len() != out_blocks * 8
    {
        return Err(invalid(format!(
            "a coded projection of {in_blocks} -> {out_blocks} blocks needs {} values, got {}",
            out_blocks * 8,
            projected.len()
        )));
    }
    let slots = (in_blocks * 8) as f32;
    let axes = spiralcore_signed_axes();
    let mut atoms = Vec::with_capacity(out_blocks);
    let mut states = Vec::with_capacity(out_blocks);
    for (o, block) in projected.chunks_exact(8).enumerate() {
        let mut atom = None;
        let mut codes = [0u8; 8];
        for (j, &v) in block.iter().enumerate() {
            let k = v.abs();
            if !(k >= 1.0 && k <= slots && k.fract() == 0.0) {
                return Err(invalid(format!(
                    "block {o} channel {j}: {v} is not a coded input slot"
                )));
            }
            let slot = k as usize - 1;
            if *atom.get_or_insert(slot / 8) != slot / 8 {
                return Err(invalid(format!(
                    "block {o} reads two atoms: the projection is not a signed gather"
                )));
            }
            codes[j] = (slot % 8) as u8 | if v < 0.0 { 8 } else { 0 };
        }
        let state = axes
            .chunks_exact(8)
            .position(|row| row == codes)
            .ok_or_else(|| {
                invalid(format!(
                    "block {o}: codes {codes:?} are not a state of the Cl(0,6) table"
                ))
            })?;
        atoms.push(atom.map_or(0, |a| a as u16));
        states.push(state as u8);
    }
    Ok(FbsRoutes {
        in_blocks,
        out_blocks,
        atoms,
        states,
    })
}

/// The `(input, output)` widths of projection `part` in a trunk of width `d`,
/// MLP width `m` and `g = d/4 + d` gate rows.
fn projection_widths(part: &str, d: usize, m: usize) -> (usize, usize) {
    match part {
        "rec.in" => (d, 2 * d),
        "rec.gate" => (d, d / 4 + d),
        "rec.out" => (d, d),
        "mlp.gate" | "mlp.up" => (d, m),
        _ => (m, d),
    }
}

/// Read the routes of one projection variable back through the model's own GPU
/// projection on the coded carrier.
fn read_routes(
    model: &StackModel,
    name: &str,
    in_width: usize,
    out_width: usize,
) -> Result<FbsRoutes> {
    let queries = model
        .variables()
        .get(name)
        .ok_or_else(|| invalid(format!("the trunk lacks {name}")))?
        .as_tensor();
    let carrier = Tensor::from_vec(coded_carrier(in_width / 8), (1, in_width), queries.device())?;
    let projected = model
        .linear(&carrier, queries)?
        .flatten_all()?
        .to_vec1::<f32>()?;
    decode_coded_routes(&projected, in_width / 8, out_width / 8)
        .map_err(|e| invalid(format!("{name}: {e}")))
}

/// Refuse a copied table that holds a value that is not finite, naming it: a
/// diverged trunk must not reach the JSON handoff, which would write `null`.
pub fn finite_table(name: &str, values: Vec<f32>) -> Result<Vec<f32>> {
    match values.iter().position(|v| !v.is_finite()) {
        Some(i) => Err(invalid(format!(
            "{name}[{i}] = {} is not finite",
            values[i]
        ))),
        None => Ok(values),
    }
}

/// Refuse to read a trunk on a thread with a SpiralCore/FBS sheet arm set:
/// `linear` would route through the arm, and the snapshot would not say so.
fn check_no_sheet_arm() -> Result<()> {
    match crate::spiralcore_fbs_arms::fbs_arm() {
        Some(arm) => Err(invalid(format!(
            "a SpiralCore/FBS sheet arm ({arm:?}) is set on this thread; clear it before export"
        ))),
        None => Ok(()),
    }
}

/// Convert a SpiralCore/FBS float trunk to the integer crate's snapshot. GPU
/// only: the routes are read back through the model's CUDA projection.
pub fn fbs_trunk_snapshot(model: &StackModel) -> Result<FbsTrunkSnapshot> {
    let config = &model.config;
    let head = model.native_vector_head_spec();
    check_no_sheet_arm()?;
    check_fbs_export_config(config, model.quaternion_weights(), head)?;
    check_geometry_tables()?;
    model.validate_native_learning_path()?;
    let kept_cells = head.map_or(0, |h| h.kept_cells);
    let (d, m) = (config.width, config.mlp_hidden);
    let values = |name: &str| -> Result<Vec<f32>> {
        let values = model
            .variables()
            .get(name)
            .ok_or_else(|| invalid(format!("the trunk lacks {name}")))?
            .as_tensor()
            .flatten_all()?
            .to_vec1::<f32>()?;
        finite_table(name, values)
    };
    let mut layers = Vec::with_capacity(config.layers());
    for layer in 0..config.layers() {
        let at = |suffix: &str| format!("layers.{layer:02}.{suffix}");
        let mut routes = Vec::with_capacity(FBS_PROJECTIONS.len());
        for part in FBS_PROJECTIONS {
            let (in_width, out_width) = projection_widths(part, d, m);
            routes.push(read_routes(
                model,
                &at(&format!("{part}.spiralcore_fbs")),
                in_width,
                out_width,
            )?);
        }
        let [rec_in, rec_gate, rec_out, mlp_gate, mlp_up, mlp_down]: [FbsRoutes; 6] = routes
            .try_into()
            .map_err(|_| invalid("six projections per layer"))?;
        layers.push(FbsTrunkLayer {
            rec_norm: values(&at("rec_norm.weight"))?,
            mlp_norm: values(&at("mlp_norm.weight"))?,
            rec_in,
            rec_gate,
            rec_out,
            mlp_gate,
            mlp_up,
            mlp_down,
            // `[CONVOLUTION_WIDTH, d]` row-major: tap 0 on the current position.
            conv_taps: values(&at("rec.conv.weight"))?,
            conv_bias: values(&at("rec.conv.bias"))?,
            gate_bias: values(&at("rec.gate.bias"))?,
            decay: values(&at("rec.decay"))?,
        });
    }
    Ok(FbsTrunkSnapshot {
        vocab: config.vocab_size,
        width: d,
        mlp: m,
        context: config.context,
        rms_eps: RMS_EPSILON,
        kept_cells,
        embed: values("embedding.weight")?,
        final_norm: values("final_norm.weight")?,
        layers,
    })
}

/// The float GPU trunk's own output for every single-token input: its final
/// hidden vector and its native writer's next token. With `export_mode`, both
/// Hamilton gates run in the float sheets' D11 export mode (Q12-rounded, then
/// snapped to the 120 icosians). `fbs-compare` holds the converted snapshot
/// against it. GPU only.
pub fn fbs_float_readout(model: &StackModel, export_mode: bool) -> Result<FbsFloatReadout> {
    check_no_sheet_arm()?;
    if export_mode {
        return crate::hamilton_sheet::with_d11_export_gates(|| fbs_float_readout(model, false))
            .map(|r| FbsFloatReadout {
                export_mode: true,
                ..r
            });
    }
    let (vocab, width) = (model.config.vocab_size, model.config.width);
    let mut positions = Vec::with_capacity(vocab);
    for token in 0..vocab as u32 {
        let hidden = model
            .hidden(&[token], 1, 1)?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let next = model
            .native_next_token_ids(&[token], 1, 1)?
            .flatten_all()?
            .to_vec1::<u32>()?;
        let [next] = next.as_slice() else {
            return Err(invalid("the native writer returned other than one token"));
        };
        positions.push(FbsFloatPosition {
            token,
            next: *next,
            hidden: finite_table("hidden", hidden)?,
        });
    }
    // Eight fixed full-context sequences: past the first position they reach
    // the transition, the later convolution taps and the carried state.
    let time = model.config.context;
    let mut sequences = Vec::with_capacity(READOUT_SEQUENCES);
    for s in 0..READOUT_SEQUENCES {
        let tokens: Vec<u32> = (0..time)
            .map(|t| ((s * 37 + t * 11 + 1) % vocab) as u32)
            .collect();
        let hidden = model.hidden(&tokens, 1, time)?.to_vec2::<f32>()?;
        let next = model
            .native_next_token_ids(&tokens, 1, time)?
            .flatten_all()?
            .to_vec1::<u32>()?;
        if next.len() != time {
            return Err(invalid(
                "the native writer returned other than one token per position",
            ));
        }
        sequences.push(FbsFloatSequence {
            tokens,
            next,
            hidden: hidden
                .into_iter()
                .map(|h| finite_table("hidden", h))
                .collect::<Result<_>>()?,
        });
    }
    Ok(FbsFloatReadout {
        vocab,
        width,
        export_mode: false,
        positions,
        sequences,
    })
}

/// The fixed sequences [`fbs_float_readout`] reads, each the full context.
pub const READOUT_SEQUENCES: usize = 8;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_stack::Objective;

    /// The Stage 3 trunk's configuration at a small size.
    fn small_config(seed: u64) -> StackConfig {
        let mut config = StackConfig::transformer_control(seed);
        config.arch = StackArch::Geometric;
        config.pattern = "rr".into();
        config.width = 32;
        config.heads = 4;
        config.mlp_hidden = 32;
        config.context = 8;
        config.vocab_size = 256;
        config.rotation = true;
        config.rotation_group = RotationGroup::Quaternion;
        config.gate = Gate::Hamilton;
        config.rec_gates = RecGates::Geometric;
        config.objective = Objective::MarginHinge { rivals: 4 };
        config
    }

    fn small_head() -> WriterSpec {
        WriterSpec {
            vocabulary: 256,
            width: 32,
            kept_cells: 2,
        }
    }

    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            self.0 >> 33
        }
    }

    /// Apply `routes` to `x` (`in_blocks * 8` values) on the host with the
    /// integer table: the gather the integer session serves.
    fn host_gather(routes: &FbsRoutes, x: &[f32]) -> Vec<f32> {
        let axes = spiralcore_signed_axes();
        let mut y = Vec::with_capacity(routes.out_blocks * 8);
        for (&atom, &state) in routes.atoms.iter().zip(&routes.states) {
            for &code in &axes[usize::from(state) * 8..usize::from(state) * 8 + 8] {
                let v = x[usize::from(atom) * 8 + usize::from(code & 7)];
                y.push(if code & 8 != 0 { -v } else { v });
            }
        }
        y
    }

    #[test]
    fn geometry_tables_match_the_integer_crate() {
        check_geometry_tables().expect("the float geometry is the integer crate's");
        let plan = crate::spiralcore_fbs::ProjectionPlan::new(8, 8).expect("plan");
        let (axes, roots) = (&plan.geometry.axes, plan.geometry.roots);
        // A changed code and a reordered root are each refused.
        let mut changed = axes.clone();
        changed[9] ^= 8;
        assert!(compare_geometry_tables(&changed, &roots).is_err());
        let mut reordered = roots;
        reordered.swap(1, 2);
        assert!(compare_geometry_tables(axes, &reordered).is_err());
        assert!(compare_geometry_tables(&axes[..504], &roots).is_err());
    }

    #[test]
    fn coded_carrier_decode_recovers_a_gather_exactly() {
        let mut rng = Lcg(20261026);
        for (in_blocks, out_blocks) in [(1usize, 3usize), (4, 8), (4, 5), (32, 4)] {
            let routes = FbsRoutes {
                in_blocks,
                out_blocks,
                atoms: (0..out_blocks)
                    .map(|_| (rng.next() % in_blocks as u64) as u16)
                    .collect(),
                states: (0..out_blocks).map(|_| (rng.next() % 64) as u8).collect(),
            };
            let projected = host_gather(&routes, &coded_carrier(in_blocks));
            assert_eq!(
                decode_coded_routes(&projected, in_blocks, out_blocks).expect("decode"),
                routes
            );
        }
    }

    #[test]
    fn coded_carrier_decode_refuses_what_is_not_a_signed_gather() {
        let routes = FbsRoutes {
            in_blocks: 4,
            out_blocks: 2,
            atoms: vec![1, 3],
            states: vec![5, 40],
        };
        let good = host_gather(&routes, &coded_carrier(4));
        assert!(decode_coded_routes(&good, 4, 2).is_ok());
        type Mutation = fn(&mut Vec<f32>);
        let refused: [(&str, Mutation, &str); 6] = [
            (
                "two atoms in a block",
                |p| p[3] = if p[3] < 0.0 { -1.0 } else { 1.0 },
                "block 0 reads two atoms",
            ),
            (
                "a slot past the carrier",
                |p| p[9] = 33.0,
                "block 1 channel 1: 33 is not a coded input slot",
            ),
            ("a fractional value", |p| p[2] += 0.25, "block 0 channel 2:"),
            (
                "NaN",
                |p| p[0] = f32::NAN,
                "block 0 channel 0: NaN is not a coded input slot",
            ),
            (
                "zero",
                |p| p[15] = 0.0,
                "block 1 channel 7: 0 is not a coded input slot",
            ),
            (
                "a code row outside the group",
                |p| {
                    // Every channel reads axis 0 of atom 1 (slot 9): one atom, no permutation.
                    for v in &mut p[..8] {
                        *v = 9.0;
                    }
                },
                "block 0: codes [0, 0, 0, 0, 0, 0, 0, 0] are not a state of the Cl(0,6) table",
            ),
        ];
        for (what, mutate, reason) in refused {
            let mut projected = good.clone();
            mutate(&mut projected);
            match decode_coded_routes(&projected, 4, 2) {
                Err(e) => assert!(e.to_string().contains(reason), "{what}: {e}"),
                Ok(r) => panic!("{what} must be refused, decoded {r:?}"),
            }
        }
        assert!(decode_coded_routes(&good[..8], 4, 2).is_err());
        assert!(decode_coded_routes(&good, 0, 2).is_err());
    }

    #[test]
    fn copied_tables_refuse_non_finite_values() {
        assert_eq!(
            finite_table("t", vec![1.0, -2.5]).expect("finite"),
            vec![1.0, -2.5]
        );
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let e = finite_table("layers.01.rec.decay", vec![0.5, bad]).expect_err("refused");
            assert!(e.to_string().contains("layers.01.rec.decay[1]"), "{e}");
        }
    }

    #[test]
    fn export_config_refuses_what_the_integer_session_cannot_serve() {
        let head = small_head();
        let fbs = QuaternionWeights::SpiralcoreFbs;
        assert!(check_fbs_export_config(&small_config(1), fbs, Some(&head)).is_ok());
        type Mutation = fn(&mut StackConfig);
        let refused: [(&str, Mutation, &str); 9] = [
            (
                "a transformer",
                |c| c.arch = StackArch::Transformer,
                "the trunk is not geometric",
            ),
            (
                "a read layer",
                |c| c.pattern = "ra".into(),
                "the integer FBS trunk serves recurrence layers only",
            ),
            (
                "historical gates",
                |c| c.rec_gates = RecGates::Historical,
                "the recurrence gates are not geometric",
            ),
            (
                "no rotations",
                |c| c.rotation = false,
                "the recurrence has no learned rotations",
            ),
            (
                "the U(1) control",
                |c| c.rotation_group = RotationGroup::U1,
                "the U(1) rotation control is not served",
            ),
            (
                "a SwiGLU gate",
                |c| c.gate = Gate::Swiglu,
                "the feed-forward gate is not Hamilton",
            ),
            (
                "a pointer",
                |c| {
                    c.pointer = Some(
                        serde_json::from_value(serde_json::json!({"dim": 8})).expect("pointer"),
                    )
                },
                "a pointer head is not served",
            ),
            (
                "a memory",
                |c| c.memory = Some(serde_json::json!({})),
                "a memory, read selection or read router is not served",
            ),
            (
                "a wider vocabulary",
                |c| c.vocab_size = 512,
                "the native vector head is absent or does not match the trunk",
            ),
        ];
        for (what, mutate, reason) in refused {
            let mut config = small_config(1);
            mutate(&mut config);
            match check_fbs_export_config(&config, fbs, Some(&head)) {
                Err(e) => assert!(e.to_string().contains(reason), "{what}: {e}"),
                Ok(()) => panic!("{what} must be refused"),
            }
        }
        let config = small_config(1);
        for (what, weights, head, reason) in [
            (
                "dense projections",
                QuaternionWeights::None,
                Some(&head),
                "the projections are not SpiralCore/FBS",
            ),
            (
                "no native head",
                fbs,
                None,
                "the native vector head is absent or does not match the trunk",
            ),
        ] {
            match check_fbs_export_config(&config, weights, head) {
                Err(e) => assert!(e.to_string().contains(reason), "{what}: {e}"),
                Ok(()) => panic!("{what} must be refused"),
            }
        }
    }

    /// On one GPU: the read-back routes reproduce the model's projection bit for
    /// bit, and the trunk converts and reads out. With `UOR_FBS_CONVERTER_OUT`
    /// set to a directory, it writes `snapshot.json` and `readout.json` there for
    /// the serving build's `fbs-compare` (export, parse, serve, and a reading).
    /// The serving build is never a dependency of this crate (D29).
    #[cfg(feature = "cuda")]
    #[test]
    fn gpu_converted_trunk_reproduces_the_projection() -> Result<()> {
        let device = candle_core::Device::new_cuda(0)?;
        let mut model = StackModel::new(small_config(20261026), &device)?;
        model.set_quaternion_weights(QuaternionWeights::SpiralcoreFbs)?;
        model.set_native_vector_head(2)?;
        let snap = fbs_trunk_snapshot(&model)?;
        let (d, m) = (snap.width, snap.mlp);

        // Every projection of every layer, on random carriers: bit-identical.
        let mut rng = Lcg(7);
        for (layer, l) in snap.layers.iter().enumerate() {
            let routes = [
                &l.rec_in,
                &l.rec_gate,
                &l.rec_out,
                &l.mlp_gate,
                &l.mlp_up,
                &l.mlp_down,
            ];
            for (part, routes) in FBS_PROJECTIONS.into_iter().zip(routes) {
                let (in_width, _) = projection_widths(part, d, m);
                let name = format!("layers.{layer:02}.{part}.spiralcore_fbs");
                let queries = model.variables().get(&name).expect("queries").as_tensor();
                for _ in 0..3 {
                    let x: Vec<f32> = (0..in_width)
                        .map(|_| (rng.next() % 20001) as f32 / 1000.0 - 10.0)
                        .collect();
                    let carrier = Tensor::from_vec(x.clone(), (1, in_width), &device)?;
                    let gpu = model
                        .linear(&carrier, queries)?
                        .flatten_all()?
                        .to_vec1::<f32>()?;
                    let host = host_gather(routes, &x);
                    assert_eq!(gpu.len(), host.len(), "{name}: output width");
                    assert!(
                        gpu.iter()
                            .zip(&host)
                            .all(|(a, b)| a.to_bits() == b.to_bits()),
                        "{name}: the read-back routes differ from the GPU projection"
                    );
                }
            }
        }

        // The trunk reads out, and an active sheet arm is refused.
        let readout = fbs_float_readout(&model, false)?;
        let export = fbs_float_readout(&model, true)?;
        assert!(!readout.export_mode && export.export_mode);
        for r in [&readout, &export] {
            r.check(snap.vocab, snap.width, snap.context, r.export_mode)
                .map_err(invalid)?;
        }
        // Causality, in both modes: a token read alone from a zero state is
        // that token at the first position of a sequence. The export-mode gate
        // hook once broke this at time 1 (claude-124).
        for r in [&readout, &export] {
            for q in &r.sequences {
                let alone = &r.positions[q.tokens[0] as usize];
                let gap = alone
                    .hidden
                    .iter()
                    .zip(&q.hidden[0])
                    .map(|(a, b)| (a - b).abs())
                    .fold(0f32, f32::max);
                assert!(
                    gap <= 1e-5 && alone.next == q.next[0],
                    "export_mode {}: token {} alone differs from position 0 of its sequence (gap {gap:e})",
                    r.export_mode,
                    q.tokens[0]
                );
            }
        }
        // The export mode reaches the forward: some hidden vector changes.
        assert!(readout
            .positions
            .iter()
            .zip(&export.positions)
            .any(|(a, b)| a.hidden != b.hidden));
        crate::spiralcore_fbs_arms::set_fbs_arm(Some(
            crate::spiralcore_fbs_arms::FbsArm::OneAtomActive,
        ));
        let armed = fbs_trunk_snapshot(&model);
        crate::spiralcore_fbs_arms::set_fbs_arm(None);
        assert!(armed.is_err_and(|e| e.to_string().contains("sheet arm")));
        if let Ok(dir) = std::env::var("UOR_FBS_CONVERTER_OUT") {
            let dir = std::path::Path::new(&dir);
            for (name, json) in [
                ("snapshot.json", serde_json::to_vec(&snap)?),
                ("readout.json", serde_json::to_vec(&readout)?),
                ("readout-export.json", serde_json::to_vec(&export)?),
            ] {
                uor_r4_integer::fbs_snapshot::write_new_file(&dir.join(name), &json)?;
            }
            eprintln!(
                "fbs converter: wrote snapshot.json, readout.json and readout-export.json to {}",
                dir.display()
            );
        }
        Ok(())
    }
}
