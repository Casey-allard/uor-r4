# D1: the quaternion transport in the `rrarra` core, at matched MLP width

2026-09-28 · Claude lab track (cloud) · References #820, #973 · Pre-registered in the [whole-project synthesis §4](whole-project-synthesis-2026-09-28.md#4-decisive-measurements-pre-registered) · Work card: [#973 comment](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5864504193)

**Status.** An evidence note from the lab track, not a decision record. Labels: **Measured** (Rust runs in the lab sandbox, seeds stated) and **Derived**. The results are development measurements on code, at 1,000 updates and two seeds. Nothing here is a holdout or a language qualification. Run records: [D1 packet](../evidence/transport-attribution-d1-2026-09-28/README.md).

## 0. Result

*Measured*, development NLL on the 512 evenly spaced windows (131,072 targets):
- **Quaternion transport beats identity transport by 0.0711 nats (seed 1) and 0.0774 nats (seed 2).**
- The pre-registered rule keeps the transport if it wins by at least 0.02 in both seeds. So **the quaternion transport stays in the main-line core.**
- **Costs of the transport:**
  - It carries 332,928 more parameters (7,153,860 against 6,820,932).
  - It trained 10% and 8% slower here.
- **The earlier `norot` arm understated the effect.** It moved the rotation's parameters into a wider MLP and measured +0.059 nats (cycle 4 §7; one seed, another host and executable). At matched width, the gap is about 0.07–0.08 in both seeds.

## 1. Question and design

**Question.** D1 asks whether the geometric transport in the stack's recurrent layers is load-bearing. In the `rrarra` core, each `r` layer carries its state in quaternion lanes.
- Each lane's state is updated as `h_t = q_t ⊗ h_(t−1) + b_t`, with `q_t = λ_t u_t`.
  - `u_t` is the unit quaternion from the gate map's rotation rows, normalised.
  - `λ_t = exp(c · σ(gate) · log a)` is a token-gated decay on a learned per-lane rate `a`.
  - `b_t` is the convolved drive.
- With `rotation=false`, `u_t = 1`. That leaves a real gated linear recurrence.

**The confound in cycle 4.** Cycle 4's `norot` arm matched parameter counts, so it moved the rotation's gate rows into a wider MLP (814 instead of 749). That confounded transport with MLP width.

**Design.** D1 pins both arms at MLP 749 with `stack_mlp=`, so they differ only in the transport:
- the identity arm has 332,928 fewer parameters: 288 gate rows of 288, plus 288 biases, in each of the four `r` layers;
- the pre-registered design accepts that difference.

**Decision rule, fixed before any run** (synthesis §4, work card):
- quaternion transport at least 0.02 nats better than identity in **both** seeds → keep;
- any other outcome → drop rotation.

**Fixed conditions.**
- **Executable:** one build of `4eef03e6` for x86-64-v3 (SHA-256 `c90c7346…`), on the sandbox's Cascade Lake host.
- **Data:** the cycle-4 repository code split and lab BPE, pinned by SHA-256 in the packet.
- **Model:** `rrarra`, Lorentz reads, width 288, 6 heads, context 256.
- **Training:**
  - learning rate 4e-3, with warmup 100 and cosine decay to 10%;
  - weight decay 0.1, clip 1.0, batch 16;
  - 1,000 updates (4,096,000 target visits).
- **Evaluation:** 64 development windows every 250 updates, and the final score on 512.
- **Seeds:** 1 and 2.
  - The arms of a seed see identical windows.
  - Their initial values agree only up to the first recurrence's rotation rows. Beyond that point, the initializer stream is shifted by the missing rows.
- **Threads:** two per arm, one seed's pair at a time.

## 2. Results

*Measured.*

| Seed | Arm | Parameters | Final NLL | Bits per byte | Train tokens/s | Train s | Model SHA-256 |
|---:|---|---:|---:|---:|---:|---:|---|
| 1 | Quaternion transport | 7,153,860 | 2.599845 | 1.045385 | 709.8 | 5,770 | `89889392…` |
| 1 | Identity transport | 6,820,932 | 2.670985 | 1.073990 | 790.1 | 5,184 | `879da99a…` |
| 2 | Quaternion transport | 7,153,860 | 2.580265 | 1.037512 | 675.5 | 6,064 | `deb422e4…` |
| 2 | Identity transport | 6,820,932 | 2.657687 | 1.068643 | 730.2 | 5,609 | `59d5d3c3…` |

Identity minus quaternion: **+0.071140** (seed 1) and **+0.077422** (seed 2).

Development NLL on 64 windows during training (Δ = identity − quaternion; positive favours the quaternion transport):

| Updates | Seed 1 quaternion | Seed 1 identity | Δ | Seed 2 quaternion | Seed 2 identity | Δ |
|---:|---:|---:|---:|---:|---:|---:|
| 250 | 3.7463 | 3.7850 | +0.0387 | 3.7729 | 3.8008 | +0.0279 |
| 500 | 3.2187 | 3.2479 | +0.0292 | 3.2156 | 3.2292 | +0.0136 |
| 750 | 2.8609 | 2.9073 | +0.0465 | 2.8492 | 2.9096 | +0.0603 |
| 1,000 | 2.6820 | 2.7379 | +0.0558 | 2.6513 | 2.7368 | +0.0856 |

**What the curves show.**
- The quaternion arm leads at every evaluation in both seeds.
- The lead grows over training, from 0.01–0.04 nats early to about 0.06–0.09 at 1,000 updates.
- The report roots also keep three development continuations per arm. As in cycle 4, the greedy continuations of every arm loop (`use std::io::{Deserialize, Serialize};` repeated; `&mut c, &mut c, …`). The sampled ones are locally plausible Rust with invented identifiers. None is useful code at this budget.

## 3. Reading

**By the rule, keep the transport.** It wins by 0.071 and 0.077, each more than three times the threshold, and in both seeds.

**What this does not show:**
- whether the transport helps at full exposure, on prose or dialogue, or at 10–30M parameters;
- which property does the work: non-commutativity, the unit norm, or merely the extra gate rows. A parameter-matched real-gate control with the same extra rows would separate the last of these.

*Derived:* **why a value map cannot absorb this transport.** Ruling 11's factoring is about a different kind of transport.
- The recurrence unrolls to `h_t = Σ_s Λ_(t,s) (u_t⋯u_(s+1)) b_s`, where `Λ_(t,s)` is the product of the scalar decays in between. The decays commute with the quaternions.
- With the prefix product `U_t = u_t⋯u_1`, the ordered product is `U_t U_s⁻¹`. So `h_t = U_t · Σ_s Λ_(t,s) U_s⁻¹ b_s`.
- Each frame `U_s` depends on the whole prefix, not on token `s` alone. No per-token value map can absorb it.
- Ruling 11's factoring covers transports that depend only on each endpoint's own token.

## 4. What this changes

- **The main-line core keeps its quaternion transport.** The export to the native bundle (I2) must therefore serve, per recurrent layer and token:
  - the gate map's rotation rows;
  - a unit-quaternion normalisation;
  - one quaternion product per lane.
- **Serving under R2 (*Derived*).**
  - The quaternion products are products of runtime values, so they need product or quarter-square tables, or exact structure.
  - One option is to snap `u_t` to the 120 unit icosians. That turns each product into a Cayley-table read. Its loss is **untested**.
  - The normalisation needs a table-based inverse square root.
- **Training cost (*Measured*).** The transport cost 10% and 8% of training throughput here, which a 10–30M core would also pay.

## 5. Costs

*Measured.*
- **Wall time:** 06:14:54 to 09:36:06 UTC on 09-28, or 3 h 21 min for the four arms.
  - The seed-1 pair took 98 min, and the seed-2 pair 103 min, with two threads per arm on the 4-core sandbox.
  - A harness restart around 08:25 interrupted no run.
- **Training alone:** 5,184–6,064 s per arm.
- **Build, smoke run and runner:** about 20 minutes before the launch.
- **Storage:**
  - retained models are 27.3–28.6 MB each (107 MB of run roots in the sandbox);
  - the packet in the repository is about 0.2 MB;
  - checkpoints were deleted at completion.
- No external or paid compute.

## 6. Records

The [packet](../evidence/transport-attribution-d1-2026-09-28/README.md) holds:
- each arm's final report root: claim record, report, seal manifest and model configuration;
- the training logs;
- the runner (`sources/run-d1.sh`) and chain log;
- `packet.json`, with the runs, curves, identities, the pre-registered decision and the SHA-256 of every file.

The weights stay in the lab sandbox, pinned by SHA-256.
