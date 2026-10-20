# Trajectory-state gate and the S3 free-running generation verdict (#842)

- **Status:** Frozen trajectory-state diagnostic + executed gate for S3 item C
  (#842, tracker #824, programme #820). This record is the authoritative S3
  free-running generation verdict. Companion to the #841 gap contract
  ([`docs/free_running_eval_841.md`](free_running_eval_841.md)) and the #840
  reachability instrument
  ([`docs/free_running_reachability_840.md`](free_running_reachability_840.md)),
  whose evidence it consumes.
- **Date:** 2026-08-22.
- **Claim language:** follows [`docs/formal_vocabulary.md`](formal_vocabulary.md)
  (normative). This record establishes that no bounded trajectory-state mechanism
  is triggered at this representation and records the S3 generation claim as not
  established. It does **not** change serving behavior or add any runtime mechanism.
- **Harness:** `crates/uor-r4-api/tests/state_trajectory_gate_842.rs` (fixture
  teeth non-ignored; `state_trajectory_gate_run_842` ignored/bundle-gated). Record:
  `docs/state_trajectory_gate_842_result.json` (CID-bound, `result_cid
  blake3:86376394…`).
- **Execution scope (#830):** offline `certifier-instrument` over the deployed
  `R4Engine` generation path; teacher-free (scored against the bundle's recorded
  `t_argmax`). Reference / offline evidence is not credited as deployed generation.

## 1. The diagnostic question

#842 must decide whether a bounded trajectory-state mechanism (topic / entity /
anti-cycle memory) is warranted, and then issue the S3 generation verdict. The
Problem it guards against: adding such state without a binding diagnostic could
improve surface diversity while leaving semantic coherence unchanged and needlessly
expanding the trusted runtime. So the gate first isolates **state starvation** — a
failure bounded state could fix — from **evidence / candidate / decoder** failures,
which added state cannot fix.

## 2. The cheap diagnostic (predeclared, non-vacuous, reproducible)

For each prompt of the frozen #841 prompt-family v1 (n = 100, family CID
`blake3:6cad1dfe…`), at H = 32 under greedy on the deployed `R4Engine`, the
instrument pairs the **student-prefix** rollout against the **teacher-prefix**
rollout and probes the deployed candidate list at the student's first-divergence
step. Each prompt's failure is classified as exactly one of:

- `Survived` — the student stayed on-text the full horizon.
- `SingleStepAt0` — diverged at step 0. There is no prior trajectory at step 0
  (student and teacher share the prompt window), so trajectory state is
  definitionally irrelevant.
- `CandidateGap` — diverged at step d ≥ 1 but the recorded token was not among the
  deployed candidates; re-ranking cannot reach a token the scorer never proposed.
- `RankLimit` — diverged at d ≥ 1, recorded token was a candidate, but the
  teacher-prefix diverges at the same-or-earlier step: a context-**independent**
  rank / decoder limit the perfect context does not fix either.
- `StateStarvation` — diverged at d ≥ 1, recorded token was a candidate, AND the
  teacher-prefix survives strictly longer: the ONLY failure a bounded trajectory-
  state mechanism could address (the student's own drifted context caused the early
  loss).

**Guarantee (the classifier distinguishes every cause). Status: Structural**
(`classifier_distinguishes_every_cause`, `reachability_bar_can_report_both_verdicts`,
`at0_reachable_drop_is_zero_under_step0_invariance`): the pure classifier and the §6
ceiling test are exercised on planted cases of every category, and the ceiling test
reports `TRIGGERED` on a planted clearing shape (median 0 → 3, at0 590 → 400‰) — the
instrument is not vacuously negative and can fail.

## 3. Reachability arithmetic (two representation-independent bounds)

The frozen #841 §6 bar requires, to count as an improvement: median first-divergence
+ ≥ 2 steps AND diverged-at-step-0 − ≥ 100‰ (no teacher-forced regression > 10‰). Two
bounds cap what any trajectory-state mechanism can do against it:

1. **Step-0 invariance.** `diverged-at-step-0` counts step-0 failures; at step 0 the
   student and teacher share the prompt window, so no state mechanism changes step 0.
   The reachable at0-drop is **0‰**. The measured step-0 fraction is 590‰ > 500‰, so
   the median first-divergence is pinned at 0 regardless of any later gain.
2. **Teacher-prefix upper bound.** The drift-free reference state — perfect context
   reconstruction — is the teacher-prefix trajectory; it is the ceiling of any state
   mechanism on the student side. It sits at median 0 and diverged-at-0 590‰ —
   clearing neither §6 prong.

## 4. Result (2026-08-22; greedy; CID-bound record)

**Empirical Criterion. Status: Empirical.**
`docs/state_trajectory_gate_842_result.json` (`result_cid blake3:86376394…`;
corpus.meta CID `blake3:aa9d1767…`, the attested #833 broad-clean bundle; graph CID
`blake3:bc2366f1…`).

| metric (H = 32, greedy) | student-prefix | teacher-prefix (reference) |
|---|---|---|
| median first-divergence | 0 | 0 |
| diverged-at-step-0 | 590‰ | 590‰ |
| survived-full-horizon | 0‰ | 0‰ |
| ≤ 4-period cycle collapse | 710‰ | 110‰ |
| mean distinct-1 | 391‰ | 643‰ |
| per-step paths exct / ngram / graph | 0 / 3197 / 3 | 24 / 3112 / 64 |

Matched teacher-forced agreement is 304‰ (teacher-prefix side, over 3200 matched
steps); suffix-locality is 99/100 (student rollouts token-identical to the last-2-token
rollout). Failure diagnosis (H = 32, of 100 prompts): **survived 0 · single-step-at-0
59 · candidate-gap 16 · rank-limit 25 · state-starvation 0**.

- **The instrument reproduces #841.** Loading the deployed graph directly, the
  student-prefix numbers match `docs/free_running_841_result.json` exactly (304‰ TF,
  590‰ diverged-at-0, median 0, 99/100 suffix-local, path histogram 0/3197/3) — the
  diagnostic is validated against the frozen sibling-A record.
- **No failure is state-addressable.** 0/100 prompts are `StateStarvation`. Every
  divergence is a step-0 single-step limit (59), a candidate gap (16), or a
  context-independent rank limit (25) — none is a failure bounded trajectory state
  could fix. The reference-state consistency checks hold on real data: the
  teacher-prefix survives no longer than the student on any prompt where the token was
  rankable (it fails at the same places), and it does not help the step-0 failures (it
  diverges at step 0 too).
- **Both §6 prongs are unreachable.** reachable at0-drop 0‰ vs the 100‰ bar; the
  reference ceiling leaves the median at 0 (need + 2). `ceiling_clears_bar = false`.

## 5. Decision — NOT TRIGGERED; GENERATION-NOT-ESTABLISHED

No bounded trajectory-state mechanism is triggered: the state-addressable fraction is
0‰, and even the drift-free reference does not clear the frozen §6 bar. Per the #842
non-goals no runtime code lands. Accordingly:

- **Trigger decision: `NOT TRIGGERED`.** No topic / entity / anti-cycle state lane is
  specified or lowered; no runtime, format, or compiler change is made.
- **S3 generation verdict: `GENERATION-NOT-ESTABLISHED`.** Free-running coherent
  generation is not established at this representation scope. This is consistent with
  #840 (representation is the limiter, not more data or decoding) and meets the
  programme global falsifier: bounded student-prefix correction does not reduce the
  frozen free-running gap, and no bounded state mechanism can, so the deployed artifact
  is a teacher-forced continuation / retrieval system, not a generative engine at this
  scope.

**Limitation and re-entry.** The verdict is scoped to prompt-family v1 (continuity
domain), greedy decoding, and the current representation. Sampled-mode and judge-bearing
families remain `UNAVAILABLE` (no decoder / seed / judge identity pinned). Any re-entry
requires a representation with cross-step free-running memory (a redesign-scope question,
echoing the S1 REVISE pattern) and must re-enter under the frozen #841 §6 bar.

## 6. Repository conformance

- **Execution scope:** offline certifier-instrument measured through the deployed
  `R4Engine` generation path; teacher-free. Not credited as deployed generation.
- **Conformance mapping:** RF-14 / RF-21 / RF-22 / RF-29 evidence language (extends
  existing capabilities). No new built capability, no `model/ids.toml` row, no
  `CONFORMANCE.md` regeneration — evaluation / instrument infrastructure, identical
  treatment to #841 and #840.
- Preserves P-4, allocation-free steady state, and determinism: no runtime, format, or
  compiler code is touched; the instrument only reads the deployed engine. Empirical
  criteria bind distribution, horizon, decoder, n, provenance, and PASS/FAIL/UNAVAILABLE.
- Appends this record and reconciles [`docs/RESEARCH.md`](RESEARCH.md). It does **not**
  rewrite the #841 or #840 records.

## 7. Claim status and next action

The S3 free-running generation claim is **not established** (`GENERATION-NOT-ESTABLISHED`),
and the bounded trajectory-state mechanism is **`NOT TRIGGERED`**. This record is the
authoritative S3 generation verdict for the #824 native child #842. Next: the S3 tracker
#824 records its stage verdict (`PROMOTE` / `REVISE` / `LIMIT` / `RETIRE`) against its
three completed children (#841 quantified, #840 not-launched, #842 not-triggered) — a
separate tracker action, not part of this issue.
