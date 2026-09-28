//! Exact finite-group tracking lanes (roadmap T1(b), the B1 hypothesis), Stage A.
//!
//! A tracking lane is a token-conditioned side channel with no additive input
//! and no decay: `h_t = M[token_t] h_{t-1}` from `h_{-1} = 1`, where `M` is an
//! orthogonal transport on R^4 chosen by the token alone. Stage A trains lanes
//! on the A5 word problem and compares four parameterisations of `M`:
//!
//! - [`LaneKind::Quaternion`]: left multiplication by a freely parameterised
//!   unit quaternion. It can represent the binary icosahedral group 2I, whose
//!   quotient is A5, exactly.
//! - [`LaneKind::Phase`]: a unit complex number, i.e. a rotation in one fixed
//!   plane. Commutative, so it cannot track A5 beyond chance.
//! - [`LaneKind::ReflectionPair`]: two free reflections, `x -> H(v2) H(v1) x`,
//!   which is `x -> p x r` with `p = v2 conj(v1)` and `r = conj(v1) v2`. This is
//!   the strongest ordinary non-diagonal control (DeltaProduct with two factors).
//! - [`LaneKind::Frozen`]: identity transport, the no-transport control.
//!
//! A trained lane is served exactly by closing its per-token transports into a
//! finite group ([`LaneAutomaton::snap`]). The state becomes one group index,
//! each token is one read of a Cayley table and the class is one read of a
//! read-out table: no arithmetic, no float, no multiplier. Any lane that has
//! learned a faithful A5 representation snaps this way, whatever its
//! parameterisation, so Stage A compares learnability and exactness, not
//! serving cost.
//!
//! The A5 labels use the repository's exact 2I composition table
//! (`uor_r4_core::native_geometric::learner::group_table`), with the same
//! accumulation order as `compose_context_roots`: `state <- state * root`.
//! Offline training only; nothing here is a serving path.

use std::collections::VecDeque;
use std::time::Instant;

use candle_core::{DType, Device, Tensor, Var, D};
use candle_nn::{AdamW, Optimizer, ParamsAdamW};
use serde::{Deserialize, Serialize};
use uor_r4_core::native_geometric::learner::embedding::{canonical_h4_roots, PHI};
use uor_r4_core::native_geometric::learner::{group_table, GROUP_ORDER, ROW_STRIDE};

use crate::geometric_stack::{quaternion_scan, StackConfig, StackModel};
use crate::{invalid, Result};

/// Number of A5 elements: the 2I elements modulo sign.
pub const A5_ORDER: usize = 60;

/// Largest finite group a lane may snap to; indices must fit a byte.
pub const MAX_SNAP_ORDER: usize = 256;

/// Frobenius tolerance for merging transports during closure. Distinct 2I
/// elements under left multiplication are at least `2 * 2 sin(18 deg) = 1.236`
/// apart, so a tolerance of 0.5 cannot merge two different elements of an
/// exact representation.
pub const SNAP_TOLERANCE: f64 = 0.5;

/// SplitMix64: deterministic initialisation and data.
#[derive(Clone, Copy, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed ^ 0x6A09_E667_F3BC_C909)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn below(&mut self, bound: usize) -> usize {
        (self.next_u64() % bound as u64) as usize
    }

    fn uniform(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64 + 0.5) / (1u64 << 53) as f64
    }

    fn normal(&mut self) -> f64 {
        let (u, v) = (self.uniform(), self.uniform());
        (-2.0 * u.ln()).sqrt() * (2.0 * std::f64::consts::PI * v).cos()
    }
}

/// The A5 word problem over three generators of orders 2, 3 and 5 in A5
/// (4, 6 and 10 in 2I). Token `i` is generator `i`; the target at position
/// `t` is the A5 element of the running product `g_0 g_1 ... g_t`.
#[derive(Clone, Debug)]
pub struct A5Task {
    /// Canonical 2I root index of each generator token.
    pub generators: Vec<u8>,
    /// A5 class (0..60) of every 2I element.
    pub class_of: Vec<u8>,
    /// Index of the identity element.
    pub identity: u8,
}

impl A5Task {
    /// The first triple, in canonical root order, of elements with real parts
    /// 0, 1/2 and phi/2 (orders 4, 6 and 10 in 2I) that generates all of 2I.
    pub fn standard() -> Result<Self> {
        let table = group_table();
        let roots = canonical_h4_roots();
        let real: Vec<f64> = roots.iter().map(|r| r.to_array()[0]).collect();
        let near = |x: f64, target: f64| (x - target).abs() < 1e-9;
        let pick = |target: f64| -> Vec<usize> {
            (0..GROUP_ORDER)
                .filter(|&i| near(real[i], target))
                .collect()
        };
        let (order4, order6, order10) = (pick(0.0), pick(0.5), pick(PHI / 2.0));
        let identity = table.identity;
        for &a in &order4 {
            for &b in &order6 {
                for &c in &order10 {
                    let generators = vec![a as u8, b as u8, c as u8];
                    if closure_size(&generators, identity) == GROUP_ORDER {
                        let class_of = a5_classes()?;
                        return Ok(Self {
                            generators,
                            class_of,
                            identity,
                        });
                    }
                }
            }
        }
        Err(invalid("no generating triple of orders 4, 6 and 10 in 2I"))
    }

    pub fn vocab(&self) -> usize {
        self.generators.len()
    }

    /// One uniformly random word of `length` tokens and its per-position classes.
    pub fn sample(&self, rng: &mut Rng, length: usize) -> (Vec<u32>, Vec<u32>) {
        let table = group_table();
        let mut state = self.identity as usize;
        let mut tokens = Vec::with_capacity(length);
        let mut classes = Vec::with_capacity(length);
        for _ in 0..length {
            let token = rng.below(self.generators.len());
            let root = self.generators[token] as usize;
            state = table.product[state * ROW_STRIDE + root] as usize;
            tokens.push(token as u32);
            classes.push(self.class_of[state] as u32);
        }
        (tokens, classes)
    }

    /// A batch of `batch` words of equal `length`, flattened row-major.
    pub fn batch(&self, rng: &mut Rng, batch: usize, length: usize) -> (Vec<u32>, Vec<u32>) {
        let mut tokens = Vec::with_capacity(batch * length);
        let mut classes = Vec::with_capacity(batch * length);
        for _ in 0..batch {
            let (t, c) = self.sample(rng, length);
            tokens.extend(t);
            classes.extend(c);
        }
        (tokens, classes)
    }
}

/// Size of the subgroup of 2I generated by `generators`.
fn closure_size(generators: &[u8], identity: u8) -> usize {
    let table = group_table();
    let mut seen = [false; GROUP_ORDER];
    let mut queue = VecDeque::from([identity as usize]);
    seen[identity as usize] = true;
    let mut count = 1;
    while let Some(state) = queue.pop_front() {
        for &g in generators {
            let next = table.product[state * ROW_STRIDE + g as usize] as usize;
            if !seen[next] {
                seen[next] = true;
                count += 1;
                queue.push_back(next);
            }
        }
    }
    count
}

/// A5 class of every 2I element: `s` and `-s` share a class, numbered in
/// order of the first member's canonical index.
fn a5_classes() -> Result<Vec<u8>> {
    let table = group_table();
    let roots = canonical_h4_roots();
    let minus_one = roots
        .iter()
        .position(|r| (r.to_array()[0] + 1.0).abs() < 1e-9)
        .ok_or_else(|| invalid("2I lacks -1"))?;
    let mut class_of = vec![u8::MAX; GROUP_ORDER];
    let mut next = 0u8;
    for s in 0..GROUP_ORDER {
        if class_of[s] != u8::MAX {
            continue;
        }
        let negative = table.product[minus_one * ROW_STRIDE + s] as usize;
        class_of[s] = next;
        class_of[negative] = next;
        next += 1;
    }
    if next as usize != A5_ORDER {
        return Err(invalid(format!("2I / {{+-1}} has {next} classes, not 60")));
    }
    Ok(class_of)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaneKind {
    Quaternion,
    Phase,
    ReflectionPair,
    Frozen,
}

impl LaneKind {
    pub fn parse(name: &str) -> Result<Self> {
        match name {
            "quaternion" => Ok(Self::Quaternion),
            "phase" => Ok(Self::Phase),
            "reflection_pair" => Ok(Self::ReflectionPair),
            "frozen" => Ok(Self::Frozen),
            other => Err(invalid(format!(
                "lane kind is quaternion, phase, reflection_pair or frozen, got {other}"
            ))),
        }
    }

    /// Transport parameters per token and lane.
    fn width(self) -> usize {
        match self {
            Self::Quaternion => 4,
            Self::Phase => 1,
            Self::ReflectionPair => 8,
            Self::Frozen => 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LaneConfig {
    pub kind: LaneKind,
    pub vocab: usize,
    pub lanes: usize,
    pub hidden: usize,
    pub classes: usize,
    pub seed: u64,
}

/// Token-conditioned tracking lanes: one transport per token and lane.
pub struct LaneTransport {
    pub kind: LaneKind,
    pub vocab: usize,
    pub lanes: usize,
    var: Option<Var>,
    device: Device,
}

/// Tracking lanes with an MLP read-out, trained in f32.
pub struct LaneModel {
    pub config: LaneConfig,
    transport: LaneTransport,
    hidden_weight: Var,
    hidden_bias: Var,
    out_weight: Var,
    out_bias: Var,
    device: Device,
}

fn normal_var(rng: &mut Rng, shape: &[usize], std: f64, device: &Device) -> Result<Var> {
    let count: usize = shape.iter().product();
    let values: Vec<f32> = (0..count).map(|_| (rng.normal() * std) as f32).collect();
    Ok(Var::from_tensor(&Tensor::from_vec(values, shape, device)?)?)
}

/// Hamilton product of `[.., 4]` tensors, differentiable.
pub fn hamilton(a: &Tensor, b: &Tensor) -> Result<Tensor> {
    let part = |t: &Tensor, i: usize| t.narrow(D::Minus1, i, 1);
    let (a0, a1, a2, a3) = (part(a, 0)?, part(a, 1)?, part(a, 2)?, part(a, 3)?);
    let (b0, b1, b2, b3) = (part(b, 0)?, part(b, 1)?, part(b, 2)?, part(b, 3)?);
    let w = (((&a0 * &b0)? - (&a1 * &b1)?)? - (&a2 * &b2)?)? - (&a3 * &b3)?;
    let x = (((&a0 * &b1)? + (&a1 * &b0)?)? + (&a2 * &b3)?)? - (&a3 * &b2)?;
    let y = (((&a0 * &b2)? - (&a1 * &b3)?)? + (&a2 * &b0)?)? + (&a3 * &b1)?;
    let z = (((&a0 * &b3)? + (&a1 * &b2)?)? - (&a2 * &b1)?)? + (&a3 * &b0)?;
    Ok(Tensor::cat(&[w?, x?, y?, z?], D::Minus1)?)
}

fn conjugate(q: &Tensor) -> Result<Tensor> {
    let real = q.narrow(D::Minus1, 0, 1)?;
    let imaginary = q.narrow(D::Minus1, 1, 3)?.neg()?;
    Ok(Tensor::cat(&[real, imaginary], D::Minus1)?)
}

fn unit(v: &Tensor) -> Result<Tensor> {
    let norm = (v.sqr()?.sum_keepdim(D::Minus1)? + 1e-12)?.sqrt()?;
    Ok(v.broadcast_div(&norm)?)
}

/// Cumulative left products `q_t ... q_0` of `[batch, time, lanes, 4]`.
fn cumulative(q: &Tensor) -> Result<Tensor> {
    let (batch, time, lanes, _) = q.dims4()?;
    let first = q.narrow(1, 0, 1)?;
    let drive = if time > 1 {
        let rest = Tensor::zeros((batch, time - 1, lanes, 4), DType::F32, q.device())?;
        Tensor::cat(&[first, rest], 1)?
    } else {
        first
    };
    quaternion_scan(q, &drive)
}

impl LaneTransport {
    /// Free parameterisation: raw quaternions and reflection vectors are
    /// standard normal, so their normalisations are uniform on S3, not near the
    /// identity. Phases are wide angles.
    pub fn new(
        kind: LaneKind,
        vocab: usize,
        lanes: usize,
        rng: &mut Rng,
        device: &Device,
    ) -> Result<Self> {
        if vocab == 0 || lanes == 0 {
            return Err(invalid("tracking lanes need a vocabulary and lanes"));
        }
        let width = kind.width();
        let var = if width == 0 {
            None
        } else {
            let std = if kind == LaneKind::Phase { 3.0 } else { 1.0 };
            Some(normal_var(rng, &[vocab, lanes * width], std, device)?)
        };
        Ok(Self {
            kind,
            vocab,
            lanes,
            var,
            device: device.clone(),
        })
    }

    pub fn var(&self) -> Option<&Var> {
        self.var.as_ref()
    }

    /// Per-token transport parameters `[batch, time, lanes, width]`.
    fn gather(&self, tokens: &Tensor, batch: usize, time: usize) -> Result<Tensor> {
        let transport = self
            .var
            .as_ref()
            .ok_or_else(|| invalid("frozen lanes have no transport"))?;
        let rows = transport
            .as_tensor()
            .index_select(&tokens.flatten_all()?, 0)?;
        Ok(rows.reshape((batch, time, self.lanes, self.kind.width()))?)
    }

    /// Lane states `[batch, time, lanes, 4]` after each token.
    pub fn states(&self, tokens: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        if tokens.len() != batch * time {
            return Err(invalid("token count does not match batch * time"));
        }
        if tokens.iter().any(|&t| t as usize >= self.vocab) {
            return Err(invalid("token outside the lane vocabulary"));
        }
        let lanes = self.lanes;
        let ids = Tensor::from_vec(tokens.to_vec(), (batch, time), &self.device)?;
        match self.kind {
            LaneKind::Quaternion => cumulative(&unit(&self.gather(&ids, batch, time)?)?),
            LaneKind::Phase => {
                let theta = self.gather(&ids, batch, time)?;
                let zero = theta.zeros_like()?;
                let q = Tensor::cat(&[theta.cos()?, theta.sin()?, zero.clone(), zero], D::Minus1)?;
                cumulative(&q)
            }
            LaneKind::ReflectionPair => {
                let raw = self.gather(&ids, batch, time)?;
                let v1 = unit(&raw.narrow(D::Minus1, 0, 4)?)?;
                let v2 = unit(&raw.narrow(D::Minus1, 4, 4)?)?;
                let v1_bar = conjugate(&v1)?;
                let p = hamilton(&v2, &v1_bar)?;
                let r = hamilton(&v1_bar, &v2)?;
                // x_t = p_t x_{t-1} r_t from x_{-1} = 1 is P_t conj(Q_t), with
                // P_t = p_t...p_0 and Q_t = conj(r_t)...conj(r_0).
                let left = cumulative(&p)?;
                let right = cumulative(&conjugate(&r)?)?;
                hamilton(&left, &conjugate(&right)?)
            }
            LaneKind::Frozen => {
                let one = Tensor::ones((batch, time, lanes, 1), DType::F32, &self.device)?;
                let zero = Tensor::zeros((batch, time, lanes, 3), DType::F32, &self.device)?;
                Ok(Tensor::cat(&[one, zero], D::Minus1)?)
            }
        }
    }

    /// The 4x4 real matrix of lane `lane`'s transport for `token`, row-major.
    pub fn matrix(&self, token: usize, lane: usize) -> Result<[f64; 16]> {
        if token >= self.vocab || lane >= self.lanes {
            return Err(invalid("token or lane out of range"));
        }
        let width = self.kind.width();
        let row: Vec<f64> = match &self.var {
            None => return Ok(identity_matrix()),
            Some(transport) => transport
                .as_tensor()
                .get(token)?
                .narrow(0, lane * width, width)?
                .to_dtype(DType::F64)?
                .to_vec1::<f64>()?,
        };
        Ok(match self.kind {
            LaneKind::Quaternion => left_matrix(normalized(&row[0..4])),
            LaneKind::Phase => left_matrix([row[0].cos(), row[0].sin(), 0.0, 0.0]),
            LaneKind::ReflectionPair => {
                let v1 = normalized(&row[0..4]);
                let v2 = normalized(&row[4..8]);
                let v1_bar = [v1[0], -v1[1], -v1[2], -v1[3]];
                let p = quaternion_product(v2, v1_bar);
                let r = quaternion_product(v1_bar, v2);
                matrix_product(&left_matrix(p), &right_matrix(r))
            }
            LaneKind::Frozen => identity_matrix(),
        })
    }
}

impl LaneModel {
    pub fn new(config: LaneConfig, device: &Device) -> Result<Self> {
        if config.vocab == 0 || config.lanes == 0 || config.hidden == 0 || config.classes == 0 {
            return Err(invalid(
                "lane model needs a vocabulary, lanes, hidden units and classes",
            ));
        }
        let mut rng = Rng::new(config.seed);
        let transport =
            LaneTransport::new(config.kind, config.vocab, config.lanes, &mut rng, device)?;
        let features = config.lanes * 4;
        let hidden_weight = normal_var(
            &mut rng,
            &[config.hidden, features],
            (2.0 / features as f64).sqrt(),
            device,
        )?;
        let hidden_bias = Var::from_tensor(&Tensor::zeros(config.hidden, DType::F32, device)?)?;
        let out_weight = normal_var(
            &mut rng,
            &[config.classes, config.hidden],
            (1.0 / config.hidden as f64).sqrt(),
            device,
        )?;
        let out_bias = Var::from_tensor(&Tensor::zeros(config.classes, DType::F32, device)?)?;
        Ok(Self {
            config,
            transport,
            hidden_weight,
            hidden_bias,
            out_weight,
            out_bias,
            device: device.clone(),
        })
    }

    pub fn vars(&self) -> Vec<Var> {
        let mut vars = vec![
            self.hidden_weight.clone(),
            self.hidden_bias.clone(),
            self.out_weight.clone(),
            self.out_bias.clone(),
        ];
        if let Some(transport) = self.transport.var() {
            vars.push(transport.clone());
        }
        vars
    }

    /// Lane states `[batch, time, lanes, 4]` after each token.
    pub fn states(&self, tokens: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        self.transport.states(tokens, batch, time)
    }

    /// Class logits `[batch * time, classes]`.
    pub fn logits(&self, tokens: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        let features = self
            .states(tokens, batch, time)?
            .reshape((batch * time, self.config.lanes * 4))?;
        let hidden = features
            .matmul(&self.hidden_weight.as_tensor().t()?)?
            .broadcast_add(self.hidden_bias.as_tensor())?
            .relu()?;
        Ok(hidden
            .matmul(&self.out_weight.as_tensor().t()?)?
            .broadcast_add(self.out_bias.as_tensor())?)
    }

    /// The 4x4 real matrix of lane `lane`'s transport for `token`, row-major.
    pub fn transport_matrix(&self, token: usize, lane: usize) -> Result<[f64; 16]> {
        self.transport.matrix(token, lane)
    }
}

fn normalized(v: &[f64]) -> [f64; 4] {
    let n = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    if n == 0.0 {
        [1.0, 0.0, 0.0, 0.0]
    } else {
        [v[0] / n, v[1] / n, v[2] / n, v[3] / n]
    }
}

fn quaternion_product(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    [
        a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
        a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
        a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
        a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
    ]
}

fn identity_matrix() -> [f64; 16] {
    let mut m = [0.0; 16];
    for i in 0..4 {
        m[i * 5] = 1.0;
    }
    m
}

/// Matrix of `x -> q x`.
fn left_matrix(q: [f64; 4]) -> [f64; 16] {
    [
        q[0], -q[1], -q[2], -q[3], //
        q[1], q[0], -q[3], q[2], //
        q[2], q[3], q[0], -q[1], //
        q[3], -q[2], q[1], q[0],
    ]
}

/// Matrix of `x -> x r`.
fn right_matrix(r: [f64; 4]) -> [f64; 16] {
    [
        r[0], -r[1], -r[2], -r[3], //
        r[1], r[0], r[3], -r[2], //
        r[2], -r[3], r[0], r[1], //
        r[3], r[2], -r[1], r[0],
    ]
}

fn matrix_product(a: &[f64; 16], b: &[f64; 16]) -> [f64; 16] {
    let mut m = [0.0; 16];
    for i in 0..4 {
        for j in 0..4 {
            m[i * 4 + j] = (0..4).map(|k| a[i * 4 + k] * b[k * 4 + j]).sum();
        }
    }
    m
}

fn frobenius(a: &[f64; 16], b: &[f64; 16]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y) * (x - y))
        .sum::<f64>()
        .sqrt()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrainConfig {
    pub steps: usize,
    pub batch: usize,
    pub learning_rate: f64,
    /// Final training length; the curriculum ramps from 4 to it over the first
    /// 60% of steps.
    pub train_length: usize,
    pub data_seed: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrainRecord {
    pub step: usize,
    pub length: usize,
    pub loss: f64,
    pub accuracy: f64,
}

fn curriculum_length(step: usize, config: &TrainConfig) -> usize {
    let ramp = (config.steps as f64 * 0.6).max(1.0);
    let fraction = (step as f64 / ramp).min(1.0);
    let length = 4.0 + fraction * (config.train_length.saturating_sub(4)) as f64;
    (length.round() as usize).clamp(2, config.train_length.max(2))
}

/// Cross-entropy training at every position with a length curriculum and a
/// cosine learning-rate decay to 10%.
pub fn train(
    model: &LaneModel,
    task: &A5Task,
    config: &TrainConfig,
) -> Result<(Vec<TrainRecord>, f64)> {
    let started = Instant::now();
    let mut optimizer = AdamW::new(
        model.vars(),
        ParamsAdamW {
            lr: config.learning_rate,
            beta1: 0.9,
            beta2: 0.999,
            eps: 1e-8,
            weight_decay: 0.0,
        },
    )?;
    let mut rng = Rng::new(config.data_seed);
    let mut records = Vec::new();
    for step in 0..config.steps {
        let progress = step as f64 / config.steps.max(1) as f64;
        let scale = 0.1 + 0.9 * 0.5 * (1.0 + (std::f64::consts::PI * progress).cos());
        optimizer.set_learning_rate(config.learning_rate * scale);
        let length = curriculum_length(step, config);
        let (tokens, classes) = task.batch(&mut rng, config.batch, length);
        let logits = model.logits(&tokens, config.batch, length)?;
        let targets = Tensor::from_vec(classes.clone(), classes.len(), &model.device)?;
        let loss = candle_nn::loss::cross_entropy(&logits, &targets)?;
        optimizer.backward_step(&loss)?;
        if step % 100 == 0 || step + 1 == config.steps {
            let predicted = logits.argmax(D::Minus1)?.to_vec1::<u32>()?;
            let correct = predicted
                .iter()
                .zip(&classes)
                .filter(|(p, c)| p == c)
                .count();
            records.push(TrainRecord {
                step,
                length,
                loss: loss.to_scalar::<f32>()? as f64,
                accuracy: correct as f64 / classes.len() as f64,
            });
        }
    }
    Ok((records, started.elapsed().as_secs_f64()))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LengthAccuracy {
    pub length: usize,
    pub words: usize,
    /// Accuracy at the last position of each word.
    pub final_position: f64,
    /// Accuracy over every position of every word.
    pub all_positions: f64,
}

/// Float-model accuracy on fresh words of each length.
pub fn evaluate(
    model: &LaneModel,
    task: &A5Task,
    lengths: &[usize],
    words: usize,
    seed: u64,
) -> Result<Vec<LengthAccuracy>> {
    let mut rng = Rng::new(seed);
    let mut out = Vec::with_capacity(lengths.len());
    for &length in lengths {
        let (mut last, mut all, mut total) = (0usize, 0usize, 0usize);
        // Bound each forward pass to 2^18 positions, about 0.5 GB of
        // activations at the default widths.
        let chunk = (262_144 / length.max(1)).clamp(1, words.max(1));
        let mut remaining = words;
        while remaining > 0 {
            let batch = chunk.min(remaining);
            let (tokens, classes) = task.batch(&mut rng, batch, length);
            let predicted = model
                .logits(&tokens, batch, length)?
                .argmax(D::Minus1)?
                .to_vec1::<u32>()?;
            for word in 0..batch {
                let row = word * length;
                for t in 0..length {
                    if predicted[row + t] == classes[row + t] {
                        all += 1;
                        if t + 1 == length {
                            last += 1;
                        }
                    }
                }
            }
            total += batch * length;
            remaining -= batch;
        }
        out.push(LengthAccuracy {
            length,
            words,
            final_position: last as f64 / words.max(1) as f64,
            all_positions: all as f64 / total.max(1) as f64,
        });
    }
    Ok(out)
}

/// A lane closed into a finite group and served as a table automaton.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LaneAutomaton {
    pub lane: usize,
    /// Number of group elements.
    pub order: usize,
    /// `table[token * order + element]`: the element after reading `token`.
    pub table: Vec<u8>,
    /// Class read out from each element.
    pub readout: Vec<u8>,
    pub identity: u8,
    /// Largest Frobenius distance between a computed product and the element it
    /// was merged into during closure.
    pub max_merge_distance: f64,
    /// Largest deviation of any element's `trace / 4` from the nearest 2I real
    /// part {0, +-1/2, +-1/(2 phi), +-phi/2, +-1}. Meaningful for quaternion lanes,
    /// whose trace / 4 is the real part of the unit quaternion.
    pub max_trace_deviation: f64,
    /// Read-out agreement on the fitting words.
    pub fit_accuracy: f64,
    /// States of the equivalent minimal automaton (Moore partition refinement
    /// over the read-out): the serving table a deployment would keep. A 2I lane
    /// read out by A5 class minimises to 60 states, since `s` and `-s` agree.
    pub minimal_order: usize,
}

/// Moore partition refinement: the number of states of the minimal automaton
/// equivalent to `table` with outputs `readout`. Every state is reachable, as
/// the closure is built from the identity.
fn minimal_order(table: &[u8], readout: &[u8], vocab: usize, order: usize) -> usize {
    let mut block: Vec<usize> = readout.iter().map(|&c| usize::from(c)).collect();
    let mut blocks = {
        let mut seen = block.clone();
        seen.sort_unstable();
        seen.dedup();
        seen.len()
    };
    loop {
        let mut ids: std::collections::BTreeMap<Vec<usize>, usize> =
            std::collections::BTreeMap::new();
        let mut next = vec![0usize; order];
        for state in 0..order {
            let mut signature = Vec::with_capacity(vocab + 1);
            signature.push(block[state]);
            for token in 0..vocab {
                signature.push(block[usize::from(table[token * order + state])]);
            }
            let fresh = ids.len();
            next[state] = *ids.entry(signature).or_insert(fresh);
        }
        let refined = ids.len();
        block = next;
        if refined == blocks {
            return refined;
        }
        blocks = refined;
    }
}

/// Nearest 2I real part.
fn two_i_trace_deviation(real: f64) -> f64 {
    let values = [
        0.0,
        0.5,
        -0.5,
        PHI / 2.0,
        -PHI / 2.0,
        1.0 / (2.0 * PHI),
        -1.0 / (2.0 * PHI),
        1.0,
        -1.0,
    ];
    values
        .iter()
        .map(|v| (real - v).abs())
        .fold(f64::INFINITY, f64::min)
}

impl LaneAutomaton {
    /// Close lane `lane`'s per-token transports into a group by breadth-first
    /// search from the identity, merging products within [`SNAP_TOLERANCE`], then
    /// fit a read-out table by majority class over `fit_words` words.
    ///
    /// Returns `None` when the closure exceeds [`MAX_SNAP_ORDER`] elements (the
    /// lane has not learned a finite representation) or is inconsistent.
    pub fn snap(
        model: &LaneModel,
        task: &A5Task,
        lane: usize,
        fit_words: usize,
        fit_length: usize,
        seed: u64,
    ) -> Result<Option<Self>> {
        let generators: Vec<[f64; 16]> = (0..model.config.vocab)
            .map(|token| model.transport_matrix(token, lane))
            .collect::<Result<_>>()?;
        Self::from_generators(&generators, lane, task, fit_words, fit_length, seed)
    }

    /// [`snap`](Self::snap) from the transport matrix of each task token.
    pub fn from_generators(
        generators: &[[f64; 16]],
        lane: usize,
        task: &A5Task,
        fit_words: usize,
        fit_length: usize,
        seed: u64,
    ) -> Result<Option<Self>> {
        if generators.len() != task.vocab() {
            return Err(invalid("one generator matrix per task token"));
        }
        let vocab = generators.len();
        let mut elements = vec![identity_matrix()];
        let mut edges: Vec<(usize, usize, usize)> = Vec::new();
        let mut max_merge_distance = 0.0f64;
        let mut cursor = 0;
        while cursor < elements.len() {
            let current = elements[cursor];
            for (token, g) in generators.iter().enumerate() {
                let product = matrix_product(g, &current);
                let mut found = None;
                for (index, element) in elements.iter().enumerate() {
                    let distance = frobenius(&product, element);
                    if distance < SNAP_TOLERANCE {
                        if found.is_some() {
                            return Ok(None);
                        }
                        found = Some(index);
                        max_merge_distance = max_merge_distance.max(distance);
                    }
                }
                let target = match found {
                    Some(index) => index,
                    None => {
                        if elements.len() == MAX_SNAP_ORDER {
                            return Ok(None);
                        }
                        elements.push(product);
                        elements.len() - 1
                    }
                };
                edges.push((token, cursor, target));
            }
            cursor += 1;
        }
        let order = elements.len();
        let mut table = vec![0u8; vocab * order];
        for (token, from, to) in edges {
            table[token * order + from] = to as u8;
        }
        let max_trace_deviation = elements
            .iter()
            .map(|m| two_i_trace_deviation((m[0] + m[5] + m[10] + m[15]) / 4.0))
            .fold(0.0f64, f64::max);

        // Majority read-out over fitting words.
        let classes = A5_ORDER;
        let mut counts = vec![0u32; order * classes];
        let mut rng = Rng::new(seed);
        let mut fit_positions = Vec::with_capacity(fit_words * fit_length);
        for _ in 0..fit_words {
            let (tokens, labels) = task.sample(&mut rng, fit_length);
            let mut state = 0usize;
            for (token, label) in tokens.iter().zip(&labels) {
                state = table[*token as usize * order + state] as usize;
                counts[state * classes + *label as usize] += 1;
                fit_positions.push((state, *label));
            }
        }
        let readout: Vec<u8> = (0..order)
            .map(|state| {
                let row = &counts[state * classes..(state + 1) * classes];
                let mut best = 0usize;
                for (class, &count) in row.iter().enumerate() {
                    if count > row[best] {
                        best = class;
                    }
                }
                best as u8
            })
            .collect();
        let agree = fit_positions
            .iter()
            .filter(|(state, label)| readout[*state] as u32 == *label)
            .count();
        let minimal_order = minimal_order(&table, &readout, vocab, order);
        Ok(Some(Self {
            lane,
            order,
            table,
            readout,
            identity: 0,
            max_merge_distance,
            max_trace_deviation,
            fit_accuracy: agree as f64 / fit_positions.len().max(1) as f64,
            minimal_order,
        }))
    }

    /// Serve one word: one table read per token for the state, one for the class.
    pub fn run(&self, tokens: &[u32]) -> Vec<u8> {
        let mut state = self.identity as usize;
        tokens
            .iter()
            .map(|&token| {
                state = self.table[token as usize * self.order + state] as usize;
                self.readout[state]
            })
            .collect()
    }

    /// Exact automaton accuracy on fresh words of each length.
    pub fn evaluate(
        &self,
        task: &A5Task,
        lengths: &[usize],
        words: usize,
        seed: u64,
    ) -> Vec<LengthAccuracy> {
        let mut rng = Rng::new(seed);
        lengths
            .iter()
            .map(|&length| {
                let (mut last, mut all) = (0usize, 0usize);
                for _ in 0..words {
                    let (tokens, labels) = task.sample(&mut rng, length);
                    let predicted = self.run(&tokens);
                    for (t, (p, l)) in predicted.iter().zip(&labels).enumerate() {
                        if *p as u32 == *l {
                            all += 1;
                            if t + 1 == length {
                                last += 1;
                            }
                        }
                    }
                }
                LengthAccuracy {
                    length,
                    words,
                    final_position: last as f64 / words.max(1) as f64,
                    all_positions: all as f64 / (words * length).max(1) as f64,
                }
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Stage B: tracking lanes inside the geometric stack.

/// Text token ids lie below this; the A5 generator tokens follow it.
pub const TEXT_VOCAB: usize = 4096;

/// A geometric stack with an optional tracking-lane side channel added to its
/// embeddings, and an auxiliary A5 read-out over its final states. The lanes
/// see every token, text and A5 alike, from the same free initialisation.
pub struct TrackedStack {
    pub stack: StackModel,
    pub lanes: Option<LaneTransport>,
    projection: Option<Var>,
    aux_weight: Var,
    aux_bias: Var,
}

impl TrackedStack {
    /// `config.vocab_size` must be `TEXT_VOCAB + task.vocab()`.
    pub fn new(
        config: StackConfig,
        lanes: Option<(LaneKind, usize)>,
        task: &A5Task,
        seed: u64,
        device: &Device,
    ) -> Result<Self> {
        if config.vocab_size != TEXT_VOCAB + task.vocab() {
            return Err(invalid(
                "a tracked stack's vocabulary is the text vocabulary plus the A5 tokens",
            ));
        }
        let (width, vocab) = (config.width, config.vocab_size);
        let stack = StackModel::new(config, device)?;
        let mut rng = Rng::new(seed ^ 0x7472_6163_6B65_6421);
        let (lanes, projection) = match lanes {
            None => (None, None),
            Some((kind, count)) => {
                let transport = LaneTransport::new(kind, vocab, count, &mut rng, device)?;
                // Zero-initialised, like any new residual branch: the side
                // channel starts silent and text training is undisturbed
                // until the projection learns to use it.
                let projection =
                    Var::from_tensor(&Tensor::zeros((width, count * 4), DType::F32, device)?)?;
                (Some(transport), Some(projection))
            }
        };
        let aux_weight = normal_var(&mut rng, &[A5_ORDER, width], 0.02, device)?;
        let aux_bias = Var::from_tensor(&Tensor::zeros(A5_ORDER, DType::F32, device)?)?;
        Ok(Self {
            stack,
            lanes,
            projection,
            aux_weight,
            aux_bias,
        })
    }

    /// Stack parameters, and the lane transport, projection and A5 read-out.
    pub fn parameter_counts(&self) -> (usize, usize) {
        let side = self
            .lanes
            .as_ref()
            .and_then(LaneTransport::var)
            .map_or(0, |v| v.elem_count())
            + self.projection.as_ref().map_or(0, |v| v.elem_count())
            + self.aux_weight.elem_count()
            + self.aux_bias.elem_count();
        (self.stack.parameter_count(), side)
    }

    /// Decayed matrices and other variables, grouped as the stack groups its
    /// own, and the lane transports, which train at their own rate.
    pub fn optimizer_groups(&self) -> (Vec<Var>, Vec<Var>, Vec<Var>) {
        let (mut decayed, mut plain) = self.stack.optimizer_groups();
        decayed.push(self.aux_weight.clone());
        plain.push(self.aux_bias.clone());
        if let Some(projection) = &self.projection {
            decayed.push(projection.clone());
        }
        let lanes = self
            .lanes
            .as_ref()
            .and_then(LaneTransport::var)
            .map(|var| vec![var.clone()])
            .unwrap_or_default();
        (decayed, plain, lanes)
    }

    fn trunk(&self, ids: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        let mut x = self.stack.embed(ids, batch, time)?;
        if let (Some(lanes), Some(projection)) = (&self.lanes, &self.projection) {
            let features = lanes
                .states(ids, batch, time)?
                .reshape((batch * time, lanes.lanes * 4))?;
            let side = features.matmul(&projection.as_tensor().t()?)?.reshape((
                batch,
                time,
                self.stack.config.width,
            ))?;
            x = x.add(&side)?;
        }
        self.stack.hidden_from_input(x)
    }

    /// Next-token logits `[batch * time, vocabulary]`.
    pub fn text_logits(&self, ids: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        self.stack.head(&self.trunk(ids, batch, time)?)
    }

    /// Auxiliary A5 class logits `[batch * time, 60]`.
    pub fn a5_logits(&self, ids: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        Ok(self
            .trunk(ids, batch, time)?
            .matmul(&self.aux_weight.as_tensor().t()?)?
            .broadcast_add(self.aux_bias.as_tensor())?)
    }

    /// Transport matrices of lane `lane` for the A5 tokens, in task order.
    pub fn lane_generators(&self, task: &A5Task, lane: usize) -> Result<Vec<[f64; 16]>> {
        let lanes = self
            .lanes
            .as_ref()
            .ok_or_else(|| invalid("this stack has no tracking lanes"))?;
        (0..task.vocab())
            .map(|token| lanes.matrix(TEXT_VOCAB + token, lane))
            .collect()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MixedConfig {
    pub steps: usize,
    pub batch: usize,
    pub context: usize,
    pub learning_rate: f64,
    pub warmup: usize,
    pub weight_decay: f64,
    /// Weight of the A5 loss beside the text loss.
    pub a5_weight: f64,
    /// Shortest A5 word; the curriculum ramps to `context` over the first 60%
    /// of steps.
    pub a5_start_length: usize,
    /// Peak learning rate of the lane transports (Stage A learned A5 reliably
    /// at 0.01-0.03); it follows the same schedule.
    pub lane_learning_rate: f64,
    pub data_seed: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MixedRecord {
    pub step: usize,
    pub text_loss: f64,
    pub a5_loss: f64,
    pub a5_length: usize,
    pub a5_accuracy: f64,
    pub seconds: f64,
}

fn a5_curriculum(step: usize, config: &MixedConfig) -> usize {
    let ramp = (config.steps as f64 * 0.6).max(1.0);
    let fraction = (step as f64 / ramp).min(1.0);
    let start = config.a5_start_length.clamp(1, config.context);
    let length = start as f64 + fraction * (config.context - start) as f64;
    (length.round() as usize).clamp(start, config.context)
}

/// Linear warm-up, then cosine decay to 10%.
fn mixed_schedule(step: usize, config: &MixedConfig) -> f64 {
    if step < config.warmup {
        return (step + 1) as f64 / config.warmup as f64;
    }
    let span = config.steps.saturating_sub(config.warmup).max(1);
    let progress = (step - config.warmup) as f64 / span as f64;
    0.1 + 0.9 * 0.5 * (1.0 + (std::f64::consts::PI * progress.min(1.0)).cos())
}

/// `batch` random windows of `context` inputs and their next-token targets.
fn text_batch(
    text: &[u16],
    rng: &mut Rng,
    batch: usize,
    context: usize,
) -> Result<(Vec<u32>, Vec<u32>)> {
    if text.len() <= context + 1 {
        return Err(invalid("text is shorter than one window"));
    }
    let mut ids = Vec::with_capacity(batch * context);
    let mut targets = Vec::with_capacity(batch * context);
    for _ in 0..batch {
        let start = rng.below(text.len() - context - 1);
        ids.extend(text[start..start + context].iter().map(|&t| u32::from(t)));
        targets.extend(
            text[start + 1..start + context + 1]
                .iter()
                .map(|&t| u32::from(t)),
        );
    }
    Ok((ids, targets))
}

/// One text window and one A5 window batch per update; the loss is the text
/// cross-entropy plus `a5_weight` times the A5 cross-entropy at every position.
pub fn train_mixed(
    model: &TrackedStack,
    task: &A5Task,
    text: &[u16],
    config: &MixedConfig,
) -> Result<(Vec<MixedRecord>, f64)> {
    if text.iter().any(|&t| t as usize >= TEXT_VOCAB) {
        return Err(invalid("text token outside the text vocabulary"));
    }
    let started = Instant::now();
    let (decayed, plain, lanes) = model.optimizer_groups();
    let params = |weight_decay| ParamsAdamW {
        lr: config.learning_rate,
        beta1: 0.9,
        beta2: 0.95,
        eps: 1e-8,
        weight_decay,
    };
    let mut decayed_optimizer = AdamW::new(decayed, params(config.weight_decay))?;
    let mut plain_optimizer = AdamW::new(plain, params(0.0))?;
    let mut lane_optimizer = AdamW::new(
        lanes,
        ParamsAdamW {
            lr: config.lane_learning_rate,
            ..params(0.0)
        },
    )?;
    let device = model.stack.device().clone();
    let mut rng = Rng::new(config.data_seed);
    let mut records = Vec::new();
    for step in 0..config.steps {
        let lr = config.learning_rate * mixed_schedule(step, config);
        decayed_optimizer.set_learning_rate(lr);
        plain_optimizer.set_learning_rate(lr);
        lane_optimizer.set_learning_rate(config.lane_learning_rate * mixed_schedule(step, config));
        let (ids, targets) = text_batch(text, &mut rng, config.batch, config.context)?;
        let text_logits = model.text_logits(&ids, config.batch, config.context)?;
        let text_targets = Tensor::from_vec(targets, ids.len(), &device)?;
        let text_loss = candle_nn::loss::cross_entropy(&text_logits, &text_targets)?;
        let length = a5_curriculum(step, config);
        let (words, classes) = task.batch(&mut rng, config.batch, length);
        let a5_ids: Vec<u32> = words.iter().map(|&t| TEXT_VOCAB as u32 + t).collect();
        let a5_logits = model.a5_logits(&a5_ids, config.batch, length)?;
        let a5_targets = Tensor::from_vec(classes.clone(), classes.len(), &device)?;
        let a5_loss = candle_nn::loss::cross_entropy(&a5_logits, &a5_targets)?;
        let loss = (&text_loss + (&a5_loss * config.a5_weight)?)?;
        let grads = loss.backward()?;
        decayed_optimizer.step(&grads)?;
        plain_optimizer.step(&grads)?;
        lane_optimizer.step(&grads)?;
        if step % 50 == 0 || step + 1 == config.steps {
            let predicted = a5_logits.argmax(D::Minus1)?.to_vec1::<u32>()?;
            let correct = predicted
                .iter()
                .zip(&classes)
                .filter(|(p, c)| p == c)
                .count();
            records.push(MixedRecord {
                step,
                text_loss: text_loss.to_scalar::<f32>()? as f64,
                a5_loss: a5_loss.to_scalar::<f32>()? as f64,
                a5_length: length,
                a5_accuracy: correct as f64 / classes.len() as f64,
                seconds: started.elapsed().as_secs_f64(),
            });
        }
    }
    Ok((records, started.elapsed().as_secs_f64()))
}

/// Mean next-token NLL (nats per token) over `windows` evenly spaced windows
/// of `dev`.
pub fn text_nll(
    model: &TrackedStack,
    dev: &[u16],
    context: usize,
    windows: usize,
    batch: usize,
) -> Result<f64> {
    let span = context + 1;
    if dev.len() < span || windows == 0 || batch == 0 {
        return Err(invalid("development text needs at least one window"));
    }
    let stride = ((dev.len() - span) / windows).max(1);
    let starts: Vec<usize> = (0..windows)
        .map(|w| w * stride)
        .filter(|s| s + span <= dev.len())
        .collect();
    let device = model.stack.device().clone();
    let (mut total, mut count) = (0.0f64, 0usize);
    for chunk in starts.chunks(batch) {
        let mut ids = Vec::with_capacity(chunk.len() * context);
        let mut targets = Vec::with_capacity(chunk.len() * context);
        for &start in chunk {
            ids.extend(dev[start..start + context].iter().map(|&t| u32::from(t)));
            targets.extend(dev[start + 1..start + span].iter().map(|&t| u32::from(t)));
        }
        let logits = model.text_logits(&ids, chunk.len(), context)?.detach();
        let targets = Tensor::from_vec(targets, ids.len(), &device)?;
        let loss = candle_nn::loss::cross_entropy(&logits, &targets)?;
        total += loss.to_scalar::<f32>()? as f64 * ids.len() as f64;
        count += ids.len();
    }
    Ok(total / count.max(1) as f64)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrefixAccuracy {
    pub length: usize,
    pub words: usize,
    /// Accuracy at 1-indexed positions 1, 2, 4, ... up to `length`.
    pub by_position: Vec<(usize, f64)>,
    pub all_positions: f64,
}

/// The stack's auxiliary A5 accuracy on fresh words of `length`, which must fit
/// the context.
pub fn a5_stack_accuracy(
    model: &TrackedStack,
    task: &A5Task,
    length: usize,
    words: usize,
    batch: usize,
    seed: u64,
) -> Result<PrefixAccuracy> {
    if words == 0 || batch == 0 {
        return Err(invalid("A5 evaluation needs words"));
    }
    let mut positions = Vec::new();
    let mut p = 1;
    while p <= length {
        positions.push(p);
        p *= 2;
    }
    if positions.last() != Some(&length) {
        positions.push(length);
    }
    let mut rng = Rng::new(seed);
    let mut hits = vec![0usize; length];
    let mut remaining = words;
    while remaining > 0 {
        let rows = batch.min(remaining);
        let (tokens, classes) = task.batch(&mut rng, rows, length);
        let ids: Vec<u32> = tokens.iter().map(|&t| TEXT_VOCAB as u32 + t).collect();
        let predicted = model
            .a5_logits(&ids, rows, length)?
            .detach()
            .argmax(D::Minus1)?
            .to_vec1::<u32>()?;
        for (index, (p, c)) in predicted.iter().zip(&classes).enumerate() {
            if p == c {
                hits[index % length] += 1;
            }
        }
        remaining -= rows;
    }
    Ok(PrefixAccuracy {
        length,
        words,
        by_position: positions
            .iter()
            .map(|&p| (p, hits[p - 1] as f64 / words as f64))
            .collect(),
        all_positions: hits.iter().sum::<usize>() as f64 / (words * length) as f64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a5_task_has_sixty_paired_classes_and_a_generating_triple() -> Result<()> {
        let task = A5Task::standard()?;
        assert_eq!(task.generators.len(), 3);
        let mut sizes = [0usize; A5_ORDER];
        for &class in &task.class_of {
            sizes[class as usize] += 1;
        }
        assert!(sizes.iter().all(|&n| n == 2));
        assert_eq!(closure_size(&task.generators, task.identity), GROUP_ORDER);
        Ok(())
    }

    #[test]
    fn tensor_hamilton_matches_the_scalar_product() -> Result<()> {
        let device = Device::Cpu;
        let (a, b) = ([0.3, -0.5, 0.7, 0.1], [-0.2, 0.9, 0.4, -0.6]);
        let expected = quaternion_product(a, b);
        let ta = Tensor::from_vec(a.map(|x| x as f32).to_vec(), (1, 4), &device)?;
        let tb = Tensor::from_vec(b.map(|x| x as f32).to_vec(), (1, 4), &device)?;
        let got = hamilton(&ta, &tb)?.flatten_all()?.to_vec1::<f32>()?;
        for (g, e) in got.iter().zip(expected) {
            assert!((*g as f64 - e).abs() < 1e-6);
        }
        Ok(())
    }

    /// Exact canonical generators close to all 120 elements, and the served
    /// automaton tracks every word exactly at any length.
    #[test]
    fn exact_generators_snap_to_2i_and_serve_exactly() -> Result<()> {
        let task = A5Task::standard()?;
        let device = Device::Cpu;
        let config = LaneConfig {
            kind: LaneKind::Quaternion,
            vocab: task.vocab(),
            lanes: 1,
            hidden: 4,
            classes: A5_ORDER,
            seed: 1,
        };
        let mut model = LaneModel::new(config, &device)?;
        // Lane transport for token i: left multiplication by the inverse of
        // generator i, so the lane state is the inverse of the label element.
        let roots = canonical_h4_roots();
        let mut raw = Vec::new();
        for &g in &task.generators {
            let q = roots[g as usize].to_array();
            raw.extend([q[0] as f32, -q[1] as f32, -q[2] as f32, -q[3] as f32]);
        }
        model.transport.var = Some(Var::from_tensor(&Tensor::from_vec(
            raw,
            (task.vocab(), 4),
            &device,
        )?)?);
        let automaton = LaneAutomaton::snap(&model, &task, 0, 64, 32, 7)?
            .ok_or_else(|| invalid("exact generators did not close"))?;
        assert_eq!(automaton.order, GROUP_ORDER);
        // Read out by A5 class, s and -s are equivalent: 2I minimises to A5.
        assert_eq!(automaton.minimal_order, A5_ORDER);
        assert!(automaton.max_trace_deviation < 1e-6);
        assert!((automaton.fit_accuracy - 1.0).abs() < 1e-12);
        let served = automaton.evaluate(&task, &[1000], 8, 11);
        assert!((served[0].all_positions - 1.0).abs() < 1e-12);
        Ok(())
    }

    /// Without lanes, the tracked stack's embed -> hidden_from_input -> head
    /// path is the stack's own forward pass, so the split changes nothing.
    #[test]
    fn tracked_stack_without_lanes_equals_the_stack_forward() -> Result<()> {
        let task = A5Task::standard()?;
        let device = Device::Cpu;
        let config = StackConfig {
            arch: crate::geometric_stack::StackArch::Geometric,
            vocab_size: TEXT_VOCAB + task.vocab(),
            width: 16,
            heads: 2,
            mlp_hidden: 16,
            context: 8,
            pattern: "ra".into(),
            read: crate::geometric_stack::ReadScore::Dot,
            rotation: true,
            seed: 5,
        };
        let tracked = TrackedStack::new(config, None, &task, 5, &device)?;
        let ids: Vec<u32> = (0..16).map(|i| (i * 257 % 4099) as u32).collect();
        let direct = tracked
            .stack
            .forward(&ids, 2, 8)?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let split = tracked
            .text_logits(&ids, 2, 8)?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert_eq!(direct, split);
        let with_lanes = TrackedStack::new(
            tracked.stack.config.clone(),
            Some((LaneKind::Quaternion, 2)),
            &task,
            5,
            &device,
        )?;
        let (_, side) = with_lanes.parameter_counts();
        assert_eq!(side, 4099 * 8 + 16 * 8 + 60 * 16 + 60);
        assert_eq!(with_lanes.a5_logits(&ids, 2, 8)?.dims(), &[16, A5_ORDER]);
        Ok(())
    }

    /// Every lane kind's float states equal the product of the per-token
    /// matrices that `transport_matrix` reports (for reflection pairs,
    /// `x -> p x r` through two cumulative scans), so snapping serves the same
    /// transport the float model trained.
    #[test]
    fn lane_states_match_their_transport_matrices() -> Result<()> {
        let device = Device::Cpu;
        for kind in [
            LaneKind::Quaternion,
            LaneKind::Phase,
            LaneKind::ReflectionPair,
            LaneKind::Frozen,
        ] {
            let config = LaneConfig {
                kind,
                vocab: 3,
                lanes: 2,
                hidden: 4,
                classes: A5_ORDER,
                seed: 9,
            };
            let model = LaneModel::new(config, &device)?;
            let tokens = [2u32, 0, 1, 1, 2];
            let states = model.states(&tokens, 1, tokens.len())?;
            for lane in 0..2 {
                let mut x = [1.0, 0.0, 0.0, 0.0];
                for (t, &token) in tokens.iter().enumerate() {
                    let m = model.transport_matrix(token as usize, lane)?;
                    x = [0, 1, 2, 3].map(|i| (0..4).map(|k| m[i * 4 + k] * x[k]).sum());
                    let got = states.get(0)?.get(t)?.get(lane)?.to_vec1::<f32>()?;
                    for (g, e) in got.iter().zip(x) {
                        assert!(
                            (*g as f64 - e).abs() < 1e-4,
                            "{kind:?} lane {lane} step {t}"
                        );
                    }
                }
            }
        }
        Ok(())
    }
}
