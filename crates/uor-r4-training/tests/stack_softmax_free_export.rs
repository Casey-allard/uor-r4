//! Softmax-free geometric stacks (flock rank reads, `ReadWeighting::Rank` and
//! `ReadWeighting::HammingRank`) export under schema `uor-r4.lut-stack/3` and
//! the D11 engine serves them close to the float model; a softmax read over a
//! flock is still refused, and a plain softmax stack exports as before.

use candle_core::{Device, Tensor};
use serde_json::{json, Value};
use uor_r4_integer::stack::IntegerStackModel;
use uor_r4_lut::format::StackArtifact;
use uor_r4_training::flock::FlockSelect;
use uor_r4_training::geometric_stack::{
    ReadScore, ReadWeighting, RotationGroup, StackArch, StackConfig, StackModel,
};
use uor_r4_training::stack_export::export_stack;

const VOCAB: usize = 96;
const CONTEXT: usize = 16;

/// A small geometric stack with two read layers and an optional flock
/// selection, its tensors moved deterministically off the initialization's
/// identities (random 4-bit-quantized weights, no training).
fn small(select: Option<FlockSelect>, seed: u64) -> StackModel {
    let mut config = StackConfig::transformer_control(seed);
    config.arch = StackArch::Geometric;
    config.vocab_size = VOCAB;
    config.width = 64;
    config.heads = 2;
    config.mlp_hidden = 40;
    config.context = CONTEXT;
    config.pattern = "rarr".to_owned();
    config.read = ReadScore::Dot;
    config.rotation = true;
    config.select = select;
    let model = StackModel::new(config, &Device::Cpu).expect("small stack");
    let mut state = seed;
    for (name, var) in model.variables() {
        let values = var
            .as_tensor()
            .flatten_all()
            .expect("flatten")
            .to_vec1::<f32>()
            .expect("values");
        let scale = if name == "embedding.weight" {
            0.3
        } else if name.ends_with("norm.weight") || name.contains("conv.weight") {
            0.3
        } else if name.contains(".age") || name.contains("bias") || name.contains("offset") {
            0.5
        } else {
            0.05
        };
        let changed: Vec<f32> = values
            .iter()
            .map(|v| v + (2.0 * scale * centered(&mut state)) as f32)
            .collect();
        var.set(&Tensor::from_vec(changed, var.as_tensor().shape(), &Device::Cpu).expect("tensor"))
            .expect("set");
    }
    model
}

/// SplitMix64, uniform in `[-0.5, 0.5)`.
fn centered(state: &mut u64) -> f64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64 - 0.5
}

fn header(bytes: &[u8]) -> Value {
    let len = u64::from_le_bytes(bytes[8..16].try_into().expect("length")) as usize;
    serde_json::from_slice(&bytes[16..16 + len]).expect("header")
}

/// NLL of `target` and the argmax of one row of logits.
fn score(logits: impl Iterator<Item = f64> + Clone, target: usize) -> (f64, usize) {
    let max = logits.clone().fold(f64::NEG_INFINITY, f64::max);
    let log_sum = logits.clone().map(|v| (v - max).exp()).sum::<f64>().ln() + max;
    let (mut best, mut best_value, mut nll) = (0, f64::NEG_INFINITY, 0.0);
    for (i, v) in logits.enumerate() {
        if v > best_value {
            (best, best_value) = (i, v);
        }
        if i == target {
            nll = log_sum - v;
        }
    }
    (nll, best)
}

/// Mean per-target NLL of the D11 engine and of the float model on the same
/// windows, and their top-1 agreement.
fn d11_and_float(model: &StackModel, d11: &IntegerStackModel) -> (f64, f64, f64, usize) {
    let mut state = 77u64;
    let (mut nll11, mut nllf, mut agree, mut targets) = (0.0, 0.0, 0usize, 0usize);
    let mut session = d11.session();
    for _ in 0..4 {
        let seq: Vec<u32> = (0..=CONTEXT)
            .map(|_| ((centered(&mut state) + 0.5) * VOCAB as f64) as u32)
            .collect();
        let (ids, next) = (&seq[..CONTEXT], &seq[1..]);
        let float = model
            .forward(ids, 1, CONTEXT)
            .expect("float forward")
            .to_vec2::<f32>()
            .expect("logits");
        session.reset();
        for (t, &id) in ids.iter().enumerate() {
            let logits = session.step(id).expect("D11 step");
            let target = next[t] as usize;
            let (n11, top11) = score(logits.iter().map(|&v| f64::from(v) / 65536.0), target);
            let (nf, topf) = score(float[t].iter().map(|&v| f64::from(v)), target);
            nll11 += n11;
            nllf += nf;
            agree += usize::from(top11 == topf);
            targets += 1;
        }
    }
    let n = targets as f64;
    (nll11 / n, nllf / n, agree as f64 / n, targets)
}

#[test]
fn softmax_free_stacks_export_under_schema_3_and_the_d11_engine_serves_them() {
    let select = FlockSelect {
        sink: 0,
        window: 2,
        k: 2,
    };
    for (weighting, binary) in [
        (ReadWeighting::Rank, false),
        (ReadWeighting::HammingRank, true),
    ] {
        let mut model = small(Some(select), 11);
        model.set_read_weighting(weighting).expect("weighting");
        let (bytes, _) = export_stack(&model, json!({"test": "softmax-free"}), None, None)
            .expect("a softmax-free read exports");
        let header = header(&bytes);
        assert_eq!(header["schema"], json!("uor-r4.lut-stack/3"));
        assert_eq!(header["shape"]["read_select"], json!({"window": 2, "k": 2}));
        assert_eq!(header["shape"]["read_weights"], json!("rank"));
        assert_eq!(
            header["shape"]["read_binary"],
            json!(binary),
            "{weighting:?} read_binary"
        );
        // The D10 comparator refuses schema /3 by design.
        assert!(StackArtifact::parse(bytes.clone()).is_err());
        let d11 = IntegerStackModel::parse(&bytes).expect("the D11 engine loads schema /3");
        assert_eq!(d11.shape().read_binary(), binary);
        let (nll11, nllf, agreement, targets) = d11_and_float(&model, &d11);
        eprintln!(
            "{weighting:?}: d11 nll {nll11:.4}, float nll {nllf:.4}, gap {:.4} nats, top-1 \
             agreement {agreement:.3} over {targets} targets",
            nll11 - nllf
        );
        assert!(
            (nll11 - nllf).abs() < 0.25,
            "{weighting:?}: D11 NLL {nll11} vs float {nllf}"
        );
    }
}

#[test]
fn a_softmax_read_over_a_flock_is_refused_and_a_plain_softmax_stack_exports_as_before() {
    let select = FlockSelect {
        sink: 0,
        window: 2,
        k: 2,
    };
    let flocked = small(Some(select), 12);
    assert_eq!(flocked.read_weighting(), ReadWeighting::Softmax);
    let refusal = export_stack(&flocked, json!({}), None, None).expect_err("softmax + flock");
    assert!(refusal.to_string().contains("flock selection"), "{refusal}");

    let plain = small(None, 12);
    let (bytes, _) = export_stack(&plain, json!({}), None, None).expect("plain export");
    let header = header(&bytes);
    assert_eq!(header["schema"], json!("uor-r4.lut-stack/1"));
    for field in ["read_select", "read_weights", "read_binary"] {
        assert!(header["shape"].get(field).is_none(), "{field} written");
    }
    // Both engines still serve it.
    StackArtifact::parse(bytes.clone()).expect("D10 parse");
    IntegerStackModel::parse(&bytes).expect("D11 parse");
}

#[test]
fn a_softmax_free_read_on_the_u1_transport_control_is_still_refused() {
    let mut config = StackConfig::transformer_control(5);
    config.arch = StackArch::Geometric;
    config.vocab_size = VOCAB;
    config.width = 64;
    config.heads = 2;
    config.mlp_hidden = 40;
    config.context = CONTEXT;
    config.pattern = "rarr".to_owned();
    config.read = ReadScore::Dot;
    config.rotation = true;
    config.rotation_group = RotationGroup::U1;
    config.select = Some(FlockSelect {
        sink: 0,
        window: 2,
        k: 2,
    });
    let mut model = StackModel::new(config, &Device::Cpu).expect("u1 stack");
    model
        .set_read_weighting(ReadWeighting::Rank)
        .expect("rank weighting");
    let refusal = export_stack(&model, json!({}), None, None).expect_err("u1 + rank");
    assert!(refusal.to_string().contains("U(1)"), "{refusal}");
}

#[test]
fn a_learned_rank_read_exports_its_tables_and_the_d11_engine_serves_them() {
    let select = FlockSelect {
        sink: 0,
        window: 2,
        k: 2,
    };
    let mut model = small(Some(select), 13);
    model
        .set_read_weighting(ReadWeighting::LearnedRank)
        .expect("learned rank");
    // A sharp learned profile, far from the fixed 1/(r+1) table.
    let side = select.window + select.k + 2;
    for (name, var) in model.variables() {
        if name.ends_with("read.rank_logits") {
            let heads = var.as_tensor().dims()[0];
            let values: Vec<f32> = (0..heads * side)
                .map(|i| {
                    if i % side == 0 {
                        4.0
                    } else {
                        -((i % side) as f32) * 0.5
                    }
                })
                .collect();
            var.set(&Tensor::from_vec(values, (heads, side), &Device::Cpu).expect("logits"))
                .expect("set logits");
        }
    }
    let (bytes, _) = export_stack(&model, json!({"test": "learned"}), None, None)
        .expect("a learned rank read exports");
    let header = header(&bytes);
    assert_eq!(header["schema"], json!("uor-r4.lut-stack/3"));
    assert_eq!(header["shape"]["read_weights"], json!("learned"));
    assert_eq!(header["shape"]["read_binary"], json!(false));
    assert!(StackArtifact::parse(bytes.clone()).is_err());
    let d11 = IntegerStackModel::parse(&bytes).expect("the D11 engine loads the learned tables");
    let (nll11, nllf, agreement, targets) = d11_and_float(&model, &d11);
    eprintln!(
        "LearnedRank: d11 nll {nll11:.4}, float nll {nllf:.4}, gap {:.4} nats, top-1 agreement \
         {agreement:.3} over {targets} targets",
        nll11 - nllf
    );
    assert!(
        (nll11 - nllf).abs() < 0.25,
        "LearnedRank: D11 NLL {nll11} vs float {nllf}"
    );
    // The same weights read as the fixed table differ: the learned table is served.
    model
        .set_read_weighting(ReadWeighting::Rank)
        .expect("fixed rank");
    let (fixed_bytes, _) =
        export_stack(&model, json!({"test": "learned"}), None, None).expect("fixed rank export");
    let fixed_d11 = IntegerStackModel::parse(&fixed_bytes).expect("fixed parse");
    let (fixed_nll, _, _, _) = d11_and_float(&model, &fixed_d11);
    assert!(
        (fixed_nll - nll11).abs() > 1e-6,
        "the learned and fixed tables served identically"
    );
}
