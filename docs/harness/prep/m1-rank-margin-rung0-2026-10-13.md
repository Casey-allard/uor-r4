# Pre-registration - M1 (rank-margin-objective)

- date: 2026-10-13 (generated 2026-10-12 late, US-Eastern; the date follows the branch and the sheet filename)
- milestone: M1 #2029 - Language base: a geometric stack language model good enough to carry chat
- mechanism: rank-margin-objective
- mechanism brief: docs/mechanisms/rank-margin-objective.md
- run: Rung 0 on the compliant trunk: train the recurrence-only geometric stack (pattern rrrrrr, no read, no pointer) with the margin hinge objective on mw-balp; arms M1 (hinge, 1 rival) and M8 (hinge, 8 rivals), 2 seeds each; GPU-only on one Vast pod (D26)
- line: evaluation integrity - count 1/3 (D21/D22 count is owned by the milestone, not this sheet; no count is reset by this sheet)
- amendments: **Amendment 1** (2026-10-13) - vacuity-proof bars per Claude lab's first `NOT_FIT` review; **Amendment 2** (kimi/pm1, 2026-10-13) - decisive bars moved to scale-invariant rank measures (top-1, recall@8) vs a GPU-computed count-predictor reference; hinge/unviolated kept as floors+diagnostics; token-margin term isolated from address_margin; deps 1-3 pinned as landed; data MD5/sha256/dev-window/epoch pins added; dep 4 rescoped to composed-path qualification (reviews: [#2029 comment 6137914625](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6137914625), [#6138896299](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6138896299), short [#6139080745](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6139080745)); **Amendment 3** (kimi/pm1, 2026-10-13) - exact dev scored-set/window/denominator policy and count-predictor units pinned per [PM ratification 6139449267](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6139449267); config pin completed (`rec_gates=geometric`, `gate=hamilton`); Codex complete-path repair registered as pre-execution dependency; **Amendment 4** (Codex, 2026-10-13, scoped coordination [6140439165](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6140439165)) - correct UORT header accounting per [6139637354](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6139637354), pin paired seeds 1/2, verified tokenizer restoration, exact CLI/source/init and finite qualification; acceptance bars unchanged, scientific verdict FIT and all holds retained; **Amendment 5** (kimi/pm1, 2026-10-13) - count-predictor reference rescoped to train-split response-target frequencies per Claude's verified CONTEST ([6145360853](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6145360853), [6145601377](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6145601377)); dev-response oracle demoted to diagnostic; verdict reverts to PENDING pending the code fix + Claude's re-review; **Amendment 6** (kimi/pm1, 2026-10-13) - mechanical CLI pins from Claude's CONFIRM (`train_mask`, `expected_train_targets`, train-mask sha256); dep 6 marked landed; verdict restored to FIT — training hold remains the owner's to release.

## Bar (pre-registered)

Amended three times for the design on 2026-10-13, followed by Amendment 4's scoped factual/source pins and Amendment 5's count-predictor reference correction: Amendment 1 closed the vacuity and tolerance holes in Claude lab's first
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
- **Reference — the count predictor (Amendment 5; corrected scope).** A token-frequency count table over
  the **response-masked target positions of the TRAIN split** (`mw-balp/train/`: 7,512,797 tokens,
  2,425,925 response-masked targets; ties by smallest token identity), computed on the leased GPU inside the
  run, scoring every position of the scored set defined in the dev evaluation policy below. It is a
  **count table only — no normalized exponential, no training, no model parameters** — lawful under D23, and
  it must **never read dev targets**. Additionally the run reports a **dev-response oracle** (the same table
  built on the dev response targets) as a **diagnostic only**, never as the reference the bars are measured
  against. Its top-1, recall@8, hinge loss and unviolated rate are reported alongside, at both rival counts —
  the diagnostics in **count units**, never compared numerically with the model's mean-L1-units hinge.
- **Why the reference was rescoped (Amendment 5).** The first definition counted every dev token, but the
  bars score only masked response positions. Measured on the pinned dev panel: the whole-stream reference
  scores **0.17% top-1 / 16.6% recall@8**, while the response-frequency table scores **8.6% / 28.9%** — and a
  constant emitter (always-token-1) matches the response-frequency level, so against the wrong reference the
  +5pp bars are passable by a constant: the vacuity class Amendments 1–2 closed. Claude's CONTEST
  ([6145360853](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6145360853)) was
  reproduced exactly by the PM (0.17%/16.57% vs 8.58%/28.94%).
- **Expected reference level.** The dev-response oracle ≈ **8.6% top-1 / 28.9% recall@8**; the train-split
  response reference is expected near that level and is **measured and reported in-run**. The decisive bars
  are unchanged: top-1 and recall@8 each beat the corrected reference by ≥ 5 percentage points on both
  seeds; the anti-vacuity floors (hinge < 0.5; unviolated ≥ max(5× untrained, 1%)) are unchanged.
- **Anti-vacuity guards (Amendment 1's floors, kept alongside; not the decisive bars).** Final dev
  **token-margin hinge loss < 0.5 raw-score units** on both seeds, and dev **unviolated-target rate
  >= max(5x the same-seed untrained constant rate, 1% absolute)** on both seeds. Honest note: the untrained
  constant is ≈ 0, so `5 x 0 = 0` is vacuous and the **1% absolute floor** is the operative device; the
  0.5 floor is retained as the existing anti-vacuity guard; the count predictor's hinge has different units and does not calibrate this floor.
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
- dev pin: `data/ft-dev.tar`,143360 bytes, md5 `0a1d0a05020a5d80b5d008fd5b731134`, sha256 `45e93e32918f8ad5c977ad3a98926b561173049aacd8bfce2b6e2e49a0390b5c`. Its `ft/dev/tokens.u16` member is85776 physical bytes:64-byte UORT header +42856 u16 payload tokens. The header declares42856 tokens, matching the manifest; there is **no32-token data discrepancy**. The previous42888 figure counted32 header words as tokens. `ft/dev/response_mask.u8` has42856 bytes,14287 positive/eligible within-document target positions across400 documents. The manifest's `"split":"train"` label remains a generator artifact; this is the pinned dev panel. Probe exclusion was already applied at generation (362 probe n-grams,8 words;0 rejected draws). The unchanged payload covers111 global384-token blocks +232 tail tokens; actual document-bounded windows are counted separately under the dev policy. No corpus/tar is regenerated, padded or changed.
- tokenizer pin: sha256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`, vocab 4096, bos 0 / eos 1 / unk 2.
- dose pin: 7,324 steps x batch 16 x context 384 = 44,957,696 tokens, about **6 passes** over the 7,512,797-token train split.

## Dev evaluation policy (Amendments 3–4, per [PM ratification 6139449267](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6139449267))

The exact policy Codex's CLI/metrics agent implements against, pinned so the bars' denominator and units
cannot drift:

- **Authoritative source.** The decoded UORT token payload is authoritative: **42,856 u16 tokens** in the pinned `data/ft-dev.tar`. Existing `geometric-stack::read_tokens` uses `MmapCorpusReader` to exclude the64-byte header. Physical file size85776 and decoded token count42856 are separately reported; both agree with the manifest. The response mask length is42856.
- **Scored set.** Every position `p` with `response_mask[p]=1` that has at least one in-document
  predecessor. Context **resets at each `<|bos|>` document boundary** (bos_id = 0); no scored position
  reads across a boundary.
- **Coverage.** All42,856 decoded positions are eligible for partitioning: **111 global384-token inventory blocks plus232 tail tokens**. The executable partitions documents at the verified tokenizer BOS, then uses context-bounded windows with overlap where needed; only each window's new target range scores, so every eligible masked position is counted exactly once. Short document windows use after-document unscored BOS padding for batching. These actual document windows are reported separately; global inventory blocks are **not** a rule for excluding data or crossing BOS. The tail context may overlap, but its target positions score once.
- **Denominator.** The run **computes and reports** the scored count under this rule. The sheet's noise
  analysis cites n ≈ 14.3k (manifest `response_tokens` = 14,287, Wilson 95% half-width < 0.83 points); the
  pre-registered expectation is that the counted denominator **equals the mask sum over scored positions**,
  and any mismatch with 14,287 is reported, not absorbed.
- **Count-predictor score units (population corrected by Amendment 5).** The decisive bars need only an
  ordering: `score(t)` = occurrence count of `t` over the **response-masked target positions of the TRAIN
  split** (`mw-balp/train/`: 7,512,797 tokens, 2,425,925 response-masked targets), ties by smallest token
  identity; top-1 = argmax; recall@8 = membership among the 8 highest-count identities. The count
  predictor's hinge/unviolated diagnostics use raw counts as the score unit (margin-1 hinge in **count
  units**) and are reported in their own units — **never numerically compared** with the model's
  mean-L1-units hinge.

## Steps
- steps: 7324 steps, batch 16, lr 0.002, warmup 200, min_lr 0.1, weight_decay 0.1, context 384 (the geometric-stack example's pinned defaults)

## Minimum test power (D22 section 3)
- seeds: **1 and 2**, paired across arms: M1(seed 1), M8(seed 1), M1(seed 2), M8(seed 2). Each pair uses the same seeded parameter initializer and training-window RNG stream; only the declared rival count differs. No cross-backend bitwise training-repeatability claim. Two seeds satisfy the decisive-run seed requirement.
- fresh rows: not the decisive axis here - the two seeds carry D22 section 3, and rung 0 scores the pinned mw-balp dev panel (111 global inventory blocks +232 tail tokens; actual document windows and coverage policy above) whose denominator is pinned above rather than estimated here. No fresh draw is scored by rung 0; the frozen panels are rung 0b.
- decisive run: YES - D22 section 3 needs >= 2 seeds OR >= 200 fresh rows
- trained in rather than bolted on: YES - state the pretraining or the full fine-tune that gives the base time to adapt (a short add-on is a probe, not a decisive run)
- fresh-transfer plan for a KEEP: the two decisive rank bars - (i) top-1 and (ii) recall@8, each vs the count predictor - are re-measured on a fresh held-out draw of mw-balp windows the run never trained on, with the anti-vacuity guards and the run's own dev diagnostics (hinge curve, post-warmup monotone check, L1 norm, between-seed disagreement) reported alongside. The frozen v5-reply panel is **not** the transfer measure here: rung 0 is recurrence-only and has no read or reply path, so it cannot be scored on that panel.
- noise floor: the bars must not be narrower than the dev-panel noise (the Wilson bound above); a single-row bar is not a bar

## Cost (charged cumulatively, before execution)
- preparation: no GPU - source, sheet and dataset checks on the owner's laptop (data already published; no re-tokenisation planned). Peak RAM and storage are bounded by the existing tree plus the run's own cache.
- run: one Vast pod, 1x RTX 5090 (or the next rung of the ladder), **<= $4 total UNVERIFIED projection** for the whole job (arms M1 and M8 x 2 seeds = 4 runs on one GPU, sequential); historical 1x4090 estimate was <= $2. Context 384, batch 16, 7324 steps per run. Within the owner caps (<= 4 running pods, <= $8/h across all labs).
- controls: **no separate control pod.** The bar's control is the **count predictor** (a frequency table over the **response-masked target positions of the TRAIN split** — never dev targets; the dev-response oracle is reported as a diagnostic only — computed on the same leased GPU inside the run, no training, no exponential), plus the same-seed *untrained* model's constant for the anti-vacuity guard (a zero-step forward on the same windows); the historical CE artifacts are quoted as cross-objective readings at their own scope, never as a like-for-like baseline, and no banned CE arm is run.
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
the final qualified source `05c84bece50789163bd0f049a2ae76b8a314ef45`; protected repair delivery is through [#2318](https://github.com/UOR-Foundation/uor-r4/pull/2318). This sheet is the registration the dependencies are checked against.

| Element | State | Where |
| --- | --- | --- |
| Gate (MLP) | **Merged** — the Hamilton rotation G2, `Gate::Hamilton`, no exponential in either direction; the trunk applies the op itself so bf16 activations reach the bf16 kernel | `geometric_stack.rs:4022` (dispatch), `hamilton_gate` (`:18066`); #2260 merged |
| Projection | **LANDED** — #2279 (merge `c6cd409da`): Codex's owner-directed exact SpiralCore/FBS signed-action 8D native projection, `projection=spiralcore_fbs`; the dense all-pair contraction is retired with refusals; routing credit is the declared root-edge secant surrogate (the selected-input adjoint is exact); device-qualified on a Vast RTX 5090 | `crates/uor-r4-training/src/spiralcore_fbs.rs` (+ `spiralcore_fbs.cu` via `cuda_stack_kernels.rs`); record `docs/labs/spiralcore-fbs-projection-2026-10-12/README.md` |
| Head | **LANDED** — #2301 (merge `d41dc422f`): `head=spiralcore_product` (requires `projection=spiralcore_fbs`), the shared native vector head/writer; raw score = negative mean-L1 displacement over the SAME learned embedding table at fixed scale 1; kept list `top_token_ids(8)` with smallest-identity ties; 8 product cells = 128 candidate comparisons at V=4096 (bounded above by 512); separate `token_margin` + `address_margin` (weight 1 each; the address term is a declared auxiliary surrogate); omitted targets are scored but never inserted into inference candidates; legacy probability paths refuse | `crates/uor-r4-training/src/native_vector_writer.rs` (`top_token_ids` `:95`, `margin_components` `:350`, `compatibility_scores` `:458`); trunk use `geometric_stack.rs:10253`/`:10268`/`:11167`; record `docs/labs/geometric-head-shared-writer-2026-10-13/README.md` |
| Objective | **Native geometric path implemented and finite CUDA-qualified** — `Objective::MarginHinge { rivals:1|8 }` dispatches through `native_head_loss` to `NativeVectorWriter::margin_rows`/`margin_components`: explicit omitted-target negative mean-L1 token scores plus the separate address surrogate, coefficient1 each. Historical `MarginRank`/logits kernels are not this head's objective | `geometric_stack.rs` native `loss`/`native_head_loss`; `native_vector_writer.rs` `margin_rows`; qualified source `05c84bece` |
| Recurrence | **Component law landed #2293; actual ordinary-path dispatch and credit repaired/qualified at `05c84bece`** — `rec_gates=geometric`: clamped-linear opening, exact dyadic rate/keep tables, Hamilton output gate. #2293 supplied the composed arm; its source sweep did not establish that ordinary forward called it. The complete-path repair routes ordinary calls through that arm, fixes signed secant credit and direct interior initialization, and adds entry-point guards. Final finite controls qualify the actual six-recurrence path; no optimizer step or training | `geometric_stack.rs` ordinary/composed recurrence and `RateSelect`; CUDA table/adjoint kernels; final source `05c84bece50789163bd0f049a2ae76b8a314ef45`, complete-path record |
| Read | Not present in this rung (pattern `rrrrrr`), which is exactly why rung 0 comes before rung 0b | — |

**The run is BLOCKED until dependency 4 below lands and the repair PRs (item 5) merge** (dependencies 1, 2
and 3 are landed: #2279 `c6cd409da`, #2301 `d41dc422f`, #2293 `d7934bf03`). This sheet is the registration
they are checked against; no step of it executes while the training hold stands.

## Remaining dense, squashing or host-fallback sites on the path

The table below preserves the historical sites and original component-law replacements from `d094d86df` and the early source sweep. **Those sweeps did not qualify the ordinary path:** despite #2293's composed geometric arm, ordinary recurrence still entered the historical fused core until the `05c84bece` repair. Read the rows as retained component history, not as proof that #2293 alone made an executed trunk compliant. The final qualified recurrence-only configuration at `05c84bece` routes ordinary and guarded diagnostic calls through the geometric law; its named fixed controls execute no dense contraction, squash-family law, NLL evaluator or CPU model fallback. Read layers are absent. Optimizer update is compiled/source-reviewed and not executed.

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
| objective | M1: `objective=hinge` (1 rival); M8: `objective=hinge8` (8 rivals) |
| objective terms | `token_margin` + `address_margin`, weight 1 each; every bar and hinge diagnostic reads the **token_margin** term alone (the address term is a declared auxiliary surrogate for hard admission) |
| initialization | Paired seeds 1/2 and the same seeded parameter stream per M1/M8 pair. Geometric rate-address scores initialize at `((lane %16)+0.5)/16`:0.03125,0.09375,0.15625,0.21875,0.28125,0.34375,0.40625,0.46875,0.53125,0.59375,0.65625,0.71875,0.78125,0.84375,0.90625,0.96875, repeated by lane. These direct bin centers avoid clamp saturation and preserve all 16 addresses; historical initialization/RNG draw order remains unchanged. Exact parameter count is recorded in the run manifest |
| training dtype | f32 — it matches the landed margin CUDA f32 kernel; a bf16 margin kernel is out of scope for this rung |

## Exact CLI and tokenizer restoration pin (Amendment 4)

Verified local tokenizer: `/Users/casey.allard/uor-r4-local/reply-cap96-inputs/tokenizer-d36d3e87.json`, SHA256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`. Durable private dataset `caseyallard/uor-r4-store`, commit `435d5f781cd7b8518185cf6e7fda2a7addb0532a`, member `pod-setup/rung0-complete-path/20261013-05c84bece/tokenizer-d36d3e87.json`; root confirms download/byte/SHA256 round-trip. Restore on an owned, admitted pod, without running any model:

```sh
hf download caseyallard/uor-r4-store pod-setup/rung0-complete-path/20261013-05c84bece/tokenizer-d36d3e87.json --repo-type dataset --revision 435d5f781cd7b8518185cf6e7fda2a7addb0532a --local-dir /workspace/rung0/restored-store
mkdir -p /workspace/rung0/data
cp /workspace/rung0/restored-store/pod-setup/rung0-complete-path/20261013-05c84bece/tokenizer-d36d3e87.json /workspace/rung0/data/tokenizer-d36d3e87.json
sha256sum /workspace/rung0/data/tokenizer-d36d3e87.json
```

After restoring the original pinned tars, extracting `mw-balp/train/` and `ft/dev/` under `/workspace/rung0/data`, verify the declared tar/member identities and mask inventory. Do not regenerate either parent. The future CLI command is pinned below; **training is held and the command must not execute**. Run it only after protected repair delivery, exact scientific FIT and explicit owner hold release. Use SEED=1 and 2 for both arms, with separate exclusive report roots; use `objective=hinge8` for M8.

```text
geometric-stack train arch=geometric pattern=rrrrrr width=256 heads=4 stack_mlp=384 context=384 gate=hamilton rec_gates=geometric projection=spiralcore_fbs head=spiralcore_product objective=hinge device=cuda precision=f32 data_parallel=1 train=/workspace/rung0/data/mw-balp/train/tokens.u16 train_mask=/workspace/rung0/data/mw-balp/train/response_mask.u8 valid=/workspace/rung0/data/ft/dev/tokens.u16 valid_mask=/workspace/rung0/data/ft/dev/response_mask.u8 tokenizer=/workspace/rung0/data/tokenizer-d36d3e87.json expected_dev_targets=14287 expected_train_targets=2425925 batch=16 steps=7324 lr=0.002 warmup=200 min_lr=0.1 weight_decay=0.1 clip=1 eval_every=128 eval_windows=111 final_windows=111 checkpoint_every=128 sample_tokens=0 max_seconds=inf seed=SEED out=EXCLUSIVE_REPORT_ROOT
```

The train mask member is pinned from the parent archive `data/ft-balp.tar`: member `mw-balp/train/response_mask.u8`, 7,512,797 bytes, sha256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3` (manifest field `mask_sha256`); the native CLI refuses to run without `train_mask` (fails closed). Claude's independent count check reproduced the reference inventory: 2,425,925 train masked targets = `expected_train_targets`; train-reference top-8 `[1,16,14,435,340,351,3,302]` scoring 8.58% top-1 / 28.42% recall@8 on the 14,287 dev targets, so the +5pp bars require ≈13.6% / 33.4% — unreachable by a constant emitter.

Native `UOR_NAN_TRACE=1` is refused: no host tensor diagnostic executes. Training scores all drawn token positions using the total token+address objective; the dev response mask selects evaluation targets only. Keep/fresh-transfer obligations above remain. No fresh dataset or seed is fabricated here; its future artifact/draw identity must be frozen and its actual disjointness checked before a KEEP transfer evaluation.

## Lawful quality measures with units

From the operator card's whole-path admission table, plus Amendment 2's scale-invariant rank bars and the
count-predictor reference. **No BPB, no NLL, and no normalized-exponential
evaluator is computed or reported anywhere in this rung.**

| Measure | Definition and unit |
| --- | --- |
| Raw-score top-1 | fraction of scored positions (dev evaluation policy) whose target wins on raw score; deterministic tie rule: smallest token identity. Decisive bar (i): vs the count predictor, >= 5 points on both seeds |
| Target rank / recall@K | rank of the target and the fraction with the target in the declared kept list `top_token_ids(8)` (K = 8, matching the M8 arm's rivals). Decisive bar (ii): vs the count predictor, >= 5 points on both seeds |
| Count predictor (the reference, Amendment 5) | token-frequency table over the **response-masked target positions of the TRAIN split** (7,512,797 tokens; 2,425,925 response-masked targets; never dev targets), scoring the scored set defined in the dev evaluation policy; the dev-response oracle is a diagnostic only. Its top-1, recall@8, hinge loss and unviolated rate are reported at both rival counts — the diagnostics in **count units**, never compared numerically with the model's mean-L1-units hinge. Count baseline only: no normalized exponential, no training |
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

Final finite implementation qualification is now complete at source `05c84bece`:18 controls pass. Its scope is forward/backward/output implementation and fixed artifact/metric controls. The optimizer kernel is compiled/source-reviewed only; it has not performed a parameter update. Scientific fitness of this sheet and a training release remain separate.

## Dependencies that must merge first

The FIT review checks each of these as it lands; none of them is delivered by this sheet:

1. **Codex's SpiralCore/FBS projection PR — LANDED (2026-10-13).** PR [#2279](https://github.com/UOR-Foundation/uor-r4/pull/2279), merge commit `c6cd409da`: `projection=spiralcore_fbs`, the dense all-pair contraction retired with refusals, device-qualified on a Vast RTX 5090; routing credit is the declared root-edge secant surrogate (the selected-input adjoint is exact) (record: `docs/labs/spiralcore-fbs-projection-2026-10-12/README.md`).
2. **OP3a head and shared native vector writer — LANDED (2026-10-13)** (Codex; [ownership split ratified #2029 comment 6138381873](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6138381873)). PR [#2301](https://github.com/UOR-Foundation/uor-r4/pull/2301), merge commit `d41dc422f`: `head=spiralcore_product` (requires `projection=spiralcore_fbs`); raw score = negative mean-L1 displacement over the SAME learned embedding table at fixed scale 1; kept list `top_token_ids(8)` (smallest-identity ties); 8 product cells = 128 candidate comparisons at V=4096 (bounded above by 512); the objective returns separate `token_margin` + `address_margin` terms (weight 1 each; the address term is a declared auxiliary surrogate for hard admission); omitted targets are scored but never inserted into inference candidates; legacy probability paths refuse. **The five finite CUDA controls qualify the head/writer kernels only — they do not qualify the composed full-trunk learning path** (record: `docs/labs/geometric-head-shared-writer-2026-10-13/README.md`).
3. **The recurrence component law — LANDED #2293; ordinary-path repair qualified at `05c84bece`.** PR [#2293](https://github.com/UOR-Foundation/uor-r4/pull/2293), merge `d7934bf03`, supplies `rec_gates=geometric`'s composed clamped-linear/rate-table/Hamilton law and explicit secant surrogate. Its early source sweep does not establish ordinary dispatch. Final source `05c84bece50789163bd0f049a2ae76b8a314ef45` repairs ordinary dispatch, signed credit and initialization and passes the complete configured finite controls (items4–5). `RecGates::Historical` source/init remains inactive evidence for new runs. Records: `docs/labs/recurrence-geometric-gates-2026-10-13/README.md` and `docs/labs/rung0-complete-path-2026-10-13/README.md`.
4. **The lawful device-qualification gate for the composed path — finite controls PASS, protected delivery through #2318** (Codex — same split). Final source `05c84bece50789163bd0f049a2ae76b8a314ef45` passes 18 finite controls (11 CUDA,7 metadata), including the actual batch16×context384 width256/MLP384/V4096/six-recurrence forward/backward, signed rate/keep table, device clipping, full6144-row writer, native metrics/count/mask and artifact reload/refusals. No optimizer update, corpus/model-quality evaluation, CPU model reference or training. Record: `docs/labs/rung0-complete-path-2026-10-13/README.md`. The2.04s full-shape control is not training throughput; the<= $4 training projection above remains UNVERIFIED.
5. **Codex's complete-path repairs — final source/device qualified, protected delivery through #2318** (claim `codex/rung0-complete-path-20261013`, source `05c84bece50789163bd0f049a2ae76b8a314ef45`). Actual ordinary geometric dispatch, native CLI/evaluation, row-cap8192 with element bound admitting6144, signed secant credit, direct interior initialization, device clipping and diagnostic helper guards are implemented. The existing pre-repair source is an indexed restoration patch at base `d4dd3caa9`. The final sheet/source identity is bound in the prospective FIT review on M1; the actual protected merge SHA is verified and appended to its delivery receipt. Training remains HELD.
6. **Count-predictor scope fix — LANDED (2026-10-13).** PR [#2328](https://github.com/UOR-Foundation/uor-r4/pull/2328), merge commit `9e904a83448c451f738d5c1d45224dfdf73eb84e` (Codex; routed by [PM ruling 6145669094](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6145669094)): `native_rung0.rs` counts response-masked TRAIN target positions (`MaskedTargets`), refuses DEV passed as TRAIN by content hash, keeps smallest-ID ties, adds the unmasked-exclusion fixed control, and reports the dev oracle as diagnostic-only; the reference is sha256-bound via `native_count_reference.json`. Claude's checklist confirms all six items ([6146716704](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6146716704); record: `docs/labs/count-reference-2026-10-13/README.md`).

Until the owner explicitly releases the training hold, the run does not execute (dependencies 1–6 are landed:
the complete-path repairs are delivered through #2318 and the count-reference fix through #2328). A
`TEST FITNESS: FIT` verdict lifts no hold by itself.

## Verdict

Amendment 4's prospective FIT (Codex head_review and root, reviewed pre-stamp Git blob
`bf0a0ededd2027794ff5ddcdab23f53782683df3`, SHA256
`9b1efca7277023e96c60ce066870bf1d788609d114fe52e061a2ad0b628273ed`) stands as the design record at that
blob. Amendment 5 reverted the verdict to PENDING on the contested count-predictor reference
([PM ruling 6145669094](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6145669094));
Claude's CONFIRM ([6146716704](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6146716704))
restores it, and Amendment 6 supplies the mechanical CLI pins it asked for. Global
training and the separate M2 training/rental hold remain **HELD**; only the owner can release them. FIT
records scientific design readiness, no permission to run. The optimizer update is compiled/source-reviewed
and NOT_RUN; the four-run cost remains an UNVERIFIED projection. No acceptance bar was changed.

TEST FITNESS: FIT (confirmed by Claude lab, independent non-author review, [#2029 comment 6146716704](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6146716704), at `9e904a834` + Amendments 5–6; restored by the sheet owner per the adjudication rule [6145409089](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6145409089))
