# Pre-registration - M1 (rank-margin-objective)

- date: 2026-10-13 (generated 2026-10-12 late, US-Eastern; the date follows the branch and the sheet filename)
- milestone: M1 #2029 - Language base: a geometric stack language model good enough to carry chat
- mechanism: rank-margin-objective
- mechanism brief: docs/mechanisms/rank-margin-objective.md
- run: Rung 0 on the compliant trunk: train the recurrence-only geometric stack (pattern rrrrrr, no read, no pointer) with the margin hinge objective on mw-balp; arms M1 (hinge, 1 rival) and M8 (hinge, 8 rivals), 2 seeds each; GPU-only on one Vast pod (D26)
- line: evaluation integrity - count 1/3 (D21/D22 count is owned by the milestone, not this sheet; no count is reset by this sheet)

## Bar (pre-registered)
- bar: an arm works iff dev margin loss falls monotonically AND the dev unviolated-target rate exceeds the same-seed untrained constant rate by >= 5x, on BOTH seeds; bars never narrower than panel noise (D22)
- panel: `v5-reply` (grader: chat-grade (graded by qwen2.5:7b, grade-replies, temperature 0, seed 1), frozen: yes)
- panel noise: unstated for the reply panel; on the 40-row memory panel +/-3 rows is the noise seen (docs/labs/handoff-deepseek-2026-10-12.md: 'Bars are never narrower than the panel's noise (+/-3 rows on the 40-row memory panel)'). A bar must not be narrower than its panel's noise.
  - reply-acceptable: acceptable replies on the open reply panel 37/232 -> 116/232 acceptable replies [open]
  - v5-memory-check-pass: v5 multi-turn memory panel check_pass 20/40 -> 34/40 check_pass rows [open]
- no bar in this sheet may be narrower than the panel noise above (D22 section 3).

## Base artifact
- base: none - trained from seed initialization at the pinned config; the historical 4.4M smoke config (rrarra) is context only, not a base artifact

## Data split
- split: mw-balp train (7,512,797 tokens) + its dev split; frozen v5-reply panel rows are never training data; a KEEP reports fresh held-out dev transfer (D22 section 4)
- training data never includes the frozen panel rows; scoring uses the frozen panel and, for a KEEP, a fresh held-out draw (D22 section 4)

## Steps
- steps: 7324 steps, batch 16, lr 0.002, warmup 200, min_lr 0.1, weight_decay 0.1, context 384 (the geometric-stack example's pinned defaults)

## Minimum test power (D22 section 3)
- seeds: 2 (>= 2 seeds for a decisive run)
- fresh rows: not the decisive axis here - the two seeds carry D22 section 3, and rung 0 scores the mw-balp dev split, whose own row/window count is recorded in the run manifest at build rather than estimated here. No fresh draw is scored by rung 0; the frozen panels are rung 0b.
- decisive run: YES - D22 section 3 needs >= 2 seeds OR >= 200 fresh rows
- trained in rather than bolted on: YES - state the pretraining or the full fine-tune that gives the base time to adapt (a short add-on is a probe, not a decisive run)
- fresh-transfer plan for a KEEP: the same two bars (monotone dev margin loss, >= 5x the same-seed untrained unviolated-target rate) are re-measured on a fresh held-out draw of mw-balp windows the run never trained on, with the run's own dev curve reported alongside. The frozen v5-reply panel is **not** the transfer measure here: rung 0 is recurrence-only and has no read or reply path, so it cannot be scored on that panel.
- noise floor: the bars above must not be narrower than the panel's stated noise; a single-row bar is not a bar

## Cost (charged cumulatively, before execution)
- preparation: no GPU - source, sheet and dataset checks on the owner's laptop (data already published; no re-tokenisation planned). Peak RAM and storage are bounded by the existing tree plus the run's own cache.
- run: one Vast pod, 1x RTX 5090 (or the next rung of the ladder), **<= $4 total projected** for the whole job (arms M1 and M8 x 2 seeds = 4 runs on one GPU, sequential); historical 1x4090 estimate was <= $2. Context 384, batch 16, 7324 steps per run. Within the owner caps (<= 4 running pods, <= $8/h across all labs).
- controls: **no separate control pod.** The bar's control is the same-seed *untrained* model's constant rate (a zero-step forward on the same dev windows, computed on the same leased GPU inside the run); the historical CE artifacts are quoted as cross-objective readings at their own scope, never as a like-for-like baseline, and no banned CE arm is run.
- evaluation: scored on the same pod during the run - the dev margin-loss curve and unviolated-target rate per checkpoint, the raw-score top-1 and rank measures below, and (only for a KEEP) the fresh held-out draw. No panel grading is charged here: the frozen v5-reply panel is not scored by rung 0.
- retries: inside the same <= $4 projection and the same lease - diagnose, correct and retry within the remaining budget; a self-set wall-clock estimate is not a stop rule, and a healthy run is extended and its lease renewed rather than cancelled.
- storage: checkpoints and reports are small (the pinned config is well under 100 M parameters); retained artifacts go to the iCloud store (`cloud-store put`) and the local copy is removed after the MD5 round-trip. The 128 MiB physical stop margin is respected; no artifact is deleted while it is unique evidence.
- placement (D24): model compute is GPU work only - lease the GPUs you use through `scripts/pod/uor-pod run POD --lab L --session S --gpu K -- CMD`; no CPU model compute and no host fallback (a fallback is a defect to fix, not a cost to report)
- pod/lease plan: one Vast pod through `scripts/pod/uor-pod` (D26, the default provider for new pods), 1x RTX 5090 or the next rung of the ladder; lease the GPU with the lab/session flags and `--hours` covering the whole job plus margin (projected <= 4 h), renew before expiry instead of stopping a healthy run, then release and `uor-pod down` when no other session holds it. Pod id, hourly rate and the actual lease are recorded in the run report at build time, not guessed here.
- owner caps: <= 4 running pods and <= $8.00/h across all labs, all sessions; no new spending class without an owner decision (AGENTS.md, docs/labs/compute.md section Caps)

## Stop rule
- stop: two arms x two seeds, one run each, no silent additions; self-set budgets are estimates - extend and renew a healthy run; hard stops are the owner caps, the disk floor, divergence
- self-set step/wall budgets are estimates, not stop rules: extend and renew rather than cancel a healthy run; the only hard stops are the owner caps, the disk floor and a failed or diverged run

## Owner rules that apply
- D21: Three consecutive merged PRs on one line that do not move the milestone headline force a pivot card.
- D22: A negative closes a configuration, never a mechanism. Decisive runs use >=2 seeds or >=200 fresh rows, train the mechanism in, and carry a TEST FITNESS: FIT review on the milestone before compute.
- D23: No softmax anywhere in the active project: training, serving, sampling, objectives, auxiliary losses, teachers, comparators. No table/renamed equivalent. Interlock scripts/check_no_softmax.py.
- D24: No CPU model compute; no GPU on work that cannot change a decision. A host-side fallback is a defect, not a cost to report.
- D22: a negative closes this configuration only, never the mechanism; stopping a mechanism needs the owner.

## Panel target (milestone registry)
- headline: M1: reply acceptable 37/232 -> target 116/232
- panel: v5-reply
- source: docs/harness/milestones.json (issue #2029)

## The exact executed path (merged commits / open PRs)

The rung trains the config below through this path. Each element is either merged, in flight, or not built;
the sheet is the registration the dependencies are checked against.

| Element | State | Where |
| --- | --- | --- |
| Gate (MLP) | **Merged** — the Hamilton rotation G2, `Gate::Hamilton`, no exponential in either direction; the trunk applies the op itself so bf16 activations reach the bf16 kernel | `geometric_stack.rs:4022` (dispatch), `hamilton_gate` (`:18066`); #2260 merged |
| Projection | **In flight, no PR** — Codex's owner-directed exact SpiralCore/FBS signed-action 8D native projection replaces `HamiltonLinear`'s dense all-pair contraction; claim [#2029 comment 6136794551](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6136794551) | `struct HamiltonLinear` (`:18120`) and `hamilton_linear_fwd/bwd_w/bwd_x` (`cuda_stack_kernels.rs`); #2262 merged, replacement not landed |
| Head | **Not built** — the OP3a product-key geometric head is Kimi's next source item; today the head is the tied embedding through the same dense `linear()` | `StackModel::head` (`:10049`) |
| Objective | **Merged/in-tree** — `Objective::MarginHinge` with the CUDA f32 kernels | `:1374`, `margin_row`/`margin_row_grad` (`:18574`/`:18598`), `margin_rank_rows`/`margin_rank_grad` (`cuda_stack_kernels.rs:738`/`:744`) |
| Recurrence | **Unlawful sites remain** (below) — the quaternion-transport recurrence with its held/drive carry already squash-free, and its opening, decay and output gate still exponential-family | `geometric_stack.rs:14977`/`:14978`/`:15274`, CUDA and Metal twins |
| Read | Not present in this rung (pattern `rrrrrr`), which is exactly why rung 0 comes before rung 0b | — |

**The run is BLOCKED until the four dependencies below land.** This sheet is the registration they are
checked against; no step of it executes while the training hold stands.

## Remaining dense, squashing or host-fallback sites on the path

From the completed recurrence trace at `d094d86df`. These are the sites the run would execute today; each
must be replaced before the rung is lawful, and the replacements are named source items.

| Site | What it does | Where (verified at `d094d86df`) | Named replacement |
| --- | --- | --- | --- |
| Opening sigmoid | `opening = sigmoid(gate_row[lane])` | `geometric_stack.rs:14977`; CUDA `cuda_stack_kernels.rs:853`/`:916`; Metal `metal_stack_kernels.rs:443`/`:641` | exact hard/rank selection by address — the `LatchGates::Hard` precedent (`:1745`) |
| Per-token decay rate | `lambda = exp(8 * opening * log_a[lane])` | `:14978`; CUDA `:854`/`:917`; Metal `:444`/`:642` | an **exact** rate selection (grid/table read with a stated identity) |
| Decay parameter transform | `log_a = softplus(-d)` (`-log1p(exp(-d))`) forward `:15261`, backward `:15392`; CUDA `:797`; Metal `:570` | `:14941` | as above. **Caveat, not a solution:** the served `DecayRate` codec currently encodes the same transform (`"grid codes of the rate `8 softplus(-decay)`"`, `:13055`) — reusing that table would *move* the softplus rather than remove it, so the lawful rate selection is a new source item, not a re-use |
| Output gate | `state * gelu(g[i])` (tanh-GELU) | `:15274`; CUDA `:864`-`:867` fwd, `:911`/`:916`-`:917` bwd; Metal `:474`-`:477`, `:635` | the Hamilton gate (`gate=hamilton`, `:18066`) extended to the recurrence output |
| Per-head exp in the read path | `exp` in the score/softplus normalization | CUDA `:992`; Metal `:732` | not executed by this rung (no read layer); it is rung 0b's item |
| Dense all-pair projection | `HamiltonLinear`'s quaternion contraction | `:18120` (+ its CUDA kernels/launchers) | Codex's SpiralCore/FBS 8D native projection (in flight, claim above) |
| Dense vocabulary head | tied `embedding.weight` through `linear()` | `:10049` | OP3a product-key geometric head (not built) |
| Host fallback | when `cuda_covered()` is false the read runs the exact CPU forward on host copies | `geometric_stack.rs` read path; `whole_path_device.rs` | **must refuse loudly** (D24): the rung has no read layer, so this path is not executed here, and the refusal is a named dependency (4) |

The state carry itself (the quaternion product and `keep * drive`) is already squash-free, and the served
`native_geometric` runtime is already squash-free — neither is on this rung's change list. Metal carries the
same unlawful kernels as CUDA but is unused on pods; the rung runs CUDA only.

## Configuration / initialization / parameters

| Field | Value |
| --- | --- |
| width | 256 |
| pattern | `rrrrrr` — six recurrence blocks, no read layers |
| heads | 4 |
| MLP hidden | 384 |
| context | 384 |
| vocabulary | 4096 |
| pointer | `none` — the margin objective refuses a pointer today (`:2773`) |
| gate | `hamilton` |
| projection | the post-Codex SpiralCore/FBS 8D native projection (dependency 1) |
| head | OP3a product-key geometric head (dependency 2) |
| objective | `hinge`, arms at 1 and 8 declared rivals |
| initialization | `StackConfig` default initialization at that config, one RNG seed per arm-seed pair; the **exact parameter count is recorded in the run manifest at build**, not guessed here |
| training dtype | f32 — it matches the landed margin CUDA f32 kernel; a bf16 margin kernel is out of scope for this rung |

## Lawful quality measures with units

From the operator card's whole-path admission table. **No BPB, no NLL, and no normalized-exponential
evaluator is computed or reported anywhere in this rung.**

| Measure | Definition and unit |
| --- | --- |
| Raw-score top-1 | fraction of fixed next-token targets whose raw score wins; deterministic tie rule: lower index |
| Target rank / recall@K | rank of the target and the fraction admitted to the declared kept list, K = 8 (matching the M8 arm's rivals) |
| Margin hinge | `mean_k max(0, 1 - z_target + z_rival)`, raw-score units, fixed score scale; the scores are unnormalized head outputs |
| Unviolated-target rate | fraction of dev positions with zero hinge violation at the fixed scale |
| Dev margin-loss curve | per arm, per checkpoint, per seed |

The frozen **v5-reply panel remains the M1 milestone acceptance and is not scored by this rung**: a
recurrence-only model has no read or reply path. Milestone acceptance is unchanged (reply 37/232, v5 memory
20/40) and this sheet moves neither.

## Panel noise

The panel JSON states that the reply panel's noise is **unstated**, and that the 40-row memory panel's
observed noise is **+/-3 rows**. Neither panel is scored by this rung. The rung's own bar is the monotone
dev-margin-loss plus the >= 5x unviolated-rate rule applied to **both** seeds, which is wider than seed-level
jitter by construction; a one-seed pass is *iterate*, never a negative (D22).

## Mechanism-own target and distinct outcome decisions

- **KEEP** — both bars met on both seeds: the margin hinge becomes the project's training objective. A KEEP
  reports fresh held-out dev transfer (D22 section 4) and proceeds to rung 0b (rank-read CUDA kernel,
  score-gradient stop, murmuration address credit).
- **ITERATE** — one seed passes, or the ratio is 2-5x: diagnose the configuration (objective scale, dose,
  initialization); one revised run needs a **new sheet**.
- **BLOCKER** — a diverged or non-finite loss, a kernel defect, or an observed host fallback: a tooling
  blocker to fix, **not** a mechanism negative (D22 section 1-2).
- **A mechanism stop is an owner decision.** No outcome of this run stops the rank-margin family, and this
  sheet never proposes one.

## Tokens/s

Measured on the named harness: the `geometric-stack` example's training loop
(`crates/uor-r4-training/examples/geometric-stack.rs`), one Vast RTX 5090, f32, context 384, batch 16;
useful tokens divided by synchronized GPU elapsed seconds, reported per arm and alongside the total path time.
It is a speed reading at that exact shape/dtype/context/batch and is not quality or energy evidence.

## Lawful device qualification

The kernels qualify **on device against stored golden vectors and analytic known-answer identities** — the
quaternion norm preserved under Hamilton transport, the exact icosian/E8 signed-action table identities, and
margin rows computed by hand — with **no CPU model reference** (D24). The current bootstrap parity test
compares against CPU references and is therefore itself out of step with D24/D25; replacing it with
stored/analytic expected values is dependency (4), owned by Kimi with Codex on enforcement. A parity file that
exits zero on a CPU reference is not device qualification.

## Dependencies that must merge first

The FIT review checks each of these when it lands; none of them is delivered by this sheet:

1. **Codex's SpiralCore/FBS projection PR** — replaces `HamiltonLinear`'s dense all-pair contraction.
2. **OP3a, the product-key geometric head** (Kimi) — replaces the dense tied head.
3. **The recurrence squash replacements** — opening selection, exact decay rate, output gate (Kimi source
   items).
4. **The lawful device-qualification gate** (Kimi, with Codex enforcement) — stored/analytic expected values,
   no CPU model reference.

Until all four land **and the owner explicitly releases the training hold**, the run does not execute. A
`TEST FITNESS: FIT` verdict lifts no hold by itself.

## Verdict
TEST FITNESS: PENDING
