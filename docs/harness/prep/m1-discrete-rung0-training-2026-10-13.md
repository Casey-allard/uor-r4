# Pre-registration - M1 (discrete-rung0-training)

- date: 2026-10-13
- milestone: M1 #2029 - Language base: a geometric stack language model good enough to carry chat
- mechanism: discrete-geometric-learning
- mechanism brief: docs/mechanisms/discrete-geometric-learning.md
- run: Rung 0 on the DISCRETE stack — train the merged discrete SpiralCore/FBS model (`crates/uor-r4-discrete`, merged at `5cb36188e`, construction-qualified source `a2a5b89cdddd1b32bf32f0e3c18d6b21e994e523`, 11/11 exact controls PASS in the admitted attempt3) on mw-balp with the integer margin objective and the enumerated bivector-move learning law; 2 paired seeds; GPU-only on one admitted pod (D24/D26)
- line: evaluation integrity - count 1/3 (D21/D22 count is owned by the milestone, not this sheet; no count is reset by this sheet)
- amendments: none yet
- supersedes: `m1-rank-margin-rung0-2026-10-13.md` (F32 optimizer law, FIT at Amendment 6, FROZEN by Amendment 7 under D32 — retained as the design record; its lawful pins carry over here as marked)

This sheet registers a TRAINING run on the discrete learning law. It succeeds the frozen F32
sheet under R13. The M1 reply-quality headline remains SUSPENDED under R14 — no panel target
from `prep new` or `milestones.json` applies to this sheet. Training remains HELD by the owner;
a FIT verdict on this sheet lifts no hold. D32 governs: no floating point anywhere in model
computation, learning or instruments (R13 ruling 2 — exact-only verification).

## Law under test (merged, construction-qualified)

The discrete carrier: one shared i32 codebook `vocabulary × blocks × 8` on the sixteen-root E8
anchor orbit; 64-state SpiralCore signed actions; ordered ternary FBS children/frontier.
Routing: coordinate L1 primary, coordinate Hamming secondary, stable port identity resolves
ties; left-composed BFS actions transport the carrier. Shared writer: one device table for
encode/carry/admission/ranking/scoring; admission from CURRENT codebook endpoints (geometric,
not token-ID-derived); targets never enter admission. Integer objective: numerator
`8*token_hinge + width*address_hinge` over denominator `8*width`; absent comparisons contribute
zero with explicit zero eligibility; wholly ineligible batches refuse with typed status.
Learning law: for a selected parameter (`Parameter::Route` / `Parameter::VectorBlock`), replay
the SAME full route/writer cost over baseline-fixed eligible rows under identity + 15 actual
bivector moves; commit only a strictly improving minimum-cost move; total tie order: identity,
then lowest resulting port identity, then canonical generator enumeration index. No optimizer,
no gradient, no floating point. `Parameter::Gate` (kind 3) is reserved and refuses — the
murmuration read is a separate line (#2378) and is not trained here.

## Bar (pre-registered)

Decisive bars are the scale-invariant rank measures carried over from the frozen sheet
(Amendment 2 rationale: substrate-independent, computable by exact integer comparison):

- **Bar (i) — raw-score top-1 vs the count predictor.** Top-1 accuracy over the scored set of
  the pinned dev panel beats the count predictor's top-1 by **>= 5 percentage points, on both
  seeds**. Model ranking uses the writer's exact integer score order; ties by smallest token
  identity.
- **Bar (ii) — recall@8 over the declared kept list vs the count predictor.** recall@8 over the
  writer's declared kept list (bounded candidates only, exact token identity ties) beats the
  count predictor's recall@8 by **>= 5 percentage points, on both seeds**.
- **Reference — the count predictor (carried over, frozen-sheet Amendment 5 scope).** A
  token-frequency count table over the **response-masked target positions of the TRAIN split**
  (`mw-balp/train/`: 7,512,797 tokens; 2,425,925 response-masked targets; ties by smallest
  token identity), computed on the leased GPU inside the run by exact integer counting. Count
  table only — no normalized exponential, no training, no model parameters; **never reads dev
  targets**. Reference numbers from the frozen sheet's verified computation: top-1 8.58%,
  recall@8 28.42% on the 14,287 dev targets, so the +5pp bars read ≈ **13.6% top-1** and
  ≈ **33.4% recall@8**. The dev-response oracle is diagnostic only.
- **Anti-vacuity guards (carried over, substrate-independent form).** Dev unviolated-target
  rate (fraction of scored positions with zero hinge violation, integer hinge) >= max(5× the
  same-seed untrained zero-move constant, 1% absolute). The frozen sheet's absolute float hinge
  floor (< 0.5 raw-score units) does not translate across substrates and is REPLACED by: final
  dev mean integer hinge strictly below the same-seed untrained constant's, by a margin wider
  than between-seed disagreement. Between-seed disagreement is reported.
- **Fresh transfer (D22 §4).** A KEEP also reports top-1/recall@8 on a fresh held-out draw from
  the same generator, scored identically.
- R12's four questions are answered in the Review section below.

## Base artifact

- base: the discrete model initialized by `Device::new(spec, seed, ordinal)` at the pinned
  spec (below); no artifact inheritance from the F32 trunk. The frozen trunk's learned
  artifacts are comparators at their own scope only (D28: no transformer/neural comparison).

## Data split (carried over verbatim from the frozen sheet)

- Train: `data/ft-balp.tar` (private store `datasets/caseyallard/uor-r4-store`), 326,860,800
  bytes, md5 `45fbd266ecab1e5c113e10ec7ae1e918`, sha256
  `b1d2211377a2e99d5977cc95d46875027023b08867488b1372185b3f62a1473f`. Reads `mw-balp/train/`
  only: 7,512,797 u16 tokens + `response_mask.u8` (7,512,797 bytes, sha256
  `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`); member
  `tokens_sha256` `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`.
- Dev: `data/ft-dev.tar`, 143,360 bytes, md5 `0a1d0a05020a5d80b5d008fd5b731134`, sha256
  `45e93e32918f8ad5c977ad3a98926b561173049aacd8bfce2b6e2e49a0390b5c`; `ft/dev/tokens.u16` =
  42,856 payload tokens; `ft/dev/response_mask.u8` = 42,856 bytes, 14,287 eligible targets.
- Tokenizer: sha256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`,
  vocab 4096, bos 0 / eos 1 / unk 2.
- Training data never includes frozen panel rows (D22 §4).

## Dev evaluation policy (carried over; window arithmetic re-derived for the discrete context bound)

- Scored set (window-independent): every position `p` with `response_mask[p]=1` and at least
  one in-document predecessor; context resets at each `<|bos|>`. Denominator: the run computes
  and reports it; expectation **14,287**; a mismatch is reported, not absorbed.
- The discrete crate's context bound is **1..=243** (`MAX_CONTEXT 243`), so the frozen sheet's
  384-token windows do not transfer. Windows are re-partitioned at the pinned context length
  (dependency D3 pins the exact length and coverage arithmetic); the scored set and denominator
  are unchanged. Only each window's new target range scores; each eligible masked position
  scores exactly once.
- Noise: Wilson 95% half-width < 0.83 points at n ≈ 14.3k; bars are wider than the noise.

## Steps

- BLOCKED pending dependency D1: the merged crate has no training surface (`train` returns
  `TrainingHeld`; `init` hardcodes seed 7; `Device::apply` exists but is NEVER_RUN and
  unqualified). The steps are pinned when D1 lands: spec init → in-run count-predictor
  construction → enumeration schedule over `Route`/`VectorBlock` parameters → propose/apply
  sweeps over mw-balp train batches → per-checkpoint dev scoring. No partial execution before
  FIT and owner hold release.

## Minimum test power (D22 §3)

- seeds: **1 and 2**, paired (same initializer, same corpus order; only the seed differs).
  BLOCKED pending D1 (the CLI cannot set a seed today).
- fresh rows: the scored dev set is 14,287 eligible targets (>= 200).
- decisive run: YES on the rank measures once D1/D3 land (2 seeds × 14,287 scored targets).
- trained in rather than bolted on: the discrete law trains the carrier itself — codebook
  vectors and route actions are the learned parameters; there is no bolted-on head.
- noise floor: bars (i)/(ii) are 5pp against a < 0.83pp noise floor.

## Cost (charged cumulatively, before execution)

- preparation: laptop source/sheet/dataset checks only; no re-tokenization; no GPU.
- run: BLOCKED pending D2 — one admitted pod (Vast default, 1× RTX 5090 or next ladder rung,
  `uor-pod` discipline), wall time and dose pinned by Codex's enumeration cost model. The
  frozen sheet's <= $4 four-run projection does NOT transfer (different training law); D2
  supplies the new projection, marked UNVERIFIED until measured.
- controls: no separate pod — count predictor and the untrained zero-move constant run on the
  same GPU in-run.
- evaluation: same pod during the run; no v5-reply grading charged (headline suspended, R14).
- retries: inside the same lease and projected cost; a self-set budget is not a stop rule.
- storage: checkpoints/reports small (max state 4 MiB at the largest lawful spec); iCloud
  `cloud-store put` with MD5 round-trip; local copy removed; 128 MiB stop margin.
- placement (D24): all model compute on the pod GPU; laptop does prep/grading only.
- owner caps: <= 4 running pods and <= $8.00/h across all labs; no new spending class.

## Stop rule

- stop: owner hold release is the start gate; hard stops are the owner caps, the disk floor,
  and a failed or diverged run. Self-set step/wall budgets are estimates — extend and renew
  rather than cancel a healthy run; record the extension on the owning issue.

## Dependencies (must land before the FIT verdict)

- **D1 — training surface in `crates/uor-r4-discrete` (owner: Codex).** Seed-pinnable CLI
  (no hardcoded seed 7), a corpus/batch driver bound to the pinned mw-balp hashes with
  caller-side BOS segmentation per the dev policy, the enumeration-schedule pin (which
  parameters, in what order, per sweep), integer evaluation reporting (top-1 / recall@8 /
  unviolated rate / hinge vs the count table), and `apply` admitted under this sheet with its
  own exact construction controls (epoch increment, stale/foreign refusal, non-improvement
  refusal — all already in `discrete.cu`, needing execution controls). All integer, no FP.
- **D2 — enumeration cost model (owner: Codex).** propose/apply per-move cost, sweep cost at
  the pinned spec, dose definition (moves evaluated / applied, corpus tokens seen), wall-time
  and dollar projection, tokens/s on the named harness shape. Marked UNVERIFIED until measured.
- **D3 — context length pin (owner: Kimi, with Codex).** Exact context <= 243, dev coverage
  arithmetic at that length, train windowing. Recommended 192; pinned after D1's driver exists.
- **D4 — owner hold release.** Not a sheet dependency for FIT; required for execution only.

## Owner rules that apply

- D21/D22: count owned by the milestone; a negative closes this configuration only, never the
  mechanism; stopping a mechanism needs the owner.
- D23: no softmax/exp-family anywhere — this sheet's objective, reference and evaluators are
  integer comparisons and counts. Interlock: `uor-harness ratchet d23|d25`.
- D24/D26: GPU-only model compute; pods via `scripts/pod/uor-pod`, Vast default.
- D28: no transformer/neural comparisons, teachers or graders.
- D29: AOT nvcc PTX only; NVRTC banned. Interlock: `uor-harness ratchet d25`,
  `d29_compiled_modules` test.
- D31: an instrument defect is never a blocker or a negative; repair the instrument, re-run.
- D32 + R13: no floating point in model computation, learning or instruments; exact-only
  verification.
- R12: the four authority questions are answered below; build-first governs.
- R14: no target is set off a vacuous instrument; the M1 reply headline stays suspended.

## Review (R12's four questions, pre-answered for the reviewer)

1. **What it measures:** whether the discrete enumerated-move learning law improves exact
   next-token rank measures on pinned dev data beyond a train-split frequency reference —
   a learning signal in the model's native integer substrate. Not language quality, not chat.
2. **Computable now:** the rank measures and count reference are exact integer computations;
   the scored set and denominator are pinned. Computability of the RUN awaits D1 (no training
   surface exists today — stated, not hidden).
3. **Can the instrument beat the bar in principle:** yes — integer rank comparisons have no
   precision floor; the count reference's numbers are known (8.58% / 28.42%), so the bars are
   concrete and reachable if learning works at all. If the mechanism cannot beat a frequency
   table by 5pp, that is a real negative about this configuration (never the mechanism, D22).
4. **Should it run at this stage:** construction is qualified (11/11 admitted); the next
   uncertainty is whether the discrete law learns anything at all on real data. This is the
   cheapest decisive check of exactly that question, and it gates any larger campaign.

## Verdict

TEST FITNESS: PENDING — dependencies D1–D3 unlanded; review requested from the assigned
milestone reviewer only when Codex posts D1/D2 heads. This sheet registers intent and pins
everything that is pinnable today; nothing executes.
