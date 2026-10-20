# Whole-repository review and direction reassessment — 2026-09-23

**Status of this document.** A one-off, evidence-scoped review commissioned by the owner, not a milestone result.
It changes no artifact, runs no model, and asserts no new capability. It supersedes dated "next" statements only
where it says so. Base `d5c83bf9`; author: run lead (Sisyphus), integrating eight independent specialist reviews.

---

## 0. Verdict (read this first)

1. **The serving-arithmetic goal is sound and provable; the representational bet is not supported.**
   "Runtime performs no floating-point matrix multiplication and no transformer" is achievable — the permitted
   operator set {bounded-integer add, subtract, shift, compare/select, indexed table read} provably suffices for
   any finite-state transducer, any fixed-precision ReLU/linear-threshold network, and any bounded-state
   fixed-precision recurrent network. But **no geometric mechanism in this repository has ever beaten an
   information- and compute-matched ordinary control** on any task, and the single best language artifact contains
   **no geometry at all**.

2. **The project is not stuck on geometry — it is stuck on capacity, context and objective.** The current best
   artifact is a 64-dimensional ternary recurrence with a linear readout, trained on 64-token *independent*
   windows over one repository's Markdown, for ~1.3–2.7 epochs. The measured bind is `h_dim=64` / rank-limited
   readout / no cross-window state / no input-dependent gate — not the absence of a float multiplier, and not the
   absence of H4.

3. **The current objective is a proxy trap.** Beating a tuned two-token count table on the project's own docs is
   not a language milestone. The count prior is memoryless, order-2, ~16.7M entries and directly servable; the
   right bar is a capability the previous two tokens *cannot* supply (copy-at-distance, induction, syntax,
   long-range dependency), where the count control is at chance by construction.

4. **Recommendation: make geometry a *conditional promotion* gated on a matched comparator — not an assumed
   advantage — and move the modelling bet to capacity + context + exposure + objective.** This is an honest
   **change of emphasis, not of goal**: the owner's terminal objective (fully transformerless; geometry replaces
   float matmul) stands, but the record does not support the assumption that geometry *already* confers predictive
   advantage. Keep prime/zeta/R4/H4/`Z[phi]` as **preferred candidates for exactly the witnessed failures** — the
   input-dependent gate (transport), cross-window/structural state (retained fiber), and a structured,
   LUT-friendly, exactly-closing low-bit parameterisation (quaternion/H4 gives a real 4× parameter reduction) —
   promoted only when they beat the matched ordinary fix on the same probe at matched capacity and cost. Build
   first: an input-dependent gate, cross-window state, a bilinear/rank-K or exact LUT readout, a real LUT matmul
   kernel, and a long-range evaluation. Then scale. (The K3 review found "demote" too strong and understating the
   goal change; see §4.6.)

5. **"World models" does not describe this project.** The term means (A) a latent, action-conditioned dynamics
   model used for planning (Ha–Schmidhuber; DreamerV3, *Nature* 2025) or (B) an observation-level generative
   simulator (Sora; Genie). UOR-R4 is neither; only the trivial sense ("an autoregressive latent-state sequence
   model") applies, and that buys nothing. **Do not use the term in claims.**

6. **Two headline numbers are not what they appear.** TinyStories **1.2372 BPB** is the *continuous training-time*
   probability model (shipped artifact **1.8055 / 1.7989** BPB); the **617 tok/s vs 36.1 tok/s** comparison is
   **not quality-matched** and whole-task energy is **UNAVAILABLE**.

---

## 1. Method and evidence base

Eight independent specialist reviews, each read-only, plus the lead's own reading of the authority chain:

| Stream | Agent | Scope |
|---|---|---|
| History | `uor-history-curator` | every mechanism, positive, negative, retraction; live GitHub issues/PRs |
| Runtime mechanics | `explore` | exact served computation; what is integer vs float; where geometry is causal |
| Available mechanisms | `uor-source-falsifier` | inventory + matrix-free classification + adversarial ordinary controls |
| Roadmap falsification | `uor-source-falsifier` | rank claim, proxy objective, training-signal cap, cost |
| External prior art | `uor-literature-scout` | world models, multiply-free serving, non-transformer LMs, quaternion/GATr, scaling |
| Mathematics | `uor-mathematics-professor` | can geometry replace matmul; expressivity; minimal operator set |
| Computer science | `uor-cs-professor` | binding constraints; objective; decisive experiment; architecture |
| Systems/cost | `uor-systems-cost` | is table-native cheaper on M1; sizing; minimum credible measurement |

Evidence classes used below: **measured** (a recorded run), **theorem/definition**, **derived** (follows from
source or measurement), **external** (a cited paper/repo), **judgement**.

---

## 2. What the project actually is

### 2.1 Current runtime mechanics (the seam that predicts text)

Artifact `model.tlx` (`learner/transferable_lexical.rs`, sha `03f83eb8…b250`), **466,711 bytes**:

- `h_dim=64`, `vocab=4096`, `f_dim=15`, `TL_EVENTS=4`; embedding `(vocab+1)×64` signed-4-bit; maps `wi 64×79`,
  `wh 64×64`, `wf 64×83`, readout `wo 4098×147` ternary packed with per-row power-of-two shift.
- Recurrence: `h_{t+1} = clamp( emb(x_t) + (wh·h_t >> 3) + wf·[m;f;event] + b )`. **Input-independent** transition,
  **no gate**, a `>>3` per-step contraction (≈⅛), hard-saturation clamp as the only nonlinearity.
- Readout: `logits = wo·[h; m; f; event] + b` (147 inputs), integer argmax at serving.
- **All served arithmetic is integer add/subtract/shift/compare/table-read.** No float, no softmax at inference.
  This satisfies the "no multiplier in the declared kernel" contract — but `forward_i32` reads **every** weight
  (611,814 packed-coefficient inspections per step vs 56,457 nonzero), so it is *dense* access, not the sparse
  table read the goal implies; and it is ~98× above its own memory-bandwidth ceiling.

### 2.2 Where geometry is, and is not, causal

- **In the current best artifact: nowhere.** `transferable_lexical.rs` has zero references to prime, zeta, R4,
  S3, H4, `Z[phi]` or chirality. It is a plain integer RNN. The 15-coordinate "typed block" is symbolic flags.
- **In the `.rgm`/JEPA serving path**: H4 roots, a Hopf fiber projection and a VSA codebook enter as **bounded
  additive integer score terms** — a feature/readout layer on top of an ordinary scored-table model, not a
  transport or state-update mechanism.
- **Genuinely causal geometry** exists only in dormant modules (`learner/geometric_attention.rs`'s 2I conjugacy
  kernel; `prime_route_attention.rs`; `group_table.rs`; `hopf_metric.rs` Q30) — none of which is the current
  served artifact. `prior_learning.rs` stores a 2I element table that its predictor **never reads**.
- **Net:** in the model that actually predicts language, geometry is **absent**; where it is present it is a
  bounded feature, not the computation.

### 2.3 The artifact is small, honest about density, and does not reach useful quality

`~6.68` bits/target on the served conditioning (best arm), vs a tuned `(prev,cur)` count prior at **5.1217** and
donor E at 6.8701. Greedy generation collapses. Whole-task energy `UNAVAILABLE`.

---

## 3. What the history shows

### 3.1 Mechanism families — strongest outcome at its exact scope

| Family | Best positive (scope) | Strongest negative (scope) | Status |
|---|---|---|---|
| Exact addressed memory | #1346 16/16, own capture survives replacement | geometric contribution **none** in that serving path | retained (infrastructure) |
| Ordered occurrence readers | 411/443 constructed selection | raw text 5/88; CE +2.79 synthetic | partial, not promoted |
| Relational/H4/Q8 transport | H4 −0.524968/candidate (#1325) | H4 ties categorical (#1344: 0/5,880); #1339 table 288/288 vs H4 186/288; #1321 text +1.2908 | conditional; **no advantage** |
| Signed/typed computation | 672/672 Q8 action; consumed Q8 order witness | ordinary finite table ties | retained component |
| Hopf / S3 fiber | representation with pole convention (tolerances) | no measured language benefit; Q30 uses int mul/div | conditional, NOT_RUN for language |
| S7 / E8 / icosian | none measured | "geometry does not supply free storage or advantage" | hypothesis, NOT_RUN |
| Harmonics / resonance | class filter 0.13→0.23 corrupted | learned filter reverted; capacity immaterial; sieve parked | partial/parked |
| Spin / chirality | exact sign/inverse bookkeeping | no semantic axis measured | conditional |
| Geometric attention / gated-delta | HELM-D-R4 parity + generation PASS | **transformer-compatible `f32`/matmul**; V3 H4 3/12 vs plain 12/12; V4 13/24 | reference, **not the target** |
| TLA/R4G1 frozen kernel | multiply-free by contract | not language evidence; D0-b not whole-path | infrastructure |
| Low-bit/ternary | integer == float surrogate in refit | dense `relu` recurrence saturates; D0-a census wrong | retained |
| Prime/zeta addressing | exact identity, determinism | `%120`/prime label is **not** semantic distance; element table unused | infrastructure |
| JEPA / TinyStories `.rgm` | 1.2372 BPB continuous; 1.7989 stripped | shipped 1.8055; comparator mislabelled; cold routes 85–94% | separate, non-transferable |
| Cold/real-text prior | 9.13→7.14 at 512 steps | #1294 fails all gates; order-2 count on same input **3.9691** | partial, not language |
| **`transferable_lexical` shared learner** | full reload replay; 6.6841 best arm | prose 7.3722 vs count 5.1217; greedy collapse | **active, unqualified** |

### 3.2 The five pivots and the stuck point

1. Exact-memory primitives (≤ Sept 12) → 2. learned typed/dependent reads (Sept 13–17, with authored-syntax role
   repair) → 3. **TinyStories `.rgm`** (Sept 18–19) → 4. **native addressed-memory core** after the takeover
   (Sept 19) → 5. **one shared Generate/Copy/Stop lexical learner** (Sept 21–23).

**Stuck point:** a 64-dim ternary recurrence with a linear readout on a 466 KB artifact cannot beat a tuned
two-token count table, and its generation is degenerate. The project's own diagnosis ("interface") is
**under-determined** (see §4.1, §4.4).

### 3.3 Retractions and instrument defects (the record is honest but churny)

Notable withdrawals: the takeover review's "ternary recurrence is incompatible with stable memory" and
"sequential training" claims; the "learned-associative readout" capacity diagnosis; `#1296` parity/Stage-1;
`#1294` nats-labelled-as-bits; `#1301` permutation + document-bootstrap defects; `#1328` contextual-representation
diagnosis; `#1335` readout-capacity-only; `#1336` expressivity-only; `#1337` linear-impossibility;
`#1338` "free parameters"; the Sept 23 readout-refit first draft ("integer realisation is the limit") and the
degenerate re-pricing figures; the `olx-channel-1` interval sign; and **the state-probe's "the state does not
retain `prev`"** (corrected, see below).

### 3.4 Authored fixtures mistaken for capability (flagged in the record itself)

`2,304/2,304` role repair is rule-assisted; `#1342/#1343/#1344` are fixed-layout authored panels with static cue
signatures; `#1349` Copy/Insert/Stop targets **misuse tense**; `#1351` and `#1352` fit all computed contrasts and
hold out only familiar non-computed forms; `#1292/#1300` "positives" are count-dominated. These are real
engineering results at their scope — they are not transferable language.

### 3.5 Carried-forward contradiction

The merged state probe reports `prev` recoverable at **2.1%** (raw embedding). Two **unmerged** post-merge
branches report **70.96%** (`count-state-bridge`) and **75.90%** (`observer-transport`) on **byte-identical
states** (SHA `c5334071…992f` matches the sealed probe receipt). The two unmerged branches disagree on the
number; neither is reproduced on `main`. The merged claim has already been corrected in-record to "withdrawn as a
general statement". **This is the single most important open measurement** (§8, Step 3).

---

## 4. Independent expert rulings

### 4.1 Mathematics — "can geometry replace float matmul?"

- **Serving-arithmetic goal: sound (theorem-backed).** {bounded add/sub/shift/compare/table-read} emulates any
  fixed-precision ReLU/linear-threshold network (affine = shift-add, ReLU = compare), any finite-state transducer
  (table read), and any bounded-state fixed-precision RNN (finite reachable set). Constant-integer multiplication
  is a shift-add chain; a bounded `a×b`-bit product is a shift-add network.
- **The "rank ≤ 65" claim is imprecise.** The readout input is `D = 2·h_dim + f + events = 147`, so the correct
  bound is `rank ≤ 147` (affine ≤ 148); `≤65` holds only if the readout read `[h;1]` alone. The phrase conflates
  **state dimension (64)**, **readout input width (147)** and **effective rank**.
- **The residual is a linearity/higher-order-feature fact, not a dimension fact.** The `(prev,cur)` count prior is
  a `V×V` table of generic rank `V=4096`; a rank-≤147 *linear* map — integer or float — cannot equal it. A
  **nonlinear/bilinear** readout (product features) can, in principle, for whatever the state retains.
- **Quaternion/H4/spin:** the only *theorem-level* benefit is a **4× parameter reduction / equivariance** (a
  quaternion block stores 4 reals per 4×4 vs 16); H4/2I is itself an exact 120-element table, not an escape from
  the table class. No matched test shows a language advantage.
- **Decisive test:** on one frozen state at equal work: (A) integer linear readout; (B) equal-rank **float**
  linear readout; (C) integer readout **+ explicit bilinear/count feature**. If A≈B but both ≫ C, the deficit is
  **features**, not float-vs-integer, and geometry is not implicated.

### 4.2 Computer science — architecture and path

- **Ranked binding constraints:** (1) **no cross-window state** (64-token independent windows); (2) **no
  input-dependent gate** (the `>>3` contraction gives a ~2–3-token memory horizon — the mechanism behind
  `cur` 96.7% / `prev` 2.1%); (3) **affine rank-limited readout** (the measured crux of the 0.73-bit gap);
  (4) **multiply-free as implemented (no LUT)** — not the current quality bottleneck, but binding on *every* fix;
  (5) 64-dim state; (6) exposure; (7) objective; (8) corpus; (9) 4k vocab.
- **Objective is wrong:** `C` is memoryless and servable; beating it by 0.04 bits on the head while losing 0.55 on
  the tail is not language. The right objective is a capability the previous two tokens cannot supply.
- **Decisive experiment:** freeze the recurrence, fit on the already-recorded states (i) the linear float readout
  (known 5.5136), (ii) a rank-K bilinear/quadratic float readout, (iii) a 2-layer MLP float readout; compare to `C`
  4.7846. If (iii)−(i) < ~0.10 bits → **state-limited** (enlarge state/recurrence); if ≥~0.35 → **readout-limited**
  (build a served factorised/LUT readout). One diagnostic, no retraining.
- **Architecture:** hybrid SSM/linear-attention with low-bit weights + **LUT kernels**, plus a small number of real
  attention layers (external evidence: ~7% attention restores copying/ICL that pure SSMs lack), plus the project's
  exact addressed memory + router. Build the input-dependent gate and cross-window state first.

### 4.3 Systems/cost — is table-native actually cheaper?

- **No measured evidence, and little expected.** Low-bit serving is **memory-bandwidth-bound**, not
  multiply-bound. At M1 ≈68 GB/s, one pass over 466,711 B is **6.8 µs** (~146k tok/s); measured **673.6 µs/token**
  is ~98× that. The constraint is not what makes the path fast or slow — the kernel is.
- **The constraint is not blocking quality.** The integer realisation **equals** the float surrogate (deviation 0
  over 262,272 logits), and the in-class refit moved only 0.1313 of 0.4634 bits. Even a dense float readout of the
  same state (5.5136) still trails `C` (5.1217).
- **Minimum credible measurement:** pinned artifact+tokenizer; sealed held-out prompts; decode-only ≥128 tokens,
  ≥3 repeats, median; **segments separated** (tokenize / source-select / rollout / decode / session); tokens/s;
  **J/token from measured package power** (`powermetrics`, else labelled ESTIMATE); peak+steady RSS with the
  model-resident split; instruction mix via `xctrace` else the static symbol check. Baseline: llama.cpp CPU
  Qwen2.5-1.5B Q4_K_M and **bitnet.cpp / T-MAC BitNet-b1.58-3B** on the same machine. Because quality is unequal,
  the honest form now is a **curve** (bits/target vs tok/s vs J/token vs RSS).
- **Verdict:** keep the multiply-free kernel as a **declared contract**, stop treating it as the sizing rationale;
  optimize elsewhere. Relax only if a whole-path measurement shows ≥2× slower or ≥20% worse J/token **and** ≥0.10
  bits/target recovered — the **0.10-bit trigger**.

### 4.4 Falsification of the current roadmap diagnosis

- **Rank claim: partly falsified.** Correct for the prose (target × position) matrix, but not a bound on language
  competence, and it lends false inevitability to the "interface" conclusion.
- **Count-prior objective: partly a red herring.** It was the right cheap control for the "build a table read?"
  decision (answer: no), but the wrong objective for progress.
- **Training-signal cap:** with one repository's Markdown, 64-token independent windows, a 4k vocab and <3 epochs,
  **no architecture yields general prose** — the cap is data diversity/scale and context independence. The
  joint-fit's "budget is not the constraint" is scoped far too broadly.
- **Geometry vs recurrence:** geometry has **never** been shown to contribute beyond the recurrence at matched
  information and cost; the best artifact has none.
- **Most decision-changing cheap test:** a **capacity/precision ablation** — retrain at `h_dim` 128/256 with a
  matched dense-float arm, everything else identical. If quality improves materially, the "rank-64 interface" is
  falsified and the fix is capacity, not transport geometry.

### 4.5 External prior art

- **World models:** latent predictive dynamics for planning (Ha–Schmidhuber 2018; DreamerV3, *Nature* 2025) or
  observation-level generative simulators (Sora; Genie). UOR-R4 is neither (§6).
- **Multiply-free:** a recognized axis (T-MAC, bitnet.cpp, LUT-GEMM, BitNet b1.58, MatMul-free LM) — but **every
  credible system keeps softmax attention, elementwise float and nonlinearities**; "no matmul" ≠ "no multiply".
  Measured wins come from LUT kernels replacing the GEMM multiply, not from removing arithmetic.
- **Non-transformer autoregressive:** pure SSMs lag on copying/ICL (*Repeat After Me*; *Illusion of State*;
  NVIDIA 8B study); **~7% attention in a hybrid restores parity**. Modern Hopfield is **exactly** softmax
  attention, so "energy-based associative memory" is not cheaper. DEQ/EBT keep a transformer block inside.
- **Quaternion/Clifford/Hamiltonian NNs:** real, measured value is **equivariance and parameter efficiency**
  (QRNN up to 3.3× fewer params; Quaternion Transformer up to 75% reduction; GATr E(3)-equivariance) and
  **quantization geometry** (QuIP# E8-lattice codebooks; QuaRot/SpinQuant rotations). **No** evidence of a
  language-modelling compute advantage at matched scale.
- **Hardware:** on M1/M2-class CPUs, the measured cheap path is low-bit with LUT kernels (T-MAC 30/71 tok/s on M2
  Ultra for BitNet-b1.58-3B, 60–70% energy reduction; bitnet.cpp 1.37–5.07× ARM, 55–70% energy). A table-native
  path must beat **that**, not fp16.

---

### 4.6 K3 architecture review (decision gate)

An independent K3 architecture review of this document returned **partly sound**: the evidence base and the
capacity/context diagnosis are the best-supported reading of the record, but two things required correction —
the geometry recommendation understated a goal change, and the plan ignored the measurement its own base commit
already ordered. All corrections are incorporated:

1. **"Demote geometry" → conditional promotion with a gate** (§0.4, §7.1): the owner's terminal objective stands,
   but geometry must earn a causal serving role by beating a matched ordinary fix on the same probe.
2. **The ordered transport-decode depth ladder is folded into Stage 1 as arm (iv)** (§8), on the same frozen
   states, because its outcome decides whether the "no gate / 2–3-token horizon" mechanism diagnosis is real or a
   probe artifact — and therefore whether Stage 2A's gate-first priority stands.
3. **Stage-1 confounds made explicit** (§8): rank-K sweep and document hold-out are primary, not a reject-repair;
   a positive result measures **closability-to-`C`**, not language gain, because a bilinear readout can approximate
   `C`'s table from `prev` alone.
4. **Hybrid-attention boundary stated** (§8 Stage 4): integer/LUT only; float softmax layers would be an explicit
   owner goal change, not a Stage-4 default.
5. **Count prior retained as a standing local control** in every stage; a natural-text long-range panel is added
   so Stage 3 does not repeat the authored-fixture trap.

**Falsifiers of this proposal:** (a) Stage 1 returns *state-limited* **and** the `h_dim`/gate ablation fails its
0.10-bit paired criterion — then the capacity diagnosis is wrong and no branch is defined, requiring an owner
decision; (b) arm (iv) reproduces ~71–76% on `main` — the short-horizon/gate-first diagnosis collapses; (c) any
geometric arm beats the matched ordinary fix on the same probe — Goal R revives.

## 5. Diagnosis

The project has built genuine assets — exact addressed memory with version authority, deterministic serialization
and ledgers, a working multiply-free integer kernel, a disciplined evidence/retraction culture — but it has been
**optimizing a proxy (local next-token fit on its own docs) with a model too small to represent language
(64-dim state, 64-token independent windows, no gate, 4k vocab, <3 epochs), while attributing the shortfall to an
"interface" and holding a geometric mechanism in reserve that has never beaten an ordinary control.** The result
is a stable, well-instrumented local n-gram-class model, honest negatives, and no path to useful language.

The two goals have been conflated:

- **Goal S (serving arithmetic):** no float matmul / no transformer at runtime. **Sound and provable.**
- **Goal R (representation):** geometric operators confer predictive advantage. **Unsupported by every matched test.**

---

## 6. Is this a "world model"?

No, not in either industry sense.

- **Sense A (RL/planning):** a learned, action-conditioned latent dynamics model used to plan (Ha & Schmidhuber
  2018; DreamerV3, *Nature* 640:647–653, 2025; LeCun's JEPA predicts representations, still for control). UOR-R4
  has no action conditioning and no planner.
- **Sense B (generative simulator):** an observation-level renderer (Sora "video generation models as world
  simulators"; Genie). UOR-R4 emits text tokens, not observations.

At most UOR-R4 is *trivially* "an autoregressive latent-state sequence model" — the same sense in which Mamba,
RWKV and xLSTM are. **Recommendation: do not use "world model" in project claims**; it invites the wrong
comparator and an unfalsifiable capability assertion.

---

## 7. The reframing

1. **Goal S stays; Goal R becomes a gated hypothesis, not an assumption.** Keep the multiply-free, table-native
   serving contract as the *declared kernel*. Treat prime/zeta/R4/H4/`Z[phi]` as **preferred candidates for the
   witnessed failures** (input gate, cross-window/structural state, structured low-bit closure) and as substrate
   for exact identity/version and deterministic serialization — but require each to beat an information- and
   compute-matched ordinary control before it earns a causal serving role. This preserves the owner's terminal
   objective while removing the unsupported assumption. **Change of emphasis, not of goal** (§4.6).
2. **Move the modelling bet to capacity, context, exposure and objective.**
3. **Replace the proxy objective with an information-theoretic one.** Score on a panel where the previous two
   tokens are insufficient, so the count control is at chance by construction.
4. **Do not build the addressed `(prev,cur)` local read** (8–16 MiB for 0.04–0.08 bits); the count-blend decision
   already settled that.

---

## 8. Recommended plan (staged; each stage has a reject criterion)

**Stage 0 — Correct the record (docs only).** Refresh `#820`'s stale "next"; annotate the two headline numbers;
keep the `prev`-retention contradiction open and explicit. *No compute.*

**Stage 1 — Frozen-state adjudication (cheap, decisive; no retraining).** On the already-recorded frozen states,
document-held-out, using a **pre-declared rank-K sweep** (held-out selection, not a reject-repair), with these
arms:
(i) linear float readout (known **5.5136**); (ii) rank-K bilinear/quadratic float readout; (iii) 2-layer MLP
float readout; and **(iv) a transport-aware previous-token decode on the byte-identical states** (SHA
`c5334071…992f`) — this last arm is the already-ordered adjudication and settles the **70.96% / 75.90% vs 2.1%**
contradiction that determines whether "no gate → 2–3-token horizon" is a mechanism or a probe artifact.
- If (iii)−(i) **< 0.10 bits** → **state-limited** → Stage 2A (grow state/recurrence).
- If **≥ 0.35 bits** → **readout-limited** → Stage 2B (served factorised/LUT readout).
- Middle zone (0.10–0.35) is **inconclusive**; report the rank-K curve rather than a binary.
- **Interpretation guard:** a rank-K bilinear readout can approximate `C`'s table from `prev` alone, so a positive
  result measures **closability-to-`C`**, not language gain (`C` 4.7846 is itself far from useful; the arm-(iv)
  result fixes whether the state even retains `prev`). Do not read Stage 1 as a capability result.
- *If arm (iv) reproduces ~71–76% on `main`*, the §4.2 "no gate / short horizon" ranking is a probe artifact and
  **Stage 2A's gate-first priority must be re-derived** before proceeding.

**Stage 2A — Capacity/gate ablation (cheap, one variable).** Retrain at `h_dim` 128 and 256 with an
input-dependent gate (`h' = a_t ⊙ h + b_t`, computed with bounded integer ops) and a matched dense-float control,
everything else identical. *Accept if* held-out bits/target improves by ≥0.10 with a paired interval excluding 0.
This directly tests whether "the interface" or "capacity" is binding.

**Stage 2B — Served readout with explicit higher-order features.** Add a bounded-integer bilinear/rank-K (or exact
LUT) feature block to the readout; measure against the equal-rank linear readout and against `C`. *Accept if* it
closes ≥0.10 bits at ≤2× per-token cost.

**Stage 3 — Cross-window state + a real language probe (the objective correction).** Carry the recurrent state
across window boundaries; train on a longer-context mixture; evaluate on **induction / copy-at-distance / agreement**
panels (K=32…512) against the `(prev,cur)` count control *at chance by construction* and an equal-work stateful
n-gram. **Keep the count prior as a standing local control in every stage** (it exposed the −0.0417-head/+0.552-tail
degeneracy); and because authored induction panels risk the §3.4 authored-fixture trap, **require a natural-text
long-range agreement/reading panel alongside the synthetic one**. *This is the single deciding metric.*

**Stage 4 — LUT kernel + hybrid backbone.** Implement a LUT-based low-bit matmul kernel (T-MAC-style) and evaluate
a hybrid recurrent backbone with a small number of attention-like context layers. **Boundary (K3):** any
attention-like context layer is admissible **only if its scores, normalization and aggregation execute inside the
declared bounded-integer/LUT kernel** — D0-b forbids served float and transcendentals, and softmax needs `exp`.
Importing the external "~7% attention" result as **float** softmax layers would abandon Goal S and requires an
explicit **owner goal-change decision**, not a Stage-4 default. The external 7% figure is float/GPU/~8B-scale —
label it external and unmeasured at this scale. Adopt only if it beats the incumbent at equal quality.

**Stage 5 — Scale and qualify.** Only after Stages 1–4 show a capability over the count control: scale
capacity/data, then the whole-path M1 measurement (tokens/s, J/token, RSS, instruction mix) against
llama.cpp/bitnet.cpp.

**Why this beats the current roadmap.** The current plan folds a sound arithmetic goal together with an
unsupported representational claim and optimizes a servable proxy, so it cannot converge. This plan (i) separates
the two goals, (ii) tests the binding constraint first with a cheap paired ablation, (iii) replaces the
unwinnable proxy with an information-theoretic objective, and (iv) keeps every geometric asset at the scope where
it demonstrably works (identity/addressing/cheap structured operators).

---

## 9. Sizing

- **Raise** `h_dim` 64→128/256 (`TL_MAX_H_DIM=1024` allows it): readout ≈262 KB, embeddings ≈512 KB, ~1.5–2 MB,
  L2-resident, **still multiplier-free**. This is the measured capacity blocker.
- **Keep** the 4096 vocabulary for now; cut readout cost with bounded candidate scoring, not fewer multiplies
  (the vocabulary read already dominates the op census by 3–4 orders).
- **Cut** the proposed 8–16 MiB addressed `(prev,cur)` read (0.04–0.08 bits) and the coarse lattice tier if present.
- **Report** model-resident RAM separately from the 93.9 MiB harness peak.
- **Ledger:** ≈7.87M ms remain (limit 444.7M). Record a complete prospective projection before any multi-hour
  ladder; the 3.5M-ms unreconstructed increment is bookkeeping debt.

---

## 10. Retire / park

- **Retire from the critical path:** the two-token count-prior objective as the *target*; the "rank-64 interface"
  framing; any claim of geometric predictive advantage without a matched control.
- **Park (conditional, unchanged):** S7/E8/icosian relationship state, harmonic coefficient banks, resonance
  softmax replacement, quantum-spin/relative-phase semantics — all remain NOT_RUN pending a witnessed structural
  need against an ordinary control.
- **Keep:** exact addressed memory, version authority, router, deterministic serialization, the multiply-free
  kernel (as a scoped contract), and the quaternion/H4 structure as a *parameterisation/quantization* tool.

---

## 11. Owner decisions requested

1. **Adopt the reframing** (Goal S kept, Goal R demoted to substrate)? — recommended yes.
2. **Approve Stage 1** (readout/state adjudication; no retraining, cheap)? — recommended first.
3. **Approve Stage 2A** (`h_dim`/gate capacity ablation) as the decisive capacity test? — recommended next.
4. **Adopt the information-theoretic language probe** in place of the count-prior objective? — recommended yes.
5. **Formalise the "world model" disclaimer** and correct the two headline numbers in the public README? —
   recommended yes.
