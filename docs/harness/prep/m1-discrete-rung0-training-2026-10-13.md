# Pre-registration - M1 (discrete-rung0-training)

- date: 2026-10-13
- milestone: M1 #2029 - Language base: a geometric stack language model good enough to carry chat
- mechanism: discrete-geometric-learning
- mechanism brief: docs/mechanisms/discrete-geometric-learning.md
- run: Rung 0 on the DISCRETE stack — train the merged discrete SpiralCore/FBS model (`crates/uor-r4-discrete`, executable source `e7d926fbe38537f4faa90b817705c61a780778d6` at merge `6ba08ab3f`, 25/25 exact controls PASS — 11 construction + 6 flock + 8 driver) on mw-balp with the integer margin objective and the enumerated bivector-move learning law; 2 paired seeds; GPU-only on one admitted pod (D24/D26). Construction lineage: merged at `5cb36188e`, source `a2a5b89cdddd1b32bf32f0e3c18d6b21e994e523`, 11/11 admitted attempt3.
- line: evaluation integrity - count 1/3 (D21/D22 count is owned by the milestone, not this sheet; no count is reset by this sheet)
- amendments: **Amendment 1** (kimi/pm1, 2026-10-14) - D1/D2 landed and merged (PR #2400, `6ba08ab3f`); spec/dose/cost pins completed from measured values; anti-vacuity guard redefined as strict distance-win rate (PM decision [#2029 comment 6157957610](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6157957610), corrected [6158076739](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6158076739)); Codex sheet feedback ([6156858784](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6156858784), disposition [6156948254](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6156948254)) and adjudicated audit items ([#2028 6157852969](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6157852969), Codex source check [6157905825](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6157905825)) incorporated. **Amendment 2** (kimi/pm1, 2026-10-14) - dual-review items incorporated: Claude NOT_FIT [6159155868](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6159155868) (5 required items), Codex NOT_FIT [6159155839](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6159155839) (5 items), Codex reconciliation [6159179134](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6159179134), owner pilot decision [6159173706](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6159173706), Codex pilot ACK [6159198070](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6159198070): guard re-based on measured same-seed untrained strict-win rate, fresh transfer marked UNAVAILABLE, decisive checkpoint pinned at 100% only, execution-admission diff pinned, per-kind sensitivity diagnostics, owner pilot (1,640 batches, seed 1, diagnostic only), full budget reconciliation, editorial supersessions
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
- **Anti-vacuity guards (Amendment 2 form — supersedes the carried-over guard and Amendment 1
  item 4).** The guard is the **strict distance-win rate** — numerator
  `token_strict_win_eligible_rows` (eligible scored dev positions with `td < rival_distance`
  under the pinned tie order) over denominator `token_eligible_rows`; zero-eligible data is
  UNAVAILABLE, never a zero score. Integer/rational arithmetic only. Threshold: **>= max(1%,
  min(5u, u + 5pp))** on both seeds, where `u` = the same-seed untrained strict-win rate
  measured on the full scored dev set **at checkpoint 0 before any apply**, reported per seed.
  If `u > 95%` the guard is reported **INSTRUMENT-INVALID** and the bars decide alone — never
  a negative. The old mean-hinge-vs-seed-disagreement guard is superseded (diagnostic only),
  and no below-guard result auto-closes anything: R12 attribution order applies (instrument →
  fixture → bar → mechanism); movement below the bar is iteration evidence.
- **Fresh transfer (D22 §4) — UNAVAILABLE / NOT_RUN (Amendment 2).** No pinned fresh held-out
  set exists today; an arbitrary stand-in is not bound. This sheet's claim is limited to
  first-learning observations on the pinned dev panel. Any D22 KEEP-with-transfer claim
  requires a prospective amendment binding source, sha256, token count and scored count >= 200
  before it runs.
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

- SUPERSEDED by Amendment 1 item 5 (D1/D2 landed): the original "BLOCKED pending dependency D1"
  paragraph is retained below as history. The executed steps are pinned in Amendment 1 item 5
  and corrected by Amendment 2 (decisive checkpoint at 100% only; admission diff; pilot).
- ~~BLOCKED pending dependency D1: the merged crate has no training surface (`train` returns
  `TrainingHeld`; `init` hardcodes seed 7; `Device::apply` exists but is NEVER_RUN and
  unqualified). The steps are pinned when D1 lands: spec init → in-run count-predictor
  construction → enumeration schedule over `Route`/`VectorBlock` parameters → propose/apply
  sweeps over mw-balp train batches → per-checkpoint dev scoring. No partial execution before
  FIT and owner hold release.~~

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

- **D1 — training surface in `crates/uor-r4-discrete` (owner: Codex) — LANDED (Amendment 2
  editorial).** Merged as PR #2400 (`6ba08ab3f`, source `e7d926fbe`); description below is
  the historical requirement, now satisfied. The real CLI is `init SPEC NEW_ARTIFACT SEED` /
  `init-corpus` with seeds 1/2 set in the config (not a hardcoded seed 7). The enumeration
  schedule is deterministic and common across seeds (not "seeded" — the seed perturbs
  initialization, not the schedule).
- **D2 — enumeration cost model (owner: Codex) — LANDED.** Measured values posted at
  [#2029 6158385671](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6158385671);
  reconciled full budget in Amendment 2 item 8 below.
- **D3 — context length pin (owner: Kimi, with Codex) — LANDED.** Context 192 pinned in
  Amendment 1 item 1.
- **D4 — owner hold release.** Not a sheet dependency for FIT; required for execution only —
  and (Amendment 2 item 5) necessary but not sufficient for today's binary: the pinned
  admission diff must also land and be reviewed.

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

## Amendment 1 (2026-10-14) — D1/D2/D3 landed; pins completed from measured values

**Landed inputs.** Codex's D1/D2 driver merged as PR #2400 (`6ba08ab3f`), executable source
`e7d926fbe38537f4faa90b817705c61a780778d6`: 25/25 finite controls PASS (11 Route + 6 flock +
8 driver) with 11 bounded fixture apply calls / 8 accepted authored updates / zero corpus
updates; prospective FIT by an independent non-author Codex reviewer under the owner standing
exact-head review allowance ([#2029 6158278165](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6158278165);
Claude's original conditional FIT [6158018177](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6158018177)
and its A1 conditions are preserved unchanged). D2 measured cost posted at
[6158385671](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6158385671).
Claude's prospective test-design pins ([6157700698](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6157700698))
are implemented in the driver and bind this sheet's corpus/mask/apply semantics.

1. **Spec pinned (D3 resolved).** vocabulary 4096, **blocks 4**, **context 192**, batch 64
   with final tail, writer kept K 8, stages 5, coordinate inventory 16,384 VectorBlock + 20
   read = **16,404 coordinates**. blocks = 4 lifts the forced full-vector alias bound that
   held at blocks = 2 (256 cells; audit item 1.3 / Codex item 1); residual within-cell ties
   break by smallest token identity and are reported through rival eligibility counts. The
   capacity question stays open as an instrument matter: if bars fail, inspect representation
   capacity before assigning a model negative (D22/D31).
2. **Schedule pinned.** One ascending coordinate per batch (VectorBlock coordinates, then
   active read/Route coordinates), integer seeded schedule, common schedule and corpus order
   across both seeds, checkpointed with exact resume. Do NOT multiply the whole inventory by
   every batch — the schedule attempts one coordinate per batch.
3. **Seeds.** 1 and 2, paired — unchanged.
4. **Anti-vacuity guard redefined (supersedes the "zero hinge violation" guard above;
   SUPERSEDED IN TURN by Amendment 2's measured-untrained guard — see below).** The
   guard is the **strict distance-win rate** — the fraction of eligible scored dev positions
   with `td < rival_distance` under the pinned tie order — at **>= max(5x the same-seed
   untrained zero-move strict-win rate, 1% absolute)** on both seeds. It is compared only
   against its own untrained constant; no new frequency baseline is invented, and the count
   reference keeps its raw top-1/recall@8 semantics for bars (i)/(ii). Zero-hinge (margin)
   rate and the exact hinge distribution are **reported diagnostics only**. Ineligible or
   zero-comparison rows never count as successes. The evaluator emits per-row token/address
   hinges, rival eligibility counts and aggregate strict wins; no raw (td, rival_distance)
   histogram is claimed to exist. Exact integer reference counts are recomputed in-run by the
   qualified evaluator and reported; a mismatch against expectation is reported, not absorbed.
5. **Steps unblocked.** `init --seed` → in-run masked-TRAIN count-reference construction
   (GPU, exact integer counting, DEV excluded) → coordinate schedule over train windows
   (BOS-segmented; no window crosses BOS; masked positions contribute no target and no
   eligibility; a wholly masked batch refuses, not scores zero) → propose/choose/apply →
   dev scoring at 25% / 50% / 75% / 100% of the pass (each ~9 s at the measured forward
   rate). `apply` executes under this sheet only after D4; the train/admission path refused
   until then.
6. **Dose and cost pinned from D2 measurements** (median of 3 warmed repetitions, RTX 5090,
   integer ns, PTX JIT excluded and reported separately): forward 37.8 ms per 64-row batch
   (1,691 target rows/s — rows, NOT decode tokens); proposal 3.90 s (VectorBlock) / 3.96 s
   (Route/Gate). **Dose: one complete coordinate pass per seed** = 16,404 batches ≈
   1.05M row exposures (train has 2,425,925 targets — one pass covers ~43% once). Projection:
   ~17.8 h and ~$16.2 per seed at the quoted $0.908/h; ~35.6 h / ~$32.3 total for both seeds
   on one GPU — **UNVERIFIED projection** from measured per-proposal cost, inside the owner
   caps (1 pod, $0.908/h <= $8/h). Dev evaluations add ~1 min total. Checkpoints, retries and
   lease renewals ride the same lease per the stop rule.
7. **Outcome decisions.** KEEP = bars (i)/(ii) and the strict-win guard on both seeds →
   report fresh transfer (D22 §4) and open the successor-dose question. Iterate = movement
   above untrained but below bar → same-configuration dose/schedule change only. Blocker =
   instrument or arithmetic fault → repair the instrument and re-run (D31); never a negative.
   ~~A below-guard result closes THIS configuration only (D22)~~ — SUPERSEDED by Amendment 2:
   no automatic closure language; R12 attribution order (instrument → fixture → bar →
   mechanism) governs, and movement below the bar is iteration evidence. Named successor
   configurations
   — stage-indexed BRANCH generators (audit 1.2, pending Codex/Claude's exact check) and
   orbit-indexed counterfactual candidates (audit 1.4) — are recorded, not adopted pre-run.
8. **Scope unchanged:** Route/VectorBlock parameters only; Gate (kind 3) and the flock read
   remain reserved for a separately declared arm after their own finite qualification. No
   floating point anywhere in the run (D32; the zeta-complex-plane addendum is not used by
   this sheet). Training stays HELD until the owner releases D4.

## Amendment 2 (2026-10-14) — dual-review incorporation; guard, transfer, checkpoint, admission, pilot, budget

Incorporates Claude's NOT_FIT review
([6159155868](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6159155868),
five required items plus non-blockers), Codex's NOT_FIT review
([6159155839](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6159155839),
five items), Codex's reconciliation
([6159179134](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6159179134)),
the owner pilot decision
([6159173706](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6159173706))
and Codex's pilot ACK
([6159198070](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6159198070)).
Numbering follows Claude's A2-1..A2-5, then the remaining review/owner items.

1. **Guard re-based on the measured untrained constant (A2-1; Codex item 3).** Pinned in the
   Bar section above: strict-win threshold `max(1%, min(5u, u + 5pp))` with `u` measured per
   seed at checkpoint 0 before any apply; `u > 95%` → INSTRUMENT-INVALID and bars decide
   alone; never a negative; no automatic configuration closure. All arithmetic integer or
   rational; numerator `token_strict_win_eligible_rows`, denominator `token_eligible_rows`;
   zero-eligible data is UNAVAILABLE.
2. **Fresh transfer UNAVAILABLE (A2-2; Codex item 4).** Pinned in the Bar section above. No
   pinned fresh set exists, so transfer is NOT_RUN; the claim is limited to first-learning
   observations; a KEEP-with-transfer claim needs a prospective amendment binding source,
   sha256, token count and scored count >= 200.
3. **Decisive checkpoint at 100% only (A2-3).** Bars, guard and any transfer are decided **at
   the end of the complete pass only**. Exact checkpoint steps: **0 (untrained), 4,101 /
   8,202 / 12,303 / 16,404**. Intermediate checkpoints are diagnostic and are never selected
   post hoc. Per-seed dose exposes **1,049,856** eligible targets; two seeds **2,099,712**.
   One coordinate pass is not one corpus epoch (train has 2,425,925 response-masked targets;
   one pass covers ~43% once).
4. **Execution admission pinned (A2-4; Codex reconciliation).** The run executes at
   `e7d926fbe38537f4faa90b817705c61a780778d6` **plus only an owner-authorized admission
   diff**. That diff may include: controller wiring, trained-checkpoint evaluation, and
   per-kind sensitivity receipt counters (StepReceipt currently lacks candidate-sensitivity
   counts — item 6). The diff is reviewed against this sheet; executable, PTX and source
   identities are recorded before the first apply; **any other change voids the FIT**. Owner
   D4 release is necessary but not sufficient for today's binary.
5. **Sensitivity and dose diagnostics registered (A2-5).** Per parameter kind (VectorBlock,
   Route): proposals attempted / proposals where any candidate differed from identity /
   accepted updates. An insensitive-majority pass is a dose/schedule finding (Iterate, same
   configuration), never a configuration negative.
6. **Owner pilot (6159173706; Codex ACK 6159198070).** The first execution after FIT + D4 is a
   **pilot: exactly 1,640 batches, seed 1 only, diagnostic only** — no bar or guard is
   decided, it is never a negative, and the decisive checkpoint stays the 100% full-pass end.
   Schedule fact: the first 1,640 coordinates are **VectorBlock 0..1639 only** (token IDs
   0..409 at blocks = 4); the Route row reports **attempted 0 / sensitivity NOT_RUN /
   accepted 0**. Do not interleave Route without a declared schedule amendment and a focused
   metadata check. Pilot receipts: proposal-only 6,401.380044600 s + carry 2.155425760 s +
   two full-dev evals 19.469110080 s = UNVERIFIED known-component subtotal
   **6,423.004580440 s = 1.784167939 h ≈ $1.62 at $0.908/h** (excludes unmeasured production
   apply, controller/data/IO, count reference, checkpoints, setup/build, preservation and
   retries). The full ~36 h run is decided AFTER the pilot report.
7. **Budget reconciliation (Codex item 2).** Dev evaluation uses the `evaluate` API
   (42,143,551 ns per 64 rows); 224 dev batches (223 full + 15 rows). The budget includes:
   untrained initial evals for BOTH seeds, quarter evals, carry, count-reference construction
   (recomputed each invocation), snapshots, setup/build and preservation. Production apply
   cost is unmeasured — the fixture timing is NOT an estimate of it. Full-run known-component
   subtotal: **128,201.545441912 s = 35.611540400 h ≈ $32.335278683** — UNVERIFIED; never
   claim a measured all-in figure.
8. **Editorial corrections (Codex item 5; Claude non-blockers).**
   - The old Steps paragraph and the D1–D3 dependency descriptions are marked superseded
     above (all landed).
   - Real CLI: `init SPEC NEW_ARTIFACT SEED` / `init-corpus`; seeds 1/2 live in the config.
     The schedule is deterministic and common across both seeds.
   - Dev policy wording: the discrete evaluator scores **one target per window** (the token
     after the last context position) — 14,287 targets = 14,287 windows ≈ 224 batches.
   - Capacity note recorded: the query-addressed single-leaf read limits this arm's read
     capacity; the flock content-addressed multi-leaf read is the named successor, not a
     rescue. flock and Gate are finite-qualified but excluded from this Route/VectorBlock arm.
   - B4 init limitation recorded: xorshift low-4-bit root indices give <= 16 distinct
     full-vector patterns at initialization; learning can separate them later; no
     collision-free-init claim is made.

## Verdict

TEST FITNESS: PENDING — Amendment 2 posted for Claude's one-cycle confirmation and Codex's
confirmation; execution requires D4 owner release plus the pinned admission-diff review.
Nothing executes.
