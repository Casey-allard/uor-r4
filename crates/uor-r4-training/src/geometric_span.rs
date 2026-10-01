//! Learned, ordered geometric spans for offline CPU training.
//!
//! Token rows are the original learned embedding lookup, not contextual states
//! or query/key projections. Each four-coordinate row is placed at the signed
//! nearest canonical 2I root; exact zero denotes the identity ACTION. Token
//! magnitude is intentionally unused. A nonempty span has fixed unit radius in
//! every lane, including when its product is the identity. Absence is separate.
//!
//! Four finite logits choose the earliest argmax: HOLD=0, OPEN=1, APPEND=2,
//! COMMIT=3. OPEN resets the working tuple to identity and opens an EMPTY span;
//! it does not consume its delimiter. APPEND consumes the current static token
//! action only while open, right-multiplying the working tuple by exact table
//! lookup. COMMIT copies a nonempty open tuple into held state, then closes the
//! span. An empty or unopened COMMIT preserves held state. HOLD does nothing.
//! Every output is OLD held state, before that row's action; absent held state
//! emits zero. LastToken changes only APPEND, replacing rather than composing.
//!
//! Backward is deliberately biased, with no relaxation over all group states:
//! * selected hard branches propagate working/held adjoints; selected APPEND
//!   uses the smooth Hamilton Jacobian at hard roots and the token-direction
//!   x/sqrt(|x|^2+eps^2) Jacobian, eps=2^-20 (zero input has zero derivative);
//! * controller logits receive the softmax derivative of four detached branch
//!   coordinate outcomes evaluated at the hard prior state, temperature one;
//! * active/nonempty/held-valid flags and legality tests are detached;
//! * incoming gradients at emitted valid held roots are projected onto their
//!   tangent planes, discarding radius credit because span radius is fixed.
//! The controller surrogate need not stay on the unit sphere. Its branch
//! differences are a local training surrogate, not a probability distribution
//! over executed states. These rules supply first-order gradients only. Labels
//! are never inputs here. In particular, all-HOLD/absent trajectories have no
//! content gradient: training event credit must establish valid span paths.
//!
//! Products are finite addresses, not collision-free representations of words:
//! order can matter, inverses can cancel, and different spans can share a tuple.
//! Exact source occurrence/span/version identity remains the caller's concern.

use std::sync::OnceLock;

use candle_core::{CpuStorage, CustomOp2, DType, Layout, Shape, Tensor};
use serde::{Deserialize, Serialize};
use uor_r4_core::native_geometric::learner::{
    embedding::canonical_h4_roots,
    group_table::{group_table, GROUP_ORDER, ROW_STRIDE},
};

use crate::geometric_address::{encode_lane, geometry_digest, NORM_EPSILON};
use crate::{invalid, Result};

pub const SCHEMA: &str = "uor-r4.geometric-span/1";
pub const HOLD: usize = 0;
pub const OPEN: usize = 1;
pub const APPEND: usize = 2;
pub const COMMIT: usize = 3;
const STATE_RULE: &str =
    "old-held;open-empty;append-open-right;commit-nonempty-close;unit-span-zero-absent/1";
const SURROGATE_RULE: &str =
    "selected-hamilton/token-normalized;softmax-branch-delta;flags-stop;output-tangent/1";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeometricSpanConfig {
    pub schema: String,
    pub width: usize,
    pub root_table_sha256: String,
    pub action_order: [String; 4],
    pub state_rule: String,
    pub surrogate_rule: String,
    pub norm_epsilon: f64,
}

impl GeometricSpanConfig {
    pub fn new(width: usize) -> Result<Self> {
        if width == 0 || width > 8192 || width % 4 != 0 {
            return Err(invalid(
                "geometric span width must be a multiple of four in 4..8192",
            ));
        }
        Ok(Self {
            schema: SCHEMA.into(),
            width,
            root_table_sha256: geometry_digest().to_owned(),
            action_order: [
                "hold".into(),
                "open".into(),
                "append".into(),
                "commit".into(),
            ],
            state_rule: STATE_RULE.into(),
            surrogate_rule: SURROGATE_RULE.into(),
            norm_epsilon: NORM_EPSILON,
        })
    }

    pub fn validate(&self, width: usize) -> Result<()> {
        if *self != Self::new(width)? {
            return Err(invalid(
                "geometric span config, action ordering or geometry binding differs",
            ));
        }
        Ok(())
    }
}

/// A per-call diagnostic policy; the learned operator uses Ordered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpanPolicy {
    Ordered,
    LastToken,
}

fn roots() -> &'static [[f64; 4]; GROUP_ORDER] {
    static ROOTS: OnceLock<[[f64; 4]; GROUP_ORDER]> = OnceLock::new();
    ROOTS.get_or_init(|| std::array::from_fn(|i| canonical_h4_roots()[i].to_array()))
}

fn root(id: u8) -> [f64; 4] {
    roots()[usize::from(id)]
}

fn product(a: u8, b: u8) -> u8 {
    group_table().product[usize::from(a) * ROW_STRIDE + usize::from(b)]
}

fn action(logits: &[f32]) -> usize {
    let mut best = HOLD;
    for i in 1..4 {
        if logits[i] > logits[best] {
            best = i;
        }
    }
    best
}

fn probabilities(logits: &[f32]) -> [f64; 4] {
    let maximum = f64::from(logits[action(logits)]);
    let mut p = std::array::from_fn(|i| (f64::from(logits[i]) - maximum).exp());
    let total = p.iter().sum::<f64>();
    for value in &mut p {
        *value /= total;
    }
    p
}

#[derive(Clone, Debug)]
struct State {
    working: Vec<u8>,
    held: Vec<u8>,
    active: bool,
    nonempty: bool,
    held_valid: bool,
}

impl State {
    fn empty(lanes: usize) -> Self {
        Self {
            working: vec![group_table().identity; lanes],
            held: vec![group_table().identity; lanes],
            active: false,
            nonempty: false,
            held_valid: false,
        }
    }

    fn advance(&mut self, selected: usize, token: &[u8], policy: SpanPolicy) {
        match selected {
            OPEN => {
                self.working.fill(group_table().identity);
                self.active = true;
                self.nonempty = false;
            }
            APPEND if self.active => {
                for (working, &token) in self.working.iter_mut().zip(token) {
                    *working = match policy {
                        SpanPolicy::Ordered => product(*working, token),
                        SpanPolicy::LastToken => token,
                    };
                }
                self.nonempty = true;
            }
            COMMIT => {
                if self.active && self.nonempty {
                    self.held.copy_from_slice(&self.working);
                    self.held_valid = true;
                }
                self.active = false;
                self.nonempty = false;
            }
            _ => {}
        }
    }

    // Detached geometric outcomes of all four actions at the same hard state.
    fn branch(
        &self,
        selected: usize,
        lane: usize,
        token: u8,
        policy: SpanPolicy,
    ) -> ([f64; 4], [f64; 4]) {
        let w = match selected {
            OPEN => group_table().identity,
            APPEND if self.active => match policy {
                SpanPolicy::Ordered => product(self.working[lane], token),
                SpanPolicy::LastToken => token,
            },
            _ => self.working[lane],
        };
        let h = if selected == COMMIT && self.active && self.nonempty {
            root(self.working[lane])
        } else if self.held_valid {
            root(self.held[lane])
        } else {
            [0.; 4]
        };
        (root(w), h)
    }
}

fn dot(a: [f64; 4], b: [f64; 4]) -> f64 {
    (0..4).map(|i| a[i] * b[i]).sum()
}

fn tangent(q: [f64; 4], g: [f64; 4]) -> [f64; 4] {
    let radial = dot(q, g) / dot(q, q);
    std::array::from_fn(|i| g[i] - radial * q[i])
}

fn add(target: &mut [f64; 4], value: [f64; 4]) {
    for i in 0..4 {
        target[i] += value[i];
    }
}

// Jacobians of ordinary Hamilton a*b, not conjugate(a)*b.
fn hamilton_pullback(a: [f64; 4], b: [f64; 4], g: [f64; 4]) -> ([f64; 4], [f64; 4]) {
    let da = [
        g[0] * b[0] + g[1] * b[1] + g[2] * b[2] + g[3] * b[3],
        -g[0] * b[1] + g[1] * b[0] - g[2] * b[3] + g[3] * b[2],
        -g[0] * b[2] + g[1] * b[3] + g[2] * b[0] - g[3] * b[1],
        -g[0] * b[3] - g[1] * b[2] + g[2] * b[1] + g[3] * b[0],
    ];
    let db = [
        g[0] * a[0] + g[1] * a[1] + g[2] * a[2] + g[3] * a[3],
        -g[0] * a[1] + g[1] * a[0] + g[2] * a[3] - g[3] * a[2],
        -g[0] * a[2] - g[1] * a[3] + g[2] * a[0] + g[3] * a[1],
        -g[0] * a[3] + g[1] * a[2] - g[2] * a[1] + g[3] * a[0],
    ];
    (da, db)
}

fn token_pullback(x: [f32; 4], g: [f64; 4]) -> [f64; 4] {
    let x = x.map(f64::from);
    let norm2 = dot(x, x);
    if norm2 == 0. {
        return [0.; 4];
    }
    let d = (norm2 + NORM_EPSILON * NORM_EPSILON).sqrt();
    let inner = dot(x, g);
    std::array::from_fn(|i| g[i] / d - x[i] * inner / (d * d * d))
}

fn contiguous<'a>(storage: &'a CpuStorage, layout: &Layout) -> candle_core::Result<&'a [f32]> {
    let values = storage.as_slice::<f32>()?;
    match layout.contiguous_offsets() {
        Some((start, end)) => Ok(&values[start..end]),
        None => candle_core::bail!("geometric span needs contiguous CPU F32 inputs"),
    }
}

struct SpanOp {
    batch: usize,
    time: usize,
    width: usize,
    policy: SpanPolicy,
}

impl SpanOp {
    fn validate(&self, tokens: &[f32], logits: &[f32]) -> candle_core::Result<()> {
        if tokens.len() != self.batch * self.time * self.width
            || logits.len() != self.batch * self.time * 4
        {
            candle_core::bail!("geometric span packed shape differs");
        }
        if tokens.iter().chain(logits).any(|x| !x.is_finite()) {
            candle_core::bail!("nonfinite geometric span token or controller input");
        }
        Ok(())
    }

    fn codes(&self, tokens: &[f32]) -> candle_core::Result<Vec<u8>> {
        tokens
            .chunks_exact(4)
            .map(|x| {
                encode_lane([x[0], x[1], x[2], x[3]])
                    .map(|c| c.root)
                    .map_err(|e| candle_core::Error::Msg(e.to_string()))
            })
            .collect()
    }

    fn trace(&self, codes: &[u8], logits: &[f32], b: usize) -> Vec<State> {
        let lanes = self.width / 4;
        let mut state = State::empty(lanes);
        let mut before = Vec::with_capacity(self.time);
        for t in 0..self.time {
            before.push(state.clone());
            let row = b * self.time + t;
            state.advance(
                action(&logits[row * 4..row * 4 + 4]),
                &codes[row * lanes..(row + 1) * lanes],
                self.policy,
            );
        }
        before
    }

    fn forward(&self, tokens: &[f32], logits: &[f32]) -> candle_core::Result<Vec<f32>> {
        self.validate(tokens, logits)?;
        let codes = self.codes(tokens)?;
        let mut output = vec![0.; tokens.len()];
        for b in 0..self.batch {
            for (t, state) in self.trace(&codes, logits, b).iter().enumerate() {
                if state.held_valid {
                    for lane in 0..self.width / 4 {
                        let offset = (b * self.time + t) * self.width + lane * 4;
                        for i in 0..4 {
                            output[offset + i] = root(state.held[lane])[i] as f32;
                        }
                    }
                }
            }
        }
        Ok(output)
    }

    fn backward(
        &self,
        tokens: &[f32],
        logits: &[f32],
        grad: &[f32],
    ) -> candle_core::Result<(Vec<f32>, Vec<f32>)> {
        self.validate(tokens, logits)?;
        if grad.len() != tokens.len() || grad.iter().any(|x| !x.is_finite()) {
            candle_core::bail!("nonfinite or invalid geometric span upstream gradient");
        }
        let lanes = self.width / 4;
        let codes = self.codes(tokens)?;
        let mut dt = vec![0f64; tokens.len()];
        let mut dz = vec![0f64; logits.len()];
        for b in 0..self.batch {
            let before = self.trace(&codes, logits, b);
            let mut gw = vec![[0.; 4]; lanes];
            let mut gh = vec![[0.; 4]; lanes];
            for t in (0..self.time).rev() {
                let row = b * self.time + t;
                let state = &before[t];
                let z = &logits[row * 4..row * 4 + 4];
                let selected = action(z);
                let mut branch_credit = [0.; 4];
                for lane in 0..lanes {
                    let token = codes[row * lanes + lane];
                    for (a, credit) in branch_credit.iter_mut().enumerate() {
                        let (w, h) = state.branch(a, lane, token, self.policy);
                        *credit += dot(gw[lane], w) + dot(gh[lane], h);
                    }
                }
                let p = probabilities(z);
                let mean = (0..4).map(|a| p[a] * branch_credit[a]).sum::<f64>();
                for a in 0..4 {
                    dz[row * 4 + a] = p[a] * (branch_credit[a] - mean);
                }
                for lane in 0..lanes {
                    let offset = row * self.width + lane * 4;
                    match selected {
                        OPEN => gw[lane] = [0.; 4],
                        APPEND if state.active => {
                            let token = codes[row * lanes + lane];
                            let (prior, token_grad) = match self.policy {
                                SpanPolicy::Ordered => hamilton_pullback(
                                    root(state.working[lane]),
                                    root(token),
                                    gw[lane],
                                ),
                                SpanPolicy::LastToken => ([0.; 4], gw[lane]),
                            };
                            gw[lane] = prior;
                            let value = token_pullback(
                                [
                                    tokens[offset],
                                    tokens[offset + 1],
                                    tokens[offset + 2],
                                    tokens[offset + 3],
                                ],
                                token_grad,
                            );
                            for i in 0..4 {
                                dt[offset + i] += value[i];
                            }
                        }
                        COMMIT if state.active && state.nonempty => {
                            add(&mut gw[lane], gh[lane]);
                            gh[lane] = [0.; 4];
                        }
                        _ => {}
                    }
                    // Outputs precede updates, so their adjoints enter only
                    // AFTER the current transition/controller pullback.
                    if state.held_valid {
                        let incoming = std::array::from_fn(|i| f64::from(grad[offset + i]));
                        add(&mut gh[lane], tangent(root(state.held[lane]), incoming));
                    }
                }
            }
        }
        let checked = |v: Vec<f64>| -> candle_core::Result<Vec<f32>> {
            let v = v.into_iter().map(|x| x as f32).collect::<Vec<_>>();
            if v.iter().any(|x| !x.is_finite()) {
                candle_core::bail!("nonfinite geometric span surrogate gradient");
            }
            Ok(v)
        };
        Ok((checked(dt)?, checked(dz)?))
    }
}

impl CustomOp2 for SpanOp {
    fn name(&self) -> &'static str {
        "geometric-span-ordered-actions"
    }

    fn cpu_fwd(
        &self,
        s1: &CpuStorage,
        l1: &Layout,
        s2: &CpuStorage,
        l2: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        Ok((
            CpuStorage::F32(self.forward(contiguous(s1, l1)?, contiguous(s2, l2)?)?),
            Shape::from((self.batch, self.time, self.width)),
        ))
    }

    fn bwd(
        &self,
        tokens: &Tensor,
        logits: &Tensor,
        _out: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>)> {
        let (dt, dz) = self.backward(
            &tokens.flatten_all()?.to_vec1::<f32>()?,
            &logits.flatten_all()?.to_vec1::<f32>()?,
            &grad.flatten_all()?.to_vec1::<f32>()?,
        )?;
        Ok((
            Some(Tensor::from_vec(dt, tokens.shape(), tokens.device())?),
            Some(Tensor::from_vec(dz, logits.shape(), logits.device())?),
        ))
    }
}

/// Emit the previously committed span at each row. No labels or source IDs are
/// accepted. Both fit and inference execute the same finite predicted actions.
pub fn produce(
    tokens: &Tensor,
    logits: &Tensor,
    config: &GeometricSpanConfig,
    policy: SpanPolicy,
) -> Result<Tensor> {
    let (batch, time, width) = tokens.dims3()?;
    config.validate(width)?;
    if batch == 0
        || time == 0
        || logits.dims() != [batch, time, 4]
        || tokens.dtype() != DType::F32
        || logits.dtype() != DType::F32
        || !tokens.device().is_cpu()
        || !logits.device().is_cpu()
    {
        return Err(invalid(
            "geometric span requires nonempty CPU F32 tokens[B,T,W] and logits[B,T,4]",
        ));
    }
    let _ = batch
        .checked_mul(time)
        .and_then(|n| n.checked_mul(width))
        .ok_or_else(|| invalid("geometric span shape overflows address space"))?;
    Ok(tokens.contiguous()?.apply_op2(
        &logits.contiguous()?,
        SpanOp {
            batch,
            time,
            width,
            policy,
        },
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::{Device, Var};

    fn logits(actions: &[usize]) -> Vec<f32> {
        actions
            .iter()
            .flat_map(|&a| (0..4).map(move |i| if i == a { 1. } else { -0.4 }))
            .collect()
    }

    fn op(time: usize, policy: SpanPolicy) -> SpanOp {
        SpanOp {
            batch: 1,
            time,
            width: 4,
            policy,
        }
    }

    fn coordinates(codes: &[u8]) -> Vec<f32> {
        codes
            .iter()
            .flat_map(|&c| root(c).map(|x| x as f32))
            .collect()
    }

    fn encoded_output(output: &[f32], t: usize) -> crate::geometric_address::AddressCode {
        encode_lane(output[t * 4..t * 4 + 4].try_into().unwrap()).unwrap()
    }

    fn hamilton(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
        [
            a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
            a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
            a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
            a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
        ]
    }

    #[test]
    fn geometric_span_fsm_distinguishes_empty_identity_and_old_held() {
        let identity = group_table().identity;
        let a = encode_lane([0., 1., 0., 0.]).unwrap().root;
        let inverse = group_table().inverse[usize::from(a)];
        let actions = [
            APPEND, COMMIT, OPEN, COMMIT, HOLD, OPEN, APPEND, APPEND, COMMIT, HOLD, OPEN, COMMIT,
            HOLD,
        ];
        let mut token = vec![identity; actions.len()];
        token[6] = a;
        token[7] = inverse;
        let result = op(actions.len(), SpanPolicy::Ordered)
            .forward(&coordinates(&token), &logits(&actions))
            .unwrap();
        for t in 0..=8 {
            assert!(!encoded_output(&result, t).present);
        }
        for t in 9..actions.len() {
            let code = encoded_output(&result, t);
            assert!(code.present);
            assert_eq!(code.root, identity);
            assert_eq!(code.radius_bin, 16);
        }
        // OPEN and COMMIT delimiters are not multiplied into content.
        let mut changed = coordinates(&token);
        for t in [2, 3, 5, 8, 10, 11] {
            changed[t * 4..t * 4 + 4].copy_from_slice(&root(a).map(|x| x as f32));
        }
        assert_eq!(
            result,
            op(actions.len(), SpanPolicy::Ordered)
                .forward(&changed, &logits(&actions))
                .unwrap()
        );
        // The zero embedding is the identity action, not an absent token.
        let zero = op(4, SpanPolicy::Ordered)
            .forward(&vec![0.; 16], &logits(&[OPEN, APPEND, COMMIT, HOLD]))
            .unwrap();
        assert_eq!(encoded_output(&zero, 3).root, identity);
        assert!(encoded_output(&zero, 3).present);
    }

    #[test]
    fn geometric_span_order_control_batch_causality_and_last_token() {
        let id = group_table().identity;
        let a = encode_lane([0., 1., 0., 0.]).unwrap().root;
        let b = encode_lane([0., 0., 1., 0.]).unwrap().root;
        let anchor = encode_lane([0.5, 0.5, 0.5, 0.5]).unwrap().root;
        let tail = encode_lane([0., 0., 0., 1.]).unwrap().root;
        let first = [id, anchor, a, b, tail, id, id];
        let second = [id, anchor, b, a, tail, id, id];
        let acts = [OPEN, APPEND, APPEND, APPEND, APPEND, COMMIT, HOLD];
        let tokens = coordinates(&first.into_iter().chain(second).collect::<Vec<_>>());
        let controller = logits(&acts.into_iter().chain(acts).collect::<Vec<_>>());
        let ordered = SpanOp {
            batch: 2,
            time: 7,
            width: 4,
            policy: SpanPolicy::Ordered,
        }
        .forward(&tokens, &controller)
        .unwrap();
        assert_ne!(
            encoded_output(&ordered, 6).root,
            encoded_output(&ordered, 13).root
        );
        assert_eq!(
            encoded_output(&ordered, 6).root,
            product(product(product(anchor, a), b), tail)
        );
        let last = SpanOp {
            batch: 2,
            time: 7,
            width: 4,
            policy: SpanPolicy::LastToken,
        }
        .forward(&tokens, &controller)
        .unwrap();
        assert_eq!(encoded_output(&last, 6).root, tail);
        assert_eq!(encoded_output(&last, 13).root, tail);
        // Every prefix agrees, and batch two starts absent rather than carrying
        // batch one's committed identity.
        assert!(!encoded_output(&ordered, 7).present);
        for prefix in 1..=7 {
            let short = op(prefix, SpanPolicy::Ordered)
                .forward(&tokens[..prefix * 4], &controller[..prefix * 4])
                .unwrap();
            assert_eq!(short, ordered[..prefix * 4]);
        }
        assert_eq!(action(&[0.; 4]), HOLD);
        assert_eq!(action(&[0., 2., 2., 1.]), OPEN);
    }

    #[test]
    fn geometric_span_declared_surrogate_matches_local_finite_differences() {
        let actions = [OPEN, APPEND, APPEND, COMMIT, HOLD];
        let z = logits(&actions);
        let x = vec![
            1., 0., 0., 0., 0.9, 0.2, -0.1, 0.3, 0.3, 1.1, 0.2, -0.1, 1., 0., 0., 0., 1., 0., 0.,
            0.,
        ];
        let scan = op(5, SpanPolicy::Ordered);
        let codes = scan.codes(&x).unwrap();
        let state = scan.trace(&codes, &z, 0);
        let target = root(state[4].held[0]);
        let upstream = [0.3f32, -0.7, 0.4, 0.2];
        let g = tangent(target, upstream.map(f64::from));
        let mut grad = vec![0.; 20];
        grad[16..20].copy_from_slice(&upstream);
        let (dx, dz) = scan.backward(&x, &z, &grad).unwrap();
        let eps = 1e-5;
        let normalize = |v: [f64; 4]| {
            let d = (dot(v, v) + NORM_EPSILON * NORM_EPSILON).sqrt();
            v.map(|q| q / d)
        };
        let base_a: [f64; 4] = std::array::from_fn(|i| f64::from(x[4 + i]));
        let base_b: [f64; 4] = std::array::from_fn(|i| f64::from(x[8 + i]));
        let na = normalize(base_a);
        let nb = normalize(base_b);
        // This is a finite difference of the declared frozen-hard-root local
        // surrogate, deliberately not a finite difference of quantization.
        let smooth = |a: [f64; 4], b: [f64; 4]| {
            let aa = std::array::from_fn(|i| root(codes[1])[i] + normalize(a)[i] - na[i]);
            let bb = std::array::from_fn(|i| root(codes[2])[i] + normalize(b)[i] - nb[i]);
            dot(g, hamilton(aa, bb))
        };
        for which in 0..2 {
            for i in 0..4 {
                let (mut ap, mut am, mut bp, mut bm) = (base_a, base_a, base_b, base_b);
                if which == 0 {
                    ap[i] += eps;
                    am[i] -= eps;
                } else {
                    bp[i] += eps;
                    bm[i] -= eps;
                }
                let numerical = (smooth(ap, bp) - smooth(am, bm)) / (2. * eps);
                assert!((f64::from(dx[(which + 1) * 4 + i]) - numerical).abs() < 2e-6);
            }
        }
        // Controller branch credit at the second APPEND, before the following
        // hard COMMIT, uses four detached branch outcomes at that hard state.
        let branch: [f64; 4] =
            std::array::from_fn(|a| dot(g, state[2].branch(a, 0, codes[2], SpanPolicy::Ordered).0));
        let smooth_gate = |zz: [f64; 4]| {
            let max = zz.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let e = zz.map(|v| (v - max).exp());
            (0..4).map(|i| e[i] * branch[i]).sum::<f64>() / e.iter().sum::<f64>()
        };
        for i in 0..4 {
            let mut plus = std::array::from_fn(|j| f64::from(z[8 + j]));
            let mut minus = plus;
            plus[i] += eps;
            minus[i] -= eps;
            let numerical = (smooth_gate(plus) - smooth_gate(minus)) / (2. * eps);
            assert!((f64::from(dz[8 + i]) - numerical).abs() < 2e-6);
        }
        assert!(dx[4..8].iter().any(|x| x.abs() > 1e-6));
        assert!(dz[8..12].iter().any(|x| x.abs() > 1e-6));
        for t in [0, 3, 4] {
            assert_eq!(&dx[t * 4..t * 4 + 4], &[0.; 4]);
        }
        // Fixed unit radius has no upstream learning channel.
        grad[16..20].copy_from_slice(&target.map(|q| q as f32));
        let (radial_x, radial_z) = scan.backward(&x, &z, &grad).unwrap();
        assert!(radial_x.iter().chain(&radial_z).all(|g| g.abs() < 1e-6));
    }

    #[test]
    fn geometric_span_actual_autograd_reaches_earlier_actions_and_controller() {
        let x = vec![
            1., 0., 0., 0., 0.9, 0.2, -0.1, 0.3, 0.3, 1.1, 0.2, -0.1, 1., 0., 0., 0., 1., 0., 0.,
            0.,
        ];
        let z = logits(&[OPEN, APPEND, APPEND, COMMIT, HOLD]);
        let tokens = Var::from_vec(x.clone(), (1, 5, 4), &Device::Cpu).unwrap();
        let control = Var::from_vec(z.clone(), (1, 5, 4), &Device::Cpu).unwrap();
        let config = GeometricSpanConfig::new(4).unwrap();
        let prior = produce(&tokens, &control, &config, SpanPolicy::Ordered).unwrap();
        let direction =
            Tensor::from_vec(vec![0.3f32, -0.7, 0.4, 0.2], (1, 1, 4), &Device::Cpu).unwrap();
        let loss = prior
            .narrow(1, 4, 1)
            .unwrap()
            .mul(&direction)
            .unwrap()
            .sum_all()
            .unwrap();
        let gradients = loss.backward().unwrap();
        let dx = gradients
            .get(&tokens)
            .unwrap()
            .flatten_all()
            .unwrap()
            .to_vec1::<f32>()
            .unwrap();
        let dz = gradients
            .get(&control)
            .unwrap()
            .flatten_all()
            .unwrap()
            .to_vec1::<f32>()
            .unwrap();
        let mut upstream = vec![0.; 20];
        upstream[16..20].copy_from_slice(&[0.3, -0.7, 0.4, 0.2]);
        let expected = op(5, SpanPolicy::Ordered)
            .backward(&x, &z, &upstream)
            .unwrap();
        assert_eq!(dx, expected.0);
        assert_eq!(dz, expected.1);
        assert!(dx[4..8].iter().any(|x| x.abs() > 1e-6));
        assert!(dz[8..12].iter().any(|x| x.abs() > 1e-6));
        assert!(dx.iter().chain(&dz).all(|x| x.is_finite()));
        let all_hold = logits(&[HOLD; 5]);
        let no_path = op(5, SpanPolicy::Ordered)
            .backward(&x, &all_hold, &upstream)
            .unwrap();
        assert!(no_path.0.iter().chain(&no_path.1).all(|x| *x == 0.));
        let last = op(5, SpanPolicy::LastToken)
            .backward(&x, &z, &upstream)
            .unwrap();
        assert_eq!(&last.0[4..8], &[0.; 4]);
        assert!(last.0[8..12].iter().any(|x| x.abs() > 1e-6));
    }

    #[test]
    fn geometric_span_rejects_invalid_inputs_and_binds_configuration() {
        let config = GeometricSpanConfig::new(4).unwrap();
        let json = serde_json::to_string(&config).unwrap();
        let loaded: GeometricSpanConfig = serde_json::from_str(&json).unwrap();
        loaded.validate(4).unwrap();
        let mut bad = loaded.clone();
        bad.action_order.swap(1, 2);
        assert!(bad.validate(4).is_err());
        let mut bad = loaded.clone();
        bad.root_table_sha256.push('0');
        assert!(bad.validate(4).is_err());
        for width in [0, 1, 5, 8196] {
            assert!(GeometricSpanConfig::new(width).is_err());
        }
        let mut object = serde_json::to_value(config).unwrap();
        object["ignored"] = true.into();
        assert!(serde_json::from_value::<GeometricSpanConfig>(object).is_err());
        let scan = op(1, SpanPolicy::Ordered);
        assert!(scan.forward(&[0., f32::NAN, 0., 0.], &[0.; 4]).is_err());
        assert!(scan
            .forward(&[0.; 4], &[0., 0., f32::INFINITY, 0.])
            .is_err());
        assert!(scan.backward(&[0.; 4], &[0.; 4], &[f32::NAN; 4]).is_err());
        let p = probabilities(&[-f32::MAX, f32::MAX, 0., 1.]);
        assert!(p.iter().all(|x| x.is_finite()));
        assert_eq!(p[1], 1.);
        let wrong = Tensor::zeros((1, 1, 3), DType::F32, &Device::Cpu).unwrap();
        let tokens = Tensor::zeros((1, 1, 4), DType::F32, &Device::Cpu).unwrap();
        assert!(produce(&tokens, &wrong, &loaded, SpanPolicy::Ordered).is_err());
    }
}
