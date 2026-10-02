//! Offline learned K2 value head with a current-packed native q4 hard bridge.
//!
//! Token, actual retained own/next-neighbor roots, prior held-span roots and
//! explicit span validity score two root120/category32 atoms per value lane.
//! Four independent lanes preserve 16 coordinates/head. Context/span roots
//! supply finite features; they are not asserted to be value frames. Values
//! use the canonical fixed integer codec basis, with no donor vector or label
//! input. All floating factor operations below are offline training work.
//!
//! Hard earliest argmax chooses zero or one of 31 physical dyadic radii and a
//! signed root for each atom. The integer decoder checks the pair's Q16 sum;
//! overflow rejects the forward, without clipping or changing the alphabet.
//! Occurrence validity is explicit and separate from numerical zero. Missing
//! occurrences emit two ABSENT packets; valid zero emits PRESENT_ZERO packets.
//!
//! Backward is a declared biased first-order surrogate, not an argmax
//! derivative: each atom receives the full output adjoint, a root softmax over
//! actual decoded Q16 prototypes at its fixed hard radius, and a separate
//! local-radius softmax at its fixed hard root. No tangent projection discards
//! antipodal credit. Legacy category zero stops answer gradients. The explicit
//! q4 policy adds a bounded zero/anchor joint direction-radius escape bridge.
//! Both atoms initialize at nonzero radii; invalid occurrences always stop credit.
//!
//! The values Tensor preserves the training graph but converts decoded Q16 to
//! F32 (large integers need not roundtrip). Trace.values_q16 is authoritative
//! for integer callers. The strict typed-code forward delegates hard decisions
//! to the actual native producer; F32 scores are only its backward surrogate.
//! The standalone serializer retains head weights/config; full source-dependency
//! admission and persistent native artifact loading live in the compiler module.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use candle_core::{CpuStorage, CustomOp2, DType, Device, Layout, Shape, Tensor, Var};
use safetensors::{tensor::TensorView, Dtype as SafeDtype, SafeTensors};
use serde::{Deserialize, Serialize};
use uor_r4_integer::geometric_value::{NativeGeometricValues, ValuePacket, ValueState};
use uor_r4_integer::h4_tables::H4Code;

use crate::geometric_value_native::{ValuePacketRecord, ValuePacketStatus};
use crate::{invalid, sha256_bytes, Result};

pub const SCHEMA: &str = "uor-r4.geometric-value-producer-offline/1";
pub const Q4_NATIVE_CHOICE_SURROGATE: &str = "typed-native-state/old-held-codes;fresh-current-packed-q4-factor-tables;per-factor-Q24/i64-earliest-hard-root-category;integer-packet-Q16-forward;F32-logits-backward-probabilities-only;native-choice-conditioned-zero-anchor/local-neighborhood/1";
pub const Q4_SCHEMA: &str = "uor-r4.geometric-value-producer-offline/2";
pub const Q4_SURROGATE: &str = "nat-shadow-hard-quarter-grid-identity-STE;project[-1.75,1.75];positive-local-radius-plus-zero;zero-joint-root120-anchor17-13;invalid-stop;both-atoms-full-adjoint;first-order/1";
pub const VALUE_LANES: usize = 4;
pub const VALUE_WIDTH: usize = 16;
pub const ATOMS: usize = 2;
const ROOTS: usize = 120;
const CATEGORIES: usize = 32;
const RULE: &str = "token+retained-own+in-head-next-neighbor+valid-prior-span+span-valid-bias;value-lane-mod-latent-lanes;two-independent-atoms;earliest-argmax;checked-Q16-pair/1";
const SURROGATE: &str = "full-ambient-decoded-Q16-root-softmax-T1-at-fixed-hard-radius;local-present-radius-softmax-at-fixed-hard-root;zero-and-invalid-answer-stop;both-atoms-same-output-adjoint;first-order-only/1";
const INITIALIZATION: &str = "xorshift64-nonzero-uniform[-.02,.02];root-identity+.05;atom0-category17+1;atom1-category13+1/1";
const CATEGORY: &str = "0=PRESENT_ZERO;1..31=PRESENT_NONZERO-radius-bin=category-1;exponents[-16,14];invalid-occurrence=ABSENT/1";
const FAMILIES: [(&str, usize); 2] = [("root", ROOTS), ("category", CATEGORIES)];
const SOURCE_FILES: [&str; 2] = ["metadata.json", "value-parameters.safetensors"];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValueProducerConfig {
    pub schema: String,
    pub vocab_size: usize,
    pub heads: usize,
    pub latent_lanes_per_head: usize,
    pub seed: u64,
    pub codec_sha256: String,
    pub rule: String,
    pub surrogate: String,
    pub initialization: String,
    pub category_rule: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coefficient_policy: Option<String>,
}
impl ValueProducerConfig {
    fn shapes(&self) -> BTreeMap<String, Vec<usize>> {
        let mut shapes = BTreeMap::new();
        for (family, classes) in FAMILIES {
            shapes.insert(
                format!("token_{family}"),
                vec![self.vocab_size, self.heads, VALUE_LANES, ATOMS, classes],
            );
            for factor in ["own", "neighbor", "span"] {
                if factor == "neighbor" && self.latent_lanes_per_head == 1 {
                    continue;
                }
                shapes.insert(
                    format!("{factor}_{family}"),
                    vec![self.heads, VALUE_LANES, ATOMS, classes, 4],
                );
            }
            shapes.insert(
                format!("span_valid_{family}"),
                vec![self.heads, VALUE_LANES, ATOMS, classes],
            );
        }
        shapes
    }
}

pub struct ValueProducerWeights {
    config: ValueProducerConfig,
    parameters: BTreeMap<String, Var>,
    codec: Arc<NativeGeometricValues>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValueProducerTrace {
    pub batch: usize,
    pub heads: usize,
    pub time: usize,
    /// Structural occurrence validity [B,T], never inferred from coordinates.
    pub occurrence_valid: Vec<bool>,
    /// Two primitive packets per lane, ordered [B,H,T,4].
    pub packets: Vec<[ValuePacketRecord; ATOMS]>,
    /// Checked integer sums [B,H,T,16], before F32 reconstruction.
    pub values_q16: Vec<i32>,
}
pub struct ValueProducerOutput {
    /// Differentiable offline F32 reconstruction [B,H,T,16].
    pub values: Tensor,
    /// [B,T,H,4,2,120] and [B,T,H,4,2,32], respectively.
    pub root_logits: Tensor,
    pub category_logits: Tensor,
    pub trace: ValueProducerTrace,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceMetadata {
    config: ValueProducerConfig,
    parameter_bytes: usize,
    parameter_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    packed_sha256: Option<String>,
}

impl ValueProducerWeights {
    pub fn new(
        vocab_size: usize,
        heads: usize,
        latent_lanes_per_head: usize,
        seed: u64,
    ) -> Result<Self> {
        if !(1..=4096).contains(&vocab_size)
            || !(1..=2).contains(&heads)
            || !(1..=4).contains(&latent_lanes_per_head)
        {
            return Err(invalid(
                "value producer needs vocab1..4096, heads1..2, latent lanes1..4",
            ));
        }
        let codec =
            Arc::new(NativeGeometricValues::canonical().map_err(|e| invalid(e.to_string()))?);
        let config = ValueProducerConfig {
            schema: SCHEMA.into(),
            vocab_size,
            heads,
            latent_lanes_per_head,
            seed,
            codec_sha256: sha256_bytes(&codec.to_bytes()),
            rule: RULE.into(),
            surrogate: SURROGATE.into(),
            initialization: INITIALIZATION.into(),
            category_rule: CATEGORY.into(),
            coefficient_policy: None,
        };
        let mut rng = if seed == 0 { 0x9e3779b97f4a7c15 } else { seed };
        let mut draw = || {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let x = ((rng >> 40) as f32 / 16_777_216. - 0.5) * 0.04;
            if x == 0. {
                0.001
            } else {
                x
            }
        };
        let mut parameters = BTreeMap::new();
        for (name, shape) in config.shapes() {
            let mut values = (0..shape.iter().product())
                .map(|_| draw())
                .collect::<Vec<_>>();
            if name == "token_root" {
                for row in values.chunks_exact_mut(ROOTS) {
                    row[1] += 0.05;
                }
            } else if name == "token_category" {
                for (atom, row) in values.chunks_exact_mut(CATEGORIES).enumerate() {
                    row[if atom % ATOMS == 0 { 17 } else { 13 }] += 1.;
                }
            }
            parameters.insert(name, Var::from_vec(values, shape, &Device::Cpu)?);
        }
        Ok(Self {
            config,
            parameters,
            codec,
        })
    }
    /// Explicit new training/source policy. Consumes the imported head; saved
    /// legacy artifacts are never reinterpreted. Only the range projection changes
    /// its parameter values; the hard q4 grid is applied on each forward/export.
    pub fn into_q4(mut self) -> Result<Self> {
        self.validate_parameters()?;
        self.config.schema = Q4_SCHEMA.into();
        self.config.coefficient_policy = Some(uor_r4_integer::geometric_value_q4::POLICY.into());
        self.config.surrogate = Q4_SURROGATE.into();
        self.project_shadow_range()?;
        Ok(self)
    }
    pub fn is_q4(&self) -> bool {
        self.config.schema == Q4_SCHEMA
    }
    pub fn packed_coefficients(&self) -> Result<Vec<u8>> {
        if !self.is_q4() {
            return Err(invalid("legacy value coefficients are not strict q4"));
        }
        self.validate_parameters()?;
        let c = uor_r4_integer::geometric_value_q4::ValueQ4Config {
            vocab_size: self.config.vocab_size,
            heads: self.config.heads,
            latent_lanes_per_head: self.config.latent_lanes_per_head,
        };
        let mut q = Vec::new();
        for (name, _) in c.coefficient_shapes().map_err(|e| invalid(e.to_string()))? {
            q.extend(
                self.tensor(&name)?
                    .flatten_all()?
                    .to_vec1::<f32>()?
                    .into_iter()
                    .map(|w| (w * 4.).round() as i8),
            );
        }
        uor_r4_integer::geometric_value_q4::pack_coefficients(&q)
            .map_err(|e| invalid(e.to_string()))
    }
    pub fn project_shadow_range(&self) -> Result<()> {
        if !self.is_q4() {
            return Err(invalid(
                "value shadow projection requires explicit q4 policy",
            ));
        }
        // Validate every variable before mutating any of them.
        let mut projected = Vec::new();
        for (name, shape) in self.config.shapes() {
            let tensor = self.tensor(&name)?;
            finite_tensor(tensor, &shape)?;
            let values = tensor
                .flatten_all()?
                .to_vec1::<f32>()?
                .into_iter()
                .map(|v| v.clamp(-1.75, 1.75))
                .collect::<Vec<_>>();
            projected.push((name, Tensor::from_vec(values, shape, &Device::Cpu)?));
        }
        for (name, tensor) in projected {
            self.parameters[&name].set(&tensor)?;
        }
        Ok(())
    }
    fn coefficient(&self, name: &str) -> Result<Tensor> {
        let shadow = self.tensor(name)?;
        if !self.is_q4() {
            return Ok(shadow.clone());
        }
        let hard = Tensor::from_vec(
            shadow
                .flatten_all()?
                .to_vec1::<f32>()?
                .into_iter()
                .map(|v| (v * 4.).round() * 0.25)
                .collect::<Vec<_>>(),
            shadow.shape(),
            &Device::Cpu,
        )?;
        // Nat/logit-valued shadows: slope one, explicitly ignoring grid rounding.
        Ok((&hard + (shadow - shadow.detach())?)?)
    }
    pub fn config(&self) -> &ValueProducerConfig {
        &self.config
    }
    pub fn parameters(&self) -> &BTreeMap<String, Var> {
        &self.parameters
    }

    /// Exclusive standalone offline head snapshot. The enclosing fit receipt
    /// must additionally bind its actual context/base/tokenizer/data inputs.
    /// This does not export or admit a compiled native serving producer.
    pub fn save_source(&self, directory: &Path) -> Result<()> {
        self.validate_parameters()?;
        let bytes = self
            .parameters
            .iter()
            .map(|(name, value)| {
                Ok((
                    name.clone(),
                    value
                        .flatten_all()?
                        .to_vec1::<f32>()?
                        .into_iter()
                        .flat_map(f32::to_le_bytes)
                        .collect::<Vec<_>>(),
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        let views = self
            .config
            .shapes()
            .into_iter()
            .map(|(name, shape)| {
                Ok((
                    name.clone(),
                    TensorView::new(SafeDtype::F32, shape, &bytes[&name])?,
                ))
            })
            .collect::<std::result::Result<Vec<_>, safetensors::SafeTensorError>>()?;
        let parameters = safetensors::serialize(views, None)?;
        let metadata = serde_json::to_vec_pretty(&SourceMetadata {
            config: self.config.clone(),
            parameter_bytes: parameters.len(),
            parameter_sha256: sha256_bytes(&parameters),
            packed_sha256: if self.is_q4() {
                Some(sha256_bytes(&self.packed_coefficients()?))
            } else {
                None
            },
        })?;
        fs::create_dir(directory)?;
        fs::File::create_new(directory.join(SOURCE_FILES[0]))?.write_all(&metadata)?;
        fs::File::create_new(directory.join(SOURCE_FILES[1]))?.write_all(&parameters)?;
        Ok(())
    }

    pub fn load_source(directory: &Path) -> Result<Self> {
        let mut files = BTreeSet::new();
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                return Err(invalid("value source has nonregular file"));
            }
            files.insert(
                entry
                    .file_name()
                    .into_string()
                    .map_err(|_| invalid("value source non-UTF8 filename"))?,
            );
        }
        if files != SOURCE_FILES.iter().map(|x| x.to_string()).collect() {
            return Err(invalid("value source file set differs"));
        }
        let metadata: SourceMetadata =
            serde_json::from_slice(&fs::read(directory.join(SOURCE_FILES[0]))?)?;
        let weights = Self::new(
            metadata.config.vocab_size,
            metadata.config.heads,
            metadata.config.latent_lanes_per_head,
            metadata.config.seed,
        )?;
        let weights = match metadata.config.schema.as_str() {
            SCHEMA => weights,
            Q4_SCHEMA => weights.into_q4()?,
            _ => return Err(invalid("unknown value source schema")),
        };
        if weights.config != metadata.config {
            return Err(invalid(
                "value source configuration/codec/surrogate differs",
            ));
        }
        let bytes = fs::read(directory.join(SOURCE_FILES[1]))?;
        if bytes.len() != metadata.parameter_bytes
            || sha256_bytes(&bytes) != metadata.parameter_sha256
        {
            return Err(invalid("value source parameter identity differs"));
        }
        let archive = SafeTensors::deserialize(&bytes)?;
        let shapes = weights.config.shapes();
        if archive
            .names()
            .iter()
            .map(|x| x.to_string())
            .collect::<BTreeSet<_>>()
            != shapes.keys().cloned().collect()
        {
            return Err(invalid("value source parameter names differ"));
        }
        for (name, shape) in shapes {
            let view = archive.tensor(&name)?;
            if view.dtype() != SafeDtype::F32
                || view.shape() != shape
                || view.data().len() != shape.iter().product::<usize>() * 4
            {
                return Err(invalid("value source parameter shape/dtype/size differs"));
            }
            let values = view
                .data()
                .chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect::<Vec<_>>();
            weights.parameters[&name].set(&Tensor::from_vec(values, shape, &Device::Cpu)?)?;
        }
        weights.validate_parameters()?;
        let packed_hash = if weights.is_q4() {
            Some(sha256_bytes(&weights.packed_coefficients()?))
        } else {
            None
        };
        if metadata.packed_sha256 != packed_hash {
            return Err(invalid("value source packed coefficient identity differs"));
        }
        Ok(weights)
    }

    fn tensor(&self, name: &str) -> Result<&Tensor> {
        self.parameters
            .get(name)
            .map(Var::as_tensor)
            .ok_or_else(|| invalid(format!("missing value producer parameter {name}")))
    }
    fn validate_parameters(&self) -> Result<()> {
        for (name, shape) in self.config.shapes() {
            let tensor = self.tensor(&name)?;
            finite_tensor(tensor, &shape)?;
            if self.is_q4()
                && tensor
                    .flatten_all()?
                    .to_vec1::<f32>()?
                    .iter()
                    .any(|x| x.abs() > 1.75)
            {
                return Err(invalid(
                    "q4 value shadows exceed fixed [-1.75,1.75] range; project after updates",
                ));
            }
        }
        Ok(())
    }

    /// Serving-identical hard packet decisions from the CURRENT live q4 grid.
    /// Authoritative inputs are native code IDs, not a nearest-root recovery from
    /// floats. A new native producer is constructed per call; no compiled snapshot
    /// can silently survive an optimizer update. Its argmax is detached, while
    /// current F32 factor logits supply only the declared backward probabilities.
    #[allow(clippy::too_many_arguments)]
    pub fn forward_q4_codes(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        latent: &[H4Code],
        held: &[H4Code],
        span_valid: &[bool],
        occurrence_valid: &[bool],
    ) -> Result<ValueProducerOutput> {
        if !self.is_q4() {
            return Err(invalid(
                "native-choice value bridge requires strict q4 source",
            ));
        }
        self.validate_parameters()?;
        let rows = batch
            .checked_mul(time)
            .ok_or_else(|| invalid("native-choice value row overflow"))?;
        let lanes = self.config.heads * self.config.latent_lanes_per_head;
        let count = rows
            .checked_mul(lanes)
            .ok_or_else(|| invalid("native-choice feature overflow"))?;
        if batch == 0
            || time == 0
            || ids.len() != rows
            || latent.len() != count
            || held.len() != count
            || span_valid.len() != rows
            || occurrence_valid.len() != rows
            || ids.iter().any(|&id| id as usize >= self.config.vocab_size)
        {
            return Err(invalid(
                "native-choice value code/token/validity shape differs",
            ));
        }
        let config = uor_r4_integer::geometric_value_q4::ValueQ4Config {
            vocab_size: self.config.vocab_size,
            heads: self.config.heads,
            latent_lanes_per_head: self.config.latent_lanes_per_head,
        };
        let native = uor_r4_integer::geometric_value_q4::NativeValueQ4::new(
            config,
            &self.packed_coefficients()?,
        )
        .and_then(|q| q.into_native())
        .map_err(|e| invalid(e.to_string()))?;
        let slots = self.config.heads * VALUE_LANES * ATOMS;
        let choice_count = rows
            .checked_mul(slots)
            .ok_or_else(|| invalid("native-choice value count overflow"))?;
        let mut roots = Vec::with_capacity(choice_count);
        let mut categories = Vec::with_capacity(choice_count);
        for row in 0..rows {
            let first = row * lanes;
            let produced = native
                .produce(
                    ids[row] as usize,
                    &latent[first..first + lanes],
                    if span_valid[row] {
                        Some(&held[first..first + lanes])
                    } else {
                        None
                    },
                    occurrence_valid[row],
                )
                .map_err(|e| invalid(e.to_string()))?;
            // Raw choices preserve the selected root even when category0 renders
            // a canonical PRESENT_ZERO packet. They also condition the backward.
            roots.extend_from_slice(&produced.root_choices[..slots]);
            categories.extend_from_slice(&produced.categories[..slots]);
        }
        let basis = uor_r4_integer::geometric_value_q4::canonical_basis_q25();
        let feature = |codes: &[H4Code]| -> Result<Tensor> {
            Ok(Tensor::from_vec(
                codes
                    .iter()
                    .flat_map(|code| basis[usize::from(code.index())].map(|x| x as f32 / 33554432.))
                    .collect::<Vec<_>>(),
                vec![
                    batch,
                    time,
                    self.config.heads,
                    self.config.latent_lanes_per_head,
                    4,
                ],
                &Device::Cpu,
            )?)
        };
        self.forward_with_choices(
            ids,
            &feature(latent)?,
            &feature(held)?,
            span_valid,
            occurrence_valid,
            Some(NativeValueChoices { roots, categories }),
        )
    }

    /// Inputs are actual predicted causal states and explicit structural flags.
    /// Both root tensors use [B,T,H,L,4]. A false span flag masks its feature;
    /// it does not make the numerical source occurrence absent. This pointwise
    /// head performs no look-ahead and does not accept teacher/donor vectors.
    /// This preserves the historical F32 hard-score selector, including the first
    /// q4 projection comparison. It cannot establish native q4 packet parity;
    /// actual strict training callers use `forward_q4_codes` instead.
    pub fn forward(
        &self,
        ids: &[u32],
        latent_roots: &Tensor,
        held_span: &Tensor,
        span_valid: &[bool],
        occurrence_valid: &[bool],
    ) -> Result<ValueProducerOutput> {
        self.forward_with_choices(
            ids,
            latent_roots,
            held_span,
            span_valid,
            occurrence_valid,
            None,
        )
    }

    fn forward_with_choices(
        &self,
        ids: &[u32],
        latent_roots: &Tensor,
        held_span: &Tensor,
        span_valid: &[bool],
        occurrence_valid: &[bool],
        native_choices: Option<NativeValueChoices>,
    ) -> Result<ValueProducerOutput> {
        self.validate_parameters()?;
        let shape = latent_roots.dims();
        if shape.len() != 5
            || shape[0] == 0
            || shape[1] == 0
            || shape[2] != self.config.heads
            || shape[3] != self.config.latent_lanes_per_head
            || shape[4] != 4
        {
            return Err(invalid(
                "value producer needs latent roots[B,T,H,L,4] matching config",
            ));
        }
        let (batch, time, heads, lanes) = (shape[0], shape[1], shape[2], shape[3]);
        let rows = batch
            .checked_mul(time)
            .ok_or_else(|| invalid("value producer row overflow"))?;
        if ids.len() != rows
            || span_valid.len() != rows
            || occurrence_valid.len() != rows
            || ids.iter().any(|&id| id as usize >= self.config.vocab_size)
        {
            return Err(invalid(
                "value producer token/validity shape or token range differs",
            ));
        }
        finite_tensor(latent_roots, shape)?;
        finite_tensor(held_span, shape)?;
        let validity = Tensor::from_vec(
            span_valid
                .iter()
                .map(|&x| if x { 1f32 } else { 0. })
                .collect(),
            (rows, 1),
            &Device::Cpu,
        )?;
        let token_ids = Tensor::from_vec(ids.to_vec(), rows, &Device::Cpu)?;
        let slots = heads * VALUE_LANES * ATOMS;
        let mut logits = Vec::new();
        for (family, classes) in FAMILIES {
            let tokens = self
                .coefficient(&format!("token_{family}"))?
                .reshape((self.config.vocab_size, slots * classes))?
                .index_select(&token_ids, 0)?;
            let mut pieces = Vec::new();
            for head in 0..heads {
                for value_lane in 0..VALUE_LANES {
                    let own_lane = value_lane % lanes;
                    let feature = |tensor: &Tensor, lane| -> Result<Tensor> {
                        Ok(tensor
                            .narrow(2, head, 1)?
                            .narrow(3, lane, 1)?
                            .reshape((rows, 4))?)
                    };
                    let own = feature(latent_roots, own_lane)?;
                    let neighbor = if lanes > 1 {
                        Some(feature(latent_roots, (own_lane + 1) % lanes)?)
                    } else {
                        None
                    };
                    let span = feature(held_span, own_lane)?.broadcast_mul(&validity)?;
                    for atom in 0..ATOMS {
                        let slot = (head * VALUE_LANES + value_lane) * ATOMS + atom;
                        let mut score = tokens.narrow(1, slot * classes, classes)?;
                        for (factor, input) in [
                            ("own", Some(&own)),
                            ("neighbor", neighbor.as_ref()),
                            ("span", Some(&span)),
                        ] {
                            if let Some(input) = input {
                                let basis = self
                                    .coefficient(&format!("{factor}_{family}"))?
                                    .reshape((slots, classes, 4))?
                                    .narrow(0, slot, 1)?
                                    .squeeze(0)?;
                                score = (&score + input.matmul(&basis.t()?)?)?;
                            }
                        }
                        let bias = self
                            .coefficient(&format!("span_valid_{family}"))?
                            .reshape((slots, classes))?
                            .narrow(0, slot, 1)?;
                        score = (&score + validity.broadcast_mul(&bias)?)?;
                        pieces.push(score);
                    }
                }
            }
            logits.push(
                Tensor::cat(&pieces, 1)?
                    .reshape(vec![batch, time, heads, VALUE_LANES, ATOMS, classes])?
                    .contiguous()?,
            );
        }
        let root_logits = logits.remove(0);
        let category_logits = logits.remove(0);
        let op = ValueEmit {
            batch,
            time,
            heads,
            occurrence_valid: occurrence_valid.to_vec(),
            q4_zero_escape: self.is_q4(),
            native_choices,
            codec: Arc::clone(&self.codec),
        };
        let trace = op.trace(
            &root_logits.flatten_all()?.to_vec1::<f32>()?,
            &category_logits.flatten_all()?.to_vec1::<f32>()?,
        )?;
        let values = root_logits.apply_op2(&category_logits, op)?;
        Ok(ValueProducerOutput {
            values,
            root_logits,
            category_logits,
            trace,
        })
    }
}

fn finite_tensor(tensor: &Tensor, shape: &[usize]) -> Result<()> {
    if tensor.dims() != shape
        || tensor.dtype() != DType::F32
        || !tensor.device().is_cpu()
        || tensor
            .flatten_all()?
            .to_vec1::<f32>()?
            .iter()
            .any(|x| !x.is_finite())
    {
        return Err(invalid(
            "value producer requires matching finite CPU F32 tensor",
        ));
    }
    Ok(())
}
fn best(values: &[f32]) -> usize {
    let mut chosen = 0;
    for i in 1..values.len() {
        if values[i] > values[chosen] {
            chosen = i;
        }
    }
    chosen
}
fn probabilities(values: &[f32]) -> Vec<f64> {
    let maximum = values.iter().copied().fold(f32::NEG_INFINITY, f32::max) as f64;
    let mut p: Vec<_> = values
        .iter()
        .map(|&v| (f64::from(v) - maximum).exp())
        .collect();
    let total = p.iter().sum::<f64>();
    for value in &mut p {
        *value /= total;
    }
    p
}
fn packet(root: usize, category: usize) -> candle_core::Result<ValuePacket> {
    if category == 0 {
        Ok(ValuePacket::present_zero())
    } else {
        ValuePacket::present_nonzero(root as u8, (category - 1) as u8)
            .map_err(|e| candle_core::Error::Msg(e.to_string()))
    }
}
fn record(packet: ValuePacket) -> ValuePacketRecord {
    ValuePacketRecord {
        status: match packet.state() {
            ValueState::Absent => ValuePacketStatus::Absent,
            ValueState::PresentZero => ValuePacketStatus::PresentZero,
            ValueState::PresentNonzero => ValuePacketStatus::PresentNonzero,
        },
        root: packet.root().index(),
        radius_bin: packet.radius_bin(),
    }
}
fn checked_f32(values: Vec<f64>) -> candle_core::Result<Vec<f32>> {
    if values
        .iter()
        .any(|&v| !v.is_finite() || !((v as f32).is_finite()))
    {
        candle_core::bail!("nonfinite value producer gradient");
    }
    Ok(values.into_iter().map(|x| x as f32).collect())
}
fn contiguous<'a>(storage: &'a CpuStorage, layout: &Layout) -> candle_core::Result<&'a [f32]> {
    let values = storage.as_slice::<f32>()?;
    match layout.contiguous_offsets() {
        Some((start, end)) => Ok(&values[start..end]),
        None => candle_core::bail!("value emission requires contiguous F32"),
    }
}

// Flattened [B,T,H,4,2], matching the differentiable score tensors. These
// decisions are private, computed only from current packed coefficients and
// authoritative native inputs; they are never teacher labels.
struct NativeValueChoices {
    roots: Vec<u8>,
    categories: Vec<u8>,
}
struct ValueEmit {
    batch: usize,
    time: usize,
    heads: usize,
    occurrence_valid: Vec<bool>,
    q4_zero_escape: bool,
    native_choices: Option<NativeValueChoices>,
    codec: Arc<NativeGeometricValues>,
}
impl ValueEmit {
    fn atom(&self, b: usize, t: usize, h: usize, l: usize, a: usize) -> usize {
        ((((b * self.time + t) * self.heads + h) * VALUE_LANES + l) * ATOMS) + a
    }
    fn validate(&self, roots: &[f32], categories: &[f32]) -> candle_core::Result<()> {
        let atoms = self.batch * self.time * self.heads * VALUE_LANES * ATOMS;
        if roots.len() != atoms * ROOTS
            || categories.len() != atoms * CATEGORIES
            || self.occurrence_valid.len() != self.batch * self.time
            || roots.iter().chain(categories).any(|x| !x.is_finite())
        {
            candle_core::bail!("value emission logits/validity shape or finiteness differs");
        }
        if let Some(choices) = &self.native_choices {
            if !self.q4_zero_escape
                || choices.roots.len() != atoms
                || choices.categories.len() != atoms
                || choices.roots.iter().any(|&r| usize::from(r) >= ROOTS)
                || choices
                    .categories
                    .iter()
                    .any(|&c| usize::from(c) >= CATEGORIES)
            {
                candle_core::bail!("native value choices shape/range/policy differs");
            }
        }
        Ok(())
    }
    fn selected(&self, at: usize, roots: &[f32], categories: &[f32]) -> (usize, usize) {
        match &self.native_choices {
            Some(c) => (usize::from(c.roots[at]), usize::from(c.categories[at])),
            None => (best(roots), best(categories)),
        }
    }
    fn trace(&self, roots: &[f32], categories: &[f32]) -> candle_core::Result<ValueProducerTrace> {
        self.validate(roots, categories)?;
        let mut packets = Vec::new();
        let mut values_q16 = Vec::new();
        for b in 0..self.batch {
            for h in 0..self.heads {
                for t in 0..self.time {
                    for l in 0..VALUE_LANES {
                        let mut pair = [ValuePacket::absent(); ATOMS];
                        if self.occurrence_valid[b * self.time + t] {
                            for (a, value) in pair.iter_mut().enumerate() {
                                let at = self.atom(b, t, h, l, a);
                                let (root, category) = self.selected(
                                    at,
                                    &roots[at * ROOTS..(at + 1) * ROOTS],
                                    &categories[at * CATEGORIES..(at + 1) * CATEGORIES],
                                );
                                *value = packet(root, category)?;
                            }
                        }
                        let sum = self
                            .codec
                            .decode_pair(pair[0], pair[1])
                            .map_err(|e| candle_core::Error::Msg(e.to_string()))?;
                        values_q16.extend(sum);
                        packets.push(pair.map(record));
                    }
                }
            }
        }
        Ok(ValueProducerTrace {
            batch: self.batch,
            heads: self.heads,
            time: self.time,
            occurrence_valid: self.occurrence_valid.clone(),
            packets,
            values_q16,
        })
    }
    fn prototype(&self, root: usize, category: usize) -> candle_core::Result<[f64; 4]> {
        Ok(self
            .codec
            .decode(packet(root, category)?)
            .map(|x| f64::from(x) / 65_536.))
    }
    fn backward(
        &self,
        roots: &[f32],
        categories: &[f32],
        grad: &[f32],
    ) -> candle_core::Result<(Vec<f32>, Vec<f32>)> {
        self.validate(roots, categories)?;
        if grad.len() != self.batch * self.heads * self.time * VALUE_WIDTH
            || grad.iter().any(|x| !x.is_finite())
        {
            candle_core::bail!("value emission gradient shape/finiteness differs");
        }
        let mut dr = vec![0f64; roots.len()];
        let mut dc = vec![0f64; categories.len()];
        for b in 0..self.batch {
            for h in 0..self.heads {
                for t in 0..self.time {
                    if !self.occurrence_valid[b * self.time + t] {
                        continue;
                    }
                    for l in 0..VALUE_LANES {
                        let start = ((b * self.heads + h) * self.time + t) * VALUE_WIDTH + l * 4;
                        let g = &grad[start..start + 4];
                        let dot = |prototype: [f64; 4]| -> f64 {
                            (0..4).map(|i| f64::from(g[i]) * prototype[i]).sum()
                        };
                        for a in 0..ATOMS {
                            let at = self.atom(b, t, h, l, a);
                            let rs = &roots[at * ROOTS..(at + 1) * ROOTS];
                            let cs = &categories[at * CATEGORIES..(at + 1) * CATEGORIES];
                            let (root, category) = self.selected(at, rs, cs);
                            if category == 0 {
                                if self.q4_zero_escape {
                                    let anchor = if a == 0 { 17 } else { 13 };
                                    let pc = probabilities(&[cs[0], cs[anchor]]);
                                    let pr = probabilities(rs);
                                    let scores = (0..ROOTS)
                                        .map(|r| self.prototype(r, anchor).map(dot))
                                        .collect::<candle_core::Result<Vec<_>>>()?;
                                    let mean: f64 =
                                        pr.iter().zip(&scores).map(|(p, s)| p * s).sum();
                                    for r in 0..ROOTS {
                                        dr[at * ROOTS + r] = pc[1] * pr[r] * (scores[r] - mean);
                                    }
                                    let category_credit = pc[0] * pc[1] * mean;
                                    dc[at * CATEGORIES] = -category_credit;
                                    dc[at * CATEGORIES + anchor] = category_credit;
                                }
                                continue;
                            }
                            let p = probabilities(rs);
                            let scores = (0..ROOTS)
                                .map(|r| self.prototype(r, category).map(dot))
                                .collect::<candle_core::Result<Vec<_>>>()?;
                            let mean: f64 = p.iter().zip(&scores).map(|(p, s)| p * s).sum();
                            for r in 0..ROOTS {
                                dr[at * ROOTS + r] = p[r] * (scores[r] - mean);
                            }
                            let lo = category.saturating_sub(1).max(1);
                            let hi = (category + 1).min(CATEGORIES - 1);
                            let candidates = if self.q4_zero_escape {
                                std::iter::once(0).chain(lo..=hi).collect::<Vec<_>>()
                            } else {
                                (lo..=hi).collect::<Vec<_>>()
                            };
                            let p = probabilities(
                                &candidates.iter().map(|&c| cs[c]).collect::<Vec<_>>(),
                            );
                            let scores = candidates
                                .iter()
                                .map(|&c| self.prototype(root, c).map(dot))
                                .collect::<candle_core::Result<Vec<_>>>()?;
                            let mean: f64 = p.iter().zip(&scores).map(|(p, s)| p * s).sum();
                            for (i, &c) in candidates.iter().enumerate() {
                                dc[at * CATEGORIES + c] = p[i] * (scores[i] - mean);
                            }
                        }
                    }
                }
            }
        }
        Ok((checked_f32(dr)?, checked_f32(dc)?))
    }
}
impl CustomOp2 for ValueEmit {
    fn name(&self) -> &'static str {
        "offline-geometric-K2-value-emission"
    }
    fn cpu_fwd(
        &self,
        a: &CpuStorage,
        la: &Layout,
        b: &CpuStorage,
        lb: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        let trace = self.trace(contiguous(a, la)?, contiguous(b, lb)?)?;
        let output = trace
            .values_q16
            .into_iter()
            .map(|x| (f64::from(x) / 65_536.) as f32)
            .collect();
        Ok((
            CpuStorage::F32(output),
            Shape::from((self.batch, self.heads, self.time, VALUE_WIDTH)),
        ))
    }
    fn bwd(
        &self,
        a: &Tensor,
        b: &Tensor,
        _out: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>)> {
        let (da, db) = self.backward(
            &a.flatten_all()?.to_vec1::<f32>()?,
            &b.flatten_all()?.to_vec1::<f32>()?,
            &grad.flatten_all()?.to_vec1::<f32>()?,
        )?;
        Ok((
            Some(Tensor::from_vec(da, a.shape(), a.device())?),
            Some(Tensor::from_vec(db, b.shape(), b.device())?),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_context::ContextWeights;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn norm(values: &[f32]) -> f64 {
        values.iter().map(|&x| f64::from(x).abs()).sum()
    }
    fn emitter() -> Result<(ValueEmit, Vec<f32>, Vec<f32>)> {
        let op = ValueEmit {
            batch: 1,
            time: 1,
            heads: 1,
            occurrence_valid: vec![true],
            q4_zero_escape: false,
            native_choices: None,
            codec: Arc::new(
                NativeGeometricValues::canonical().map_err(|e| invalid(e.to_string()))?,
            ),
        };
        let mut roots = vec![-0.3; VALUE_LANES * ATOMS * ROOTS];
        let mut cats = vec![-0.4; VALUE_LANES * ATOMS * CATEGORIES];
        for row in roots.chunks_exact_mut(ROOTS) {
            row[1] = 0.8;
        }
        for row in cats.chunks_exact_mut(CATEGORIES) {
            row[17] = 1.;
            row[16] = 0.1;
            row[18] = 0.2;
        }
        Ok((op, roots, cats))
    }
    fn identity_roots(batch: usize, time: usize, heads: usize, lanes: usize) -> Result<Tensor> {
        Ok(Tensor::from_vec(
            [1f32, 0., 0., 0.].repeat(batch * time * heads * lanes),
            vec![batch, time, heads, lanes, 4],
            &Device::Cpu,
        )?)
    }

    #[test]
    fn geometric_value_producer_antipodal_credit_reaches_both_atoms() -> Result<()> {
        let (op, roots, cats) = emitter()?;
        let r = Var::from_vec(roots.clone(), roots.len(), &Device::Cpu)?;
        let c = Var::from_vec(cats.clone(), cats.len(), &Device::Cpu)?;
        let output = r.apply_op2(&c, op)?;
        // Parallel positive output adjoint: reducing this objective should
        // lower +identity and raise -identity, which tangent credit erases.
        let loss = output
            .reshape((VALUE_LANES, 4))?
            .narrow(1, 0, 1)?
            .sum_all()?;
        let grad = loss.backward()?;
        let dr = grad
            .get(r.as_tensor())
            .ok_or_else(|| invalid("root gradient missing"))?
            .to_vec1::<f32>()?;
        let dc = grad
            .get(c.as_tensor())
            .ok_or_else(|| invalid("category gradient missing"))?
            .to_vec1::<f32>()?;
        for atom in 0..VALUE_LANES * ATOMS {
            assert!(dr[atom * ROOTS] < 0.);
            assert!(dr[atom * ROOTS + 1] > 0.);
            for category in 0..CATEGORIES {
                if !(16..=18).contains(&category) {
                    assert_eq!(dc[atom * CATEGORIES + category], 0.);
                }
            }
        }
        Ok(())
    }

    #[test]
    fn geometric_value_producer_surrogate_matches_frozen_branch_difference() -> Result<()> {
        let (op, roots, cats) = emitter()?;
        let g = [0.7f32, 0.3, -0.2, 0.1].repeat(VALUE_LANES);
        let (dr, dc) = op.backward(&roots, &cats, &g)?;
        // Differentiate the declared smooth local function with hard root,
        // hard radius and its neighbor set fixed, never the hard argmax.
        let scalar = |r: &[f32], c: &[f32]| -> Result<f64> {
            let angular = (0..ROOTS)
                .map(|i| op.prototype(i, 17))
                .collect::<candle_core::Result<Vec<_>>>()?;
            let dot = |v: [f64; 4]| (0..4).map(|i| v[i] * f64::from(g[i])).sum::<f64>();
            let direction = probabilities(&r[..ROOTS])
                .iter()
                .zip(angular)
                .map(|(p, v)| p * dot(v))
                .sum::<f64>();
            let radii = (16..=18)
                .map(|i| op.prototype(1, i))
                .collect::<candle_core::Result<Vec<_>>>()?;
            Ok(direction
                + probabilities(&c[16..=18])
                    .iter()
                    .zip(radii)
                    .map(|(p, v)| p * dot(v))
                    .sum::<f64>())
        };
        for (angular, index) in [
            (true, 0),
            (true, 1),
            (true, 24),
            (false, 16),
            (false, 17),
            (false, 18),
        ] {
            let (mut rp, mut rm, mut cp, mut cm) =
                (roots.clone(), roots.clone(), cats.clone(), cats.clone());
            let epsilon = 0.001f32;
            let analytic = if angular {
                rp[index] += epsilon;
                rm[index] -= epsilon;
                dr[index]
            } else {
                cp[index] += epsilon;
                cm[index] -= epsilon;
                dc[index]
            };
            let finite = (scalar(&rp, &cp)? - scalar(&rm, &cm)?) / (2. * f64::from(epsilon));
            assert!((f64::from(analytic) - finite).abs() < 0.0001);
        }
        Ok(())
    }

    #[test]
    fn geometric_value_producer_status_cancellation_overflow_and_zero_stop() -> Result<()> {
        let (mut op, mut roots, mut cats) = emitter()?;
        for lane in 0..VALUE_LANES {
            let second = (lane * ATOMS + 1) * ROOTS;
            roots[second] = 2.;
        }
        let trace = op.trace(&roots, &cats)?;
        assert!(trace.values_q16.iter().all(|&v| v == 0));
        assert!(trace
            .packets
            .iter()
            .flatten()
            .all(|p| p.status == ValuePacketStatus::PresentNonzero));
        for row in cats.chunks_exact_mut(CATEGORIES) {
            row[0] = 3.;
        }
        let trace = op.trace(&roots, &cats)?;
        assert!(trace
            .packets
            .iter()
            .flatten()
            .all(|p| p.status == ValuePacketStatus::PresentZero));
        let (dr, dc) = op.backward(&roots, &cats, &vec![1.; VALUE_WIDTH])?;
        assert_eq!(norm(&dr) + norm(&dc), 0.);
        op.occurrence_valid[0] = false;
        let trace = op.trace(&roots, &cats)?;
        assert!(trace
            .packets
            .iter()
            .flatten()
            .all(|p| p.status == ValuePacketStatus::Absent));
        assert!(!trace.occurrence_valid[0]);
        for row in roots.chunks_exact_mut(ROOTS) {
            row[1] = 4.;
        }
        for row in cats.chunks_exact_mut(CATEGORIES) {
            row[31] = 4.;
        }
        assert!(op.trace(&roots, &cats).is_ok());
        op.occurrence_valid[0] = true;
        assert!(
            op.trace(&roots, &cats).is_err(),
            "positive pair overflow must reject, not clip"
        );
        Ok(())
    }

    #[test]
    fn geometric_value_producer_actual_latent_history_gradient_and_causal_features() -> Result<()> {
        let context = ContextWeights::new_finite_choice(3, 32, 2, 77)?;
        let weights = ValueProducerWeights::new(3, 2, 4, 21)?;
        let latent = context.forward(&[0, 1, 2], 1, 3, false)?.latent_roots;
        let span = Var::from_tensor(&identity_roots(1, 3, 2, 4)?)?;
        let output =
            weights.forward(&[0, 1, 2], &latent, &span, &[false, true, true], &[true; 3])?;
        assert_eq!(output.values.dims(), [1, 2, 3, 16]);
        let loss = output.values.narrow(2, 2, 1)?.sqr()?.sum_all()?;
        let grad = loss.backward()?;
        for (name, variable) in weights.parameters() {
            let values = grad
                .get(variable.as_tensor())
                .ok_or_else(|| invalid(format!("missing {name} gradient")))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            assert!(norm(&values) > 0., "no output credit to {name}");
        }
        let transitions = grad
            .get(context.parameters()["token_transition"].as_tensor())
            .ok_or_else(|| invalid("missing context transition gradient"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let first_token_row = 2 * 4 * ROOTS;
        assert!(
            norm(&transitions[..first_token_row]) > 1e-12,
            "terminal value loss must reach earlier token0 transitions"
        );
        let ds = grad
            .get(span.as_tensor())
            .ok_or_else(|| invalid("missing span gradient"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(norm(&ds) > 0.);
        assert_eq!(norm(&ds[..2 * 4 * 4]), 0.);
        // Only the final token/history changes. Earlier pointwise head outputs
        // stay identical, and an invalid span's supplied coordinates are inert.
        let changed_latent = context.forward(&[0, 1, 0], 1, 3, false)?.latent_roots;
        let mut changed_span = span.flatten_all()?.to_vec1::<f32>()?;
        changed_span[..2 * 4 * 4].fill(123.);
        let changed_span = Tensor::from_vec(changed_span, span.shape(), &Device::Cpu)?;
        let changed = weights.forward(
            &[0, 1, 0],
            &changed_latent,
            &changed_span,
            &[false, true, true],
            &[true; 3],
        )?;
        for head in 0..2 {
            let at = head * 3 * VALUE_WIDTH;
            assert_eq!(
                output.trace.values_q16[at..at + 2 * VALUE_WIDTH],
                changed.trace.values_q16[at..at + 2 * VALUE_WIDTH]
            );
        }
        Ok(())
    }

    #[test]
    fn geometric_value_producer_saved_source_roundtrip_and_admission() -> Result<()> {
        static SERIAL: AtomicU64 = AtomicU64::new(0);
        let clock = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| invalid(e.to_string()))?
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "uor-value-producer-{}-{clock}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        let weights = ValueProducerWeights::new(3, 1, 1, 31)?;
        let latent = identity_roots(1, 2, 1, 1)?;
        let before = weights.forward(&[0, 1], &latent, &latent, &[false, true], &[true, true])?;
        weights.save_source(&directory)?;
        assert!(weights.save_source(&directory).is_err());
        let restored = ValueProducerWeights::load_source(&directory)?;
        assert_eq!(weights.config(), restored.config());
        assert_eq!(
            before.trace,
            restored
                .forward(&[0, 1], &latent, &latent, &[false, true], &[true, true])?
                .trace
        );
        for (name, value) in weights.parameters() {
            assert_eq!(
                value.flatten_all()?.to_vec1::<f32>()?,
                restored.parameters()[name]
                    .flatten_all()?
                    .to_vec1::<f32>()?
            );
        }
        let mut metadata: SourceMetadata =
            serde_json::from_slice(&fs::read(directory.join(SOURCE_FILES[0]))?)?;
        metadata.config.surrogate.push_str("unbound-change");
        fs::write(
            directory.join(SOURCE_FILES[0]),
            serde_json::to_vec(&metadata)?,
        )?;
        assert!(ValueProducerWeights::load_source(&directory).is_err());
        assert!(weights
            .forward(&[3, 1], &latent, &latent, &[false, true], &[true, true])
            .is_err());
        assert!(weights
            .forward(&[0, 1], &latent, &latent, &[false], &[true, true])
            .is_err());
        fs::remove_dir_all(directory)?;
        Ok(())
    }

    #[test]
    fn value_q4_source_grid_ste_and_legacy_roundtrip() -> Result<()> {
        let weights = ValueProducerWeights::new(3, 1, 2, 73)?;
        assert!(!serde_json::to_string(weights.config())?.contains("coefficient_policy"));
        assert!(weights.packed_coefficients().is_err());
        let shape = weights.parameters()["token_root"].shape().clone();
        let mut changed = weights.parameters()["token_root"]
            .flatten_all()?
            .to_vec1::<f32>()?;
        changed[..4].copy_from_slice(&[0.125, -0.125, 3., -3.]);
        weights.parameters()["token_root"].set(&Tensor::from_vec(changed, shape, &Device::Cpu)?)?;
        let weights = weights.into_q4()?;
        assert_eq!(
            &weights
                .coefficient("token_root")?
                .flatten_all()?
                .to_vec1::<f32>()?[..4],
            &[0.25, -0.25, 1.75, -1.75]
        );
        let grad = weights.coefficient("token_root")?.sum_all()?.backward()?;
        let g = grad
            .get(weights.parameters()["token_root"].as_tensor())
            .ok_or_else(|| invalid("q4 shadow STE disconnected"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(g.iter().all(|x| *x == 1.));
        let directory = std::env::temp_dir().join(format!(
            "uor-value-q4-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| invalid(e.to_string()))?
                .as_nanos()
        ));
        weights.save_source(&directory)?;
        let loaded = ValueProducerWeights::load_source(&directory)?;
        assert_eq!(weights.config(), loaded.config());
        assert_eq!(
            weights.packed_coefficients()?,
            loaded.packed_coefficients()?
        );
        for (name, var) in weights.parameters() {
            assert_eq!(
                var.flatten_all()?.to_vec1::<f32>()?,
                loaded.parameters()[name].flatten_all()?.to_vec1::<f32>()?
            );
        }
        let input = identity_roots(1, 2, 1, 2)?;
        assert_eq!(
            weights
                .forward(&[0, 1], &input, &input, &[false, true], &[true, true])?
                .trace,
            loaded
                .forward(&[0, 1], &input, &input, &[false, true], &[true, true])?
                .trace
        );
        let path = directory.join(SOURCE_FILES[0]);
        let mut metadata: SourceMetadata = serde_json::from_slice(&fs::read(&path)?)?;
        metadata.packed_sha256 = Some("0".repeat(64));
        fs::write(&path, serde_json::to_vec(&metadata)?)?;
        assert!(ValueProducerWeights::load_source(&directory).is_err());
        metadata.config.coefficient_policy = Some("unknown".into());
        fs::write(&path, serde_json::to_vec(&metadata)?)?;
        assert!(ValueProducerWeights::load_source(&directory).is_err());
        fs::remove_dir_all(directory)?;
        Ok(())
    }

    #[test]
    fn value_q4_zero_anchor_credit_is_joint_and_absence_stays_blocked() -> Result<()> {
        let (mut op, mut roots, mut cats) = emitter()?;
        roots.fill(0.);
        cats.fill(0.);
        for c in cats.chunks_exact_mut(CATEGORIES) {
            c[0] = 1.;
        }
        let gradient = [1f32, 0., 0., 0.].repeat(VALUE_LANES);
        let legacy = op.backward(&roots, &cats, &gradient)?;
        assert_eq!(norm(&legacy.0) + norm(&legacy.1), 0.);
        op.q4_zero_escape = true;
        let (dr, dc) = op.backward(&roots, &cats, &gradient)?;
        assert!(op
            .trace(&roots, &cats)?
            .packets
            .iter()
            .flatten()
            .all(|p| p.status == ValuePacketStatus::PresentZero));
        assert!(dr[0] < 0. && dr[1] > 0.);
        assert!(
            norm(&dc) < 1e-7,
            "symmetric directions initially give zero category credit"
        );
        assert!(
            (dr[1] / dr[ROOTS + 1] - 16.).abs() < 1e-4,
            "anchors have their declared physical radii"
        );
        for row in roots.chunks_exact_mut(ROOTS) {
            row[1] = 0.75;
        }
        let (dr, dc) = op.backward(&roots, &cats, &gradient)?;
        assert!(dc[0] < 0. && dc[17] > 0. && dc[CATEGORIES] < 0. && dc[CATEGORIES + 13] > 0.);
        // Finite differences of the declared zero-branch relaxation, not hard argmax.
        let objective = |r: &[f32], c: &[f32]| -> Result<f64> {
            let pr = probabilities(r);
            let pc = probabilities(&[c[0], c[17]]);
            let mut mean = 0.;
            for (root, p) in pr.iter().enumerate() {
                mean += p * op.prototype(root, 17)?[0];
            }
            Ok(pc[1] * mean)
        };
        for index in [0, 1, 19] {
            let mut plus = roots[..ROOTS].to_vec();
            let mut minus = plus.clone();
            plus[index] += 0.001;
            minus[index] -= 0.001;
            let fd = (objective(&plus, &cats[..CATEGORIES])?
                - objective(&minus, &cats[..CATEGORIES])?)
                / f64::from(plus[index] - minus[index]);
            assert!((fd - f64::from(dr[index])).abs() < 2e-6);
        }
        let mut positive = cats.clone();
        for row in positive.chunks_exact_mut(CATEGORIES) {
            row[17] = 2.;
        }
        let (_, positive_dc) = op.backward(&roots, &positive, &gradient)?;
        assert!(positive_dc[0] < 0. && positive_dc[16] != 0. && positive_dc[18] != 0.);
        assert_eq!(positive_dc[30], 0.);
        op.occurrence_valid[0] = false;
        let absent = op.backward(&roots, &cats, &gradient)?;
        assert_eq!(norm(&absent.0) + norm(&absent.1), 0.);
        assert!(op
            .trace(&roots, &cats)?
            .packets
            .iter()
            .flatten()
            .all(|p| p.status == ValuePacketStatus::Absent));
        Ok(())
    }

    #[test]
    fn value_q4_native_choices_match_live_coefficients_and_condition_backward() -> Result<()> {
        use uor_r4_integer::geometric_value_q4::{NativeValueQ4, ValueQ4Config};
        let weights = ValueProducerWeights::new(3, 2, 2, 101)?.into_q4()?;
        let config = ValueQ4Config {
            vocab_size: 3,
            heads: 2,
            latent_lanes_per_head: 2,
        };
        let ids = [0, 1, 2, 0];
        let codes = (0..16)
            .map(|i| H4Code::try_from((i + 1) as u8).map_err(|e| invalid(e.to_string())))
            .collect::<Result<Vec<_>>>()?;
        let held = codes.iter().rev().copied().collect::<Vec<_>>();
        let span = [false, true, true, false];
        let valid = [true, true, false, true];
        let check = |output: &ValueProducerOutput| -> Result<()> {
            let native = NativeValueQ4::new(config, &weights.packed_coefficients()?)
                .and_then(|x| x.into_native())
                .map_err(|e| invalid(e.to_string()))?;
            for row in 0..4 {
                let actual = native
                    .produce(
                        ids[row] as usize,
                        &codes[row * 4..row * 4 + 4],
                        if span[row] {
                            Some(&held[row * 4..row * 4 + 4])
                        } else {
                            None
                        },
                        valid[row],
                    )
                    .map_err(|e| invalid(e.to_string()))?;
                let (b, t) = (row / 2, row % 2);
                for h in 0..2 {
                    let at = (b * 2 + h) * 2 + t;
                    assert_eq!(
                        &output.trace.values_q16[at * 16..at * 16 + 16],
                        &actual.values_q16[h * 16..h * 16 + 16]
                    );
                    for lane in 0..4 {
                        assert_eq!(
                            output.trace.packets[at * 4 + lane],
                            actual.packets[h * 4 + lane].map(record)
                        );
                    }
                }
            }
            assert_eq!(
                output.values.flatten_all()?.to_vec1::<f32>()?,
                output
                    .trace
                    .values_q16
                    .iter()
                    .map(|&v| (f64::from(v) / 65536.) as f32)
                    .collect::<Vec<_>>()
            );
            Ok(())
        };
        let first = weights.forward_q4_codes(&ids, 2, 2, &codes, &held, &span, &valid)?;
        check(&first)?;
        let var = &weights.parameters()["token_root"];
        let mut root = var.flatten_all()?.to_vec1::<f32>()?;
        root[7] = 1.5;
        var.set(&Tensor::from_vec(root, var.shape(), &Device::Cpu)?)?;
        let second = weights.forward_q4_codes(&ids, 2, 2, &codes, &held, &span, &valid)?;
        check(&second)?;
        assert_ne!(
            first.trace, second.trace,
            "new hard pass must not reuse stale packed tables"
        );
        assert_eq!(second.trace.packets[0][0].root, 7);
        let var = &weights.parameters()["token_category"];
        let mut categories = var.flatten_all()?.to_vec1::<f32>()?;
        categories[0] = 1.75;
        var.set(&Tensor::from_vec(categories, var.shape(), &Device::Cpu)?)?;
        let zero = weights.forward_q4_codes(&ids, 2, 2, &codes, &held, &span, &valid)?;
        check(&zero)?;
        assert_eq!(
            zero.trace.packets[0][0].status,
            ValuePacketStatus::PresentZero
        );
        assert_eq!(
            zero.trace.packets[0][0].root, 1,
            "packet placeholder is not selected root"
        );
        let native = NativeValueQ4::new(config, &weights.packed_coefficients()?)
            .and_then(|x| x.into_native())
            .map_err(|e| invalid(e.to_string()))?;
        let selected = native
            .produce(0, &codes[..4], None, true)
            .map_err(|e| invalid(e.to_string()))?;
        assert_eq!((selected.root_choices[0], selected.categories[0]), (7, 0));
        // row2 is structurally absent, even when the same source has valid zeros.
        assert!(zero.trace.packets[16..20]
            .iter()
            .flatten()
            .all(|p| p.status == ValuePacketStatus::Absent));
        assert!(weights
            .forward_q4_codes(&ids, 2, 2, &codes[..15], &held, &span, &valid)
            .is_err());

        // Force a score/decision disagreement to pin the backward's conditional
        // point. Positive native category18/root+j must override float zero/+1.
        let (mut op, roots, mut cats) = emitter()?;
        op.q4_zero_escape = true;
        cats.fill(0.);
        for row in cats.chunks_exact_mut(CATEGORIES) {
            row[0] = 1.;
        }
        op.native_choices = Some(NativeValueChoices {
            roots: vec![5; 8],
            categories: vec![18; 8],
        });
        let trace = op.trace(&roots, &cats)?;
        assert_eq!(
            (trace.packets[0][0].root, trace.packets[0][0].radius_bin),
            (5, 17)
        );
        let (_, dc) = op.backward(&roots, &cats, &[0f32, 0., 1., 0.].repeat(4))?;
        assert!(
            dc[19] > 0. && dc[17] < 0.,
            "native positive root/radius must condition local credit"
        );
        assert_eq!(dc[13], 0.);
        // Conversely native zero retains raw root7, but renders canonicalzero
        // and uses the zero-anchor branch despite float category18 winning.
        for row in cats.chunks_exact_mut(CATEGORIES) {
            row[18] = 2.;
        }
        op.native_choices = Some(NativeValueChoices {
            roots: vec![7; 8],
            categories: vec![0; 8],
        });
        assert_eq!(op.selected(0, &roots[..ROOTS], &cats[..CATEGORIES]), (7, 0));
        assert_eq!(
            op.trace(&roots, &cats)?.packets[0][0].status,
            ValuePacketStatus::PresentZero
        );
        let (dr, dc) = op.backward(&roots, &cats, &[1f32, 0., 0., 0.].repeat(4))?;
        assert!(dr[1] > 0. && dc[17] > 0. && dc[0] < 0.);
        assert_eq!(
            dc[18], 0.,
            "float winning radius cannot choose backward neighborhood"
        );
        op.occurrence_valid[0] = false;
        let (dr, dc) = op.backward(&roots, &cats, &[1f32; 16])?;
        assert_eq!(norm(&dr) + norm(&dc), 0.);
        Ok(())
    }
}
