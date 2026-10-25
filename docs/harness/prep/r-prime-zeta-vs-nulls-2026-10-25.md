# Prime-zeta codebook against injective nulls at zero updates (`r-prime-zeta-vs-nulls`, #2881, claude-112)

- **Status:** pre-registration, written before any arm runs. It registers the bar of the D68 card [`DECISION.md`](../../loop/tasks/r-prime-zeta-vs-nulls/claude-112/DECISION.md), approved in round 2. A fresh FIT and a D67 pre-run review follow on #2881. Lab: claude.
- **Parent:** the draft [`r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md`](r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md) (antigravity-27, antigravity-36). This sheet keeps its arms, pins, measures and data. It replaces the draft's outcome rule with the card's exact partition, and its battery with a pinned job. The draft stays as the record.
- **Rules:**
  - GPU only for every model quantity (§3.5, D24). The tables are rebuilt on the pod. The host computes no model quantity: it only grades stored reports.
  - No softmax, squashing or matrix product (§3.2, §3.3).
  - The two nulls are pure integer (D32).
  - A negative closes a configuration (D22). An instrument failure is never a reading (D31).

## 1. Question

At checkpoint 0 (zero updates) and K = 8, does the prime-zeta table (P8) admit more context-sensitive targets than injective codes without primes or zeta phases?
- **Q0–Q2** keep P8's 256 × 16 cell quota and unique interior roots, but place tokens by a seeded shuffle.
- **R0–R2** are 4,096 distinct E8-orbit vectors drawn without replacement, with no quota.

The alias ceiling cannot separate them, because it is 14,287 for every injective table (draft, Proof). Reach is therefore measured by admission.

## 2. Arms (7, checkpoint 0, seed 1, `V4096 B4 C192 K8`)

Each arm's config differs from P8's only in `codebook_initialization.table` (host-checked: `instrument/tests/configs.rs`).

| Arm | `prime-zeta-codebook` mode | Config sha256 | Table `file_sha256` |
|---|---|---|---|
| P8 | `prime_zeta` | `7927b1dd…` | `83c1380a…` |
| Q0 | `quota_only_injective:0` | `51986b8c…` | `33b4e459…` |
| Q1 | `quota_only_injective:1` | `94c5634d…` | `57225bbb…` |
| Q2 | `quota_only_injective:2` | `6f8b70df…` | `e03d6aef…` |
| R0 | `random_injective:0` | `6a3a457a…` | `229e93f5…` |
| R1 | `random_injective:1` | `dc188404…` | `36fec632…` |
| R2 | `random_injective:2` | `9c78bfa9…` | `25ec9bd2…` |

- The full digests are in `pz-nulls-job.sh` and the instrument's `ARMS`.
- The configs are `docs/labs/r-prime-zeta-vs-nulls/evidence/<arm>.json`.
- The tables are rebuilt on the pod from `evidence/spec-v4096-b4-c192-k8.json` (`b66c6cf9…`) at seed 0.

## 3. Data split and base artifact

- **No base artifact.** Every arm is `init-corpus` at checkpoint 0 from its pinned table.
- **TRAIN** (`3dee858f…` / `b5e7352d…`) is bound for provenance only. No batch is run.
- **DEV** (`ft/dev`, `0919e90f…` / `a838bb1a…`): 14,287 eligible rows, of which 3,847 are P_succ rows. This is the panel of the 19 October pre-flights.
- **Fresh slices:** untouched.

## 4. Measures

- **Reach.**
  - `B3_init(P_succ)`, the primary measure: target-admitted P_succ rows from `attention-dev-b3`.
  - `B3_full`.
  - `top1_correct` and `top8_correct` from `eval-corpus`.
- **Capacity.** From `coverage-census --token-cells`:
  - `distinct_vectors` and `dev_alias_ceiling_rows`;
  - occupied cells, min and max cell occupancy;
  - `cell_collision_pairs`, which is Σ C(n_c, 2).

## 5. Bar and decision (card B, fixed here)

For each null X, `Paired { b, c } = paired_psucc(P8, X)` over the 3,847 P_succ rows, joined by row ordinal.
- **The three outcomes per null:**
  - **win:** `clears(Paired { b, c })`, which is b − c > max(24, 2·√(b + c)), decided exactly as d > 24 and d² > 4(b + c);
  - **loss:** `clears(Paired { b: c, c: b })`;
  - **tie:** otherwise.
- **The verdict, by the number of Q wins:**

| Q wins | R | Verdict | Decision |
|---|---|---|---|
| 0, all ties | any | **negative (tie)** | this configuration is closed at this power (D22). The mechanism stays ACTIVE. The quota-only table becomes a candidate pure-integer start table for a new lineage's sheet |
| 0, one or more losses | any | **negative (loss)** | as for a tie |
| 3 | 3 wins | **positive** | a "positive" row on the mechanism page: prime and zeta-phase placement raises zero-update admission. The held trained comparison gains a measured reason to run |
| 1 or 2, or 3 without 3 R wins | any | **residual** | a descriptive row with no status change; the next configuration adds seeds |

- **Reported, deciding nothing:**
  - `paired_psucc(Q_s, R_s)` for s = 0, 1, 2 (crowding at matched seed);
  - every capacity number;
  - top-1 and top-8.

## 6. Instrument checks (card C; a failure is repaired, never read)

In the job, in this order. Each one stops the job with `FAULT`.

1. **Pins:**
   - the source commit is `d1eea395d64f1d95e753f6800bfaae9c69c2f212`;
   - the five data sha256s;
   - the seven config sha256s;
   - an RTX 5090.
2. **G-Rebuild.** Each table rebuilt on the pod equals its config's table sha256. That table is the one the arm loads.
3. **G-Init.** Each checkpoint:
   - has `accepted_updates = 0` and `completed_batches = 0`;
   - has a 131,072-word (4,096 × 4 × 8) `model.table`, equal word for word to its rebuilt table's `table`;
   - carries array `routes` and `gates`, the same as the other six.
4. **G-Occ.**
   - Every arm: 4,096 distinct vectors, ceiling 14,287, zero alias tokens, zero non-representative DEV rows, and 256 occupied cells.
   - P8 and Q0–Q2: at most 16 per cell and 30,720 collision pairs.
   - R0–R2: more than 30,720 collision pairs.
5. **G-Exact.** P8 is evaluated first. It must give `B3_full` 427, `B3_init(P_succ)` 127 and top-1 38 (19 Oct receipt), before any null is evaluated.
   - Top-1 38 is that receipt's checkpoint-0 full-panel top-1. Both of its arms ran on the same model (`134fd663…`), so it is P8's number.
   - A mismatch is a G-Exact fault, and the job stops after P8 alone.

**On any fault,** the job saves its folder to `results/claude/prime-zeta-nulls-2026-10-25/fault-<time>` before exiting. A rerun moves an earlier folder aside, so it never reads that folder's markers.

**The grader** (`docs/labs/r-prime-zeta-vs-nulls/instrument/`, `pz-nulls-grade MANIFEST OUTPUT`, a detached workspace at commit `f5155fe17869c2d0ca56772cf612b464ca8df719`) re-checks gates 4 and 5. It also refuses:
- a dev-b3 report whose `config_file.sha256` is not its arm's;
- an update, read from the checkpoint's own counters: dev-b3 `model_epoch` and `completed_batches`, and eval `model_epoch` and `accepted_updates_in_model`;
- an eval or census report whose model storage hash differs from its dev-b3 report's, or is absent;
- a panel other than 14,287 / 3,847;
- summary counts that differ from the rows;
- a missing census field.

It exits 2 on an instrument fault, with no paired reading. Its 11 tests include negative gates:
- setting aside the loss branch fails 2 tests;
- setting aside the R-arm 256-cell check fails 1;
- setting aside the `accepted_updates_in_model` check fails 1.

## 7. Cost, steps and stop rule

- **Cost:** one RTX 5090 on Vast, for an hour at most. The build takes about 8 minutes, and each arm's evaluation is capped at 10 minutes. About $0.20–$0.60, inside the 4-pod / $8/h caps.
- **Steps:**
  1. FIT, then the D67 pre-run review.
  2. Run `uor-pod up --lab claude --session claude-112 --gpu 5090 --hours 1`.
  3. Ship `pz-nulls-job.sh` (sha256 `820ee91645dcb3541a212304f0fb1a518f2e7834b074a4498a909c79b4c5e80e`) over stdin into a file, and start that file detached. A watcher waits for `EXIT`.
  4. The job saves `results/claude/prime-zeta-nulls-2026-10-25/run` to the store itself.
  5. `release`, then `down`.
  6. Fetch the outputs and check SHA256SUMS. Then `pz-nulls-grade` writes `docs/labs/r-prime-zeta-vs-nulls/grade.json`.
  7. Write the REPORT, the notebook row in `docs/research/notebook/m1-prime-zeta.md`, and set the item `done`.
- **Stop:**
  - any `FAULT` is an instrument fault, to be repaired and rerun;
  - spend past the caps, or under 30 GiB of free disk;
  - `eval-corpus` refuses any checkpoint with updates.

## Minimum test power

- **The minimum detectable difference** is F. At P8's 127 admitted P_succ rows, b + c is likely at most a few hundred, so F is about 24 to 32 rows, a fifth to a quarter of P8's admissions.
- **A tie** rules out only a gain above F.
- **Seeds:** three seeds per null family exceed §4.3's minimum of two. All 3,847 P_succ rows are paired.
- **Capacity numbers** are exact counts.

## Test fitness

TEST FITNESS: FIT. A fresh-context non-author reviewer passed this sheet in round 2 (#2881 comment `6401883120`). The D67 pre-run review passed in the same record.

- **Round 1: NOT FIT,** on one Required finding: the grader's update refusal read two fields that the crate always writes as 0.
  - It was fixed in `f5155fe17`, with a negative gate.
  - Round 1's advisories were applied: the G-Init null guard, the save on fault, stale markers, the job-bytes check, the binding of eval and census reports, and the provenance of top-1 38.
- **Round 2's advisory A1,** that `.table` is a flat 131,072-word array, holds on the host for all seven rebuilt tables.
- **The reviewed bytes** of this sheet, before this section was rewritten, have sha256 `fa294ca228facf435917b0d0b95fce7045b8ad0f4f5bc2b21b5e198f13f98bd6` (commit `3a42da29f`). This section adds nothing to the test.
