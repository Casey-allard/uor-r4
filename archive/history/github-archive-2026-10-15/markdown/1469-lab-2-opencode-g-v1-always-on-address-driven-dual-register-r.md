# #1469 — Lab 2 (OpenCode): G v1 always-on address-driven dual-register read

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-28T19:29:25Z
- **Updated:** 2026-10-09T14:17:00Z
- **Closed:** 2026-09-28T23:03:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1469
- **Merged:** 2026-09-28T23:03:23Z
- **Merge commit:** 49feed4826777d83bd9baee18c8aca04611b6ed1
- **Base / Head:** main / UOR-Foundation:lab/opencode/g-v1
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab 2 (OpenCode). Class B** (measured result). References #973 and #820.

## What this is

G v1: the always-on, address-driven dual-register read for the exact store, replacing D2's trigger-gated read. Pre-registered on [#973 (comment 5876711862)](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5876711862) **before any fit**. D3's disposition in light of D6 is recorded in the [second comment](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5876712272).

**The blocker (D2).** Exact version-ordered memory works with a gold register; writes transfer to unseen phrasings; but the learned **read** trigger fires 0 of 826 gold reads on held-out query phrasing (seeds 1–2), giving held-out Updated accuracy 0.000/0.000/0.015 against 1.000 with the gold register.

**The causal change (one policy change).** A read is no longer a learned per-token fire decision. Whenever the tag stream yields `TAG_ENTITY` or `TAG_RELATION` and both are defined, the store is read twice at that position — current and previous — and two registers update. Reads never close the clause; writes (`TRIGGER_WRITE`, still learned) and EOS close it. The trigger head, gold labels and loss are unchanged; only `simulate`'s read policy changed. The memory branch carries a full sub-branch (status embedding, value projection, copy boost) per register. The store, its versions/statuses, the world, data, schedule, seeds, evaluation and control are D2's.

## Files

- `crates/uor-r4-training/src/stack_aerm.rs` — read policy, dual registers, both-register re-entry, traces, tests.
- `crates/uor-r4-training/examples/aerm-probe.rs` — schema `uor-r4.g1-aerm-probe/1`, `read_policy` stamp, `summarize gates=g1|d2`.

## Checks run (commands and outcomes)

- `cargo fmt -p uor-r4-training --check` → pass.
- `cargo check -p uor-r4-training --offline --release` and `--examples` → pass (two pre-existing `uor-r4-core` dead-code warnings, unchanged).
- `cargo test -p uor-r4-training --release --offline stack_aerm` → **11 passed, 0 failed**. New locks: reads fire with **no** read trigger; two different trailing phrasings give the same read (the D2 regression); the previous register follows updates and same-value reassertions; reads do not close the clause; untagged text is register-silent; a training step reaches every new memory parameter; the dual branch starts silent.
- Sealed 20-update smoke (engineering only, not a result) at `/Volumes/UOR-Workspace/uor-r4-lab/opencode-g1-gread/smoke-1`; both summarize gate sets mechanically verified on it.

## Frozen run (in progress)

Launched 2026-09-28T19:27:54Z under the shared model slot, executable SHA-256 `71dbdd6d…`, source `d8a11080`. Three processes (seeds 1–3, both arms), `RAYON_NUM_THREADS=2`, caps 3 GiB RSS / 9,000 s wall per process; roots under `/Volumes/UOR-Workspace/uor-r4-lab/opencode-g1-gread/`; supervision receipt `g1.supervision.json`; gate applied by `aerm-probe summarize … gates=g1`.

## Merge conditions (Class B)

- [x] Focused local checks listed above.
- [ ] Frozen run complete with sealed roots.
- [ ] One non-author technical review of the change.
- [ ] Headline numbers re-read from the sealed roots by a non-author.

Draft until the run and the non-author re-read complete.


## Timeline (7 entries; chronological: conversation comments, reviews, review comments)

### comment 5877174864 — 2026-09-28T19:42:29Z — Casey-allard

## Non-author technical review (class A condition)

Independent auditor pass `uor-evidence-auditor` (session `ses_f16814775ffey2mN7u2rZoFwNe`), which did not write this change. The reviewer had no shell tool, so it obtained the diff from the GitHub compare API (`2a7a0dec...d8a11080`) and read the files and sealed smoke roots directly; it flagged the untestable claim and asked the lead to run two checks. Both are now run by the lead and recorded here.

**Lead-run follow-ups requested by the reviewer**

- `cargo test -p uor-r4-training --release --offline stack_aerm` → **11 passed / 0 failed** (confirmed twice: before the review and after the revert check below).
- Scratch revert check (per the reviewer's recommendation): a temporary edit restoring the trigger gate in `simulate` (`&& matches!(trigger, TRIGGER_READ | TRIGGER_READ_PREVIOUS)`) makes **5 tests fail** — `an_address_tag_reads_without_any_read_trigger` (0 read events vs 4), `different_trailing_phrases_do_not_change_the_read`, `the_previous_register_follows_updates_and_reassertions` (`(0,0)` vs `(1,31)`), `episodes_label_their_slots_and_answer_from_the_gold_store`, and `a_training_step_reaches_every_memory_parameter` (`projection receives no gradient`). The file was then restored from the commit (`git status` clean, `cargo fmt --check` pass). The tests therefore do lock the change.

---

**Reviewer's report (verbatim)**

Independent evidence review — G v1 (always-on address-driven dual-register read)

Scope: commit d8a11080 on lab/opencode/g-v1 vs base 2a7a0dec (draft PR #1469), against the pre-registration in issue #973 comment 5876711862 (fetched via the GitHub API, posted 2026-09-28T19:11:34Z, before the commit's 19:25:43Z timestamp — registration genuinely precedes the code). Read-only; no file modified, no build run.

Environment limitation (material): no shell tool, so `git diff`/`gh`/`cargo` could not run locally. Diff obtained from the GitHub compare API and cross-checked against the worktree files. The claimed 11/11 is UNVERIFIED by the reviewer; the revert check NOT_RUN (reasoned from source).

Q1 — Does the diff change exactly what the pre-registration says? PASS
- The compare API reports exactly two files changed: `stack_aerm.rs` (+480/−97) and `aerm-probe.rs` (+116/−45). Every hunk is within the registered scope.
- Read policy: `TRIGGER_READ`/`TRIGGER_READ_PREVIOUS` are now no-ops that never close the clause; reads fire whenever a `TAG_ENTITY`/`TAG_RELATION` token completes a defined address, performing both reads. Exactly mechanism item 1.
- Dual registers: a full second sub-branch, zero-initialised, both re-entering and both copy-boosting. Exactly mechanism item 2.
- Trigger head/loss unchanged: gold read labels still generated; loss untouched except the `top()` call-site signature.
- No unintended changes: no hunks in `RelationWorld`/`EpisodeBuilder` (world/data generation), `schedule`, `text_batch`, the loss composition, the tokenizer, or store `write`/`read`/`candidates`. `parameter_counts`/`optimizer_groups` change only to include the five new variables; `trace()` widens to 4-tuples (a diagnostic consequence).
- Extra but registered: the `read_events` counter was pre-registered as a reported read diagnostic.

Q2 — Simulation semantics correct and self-consistent? PASS, with one note
- Clause lifecycle: only `TRIGGER_WRITE` (complete clause) and `eos` close; reads never close — verified by the write-after-read assertion.
- Read timing: fires after the token's tag is applied, i.e. at the token that completes the address — the surface independence the registration requires.
- Register persistence: hold until the next read; not reset at turn close; pushed after close handling.
- Previous-distinct value, same-value reassertion, and eviction statuses behave as specified; evicted keys are never aliased.
- EOS closes; replies are all `TAG_OTHER`, so the later close (at reply EOS instead of the query's read trigger) cannot produce spurious reads.
- Note (not a defect): the trigger used to close at the query's last user token; now the clause stays open through the reply and closes at its EOS. Equivalent here because replies carry no tags, but it is a real registered semantic change, consistently applied to gold and model simulation.

Q3 — Do the tests lock the D2 regression; would they fail on revert? PASS by construction (execution now done by the lead — see above)
- 11 tests in the module, matching the claimed count. The four new tests assert the right invariants; a full revert would not compile (tests reference `status_previous`), and a policy-only revert fails the assertions.

Q4 — Does `summarize gates=g1` mechanically apply Lab 1's five items; does `gates=d2` reproduce D2? PASS
- Item 1 uses `held_out` (not D2's in-distribution metric) with `≥ 0.90`, folded per seed. Item 2 uses `|a−c|/max(a,c) ≤ 0.0005`. Item 3 uses `text_delta ≤ 0.05`. Item 4 reports `trace`/`read_events` (reported, not gated). Item 5 calls `report_output::verify` per root and records manifest hashes.
- No gate drift: the g1 set contains exactly items 1–3 as pass conditions and demotes the D2 margin gate to reported-only, as registered. The `gates=d2` branch is the original code moved verbatim and reproduces D2's gates on the same root.

Findings
- Blockers: none. Should-fix: none.
- Notes: (1) the 20-update smoke shows `read_events: 0`, so it does not witness an eval-time read — the lock rests on the unit tests, and the frozen runs must show nonzero `read_events`. (2) The parameter tolerance uses `max(a,c)` for the ambiguous "of the total" (immaterial at 0.018 %). (3) Test (f) does not check `copy_weight`/`copy_bias` gradients. (4) The packet's "Standing merge and review criteria" section reference did not resolve at the reviewer's commit; it reviewed against the pre-registration and general policy instead.

Recommendation: Approve for class B merge, conditional on the lead running the two cheap executions above. Both are now run and recorded in this comment.


### comment 5880199564 — 2026-09-28T22:54:34Z — Casey-allard

### Acting-lead independent closeout started

Casey has delegated temporary integration leadership (#973 comment 5880102513). All six run JSONs and the three sealed roots now exist; original worker/supervisor PIDs are no longer live at this inspection. I verified that the frozen summarizer executable still has SHA256 `71dbdd6de07a8715e706a507bc54d299ff2872668a49df59bb3aa8e87efb0b43`.

I am re-reading/verifying the completed roots and invoking the existing `gates=g1` summary into my own fresh root, with no fit, inference, changed criterion or write beneath the original roots. This is light postprocessing of 278,245 report bytes, <=1 thread, <=64MiB RSS, <=60s and <=2MiB new output, charged to this review. Original source d8a11080 and all experiment conditions stay unchanged. Please retain authorship of any result-document update and avoid moving this PR head without posting its identity; a scoped negative may merge without model promotion once the actual class-B conditions are complete.


### comment 5880210257 — 2026-09-28T22:55:31Z — Casey-allard

## Frozen run complete — gate FAIL (recorded negative)

Six runs, three seeds, both arms; roots `g1-s{1,2,3}` sealed with manifests and no `error.json`; gate applied by `aerm-probe summarize … gates=g1` into `g1-summary`.

| Seed | Memory held-out Updated | Control held-out Updated | Text Δ | Params (memory/control) |
|---:|---:|---:|---:|---:|
| 1 | **0.1575** (43/273) | 0.4652 (127/273) | −0.0162 | 1,428,816 / 1,428,556 |
| 2 | **0.0733** (20/273) | 0.7729 (211/273) | −0.0118 | idem |
| 3 | **0.0293** (8/273) | 0.8132 (222/273) | −0.0335 | idem |

**Held-out pass: false every seed. Text pass: true. Parameter parity: true (0.018 %). Outcome: FAIL.**

- `read_events` 2,294 / 2,574 / 2,706 — the always-on read fires (D2's trigger fired ~0); `NotSelected` falls, and the dominant surviving failure becomes **`Unavailable`** (store/key divergence).
- In-distribution the memory arm is 1.000 on every class with free-running 32/32; the control is 0.90/0.51 with 14–24/32.
- Result record: `docs/integration/g1-always-on-read-result-2026-09-28.md`; evidence: `docs/evidence/g1-always-on-read-2026-09-28.json` (headline numbers read from the roots, never typed).
- Supervision receipt `g1.supervision.json` documents the wall-guard extensions (comments 5878926721, 5879945647) and the monitor handover; no guard kill occurred.

## Class-B conditions

- [x] Focused local checks (listed in the PR body; 11/11 tests; revert check).
- [x] Frozen run complete, sealed roots with manifest hashes.
- [x] Non-author review approved ([comment](https://github.com/UOR-Foundation/uor-r4/pull/1469#issuecomment-5877174864)).
- [ ] Headline numbers re-read from the sealed roots by a non-author — in progress; the PR stays draft until it returns.


### comment 5880232416 — 2026-09-28T22:57:31Z — Casey-allard

## Independent headline re-read (class-B condition) — **headline numbers confirmed**

Non-author pass `uor-evidence-auditor` (session `ses_f15c53b07ffebJTEhxrMCdNcfM`), read-only, no shell; it derived every figure from the sealed roots.

| Seed | Arm | held-out Updated | text dev_nll | params | read_events | trace |
|---|---|---|---|---|---|---|
| 1 | aerm | 43/273 = 0.157509 | 2.373827 | 1,428,816 | 2,294 | {Emission 1, NotSelected 175, Unavailable 460} |
| 1 | control | 127/273 = 0.465201 | 2.390003 | 1,428,556 | 2,692 | {} |
| 2 | aerm | 20/273 = 0.073260 | 2.384032 | 1,428,816 | 2,574 | {Emission 15, NotSelected 172, Unavailable 484} |
| 2 | control | 211/273 = 0.772894 | 2.395840 | 1,428,556 | 3,332 | {} |
| 3 | aerm | 8/273 = 0.029304 | 2.364786 | 1,428,816 | 2,706 | {Emission 20, NotSelected 59, Unavailable 519} |
| 3 | control | 222/273 = 0.813187 | 2.398285 | 1,428,556 | 2,866 | {} |

- **Gate summary** `g1-summary/summary.json`: schema, `outcome: FAIL`, gates, and every per-seed row match the run files. Gate mechanics: held-out ≥ 0.90 fails all three; text Δ ≤ 0.05 passes; parameter relative difference 260/1,428,556 = 0.000182 passes. **FAIL mechanically supported.**
- **Root integrity:** each root has `manifest.json`, `summary.json`, `attempt.json`, both run files, and no `error.json`. Manifest hashes are consistent across the summary, supervision and evidence documents; I have now recomputed them byte-for-byte and they match: `8f918ac0…`, `9861149b…`, `55730477…` (the re-reader's one NOT_RUN is closed).
- **Supervision:** per-seed `train_seconds + eval_seconds` sums (12,305.33 / 12,306.42 / 12,315.29) match `per_process_seconds` (12,305.3 / 12,306.5 / 12,315.3).
- **Verdict: headline numbers confirmed.** No mismatch, no missing key affecting a claim. Two cosmetic fixes it raised are applied in the record: `WrongValue` is stated as absent (no key) rather than 0, and the control's in-distribution figures are labelled seed 1 with the seed-2/3 values given.

All four class-B conditions now hold: checks, frozen sealed roots, non-author review, and non-author headline re-read.


### comment 5880235111 — 2026-09-28T22:57:45Z — Casey-allard

## Independent result re-read COMPLETE at 9d29e5dc; two bounded text corrections

The source has not changed since reviewed/executed `d8a11080`; `9d29e5dc` adds only the result Markdown and evidence JSON. My fresh `gates=g1` summary verifies all three sealed roots, all six runs are complete, and the committed gate settings, all per-seed gate fields, held-out Updated counts, traces and root hashes match my independent re-read exactly. Receipt: `/Volumes/UOR-Workspace/uor-r4-lab/astra-acting-lead-20260928-2245/g1-committed-evidence-check.json` and fresh sealed `g1-nonauthor-summary/`.

**Gate disposition is settled:** FAIL / not yet promoted at this scope. Memory held-out Updated 43/273, 20/273, 8/273; control 127/273, 211/273, 222/273. Text and parameter checks pass. This is reusable source and negative evidence, not permission to retire exact memory or geometric addressing.

Please make just these text corrections, then I will recheck the new head and take the protected merge path; no rerun or new review campaign:
1. In the in-distribution paragraph, label the five control counts as **seed 1** or give all three seeds. They are not constant: e.g. Updated is 308/307/308, Previous 88/97/95 and First 71/71/75. The gate table and evidence are already correct.
2. Replace “always-on reading is necessary for reads to fire” with the exact observation: “the policy removes the learned read-trigger condition; reads fired, but held-out generalization remained poor.” It is not logically necessary that all valid readers be always-on.

**Next-work constraint:** this runner saved no trained weights. Diagnose only what the retained traces actually expose; do not describe a replay of the same trained models unless the artifacts exist. If the needed write/key/eviction trace was not retained, say so and make checkpoint/export plus that instrumentation part of the next justified, prospectively bounded experiment. No automatic refit just to recreate a lost probe. This protects the shared-model goal without rewriting this completed study.


### comment 5880254240 — 2026-09-28T22:59:32Z — Casey-allard

Acting-lead delivery: the count-label correction is present at 8b729541. I am applying the remaining one-sentence necessity wording correction through a normal contents commit on this PR branch, guarded by the current head and file SHA. No source or experiment changes; no rewrite of your local checkout or branch history. Any concurrent head/file movement stops the correction or requires a fresh review before merge.

### comment 5880287835 — 2026-09-28T23:02:32Z — Casey-allard

## Acting-lead acceptance for protected delivery at d467977d

All existing class-A/B conditions are now satisfied for **source and scoped negative research evidence**, not model promotion. The original non-author technical review, author-run 11-test/revert checks, and my independent execution of the frozen summarizer apply to unchanged experiment source `d8a11080`. The committed gate/evidence fields match all six sealed runs. The result-document corrections have landed: seed-labelled control counts, absent trace key not fabricated as a measurement, D12 non-promotion scope, and removal of the unsupported logical-necessity claim. The final commit changes only that one sentence.

The independently verified frozen outcome remains FAIL in all three seeds for held-out Updated, while text and parameter checks pass. Exact memory and geometric addressing remain active mechanisms under D12; this probe is not adopted by the main dialogue model. No trained-weight checkpoint is inferred from its JSON reports.

I am taking the existing protected merge path at **d467977de48183f141d0cc7ae397fb5ab126f78c**, then verifying the delivered diff. This closes the PR's pending verification rather than restarting the experiment. Lab 2 retains the next source/trace-limited diagnostic, and must explicitly account for the absence of trained weights before suggesting any replay or new fit. Original supervision limitations and failed gates remain on record.



---
_Comment/review ids in this file: 5877174864, 5880199564, 5880210257, 5880232416, 5880235111, 5880254240, 5880287835_

