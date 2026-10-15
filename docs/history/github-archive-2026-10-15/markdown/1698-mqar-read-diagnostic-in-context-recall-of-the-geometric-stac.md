# #1698 — MQAR read diagnostic: in-context recall of the geometric stack by distance (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-04T21:34:19Z
- **Updated:** 2026-10-09T13:08:45Z
- **Closed:** 2026-10-04T21:36:01Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1698
- **Merged:** 2026-10-04T21:36:01Z
- **Merge commit:** f92ba0f786e8f17f00dd3bf5537efe321a4d1193
- **Base / Head:** main / UOR-Foundation:codex/mqar-bench
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820.

## What this adds

`crates/uor-r4-training/examples/mqar-bench.rs` is a self-contained Rust diagnostic. It trains small geometric stacks (`StackModel`) on synthetic multi-query associative recall (MQAR) and measures in-context recall by query distance. There is no transformer control.

**The task.** Each window is filler tokens holding 8 key-value pairs per distance bucket.
- A pair writes key `k` at position `p` and value `v` at `p+1`. Its one query repeats `k` at `q = p + d`, and the target at `q` is `v`.
- Loss weight falls only on the query positions (`weighted_loss`).
- Distances are drawn uniformly inside each bucket, so no fixed offset reaches the key. The buckets are d16 = 12–24, d64 = 48–96, d200 = 150–300 and d400 = 350–480. A d1000 bucket (1000–1100) is defined and is used automatically when `context >= 1101`.
- Keys, values and filler come from disjoint token ranges of a 512-token vocabulary.
- Every sequence draws fresh pairings from its own seed. A pairing recurs across sequences only at the rate of independent uniform draws, so recall must come from the context.

**The arm registry, from coordination with the Codex lab.** The generator, training loop and per-distance scoring see an arm only through the `ContextArm` trait. To add an arm, add one `ArmSpec` variant, its parse branch and its constructor.
- Every report uses a fixed schema (`uor-r4/mqar-bench/arm-record/v1`): `task`, `arm{label, kind, parameters, config}` and `results{context_access, final_in_class_fresh_pairings, final_held_out_class_pairings, curve, timing}`.
- `context_access` records the positions each arm scores per query token.
- Scoring breaks wrong answers into three kinds: another pair's value, the most recent pair's value, and an absent value.
- `mode=task-baselines` reports the data-only recency baseline. That baseline answers with the most recently written value and does no key matching.

**Checks run at the head:**
- `cargo fmt --all -- --check` passes.
- `cargo check -p uor-r4-training --example mqar-bench` passes.
- `cargo test --release -p uor-r4-training --example mqar-bench` passes 5 tests: the answer is at the stated distance (key at `q-d`, value at `q-d+1`, each key exactly twice, each value once), pairings are fresh with no held-out pairing in training, weights fall only on queries, one stack arm trains and scores and counts its context access, and the recency baseline matches a brute-force scan.

## Runs

All arms share the same settings:
- CPU at 4 threads (`RAYON_NUM_THREADS=4`).
- Context 512, width 128, 4 heads, MLP 384, 6 layers.
- Batch 8, 1800 steps, lr 1e-3 with 100 warmup steps and cosine decay to 1e-4, weight decay 0.1, clip 1.0, seed 1.
- Evaluation on 64 fresh sequences: 512 queries per bucket, argmax over the whole vocabulary.

The binary is from commit `eba6a15f`. Report roots are sealed under `~/uor-r4-local/mqar-bench/grid3-ctx512-eba6a15f/`. The recency baseline is from `a87a858a` and is in the `task-baselines-a87a858a` root of the same directory.

The device was CPU, not Metal. Metal is about 2× slower on the L2 read, which has no Metal kernel and runs through host copies, and it was no faster for the other arms on this loaded machine. Context 512 was chosen over 1152 so that every arm gets the same 1800 steps inside 20 minutes: at 1152 the L2 arm takes about 4 s per step.

### Recall accuracy, fresh pairings (correct / 512 per bucket)

| arm | pattern / read | params | d16 | d64 | d200 | d400 | all | positions scored / query token | wall |
|---|---|---|---|---|---|---|---|---|---|
| A | `aaaaaa` / l2 | 1,360,584 | **0.807** (413) | 0.176 (90) | 0.010 (5) | 0.029 (15) | 0.255 | 8,564 | 1064 s |
| A-dot | `aaaaaa` / dot | 1,360,536 | **0.777** (398) | 0.123 (63) | 0.014 (7) | 0.012 (6) | 0.231 | 8,564 | 938 s |
| A-flat-age | `aaaaaa` / l2, age bias zeroed at init | 1,360,584 | 0.115 (59) | 0.027 (14) | 0.010 (5) | 0.010 (5) | 0.041 | 8,564 | 1035 s |
| B | `rrarra` / l2 | 1,370,008 | 0.014 (7) | 0.008 (4) | 0.002 (1) | 0.006 (3) | 0.007 | 2,855 | 977 s |
| C | `rrrrrr` | 1,374,720 | 0.020 (10) | 0.010 (5) | 0.000 (0) | 0.006 (3) | 0.009 | 0 | 729 s |
| recency baseline (data only) | – | – | 0.445 | 0.051 | 0.000 | 0.000 | – | – | – |

Reference points for this table:
- Chance over the value range is 1/224 = 0.0045.
- The long bucket d1000 is **UNAVAILABLE** in this grid, because context 512 was used.
- The held-out-class evaluation scored 0/1024 for every arm. It is adversarial by construction. Training never sees pairing class `(k+v) % 4 == 0`, so a model that learns the class rule as a prior is penalized there even when it reads the context. All arms learned that prior: B and C sit at NLL 5.13 = ln 168, which is uniform over the class-admissible values. This result therefore measures prior-over-context, not recall, and the fresh-pairing table above is the recall measurement.
- An earlier run of arm A at commit `96e9cdb5` (`grid2-ctx512/A-aaaaaa-l2`) gave the same final numbers.
- Two interrupted roots are preserved unsealed: `grid1-ctx512/A-aaaaaa-l2` and `grid2-ctx512/A-aaaaaa-dot`.

**Wrong answers for arm A (l2):**
- Most wrong answers are another pair's value from the same window: 97/99 at d16, 385/422 at d64, 458/507 at d200 and 431/497 at d400.
- About 25–30% of the wrong answers are the most recently written value: 30, 96, 119 and 139 in the same bucket order. A uniform choice among the other 31 window values would give about 3%.

**Learning curves (in-class accuracy on 8 sequences, d16 / d64 / d200 / d400):**
- Arm A rose to 0.81 / 0.12 / 0.00 / 0.00 at step 1200 and reached 0.83 / 0.16 / 0.02 / 0.02 at step 1800. Short-range recall saturated; d64 was still creeping up; d200 and d400 stayed flat.
- Arm B stayed at chance for the whole run.

## Interpretation

These results are measured at this scale: about 1.4M parameters, 1800 × 8 windows and context 512. They are not proofs.

1. **Arm A fails beyond about 16 tokens, so the read mechanism as configured is the limit, and the defect is distance-dependent.**
   - At d16, A beats the recency baseline (0.81 against 0.45), so the reads do match keys at short range.
   - Recall then collapses with distance: 0.18 at d64 and about 0.01–0.03 at d200 and d400. When it fails, the read lands on another pair's value, disproportionately the most recent one.
   - **Score scale:** not the defect. Dot reproduces L2 within noise (0.78 / 0.12 / 0.01 / 0.01), so neither the L2 distance form nor its learned β/offset is what fails.
   - **Position handling:** the learned age table is load-bearing. Zeroing it at initialization removes most recall, even at d16 (0.115). Without the per-distance age bias, the stack has no other position signal: the reads carry no RoPE, and an all-read stack has no recurrence. Previous-token binding (key at `p` to value at `p+1`) then becomes hard to learn. With the default ALiBi-style slopes (2^(-8(h+1)/H); for H = 4 at d = 200 that gives −50, −12.5, −3.1 and −0.8), only one head of four can see d200 at all at initialization.
   - **Softmax sharpness against distractors:** the content score must single out one key among about 400 earlier positions. That needs a margin of several nats over every distractor plus the age penalty. The misses suggest the learned scores never reach that margin at long range; this is a hypothesis.
   - **Value path:** the d16 accuracy shows the value path can carry the answer once the right position wins.
   - The decisive next probe, not run here, would log each read head's softmax mass on the true key position by distance. It would separate a missing match (no mass on the key) from a match that is too flat (mass present but below the distractors).
2. **Arm B fails where A partially succeeds, so in this budget the recurrence crowds out the reads.** With two reads among four recurrence layers, B learned nothing from context: it sits exactly at the class-prior NLL and at chance, the same as pure recurrence C. This matches the D19 observation that with the sieve off the network reaches 0–5/109, although this bench is not the D19 panel.
3. **The chat model's failure is not only a curriculum issue.** At this scale the all-read stack cannot retrieve beyond short range, and the production mix cannot retrieve at all. A longer schedule could still raise A's d64 accuracy, which was rising slowly. It is not shown to rescue d200 or d400, which stayed flat after step 600.

## Proposed next arm: φ-phase exact-address read (design note only; not built)

**Measured motivation.** The stack's reads lose recall with distance, because one key must win a softmax over every earlier position against a recency prior. The external exact log sieve reaches 105/109 because admission is exact and independent of distance. This proposal keeps that exactness and makes the ranking learned and geometric.

**Ownership, as agreed in #820:**
- Claude owns phase storage and summaries, plus the learned ranking.
- Codex owns candidate admission and routing: prime/n-let identity and exact postings.

**Phases.** At query position `t`, the prefix `[0, t)` is partitioned by the Zeckendorf representation of `t`: `t = F_{k1} + F_{k2} + …` with non-consecutive Fibonacci numbers. That gives O(log_φ t) phases, oldest first and largest first, so recent context sits in short phases. Phase boundaries come from integer Fibonacci arithmetic, using `φ^k = F_k φ + F_{k−1}` in exact `Z[φ]`, with no floating point. When `t` advances, phases merge exactly as Zeckendorf carries do, the Fibonacci analogue of a Fenwick tree. A phase's identity is `(k, start)`.

Each phase holds two things:
- **(a) An exact index (Codex).** Postings from content identity to positions inside the phase, using `token_prime` and ordered n-let products. This is membership only: a prime or hash identity is never a distance.
- **(b) A geometric S³ summary (Claude).** A phase-local quaternion summary of the stack state over the phase, as a normalized quaternion product or mean with the retained fiber kept explicitly. It is used only for gated soft context and never as a candidate address.

**Read.** For the query token at `t`:
1. Admission returns the admitted phases and the exact positions in them.
2. The read descends into the admitted phases only.
3. A learned geometric score ranks the admitted positions plus a NoRead slot. The score is L2 or Lorentz over q/k, with an age feature measured within the phase, not a global recency slope.
4. The values of the winning positions are mixed as in the current fused read.
5. Phases with no admission contribute only through their gated S³ summary.

Admission and ranking stay separate. Scoring touches |admitted| positions instead of `t + 1`, which `context_access` records directly.

**Admission interface the read would consume (Codex implements):**

```rust
/// Exact, deterministic, parameter-free candidate admission over a causal prefix.
pub trait CandidateAdmission {
    /// Streaming update: the token at `position` joins the prefix (positions arrive in order).
    fn observe(&mut self, position: usize, token: u32) -> Result<(), AdmissionError>;
    /// Admitted candidates for the query token at `query_position`, using only
    /// positions `< query_position` already observed.
    fn admit(&self, query_position: usize, query: &AdmissionQuery) -> Result<Admission, AdmissionError>;
}
pub struct AdmissionQuery { pub tokens: Vec<u32> /* the query's n-let, ending at query_position */ }
pub struct Admission {
    /// Phases with at least one exact posting match: (fibonacci index k, phase start).
    pub phases: Vec<(u32, usize)>,
    /// Exact admitted positions, each strictly before query_position, ascending.
    pub positions: Vec<usize>,
    /// Which exact identity matched each position (prime / n-let key), for audit; not a score.
    pub matched_keys: Vec<u64>,
    /// True when a posting cap dropped candidates (then the read must not claim absence).
    pub truncated: bool,
}
```

The arm would join this bench as `ArmSpec::PhaseRead { admission, ranker, ... }`, with no other change to the task, the loop or the scoring.

**How to make the test decisive.** In this MQAR the key is a single exact token, so exact admission alone would solve it, as the sieve does. The phase-read arm must therefore report two things separately:
- admission recall: whether the true key position is admitted;
- ranking accuracy among admitted candidates.

It should also be run with decoys: K non-matching admitted positions per query, or multi-token n-let keys that share prefixes. That shows the learned ranking does the work, not only the index. A matched control should admit the same number of random positions.

The hypothesis is that d200 and d400 recall approaches d16 at a fraction of the positions scored. That is unmeasured.

## Cost

- Agent wall time from start to this PR: about 2 h (15:36–17:35 ET).
- Model compute, all on CPU at 4 threads:
  - grid3: 4,743 s;
  - an earlier grid2 A-l2 run: 985 s;
  - interrupted grid1 and grid2 partial runs: about 7 min;
  - smoke runs: about 3 min;
  - release and test builds: about 12 min.
- Peak RSS was under 1 GB per run.
- New storage is 832 KB of report roots, plus build artifacts in the existing `~/.cache/uor-claude-target`.
- No GPU or external compute was used.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5984631417 — 2026-10-04T21:35:09Z — Casey-allard

Exact-head review at a87a858a114ff52efc3cae3af7b26ef2271241c1: **approved**.
- **Scope:** one new example plus its tests. No existing model code changes.
- **Checks:** `cargo fmt`, `cargo check --example mqar-bench` and 5 focused tests pass at this head, including the answer-distance and fresh-pairing tests.
- **Reports:** sealed under `grid3-ctx512-eba6a15f`.
- **Arms:** the arm registry is pluggable, as Codex asked.

Queuing. The bench is the shared measuring stick, so it should be on main before the follow-up arms.


---
_Comment/review ids in this file: 5984631417_

