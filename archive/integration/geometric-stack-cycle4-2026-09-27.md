# Cycle 4: a capacity-matched, parallel-trainable geometric stack

2026-09-27 · Claude lab track · References #820, #973 · Work card: [#973 comment](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5851700710)

**Status.** An evidence note from the lab track, not a decision record. Run records: [cycle-4 packet](../evidence/geometric-stack-cycle4-2026-09-27/README.md). Labels: **Measured** (Rust runs in the lab sandbox, seeds stated), **Derived**, **Literature**, **Hypothesis**. Results are development measurements on code; nothing here is a final holdout or a language qualification.

## 0. Findings

*Measured* on code, one seed per arm unless stated. Development NLL on the repository split's 512 evenly spaced windows (131,072 targets).

- **At full exposure the geometric stack matches the transformer control: 1.9981 against 2.0111 nats** (§6).
  - Setup: 7,324 updates (29,999,104 target visits), 7.15M against 7.16M parameters, the same windows.
  - The stack is 0.0130 nats lower (0.0052 bits per byte). That is within the seed spread seen at 1,000 updates, so it reads as parity, not an advantage.
  - Both train at the same speed: 789 and 788 tokens/s.
  - By the card's rule, fixed before the pilots, the stack is **viable**. Under the programme's ROADMAP (§2) it becomes the main-line candidate, and the director's D0 decision ([#820](https://github.com/UOR-Foundation/uor-r4/issues/820), 2026-09-28) adopted it as the main-line core. Its next step is export to the native, multiplier-free serving contract (D11, R1–R5); integer serving under D10 is now a frozen comparator.
- **At 1,000 updates the stack led the control at every learning rate piloted** (§5): by 0.158 nats at each arm's selected rate (4e-3 for the stack, 2e-3 for the control). The lead narrowed as training continued and had closed by about 2,750 updates (§6).
- **Configuration matters more than the read's score** (§7).
  - Reads-only Lorentz is the best 1,000-update configuration measured: 2.5531 and 2.5382 over two seeds.
  - Inside `rrarra`, Dot and Lorentz trade places across two seeds.
  - With every layer a read, Lorentz leads Dot by 0.076 at seed 1.
  - Under ROADMAP ruling 8, an all-reads stack is a transformer comparator, not a candidate backbone.
- **Integer serving under D10 costs little, and the cost is in the exported parameters, not the arithmetic** (§8).
  - The stacks lose 0.011–0.013 nats with round-to-nearest, and 0.006–0.007 with GPTQ, which recalibrates only the matrices.
  - The integer arithmetic adds nothing measurable.
- **Generated code is still not useful.** Every continuation repeats or is malformed, float or integer, at 1,000 updates and at full exposure.
- **The stack has a dialogue path** (`dialogue-train`, `lut-chat`, `scripts/geometric-stack-chat-m1.sh`). It is checked on synthetic data only; the prepared corpus is on the owner's machine.

## 1. Why capacity, and why a parallel layout

**The retained native model is small and sequential.** *Derived* from its shape inventory (`uor_r4_integer::config::JointConfig::shapes`):
- one gated recurrent layer and one 64-wide read at width 256;
- 1,678,466 parameters, of which 625,794 are not the embedding or output bias;
- trained position by position, because each step's read depends on the state that the same step updates.

**The retained reference is four times larger.** #1017 is a Llama-shaped transformer: 6 layers at width 288, 6 heads, SwiGLU 768, tied embedding, 7,155,360 parameters (5,975,712 outside the embedding).
- On the 233,472-target comparison tail it scores **1.574024** nats per token.
- The native continuation finals score **1.975–1.996**, 0.40 nats behind, and produced 0/5 acceptable prose (*Measured*, [current state](current-state.md)).

**No experiment has separated capacity from mechanism.** The main line's campaigns held the native model's size fixed. The question this cycle asks is:
- at #1017's parameter count and on identical data, does a native geometric model close the gap to an ordinary transformer?
- or does its mechanism cost quality?

Answering it needs a native model that can be made deeper. That in turn needs training that processes a whole window at once rather than one position at a time.

## 2. The geometric stack

`crates/uor-r4-training/src/geometric_stack.rs`. A stack of pre-norm residual layers. Each layer is one temporal mixer followed by a SwiGLU MLP. `pattern` picks the mixer of each layer: `r` for the recurrence, `a` for the read.

The default `rrarra` has two recurrence layers for each read, the ratio of Griffin (*Literature*, arXiv 2402.19427). There the recurrent layer is a real gated linear recurrence and the read is local attention.

### 2.1 Quaternion transport recurrence (`r`)

Each lane of four channels carries a quaternion state:

```text
h_t = lambda_t (u_t * h_{t-1}) + sqrt(1 - lambda_t^2) c_t
```

- `*` is the Hamilton product.
- `u_t` is a unit quaternion computed from the input. The implementation normalizes `raw_t / sqrt(|raw_t|^2 + 1e-6)`, so `u_t` is unit up to that epsilon. `raw_t` is a learned linear map of the normalized input plus a bias that starts at the identity quaternion `(1, 0, 0, 0)`.
- `lambda_t = a^(8 r_t)` is a gated decay:
  - `r_t` is a sigmoid gate of the input;
  - `log a = -softplus(-decay)` is learned per lane;
  - at a fully open gate, the lanes start with timescales spread geometrically from 2 to 1,000 tokens.
- `c_t` is the input branch after a width-4 causal depthwise convolution, whose taps start at the identity.
- The drive factor is implemented as `sqrt(max(1 - lambda^2, 1e-6))`. It is Griffin's normalization: for uncorrelated drives of equal variance it keeps the state's variance at the drive's.

The layer outputs `W_out (h_t ⊙ gelu(g_t))`, where `g_t` is a second input branch.

- **Whole-window training, not a parallel scan over time.** *Derived for the first part, as implemented for the second.*
  - The gates, rotation and drive depend only on the layer's input, not on the recurrent state. So every projection of a layer runs once per window as one matrix product. In the retained learner, each position's read and update depend on the state the previous position produced.
  - Only the scan itself runs position by position: a few multiply-adds per lane and step, batched over windows and parallel across them.
  - The transition acts on `h` by left multiplication, a linear map, so the recurrence is an affine scan. Its composition law `(q2, b2) ∘ (q1, b1) = (q2 * q1, q2 * b1 + b2)` is associative. A parallel prefix scan over time is therefore possible, but this implementation does not use one.
- **Non-expanding transitions, not a bound on the states.** *Derived.*
  - A unit quaternion preserves the norm and `lambda_t < 1`. Every product of transitions is therefore non-expanding: `|q_t * ... * q_s| <= 1`.
  - This does not bound the driven state uniformly.
    - As implemented, `|u_t| = |raw_t| / sqrt(|raw_t|^2 + 1e-6)`, slightly below 1, and the drive multiplier is `kappa = sqrt(max(1 - lambda^2, 1e-6))`.
    - For constant coefficients and drives of norm at most `B`, `|h_t| <= rho |h_{t-1}| + kappa B` with `rho = lambda |u| < 1`. The asymptotic bound is therefore `kappa B / (1 - rho)`.
    - With a unit rotation and an unclamped drive this becomes `sqrt((1 + lambda) / (1 - lambda)) B`, which grows as `lambda` approaches 1.
    - It says nothing about the residual stream across layers.
  - Setting `u_t` to the identity (`rotation=false`) leaves a real gated linear recurrence with one decay per lane. That is the ablation of the transport.
- **Exact backward.** *Derived.* It is the reverse scan:
  - the total gradient of `h_t` is `G_t = dh_t + conj(q_{t+1}) * G_{t+1}`;
  - then `db_t = G_t` and `dq_t = G_t * conj(h_{t-1})`.

  Both identities follow from `L(q)^T = L(conj q)` and `R(p)^T = R(conj p)` for the left- and right-multiplication matrices.

### 2.2 Multi-head read (`a`)

Six heads of width 48. Row `t` reads positions `0..t` and a NoRead slot whose value is zero, with scores:

```text
Lorentz:  s_tj = -beta_h (arcosh(z_tj) - delta_h) + age_h(t - j),   z = sqrt(1+|q|^2) sqrt(1+|k|^2) - <q, k>
Dot:      s_tj = <q_t, k_j> / sqrt(48) + age_h(t - j)
```

- The Lorentz score is the lifted hyperboloid distance of cycles 2 and 3.
- `beta_h` and `delta_h` are learned per head, with a flat start (`beta = 1`).
- `age_h` is a learned bias per head and distance. It starts at ALiBi slopes `2^(-8(h+1)/6)`, from 0.40 to 0.0039 per token (*Literature*, arXiv 2108.12409).
- The NoRead logit is a learned linear function of the query position.
- There is no rotary position embedding. Position reaches the read through the recurrence layers and the age term.

### 2.3 The control

#1017's exact shape, trained from scratch on the same windows:
- RoPE softmax attention with 6 heads of width 48 (theta 10,000, rotate-half pairing);
- SwiGLU 768;
- RMSNorm with epsilon 1e-5;
- tied embedding.

Its attention runs on the same fused kernel as the read, with the Dot score and no NoRead or age term.

### 2.4 Parameter parity

The geometric stack's MLP width is chosen so its total matches the control's.

| Model | Pattern | MLP | Parameters |
|---|---|---:|---:|
| Transformer control (#1017 shape) | `aaaaaa` | 768 | 7,155,360 |
| Geometric stack, Lorentz, rotation | `rrarra` | 749 | 7,153,860 |

The recurrence layer has four times as many 288-wide input channels in its gates and branches as a read layer has outside its projections. The stack pays for them with a slightly narrower MLP.

## 3. Implementation and throughput

Everything is Rust on Candle 0.9.2, with the crate's `forbid(unsafe_code)` kept.

**Fused ops.** Five ops are fused, each with an exact backward. Each is parallel over rows, windows or (window, head) blocks.

| Op | What it computes |
|---|---|
| Recurrence core | Convolution, decay gate, rotation normalization, scan and output gate |
| Read | Inner products, score transform, mask, NoRead, softmax, value mix and, for the control, RoPE |
| RMSNorm | Row normalization |
| SwiGLU | `silu(gate) * up` |
| Cross-entropy | Mean next-token loss |

- **Why fuse.** The recurrence core scans time sequentially inside each window and runs windows in parallel (§2.1). Candle's CPU elementwise ops are single-threaded, and its batched matrix product runs one small matrix at a time. Fusing took single-arm throughput on 4 threads as follows.

  | Arm | Before (tokens/s) | After (tokens/s) |
  |---|---:|---:|
  | Transformer control | 770 | 1,500 |
  | Geometric stack | 727 | 1,720 |

  Two arms run concurrently at 2 threads each reach about 1,100 tokens/s each, which is how the runs below were scheduled. *Measured*, idle sandbox, batch 16 × 256.
- **Checks.** Eleven focused tests:
  - the scan against the sequential recurrence;
  - each fused op against its composition of Candle operations, forward and backward: the recurrence core with rotation on and off, the Lorentz and Dot reads, the RoPE read, RMSNorm, SwiGLU and cross-entropy;
  - finite-difference gradients through the scan, the reads and the whole model;
  - causality of every model kind;
  - parameter parity;
  - save and load.

## 4. Data

- **Repository code.**
  - Cycle 3's corpus: 7,000,992 training and 206,844 development tokens.
  - The lab BPE with 4,096 tokens, 3.64 bytes per token.
  - The development split is unchanged, so the retained native model's cycle-3 results on it stay comparable.
- **Registry code.**
  - The Rust sources of the sandbox's Cargo registry, the latest version of each of 222 crates.
  - Files up to 200 kB, 5,975 files, 72,414,505 bytes.
  - Encoded with the same BPE: 29,039,409 tokens at 2.49 bytes per token. The BPE was fit on this repository.
- **Main runs.** They draw windows from the two training streams with equal probability. Over 29,999,104 target visits that is about 2.1 passes over the repository split and 0.5 over the registry.

| File | SHA-256 |
|---|---|
| `c3/data/code/train.u16` | `b12707b012e5447f0c613236ea7f8819f2cc123861dafe49f7f8226ccc457137` |
| `c3/data/code/valid.u16` | `3f7c50ef98fa707f4523aa3f7c6a97f496ee184fcfa125a84979fed9e7b91cb8` |
| `c3/data/code/lens.u16` | `edc0a8d6bcabf8c2a789113a71ebdd4ce6a3dd53de08e77aff13c56bc7c2e61f` |
| `c3/data/code/merges.txt` | `7453bfa086a18d36948d23fd060d2a74bc3ba8d87882d457cb397396dde84405` |
| `c4/data/registry.txt` | `cc96831b8431ccb7d428d8e5d1e7cacc18ce1e951b5ee957fd58e611f5e953ef` |
| `c4/data/registry.u16` | `af93ca73e51a798e4a19e1a470e97037939c95f7499e65ffb8e3526748026f65` |

## 5. Learning-rate pilot

*Measured.* Settings:
- 1,000 updates of 16 × 256 tokens (4,096,000 target visits per run), seed 1, on the repository code split only;
- warmup 100, then cosine decay to 10% of the peak rate;
- AdamW with beta2 0.95, weight decay 0.1 on matrices, clipping at 1.0.

Both arms drew identical windows. The final evaluation covers the 512 evenly spaced development windows of `joint-read-geometry`: 131,072 targets, the same windows as cycle 3's context-256 finals.

Final NLL in nats per token, with bits per byte in parentheses:

| Learning rate | Transformer control | Geometric stack | Stack − control |
|---|---:|---:|---:|
| 1e-3 | 2.8608 (1.1503) | 2.7851 (1.1199) | −0.076 |
| 2e-3 | **2.7680** (1.1130) | 2.6790 (1.0772) | −0.089 |
| 4e-3 | 2.8339 (1.1395) | **2.6097** (1.0494) | −0.224 |

- **Selection.** The rule was fixed before any pilot run finished: each arm's rate is the one with the lowest final NLL. That gives 2e-3 for the control and 4e-3 for the stack. At the selected rates the stack leads by **0.158 nats** (0.064 bits per byte).
- **Fairness.** The control's best rate lies inside the grid. The stack's best rate is the grid's largest, so its optimum may be higher. The tuning therefore does not disadvantage the control.
- **At every rate the stack leads.** At 4e-3 the control scores worse than at 2e-3 (2.834 against 2.768), while the stack improves further. That is underperformance of the control at that rate, not a measured numerical instability.
- **Context, not a matched comparison.** The retained native learner of cycle 3 scored 3.119–3.124 with the flat Lorentz read and 3.169–3.185 with the Dot read, on the same windows and data after the same 4.1M target visits. It had width 128 and about 0.69M parameters. Both 7.2M-parameter arms here are 0.35–0.58 nats better. Size, depth, layer structure and training configuration all differ between the two, so this cycle does not attribute the gap to any one of them.
- **Samples.** Greedy continuations of three development prompts fall into repetition loops in both arms at this budget, for example a repeated `use std::path::PathBuf;` line. The report roots keep all six arms' greedy and sampled continuations.
- **Cost.** Two runs shared the 4-core sandbox, with two threads each.
  - Training alone ran at 942–1,222 tokens/s per run, or 3,353–4,350 s per run.
  - Complete elapsed time per round, including evaluation, checkpointing and sampling, was 67.5, 59.9 and 73.8 minutes: 3 h 21 min for all six runs.
  - Rounds 1 and 2 overlapped builds and tests of this branch and of #1410.
  - These are training costs only. Neither number measures native serving.
- **Identities and records.** The [run packet](../evidence/geometric-stack-cycle4-2026-09-27/README.md) holds each run's complete report: settings, curve, final metrics, samples as token ids and text, and input, executable and model SHA-256. It also holds the launchers and the chosen-rate log, with hashes of every file.
  - Executable `fcf710ad…` (commit `dca1b790`, host-tuned release build).
  - Training split `b12707b0…`, development split `3f7c50ef…`.
- **Scope.** One seed per arm and a short budget.
  - The pilot carries no uncertainty estimate for these architectures. Cycle 3's seed spread belongs to the retained learner, not to these models.
  - The differences are between complete architecture-and-training packages, each at its own selected rate.
  - They measure early training. The main comparison (§6) measures the endpoint.

## 6. Main comparison

*Measured.* Both arms trained from scratch with seed 1 on identical windows.
- Exposure: 7,324 updates of 16 × 256 targets, or 29,999,104 target visits.
- Schedule: warmup 200, then cosine decay to 10%, with weight decay 0.1 and clipping at 1.0.
- Rates: each arm's pilot-selected rate, 2e-3 for the control and 4e-3 for the stack (`rrarra`, Lorentz reads, rotation).
- Data: windows are drawn from the repository and registry streams with equal probability (§4).
- Evaluation: 64 development windows every 250 updates, and the final score on 512.
- Threads: two per arm, on the shared 4-core sandbox.

| Arm | Parameters | Final NLL | Bits per byte | Train tokens/s | Model SHA-256 |
|---|---:|---:|---:|---:|---|
| Geometric stack | 7,153,860 | **1.998113** | 0.803432 | 789.3 | `3eb1ebbb…` |
| Transformer control | 7,155,360 | 2.011149 | 0.808673 | 788.1 | `ba5e07e7…` |

- **Reading, by the card's rule** (fixed before the pilots):
  - within 0.03 nats of the control, or better → viable; next, integer serving and an owner M1 run;
  - behind by more than 0.03 → ablations;
  - more than twice as slow → kernels.

  The stack is 0.0130 nats *below* the control, at equal speed, so it is **viable**.
- **What the difference does and does not show.**
  - It is one seed per arm. At 1,000 updates, seeds of one configuration differed by up to 0.015 nats, and the two read scores traded places by ±0.021 (§7). A 0.013 difference is inside that spread: this is parity with an equal-size transformer, not a measured advantage.
  - The 64-window curve changed leader twice (below).
- **What comes next.** Under the ROADMAP (§2 consolidation, ruling 10, D11 as recorded there), "integer serving" now means the native, multiplier-free contract: the stack's export to a D11 bundle (interface I2), served by table and exact-structure kernels. The D10 engine of §8 is a frozen comparator.
- **Continuations.** The report roots keep three development prompts per arm, greedy and sampled. Both arms' greedy continuations fall into loops (`Model, Model, Model, …`; `&mut e, &mut e, …`). The sampled ones are locally plausible Rust with invented identifiers and repeated assignments. Neither is useful code.
- **Records.** The packet's `main/` directory holds:
  - each arm's final report with its claim and seal manifest, and the model configuration;
  - the training log, and the checkpoint state the final root resumed from;
  - every earlier attempt and the chain logs.

  The weights stay outside the repository, pinned by SHA-256. They are in the scratch sandbox and on the temporary transfer branch `transfer/cycle4-main-20260928`, with the last resumable checkpoints (step 7,300).

Development NLL on 64 windows during training (stack − control; negative favours the stack):

| Updates | Stack | Control | Stack − control |
|---:|---:|---:|---:|
| 250 | 4.2340 | 4.2840 | −0.0500 |
| 500 | 3.6557 | 3.6974 | −0.0417 |
| 1,000 | 3.0170 | 3.2508 | −0.2338 |
| 1,500 | 2.7933 | 2.8991 | −0.1058 |
| 2,000 | 2.6466 | 2.6892 | −0.0427 |
| 2,500 | 2.5662 | 2.5824 | −0.0162 |
| 3,000 | 2.4734 | 2.4817 | −0.0083 |
| 3,500 | 2.4273 | 2.4151 | +0.0122 |
| 4,000 | 2.3535 | 2.3440 | +0.0094 |
| 4,500 | 2.2853 | 2.2778 | +0.0075 |
| 5,000 | 2.2335 | 2.2301 | +0.0034 |
| 5,500 | 2.1822 | 2.1732 | +0.0090 |
| 6,000 | 2.1428 | 2.1509 | −0.0081 |
| 6,500 | 2.1054 | 2.1123 | −0.0069 |
| 7,000 | 2.0764 | 2.0960 | −0.0197 |
| 7,324 | 2.0764 | 2.0907 | −0.0143 |

**Provenance of this run.** The comparison ran three times before it finished; only the last counts.
- **First attempt: lost.** It launched at 11:23 UTC on 09-27 with the pinned build of `b87acd63`. It reached about 3,750 updates. A container restart at about 17:00 then rolled the sandbox back to its 08:24 state, and the run was lost with its checkpoints.
- **Second attempt: trapped.** The restart moved the sandbox from a 2.1 GHz Xeon with AVX512-VBMI to a 2.8 GHz Cascade Lake Xeon. Relaunched at 17:08 with the pinned build, which is tuned for the earlier host, the run trapped on an illegal instruction at 17:39, before its first checkpoint.
- **The counted run.**
  - `b87acd63` was rebuilt from source with `-C target-cpu=x86-64-v3` (SHA-256 `0b3b94dc…`) and smoke-tested through training, checkpointing, evaluation and sampling.
  - The comparison restarted at 17:52 and wrote a checkpoint at 250 updates.
  - Two later restarts (18:54 and 19:29) each discarded work done after the session's last idle snapshot.
  - The final roots (`*_r4`) resumed from the 250-update checkpoint at 19:29 and ran to the end without interruption. A resumed run reproduces an uninterrupted one on the same executable and host. With this build, a 6-update run of the stack's configuration and one stopped after its first update and resumed give the same model SHA-256 (`7e94e455…`).
- **Checkpoint interval.** It changed from 250 to 50 updates at the resume. The interval is not part of the run's lineage and does not affect its numerics.
- **Reproducing it elsewhere.** The matrix library chooses its blocking from the host's caches, so a rerun on another CPU follows a different floating-point path. The data is pinned by SHA-256 (§4). The registry stream was built from this sandbox's Cargo registry and cannot be rebuilt on the owner's machine.

**Queued: the reads-only Lorentz stack at the same exposure.** At 1,000 updates the reads-only Lorentz stack is the best configuration measured (§7). A third arm trains it (pattern `aaaaaa`, no rotation, learning rate 4e-3) with the main pair's executable, data, seed and settings, after the cycle-5 arms and the reads-only Dot seed (`c5/pipeline4.sh` in the lab sandbox). It reuses the main pair's control. Its reading, fixed before it runs and amended before it runs (2026-09-28):
- Under [ROADMAP](../../ROADMAP.md) ruling 8 (R4), a stack whose token mixing is all dense all-pairs reads is a transformer comparator.
- Its score is therefore recorded as a comparator reading, and the owner M1 scripts keep `rrarra` whatever it scores. This replaces the earlier rule that would have made `aaaaaa` the default at 0.03 nats below `rrarra`.
- Its distances from the `rrarra` stack and from the control are reported either way.

## 7. Ablations

*Measured*, seed 1. Each arm makes one architectural change to the geometric stack and keeps the settings of the pilot's selected stack run:
- learning rate 4e-3;
- 1,000 updates, 4,096,000 target visits, on the repository code split;
- identical training windows, because the seed is shared;
- the 512-window final evaluation (131,072 targets).

Each arm's MLP width is re-matched to the control's parameter count. The full stack is the pilot run itself (commit `dca1b790`); the ablations ran from `b87acd63`, which changes only the example's sample decoding, evaluate mode and early-stop checkpoint retention, not the model, optimizer or training loop. Every run's report, claim record, seal manifest and model configuration are in the [packet](../evidence/geometric-stack-cycle4-2026-09-27/README.md) under `pilot/` and `ablation/`, with the model hashes; the weights stay outside.

| Arm | What changes | MLP | Final NLL (bits/byte) | − full stack |
|---|---|---:|---:|---:|
| Full stack | `rrarra`, Lorentz reads, quaternion transport | 749 | 2.6097 (1.0494) | — |
| `dot` | Dot reads instead of Lorentz | 749 | 2.5889 (1.0410) | −0.021 |
| `norot` | Identity transport (a real gated linear recurrence) | 814 | 2.6688 (1.0731) | +0.059 |
| `readsonly` | Six Lorentz read layers, no recurrence, no RoPE | 764 | **2.5531** (1.0266) | −0.057 |
| Transformer control (§5) | RoPE softmax attention, 2e-3 | 768 | 2.7680 (1.1130) | +0.158 |

Development NLL on 64 windows at 250, 500, 750 and 1,000 updates:

| Arm | 250 | 500 | 750 | 1,000 |
|---|---:|---:|---:|---:|
| Full stack | 3.7626 | 3.2337 | 2.8752 | 2.6864 |
| `dot` | 3.7893 | 3.2204 | 2.8493 | 2.6640 |
| `norot` | 3.7739 | 3.2313 | 2.9008 | 2.7374 |
| `readsonly` | 3.7351 | 3.2036 | 2.8145 | 2.6302 |

**What seed 1 shows, at this budget.** Each arm is a whole configuration at equal parameter count, so each row compares configurations; none isolates one component with everything else fixed. Every report's greedy and sampled continuations remain repetitive or malformed at 1,000 updates. These are loss comparisons, not a qualitative capability.
- **Identity transport scored 0.059 nats worse.** That arm also moves the rotation parameters into a wider MLP (814 instead of 749), so the difference belongs to the pair of changes, not to the transport alone.
- **Dot reads scored 0.021 nats better than Lorentz reads inside the stack.** That is below the 0.03 threshold stated on the card, so a second seed follows before any conclusion. In the retained one-layer learner, cycle 3 measured the opposite sign, 0.05–0.06 nats in favour of Lorentz at context 256.
- **The reads-only configuration scored best.** Six read layers (MLP 764) scored 0.057 nats below the `rrarra` stack and 0.215 below the RoPE control.
  - So a stack without recurrent layers also leads the control at this budget.
  - It does not isolate what the recurrence contributes inside `rrarra`: replacing four recurrent layers also changes the MLP width and puts a read at every depth.
  - Three things distinguish these read layers from the control's attention: the learned per-distance age bias with its ALiBi start, the NoRead slot, and the score. The reads-only pair below separates the score from the other two.

**Runs added in response.** Each is justified by the rule above or by the attribution question.
- A second seed of the Lorentz and Dot stacks.
- A reads-only Dot arm (seed 1), which separates the score from the age bias and NoRead within the best configuration.
- A second seed of the reads-only Lorentz arm.

The main comparison runs after them, unchanged.

**Second seed of the Lorentz and Dot stacks.** Everything is as in seed 1 except the seed, which changes the initialization and the training windows. Both arms share it, so each seed gives a paired difference.

| Seed | Lorentz | Dot | Lorentz − Dot |
|---|---:|---:|---:|
| 1 | 2.6097 | 2.5889 | +0.0208 |
| 2 | 2.5699 | 2.5909 | −0.0210 |
| Mean | 2.5898 | 2.5899 | −0.0001 |

Development NLL on 64 windows at 250, 500, 750 and 1,000 updates, seed 2: Lorentz 3.7598, 3.2040, 2.8475, 2.6517; Dot 3.7983, 3.2227, 2.8559, 2.6694.

- The two seeds give paired differences of equal size and opposite sign. So at this budget neither score has a consistent advantage inside the stack. Two seeds do not establish equivalence. The Lorentz arm's own seed spread, 0.040 nats, is twice the difference either seed showed.
- This does not transfer cycle 3's result to the stack or refute it. Cycle 3 measured 0.05–0.06 nats in favour of Lorentz in the retained one-layer learner at context 256. One hypothesis, not tested here, is that a deeper stack with MLPs builds the geometry the Lorentz score supplies to a shallow model.
- The reads-only pair tests the score where every layer is a read.

**Reads-only pair.** Six read layers (MLP 764), no recurrence and no RoPE, otherwise as above.

| Seed | Lorentz | Dot | Lorentz − Dot |
|---|---:|---:|---:|
| 1 | 2.5531 | 2.6291 | −0.0760 |
| 2 | 2.5382 | queued | — |

Development NLL on 64 windows at 250, 500, 750 and 1,000 updates:

| Arm | 250 | 500 | 750 | 1,000 |
|---|---:|---:|---:|---:|
| Lorentz, seed 1 | 3.7351 | 3.2036 | 2.8145 | 2.6302 |
| Dot, seed 1 | 3.8586 | 3.2925 | 2.9047 | 2.7023 |
| Lorentz, seed 2 | 3.7352 | 3.1695 | 2.8032 | 2.6106 |

- **With every layer a read, the Lorentz score is 0.076 nats better than Dot**, with identical initialization and windows (seed 1). That is above the card's 0.03 threshold.
  - The lead holds at every evaluation: 0.12, 0.09, 0.09 and 0.07 nats.
  - Lorentz's second seed lands 0.015 from its first.
  - There is one Dot seed, so the Dot arm's own seed spread is not measured here. In the full stack it was 0.002.
  - **Queued: Dot, seed 2.** It runs after the cycle-5 arms (`c5/pipeline3.sh` in the lab sandbox), with the same executable, data and settings, alone with four threads. Its reading, fixed before it runs:
    - if the Lorentz mean over two seeds is at least 0.03 nats below the Dot mean, the Lorentz score is recorded as better in the reads-only stack over two seeds at this dose;
    - if the two means are within 0.03 nats of each other, no advantage is recorded;
    - if Dot is at least 0.03 nats better, that is recorded;
    - the Dot seed spread is reported beside the result either way.
- **The score's effect depends on the configuration.** It is not measurable in `rrarra` (±0.021 over two seeds) and is 0.076 when all six mixers are reads. A *Hypothesis*, not tested: the recurrence supplies ordering or recency structure that the Lorentz geometry otherwise gives the reads.
- **The reads-only lead depended on the score.** With Dot reads, reads-only (2.6291) scores worse than the `rrarra` Dot stack (2.5889, seed 1).
- **Against the control (2.7680), each arm at its pilot-selected learning rate:**
  - reads-only Dot leads by 0.139, from what separates the reads from the control's attention apart from the score: the learned age bias with its ALiBi start, the NoRead slot, and no RoPE;
  - the Lorentz score adds 0.076 on top.
  - These are configuration comparisons, with the caveats above. The reads-only arms also use the stack's pilot learning rate; their own optimum was not searched.

**Interruptions.** The container restarted twice, found at about 08:22 and 08:40 UTC, each time before the seed-2 pair reached its first checkpoint. The pair restarted from scratch in new report roots, `*_r1` and then `*_r2`; the interrupted roots are kept.
- The pipeline now resumes any run from its latest checkpoint into a new root.
- It checkpoints every 50 updates instead of 250. The cadence is outside the resume lineage and changes no computation, since the checkpoint carries the sampler state.

**Cost.**
- Elapsed from claim to seal: `dot` 56.6 min and `norot` 57.5 min, run together with two threads each; `readsonly` 46.5 min, alone with four threads. Thread count changes speed and floating-point summation order, not the model or the update rule.
- Training alone took 2,739–3,386 s per arm. The stage ran from 05:58 to 07:42 UTC.
- The two interrupted seed-2 attempts cost up to 56 minutes of two-arm time and kept no result.
- The completed seed-2 pair (`*_r2`) took 74.4 min (Lorentz) and 69.6 min (Dot) from claim to seal, with two threads each. It shared the machine with the integer-serving work of §8, and trained at 934 and 999 tokens/s.
- The reads-only pair took 72.6 min (Dot, seed 1) and 83.9 min (Lorentz, seed 2), with two threads each, at 957 and 828 tokens/s, sharing the machine with builds and benchmarks of the next cycle's code.

## 8. Integer serving under D10

*Measured*, except where marked *Derived*. Owner decision D10 permits serving without floating point: learned weight maps run without a multiplier, and runtime products may use the hardware multiplier. [`uor_r4_lut::stack`](../../crates/uor-r4-lut/src/stack.rs) serves the geometric stack that way. It uses the hardware multiplier and divider on runtime values, so it does not meet the stricter multiplier-free contract of D0-b, which remains the native model's serving target. The control has the shape of a Llama checkpoint, so it is renamed to one and served by the existing Llama engine. [`stack_export`](../../crates/uor-r4-training/src/stack_export.rs) writes both artifacts, and the example's `export`, `lut-evaluate` and `lut-sample` modes run them.

**How each part is served.**

| Part | Served as |
|---|---|
| Weight maps: embedding, projections, MLP, head | 4-bit weights in groups of 32 with scales `(16 + m) 2^(e − 4)`, read through per-activation tables by the Llama engine's vector kernels; round-to-nearest export |
| Norm gains | folded into the maps that read the normalized state |
| Convolution taps, decay rates `8 softplus(−decay)`, Lorentz `β` | one grid code each, `±(16 + m) 2^(e − 4)`, applied by shifts and adds |
| Biases, age tables, Lorentz offsets | integers, added |
| Quaternion transport, gating, scores, value mixing | products of runtime values, on the hardware multiplier |
| Sigmoid, exp, SiLU, GELU, arcosh | sealed tables with linear interpolation |
| `sqrt(1 − λ²)`, rotation norms, Lorentz lifts `sqrt(1 + |x|²)` | integer square roots |
| Softmax, rotation normalization | one division per head or quaternion |

The recurrence state is held at `2^−32` in 64-bit integers. Read keys and values are held at `2^−16` in 32-bit integers.

**Check.**
- A float stack built from the artifact's own values matches the integer engine within `5 × 10^−4` nats per log-probability. Its values are the dequantized matrices, grid-code scalars, integer biases and unit gains.
- The test logits span about 16 nats. The check covers Dot and Lorentz reads, with and without rotation, and the patterns `rarr`, `ra` and `aa`. It uses random small stacks (unit test `integer_stack_matches_its_grid_reference`).
- So the integer arithmetic adds far less error than representing the parameters does, on those small stacks.
- The trained models' split is measured below.

**Trained models.** Each seed-1 model at 1,000 updates was exported with commit `2a681bb7` and scored on the 512 final-evaluation windows (131,072 targets), with a fresh integer session per window. The float column reproduces each run's final evaluation exactly. The records are in the packet under `integer/`; the artifacts (4.5–5.0 MB each) stay outside with their hashes.

| Model | Float NLL | Integer NLL | Integer − float | Top-1 agreement | Engine tokens/s, one thread |
|---|---:|---:|---:|---:|---:|
| Full stack (Lorentz) | 2.6097 | 2.6218 | +0.0121 | 92.7% | 801 |
| `dot` | 2.5889 | 2.5997 | +0.0108 | 93.0% | 822 |
| `norot` | 2.6688 | 2.6802 | +0.0114 | 93.1% | 862 |
| `readsonly` | 2.5531 | 2.5659 | +0.0128 | 93.3% | 677 |
| Transformer control | 2.7680 | 2.7757 | +0.0077 | 93.6% | 958 |

- Integer serving costs the stacks 0.011–0.013 nats and the control 0.008. The full stack's lead over the control is 0.154 nats in integers, against 0.158 in float. The ablations keep their float order.
- **The gap is in the exported parameters, not the integer arithmetic.** The same grid reference, built from each trained artifact with its own head, was scored on the same 512 windows (commit `feeef7e4`, `lut-evaluate reference=true`). The reference replaces every float parameter with its exported value: the 4-bit matrices, grid-code scalars, integer biases, folded unit gains and the head. It then computes in float:

  | Model | Float | Reference: exported parameters, float arithmetic | Integer | Parameter representation | Integer arithmetic |
  |---|---:|---:|---:|---:|---:|
  | Full stack | 2.609729 | 2.621839 | 2.621838 | +0.012109 | −0.000000 |
  | `dot` | 2.588881 | 2.599691 | 2.599690 | +0.010810 | −0.000001 |
  | `norot` | 2.668833 | 2.680205 | 2.680204 | +0.011372 | −0.000002 |
  | `readsonly` | 2.553095 | 2.565851 | 2.565851 | +0.012756 | +0.000000 |
  | Control | 2.768041 | 2.775585 | 2.775736 | +0.007543 | +0.000152 |

  - The integer arithmetic adds at most 0.000152 nats per target: the control's, whose engine keeps an 8-bit key/value cache. It adds none measurable for the stacks, which keep 32-bit keys and values.
  - All of the rest comes from representing the parameters. The 4-bit matrices are the largest part, but not the only one. The split among matrices, scalars, biases and gains was not measured. GPTQ changes only the matrices, and removes 40.5–54.3% of the gap (below).
- **Calibrated rounding of the matrices removes 40.5–54.3% of the gap.** Commit `6bc7dd6d` gives the stack exporter GPTQ (Frantar et al. 2022, arXiv 2210.17323), which the Llama exporter already had. GPTQ rounds each matrix's columns in order and spreads each rounding error over the columns not yet rounded, so the rounded map reproduces the float map on calibration inputs rather than the float weights.
  - **Calibration.** The float model runs over 64 evenly spaced 256-token windows of its own training split (16,384 positions). A capture in the forward pass collects the input moments of every weight map. Normalized states are taken before their gains, which the export folds into the maps. The embedding, a lookup, still rounds to nearest.
  - **The control** is calibrated by the same capture, with its moments mapped to the Llama exporter's sites. A unit test checks that they equal those of that exporter's own float model within `10^−4` at every site.
  - **Cost.** Collecting the moments takes 9–16 s of float forward passes, and the whole export 19–28 s, on one thread.
  - **Results.** The same 512 windows, with the grid reference (damping 0.01):

  | Model | Float | Integer, round-to-nearest | Integer, GPTQ | Parameter representation, GPTQ | Integer arithmetic, GPTQ | Top-1 agreement, round-to-nearest → GPTQ |
  |---|---:|---:|---:|---:|---:|---:|
  | Full stack | 2.609729 | 2.621838 (+0.012109) | 2.616711 (+0.006982) | +0.006982 | −0.000000 | 92.7% → 94.7% |
  | `dot` | 2.588881 | 2.599690 (+0.010810) | 2.595309 (+0.006428) | +0.006428 | −0.000000 | 93.0% → 94.6% |
  | `norot` | 2.668833 | 2.680204 (+0.011372) | 2.674789 (+0.005955) | +0.005956 | −0.000001 | 93.1% → 94.8% |
  | `readsonly` | 2.553095 | 2.565851 (+0.012756) | 2.560338 (+0.007243) | +0.007243 | +0.000000 | 93.3% → 94.9% |
  | Control | 2.768041 | 2.775736 (+0.007695) | 2.771562 (+0.003520) | +0.003331 | +0.000189 | 93.6% → 95.5% |

  - GPTQ removes 40.5–47.6% of each stack's integer gap and 54.3% of the control's. The stacks now lose 0.0060–0.0072 nats in integers, and the control 0.0035.
  - On the calibration inputs, the mean relative output error of the calibrated matrices falls from 0.045–0.054 to 0.027–0.037. No matrix got worse.
  - The integer arithmetic is unchanged: nothing measurable for the stacks, and 0.00019 for the control.
  - The full stack's lead over the control in integers is 0.155 nats with GPTQ, against 0.158 in float.
  - The records are in the packet under `integer/<model>/gptq-export/` and `gptq-evaluation/`. The artifacts stay outside with their hashes. These runs shared the machine with the main comparison and with builds, so their engine rates are not reported.
- The integer engine's own continuations are in the packet under `integer/<model>/samples/`: three development prompts, greedy and sampled, with commit `cb60c8bd`. Like the float ones, the greedy ones repeat and the sampled ones are locally plausible but malformed.
- The engine rates time the engine's steps alone, on 64 of the windows, with commit `cb60c8bd`. The 512-window runs timed each window's whole loop, including scoring every position's 4,096 logits in f64, and ran 2–3% slower.
- The tokens per second are not an architecture comparison:
  - they were measured on one thread while two training runs shared the 4-core sandbox;
  - the stack engine's reads use scalar loops over 32-bit keys and values, while the Llama engine uses vector kernels over 8-bit ones.

**Per-token work (*Derived* from the shapes and artifact headers).**
- Weights: every model reads about 7.2 million packed weights per token (3.8 MB with scales). The float model's fp32 weights are 28.6 MB.
  - The stack reads 1.2% more than the control, because its MLP is padded from 749 to 768.
- Cache: at position `t`, each read layer reads `t` keys and `t` values of width 288, while each recurrence layer reads its fixed state of 288. At the end of the context (`t = 256`):
  - the control's six attention layers read 884,736 cached values per token;
  - `readsonly`'s six read layers read the same;
  - the `rrarra` stack's two read layers read 294,912, plus four states of 288.

## 9. What this changes

- **For the native architecture.** At equal parameters and 30M target visits on code, a recurrence-primary geometric stack matches a transformer of the #1017 shape: −0.013 nats at one seed, at equal speed. The stack has quaternion-transport recurrences and two Lorentz reads.
  - Under the ROADMAP's §2 consolidation rule it becomes the main-line candidate.
  - Its next step is the export of this checkpoint to the native serving contract (D11; interfaces I1 and I2 of the director plan), not more training.
  - It does not establish prose or chat quality, an advantage beyond one seed, or any energy result.
- **For the read's geometry.** At this scale the Lorentz score's benefit depends on the configuration.
  - It is not measurable in `rrarra`.
  - It is 0.076 nats at one seed when every layer is a read. Such a stack is a transformer comparator under ruling 8.
  - The reads-only Dot seed 2, cycle 5's memory arms and the reads-only full-exposure arm are held until this reading is taken up (director plan, 2026-09-28).
- **For serving.** Under D10 the stack is served with no floating-point arithmetic and no multiplier on learned weights. GPTQ halves the cost of representing its parameters.
  - The engine is dense and reads every weight per token.
  - It uses the hardware multiplier and divider on runtime values. A disassembly census of its ARM64 build (Lab 3, on #1437) counts 2 `udiv` and 24 `fmov` register transfers on the step path.
  - D10 is superseded (ROADMAP ruling 10; D11), so this engine is now a frozen comparator. The stack's mission serving moves to table and exact-structure kernels (T3), fed by the stack export.
- **For chat.** The stack can learn the prepared dialogue corpus under the retained study's episodes, and talk through the integer engine.
  - Replies follow the study's stop rules and panel limits.
  - This path is checked on synthetic data only.

## 10. Costs

*Measured*, lab sandbox (4 cores, 15 GB), charged cumulatively; no paid or external compute. Elapsed times are wall clock from the run records.

| Stage | Elapsed | Notes |
|---|---:|---|
| Pilot, three rounds | 3 h 21 min | two arms at a time, two threads each |
| Seed-1 ablations | 1 h 44 min | `dot` and `norot` together, then `readsonly` |
| Seed-2 pair | 2 h 17 min | including about 56 min lost to two container restarts before the first checkpoint |
| Reads-only pair | 1 h 24 min | two arms at a time |
| Main comparison, counted run | 10 h 41 min | two arms, two threads each: 25 min to the 250-update checkpoint, then 10 h 16 min from the resume; 38,007 and 38,064 s of training per arm |
| Main comparison, lost attempts | about 7 h 20 min | the first attempt (11:23 to the ~17:00 rollback), the host-tuned build's 31 min before its trap, and work after two idle snapshots (about 37 and 32 min) |
| Rollback recovery | not metered separately | branch replay from the session record, rebuilds for the new host, smoke and resume checks |
| Integer stages | about 50 min of one-thread engine time | evaluations 14.1 min, references 14.9, GPTQ exports 1.9, GPTQ evaluations 17.2, step timing 1.7, samples 6 s; round-to-nearest exports untimed; run beside training |
| Implementation, tests and builds | not metered separately | |

- **Storage.** The packet in the repository is 2.9 MB (205 files). Models, checkpoints and executables stay outside the repository, pinned by SHA-256 in the packet; the main pair's final models and last checkpoints are on the temporary transfer branch for the owner's copy.
