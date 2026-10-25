# Pre-registration — M1 W2 code-utilization diagnostic

## 1. Header

- **Date and author:** 2026-10-25, lab claude (unit claude-89). The same lab ran W2.
- **Item:** [`m1-w2-code-utilization`](../../roadmap/items/m1-w2-code-utilization.md) (#3179). It depends on `m1-w2-capacity-capture` (#2845), which is done and merged as `9ae329d4e`.
- **Why now:** W2 gave H_mixed, row 3 of [the W2 sheet](m1-read-code-capacity-2026-10-17.md)'s §4.7. Its next step is "Code-utilization/capacity diagnostic; do not change the learning law yet".
- **Scope:** a diagnostic only. It is not a qualification, not a capability claim and not a fresh-transfer result.
  - One checkpoint: seed-1 Q4 (`4477eb5c…`, epoch 407, `ReadSpec::Route`, K=8).
  - One panel: DEV, 14,287 rows, ordered binding `e4ef44fb…`.
  - The model-matched TRAIN stream (1,049,856 targets).
- **Compute:** host only. It counts and scores GPU observations already stored by W2. That is the W2 sheet's D24 class (§4.5 there), the "CPU audits stored GPU observations" pattern. No model forward, init or apply runs. The GPU arms that would need one are named in §7 and are not registered here.
- **Not changed by this sheet:**
  - W2's H_mixed band stands, and W4 stays untriggered.
  - No learning law, bar, status or gate changes.
  - Stage T's gate ("only warranted if W2 blames the learning law (H_law)", #2836) is a decision review's call, not this sheet's.

## 2. Question

The trained model's prediction is a function of its 16-bit read code alone: four root ids feed admission and the writer. So:
- a table keyed by the code estimates what any code-keyed output map can reach;
- W2's two bounds split the shortfall between admission and the output map;
- the M1 bar compares against the T2 table, 6,300 of 14,287, which is W2's G.

**Question.** At the W2 scope, where does the read lose the target information that a code-keyed output map would need? And does any readout of this read project to the M1 T2 bar? Three sub-questions, each decided by one paired comparison:
- **QA, admission.** Could a rule keyed on the code admit the target's cell more often than the writer's address key (1,920 rows)?
- **QB, headroom.** Does the code table's data-scaling trend reach W2's H_law line? If it does, the full-TRAIN capture is worth a GPU run.
- **QC, beyond the last token.** Does the code carry next-token information that the query token does not?

The utilization and coverage arms are descriptive and cannot fire a band.

## 3. Inputs and notation

- **Inputs.** HF store `caseyallard/uor-r4-store`:
  - `results/claude/w2-capacity-2026-10-25/run-b.tar` `6aa0a766…`, which holds:
    - `w2-out-b/train-stream.bin` `70a58767…` (4,199,424 B; per TRAIN row, u16 code then u16 target);
    - `w2-out-b/query-sidecar.bin` `6d5d4f5d…` (per TRAIN row, the u16 query token; written beside the stream, `w2_capture.rs` `QuerySidecarWriter::create(&dir)`);
    - `capture.json` `d22a4eb2…` (per DEV row: code, query, target, the 16 admitted-cell slots with `0xFFFF` absent, writer top-8 and hit flags; plus the 4,096 token cells).
  - `record.json` `69717971…`.
  - `w2-inputs/2026-10-25/` (`SHA256SUMS`).
  - Arm R only: the Q1–Q4 checkpoints `a5a22dce…`, `dde889aa…`, `102beed4…` and `4477eb5c…`.
- **DEV row r.** It has:
  - code c_r and query q_r;
  - target y_r;
  - admitted cell set S_r (the non-absent slots), with k_r = |S_r|;
  - writer top-1 w_r.
- **TRAIN row i,** for i = 0..1,049,855 in stream order, has code c_i, target y_i and query q_i.
- **Subsamples N_j, for j = 0..4.** N_j is the stride-2^j subsample: the rows with i mod 2^j = 0. Its sizes are 1,049,856; 524,928; 262,464; 131,232 and 65,616. A stride keeps every subsample spread over the whole stream, so the data-scaling curve is not confounded with any ordering of the corpus. The first-rows prefixes of the same sizes are reported only as descriptive figures.
- **Cells and counts.** cell(t) = token_cells[t], in 0..=255. n(κ, t) counts the TRAIN rows in the subsample that have key κ and target t, and n(κ) = Σ_t n(κ, t). An arm with no N named uses N_0, the full model-matched stream.
- **Tie and fallback rule (W2's).** Highest count wins, then the lowest token id, or the lowest cell id for cell rankings. An unseen key falls back as each arm names.
- **"X beats Y" on DEV (W2's paired floor, in squares).** Let b = rows where Y hits and X misses, and c = rows where X hits and Y misses. X beats Y iff c > b and (c − b)² > max(576, 4(b + c)). The floor applies even when the exact test of precedence 1 rejects, so a small, significant edge of 24 rows or fewer reads as "neither beats".

## 4. Arms

Every quantity is an exact integer or an exact integer ratio. Information in bits would need logarithms and float types, so the W2 sheet's "code–target mutual information" is measured in two exact forms: leave-one-out top-1 (the guessing form) and collision agreement (the order-2 form).

**Decisive arms**

| Arm | Exact definition |
|---|---|
| A_W | #{r : cell(y_r) ∈ S_r}. This is the writer's admission, and must equal 1,920. |
| A_T | For row r, rank cells by s_r(κ) = Σ_{t: cell(t)=κ} n(c_r, t) at N_0. Ties go to the unigram cell mass Σ_{t: cell(t)=κ} n(t), then to the lowest cell id. Take the top k_r. A hit is cell(y_r) among them. An unseen code ranks by unigram cell mass alone. |
| A_U | The top k_r cells by unigram cell mass: one fixed list. |
| T(N_j), G(N_j) | W2's code table and query-token bigram, built on subsample N_j, with the unigram of the same subsample as fallback. T(N_0) must equal 3,414 and G(N_0) 6,300. |
| d_T, T_proj | d_T = T(N_0) − T(N_1) if T(N_0) beats T(N_1), else 0. T_proj = T(N_0) + 2·d_T, that is, two more doublings. The full TRAIN is 2,425,925 targets, 2.31× N_0 or about 1.2 doublings, so the projection is generous. |
| J | The joint table keyed by (c_r, q_r) at N_0. A seen pair takes argmax_t n((c_r, q_r), t). An unseen pair takes G's prediction for q_r, which itself falls back to the unigram. |

**Descriptive arms** (no band)

| Arm | Exact definition |
|---|---|
| A_G | A_T with n(q_r, t) in place of n(c_r, t). |
| A_Ω | The DEV oracle: each DEV code admits its top k_r cells, ranked by DEV target count under that code, then unigram cell mass, then cell id. When k_r is constant within each code (expected k_r = 8 on every row), this is the ceiling for any code-keyed admission on this panel. |
| Ω_code, Ω_query, Ω_joint | The DEV oracle top-1: Σ over DEV keys of the largest target count. Each is reported with the number of single-row keys it includes. |
| T_adm, T_non | T's hits on rows where cell(y_r) ∈ S_r, and the rest. Exact counts, which replace W2's bound of ≥ 2,276. |
| Cell split | Rows where cell(w_r) = cell(y_r) against the writer's 149 hits, and the same for T. This splits the output map's loss into wrong cell and right cell, wrong token. |
| Candidate load | For each admission set X_r, the number of tokens in its cells, Σ_{κ ∈ X_r} #{t : cell(t) = κ}: minimum, quartiles and maximum over r. It is reported for S_r (the writer) and for the A_T, A_U and A_Ω sets, next to the QA reading. A mass-ranked set can admit more targets by admitting more candidates, and this shows how much more work it leaves the output map. |
| T12_k (k = 0..3) | The code with block k's root removed: 4,096 keys, the same count as the query token's. Unigram fallback. |
| T8_jk (6 pairs), Gc | The 8-bit block pairs: (0,3) must equal W2's address-only 1,675, and (1,2) its interior-only 1,628. Gc is the query-cell table keyed by cell(q_r), 256 keys. |
| J_m2 | J, but the joint key is used only when n((c_r, q_r)) ≥ 2. |
| Utilization | Distinct codes on TRAIN and on DEV; per-block root histograms (4 × 16); the collision effective count N_0² / Σ_c n(c)² as a fraction; query determinacy Σ_c max_q n(c, q). |
| Leave-one-out and resubstitution top-1 on TRAIN, for U, G, T and J | Resubstitution: row i counts if y_i = argmax_t n(κ_i, t), its own row included. Leave-one-out: row i counts if y_i = argmax_t (n(κ_i, t) − [t = y_i]). A key left with no count falls back, also leave-one-out. |
| Collision agreement, for the unigram, query, code and joint keys | Σ_κ Σ_t n(κ,t)(n(κ,t) − 1) / Σ_κ n(κ)(n(κ) − 1), as a numerator and a denominator. |
| R (route-learning coverage) | The four stored checkpoints are read as stored JSON and compared on the host. It counts the route trits and vector blocks that differ between Q1→Q2→Q3→Q4, and the DEV rows whose target token had any of its four blocks change between Q1 and Q4. Init→Q1 is not host-measurable, because init is the `discrete_init` CUDA kernel (see G2). |

## 5. Bars and decision table

Precedence 0 and 1 apply first. After that, QA, QB and QC are each read on their own, and within each question the bands are disjoint and exhaustive.

| P | Observation | Reading | Next |
|---|---|---|---|
| 0 | Any of: a pin mismatch; a recomputed W2 number differs from the record (T 3,414, G 6,300, U 1,226, W 149, A_W 1,920, restricted table 1,138, restricted bigram 1,482, coverage 14,175, codes ≥ 2 41,806, address-only 1,675, interior-only 1,628); a conservation failure (§6); T_adm > 1,138; a token ≥ 4,096 or a cell ≥ 256 | INSTRUMENT FAULT | Repair (D31). No band is read. |
| 1 | A decisive pair has b + c < 100 and the exact two-sided McNemar test does not reject at α = 0.05. For QC only, it also applies when fewer than 1,429 DEV rows (10%) have a seen joint key. QB is exempt: its d_T gate already reads a noise-level gain as d_T = 0, so an underpowered T(N_0)-against-T(N_1) pair gives B2. In QA, A_T against A_W is always decisive, while A_T against A_U is decisive only when A_T beats A_W. If that second pair is then underpowered, QA reads A1u, never UNDERPOWERED as a whole. | UNDERPOWERED for that question (QA or QC) | Report the counts. The other questions are still read. |
| A1a | A_T beats A_W, the A_T-against-A_U pair is powered, and A_T beats A_U | The code holds cell information beyond cell frequency that the writer's address key does not use. | `m1-address-key-full-vector` (#2834) is the next M1 mechanism unit. A_T is its zero-training reference and A_Ω its ceiling on this panel. |
| A1b | A_T beats A_W, the A_T-against-A_U pair is powered, and A_T does not beat A_U | The address key admits worse than frequency ranking. The code's own cell information is not shown. | #2834 is the next M1 mechanism unit, with A_U as the reference it must beat. |
| A1u | A_T beats A_W, and the A_T-against-A_U pair is underpowered (precedence 1) | The address key admits worse than code-keyed counts. Whether that gain goes beyond frequency is unread. | #2834 is the next M1 mechanism unit. A_T and A_U are both reported, and #2834 must beat the larger. |
| A2 | Neither A_T nor A_W beats the other | No code-keyed admission gain over the address key is detected at this floor. That is a failure to detect, not a showing of no gain. | Record this on #2834 as a bound on its lever. |
| A3 | A_W beats A_T | The trained address key admits better than code counts do. | Admission is not where the code's hits are lost. The output map and the read are the levers. |
| B1 | T_proj > 6,300 and (T_proj − 6,300)² > 18,288. That is W2's H_law line M + F ≈ 6,435, with F² = 4 · 4,572; in integers, T_proj ≥ 6,436. With T(N_0) = 3,414 this needs d_T ≥ 1,511, that is T(N_1) ≤ 1,903. | More TRAIN data could plausibly move T across the H_law line. | Register G1 (§7) under its own pre-run review before any learning-law reading. |
| B2 | Otherwise | Measured: at this read, the code table does not project to the H_law line with more data. B2 is the expected outcome. Its job is to decline G1 on a measured curve, not to add evidence beyond that curve. | G1 is declined. Repairs that keep the read fixed (admission, output map) are not expected to lift the writer past T's projection, which is below the T2 bar. That is a measurement, not a proof, unless the oracle flag below fires. Whether the read itself must change is the question of a decision card for the top session; B2 is one input to it. |
| C1 | J beats G | The code holds next-token information beyond the query token. | C1 is one input favouring a read that keeps both the code and the query token's identity. J is the reference number for that read-change card. |
| C2 | Neither beats the other | No gain from the code beyond the query token is measurable at N_0 at this floor. That is a failure to detect, not a showing of no gain. | Report J_m2 and the leave-one-out forms. A read-change card may cite C2 only as the absence of a detected gain. |
| C3 | G beats J | The joint key's sparsity costs more than the code adds. | Report J_m2 and the leave-one-out forms. No reading beyond sparsity. |

Two flags carry no band:
- **Address key versus a fixed list.** If A_U beats A_W, the trained address key admits fewer target cells than a fixed list of the most frequent cells. That goes to #2834's decision review as an address-objective finding, after the instrument-versus-mechanism classification (CHARTER §4.2).
- **The oracle ceiling.** If Ω_code ≤ 6,300, no function of this read can reach the T2 bar on this panel. That is recorded as a proof that strengthens B2.

**What this decides.**
- QA routes the next M1 mechanism unit: #2834 or the read.
- QB decides whether G1 spends GPU.
- QC says whether the read's 16 bits add anything to the last token.

QA, QB and QC are inputs a decision review of Stage T's gate would weigh; R is descriptive context only. This sheet does not change that gate.

## 6. Instrument checks

1. **Pins and sizes.** Every input is authenticated by sha256 before any count: the pins in §3 and `SHA256SUMS`. The TRAIN stream must be exactly 4 × 1,049,856 = 4,199,424 B and the query sidecar exactly 2 × 1,049,856 = 2,099,712 B, so a misjoined or truncated sidecar fails before any join.
2. **W2 reproduction.** Every W2 number in precedence 0 is recomputed exactly.
3. **Conservation.**
   - For T, G and U on each subsample, Σ n = |N_j|.
   - The joint total equals N_0, and the cell masses sum to N_0.
   - Every DEV row has k_r in 0..=16, and Σ k_r is reported (expected 114,296 = 14,287 × 8).
4. **Bounds.**
   - T_adm ≤ 1,138 and T − T_adm ≥ 2,276.
   - Each leave-one-out count is at most its resubstitution count.
   - T ≤ Ω_code, and A_T ≤ A_Ω when k_r is constant within each code.
5. **Determinism.** Two runs give byte-identical output.
6. **No model compute.** The tool builds without `--features cuda` and links no device code. Arm R reads checkpoints as stored JSON and never loads one into a `Device`.
7. **Banned presence and ratchets.** Banned presence and the D23, D25 and D32 ratchets pass on the new source. Integer types only, timers and tests included: this is the W2 run-A lesson.
8. **Arm R.** In each quarter, the changed parameters (one route trit or one vector block each) must be at most that quarter's increase in committed moves:
   - **When the cursor has no `block` field** (`coordinate_block = 1`, expected for this 15 Oct run, which predates BVG-K): the bound is the increase in `accepted_updates`, because each accepted update moves one parameter.
   - **When a `block` cursor is present:** the bound is the increase in its `committed_moves`, which `training.rs` keeps between `accepted_updates` and K × `accepted_updates`.
   - **On failure:** a missing or mismatched Q1–Q3 file, or a broken bound, makes R alone NOT_RUN. It never substitutes another input, and it never blocks QA, QB or QC.

Any other failure is an instrument fault. It is repaired; it is never a reading.

## 7. GPU arms named, not run

- **G1, the full-TRAIN capture.** The production forward over all 2,425,925 masked TRAIN targets: the W2 sheet's §4.4 item 5, which W2's fallback skipped. It runs only on B1, under its own pre-run review. W2's TRAIN pass took 480 s for N_0, so G1 is about 25 minutes on one RTX 5090 inside a 1-hour lease, about $1.11 at W2's $1.106/h. It would re-read W2's band at full TRAIN.
- **G2, the seed-1 init snapshot.** The `discrete_init` kernel only: no forward and no apply. It would extend R back to init. No band needs it; it rides on a G1 lease if G1 runs.
- **Read codes at Q1–Q3.** A forward at the earlier checkpoints would show how the code distribution moved with training. It is not registered.

## Base artifact

- base: the seed-1 Q4 checkpoint `4477eb5c…` (epoch 407, 407 accepted updates), as captured by W2 run B (`run-b.tar` `6aa0a766…`, record `69717971…`); for arm R, Q1–Q3 `a5a22dce…`, `dde889aa…` and `102beed4…`.

## Data split

- split: tables are built on the model-matched TRAIN stream and its stride subsamples N_j, and scored on the 14,287-row DEV panel. The leave-one-out arms are TRAIN-internal and descriptive. There is no fresh draw: this is a diagnostic, not a KEEP.

## Steps

- steps:
  1. Authenticate the inputs.
  2. Reproduce W2 (precedence 0).
  3. Build the tables on each subsample.
  4. Score every arm.
  5. Compute the paired (b, c), the floors and the bands.
  6. Compare the checkpoints for R.
  7. Seal the record (`uor-r4.w2-code-utilization/1`: the bands, every aggregate, (b, c) per pair, and per-row DEV hit flags) in the HF store under `results/claude/`.
  8. Write one notebook row in `m1-discrete-attention`.
  9. Register G1, or record it declined, per QB.

## Minimum test power

- seeds: not applicable. Nothing is trained, and the inputs are fixed stored observations.
- panel: 14,287 paired DEV rows. Every decisive comparison uses W2's paired floor in squares: (c − b)² > max(576, 4(b + c)), with no root computed. Below b + c = 100 the exact McNemar test applies too (precedence 1).
- QC coverage guard: at least 1,429 DEV rows with a seen joint key. The TRAIN leave-one-out arms (1,049,856 rows) are descriptive.
- multiplicity: each of QA, QB and QC routes a different decision, so each is read at its own floor, with no family-wide correction. The descriptive arms never fire a band, however many there are.

## Cost

- GPU: $0. Every registered arm is host-only.
- Host: the downloads (`run-b.tar` and, for R, the checkpoints) and one run of a Rust host tool taking minutes. Building the tool is one loop unit, reusing `w2_capacity.rs`'s tables, tie rules, floors and McNemar test. The record is under 10 MB. `df` must show at least 30 GiB free before the downloads.
- Conditional: G1, about $1.11 inside a 1-hour lease, only on B1 and only after its own pre-run review.

## Stop conditions

- stop: any precedence-0 check stops the run for repair, with no band read. The rules above are fixed before any number is seen, so a band is never re-read under changed rules. The GPU caps and the disk floor are hard stops. Self-set time budgets are estimates, not stop rules.

## Fitness review

TEST FITNESS: NOT_FIT (round 1, 2026-10-25). Fresh-context non-author reviewer: a claude subagent that had not seen the authoring, which CHARTER §4.5 counts as non-author. One blocker: instrument check 8 stated the arm R bound without the block-cursor case and let its failure stop every band. Seven should-fix items: split A1 against the fixed list A_U, report candidate load per admission set, state QB's integer threshold, exempt QB from precedence 1, reword C2 and B2 as failures to detect, keep R descriptive, and use stride subsamples with exact byte sizes. All are resolved in this text.

TEST FITNESS: FIT (round 2, 2026-10-25). The same non-author reviewer, after the revision: the bands of QA, QB and QC are each disjoint and exhaustive, with correct integer thresholds, and every arm is host-computable from the stored fields. The reviewer proposed three optional nits, applied after the verdict: the A2 and C1 wording, and an unsplit A1 reading under precedence 1. The fresh exact-head review at `776b2d8c1` then asked for two more changes, both now in: that reading gets its own row, A1u, and the A_T-against-A_U pair is decisive only when A_T beats A_W. Neither changes any threshold or any outcome's routing. This FIT clears host post-processing only. It clears no GPU arm; G1 needs its own pre-run review.
