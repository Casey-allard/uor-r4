//! Oracle tests of the multiplier-free (D11) stack engine,
//! `uor_r4_integer::stack`, against the frozen D10 comparator,
//! `uor_r4_lut::stack`, on artifacts written by `stack_export::export_stack`:
//! the two engines must return identical logits (and, for a pointer-copy
//! head, identical Q30 mixtures) at every position, and the D11 loader must
//! reject malformed containers without panicking.
//!
//! Run in release (`cargo test -p uor-r4-training --release --test
//! stack_d11_oracle`): the D10 engine's plain integer operators wrap there, as
//! they do in every served build, and the D11 engine reproduces that
//! arithmetic.

use candle_core::{Device, Tensor};
use serde_json::{json, Value};
use uor_r4_integer::stack::{IntegerStackModel, StackError};
use uor_r4_lut::format::StackArtifact;
use uor_r4_lut::stack::StackModel as D10Model;
use uor_r4_training::geometric_stack::{
    PointerConfig, ReadScore, StackArch, StackConfig, StackModel,
};
use uor_r4_training::stack_export::{export_stack, StackCalibration};

/// A small geometric stack of the given pattern, read and transport.
#[allow(clippy::too_many_arguments)]
fn small(
    pattern: &str,
    read: ReadScore,
    rotation: bool,
    width: usize,
    heads: usize,
    mlp: usize,
    context: usize,
    seed: u64,
) -> StackModel {
    small_with(
        pattern, read, rotation, width, heads, mlp, context, seed, None,
    )
}

/// [`small`] with an optional pointer-copy head.
#[allow(clippy::too_many_arguments)]
fn small_with(
    pattern: &str,
    read: ReadScore,
    rotation: bool,
    width: usize,
    heads: usize,
    mlp: usize,
    context: usize,
    seed: u64,
    pointer: Option<PointerConfig>,
) -> StackModel {
    let mut config = StackConfig::transformer_control(seed);
    config.arch = StackArch::Geometric;
    config.vocab_size = 96;
    config.width = width;
    config.heads = heads;
    config.mlp_hidden = mlp;
    config.context = context;
    config.pattern = pattern.to_owned();
    config.read = read;
    config.rotation = rotation;
    config.pointer = pointer;
    StackModel::new(config, &Device::Cpu).expect("small stack")
}

/// SplitMix64 draws.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `[-0.5, 0.5)`.
    fn centered(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64 - 0.5
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}

/// Perturb every tensor away from the initialization's identities (unit
/// rotations, zero ages, unit gains), then multiply the tensors whose name
/// ends with a listed suffix by its factor.
fn perturb(model: &StackModel, seed: u64, amplify: &[(&str, f32)]) {
    let mut rng = Rng(seed);
    for (name, var) in model.variables() {
        let values = var
            .as_tensor()
            .flatten_all()
            .expect("flatten")
            .to_vec1::<f32>()
            .expect("values");
        let scale = if name == "embedding.weight" {
            0.3
        } else if name.ends_with("norm.weight") {
            0.4
        } else if name.contains(".age") || name.contains("bias") || name.contains("offset") {
            0.5
        } else if name.contains("conv.weight")
            || name.contains("decay")
            || name.contains("log_beta")
        {
            0.3
        } else {
            0.05
        };
        let factor = amplify
            .iter()
            .find(|(suffix, _)| name.ends_with(suffix))
            .map_or(1.0, |(_, f)| *f);
        let changed: Vec<f32> = values
            .iter()
            .map(|v| (v + (rng.centered() * 2.0 * scale) as f32) * factor)
            .collect();
        var.set(&Tensor::from_vec(changed, var.as_tensor().shape(), &Device::Cpu).expect("tensor"))
            .expect("set");
    }
}

fn tokens(count: usize, vocab: u64, seed: u64) -> Vec<u32> {
    let mut rng = Rng(seed);
    (0..count).map(|_| rng.below(vocab) as u32).collect()
}

fn engines(bytes: &[u8]) -> (D10Model, IntegerStackModel) {
    let d10 = D10Model::from_artifact(StackArtifact::parse(bytes.to_vec()).expect("D10 parse"))
        .expect("D10 model");
    let d11 = IntegerStackModel::parse(bytes).expect("D11 model");
    (d10, d11)
}

/// Step both engines over every sequence (a fresh stream each, through
/// `reset`) and require identical logits, and identical pointer mixtures
/// where the artifact has a pointer head, at every position. Returns the
/// number of positions compared.
fn compare(label: &str, d10: &D10Model, d11: &IntegerStackModel, sequences: &[Vec<u32>]) -> usize {
    let mut s10 = d10.session();
    let mut s11 = d11.session();
    let mut positions = 0;
    for (n, sequence) in sequences.iter().enumerate() {
        s10.reset();
        s11.reset();
        for (t, &id) in sequence.iter().enumerate() {
            let want = s10.step(id).expect("D10 step").to_vec();
            let got = s11.step(id).expect("D11 step");
            if got != want.as_slice() {
                let first = got.iter().zip(&want).position(|(a, b)| a != b);
                panic!(
                    "{label}: sequence {n} position {t}: logits differ first at {first:?} ({:?} vs {:?})",
                    first.map(|i| got[i]),
                    first.map(|i| want[i])
                );
            }
            let (got, want) = (s11.mixture(), s10.mixture());
            assert_eq!(
                got.is_some(),
                want.is_some(),
                "{label}: pointer heads differ"
            );
            if let (Some(got), Some(want)) = (got, want) {
                if got != want {
                    let first = got.iter().zip(want).position(|(a, b)| a != b);
                    panic!(
                        "{label}: sequence {n} position {t}: mixtures differ first at {first:?} ({:?} vs {:?})",
                        first.map(|i| got[i]),
                        first.map(|i| want[i])
                    );
                }
                // A distribution: Q30 values that sum to 1 within the
                // per-id rounding (half a quantum each) and the reciprocals.
                let total: i64 = got.iter().map(|&v| i64::from(v)).sum();
                assert!(
                    got.iter().all(|&v| v >= 0),
                    "{label}: a negative probability"
                );
                assert!(
                    (total - (1 << 30)).abs() <= got.len() as i64 + (1 << 12),
                    "{label}: the mixture sums to {total}"
                );
                assert_eq!(s11.next_token_scores(), got);
            } else {
                assert_eq!(s11.next_token_scores(), s11.logits());
            }
            positions += 1;
        }
        assert_eq!(s11.position(), sequence.len());
    }
    positions
}

/// The sequences of one model: the full context twice (random ids, and a
/// short cycle repeated), a short stream and a single token.
fn sequences(context: usize, seed: u64) -> Vec<Vec<u32>> {
    let cycle: Vec<u32> = (0..context).map(|i| [5u32, 17, 5, 90][i % 4]).collect();
    vec![
        tokens(context, 96, seed),
        cycle,
        tokens(context / 3 + 1, 96, seed + 1),
        vec![95],
    ]
}

#[allow(clippy::too_many_arguments)]
fn check(
    pattern: &str,
    read: ReadScore,
    rotation: bool,
    width: usize,
    heads: usize,
    mlp: usize,
    context: usize,
    amplify: &[(&str, f32)],
    seed: u64,
) -> usize {
    let model = small(pattern, read, rotation, width, heads, mlp, context, seed);
    perturb(&model, seed + 100, amplify);
    let (bytes, _) =
        export_stack(&model, json!({"test": "d11-oracle"}), None, None).expect("export");
    let (d10, d11) = engines(&bytes);
    assert_eq!(d11.artifact_sha256(), d10.artifact_sha256());
    let label = format!("{pattern} {read:?} rotation={rotation} width={width} heads={heads}");
    compare(&label, &d10, &d11, &sequences(context, seed + 7))
}

#[test]
fn d11_logits_equal_d10_logits_on_random_stacks() {
    let mut positions = 0;
    for (pattern, read, rotation) in [
        ("rrar", ReadScore::Lorentz, true),
        ("rarr", ReadScore::Lorentz, true),
        ("rrar", ReadScore::Dot, true),
        ("rarr", ReadScore::Dot, false),
        ("rarr", ReadScore::Lorentz, false),
        ("ra", ReadScore::Dot, true),
        ("aa", ReadScore::Lorentz, false),
        ("r", ReadScore::Dot, true),
        ("rrar", ReadScore::L2, true),
        ("rarr", ReadScore::L2, false),
        ("aa", ReadScore::L2, true),
    ] {
        positions += check(pattern, read, rotation, 64, 2, 40, 24, &[], 3);
    }
    // A width that is not a power of two (three scale groups), six heads of 16
    // and the real model's pattern, over a longer context.
    positions += check("rrarra", ReadScore::Lorentz, true, 96, 6, 70, 40, &[], 11);
    positions += check("rrarra", ReadScore::Dot, true, 96, 3, 70, 40, &[], 12);
    positions += check("rrarra", ReadScore::L2, true, 96, 6, 70, 40, &[], 13);
    eprintln!("compared {positions} positions");
}

/// Amplified weights (read maps by 2^14 to 2^15, recurrence maps by 2^9 and
/// 2^12, the MLP up map by 2^6) move queries, keys, values and gates toward
/// the `i32` limit, where scores, rotations and Lorentz distances take
/// extreme values and the saturating conversions can engage. The D11 engine
/// must still agree exactly. (Which of the D10 engine's internal 64- or
/// 128-bit summation branches each position takes is not observed here; both
/// are exact.)
#[test]
fn d11_logits_equal_d10_logits_on_amplified_stacks() {
    let loud = [
        ("read.query.weight", 32768.0f32),
        ("read.key.weight", 32768.0),
        ("read.value.weight", 16384.0),
        ("rec.in.weight", 512.0),
        ("rec.gate.weight", 4096.0),
        ("mlp.up.weight", 64.0),
    ];
    let mut positions = 0;
    for (pattern, read, rotation) in [
        ("rarr", ReadScore::Lorentz, true),
        ("rarr", ReadScore::Dot, true),
        ("arra", ReadScore::Dot, false),
        ("rarr", ReadScore::L2, true),
        ("arra", ReadScore::L2, false),
    ] {
        positions += check(pattern, read, rotation, 64, 2, 40, 24, &loud, 21);
    }
    eprintln!("compared {positions} amplified positions");
}

/// A GPTQ-calibrated export (different nibbles and scales) and the retained
/// `StackCalibration` path.
#[test]
fn d11_logits_equal_d10_logits_on_a_calibrated_export() {
    let model = small("rarr", ReadScore::Lorentz, true, 64, 2, 40, 24, 5);
    perturb(&model, 55, &[]);
    let calibration =
        StackCalibration::collect(&model, &tokens(16 * 24, 96, 9), 8, 24).expect("calibration");
    let (bytes, report) =
        export_stack(&model, json!({}), Some((&calibration, 0.01)), None).expect("export");
    assert_eq!(report["method"]["quantizer"], "gptq");
    let (d10, d11) = engines(&bytes);
    compare("gptq", &d10, &d11, &sequences(24, 77));
}

/// An L2 artifact without its beta or offset table, a Dot artifact relabelled
/// `l2` (no scales) and an unknown read name are refused by both engines; an
/// L2 artifact never needs the arcosh table, and relabelling a Lorentz
/// artifact `l2` serves the L2 score in both engines alike.
#[test]
fn l2_reads_require_their_scale_and_offset_in_both_engines() {
    let d10_accepts = |bytes: &[u8]| {
        StackArtifact::parse(bytes.to_vec())
            .and_then(D10Model::from_artifact)
            .is_ok()
    };
    let model = small("rarr", ReadScore::L2, true, 64, 2, 40, 12, 9);
    perturb(&model, 99, &[]);
    let (valid, _) = export_stack(&model, json!({}), None, None).expect("L2 export");
    let (header, _) = split(&valid);
    assert_eq!(header["shape"]["read"], "l2");
    assert!(header["tables"]
        .as_array()
        .expect("tables")
        .iter()
        .all(|t| t["name"] != "arcosh"));
    let (d10, d11) = engines(&valid);
    compare("l2", &d10, &d11, &sequences(12, 5));
    let rename = |name: &'static str| {
        with_header(&valid, move |h| {
            entry(h, "tables", name)["name"] = json!(format!("{name}.gone"));
        })
    };
    let dot = small("rarr", ReadScore::Dot, true, 64, 2, 40, 12, 9);
    perturb(&dot, 99, &[]);
    let (dot_bytes, _) = export_stack(&dot, json!({}), None, None).expect("Dot export");
    for (label, bytes) in [
        ("l2 without beta", rename("l1.beta")),
        ("l2 without offset", rename("l1.offset")),
        (
            "dot relabelled l2",
            with_header(&dot_bytes, |h| h["shape"]["read"] = json!("l2")),
        ),
        (
            "unknown read",
            with_header(&valid, |h| h["shape"]["read"] = json!("euclidean")),
        ),
        (
            "upper-case read",
            with_header(&valid, |h| h["shape"]["read"] = json!("L2")),
        ),
    ] {
        assert!(
            IntegerStackModel::parse(&bytes).is_err(),
            "D11 accepted {label}"
        );
        assert!(!d10_accepts(&bytes), "D10 accepted {label}");
    }
    // A Lorentz artifact read as L2 (its beta and offset reused, its arcosh
    // table ignored) is a different model that both engines serve alike.
    let lorentz = small("rarr", ReadScore::Lorentz, true, 64, 2, 40, 12, 9);
    perturb(&lorentz, 99, &[]);
    let (lorentz_bytes, _) = export_stack(&lorentz, json!({}), None, None).expect("export");
    let relabelled = with_header(&lorentz_bytes, |h| h["shape"]["read"] = json!("l2"));
    let (d10, d11) = engines(&relabelled);
    compare("lorentz relabelled l2", &d10, &d11, &sequences(12, 6));
}

/// Both engines refuse a token outside the vocabulary and a step past the
/// context, and a reset session starts over bit-identically.
#[test]
fn sessions_refuse_bad_tokens_and_full_contexts() {
    let model = small("rarr", ReadScore::Dot, true, 64, 2, 40, 8, 4);
    perturb(&model, 44, &[]);
    let (bytes, _) = export_stack(&model, json!({}), None, None).expect("export");
    let (d10, d11) = engines(&bytes);
    let mut s10 = d10.session();
    let mut s11 = d11.session();
    assert!(s10.step(96).is_err());
    assert!(matches!(
        s11.step(96),
        Err(StackError::Token {
            token: 96,
            vocab: 96
        })
    ));
    let ids = tokens(8, 96, 1);
    let mut first = Vec::new();
    for &id in &ids {
        s10.step(id).expect("D10 step");
        first.push(s11.step(id).expect("D11 step").to_vec());
    }
    assert!(s10.step(0).is_err());
    assert!(matches!(
        s11.step(0),
        Err(StackError::ContextFull { context: 8 })
    ));
    s11.reset();
    for (&id, want) in ids.iter().zip(&first) {
        assert_eq!(s11.step(id).expect("D11 step after reset"), want.as_slice());
    }
}

// ---------------------------------------------------------------------------
// The pointer-copy head.

/// A perturbed stack with a pointer head of width `dim` and the given score,
/// its pointer maps amplified by `loud` so that the copy attention and the
/// gate are far from uniform, and its artifact.
#[allow(clippy::too_many_arguments)]
fn pointer_artifact(
    pattern: &str,
    read: ReadScore,
    score: ReadScore,
    dim: usize,
    context: usize,
    loud: f32,
    seed: u64,
) -> (StackModel, Vec<u8>) {
    let pointer = PointerConfig {
        score,
        ..PointerConfig::new(dim)
    };
    let model = small_with(pattern, read, true, 64, 2, 40, context, seed, Some(pointer));
    perturb(
        &model,
        seed + 100,
        &[
            ("pointer.query.weight", loud),
            ("pointer.key.weight", loud),
            ("pointer.gate.weight", loud),
        ],
    );
    let (bytes, _) =
        export_stack(&model, json!({"test": "d11-pointer"}), None, None).expect("pointer export");
    (model, bytes)
}

#[test]
fn d11_mixtures_equal_d10_mixtures_on_random_pointer_stacks() {
    let mut positions = 0;
    for (pattern, read, score, dim, loud) in [
        ("rarr", ReadScore::Dot, ReadScore::Dot, 8, 1.0f32),
        ("rarr", ReadScore::Lorentz, ReadScore::Dot, 32, 4.0),
        ("ra", ReadScore::Dot, ReadScore::Lorentz, 8, 4.0),
        ("rarr", ReadScore::L2, ReadScore::Lorentz, 16, 1.0),
        ("aa", ReadScore::Lorentz, ReadScore::Lorentz, 4, 16.0),
        ("r", ReadScore::Dot, ReadScore::Dot, 1, 64.0),
    ] {
        let (_, bytes) = pointer_artifact(pattern, read, score, dim, 24, loud, 31);
        let (d10, d11) = engines(&bytes);
        let shape = d11.pointer().expect("a pointer head");
        assert_eq!(shape.dim, dim);
        assert_eq!(shape.lorentz(), score == ReadScore::Lorentz);
        // The pointer maps are read once per token.
        let plain = small(pattern, read, true, 64, 2, 40, 24, 31);
        let (plain_bytes, _) = export_stack(&plain, json!({}), None, None).expect("export");
        let plain = IntegerStackModel::parse(&plain_bytes).expect("plain");
        assert_eq!(
            d11.weights_per_token(),
            plain.weights_per_token() + ((2 * dim + 1) * 64) as u64
        );
        let label = format!("pointer {pattern} {read:?} {score:?} dim {dim} x{loud}");
        positions += compare(&label, &d10, &d11, &sequences(24, 41));
    }
    eprintln!("compared {positions} pointer positions");
}

/// Pointer queries and keys near the `i32` limit (scores far past the exp
/// table's range, so the copy attention is one-hot) and a saturated gate.
#[test]
fn d11_mixtures_equal_d10_mixtures_on_amplified_pointers() {
    let mut positions = 0;
    for (score, loud) in [
        (ReadScore::Dot, 4096.0f32),
        (ReadScore::Dot, 65536.0),
        (ReadScore::Lorentz, 4096.0),
        (ReadScore::Lorentz, 65536.0),
    ] {
        let (_, bytes) = pointer_artifact("rarr", ReadScore::Dot, score, 16, 24, loud, 7);
        let (d10, d11) = engines(&bytes);
        positions += compare(
            &format!("amplified pointer {score:?} x{loud}"),
            &d10,
            &d11,
            &sequences(24, 3),
        );
    }
    eprintln!("compared {positions} amplified pointer positions");
}

/// A pointer session snapshot restores bit-identically (keys, lifts and the
/// pending mixture), and a plain model's snapshot carries no pointer state.
#[test]
fn a_pointer_session_restores_bit_identically() {
    for score in [ReadScore::Dot, ReadScore::Lorentz] {
        let (_, bytes) = pointer_artifact("rarr", ReadScore::Dot, score, 8, 16, 4.0, 13);
        let d11 = IntegerStackModel::parse(&bytes).expect("D11 model");
        let ids = tokens(16, 96, 2);
        let mut straight = d11.session();
        let mut want = Vec::new();
        for &id in &ids {
            straight.step(id).expect("step");
            want.push(straight.next_token_scores().to_vec());
        }
        let mut first = d11.session();
        for &id in &ids[..7] {
            first.step(id).expect("step");
        }
        let saved = first.save_state();
        assert_eq!(saved.pointer_keys.len(), 7 * 8);
        assert_eq!(saved.mixture.len(), 96);
        assert_eq!(
            saved.pointer_lifts.len(),
            if score == ReadScore::Lorentz { 7 } else { 0 }
        );
        let mut resumed = d11.session();
        resumed.restore_state(&saved).expect("restore");
        assert_eq!(resumed.next_token_scores(), want[6].as_slice());
        for (t, &id) in ids.iter().enumerate().skip(7) {
            resumed.step(id).expect("step");
            assert_eq!(
                resumed.next_token_scores(),
                want[t].as_slice(),
                "{score:?} {t}"
            );
        }
        // A snapshot with the wrong number of pointer keys is refused.
        let mut broken = saved.clone();
        broken.pointer_keys.pop();
        assert!(d11.session().restore_state(&broken).is_err());
        let mut broken = saved.clone();
        broken.mixture.clear();
        assert!(d11.session().restore_state(&broken).is_err());
    }
    let plain = small("rarr", ReadScore::Dot, true, 64, 2, 40, 8, 4);
    let (bytes, _) = export_stack(&plain, json!({}), None, None).expect("export");
    let d11 = IntegerStackModel::parse(&bytes).expect("D11 model");
    let mut session = d11.session();
    session.step(3).expect("step");
    let mut saved = session.save_state();
    assert!(saved.pointer_keys.is_empty() && saved.mixture.is_empty());
    saved.mixture = vec![0; 96];
    assert!(d11.session().restore_state(&saved).is_err());
}

/// A pointer model refuses a positive copy scale, whose logit boost the D10
/// comparator never applies before forming its mixture, and a snapshot that
/// carries one; zero is accepted and a refused session still matches D10. A
/// plain model accepts the scale.
#[test]
fn a_pointer_model_refuses_a_copy_scale() {
    let (_, bytes) = pointer_artifact("rarr", ReadScore::Dot, ReadScore::Dot, 8, 16, 4.0, 17);
    let (d10, d11) = engines(&bytes);
    let mut session = d11.session();
    for scale in [1, 1 << 16, i32::MAX] {
        assert!(matches!(
            session.set_copy_scale(scale),
            Err(StackError::CopyScaleWithPointer)
        ));
    }
    assert_eq!(session.copy_scale(), 0);
    session.set_copy_scale(0).expect("a zero scale");
    session
        .set_copy_scale(-5)
        .expect("a negative scale clamps to zero");
    assert_eq!(session.copy_scale(), 0);
    let ids = tokens(16, 96, 3);
    let mut s10 = d10.session();
    for &id in &ids {
        let want = s10.step(id).expect("D10 step").to_vec();
        assert_eq!(session.step(id).expect("D11 step"), want.as_slice());
        assert_eq!(session.mixture(), s10.mixture());
    }
    let mut saved = session.save_state();
    saved.copy_scale_q16 = 1 << 16;
    assert!(matches!(
        d11.session().restore_state(&saved),
        Err(StackError::CopyScaleWithPointer)
    ));

    let plain = small("rarr", ReadScore::Dot, true, 64, 2, 40, 8, 4);
    let (bytes, _) = export_stack(&plain, json!({}), None, None).expect("export");
    let d11 = IntegerStackModel::parse(&bytes).expect("D11 model");
    let mut session = d11.session();
    session.set_copy_scale(1 << 16).expect("a plain model");
    assert_eq!(session.copy_scale(), 1 << 16);
}

/// The two engines' pointer-width limits are one constant.
#[test]
fn both_engines_share_the_pointer_width_limit() {
    assert_eq!(
        uor_r4_lut::format::MAX_POINTER_DIM,
        uor_r4_integer::stack::MAX_POINTER_DIM
    );
}

/// A pointer artifact without one of its maps or tables, with an invalid
/// width, score or scale, or a Lorentz pointer without the arcosh table, is
/// refused by both engines; a plain artifact's shape has no `pointer` key.
#[test]
fn pointer_artifacts_require_their_maps_and_tables_in_both_engines() {
    let d10_accepts = |bytes: &[u8]| {
        StackArtifact::parse(bytes.to_vec())
            .and_then(D10Model::from_artifact)
            .is_ok()
    };
    let (_, dot) = pointer_artifact("rarr", ReadScore::Dot, ReadScore::Dot, 8, 12, 1.0, 5);
    let (_, lorentz) = pointer_artifact("rarr", ReadScore::Dot, ReadScore::Lorentz, 8, 12, 1.0, 5);
    let (header, _) = split(&dot);
    assert_eq!(header["shape"]["pointer"]["dim"], 8);
    assert_eq!(header["shape"]["pointer"]["score"], "dot");
    assert_eq!(
        header["shape"]["pointer"]["score_scale_q30"],
        (2f64.powi(30) / 8f64.sqrt()).round() as i64
    );
    // A Dot read with a Dot pointer has no arcosh table; a Lorentz pointer
    // brings it.
    let has_table = |header: &Value, name: &str| {
        header["tables"]
            .as_array()
            .expect("tables")
            .iter()
            .any(|t| t["name"] == name)
    };
    assert!(!has_table(&header, "arcosh") && !has_table(&header, "pointer_beta"));
    let (lorentz_header, _) = split(&lorentz);
    assert!(has_table(&lorentz_header, "arcosh") && has_table(&lorentz_header, "pointer_beta"));
    let plain = small("rarr", ReadScore::Dot, true, 64, 2, 40, 12, 5);
    let (plain_bytes, _) = export_stack(&plain, json!({}), None, None).expect("export");
    assert!(split(&plain_bytes).0["shape"].get("pointer").is_none());
    let rename = |bytes: &[u8], list: &'static str, name: &'static str| {
        with_header(bytes, move |h| {
            entry(h, list, name)["name"] = json!(format!("{name}.gone"));
        })
    };
    let cases: Vec<(&str, Vec<u8>)> = vec![
        ("no query", rename(&dot, "matrices", "pointer_query")),
        ("no key", rename(&dot, "matrices", "pointer_key")),
        ("no gate", rename(&dot, "matrices", "pointer_gate")),
        ("no gate bias", rename(&dot, "tables", "pointer_gate_bias")),
        ("no beta", rename(&lorentz, "tables", "pointer_beta")),
        ("no arcosh", rename(&lorentz, "tables", "arcosh")),
        (
            "dot relabelled lorentz",
            with_header(&dot, |h| h["shape"]["pointer"]["score"] = json!("lorentz")),
        ),
        (
            "an l2 pointer",
            with_header(&dot, |h| h["shape"]["pointer"]["score"] = json!("l2")),
        ),
        (
            "a wider pointer",
            with_header(&dot, |h| h["shape"]["pointer"]["dim"] = json!(9)),
        ),
        (
            "a zero-width pointer",
            with_header(&dot, |h| h["shape"]["pointer"]["dim"] = json!(0)),
        ),
        (
            "a pointer over 256",
            with_header(&dot, |h| h["shape"]["pointer"]["dim"] = json!(257)),
        ),
        (
            "a zero scale",
            with_header(&dot, |h| {
                h["shape"]["pointer"]["score_scale_q30"] = json!(0)
            }),
        ),
        (
            "a missing width",
            with_header(&dot, |h| {
                h["shape"]["pointer"]
                    .as_object_mut()
                    .expect("pointer")
                    .remove("dim");
            }),
        ),
        (
            "a non-positive beta",
            with_table_value(&lorentz, "pointer_beta", 0, &(-1100i16).to_le_bytes()),
        ),
    ];
    for (label, bytes) in &cases {
        let result = IntegerStackModel::parse(bytes);
        assert!(result.is_err(), "D11 accepted {label}");
        assert!(!d10_accepts(bytes), "D10 accepted {label}");
        eprintln!(
            "{label}: {}",
            result.err().map(|e| e.to_string()).unwrap_or_default()
        );
    }
    // The pointer schema is bound to the pointer record in both engines: a
    // pointer artifact relabelled with the plain schema (what an engine built
    // before the pointer port would read as a plain model) is refused, and so
    // is a pointer schema without its record.
    assert_eq!(split(&dot).0["schema"], "uor-r4.lut-stack/2");
    assert_eq!(split(&plain_bytes).0["schema"], "uor-r4.lut-stack/1");
    for (label, bytes) in [
        (
            "a pointer artifact under the plain schema",
            with_header(&dot, |h| h["schema"] = json!("uor-r4.lut-stack/1")),
        ),
        (
            "the pointer schema without its record",
            with_header(&dot, |h| {
                h["shape"].as_object_mut().expect("shape").remove("pointer");
            }),
        ),
    ] {
        assert!(
            IntegerStackModel::parse(&bytes).is_err(),
            "D11 accepted {label}"
        );
        assert!(!d10_accepts(&bytes), "D10 accepted {label}");
    }
    // Removing the pointer record and relabelling the plain schema serves the
    // plain model in both engines alike (its maps are then unused sections).
    let without = with_header(&dot, |h| {
        h["shape"].as_object_mut().expect("shape").remove("pointer");
        h["schema"] = json!("uor-r4.lut-stack/1");
    });
    let (d10, d11) = engines(&without);
    assert!(d11.pointer().is_none() && d10.pointer().is_none());
    compare("pointer record removed", &d10, &d11, &sequences(12, 8));
}

// ---------------------------------------------------------------------------
// Malformed containers.

/// The header and data of a container.
fn split(bytes: &[u8]) -> (Value, Vec<u8>) {
    let len = u64::from_le_bytes(bytes[8..16].try_into().expect("length")) as usize;
    let header: Value = serde_json::from_slice(&bytes[16..16 + len]).expect("header");
    let start = (16 + len).div_ceil(64) * 64;
    (header, bytes[start..].to_vec())
}

/// A container from a header and data (sections keep their data offsets).
fn join(header: &Value, data: &[u8]) -> Vec<u8> {
    let json = serde_json::to_vec(header).expect("json");
    let mut out = b"UORLUT01".to_vec();
    out.extend_from_slice(&(json.len() as u64).to_le_bytes());
    out.extend_from_slice(&json);
    out.resize(out.len().div_ceil(64) * 64, 0);
    out.extend_from_slice(data);
    out
}

fn with_header(bytes: &[u8], edit: impl FnOnce(&mut Value)) -> Vec<u8> {
    let (mut header, data) = split(bytes);
    edit(&mut header);
    join(&header, &data)
}

fn entry<'a>(header: &'a mut Value, list: &str, name: &str) -> &'a mut Value {
    header[list]
        .as_array_mut()
        .expect("list")
        .iter_mut()
        .find(|e| e["name"] == name)
        .expect("entry")
}

/// Overwrite the data of table `name` at element `index` (2-byte or 4-byte).
fn with_table_value(bytes: &[u8], name: &str, index: usize, value: &[u8]) -> Vec<u8> {
    let (mut header, mut data) = split(bytes);
    let t = entry(&mut header, "tables", name).clone();
    let at = t["span"]["offset"].as_u64().expect("offset") as usize + index * value.len();
    data[at..at + value.len()].copy_from_slice(value);
    join(&header, &data)
}

#[test]
fn malformed_containers_are_rejected() {
    let model = small("rarr", ReadScore::Lorentz, true, 64, 2, 40, 16, 8);
    perturb(&model, 88, &[]);
    let (valid, _) = export_stack(&model, json!({}), None, None).expect("export");
    assert!(IntegerStackModel::parse(&valid).is_ok());
    // A rebuilt container with an unchanged header is still valid.
    assert!(IntegerStackModel::parse(&with_header(&valid, |_| {})).is_ok());
    let d10_accepts = |bytes: &[u8]| {
        StackArtifact::parse(bytes.to_vec())
            .and_then(D10Model::from_artifact)
            .is_ok()
    };

    let mut bad_magic = valid.clone();
    bad_magic[0] = b'X';
    let mut too_long = valid.clone();
    too_long[8..16].copy_from_slice(&(valid.len() as u64).to_le_bytes());
    let mut huge = valid.clone();
    huge[8..16].copy_from_slice(&(1u64 << 40).to_le_bytes());
    let mut broken_json = valid.clone();
    broken_json[16] = b'[';
    let truncated = valid[..valid.len() - 1].to_vec();
    let mut cases: Vec<(&str, Vec<u8>, bool)> = vec![
        ("empty", Vec::new(), true),
        ("short", valid[..15].to_vec(), true),
        ("magic", bad_magic, true),
        ("header past the end", too_long, true),
        ("header over 64 MiB", huge, true),
        ("invalid JSON", broken_json, true),
        ("truncated data", truncated, true),
    ];
    let header_edits: Vec<(&str, Box<dyn Fn(&mut Value)>, bool)> = vec![
        (
            "schema",
            Box::new(|h: &mut Value| h["schema"] = json!("uor-r4.lut-llama/1")),
            true,
        ),
        (
            "group",
            Box::new(|h: &mut Value| h["group"] = json!(16)),
            true,
        ),
        (
            "pattern letter",
            Box::new(|h: &mut Value| h["shape"]["pattern"] = json!("rxrr")),
            true,
        ),
        (
            "read score",
            Box::new(|h: &mut Value| h["shape"]["read"] = json!("cosine")),
            true,
        ),
        (
            "zero vocabulary",
            Box::new(|h: &mut Value| h["shape"]["vocab"] = json!(0)),
            true,
        ),
        (
            "heads",
            Box::new(|h: &mut Value| h["shape"]["heads"] = json!(3)),
            true,
        ),
        (
            "context",
            Box::new(|h: &mut Value| h["shape"]["context"] = json!(17)),
            true,
        ),
        (
            "rotation",
            Box::new(|h: &mut Value| h["shape"]["rotation"] = json!(false)),
            true,
        ),
        (
            "missing field",
            Box::new(|h: &mut Value| {
                h["numerics"]
                    .as_object_mut()
                    .expect("numerics")
                    .remove("exp_step_log2");
            }),
            true,
        ),
        (
            "negative size",
            Box::new(|h: &mut Value| h["shape"]["mlp"] = json!(-64)),
            true,
        ),
        (
            "exp step",
            Box::new(|h: &mut Value| h["numerics"]["exp_step_log2"] = json!(0)),
            true,
        ),
        (
            "score scale",
            Box::new(|h: &mut Value| h["numerics"]["score_scale_q30"] = json!(0)),
            true,
        ),
        (
            "silu range",
            Box::new(|h: &mut Value| h["numerics"]["silu_range_log2"] = json!(5)),
            true,
        ),
        (
            "matrix offset",
            Box::new(|h: &mut Value| {
                entry(h, "matrices", "head")["nibbles"]["offset"] = json!(1u64 << 40);
            }),
            true,
        ),
        (
            "matrix size",
            Box::new(|h: &mut Value| {
                entry(h, "matrices", "l0.rec_out")["rows"] = json!(63);
            }),
            true,
        ),
        (
            "missing matrix",
            Box::new(|h: &mut Value| {
                entry(h, "matrices", "head")["name"] = json!("tail");
            }),
            true,
        ),
        (
            "duplicate matrix",
            Box::new(|h: &mut Value| {
                let copy = entry(h, "matrices", "head").clone();
                h["matrices"].as_array_mut().expect("matrices").push(copy);
            }),
            false,
        ),
        (
            "table kind",
            Box::new(|h: &mut Value| entry(h, "tables", "exp")["kind"] = json!("i32")),
            true,
        ),
        (
            "table length",
            Box::new(|h: &mut Value| {
                entry(h, "tables", "l0.conv_bias")["len"] = json!(63);
            }),
            true,
        ),
        (
            "missing table",
            Box::new(|h: &mut Value| {
                entry(h, "tables", "arcosh")["name"] = json!("cosh");
            }),
            true,
        ),
        (
            "matrix exponent",
            Box::new(|h: &mut Value| {
                entry(h, "matrices", "l1.query")["exp_base"] = json!(1 << 20);
            }),
            false,
        ),
        (
            "epsilon exponent",
            Box::new(|h: &mut Value| {
                h["numerics"]["rms_eps"]["exp"] = json!(i32::MIN);
            }),
            false,
        ),
    ];
    for (label, edit, d10_rejects) in &header_edits {
        cases.push((label, with_header(&valid, edit), *d10_rejects));
    }
    // Data edits: an invalid tap code, a non-positive rate, a decreasing
    // arcosh table and an increasing exp table (refused by D11 only).
    cases.push((
        "invalid grid code",
        with_table_value(&valid, "l0.conv_taps", 3, &5i16.to_le_bytes()),
        true,
    ));
    cases.push((
        "non-positive rate",
        with_table_value(&valid, "l2.decay_rate", 0, &(-1100i16).to_le_bytes()),
        true,
    ));
    cases.push((
        "arcosh order",
        with_table_value(&valid, "arcosh", 2000, &0u32.to_le_bytes()),
        true,
    ));
    cases.push((
        "exp order",
        with_table_value(&valid, "exp", 100, &u32::MAX.to_le_bytes()),
        false,
    ));
    for (label, bytes, d10_rejects) in &cases {
        let result = IntegerStackModel::parse(bytes);
        assert!(
            result.is_err(),
            "D11 accepted a malformed container: {label}"
        );
        if *d10_rejects {
            assert!(!d10_accepts(bytes), "D10 accepted {label}");
        }
        eprintln!(
            "{label}: {}",
            result.err().map(|e| e.to_string()).unwrap_or_default()
        );
    }
}

/// Random corruption never panics the loader; wherever both engines accept a
/// corrupted artifact (a corrupted weight or table entry), their logits agree.
#[test]
fn corrupted_artifacts_never_panic_and_accepted_ones_agree() {
    let model = small("rarr", ReadScore::Lorentz, true, 64, 2, 40, 12, 6);
    perturb(&model, 66, &[]);
    let (valid, _) = export_stack(&model, json!({}), None, None).expect("export");
    let header_len = u64::from_le_bytes(valid[8..16].try_into().expect("length")) as usize;
    let mut rng = Rng(1234);
    let (mut accepted, mut compared) = (0, 0);
    for trial in 0..300 {
        let mut bytes = valid.clone();
        let flips = 1 + rng.below(4) as usize;
        for _ in 0..flips {
            // Half the trials corrupt the header, half the data.
            let at = if trial % 2 == 0 {
                rng.below((16 + header_len) as u64) as usize
            } else {
                16 + header_len + rng.below((bytes.len() - 16 - header_len) as u64) as usize
            };
            bytes[at] ^= 1 << rng.below(8);
        }
        let Ok(d11) = IntegerStackModel::parse(&bytes) else {
            continue;
        };
        accepted += 1;
        let d10 = StackArtifact::parse(bytes.clone()).and_then(D10Model::from_artifact);
        let ids = tokens(12, 96, trial);
        match d10 {
            Ok(d10) => {
                compare(&format!("corruption {trial}"), &d10, &d11, &[ids]);
                compared += 1;
            }
            Err(_) => {
                // Accepted by D11 only: still no panic over a whole context.
                let mut session = d11.session();
                for &id in &ids {
                    session.step(id).expect("D11 step on a corrupted artifact");
                }
            }
        }
    }
    eprintln!("{accepted} corrupted artifacts accepted, {compared} compared with D10");
    assert!(
        compared > 0,
        "no corrupted artifact was accepted by both engines"
    );
}
