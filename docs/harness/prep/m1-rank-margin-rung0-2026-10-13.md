# Pre-registration - M1 (rank-margin-objective)

- date: 2026-10-13 (generated 2026-10-12 late, US-Eastern; the date follows the branch and the sheet filename)
- milestone: M1 #2029 - Language base: a geometric stack language model good enough to carry chat
- mechanism: rank-margin-objective
- mechanism brief: docs/mechanisms/rank-margin-objective.md
- run: Rung 0 on the compliant trunk: train the recurrence-only geometric stack (pattern rrrrrr, no read, no pointer) with the margin hinge objective on mw-balp; arms M1 (hinge, 1 rival) and M8 (hinge, 8 rivals), 2 seeds each; GPU-only on one Vast pod (D26)
- line: evaluation integrity - count 1/3 (D21/D22 count is owned by the milestone, not this sheet; no count is reset by this sheet)
- amendments: **Amendment 1** (2026-10-13) - vacuity-proof bars per Claude lab's first `NOT_FIT` review; **Amendment 2** (kimi/pm1, 2026-10-13) - decisive bars moved to scale-invariant rank measures (top-1, recall@8) vs a GPU-computed count-predictor reference; hinge/unviolated kept as floors+diagnostics; token-margin term isolated from address_margin; deps 1-3 pinned as landed; data MD5/sha256/dev-window/epoch pins added; dep 4 rescoped to composed-path qualification (reviews: [#2029 comment 6137914625](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6137914625), [#6138896299](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6138896299), short [#6139080745](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6139080745)); **Amendment 3** (kimi/pm1, 2026-10-13) - exact dev scored-set/window/denominator policy and count-predictor units pinned per [PM ratification 6139449267](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6139449267); config pin completed (`rec_gates=geometric`, `gate=hamilton`); Codex complete-path repair registered as pre-execution dependency.

## Bar (pre-registered)

Amended three times on 2026-10-13: Amendment 1 closed the vacuity and tolerance holes in Claude lab's first
`NOT_FIT` review ([#2029 comment 6137914625](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6137914625));
**Amendment 2** answers Claude's FIT re-review ([#2029 comment 6138896299](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6138896299);
short version [6139080745](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6139080745)):
the decisive bars are **scale-invariant rank measures**, because the head's raw score is a negative
mean-L1 displacement over the **learned** token table at fixed scale 1
(`docs/labs/geometric-head-shared-writer-2026-10-13/README.md`; interface receipt
[#2029 comment 6138582838](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6138582838)) —
a hinge floor in table units moves with the table norm (weight decay 0.1 only partly resists), so it cannot
be the decisive bar; **Amendment 3** pins the dev evaluation policy below (scored set, coverage, denominator,
count-predictor units). These are **dev-panel bars**: measured on the pinned mw-balp dev panel during the
run; the frozen `v5-reply` panel is **not** scored by this rung.

- **Bar (i) — raw-score top-1, vs the count predictor.** Raw-score top-1 accuracy over the scored set of
  the pinned dev panel (dev evaluation policy below) **beats the count predictor's top-1 by >= 5 percentage
  points, on both seeds**.
- **Bar (ii) — recall@8 over the declared kept list, vs the count predictor.** recall@8 over the head's
  declared kept list **`top_token_ids(8)`** (bounded candidates only, smallest exact token identity on a
  tie, no target-aware insertion; `native_vector_writer.rs:95`) **beats the count predictor's recall@8 by
  >= 5 percentage points, on both seeds**.
- **Reference — the count predictor.** A token-frequency count predictor computed on the **same dev
  windows**, on the leased GPU inside the run: one frequency table over the dev tokens, scoring every
  position of the scored set defined in the dev evaluation policy below (ties by smallest token identity);
  its score unit is the raw occurrence count (policy below). It is a **count baseline only — no normalized
  exponential, no training, no model parameters** — and is lawful under the no-exponential rule (D23). Its
  top-1, recall@8, hinge loss and unviolated rate are reported alongside, at both rival counts — the
  diagnostics in **count units**, never compared numerically with the model's mean-L1-units hinge.
- **Anti-vacuity guards (Amendment 1's floors, kept alongside; not the decisive bars).** Final dev
  **token-margin hinge loss < 0.5 raw-score units** on both seeds, and dev **unviolated-target rate
  >= max(5x the same-seed untrained constant rate, 1% absolute)** on both seeds. Honest note: the untrained
  constant is ≈ 0, so `5 x 0 = 0` is vacuous and the **1% absolute floor** is the operative device; the
  0.5 floor is calibrated by the count predictor's own reported hinge loss.
- **The bars' noise floor.** The scored denominator is n ≈ 14.3k — the dev manifest's `response_tokens`
  = **14,287**; the run **computes and reports** the counted value under the dev evaluation policy below,
  and any mismatch with 14,287 is reported, not absorbed. The Wilson 95% half-width at that n is **< 0.83
  points at any plausible accuracy** (0.66 at 20%, 0.82 at 50%), so the 5-point margin sits far above
  interval noise; the run additionally reports the **between-seed disagreement** on both bars as the
  empirical noise readout. A bar must not be narrower than its panel's noise (D22 section 3).
- **Reported diagnostics (not decisive).** The dev token-margin hinge-loss curve, the unviolated rate, the
  monotone-curve diagnostic (below), and the **mean token-table L1 norm per checkpoint**, so table-scale
  drift is visible.
- **Monotone-curve diagnostic, anchored post-warmup.** Dev margin loss is evaluated every 128 steps with
  the example harness; for **steps >= 1024** (warmup is 200 steps), no evaluation point exceeds the running
  **post-warmup** minimum by more than **2% of that minimum**, and the final-third mean is below the
  first-third mean (thirds of the post-warmup evaluations).
- **The token term is read alone.** The objective returns separate **`token_margin`** and
  **`address_margin`** terms (coefficient 1 each; `native_vector_writer.rs` `margin_components` `:350`);
  `address_margin` is a declared auxiliary surrogate for hard admission, never the derivative of top-K
  admission. Every bar and every hinge diagnostic above reads the **token term alone**, never the
  token+address total.
- panel: `v5-reply` (grader: chat-grade (graded by qwen2.5:7b, grade-replies, temperature 0, seed 1), frozen: yes)
- panel noise: unstated for the reply panel; on the 40-row memory panel +/-3 rows is the noise seen (docs/labs/handoff-deepseek-2026-10-12.md: 'Bars are never narrower than the panel's noise (+/-3 rows on the 40-row memory panel)'). A bar must not be narrower than its panel's noise.
  - reply-acceptable: acceptable replies on the open reply panel 37/232 -> 116/232 acceptable replies [open]
  - v5-memory-check-pass: v5 multi-turn memory panel check_pass 20/40 -> 34/40 check_pass rows [open]
- no bar in this sheet may be narrower than the panel noise above (D22 section 3); for this rung the noise
  that binds is the dev-panel Wilson bound stated above.

## Base artifact
- base: none - trained from seed initialization at the pinned config; the historical 4.4M smoke config (rrarra) is context only, not a base artifact

## Data split and pins (Amendment 2)

- split: mw-balp train + its dev split; the frozen v5-reply panel rows are never training data; a KEEP reports fresh held-out dev transfer (D22 section 4).
- train pin: `data/ft-balp.tar` (private store `datasets/caseyallard/uor-r4-store`), 326,860,800 bytes, md5 `45fbd266ecab1e5c113e10ec7ae1e918`, sha256 `b1d2211377a2e99d5977cc95d46875027023b08867488b1372185b3f62a1473f`. The run reads only the `mw-balp/train/` member: 7,512,797 tokens (u16), response mask (u8), manifest schema `uor-r4-chat-corpus/v1`, m-world-v2, seed 13, 60,000 conversations, vocab 4096, member `tokens_sha256` `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`. The tar also carries `balp/` and `mw-orcp/` members the run does **not** read, and the store's `data/MD5SUMS` has **no** ft-balp entry (store-hygiene note) — the md5 above is the pin.
- dev pin: `data/ft-dev.tar`, 143,360 bytes, md5 `0a1d0a05020a5d80b5d008fd5b731134` (matches the store's MD5SUMS), sha256 `45e93e32918f8ad5c977ad3a98926b561173049aacd8bfce2b6e2e49a0390b5c`. Extracted manifest (`local/sheet-pins/ft/dev/manifest.json`): 400 conversations, m-world-v2, vocab 4096, manifest `tokens: 42,856` and `response_tokens: 14,287`, probe exclusion already applied at generation (362 probe n-grams, 8 words; 0 rejected draws). The file is labelled `"split": "train"` — a generator artifact; this file **is** the frozen dev panel this sheet scores, and its token FILE is authoritative (42,888 u16 = 85,776 bytes; the manifest's `tokens: 42,856` differs by 32 and **both numbers are reported**, never silently reconciled — see the dev evaluation policy below). Coverage: 111 non-overlapping 384-token windows **plus one tail window** over the remaining 264 positions (policy below).
- tokenizer pin: sha256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`, vocab 4096, bos 0 / eos 1 / unk 2.
- dose pin: 7,324 steps x batch 16 x context 384 = 44,957,696 tokens, about **6 passes** over the 7,512,797-token train split.

## Dev evaluation policy (Amendment 3, per [PM ratification 6139449267](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6139449267))

The exact policy Codex's CLI/metrics agent implements against, pinned so the bars' denominator and units
cannot drift:

- **Authoritative source.** The dev token FILE is authoritative: **42,888 u16 tokens** in `data/ft-dev.tar`
  (md5 `0a1d0a05020a5d80b5d008fd5b731134`, sha256 `45e93e32918f8ad5c977ad3a98926b561173049aacd8bfce2b6e2e49a0390b5c`).
  The manifest's `tokens: 42,856` differs by 32; the run **reports both numbers and flags the mismatch** —
  it never silently picks one.
- **Scored set.** Every position `p` with `response_mask[p]=1` that has at least one in-document
  predecessor. Context **resets at each `<|bos|>` document boundary** (bos_id = 0); no scored position
  reads across a boundary.
- **Coverage.** All 42,888 positions are covered: **111 non-overlapping 384-token windows plus one tail
  window** covering the remaining 264 positions; the tail window's context may overlap the previous window
  but it scores only its tail positions. Windows are a **batching device**, not an exclusion rule.
- **Denominator.** The run **computes and reports** the scored count under this rule. The sheet's noise
  analysis cites n ≈ 14.3k (manifest `response_tokens` = 14,287, Wilson 95% half-width < 0.83 points); the
  pre-registered expectation is that the counted denominator **equals the mask sum over scored positions**,
  and any mismatch with 14,287 is reported, not absorbed.
- **Count-predictor score units.** The decisive bars need only an ordering: `score(t)` = occurrence count
  of `t` in the dev stream, ties by smallest token identity; top-1 = argmax; recall@8 = membership among the
  8 highest-count identities. The count predictor's hinge/unviolated diagnostics use raw counts as the
  score unit (margin-1 hinge in **count units**) and are reported in their own units — **never numerically
  compared** with the model's mean-L1-units hinge.

## Steps
- steps: 7324 steps, batch 16, lr 0.002, warmup 200, min_lr 0.1, weight_decay 0.1, context 384 (the geometric-stack example's pinned defaults)

## Minimum test power (D22 section 3)
- seeds: 2 (>= 2 seeds for a decisive run)
- fresh rows: not the decisive axis here - the two seeds carry D22 section 3, and rung 0 scores the pinned mw-balp dev panel (111 + 1 tail windows; dev evaluation policy above) whose denominator is pinned above rather than estimated here. No fresh draw is scored by rung 0; the frozen panels are rung 0b.
- decisive run: YES - D22 section 3 needs >= 2 seeds OR >= 200 fresh rows
- trained in rather than bolted on: YES - state the pretraining or the full fine-tune that gives the base time to adapt (a short add-on is a probe, not a decisive run)
- fresh-transfer plan for a KEEP: the two decisive rank bars - (i) top-1 and (ii) recall@8, each vs the count predictor - are re-measured on a fresh held-out draw of mw-balp windows the run never trained on, with the anti-vacuity guards and the run's own dev diagnostics (hinge curve, post-warmup monotone check, L1 norm, between-seed disagreement) reported alongside. The frozen v5-reply panel is **not** the transfer measure here: rung 0 is recurrence-only and has no read or reply path, so it cannot be scored on that panel.
- noise floor: the bars must not be narrower than the dev-panel noise (the Wilson bound above); a single-row bar is not a bar

## Cost (charged cumulatively, before execution)
- preparation: no GPU - source, sheet and dataset checks on the owner's laptop (data already published; no re-tokenisation planned). Peak RAM and storage are bounded by the existing tree plus the run's own cache.
- run: one Vast pod, 1x RTX 5090 (or the next rung of the ladder), **<= $4 total projected** for the whole job (arms M1 and M8 x 2 seeds = 4 runs on one GPU, sequential); historical 1x4090 estimate was <= $2. Context 384, batch 16, 7324 steps per run. Within the owner caps (<= 4 running pods, <= $8/h across all labs).
- controls: **no separate control pod.** The bar's control is the **count predictor** (a frequency table over the same dev windows, computed on the same leased GPU inside the run — no training, no exponential), plus the same-seed *untrained* model's constant for the anti-vacuity guard (a zero-step forward on the same windows); the historical CE artifacts are quoted as cross-objective readings at their own scope, never as a like-for-like baseline, and no banned CE arm is run.
- evaluation: scored on the same pod during the run - the decisive top-1 and recall@8 against the count predictor, the anti-vacuity guards (token-margin hinge loss, unviolated rate), the reported diagnostics (dev curve, table L1 norm, between-seed disagreement), and (only for a KEEP) the fresh held-out draw. No panel grading is charged here: the frozen v5-reply panel is not scored by rung 0.
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

The rung trains the config below through this path. Each element is merged or in flight on `origin/main` at
`96c0ccb0a`; the sheet is the registration the dependencies are checked against.

| Element | State | Where |
| --- | --- | --- |
| Gate (MLP) | **Merged** — the Hamilton rotation G2, `Gate::Hamilton`, no exponential in either direction; the trunk applies the op itself so bf16 activations reach the bf16 kernel | `geometric_stack.rs:4022` (dispatch), `hamilton_gate` (`:18066`); #2260 merged |
| Projection | **LANDED** — #2279 (merge `c6cd409da`): Codex's owner-directed exact SpiralCore/FBS signed-action 8D native projection, `projection=spiralcore_fbs`; the dense all-pair contraction is retired with refusals; routing credit is the declared root-edge secant surrogate (the selected-input adjoint is exact); device-qualified on a Vast RTX 5090 | `crates/uor-r4-training/src/spiralcore_fbs.rs` (+ `spiralcore_fbs.cu` via `cuda_stack_kernels.rs`); record `docs/labs/spiralcore-fbs-projection-2026-10-12/README.md` |
| Head | **LANDED** — #2301 (merge `d41dc422f`): `head=spiralcore_product` (requires `projection=spiralcore_fbs`), the shared native vector head/writer; raw score = negative mean-L1 displacement over the SAME learned embedding table at fixed scale 1; kept list `top_token_ids(8)` with smallest-identity ties; 8 product cells = 128 candidate comparisons at V=4096 (bounded above by 512); separate `token_margin` + `address_margin` (weight 1 each; the address term is a declared auxiliary surrogate); omitted targets are scored but never inserted into inference candidates; legacy probability paths refuse | `crates/uor-r4-training/src/native_vector_writer.rs` (`top_token_ids` `:95`, `margin_components` `:350`, `compatibility_scores` `:458`); trunk use `geometric_stack.rs:10253`/`:10268`/`:11167`; record `docs/labs/geometric-head-shared-writer-2026-10-13/README.md` |
| Objective | **Merged/in-tree** — `Objective::MarginHinge` with the CUDA f32 kernels; returns the separate token/address margins above | `:1374`, `margin_row`/`margin_row_grad` (`:18574`/`:18598`), `margin_rank_rows`/`margin_rank_grad` (`cuda_stack_kernels.rs:738`/`:744`) |
| Recurrence | **LANDED** — `rec_gates=geometric` (#2293, merge `d7934bf03`): clamped-linear opening, exact dyadic rate-select table (`lambda = 2^-(s+1)`, keeps precomputed), Hamilton output gate; rate-selector credit is the declared secant surrogate; the executed recurrence path is squash-free by source sweep, and `RecGates::Historical` is preserved as the reproduction path, prohibited for new runs | `geometric_stack.rs` `RecGates` (`:1419`) and the `rate_select` CUDA kernels (`cuda_stack_kernels.rs`); record `docs/labs/recurrence-geometric-gates-2026-10-13/README.md` |
| Read | Not present in this rung (pattern `rrrrrr`), which is exactly why rung 0 comes before rung 0b | — |

**The run is BLOCKED until dependency 4 below lands and the repair PRs (item 5) merge** (dependencies 1, 2
and 3 are landed: #2279 `c6cd409da`, #2301 `d41dc422f`, #2293 `d7934bf03`). This sheet is the registration
they are checked against; no step of it executes while the training hold stands.

## Remaining dense, squashing or host-fallback sites on the path

From the completed recurrence trace at `d094d86df`, plus a sweep of the landed replacement path on fresh
`origin/main` at `0c8a1722a`. No dense, squash-family or host-fallback site remains on the declared executed
path: the dense all-pair projection row is resolved by #2279, the four recurrence rows by #2293, the dense
tied head by #2301; `native_vector_writer.rs` carries 0 squash-family tokens; the read-path `exp` is not
executed (no read layer).

| Site | What it does | Where (verified at `d094d86df`) | Named replacement |
| --- | --- | --- | --- |
| Opening sigmoid | `opening = sigmoid(gate_row[lane])` | `geometric_stack.rs:14977`; CUDA `cuda_stack_kernels.rs:853`/`:916`; Metal `metal_stack_kernels.rs:443`/`:641` | **Resolved by #2293** — the clamped-linear opening (`score = clamp(gate opening + decay, 0, 1)`, exact subgradient) |
| Per-token decay rate | `lambda = exp(8 * opening * log_a[lane])` | `:14978`; CUDA `:854`/`:917`; Metal `:444`/`:642` | **Resolved by #2293** — `rate_select`, an exact threshold table read (dyadic entries `lambda = 2^-(s+1)`, `keep = sqrt(1 - lambda^2)` precomputed) |
| Decay parameter transform | `log_a = softplus(-d)` (`-log1p(exp(-d))`) forward `:15261`, backward `:15392`; CUDA `:797`; Metal `:570` | `:14941` | **Resolved by #2293** — the per-lane decay parameter is read raw as the rate-selector score; no softplus transform remains in the geometric arm, so the codec caveat no longer applies to this rung's config |
| Output gate | `state * gelu(g[i])` (tanh-GELU) | `:15274`; CUDA `:864`-`:867` fwd, `:911`/`:916`-`:917` bwd; Metal `:474`-`:477`, `:635` | **Resolved by #2293** — the Hamilton rotation gate (`hamilton_gate`) on the recurrence output, the pre-declared `gate=hamilton` extension |
| Per-head exp in the read path | `exp` in the score/softplus normalization | CUDA `:992`; Metal `:732` | not executed by this rung (no read layer); it is rung 0b's item |
| Dense all-pair projection | `HamiltonLinear`'s quaternion contraction | `:18120` (+ its CUDA kernels/launchers) | Codex's SpiralCore/FBS 8D native projection — **landed** (#2279, `c6cd409da`); the dense contraction is retired with refusals |
| Dense vocabulary head | tied `embedding.weight` through `linear()` | `:10049` | **Resolved by #2301** — `head=spiralcore_product` replaces the dense tied head; legacy probability artifact admission, sampled choice, exponential table pool and probability loss refuse rather than convert |
| Host fallback | when `cuda_covered()` is false the read runs the exact CPU forward on host copies | `geometric_stack.rs` read path; `whole_path_device.rs` | **must refuse loudly** (D24): the rung has no read layer, so this path is not executed here; the declared native config already refuses non-CUDA model execution (`geometric_stack.rs:3854`; export/QAT under `rec_gates=geometric` at `:3210`), and the composed-path qualification is dependency (4) |

**Known remainder (not a dense, squash or fallback site).** A full-V **compatibility score allocation**
remains (`native_vector_writer.rs:456`): a rows x V score buffer for candidate allocation only, with no
additional vector comparisons and no dense parameter map; the bounded path is 128 comparisons at V=4096
(512 at the fixed maximum). Legacy probability paths refuse rather than execute (#2301's refusal list;
legacy head conversion refused at `geometric_stack.rs:10214`).

The state carry itself (the quaternion product and `keep * drive`) is already squash-free, and the served
`native_geometric` runtime is already squash-free — neither is on this rung's change list. Metal carries the
same unlawful kernels as CUDA but is unused on pods; the rung runs CUDA only.

## Configuration / initialization / parameters

| Field | Value |
| --- | --- |
| config line | `head=spiralcore_product projection=spiralcore_fbs rec_gates=geometric gate=hamilton`; F32; CUDA-only; no sampling (raw-score greedy emission replaces probability sampling on this path) |
| width | 256 |
| pattern | `rrrrrr` — six recurrence blocks, no read layers |
| heads | 4 |
| MLP hidden | 384 |
| context | 384 |
| vocabulary | 4096 |
| pointer | `none` — the margin objective refuses a pointer today (`:2773`) |
| gate | `hamilton` |
| rec gates | `rec_gates=geometric` — landed #2293; `historical` (the serde default) is preserved as the reproduction path and prohibited for new runs |
| projection | `spiralcore_fbs` — the landed SpiralCore/FBS 8D native projection (dependency 1, satisfied: #2279 `c6cd409da`) |
| head | `head=spiralcore_product` — landed #2301 (dependency 2, satisfied: `d41dc422f`); 8 product cells, kept list 8; requires `projection=spiralcore_fbs` |
| objective | `hinge`, arms at 1 and 8 declared rivals |
| objective terms | `token_margin` + `address_margin`, weight 1 each; every bar and hinge diagnostic reads the **token_margin** term alone (the address term is a declared auxiliary surrogate for hard admission) |
| initialization | `StackConfig` default initialization at that config, one RNG seed per arm-seed pair; the **exact parameter count is recorded in the run manifest at build**, not guessed here |
| training dtype | f32 — it matches the landed margin CUDA f32 kernel; a bf16 margin kernel is out of scope for this rung |

## Lawful quality measures with units

From the operator card's whole-path admission table, plus Amendment 2's scale-invariant rank bars and the
count-predictor reference. **No BPB, no NLL, and no normalized-exponential
evaluator is computed or reported anywhere in this rung.**

| Measure | Definition and unit |
| --- | --- |
| Raw-score top-1 | fraction of scored positions (dev evaluation policy) whose target wins on raw score; deterministic tie rule: smallest token identity. Decisive bar (i): vs the count predictor, >= 5 points on both seeds |
| Target rank / recall@K | rank of the target and the fraction with the target in the declared kept list `top_token_ids(8)` (K = 8, matching the M8 arm's rivals). Decisive bar (ii): vs the count predictor, >= 5 points on both seeds |
| Count predictor (the reference) | token-frequency table over the same dev windows, scoring the scored set defined in the dev evaluation policy; its top-1, recall@8, hinge loss and unviolated rate are reported at both rival counts — the diagnostics in **count units**, never compared numerically with the model's mean-L1-units hinge. Count baseline only: no normalized exponential, no training |
| Margin hinge (token term) | `mean_k max(0, 1 - z_target + z_rival)` over the head's token scores, raw-score units at the fixed scale 1, read from the `token_margin` term alone; anti-vacuity guard < 0.5 |
| Unviolated-target rate | fraction of dev positions with zero token-margin hinge violation; anti-vacuity guard >= max(5x the untrained constant, 1%), the 1% floor operative |
| Dev margin-loss curve (diagnostic) | per arm, per checkpoint, per seed |
| Mean token-table L1 norm (diagnostic) | per checkpoint, so table-scale drift is visible |

The frozen **v5-reply panel remains the M1 milestone acceptance and is not scored by this rung**: a
recurrence-only model has no read or reply path. Milestone acceptance is unchanged (reply 37/232, v5 memory
20/40) and this sheet moves neither.

## Panel noise

The panel JSON states that the reply panel's noise is **unstated**, and that the 40-row memory panel's
observed noise is **+/-3 rows**. Neither panel is scored by this rung. The noise that binds this rung is the
**dev-panel** noise: at the pre-registered scored denominator n ≈ 14.3k (manifest `response_tokens` 14,287;
the run computes and reports the counted value — dev evaluation policy above) the Wilson 95% half-width is
< 0.83 points at any plausible accuracy, so the 5-point bar margin sits far above interval noise; the run
also reports the **between-seed disagreement** as the empirical readout. The bars are applied to **both**
seeds; a one-seed pass is *iterate*, never a negative (D22).

## Mechanism-own target and distinct outcome decisions

- **KEEP** — the two decisive rank bars (top-1 and recall@8, each vs the count predictor by >= 5 points) met
  on both seeds, with the anti-vacuity guards (token-margin hinge loss < 0.5; unviolated rate floored) also
  met: the margin hinge becomes the project's training objective. A KEEP reports fresh held-out dev transfer
  (D22 section 4) and proceeds to rung 0b (rank-read CUDA kernel, score-gradient stop, murmuration address
  credit).
- **ITERATE** — one seed passes, or a positive but sub-bar margin on the rank measures: diagnose the
  configuration (objective scale, dose, initialization); one revised run needs a **new sheet**.
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

The executed path qualifies **on device against stored-or-analytic expected values, with no CPU model
reference** (D24). Dependency (4) is rescoped by Amendment 2 to the **composed path the run executes** —
projection + geometric recurrence + head/objective: the expected values for that composed path are stored or
analytic (golden vectors, known-answer identities, hand-computed margin rows). #2301's five finite CUDA
controls qualify the head/writer kernels **only** — they do not qualify the composed full-trunk learning
path — and the current bootstrap parity test compares against CPU references, itself out of step with
D24/D25; a parity file that exits zero on a CPU reference is not device qualification. Codex's fixed-controls
pattern is the template, and Codex's next stated step is to "freeze and qualify the complete declared
learning/output path".

## Dependencies that must merge first

The FIT review checks each of these as it lands; none of them is delivered by this sheet:

1. **Codex's SpiralCore/FBS projection PR — LANDED (2026-10-13).** PR [#2279](https://github.com/UOR-Foundation/uor-r4/pull/2279), merge commit `c6cd409da`: `projection=spiralcore_fbs`, the dense all-pair contraction retired with refusals, device-qualified on a Vast RTX 5090; routing credit is the declared root-edge secant surrogate (the selected-input adjoint is exact) (record: `docs/labs/spiralcore-fbs-projection-2026-10-12/README.md`).
2. **OP3a head and shared native vector writer — LANDED (2026-10-13)** (Codex; [ownership split ratified #2029 comment 6138381873](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6138381873)). PR [#2301](https://github.com/UOR-Foundation/uor-r4/pull/2301), merge commit `d41dc422f`: `head=spiralcore_product` (requires `projection=spiralcore_fbs`); raw score = negative mean-L1 displacement over the SAME learned embedding table at fixed scale 1; kept list `top_token_ids(8)` (smallest-identity ties); 8 product cells = 128 candidate comparisons at V=4096 (bounded above by 512); the objective returns separate `token_margin` + `address_margin` terms (weight 1 each; the address term is a declared auxiliary surrogate for hard admission); omitted targets are scored but never inserted into inference candidates; legacy probability paths refuse. **The five finite CUDA controls qualify the head/writer kernels only — they do not qualify the composed full-trunk learning path** (record: `docs/labs/geometric-head-shared-writer-2026-10-13/README.md`).
3. **The recurrence squash replacements — LANDED (2026-10-13).** PR [#2293](https://github.com/UOR-Foundation/uor-r4/pull/2293), merge commit `d7934bf03`: `rec_gates=geometric` — clamped-linear opening, exact dyadic rate-select table (`lambda = 2^-(s+1)`, keeps precomputed), Hamilton output gate; rate-selector credit is the declared secant surrogate; the executed recurrence path is verified squash-free by source sweep, and `RecGates::Historical` is preserved as the reproduction path, prohibited for new runs (record: `docs/labs/recurrence-geometric-gates-2026-10-13/README.md`).
4. **The lawful device-qualification gate for the composed path — OPEN** (Codex — same split). Stored-or-analytic expected values for the composed projection + recurrence + head path the run executes, no CPU model reference; Codex's next stated step is to freeze and qualify the complete declared learning/output path.
5. **Codex's complete-path repair PRs — IN FLIGHT; must merge before execution** (claim `codex/rung0-complete-path-20261013`; ratified in [PM ratification 6139449267](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6139449267)). Scope: ordinary-recurrence geometric dispatch, CLI `rec_gates`/native-eval wiring, head row-cap 4096 to 6144, `RateSelect` backward secant sign, and direct interior rate-bin decay initialization (`Historical` init untouched, so existing artifacts reproduce exactly). Their commits will be cited in the FIT re-request. **Initialization pin:** the repair PR must record the exact interior bin scores it uses (values + rationale, e.g. dyadic-table midpoints) so this sheet can cite them once landed.

Until dependency **4** lands, the **repair PRs (item 5) merge**, **and the owner explicitly releases the
training hold**, the run does not execute. A `TEST FITNESS: FIT` verdict lifts no hold by itself.

## Verdict
TEST FITNESS: PENDING
