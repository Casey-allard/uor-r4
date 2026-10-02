//! Joint answer-credit continuation of retained geometric context and K2 values.
//! Offline donor targets never enter the learned prediction interfaces.
#[path = "attention-geometric-value-learned/continuation.rs"]
mod continuation;
#[path = "attention-geometric-value-learned/credit.rs"]
mod credit;
#[path = "attention-geometric-value-learned/data.rs"]
mod data;
use candle_core::{backprop::GradStore, Device, Tensor, Var};
use candle_nn::{AdamW, Optimizer, ParamsAdamW};
use continuation::Continuation;
use credit::AuxiliaryCredit;
use data::{batch, draw, episode, noise, Episode, Rng};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use uor_r4_core::report_output;
use uor_r4_training::geometric_context::{
    self, CompiledContext, ContextSourcePaths, ContextWeights,
};
use uor_r4_training::geometric_event::{self, CompiledEvents};
use uor_r4_training::geometric_potential_native::{
    CompiledGeometricPotentials, PotentialSourceBinding,
};
use uor_r4_training::geometric_read_native::{
    CompiledGeometricRead, NativeReadTrace, ReadSourceBinding,
};
use uor_r4_training::geometric_span_native::{self, CompiledSpanActions, SpanSourceBinding};
use uor_r4_training::geometric_stack::{
    logits_cross_entropy, ReadBinding, ReadBindingTarget, StackModel,
};
use uor_r4_training::geometric_value_native::{
    project_residual_q16, CompiledGeometricValues, ValuePacketStatus, ValueProjectionTrace,
    ValueSourceBinding,
};
use uor_r4_training::geometric_value_producer::{ValueProducerOutput, ValueProducerWeights};
use uor_r4_training::geometric_value_producer_native::{
    CompiledValueProducer, ValueProducerSourcePaths,
};
use uor_r4_training::{sha256_bytes, sha256_file, Result, TrainingError};

const EVAL_HASH: &str = "dabdaa1a2a8dcf23e1cfe5164ef00b97b923c3b74dc09dcd1d4a307d1ff64915";
const STRESS_HASH: &str = "e3c81984b7b41e0fcb329fde026318d067b6397e93832508c3ee60180a5a9d63";
const TRANSITIONS: [&str; 3] = ["token_transition", "self_transition", "neighbor_transition"];
const BATCH: usize = 8;
fn invalid(s: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(s.into())
}
fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    fs::write(path, serde_json::to_vec(value)?)?;
    Ok(())
}

#[derive(Serialize)]
struct Args {
    mode: String,
    seed: u64,
    steps: usize,
    rate: f64,
    max_seconds: u64,
    evaluation_reserve_seconds: u64,
    model: PathBuf,
    context_source: PathBuf,
    context_native: PathBuf,
    event_source: PathBuf,
    event_native: PathBuf,
    span_native: PathBuf,
    potential_native: PathBuf,
    evaluation: PathBuf,
    stress: PathBuf,
    prior_k1: PathBuf,
    prior_k2: PathBuf,
    continuation_parent: Option<PathBuf>,
    teacher_replay: Option<PathBuf>,
    auxiliary_credit: AuxiliaryCredit,
    out: PathBuf,
}
impl Args {
    fn parse() -> Result<Self> {
        let mut o = BTreeMap::new();
        for a in std::env::args().skip(1) {
            let (k, v) = a.split_once('=').ok_or_else(|| invalid("use key=value"))?;
            if v.is_empty() || o.insert(k.to_string(), v.to_string()).is_some() {
                return Err(invalid("empty/repeated argument"));
            }
        }
        let mode = o.remove("mode").unwrap_or_else(|| "fit".into());
        let seed = o
            .remove("seed")
            .ok_or_else(|| invalid("seed required"))?
            .parse::<u64>()
            .map_err(|_| invalid("seed"))?;
        let steps = o
            .remove("steps")
            .unwrap_or_else(|| "640".into())
            .parse::<usize>()
            .map_err(|_| invalid("steps"))?;
        let rate = o
            .remove("rate")
            .unwrap_or_else(|| "0.003".into())
            .parse::<f64>()
            .map_err(|_| invalid("rate"))?;
        let max_seconds = o
            .remove("max_seconds")
            .ok_or_else(|| invalid("max_seconds required; use the recorded work-card budget"))?
            .parse::<u64>()
            .map_err(|_| invalid("max_seconds"))?;
        let evaluation_reserve_seconds = o
            .remove("evaluation_reserve_seconds")
            .ok_or_else(|| invalid("evaluation_reserve_seconds required"))?
            .parse::<u64>()
            .map_err(|_| invalid("evaluation_reserve_seconds"))?;
        let continuation_parent = o.remove("continuation_parent").map(PathBuf::from);
        let teacher_replay = o.remove("teacher_replay").map(PathBuf::from);
        let auxiliary_credit = match o.remove("auxiliary_credit").as_deref() {
            None => AuxiliaryCredit::Legacy,
            Some("query_read") => AuxiliaryCredit::QueryRead,
            Some("uniform_matched_mass") => AuxiliaryCredit::UniformMatchedMass,
            _ => return Err(invalid("auxiliary_credit query_read|uniform_matched_mass")),
        };
        if continuation_parent.is_some() != teacher_replay.is_some()
            || continuation_parent.is_some() != (auxiliary_credit != AuxiliaryCredit::Legacy)
        {
            return Err(invalid("continuation_parent, teacher_replay and auxiliary_credit must be supplied together"));
        }
        if !["fit", "check"].contains(&mode.as_str())
            || ![1, 2].contains(&seed)
            || steps == 0
            || steps > 640
            || !rate.is_finite()
            || rate <= 0.
            || rate > 0.03
            || max_seconds == 0
            || max_seconds > 86400
            || evaluation_reserve_seconds == 0
            || evaluation_reserve_seconds >= max_seconds
        {
            return Err(invalid("mode fit/check; seed1/2; steps1..640; rate(0,.03]; explicit positive bounded deadline/reserve"));
        }
        if continuation_parent.is_some() && mode == "fit" && steps != 640 {
            return Err(invalid(
                "declared paired continuation dose is 640 new updates",
            ));
        }
        let mut path = |key: &str| {
            o.remove(key)
                .map(PathBuf::from)
                .ok_or_else(|| invalid(format!("missing {key}")))
        };
        let result = Self {
            mode,
            seed,
            steps,
            rate,
            max_seconds,
            evaluation_reserve_seconds,
            continuation_parent,
            teacher_replay,
            auxiliary_credit,
            model: path("model")?,
            context_source: path("context_source")?,
            context_native: path("context_native")?,
            event_source: path("event_source")?,
            event_native: path("event_native")?,
            span_native: path("span_native")?,
            potential_native: path("potential_native")?,
            evaluation: path("evaluation")?,
            stress: path("stress")?,
            prior_k1: path("prior_k1")?,
            prior_k2: path("prior_k2")?,
            out: path("out")?,
        };
        if !o.is_empty() {
            return Err(invalid("unknown option"));
        }
        Ok(result)
    }
    fn dependencies(&self) -> ContextSourcePaths<'_> {
        ContextSourcePaths {
            base: &self.model,
            event_source: &self.event_source,
            event_native: &self.event_native,
            span_native: &self.span_native,
            potential_native: &self.potential_native,
        }
    }
}

fn panel(path: &Path, hash: &str, output: &Path) -> Result<Vec<Episode>> {
    let bytes = fs::read(path)?;
    if sha256_bytes(&bytes) != hash {
        return Err(invalid("pinned development panel hash differs"));
    }
    let rows: Vec<Episode> = serde_json::from_slice(&bytes)?;
    if rows.len() != 128
        || rows.iter().any(|e| {
            e.ids.is_empty()
                || e.ids.len() > 128
                || e.roles.len() != e.ids.len()
                || e.query + 1 != e.ids.len()
                || e.source >= e.query
                || e.ids[e.source] != e.answer
                || e.ids[e.query] != 34
                || e.ids.iter().any(|&x| x >= 40)
        })
    {
        return Err(invalid("panel occurrence/shape contract"));
    }
    fs::write(output, bytes)?;
    Ok(rows)
}
fn file_inventory(directory: &Path) -> Result<Value> {
    let mut files = BTreeMap::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            return Err(invalid("input artifact must have flat regular files"));
        }
        files.insert(
            entry.file_name().to_string_lossy().into_owned(),
            json!({"bytes":entry.metadata()?.len(),"sha256":sha256_file(&entry.path())?}),
        );
    }
    Ok(json!({"path":directory,"files":files}))
}
fn parameter_hashes(parameters: &BTreeMap<String, Var>) -> Result<BTreeMap<String, String>> {
    parameters
        .iter()
        .map(|(n, v)| {
            Ok((
                n.clone(),
                sha256_bytes(
                    &v.flatten_all()?
                        .to_vec1::<f32>()?
                        .into_iter()
                        .flat_map(f32::to_le_bytes)
                        .collect::<Vec<_>>(),
                ),
            ))
        })
        .collect()
}
fn trainable(
    context: &ContextWeights,
    values: &ValueProducerWeights,
) -> Result<Vec<(String, Var)>> {
    let mut out = Vec::new();
    for name in TRANSITIONS {
        out.push((
            format!("context.{name}"),
            context
                .parameters()
                .get(name)
                .ok_or_else(|| invalid("missing transition family"))?
                .clone(),
        ));
    }
    out.extend(
        values
            .parameters()
            .iter()
            .map(|(n, v)| (format!("value.{n}"), v.clone())),
    );
    Ok(out)
}
fn gradient_families(vars: &[(String, Var)], g: &GradStore) -> Result<BTreeMap<String, f64>> {
    vars.iter()
        .map(|(n, v)| {
            let value = g
                .get(v.as_tensor())
                .map(|x| x.sqr()?.sum_all()?.to_scalar::<f32>())
                .transpose()?
                .map(f64::from)
                .unwrap_or(0.)
                .sqrt();
            if !value.is_finite() {
                return Err(invalid("nonfinite gradient"));
            }
            Ok((n.clone(), value))
        })
        .collect()
}
fn clip(vars: &[(String, Var)], g: &mut GradStore) -> Result<f64> {
    let norm = gradient_families(vars, g)?
        .values()
        .map(|x| x * x)
        .sum::<f64>()
        .sqrt();
    if !norm.is_finite() {
        return Err(invalid("nonfinite trainable gradient norm"));
    }
    if norm > 1. {
        for (_, v) in vars {
            if let Some(x) = g.get(v.as_tensor()) {
                g.insert(v.as_tensor(), x.affine(1. / norm, 0.)?);
            }
        }
    }
    Ok(norm)
}
fn argmax(v: &[f32]) -> Result<u32> {
    if v.is_empty() || v.iter().any(|x| !x.is_finite()) {
        return Err(invalid("nonfinite/empty logits"));
    }
    let mut best = 0;
    for i in 1..v.len() {
        if v[i] > v[best] {
            best = i;
        }
    }
    Ok(best as u32)
}
fn validate_logits(v: &[Vec<f32>], n: usize) -> Result<()> {
    if v.len() != n
        || v.iter()
            .any(|r| r.len() != 40 || r.iter().any(|x| !x.is_finite()))
    {
        return Err(invalid("logit shape/finiteness"));
    }
    Ok(())
}

struct Frozen {
    model: StackModel,
    parent: CompiledContext,
    events: CompiledEvents,
    span: CompiledSpanActions,
    potential: CompiledGeometricPotentials,
    reducer: CompiledGeometricRead,
    codec: CompiledGeometricValues,
    tokenizer: Vec<u8>,
    initial_native_values: Option<CompiledValueProducer>,
}
impl Frozen {
    fn load(a: &Args) -> Result<(Self, ContextWeights)> {
        let expected = [
            "235614f2e1861dd79e2e4ea8dac3cdaaec13243f145c04d9a77fbda3e9bb47fa",
            "3909b1129e4cf2a2aa72c3cc7ad3172b8eb1c847b881ce054418e31f44c21873",
        ][(a.seed - 1) as usize];
        if sha256_file(&a.model.join("model.safetensors"))? != expected {
            return Err(invalid("accepted seed/model mismatch"));
        }
        let expected_context = if a.continuation_parent.is_some() {
            [
                "102d6ec3ff84c06ff581831b63fbecf72e18fe328363f84bd0ea2d0a080c460c",
                "3b1e0fbad2d8783c94b36b85c28532712427985dff02fcde9326d51100159b1b",
            ]
        } else {
            [
                "0a2f5f1903c4e4101ea2f2b9fef5062d73917e99ac6b9bda8bafceb7fcf7a3a8",
                "fb8133d136af654488bff8f6c7d1e5be7444a9394e44a6da09d9bed5e8896f8a",
            ]
        }[(a.seed - 1) as usize];
        if sha256_file(&a.context_source.join("context-parameters.safetensors"))?
            != expected_context
        {
            return Err(invalid("accepted 640-update context parent differs"));
        }
        let model = StackModel::load(&a.model, &Device::Cpu)?;
        if model.config.width != 32
            || model.config.heads != 2
            || model.config.context != 128
            || model.config.vocab_size != 40
        {
            return Err(invalid("width32/H2/context128/vocab40 required"));
        }
        let tokenizer = fs::read(a.span_native.join("tokenizer-identity.bin"))?;
        let span = CompiledSpanActions::load(
            &a.span_native,
            &SpanSourceBinding::from_files(
                &a.model.join("model.safetensors"),
                &a.model.join("config.json"),
                &tokenizer,
            )?,
            model
                .geometric_span()
                .ok_or_else(|| invalid("span absent"))?,
        )?;
        let potential = CompiledGeometricPotentials::load(
            &a.potential_native,
            &PotentialSourceBinding::from_directory(&a.model, &tokenizer)?,
        )?;
        let events = CompiledEvents::load(&a.event_native, &a.event_source, &a.model, &tokenizer)?;
        let parent = CompiledContext::load(
            &a.context_native,
            &a.context_source,
            a.dependencies(),
            &tokenizer,
        )?;
        let retained =
            ContextWeights::load_source(&a.context_source, a.dependencies(), &tokenizer)?;
        parent.validate_for(&retained)?;
        let source = ReadSourceBinding::from_directory(&a.model, &tokenizer, &potential, 2)?;
        let age = model
            .variables()
            .get("layers.02.read.age")
            .ok_or_else(|| invalid("age absent"))?
            .as_tensor();
        CompiledGeometricRead::compile(age, &potential, &source)?
            .save(&a.out.join("native-read"))?;
        let reducer = CompiledGeometricRead::load(&a.out.join("native-read"), &source)?;
        let value_source =
            ValueSourceBinding::from_directory(&a.model, &tokenizer, &potential, &reducer, 2)?;
        CompiledGeometricValues::compile(&value_source)?.save(&a.out.join("teacher-codec"))?;
        let codec = CompiledGeometricValues::load(&a.out.join("teacher-codec"), &value_source)?;
        Ok((
            Self {
                model,
                parent,
                events,
                span,
                potential,
                reducer,
                codec,
                tokenizer,
                initial_native_values: None,
            },
            retained.into_finite_choice()?,
        ))
    }
    fn value_output(
        &self,
        c: &ContextWeights,
        v: &ValueProducerWeights,
        ids: &[u32],
        b: usize,
        t: usize,
        reset: bool,
    ) -> Result<ValueProducerOutput> {
        let context = c.forward(ids, b, t, reset)?;
        let events = geometric_event::trace_native(ids, b, t, &self.events, false)?;
        let trace =
            geometric_span_native::trace_native_events(ids, b, t, &events.actions, &self.span)?;
        let span_valid = trace
            .prior_codes
            .iter()
            .map(Option::is_some)
            .collect::<Vec<_>>();
        let held =
            geometric_span_native::produce_native_events(ids, b, t, &events.actions, &self.span)?
                .reshape((b, t, 2, 4, 4))?;
        v.forward(
            ids,
            &context.latent_roots,
            &held,
            &span_valid,
            &vec![true; b * t],
        )
    }
}

#[derive(Clone)]
struct Targets {
    roots: Vec<u32>,
    categories: Vec<u32>,
    root_weights: Vec<f32>,
    category_weights: Vec<f32>,
    normalizer: Option<f64>,
}
impl Targets {
    fn from_trace(trace: &ValueProjectionTrace, lengths: &[usize]) -> Result<Self> {
        if trace.batch != lengths.len()
            || trace.heads != 2
            || trace.value_width != 16
            || trace.lanes.len() != trace.batch * 2 * trace.time * 4
            || lengths.iter().any(|&n| n == 0 || n > trace.time)
        {
            return Err(invalid("teacher trace shape"));
        }
        let mut out = Self {
            roots: Vec::new(),
            categories: Vec::new(),
            root_weights: Vec::new(),
            category_weights: Vec::new(),
            normalizer: None,
        };
        // Teacher B,H,T,lane -> predicted B,T,H,lane,atom. No token-role filter.
        for (b, &length) in lengths.iter().enumerate() {
            for t in 0..trace.time {
                for h in 0..2 {
                    for lane in 0..4 {
                        let target = &trace.lanes[((b * 2 + h) * trace.time + t) * 4 + lane];
                        let residual = target
                            .residual_packet
                            .as_ref()
                            .ok_or_else(|| invalid("K2 residual target absent"))?;
                        for packet in [&target.packet, residual] {
                            packet.packet()?;
                            let valid = t < length
                                && target.donor_valid
                                && packet.status != ValuePacketStatus::Absent;
                            let nonzero = packet.status == ValuePacketStatus::PresentNonzero;
                            out.roots.push(u32::from(packet.root));
                            out.categories.push(if nonzero {
                                1 + u32::from(packet.radius_bin)
                            } else {
                                0
                            });
                            out.root_weights
                                .push(if valid && nonzero { 1. } else { 0. });
                            out.category_weights.push(if valid { 1. } else { 0. });
                        }
                    }
                }
            }
        }
        Ok(out)
    }
    fn losses(&self, output: &ValueProducerOutput) -> Result<(Tensor, Tensor)> {
        let n = self.roots.len();
        let root = credit::loss(
            &output.root_logits.reshape((n, 120))?,
            &self.roots,
            &self.root_weights,
            self.normalizer,
        )?;
        let category = credit::loss(
            &output.category_logits.reshape((n, 32))?,
            &self.categories,
            &self.category_weights,
            self.normalizer,
        )?;
        Ok((root, category))
    }
    fn bytes(&self) -> Vec<u8> {
        if self.normalizer.is_some() {
            return self
                .roots
                .iter()
                .zip(&self.categories)
                .zip(self.root_weights.iter().zip(&self.category_weights))
                .flat_map(|((r, c), (rw, cw))| {
                    let mut row = vec![*r as u8, *c as u8];
                    row.extend(rw.to_le_bytes());
                    row.extend(cw.to_le_bytes());
                    row
                })
                .collect();
        }
        self.roots
            .iter()
            .zip(&self.categories)
            .zip(self.root_weights.iter().zip(&self.category_weights))
            .flat_map(|((r, c), (rw, cw))| [*r as u8, *c as u8, *rw as u8, *cw as u8])
            .collect()
    }
}

fn teacher(
    f: &Frozen,
    ids: &[u32],
    lengths: &[usize],
    queries: &[usize],
    time: usize,
    policy: AuxiliaryCredit,
) -> Result<(Targets, Targets, Value, Vec<u8>)> {
    let (_, trace) = f.model.forward_geometric_context_read_native_with_trace(
        ids,
        lengths.len(),
        time,
        &f.parent,
        &f.events,
        &f.span,
        &f.potential,
        &f.reducer,
        false,
    )?;
    let projected =
        project_residual_q16(&trace.values_q16, lengths.len(), 2, time, &f.codec)?.trace;
    let legacy = Targets::from_trace(&projected, lengths)?;
    let (targets, credit, raw) = credit::apply(&legacy, &trace, lengths, queries, policy)?;
    let metadata = json!({"donor_q16_sha256":sha256_bytes(&projected.donor_q16.iter().copied().flat_map(i32::to_le_bytes).collect::<Vec<_>>()),"unique_vectors":projected.unique_vectors,"candidate_distances":projected.candidate_distances,"squared_error_q32":projected.sum_squared_error_q32.to_string(),"max_abs_error_q16":projected.max_abs_error_q16,"credit":credit,"boolean_target_masks_sha256":sha256_bytes(&legacy.bytes())});
    Ok((targets, legacy, metadata, raw))
}

fn selected_packets(output: &ValueProducerOutput, lengths: &[usize], time: usize) -> Value {
    let mut categories = [0usize; 32];
    let mut roots = [0usize; 120];
    for (b, &length) in lengths.iter().enumerate() {
        for h in 0..2 {
            for t in 0..length {
                let at = ((b * 2 + h) * time + t) * 4;
                for packet in output.trace.packets[at..at + 4].iter().flatten() {
                    match packet.status {
                        ValuePacketStatus::PresentZero => categories[0] += 1,
                        ValuePacketStatus::PresentNonzero => {
                            categories[1 + usize::from(packet.radius_bin)] += 1;
                            roots[usize::from(packet.root)] += 1;
                        }
                        ValuePacketStatus::Absent => {}
                    }
                }
            }
        }
    }
    json!({"category_counts":categories.to_vec(),"nonzero_root_counts":roots.to_vec(),"actual_atoms":lengths.iter().sum::<usize>()*16,"zero_atoms":categories[0]})
}

#[derive(Deserialize)]
struct PriorRow {
    index: usize,
    episode: Episode,
    baseline_prediction: u32,
    projected_prediction: u32,
    baseline_answer_logits: Vec<f32>,
}
struct Priors {
    donor: Vec<u32>,
    donor_logits: Vec<Vec<f32>>,
    k1: Vec<u32>,
    k2: Vec<u32>,
    continuation: Option<continuation::PanelReference>,
}
fn priors(a: &Args, panel_name: &str, episodes: &[Episode]) -> Result<Priors> {
    let p1 = a.prior_k1.join(panel_name).join("rows.json");
    let p2 = a.prior_k2.join(panel_name).join("rows.json");
    // These exact accepted reports include legacy K1 receipts that predate
    // the explicit representation field. Pin both report and consumed rows,
    // rather than interpreting a missing field as permission for any parent.
    const ACCEPTED: [[&str; 3]; 4] = [
        [
            "ebde8eae6fb47f0e64747dd721686a8c132f22a126442cce11242bc093ef4d53",
            "3860097e9e65cd0b008d199ae2495c9449e40d15a6a6d80923740f5cc8d28da0",
            "5ce16cc619eec974ac1239ff32fb7b67b44ec36cb3331b58528f7bbda0ad7e97",
        ],
        [
            "862a4f7909b9dfa06c4aa6c75b39f520d693dd35f657099253b23c5e8eebe764",
            "f0c269fc342abeacf3ab686a664de19cc8e575e777c901c7131e692df30bc0a4",
            "6c0124fdc66ced6e4777306a0a47946227cfe849d766c147794896c878c141a9",
        ],
        [
            "4c774e223737ed7477fdd82ef4472b67fdedfe05f79bc5a6961b023242c74d93",
            "016220e88b66133ad47732a8c739c65860a1ee7f07b6d048bc1fa2c1c46a4f0c",
            "5a7d72f2ec18cc737fa3333394a90f8138ed8f52288cc9ee18d5e96703b50923",
        ],
        [
            "5d2d9c1b7063e45843f7d517fea1a04f1350da7355e4246b88c331d39cc389b5",
            "f8cd0b8124fb056555b74187b11d3c3e5568d24e37c656c9e961abaab11f3352",
            "e7034a0acc2caa6254f33f65e9e5a62fa5859b7784a2f21669e9b98827310242",
        ],
    ];
    let column = match panel_name {
        "original" => 1,
        "stress" => 2,
        _ => return Err(invalid("unknown prior panel")),
    };
    for (kind, root) in [&a.prior_k1, &a.prior_k2].into_iter().enumerate() {
        let expected = ACCEPTED[kind * 2 + (a.seed - 1) as usize];
        if sha256_file(&root.join("report.json"))? != expected[0]
            || sha256_file(&root.join(panel_name).join("rows.json"))? != expected[column]
        {
            return Err(invalid("accepted prior K1/K2 report or rows differ"));
        }
    }
    let left: Vec<PriorRow> = serde_json::from_slice(&fs::read(&p1)?)?;
    let right: Vec<PriorRow> = serde_json::from_slice(&fs::read(&p2)?)?;
    if left.len() != episodes.len() || right.len() != episodes.len() {
        return Err(invalid("prior rows incomplete"));
    }
    let mut out = Priors {
        donor: Vec::new(),
        donor_logits: Vec::new(),
        k1: Vec::new(),
        k2: Vec::new(),
        continuation: None,
    };
    for (i, ((x, y), e)) in left.iter().zip(&right).zip(episodes).enumerate() {
        if x.index != i
            || y.index != i
            || x.episode != *e
            || y.episode != *e
            || x.baseline_prediction != y.baseline_prediction
            || argmax(&x.baseline_answer_logits)? != x.baseline_prediction
            || argmax(&y.baseline_answer_logits)? != y.baseline_prediction
            || x.baseline_answer_logits
                .iter()
                .map(|x| x.to_bits())
                .ne(y.baseline_answer_logits.iter().map(|x| x.to_bits()))
        {
            return Err(invalid("prior episode/donor identity mismatch"));
        }
        out.donor.push(x.baseline_prediction);
        out.donor_logits.push(x.baseline_answer_logits.clone());
        out.k1.push(x.projected_prediction);
        out.k2.push(y.projected_prediction);
    }
    write_json(
        &a.out.join(format!("prior-{panel_name}-identity.json")),
        &json!({"k1":p1,"k1_sha256":sha256_file(&p1)?,"k2":p2,"k2_sha256":sha256_file(&p2)?}),
    )?;
    Ok(out)
}

fn checkpoint(
    a: &Args,
    f: &Frozen,
    c: &ContextWeights,
    v: &ValueProducerWeights,
    directory: &Path,
    step: usize,
    rng: u64,
) -> Result<()> {
    fs::create_dir(directory)?;
    c.save_source(
        &directory.join("context-source"),
        a.dependencies(),
        &f.tokenizer,
    )?;
    v.save_source(&directory.join("value-source"))?;
    write_json(
        &directory.join("checkpoint.json"),
        &json!({"completed_updates":step,"next_generator_state":rng,"optimizer_state":"NOT_SERIALIZED; weight recovery only, no exact AdamW resume","args":a,"context_config":c.config(),"value_config":v.config()}),
    )
}

struct Reloaded {
    context: ContextWeights,
    values: ValueProducerWeights,
    native_context: CompiledContext,
    native_values: CompiledValueProducer,
}
fn compile_reload(a: &Args, f: &Frozen, root: &Path) -> Result<Reloaded> {
    let cs = root.join("context-source");
    let vs = root.join("value-source");
    let context = ContextWeights::load_source(&cs, a.dependencies(), &f.tokenizer)?;
    let values = ValueProducerWeights::load_source(&vs)?;
    CompiledContext::compile(&context, &cs, a.dependencies(), &f.tokenizer)?
        .save(&root.join("native-context"))?;
    let native_context = CompiledContext::load(
        &root.join("native-context"),
        &cs,
        a.dependencies(),
        &f.tokenizer,
    )?;
    let paths = ValueProducerSourcePaths {
        value_source: &vs,
        context_source: &cs,
        context_dependencies: a.dependencies(),
    };
    CompiledValueProducer::compile(paths, &f.tokenizer)?.save(&root.join("native-values"))?;
    let native_values =
        CompiledValueProducer::load(&root.join("native-values"), paths, &f.tokenizer)?;
    native_values.validate_native_context(&native_context)?;
    Ok(Reloaded {
        context,
        values,
        native_context,
        native_values,
    })
}

fn answer_rows(
    logits: &[Vec<f32>],
    group: &[Episode],
    time: usize,
) -> Result<Vec<(u32, Vec<f32>)>> {
    validate_logits(logits, group.len() * time)?;
    group
        .iter()
        .enumerate()
        .map(|(b, e)| {
            let row = &logits[b * time + e.query];
            Ok((argmax(row)?, row.clone()))
        })
        .collect()
}
fn masses(
    f: &Frozen,
    c: &ContextWeights,
    ids: &[u32],
    group: &[Episode],
    time: usize,
    reset: bool,
) -> Result<[Vec<f32>; 2]> {
    let mut out = [Vec::new(), Vec::new()];
    for (head, values) in out.iter_mut().enumerate() {
        let target = ReadBindingTarget {
            layer: 2,
            head,
            rows: group
                .iter()
                .enumerate()
                .map(|(batch, e)| ReadBinding {
                    batch,
                    query: e.query,
                    sources: vec![e.source],
                })
                .collect(),
        };
        *values = f
            .model
            .read_binding_masses_geometric_context(
                ids,
                group.len(),
                time,
                &target,
                c,
                &f.events,
                &f.span,
                reset,
            )?
            .to_vec1::<f32>()?;
        if values.len() != group.len()
            || values.iter().any(|x| !x.is_finite() || *x < 0. || *x > 1.)
        {
            return Err(invalid("source mass outside probability range"));
        }
    }
    Ok(out)
}
fn selected_trace(trace: &NativeReadTrace, b: usize, e: &Episode) -> Result<Value> {
    let mut heads = Vec::new();
    for h in 0..2 {
        let index = (b * 2 + h) * trace.time + e.query;
        let source_index = (b * 2 + h) * trace.time + e.source;
        heads.push(json!({"source_fraction":trace.source_fraction(b,h,e.query,&[e.source])?,"source_mass":trace.source_mass(b,h,e.query,&[e.source])?,"no_read_mass":trace.no_read_mass(b,h,e.query)?,"query_reduction":trace.rows.get(index).ok_or_else(||invalid("missing native query row"))?,"source_value_q16":&trace.values_q16[source_index*16..(source_index+1)*16]}));
    }
    Ok(json!(heads))
}
fn max_delta(a: &[Vec<f32>], b: &[Vec<f32>], group: &[Episode], time: usize) -> f64 {
    group
        .iter()
        .enumerate()
        .flat_map(|(row, e)| (0..e.ids.len()).map(move |t| row * time + t))
        .flat_map(|at| a[at].iter().zip(&b[at]))
        .map(|(x, y)| (f64::from(*x) - f64::from(*y)).abs())
        .fold(0., f64::max)
}

fn prefit(
    f: &Frozen,
    c: &ContextWeights,
    v: &ValueProducerWeights,
    episodes: &[Episode],
    prior: &Priors,
    out: &Path,
    deadline: Instant,
    limit: usize,
) -> Result<(Value, Vec<u32>)> {
    fs::create_dir(out)?;
    let mut rows = Vec::new();
    let mut predictions = Vec::new();
    for (chunk, group) in episodes[..limit].chunks(BATCH).enumerate() {
        if Instant::now() >= deadline {
            break;
        }
        let (ids, _, _, time) = batch(group);
        let (donor, donor_trace) = f.model.forward_geometric_context_read_native_with_trace(
            &ids,
            group.len(),
            time,
            &f.parent,
            &f.events,
            &f.span,
            &f.potential,
            &f.reducer,
            false,
        )?;
        let logits = f
            .model
            .forward_geometric_context_learned_values(
                &ids,
                group.len(),
                time,
                c,
                v,
                &f.events,
                &f.span,
                false,
            )?
            .to_vec2::<f32>()?;
        let answers = answer_rows(&logits, group, time)?;
        let donor_answers = answer_rows(&donor.to_vec2::<f32>()?, group, time)?;
        let native_answers = if prior.continuation.is_some() {
            let native = f
                .initial_native_values
                .as_ref()
                .ok_or_else(|| invalid("continued native values absent"))?;
            let (logits, _, _) = f
                .model
                .forward_geometric_context_learned_values_native_with_trace(
                    &ids,
                    group.len(),
                    time,
                    &f.parent,
                    &f.events,
                    &f.span,
                    &f.potential,
                    &f.reducer,
                    native,
                    false,
                )?;
            Some(answer_rows(&logits.to_vec2::<f32>()?, group, time)?)
        } else {
            None
        };
        let produced = f.value_output(c, v, &ids, group.len(), time, false)?;
        for (b, e) in group.iter().enumerate() {
            let index = chunk * BATCH + b;
            if let Some(reference) = &prior.continuation {
                reference.validate_initial(
                    index,
                    &answers[b],
                    native_answers
                        .as_ref()
                        .and_then(|v| v.get(b))
                        .ok_or_else(|| invalid("continued native row absent"))?,
                    &donor_answers[b],
                )?;
            } else if donor_answers[b].0 != prior.donor[index]
                || donor_answers[b]
                    .1
                    .iter()
                    .map(|x| x.to_bits())
                    .ne(prior.donor_logits[index].iter().map(|x| x.to_bits()))
            {
                return Err(invalid(
                    "loaded retained donor prediction differs from prior reports",
                ));
            }
            let packets = (0..2)
                .map(|h| {
                    let at = ((b * 2 + h) * time + e.source) * 4;
                    produced.trace.packets[at..at + 4].to_vec()
                })
                .collect::<Vec<_>>();
            let mut record = json!({"index":index,"episode":e,"donor_prediction":donor_answers[b].0,"donor_answer_logits":donor_answers[b].1,"prior_k1":prior.k1[index],"prior_k2":prior.k2[index],"source_packets":packets,"donor_read":selected_trace(&donor_trace,b,e)?});
            if prior.continuation.is_some() {
                record["continued_prediction"] = json!(answers[b].0);
                record["continued_answer_logits"] = json!(answers[b].1);
                record["continued_native"] = json!(native_answers.as_ref().and_then(|v| v.get(b)));
            } else {
                record["untrained_prediction"] = json!(answers[b].0);
                record["untrained_answer_logits"] = json!(answers[b].1);
            }
            rows.push(record);
            predictions.push(answers[b].0);
        }
        write_json(&out.join("rows.json"), &rows)?;
    }
    let mut summary = json!({"complete":rows.len()==limit,"requested_rows":limit,"scored_rows":rows.len(),"donor_answers":rows.iter().zip(episodes).filter(|(r,e)|r["donor_prediction"]==e.answer).count()});
    let correct = predictions
        .iter()
        .zip(episodes)
        .filter(|(p, e)| **p == e.answer)
        .count();
    if prior.continuation.is_some() {
        summary["continued_answers"] = json!(correct);
        summary["scope"]=json!("saved learned parent, zero new updates; actual float/native logits and new-address donor bound to retained rows");
    } else {
        summary["untrained_answers"] = json!(correct);
        summary["scope"] = json!(
            "actual learned-head initialization, zero updates; frozen native donor comparison"
        );
    }
    write_json(&out.join("summary.json"), &summary)?;
    Ok((summary, predictions))
}

fn score(
    f: &Frozen,
    r: &Reloaded,
    episodes: &[Episode],
    prior: &Priors,
    initial: &[u32],
    out: &Path,
    deadline: Instant,
    limit: usize,
) -> Result<Value> {
    fs::create_dir(out)?;
    let mut rows = Vec::new();
    let mut differences = BTreeMap::<String, usize>::new();
    let mut maximum_logit_delta = 0f64;
    let mut original_state_drift = 0usize;
    let mut action_drift = 0usize;
    let mut address_drift = 0usize;
    let mut native_state_diff = 0usize;
    let mut native_action_diff = 0usize;
    let mut native_address_diff = 0usize;
    let mut packet_diff = 0usize;
    let mut coordinate_diff = 0usize;
    for (chunk, group) in episodes[..limit].chunks(BATCH).enumerate() {
        if Instant::now() >= deadline {
            break;
        }
        let (ids, _, _, time) = batch(group);
        let bsz = group.len();
        let float = f
            .model
            .forward_geometric_context_learned_values(
                &ids, bsz, time, &r.context, &r.values, &f.events, &f.span, false,
            )?
            .to_vec2::<f32>()?;
        let float_reset = f
            .model
            .forward_geometric_context_learned_values(
                &ids, bsz, time, &r.context, &r.values, &f.events, &f.span, true,
            )?
            .to_vec2::<f32>()?;
        let (native, native_trace, native_values) = f
            .model
            .forward_geometric_context_learned_values_native_with_trace(
                &ids,
                bsz,
                time,
                &r.native_context,
                &f.events,
                &f.span,
                &f.potential,
                &f.reducer,
                &r.native_values,
                false,
            )?;
        let (reset, reset_trace, reset_values) = f
            .model
            .forward_geometric_context_learned_values_native_with_trace(
                &ids,
                bsz,
                time,
                &r.native_context,
                &f.events,
                &f.span,
                &f.potential,
                &f.reducer,
                &r.native_values,
                true,
            )?;
        let native = native.to_vec2::<f32>()?;
        let reset = reset.to_vec2::<f32>()?;
        for logits in [&float, &float_reset, &native, &reset] {
            validate_logits(logits, ids.len())?;
        }
        let float_mass = masses(f, &r.context, &ids, group, time, false)?;
        let reset_mass = masses(f, &r.context, &ids, group, time, true)?;
        let native_context =
            geometric_context::trace_native(&ids, bsz, time, &r.native_context, false)?;
        let parent_context = geometric_context::trace_native(&ids, bsz, time, &f.parent, false)?;
        let float_context = r.context.float_trace(&ids, bsz, time, false)?;
        let float_codes = float_context.address_codes()?;
        let float_values = f
            .value_output(&r.context, &r.values, &ids, bsz, time, false)?
            .trace;
        let restored = f
            .model
            .forward_geometric_context_learned_values(
                &ids, bsz, time, &r.context, &r.values, &f.events, &f.span, false,
            )?
            .to_vec2::<f32>()?;
        if float
            .iter()
            .flatten()
            .map(|x| x.to_bits())
            .ne(restored.iter().flatten().map(|x| x.to_bits()))
        {
            return Err(invalid("float reference changed after native/reset calls"));
        }
        let predictions = [
            answer_rows(&float, group, time)?,
            answer_rows(&native, group, time)?,
            answer_rows(&float_reset, group, time)?,
            answer_rows(&reset, group, time)?,
        ];
        maximum_logit_delta = maximum_logit_delta.max(max_delta(&float, &native, group, time));
        for (b, e) in group.iter().enumerate() {
            let index = chunk * BATCH + b;
            let mut row_packet_diff = 0;
            let mut row_coord_diff = 0;
            for t in 0..e.ids.len() {
                let at = b * time + t;
                original_state_drift +=
                    usize::from(parent_context.states[at] != native_context.states[at]);
                action_drift +=
                    usize::from(parent_context.actions[at] != native_context.actions[at]);
                address_drift += parent_context.codes[at * 8..(at + 1) * 8]
                    .iter()
                    .zip(&native_context.codes[at * 8..(at + 1) * 8])
                    .filter(|(x, y)| x != y)
                    .count();
                native_state_diff +=
                    usize::from(float_context.states[at] != native_context.states[at]);
                native_action_diff +=
                    usize::from(float_context.actions[at] != native_context.actions[at]);
                native_address_diff += float_codes[at * 8..(at + 1) * 8]
                    .iter()
                    .zip(&native_context.codes[at * 8..(at + 1) * 8])
                    .filter(|(x, y)| {
                        x.root() != y.root()
                            || x.radius_bin() != y.radius_bin()
                            || x.present() != y.present()
                    })
                    .count();
                for h in 0..2 {
                    let p = ((b * 2 + h) * time + t) * 4;
                    let q = p * 4;
                    row_packet_diff += float_values.packets[p..p + 4]
                        .iter()
                        .zip(&native_values.packets[p..p + 4])
                        .flat_map(|(x, y)| x.iter().zip(y))
                        .filter(|(x, y)| x != y)
                        .count();
                    row_coord_diff += float_values.values_q16[q..q + 16]
                        .iter()
                        .zip(&native_values.values_q16[q..q + 16])
                        .filter(|(x, y)| x != y)
                        .count();
                }
            }
            packet_diff += row_packet_diff;
            coordinate_diff += row_coord_diff;
            let p: Vec<u32> = predictions.iter().map(|x| x[b].0).collect();
            let mut comparisons = vec![
                ("donor", prior.donor[index]),
                ("k1", prior.k1[index]),
                ("k2", prior.k2[index]),
                ("initial", initial[index]),
            ];
            if let Some(c) = &prior.continuation {
                comparisons.extend([
                    ("learned_parent", c.learned[index].native_prediction),
                    ("parent_address_donor", c.oracle[index].baseline_prediction),
                    ("parent_address_k2", c.oracle[index].projected_prediction),
                ]);
            }
            for (name, old) in comparisons {
                *differences
                    .entry(format!("native_changed_from_{name}"))
                    .or_default() += usize::from(p[1] != old);
                *differences
                    .entry(format!("native_gain_from_{name}"))
                    .or_default() += usize::from(p[1] == e.answer && old != e.answer);
                *differences
                    .entry(format!("native_loss_from_{name}"))
                    .or_default() += usize::from(p[1] != e.answer && old == e.answer);
            }
            let source_packets=(0..2).map(|h|{let at=((b*2+h)*time+e.source)*4;json!({"float":float_values.packets[at..at+4],"native":native_values.packets[at..at+4],"native_reset":reset_values.packets[at..at+4]})}).collect::<Vec<_>>();
            rows.push(json!({"index":index,"episode":e,"prior_donor":prior.donor[index],"prior_k1":prior.k1[index],"prior_k2":prior.k2[index],"initial":initial[index],"continued_parent":prior.continuation.as_ref().map(|c|json!({"learned":c.learned[index].native_prediction,"donor":c.oracle[index].baseline_prediction,"k2":c.oracle[index].projected_prediction})),"float_prediction":p[0],"native_prediction":p[1],"float_reset_prediction":p[2],"native_reset_prediction":p[3],"answer_logits":{"float":predictions[0][b].1,"native":predictions[1][b].1,"float_reset":predictions[2][b].1,"native_reset":predictions[3][b].1},"float_source_masses":[float_mass[0][b],float_mass[1][b]],"float_reset_source_masses":[reset_mass[0][b],reset_mass[1][b]],"native_read":selected_trace(&native_trace,b,e)?,"native_reset_read":selected_trace(&reset_trace,b,e)?,"source_packets":source_packets,"native_packet_disagreements":row_packet_diff,"native_coordinate_disagreements":row_coord_diff,"source_states":native_context.states[b*time+e.source],"query_states":native_context.states[b*time+e.query]}));
        }
        write_json(&out.join("rows.json"), &rows)?;
    }
    let mut answers = BTreeMap::new();
    for key in [
        "float_prediction",
        "native_prediction",
        "float_reset_prediction",
        "native_reset_prediction",
    ] {
        answers.insert(
            key,
            rows.iter()
                .filter(|r| r[key] == r["episode"]["answer"])
                .count(),
        );
    }
    let summary = json!({"complete":rows.len()==limit,"requested_rows":limit,"scored_rows":rows.len(),"answers":answers,"rowwise_prior_comparisons":differences,"float_native_answer_changes":rows.iter().filter(|r|r["float_prediction"]!=r["native_prediction"]).count(),"reset_changed_predictions":rows.iter().filter(|r|r["native_prediction"]!=r["native_reset_prediction"]).count(),"maximum_actual_logit_delta":maximum_logit_delta,"parent_state_position_changes":original_state_drift,"parent_action_position_changes":action_drift,"parent_address_lane_changes":address_drift,"native_state_position_disagreements":native_state_diff,"native_action_position_disagreements":native_action_diff,"native_address_lane_disagreements":native_address_diff,"native_packet_disagreements":packet_diff,"native_coordinate_disagreements":coordinate_diff,"scope":"all comparisons authored development; float source masses use separate identical scoring pass; native query rows from actual prediction pass; reset affects context state only, preserving span memory; joint answer credit includes address and value paths"});
    write_json(&out.join("summary.json"), &summary)?;
    Ok(summary)
}

fn training_batch(rng: &mut Rng, step: usize) -> Vec<Episode> {
    let n = [2, 4][step % 2];
    (0..BATCH)
        .map(|_| {
            let facts = draw(rng, n);
            let q = rng.next(n);
            let wn = (0..n).map(|_| noise(rng, 4)).collect::<Vec<_>>();
            let qn = noise(rng, 4);
            episode(&facts, &wn, &facts[q].0, &qn, step, "joint_train")
        })
        .collect()
}
fn check_frozen(
    context: &ContextWeights,
    before: &BTreeMap<String, String>,
    model: &StackModel,
    base: &BTreeMap<String, String>,
) -> Result<()> {
    for (name, hash) in parameter_hashes(context.parameters())? {
        if !TRANSITIONS.contains(&name.as_str()) && before.get(&name) != Some(&hash) {
            return Err(invalid("frozen context observation coefficient changed"));
        }
    }
    if &parameter_hashes(model.variables())? != base {
        return Err(invalid("frozen base parameter changed"));
    }
    Ok(())
}
fn run(a: &Args, start: Instant) -> Result<Value> {
    let deadline = start + Duration::from_secs(a.max_seconds);
    let fit_deadline = deadline - Duration::from_secs(a.evaluation_reserve_seconds);
    let evaluation = panel(&a.evaluation, EVAL_HASH, &a.out.join("evaluation.json"))?;
    let stress = panel(&a.stress, STRESS_HASH, &a.out.join("stress.json"))?;
    let mut original_prior = priors(a, "original", &evaluation)?;
    let mut stress_prior = priors(a, "stress", &stress)?;
    let continuation = Continuation::load(a, &evaluation, &stress)?;
    if let Some(c) = &continuation {
        original_prior.continuation = Some(c.original.clone());
        stress_prior.continuation = Some(c.stress.clone());
        write_json(&a.out.join("continuation-parent.json"), &c.metadata)?;
    }
    let mut inventory = BTreeMap::new();
    for (name, path) in [
        ("base", &a.model),
        ("context_source", &a.context_source),
        ("context_native", &a.context_native),
        ("event_source", &a.event_source),
        ("event_native", &a.event_native),
        ("span_native", &a.span_native),
        ("potential_native", &a.potential_native),
    ] {
        inventory.insert(name, file_inventory(path)?);
    }
    let executable = std::env::current_exe()?;
    write_json(
        &a.out.join("inputs.json"),
        &json!({"args":a,"artifacts":inventory,"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNAVAILABLE"),"executable":executable,"executable_sha256":sha256_file(&executable)?,"data":"new sequential authored draws from unchanged context generator; pinned original/stress development panels","teacher_format":if continuation.is_some(){"B,T,H,lane,atom: root:u8/category:u8/rootweight:LEf32/categoryweight:LEf32; query trace separate LEu64; frozen teacher masks"}else{"per step u8 root,category,root_mask,category_mask tuples in B,T,H,lane,atom order; masks exclude padding and absent atoms; root excludes zero"}}),
    )?;
    if Instant::now() >= fit_deadline {
        return Ok(json!({"complete":false,"decision":"PARTIAL_BEFORE_LOAD"}));
    }
    let (mut f, context) = Frozen::load(a)?;
    let values = if let Some(c) = &continuation {
        let (values, native) = c.values(a, &f, &context)?;
        f.initial_native_values = Some(native);
        values
    } else {
        ValueProducerWeights::new(40, 2, 4, 940100 + a.seed)?
    };
    let context_before = parameter_hashes(context.parameters())?;
    let base_before = parameter_hashes(f.model.variables())?;
    let vars = trainable(&context, &values)?;
    write_json(
        &a.out.join("parameter-inventory.json"),
        &json!({"trainable":vars.iter().map(|(n,v)|json!({"name":n,"shape":v.dims(),"elements":v.elem_count()})).collect::<Vec<_>>(),"context_before":context_before,"base_before":base_before,"value_config":values.config(),"context_config":context.config(),"optimizer":{"rate":a.rate,"beta1":0.9,"beta2":0.95,"eps":1e-8,"weight_decay":0.,"global_trainable_clip_l2":1.},"value_before":parameter_hashes(values.parameters())?,"auxiliary_credit":a.auxiliary_credit,"loss":if continuation.is_some(){"answer_mean + .1*(root_weighted_numerator/J + category_weighted_numerator/J); J=B*H*4*2; matched eligible mass; NoRead attenuation retained; moments restarted"}else{"answer_mean + .1 * (root_actual_nonzero_mean + category_actual_valid_mean)"}}),
    )?;
    let generator_seed = continuation.as_ref().map_or(940019 + a.seed, |c| c.rng);
    let parent_updates = continuation.as_ref().map_or(0, |c| c.updates);
    checkpoint(
        a,
        &f,
        &context,
        &values,
        &a.out.join("initial"),
        parent_updates,
        generator_seed,
    )?;
    let limit = if a.mode == "check" { BATCH } else { 128 };
    let (initial_original, initial_predictions) = prefit(
        &f,
        &context,
        &values,
        &evaluation,
        &original_prior,
        &a.out.join("initial-original"),
        fit_deadline,
        limit,
    )?;
    let (initial_stress, initial_stress_predictions) = prefit(
        &f,
        &context,
        &values,
        &stress,
        &stress_prior,
        &a.out.join("initial-stress"),
        fit_deadline,
        limit,
    )?;
    if initial_original["complete"] != true || initial_stress["complete"] != true {
        return Ok(
            json!({"complete":false,"decision":"PARTIAL_PREFIT_NO_QUALITY_VERDICT","original":initial_original,"stress":initial_stress}),
        );
    }
    let mut optimizer = AdamW::new(
        vars.iter().map(|(_, v)| v.clone()).collect(),
        ParamsAdamW {
            lr: a.rate,
            beta1: 0.9,
            beta2: 0.95,
            eps: 1e-8,
            weight_decay: 0.,
        },
    )?;
    let mut rng = Rng(generator_seed);
    let mut last_checkpoint = Instant::now();
    let mut history = Vec::new();
    let mut completed_backward_batches = 0usize;
    let (
        mut updates,
        mut actual_positions,
        mut padded_positions,
        mut episodes_count,
        mut batches_prepared,
    ) = (0usize, 0usize, 0usize, 0usize, 0usize);
    let (mut preparation_seconds, mut training_seconds) = (0f64, 0f64);
    let mut max_time = 0usize;
    let mut min_time = usize::MAX;
    let teacher_root = a.out.join("training-targets");
    fs::create_dir(&teacher_root)?;
    let iterations = if a.mode == "check" { 1 } else { a.steps };
    for step in 0..iterations {
        if Instant::now() >= fit_deadline {
            break;
        }
        let rng_before = rng.0;
        let absolute_step = parent_updates
            .checked_add(step)
            .ok_or_else(|| invalid("absolute step overflow"))?;
        let es = training_batch(&mut rng, absolute_step);
        let (ids, targets, weights, time) = batch(&es);
        if time > 128 {
            return Err(invalid("training exceeds full causal context128"));
        }
        let lengths = es.iter().map(|e| e.ids.len()).collect::<Vec<_>>();
        let stem = format!("step-{step:04}");
        write_json(
            &teacher_root.join(format!("{stem}.json")),
            &json!({"step":step,"generator_before":rng_before,"generator_after":rng.0,"episodes":es,"time":time,"teacher_status":"PENDING"}),
        )?;
        let prep_start = Instant::now();
        let queries = es.iter().map(|e| e.query).collect::<Vec<_>>();
        let (labels, legacy_labels, teacher_metadata, query_bytes) =
            teacher(&f, &ids, &lengths, &queries, time, a.auxiliary_credit)?;
        let label_bytes = labels.bytes();
        fs::write(teacher_root.join(format!("{stem}.bin")), &label_bytes)?;
        if !query_bytes.is_empty() {
            fs::write(teacher_root.join(format!("{stem}.query.bin")), &query_bytes)?;
            fs::write(
                teacher_root.join(format!("{stem}.teacher.bin")),
                legacy_labels.bytes(),
            )?;
        }
        write_json(
            &teacher_root.join(format!("{stem}.json")),
            &json!({"step":step,"absolute_step":absolute_step,"generator_before":rng_before,"generator_after":rng.0,"episodes":es,"draw_sha256":sha256_bytes(&serde_json::to_vec(&es)?),"time":time,"teacher":teacher_metadata,"target_sha256":sha256_bytes(&label_bytes),"target_bytes":label_bytes.len(),"order":if labels.normalizer.is_some(){"B,T,H,lane,atom; root:u8/category:u8/rootweight:LEf32/categoryweight:LEf32"}else{"B,T,H,lane,atom; root/category/rootmask/categorymask bytes"}}),
        )?;
        preparation_seconds += prep_start.elapsed().as_secs_f64();
        batches_prepared += 1;
        if Instant::now() >= fit_deadline {
            break;
        }
        let train_start = Instant::now();
        let logits = f.model.forward_geometric_context_learned_values(
            &ids, BATCH, time, &context, &values, &f.events, &f.span, false,
        )?;
        let answer = logits_cross_entropy(&logits, &targets, Some(&weights))?;
        let output = f.value_output(&context, &values, &ids, BATCH, time, false)?;
        let (root, category) = labels.losses(&output)?;
        let legacy_losses = if a.auxiliary_credit != AuxiliaryCredit::Legacy {
            let (lr, lc) = legacy_labels.losses(&output)?;
            Some([lr.to_scalar::<f32>()?, lc.to_scalar::<f32>()?])
        } else {
            None
        };
        let auxiliary = (&root + &category)?.affine(0.1, 0.)?;
        let total = (&answer + &auxiliary)?;
        let record =
            step % 40 == 0 || step + 1 == iterations || [80, 160, 320, 640].contains(&(step + 1));
        let mut pure = BTreeMap::new();
        let mut auxiliary_gradients = BTreeMap::new();
        if record {
            pure = gradient_families(&vars, &answer.backward()?)?;
            auxiliary_gradients = gradient_families(&vars, &auxiliary.backward()?)?;
            if step == 0 {
                let context_credit = pure
                    .iter()
                    .filter(|(n, _)| n.starts_with("context."))
                    .map(|(_, v)| v * v)
                    .sum::<f64>();
                let value_credit = pure
                    .iter()
                    .filter(|(n, _)| n.starts_with("value."))
                    .map(|(_, v)| v * v)
                    .sum::<f64>();
                if context_credit <= 0. || value_credit <= 0. {
                    write_json(
                        &a.out.join("gradient-failure.json"),
                        &json!({"step":step,"ordinary_answer_mean":answer.to_scalar::<f32>()?,"root_mean":root.to_scalar::<f32>()?,"category_mean":category.to_scalar::<f32>()?,"joint_answer_gradient_l2":pure,"scaled_auxiliary_gradient_l2":auxiliary_gradients,"actual_training_input":format!("training-targets/{stem}.json"),"selected_packets":selected_packets(&output,&lengths,time)}),
                    )?;
                    return Err(invalid("actual retained-parent ordinary answer gradient missing from context or value families"));
                }
            }
        }
        let mut gradients = total.backward()?;
        let norm = clip(&vars, &mut gradients)?;
        let losses = [
            answer.to_scalar::<f32>()?,
            root.to_scalar::<f32>()?,
            category.to_scalar::<f32>()?,
            total.to_scalar::<f32>()?,
        ];
        if losses.iter().any(|x| !x.is_finite()) {
            return Err(invalid("nonfinite joint objective"));
        }
        completed_backward_batches += 1;
        if a.mode == "fit" {
            optimizer.step(&gradients)?;
            updates += 1;
        }
        actual_positions += lengths.iter().sum::<usize>();
        padded_positions += ids.len();
        episodes_count += es.len();
        max_time = max_time.max(time);
        min_time = min_time.min(time);
        training_seconds += train_start.elapsed().as_secs_f64();
        if record {
            history.push(json!({"batch_step":step+1,"completed_updates":updates,"time":time,"actual_positions":lengths.iter().sum::<usize>(),"answer_denominator":weights.iter().sum::<f32>(),"root_denominator":labels.normalizer.unwrap_or_else(||labels.root_weights.iter().map(|x|f64::from(*x)).sum()),"category_denominator":labels.normalizer.unwrap_or_else(||labels.category_weights.iter().map(|x|f64::from(*x)).sum()),"root_weight_sum":labels.root_weights.iter().map(|x|f64::from(*x)).sum::<f64>(),"category_weight_sum":labels.category_weights.iter().map(|x|f64::from(*x)).sum::<f64>(),"legacy_global_position_means":legacy_losses,"auxiliary_credit":a.auxiliary_credit,"absolute_batch_step":absolute_step+1,"cumulative_updates":parent_updates+updates,"answer_mean":losses[0],"root_mean":losses[1],"category_mean":losses[2],"scaled_auxiliary":0.1f64*(f64::from(losses[1])+f64::from(losses[2])),"total_loss":losses[3],"joint_answer_gradient_l2":pure,"scaled_auxiliary_gradient_l2":auxiliary_gradients,"unclipped_trainable_gradient_l2":norm,"selected_packets":selected_packets(&output,&lengths,time),"loss_timing":"pre-update; saved checkpoint after update","elapsed_seconds":start.elapsed().as_secs_f64()}));
            write_json(&a.out.join("history.json"), &history)?;
        }
        if a.mode == "fit" && (updates % 80 == 0 || last_checkpoint.elapsed().as_secs() >= 900) {
            check_frozen(&context, &context_before, &f.model, &base_before)?;
            checkpoint(
                a,
                &f,
                &context,
                &values,
                &a.out.join(format!("checkpoint-{updates:04}")),
                parent_updates + updates,
                rng.0,
            )?;
            last_checkpoint = Instant::now();
        }
    }
    check_frozen(&context, &context_before, &f.model, &base_before)?;
    let final_root = a.out.join("trained");
    checkpoint(
        a,
        &f,
        &context,
        &values,
        &final_root,
        parent_updates + updates,
        rng.0,
    )?;
    let loaded = compile_reload(a, &f, &final_root)?;
    if parameter_hashes(context.parameters())? != parameter_hashes(loaded.context.parameters())?
        || parameter_hashes(values.parameters())? != parameter_hashes(loaded.values.parameters())?
    {
        return Err(invalid("independent source reload parameters differ"));
    }
    let original = score(
        &f,
        &loaded,
        &evaluation,
        &original_prior,
        &initial_predictions,
        &a.out.join("original"),
        deadline,
        limit,
    )?;
    let stress_result = score(
        &f,
        &loaded,
        &stress,
        &stress_prior,
        &initial_stress_predictions,
        &a.out.join("stress"),
        deadline,
        limit,
    )?;
    let complete = original["complete"] == true
        && stress_result["complete"] == true
        && (a.mode == "check" && completed_backward_batches == 1
            || a.mode == "fit" && updates == a.steps);
    let compilation_changes = [&original, &stress_result].iter().any(|s| {
        s["native_packet_disagreements"] != 0
            || s["native_state_position_disagreements"] != 0
            || s["native_action_position_disagreements"] != 0
            || s["native_address_lane_disagreements"] != 0
            || s["native_coordinate_disagreements"] != 0
            || s["float_native_answer_changes"] != 0
    });
    let decision = if !complete {
        "PARTIAL_BUDGET_NO_QUALITY_VERDICT"
    } else if a.mode == "check" {
        "CONSTRUCTION_AND_ACTUAL_PARENT_GRADIENT_ONLY_NO_FIT"
    } else if compilation_changes {
        "LEARNING_MEASURED_COMPILE_DIFFERENCES_RETAIN_ROWS"
    } else {
        "LEARNING_AND_LOADED_NATIVE_COMPARISON_MEASURED_RETAIN_ROWS"
    };
    Ok(
        json!({"schema":if continuation.is_some(){"uor-r4.geometric-value-query-credit/1"}else{"uor-r4.geometric-value-joint-learning/1"},"parent_updates":parent_updates,"cumulative_updates":parent_updates+updates,"auxiliary_credit":a.auxiliary_credit,"continuation_parent":continuation.as_ref().map(|c|&c.metadata),"complete":complete,"decision":decision,"seed":a.seed,"mode":a.mode,"requested_updates":a.steps,"completed_updates":updates,"actual_training_episodes":episodes_count,"actual_training_positions":actual_positions,"padded_training_positions":padded_positions,"batches_prepared":batches_prepared,"completed_backward_batches":completed_backward_batches,"training_time_min":if min_time==usize::MAX{0}else{min_time},"training_time_max":max_time,"full_causal_context_ceiling":128,"value_coordinates_per_head":16,"batch_size":BATCH,"teacher_preparation_seconds":preparation_seconds,"forward_backward_update_seconds":training_seconds,"elapsed_seconds":start.elapsed().as_secs_f64(),"maximum_seconds":a.max_seconds,"evaluation_reserve_seconds":a.evaluation_reserve_seconds,"initial_original":initial_original,"initial_stress":initial_stress,"original":original,"stress":stress_result,"history":history,"scope":if continuation.is_some(){"saved learned context and value parameters continued jointly with new optimizer moments; query-read versus uniform matched eligible mass; fixed parent teacher can become stale relative to changing student addresses; placement effects are joint, not sole payload causality; geometric alphabet/output unchanged; teacher only offline; authored development, no complete serving or general language claim"}else{"new geometric value head and three retained context transition families learned jointly; unchanged observation coefficients may emit changed addresses from new states; auxiliary targets from frozen donor only in training; learned prediction contains no donor values; float training/ref NoRead/trunk/read.out/head remain; authored development panels not fresh holdout; reset only latent context, span preserved; no geometry advantage, general language, energy or complete integer serving claim"}}),
    )
}
fn main() -> Result<()> {
    let a = Args::parse()?;
    report_output::claim(&a.out)?;
    let start = Instant::now();
    let result = run(&a, start);
    match &result {
        Ok(report) => write_json(&a.out.join("report.json"), report)?,
        Err(error) => write_json(
            &a.out.join("error.json"),
            &json!({"error":error.to_string(),"elapsed_seconds":start.elapsed().as_secs_f64(),"scope":"failed attempt; retained checkpoints are weights only, not exact optimizer resumes"}),
        )?,
    }
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_training::geometric_value_native::{ValuePacketRecord, ValueProjectionLane};
    #[test]
    fn learned_value_teacher_layout_masks_and_normalizes_actual_atoms() -> Result<()> {
        let (b, h, t) = (2, 2, 3);
        let mut lanes = Vec::new();
        for row in 0..b * h * t * 4 {
            let root = (row % 120) as u8;
            lanes.push(ValueProjectionLane {
                donor_valid: true,
                packet: ValuePacketRecord {
                    status: ValuePacketStatus::PresentNonzero,
                    root,
                    radius_bin: (row % 31) as u8,
                },
                residual_packet: Some(ValuePacketRecord {
                    status: ValuePacketStatus::PresentZero,
                    root: 1,
                    radius_bin: 0,
                }),
                error_q16: [0; 4],
                squared_error_q32: 0,
            });
        }
        let trace = ValueProjectionTrace {
            batch: b,
            heads: h,
            time: t,
            value_width: 16,
            donor_q16: vec![0; b * h * t * 16],
            projected_q16: vec![0; b * h * t * 16],
            lanes,
            unique_vectors: 0,
            candidate_distances: 0,
            max_abs_error_q16: 0,
            sum_squared_error_q32: 0,
        };
        let labels = Targets::from_trace(&trace, &[3, 1])?;
        for batch in 0..b {
            for time in 0..t {
                for head in 0..h {
                    for lane in 0..4 {
                        let expected = ((batch * h + head) * t + time) * 4 + lane;
                        let at = (((batch * t + time) * h + head) * 4 + lane) * 2;
                        assert_eq!(labels.roots[at], (expected % 120) as u32);
                        assert_eq!(labels.categories[at], (expected % 31 + 1) as u32);
                        assert_eq!(labels.categories[at + 1], 0);
                        let valid = batch == 0 || time == 0;
                        assert_eq!(labels.root_weights[at], if valid { 1. } else { 0. });
                        assert_eq!(labels.root_weights[at + 1], 0.);
                        assert_eq!(labels.category_weights[at + 1], if valid { 1. } else { 0. });
                    }
                }
            }
        }
        assert_eq!(labels.root_weights.iter().sum::<f32>(), 32.);
        assert_eq!(labels.category_weights.iter().sum::<f32>(), 64.);
        let loss = logits_cross_entropy(
            &Tensor::zeros(
                (labels.roots.len(), 120),
                candle_core::DType::F32,
                &Device::Cpu,
            )?,
            &labels.roots,
            Some(&labels.root_weights),
        )?
        .to_scalar::<f32>()?;
        assert!((loss - 120f32.ln()).abs() < 1e-5);
        let mut zeros = trace.clone();
        for lane in &mut zeros.lanes {
            lane.packet = ValuePacketRecord {
                status: ValuePacketStatus::PresentZero,
                root: 1,
                radius_bin: 0,
            };
            lane.residual_packet = Some(lane.packet.clone());
        }
        let zero_labels = Targets::from_trace(&zeros, &[3, 1])?;
        let output = ValueProducerOutput {
            values: Tensor::zeros((b, h, t, 16), candle_core::DType::F32, &Device::Cpu)?,
            root_logits: Tensor::zeros(
                vec![b, t, h, 4, 2, 120],
                candle_core::DType::F32,
                &Device::Cpu,
            )?,
            category_logits: Tensor::zeros(
                vec![b, t, h, 4, 2, 32],
                candle_core::DType::F32,
                &Device::Cpu,
            )?,
            trace: uor_r4_training::geometric_value_producer::ValueProducerTrace {
                batch: b,
                heads: h,
                time: t,
                occurrence_valid: vec![true; b * t],
                packets: vec![],
                values_q16: vec![],
            },
        };
        let (root, category) = zero_labels.losses(&output)?;
        assert_eq!(root.to_scalar::<f32>()?, 0.);
        assert!((category.to_scalar::<f32>()? - 32f32.ln()).abs() < 1e-5);
        zeros.lanes[0].donor_valid = false;
        zeros.lanes[1].packet.status = ValuePacketStatus::Absent;
        let excluded = Targets::from_trace(&zeros, &[3, 1])?;
        assert_eq!(&excluded.category_weights[..4], &[0., 0., 0., 1.]);
        Ok(())
    }
}
