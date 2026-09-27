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

/// Read-score geometry. `Dot` is the retained scaled dot product. `Lorentz`
/// lifts query and key to the hyperboloid x -> (sqrt(1+|x|^2), x) and scores
/// the scaled geodesic distance below a learned radius. Offline F32 training
/// implements both; this runtime serves a quantized Lorentz model through the
/// integer kernel in `crate::lorentz` under [`packed_numerical_contract`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadGeometry {
    #[default]
    Dot,
    Lorentz,
}

impl ReadGeometry {
    pub const fn is_dot(&self) -> bool {
        matches!(self, Self::Dot)
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Dot => "dot",
            Self::Lorentz => "lorentz",
        }
    }
}

/// Learned scalar log scale of the Lorentz read; absent from `Dot` models.
pub const LORENTZ_LOG_BETA: &str = "read.lorentz_log_beta";
/// Learned scalar radius of the Lorentz read: keys closer than this geodesic
/// distance score above zero against NoRead. Absent from `Dot` models.
pub const LORENTZ_OFFSET: &str = "read.lorentz_offset";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct JointConfig {
    pub vocab_size: usize,
    pub width: usize,
    pub read_width: usize,
    pub context: usize,
    pub transport: Transport,
    pub seed: u64,
    /// Absent in retained metadata, which therefore reads as `Dot`. `Dot` is
    /// never serialized, so existing configs, manifests and hashes are unchanged.
    #[serde(default, skip_serializing_if = "ReadGeometry::is_dot")]
    pub read_geometry: ReadGeometry,
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
            read_geometry: ReadGeometry::Dot,
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
        let mut shapes = BTreeMap::from([
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
        ]);
        if self.read_geometry == ReadGeometry::Lorentz {
            // Two learned scalars; `Dot` keeps its exact retained inventory.
            shapes.insert(LORENTZ_LOG_BETA.into(), vec![1]);
            shapes.insert(LORENTZ_OFFSET.into(), vec![1]);
        }
        shapes
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuantizedTrainingState {
    pub start_step: usize,
    pub ramp_steps: usize,
    pub completed_step: usize,
    pub spec: QuantizationSpec,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preparation: Option<QuantizationPreparation>,
}

/// A separately calibrated shadow can enter alpha-only code learning without
/// claiming that the continuous model underwent a quantization training ramp.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuantizationPreparation {
    CalibratedForRounding,
}

pub fn valid_quantization_clock(
    start_step: usize,
    ramp_steps: usize,
    completed_step: usize,
    preparation: Option<QuantizationPreparation>,
) -> bool {
    ramp_steps > 0
        && completed_step >= start_step
        && (preparation.is_none() || (ramp_steps == 1 && completed_step == start_step))
}

/// Exact legacy contract serialized by the accepted packed exporter. Fixed JSON
/// preserves its scalar values without evaluating a floating-point expression.
pub fn quantized_numerical_contract() -> Result<serde_json::Value> {
    Ok(serde_json::from_str(LEGACY_CONTRACT)?)
}

/// Packed-model contract for `geometry`: the retained legacy contract, unchanged
/// for `Dot`, and with the quantized Lorentz read declaration for `Lorentz`.
pub fn packed_numerical_contract(geometry: ReadGeometry) -> Result<serde_json::Value> {
    let contract = quantized_numerical_contract()?;
    Ok(match geometry {
        ReadGeometry::Dot => contract,
        ReadGeometry::Lorentz => with_lorentz_quantized_read(contract),
    })
}

/// Replace the read declaration of a quantized contract with the Lorentz one.
/// Offline quantized training and this runtime bind the same fields.
pub fn with_lorentz_quantized_read(mut contract: serde_json::Value) -> serde_json::Value {
    if let (Some(target), serde_json::Value::Object(fields)) =
        (contract.as_object_mut(), lorentz_quantized_read())
    {
        target.extend(fields);
    }
    contract
}

fn lorentz_quantized_read() -> serde_json::Value {
    serde_json::json!({
        "read":"Query/Key from RMS-normalized provisional/written states, each lifted to the hyperboloid x0=sqrt(1+|x|^2); Value=tanh(affine(normalized written state)); score=exp(read.lorentz_log_beta)*(read.lorentz_offset-arcosh(max(q0*k0-<q,k>,1+2^-20)))+learned age at the affine score interface, competing with learned NoRead",
        "read_geometry":{
            "geometry":"lorentz",
            "schema":"uor-r4.joint-lorentz-read-quantized/1",
            "minimum_inner_product":"1+2^-20, the F32 value of the training clamp 1+1e-6",
            "distance":"arcosh(z)=ln(z+sqrt((z-1)(z+1)))",
            "scale":"exp(read.lorentz_log_beta); learned scalar initialized ln(sinh(offset0)/sqrt(r))",
            "offset":"read.lorentz_offset; learned scalar radius initialized arcosh(1+2*r*d/(r+d))",
            "parameters":"read.lorentz_log_beta and read.lorentz_offset are signed16 [-32767,32767] with one frozen ceiling dyadic scale each, like the additive parameters",
            "emulator":"F32 score from the dyadic scalars and the Q8 query/key codes, then the affine score interface",
            "integer_kernel":"With Q=|q|^2, K=|k|^2 and D=<q,k> in Q8 code units, P=(2^16+Q)(2^16+K) and M=2^16+D: z-1=(sqrt(P)-M)/2^16, evaluated as (P-M^2)/(2^16(sqrt(P)+M)) when M>0, where P-M^2 is an exact integer; one floor square root with 24 guard bits and one division rounded to nearest give z-1 at Q32, clamped below at 2^-20. arcosh(1+u) is read from the sealed Q24 table (argument codes below 2^10 directly, then 1024 linearly interpolated points per octave up to 2^64). exp(read.lorentz_log_beta) is evaluated once at load at Q32 (ln2 range reduction and a Q60 Taylor series) and must stay below 2^31. The score product rounds to Q40, adds age, then rounds and clips to the Q8 score interface",
            "admission":"Bounded candidate admission is unchanged; this score ranks admitted candidates only. Integer serving requires full admission",
            "scope":"Quantized training, packed F32 emulation and integer serving of the Lorentz read. Integer and F32 scores differ by F32 rounding and interface rounding; no bitwise agreement is claimed"
        }
    })
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
