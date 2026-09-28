//! Immutable import configuration for the retained packed artifact contract.
//! The legacy contract contains JSON numeric metadata; parsing/comparing it is
//! confined to loading. No legacy floating computation is used for inference.

use crate::format::QuantizationSpec;
use crate::{invalid, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Transport {
    Quaternion,
    HouseholderPair,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadMode {
    Enabled,
    NoRead,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct JointConfig {
    pub vocab_size: usize,
    pub width: usize,
    pub read_width: usize,
    pub context: usize,
    pub transport: Transport,
    pub seed: u64,
}
impl Default for JointConfig {
    fn default() -> Self {
        Self {
            vocab_size: 4096,
            width: 256,
            read_width: 64,
            context: 256,
            transport: Transport::Quaternion,
            seed: 0,
        }
    }
}
impl JointConfig {
    pub fn validate(&self) -> Result<()> {
        if self.vocab_size != 4096
            || !matches!(self.width, 128 | 256)
            || self.read_width != 64
            || !(2..=256).contains(&self.context)
        {
            return Err(invalid(
                "joint config requires vocabulary4096, width128/256, read_width64, context2..256",
            ));
        }
        Ok(())
    }

    pub fn shapes(&self) -> BTreeMap<String, Vec<usize>> {
        let d = self.width;
        let r = self.read_width;
        BTreeMap::from([
            ("embedding.weight".into(), vec![self.vocab_size, d]),
            ("recurrent.input.weight".into(), vec![3 * d, d]),
            ("recurrent.state.weight".into(), vec![3 * d, d]),
            ("recurrent.bias".into(), vec![3 * d]),
            ("read.query.weight".into(), vec![r, d]),
            ("read.query.bias".into(), vec![r]),
            ("read.key.weight".into(), vec![r, d]),
            ("read.key.bias".into(), vec![r]),
            ("read.value.weight".into(), vec![d, d]),
            ("read.value.bias".into(), vec![d]),
            ("read.age".into(), vec![self.context - 1]),
            ("read.no_read.weight".into(), vec![1, d]),
            ("read.no_read.bias".into(), vec![1]),
            ("update.weight".into(), vec![d, 2 * d]),
            ("update.bias".into(), vec![d]),
            ("update.gate.weight".into(), vec![1, 2 * d]),
            ("update.gate.bias".into(), vec![1]),
            ("copy.gate.weight".into(), vec![1, 2 * d]),
            ("copy.gate.bias".into(), vec![1]),
            ("output.norm.weight".into(), vec![d]),
            ("output.bias".into(), vec![self.vocab_size]),
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuantizedTrainingState {
    pub start_step: usize,
    pub ramp_steps: usize,
    pub completed_step: usize,
    pub spec: QuantizationSpec,
}

/// Exact legacy contract serialized by the accepted packed exporter. Fixed JSON
/// preserves its scalar values without evaluating a floating-point expression.
pub fn quantized_numerical_contract() -> Result<serde_json::Value> {
    Ok(serde_json::from_str(LEGACY_CONTRACT)?)
}

const LEGACY_CONTRACT: &str = r#"{
  "age_initialization": "-(occurrence age)/64; age starts1 for the immediately previous write",
  "arithmetic": "F32 offline continuous dense affine and softmax computation; approximate floating transport, not exact Z[phi] or target serving",
  "copy_initial_bias": -1.0,
  "householder_delta_scale": 0.07071067811865475,
  "output": "(1-g*(1-a0))*P_vocabulary + g*sum_previous(a_i*onehot(observed_token_i)), then declared uniform mixture",
  "quantization": {
    "artifact": "Packed codes plus integer scale exponents, no floating shadow dependency; load decodes dyadic values to F32 for this emulator; training checkpoint separately retains F32 shadows and Adam moments",
    "backward": "inclusive clipped straight-through gradient; identity inside representable range, zero outside at full strength; convex identity/hard ramp during training; hard path in every evaluation",
    "cache": "Quantized parameter tensors prepared once per full-window shard; incremental sessions snapshot them once; graph-retaining STE tensors are not cached across optimizer updates",
    "calibration": "4-bit row reconstruction MSE chooses among bounded ceil(log2(maxabs/7)) minus 2, minus 1, and ceiling; ties prefer larger exponent; additive16 uses ceiling. Scales never learn or recalibrate.",
    "fused_affine_qk_null_read_scores_and_logits": {
      "codes": [
        -32767,
        32767
      ],
      "exponent": -8
    },
    "parameters": "All multiplicative parameters, including embedding and output norm, signed 4-bit [-7,7]; additive biases/age signed 16-bit [-32767,32767]; frozen parent-calibrated power-of-two per-output-row scales (one scale for vectors)",
    "remaining_float": "F32 matmul/elementwise products and accumulation, RMS and unit normalization, sigmoid/tanh/softmax, probability mixture and sampling; fixed scalars are not yet integer-lowered. State-state and attention operations remain dense/full-window.",
    "rms_normalized_and_output_hidden": {
      "codes": [
        -32767,
        32767
      ],
      "exponent": -10
    },
    "round": "nearest, ties away from zero; canonical positive zero; clip before round",
    "schema": "uor-r4.joint-recurrent-quantized-interfaces/1",
    "sigmoid_gates": {
      "codes": [
        0,
        32768
      ],
      "exponent": -15
    },
    "state_transport_provisional_read_state": {
      "codes": [
        -32767,
        32767
      ],
      "exponent": -11
    },
    "tanh_values_candidates_updates_and_unit_transport": {
      "codes": [
        -32767,
        32767
      ],
      "exponent": -14
    },
    "transport": "Quantize signed unit coordinates after normalization; do not renormalize off-grid. Approximate quaternion and Householder orthogonality only; no hemisphere folding or H4 codebook."
  },
  "quaternion_delta_scale": 0.1,
  "quaternion_sign": "q and -q remain distinct; no hemisphere canonicalization",
  "read": "Query/Key from RMS-normalized provisional/written states; Value=tanh(affine(normalized written state)); score=dot/sqrt(read_width)+learned age, competing with learned NoRead",
  "retention_initialization": "Each R4 lane has one constant initial update-gate bias with log-spaced tau4..64; learned coordinate gates may subsequently differ",
  "rho_initial_bias": -0.5,
  "rms_epsilon": 1e-05,
  "session": "Same core with detached parameters/state, exact token/occurrence tape, bounded context without eviction",
  "training_credit": "Full differentiable unroll; no state or memory detach; labels only enter next-token NLL",
  "transport": "PerR4 lane q=normalize(e0+alpha*raw), left Hamilton multiplication; ordinary control H(v)H(e0), v=normalize(e0+alpha/sqrt2*raw)",
  "transport_min_norm": 1e-06,
  "transport_zero_rule": "Clamp squared denominator before sqrt; at norm<=minimum select identity e0 with finite excluded-branch arithmetic",
  "uniform_mixture_epsilon": 1e-08,
  "write_order": "Read previous occurrences, update state, write current observed token Key/Value for next step"
}"#;
