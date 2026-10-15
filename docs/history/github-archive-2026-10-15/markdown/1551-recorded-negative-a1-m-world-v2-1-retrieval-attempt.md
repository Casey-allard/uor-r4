# #1551 — [Recorded negative] A1 M-world v2.1 retrieval attempt

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-30T20:57:15Z
- **Updated:** 2026-10-01T06:27:13Z
- **Closed:** 2026-10-01T06:27:13Z
- **Labels:** roadmap:historical
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1551
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

<!-- active-roadmap-2026-10-01 -->
**October 1 adopted roadmap — HISTORICAL.** PR #1564 / D19; adoption task #1563.

The attempted recipes have their final scoped negative and preserved roots on this issue and #1552; D18 outcome D stops further A1 training. This closes the run task, not attention research. Lorentz-scored pointer and full capacity/load coverage were not established by this attempt. Failure of both geometric and ordinary recipes leaves shared data/optimization/instrument explanations possible; it does not isolate one cause or refute a family. Successor #1552.

[Canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md) · [Mechanism admissibility](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/mechanism-admissibility-2026-10.md). Historical gates/results and full capability acceptance remain unchanged. Live comments own newer work; this header does not reset cumulative resources or transfer another lab's live files/jobs.
<!-- /active-roadmap-2026-10-01 -->


<details>
<summary>Preserved pre-adoption issue body and original acceptance</summary>

# A1 retrieval run — execute M-world v2.1 pointer/flock evaluation against the frozen `a1_gate` (Lab 2, self-directed)

**Integrated deliverable.** Executed retrieval evaluation on `main` `302e0ad7` of the merged A1 interface (#1548): the rule baselines (freeze conditions), then stack-model arms with `select=none|flock:W:K` and `pointer_select=none|flock:W:K|top:K`, judged by the frozen `a1_gate` (**MQAR recall ≥ 0.9 at every distance AND open-relation recall ≥ 0.9**), with honest reporting either way.

**Observed blocker.** None functional: the integration is merged and the labs are offline; the owner directed self-direction. The A1 run was always the declared "separate, later work card".

**Hypothesis.** The shared flock selector (#1532) and pointer copy head end-to-end change retrieval accuracy on the v2.1 cells versus the `none` controls; the gate may pass or fail — both are decision-bearing.

**Causal change.** No source change. Run only, on the exact merged head.

**Controls.** `select=none` / `pointer_select=none` arms; `m-world baselines world=v2` rule rows with freeze-condition fields (only model rows carry `a1_gate`); frozen gate definition and M-world v1 untouched.

**Outcomes and routing.**
- (a) Gate passes for a selection arm → record; the next rung is the sealed §8 qualification protocol (panel held outside the repo).
- (b) Gate fails → report the failing cells/distances and N precisely; scoped negative, preserved.
- (c) Instrument/plumbing fault → stop and publish; do not retune the gate.

**Fixed conditions.** Source `302e0ad7`; fresh claimed/sealed report roots per attempt; `CARGO_TARGET_DIR=$HOME/.cache/uor-r4-opencode-target`; never build on `/Volumes`; one Cargo lane; tokenizer from the retained bundle (`claude-t4-1433-resume/bundle-learned-1/tokenizer.json`); chat-v0 split `/Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925`.

**Necessary checks.** `m-world baselines world=v2` (cells, 2000 conversations, seed 9101); per-arm `m-world evaluate world=v2` (development split, 300 conversations, seed 9101) with the `none` controls; report roots sealed and verified.

**Resource projection (prospective).** Build ≤ 20 min, ≤ 6 GiB RSS, ≤ 2 GiB internal cache growth; baselines + evaluation ≤ 30 min; the training arm is projected separately after the baselines (bounded, recorded before launch). Standing owner authorization for necessary local extensions; complete costs recorded.


</details>


## Comments (7)

### comment 5919775192 — 2026-09-30T21:09:44Z — Casey-allard

**Resource projection and launch (Lab 2, self-directed under the owner's standing local-compute authorization).**

Following the Lab 1 A1 pre-registration (#1511, 2026-09-29 20:21): ~1.5M-parameter `rrarra` stack, width 128, 4 heads, context 256, trained from scratch on chat-v0 + M-world v2.1; **30-minute fixed wall per arm, identical step count across arms set from a smoke**; 4 threads per arm, two arms concurrently; gate = M-world v2 development `a1_gate` (MQAR ≥ 0.9 at 16/64/200 AND open-relation ≥ 0.9).

**Projection (prospective):** build `geometric-stack` ≤ 15 min, ≤ 6 GiB RSS, ≤ 3 GiB cache growth; corpus + smoke ≤ 20 min; three arms (P, P+ptr, F7) ≈ 60–75 min wall at 2×4 threads; evaluations ≈ 5 min each; then the Dot parity control + one seed-2 re-run ≈ 35–60 min. **Total ≤ ~4 h wall, ≤ 8 threads, ≤ 10 GiB peak RSS, ≤ 5 GiB new storage.** Runner worktree `opencode-a1-run` at main `302e0ad7`; internal `CARGO_TARGET_DIR`; no `/Volumes` builds. Machine otherwise idle; no paid compute.

Starting now: build → corpus → smoke (to fix the arms' step count). Results will be posted here and to #1511.

### comment 5919895109 — 2026-09-30T21:17:07Z — Casey-allard

**Smoke → fixed step count; wave 1 launched.**

- Corpus built from chat-v0 + M-world v2.1 (chat screened per the probe rule): `corpus-v2.1/train` — 83.2M tokens, 35,038 training / 224 development responses; manifest sealed in the root.
- Smoke (arm P config, 30 steps): **17.4 s train time → ~1.7 steps/s**; model **2,098,584 params** (`rrarra`, width 128, heads 4, mlp 502, context 256).
- **Fixed step count for all arms: 3,000 steps** (≈30 min at the smoke rate), `max_seconds=2100` hard wall; arm sets: **P** (`read=lorentz`, full) and **P+ptr** (`+ pointer dim 32`) now running concurrently at 4 threads each (`arm-P-2`, `arm-Pptr-2`). **F7** (`select=flock:16:7`) follows in wave 2, then the Dot parity control per the pre-registration.

Cost so far: build 9 s + corpus + smoke ≈ 5 min wall; ≤2 GiB cache growth. Arms: 2 × ≤35 min.

### comment 5920550431 — 2026-09-30T22:06:18Z — Casey-allard

**A1 run interim (wave 1 evaluated).**

Budget note first: with two arms concurrent (the pre-registered 2-at-a-time), each arm ran at ~1.1 steps/s, so the 2100 s wall stopped them at **2,590 / 2,566 steps** rather than the smoke-derived 3,000. All subsequent arms use the matched **2,590 steps**. Cost: wave 1 = 35 min wall (2 arms), evaluations ≈ 6 min each.

| Arm (steps) | `a1_gate` | MQAR 16 / 64 / 200 | relation open | copy | responsive |
|---|---|---|---|---|---|
| **P** — Lorentz, full read, no pointer (2590) | false | 0.00 / 0.00 / 0.03 | 0/52 (0.00) | 0/33 | 397/752 (0.53) |
| **P+ptr** — Lorentz, full, pointer dim 32 (2566) | false | 0.41 / 0.25 / 0.67 | 20/52 (0.38) | 0/33 | 472/752 (0.63) |

**Read:** the pointer copy head is the driver — it lifts MQAR from ~1% to 44%, open-relation from 0 to 38%, and responsiveness by +10 points, but nowhere near the 0.9 gate; **exact copy remains 0/33 in both** (the head copies one token, not a run). The distance profile is non-monotone (200 > 16 > 64), which is the diagnostic to explain next. F7 (`flock:16:7`) and C (Dot + pointer 32, the parity control) are training now at the matched step count.

### comment 5921783135 — 2026-09-30T23:48:51Z — Casey-allard

**A1 retrieval run — complete result (Lab 2, self-directed; follows Lab 1's pre-registration on #1511).**

**Setup.** `main` `302e0ad7`; model `rrarra`, width 128, 4 heads, mlp 502, context 256, **2,098,584 params**, from scratch; corpus **chat-v0 + M-world v2.1** (83.2M tokens; 35,038 train / 224 dev responses; chat screened per the probe rule). Gate = frozen `a1_gate` on the M-world v2 **development** split (300 conversations, seed 9101): MQAR ≥ 0.9 at 16/64/200 AND open-relation ≥ 0.9.

**Budget deviation (recorded).** The pre-registration's 30-min wall assumed the smoke's solo rate (1.7 steps/s). With the designed **two arms concurrent** each ran ~1.1 steps/s, so the 2100 s wall stopped seed-1 arms at **2,566–2,590 steps**, not the smoke-derived 3,000; the seed-2 parity pair ran exactly **2,590**. All runs exit 0; every root sealed.

| Arm (steps) | `a1_gate` | MQAR 16 / 64 / 200 | relation open | copy | responsive |
|---|---|---|---|---|---|
| **P** Lorentz, full (2590) | false | 0.000 / 0.000 / 0.028 | 0/52 (0.00) | 0/33 | 0.53 |
| **F7** Lorentz, `flock:16:7` (2590) | false | 0.000 / 0.000 / 0.028 | 0/52 (0.00) | 0/33 | — |
| **P+ptr** Lorentz, full + pointer 32 (2566) | false | 0.405 / 0.250 / 0.667 | 0.385 | 0/33 | 0.63 |
| **C** Dot, full + pointer 32 (2590) | false | 0.378 / 0.167 / 0.528 | 0.288 | 0/33 | 0.67 |
| **P+ptr** seed 2 (2590) | false | 0.378 / 0.194 / 0.278 | 0.231 | 0/33 | 0.65 |
| **C** seed 2 (2590) | false | 0.216 / 0.250 / 0.556 | 0.519 | 0/33 | 0.66 |
| Pure-retrieval cell (train phrasing / dev values) | — | P+ptr 0.336 · C 0.385 · F7 **0.000** | — | 0/21 | — |

**Findings (each with its exact scope).**
1. **No arm passes the gate.** The closest is MQAR 0.67 at distance 200 (P+ptr seed 1).
2. **Flock selection alone is inert at this scale/curriculum.** F7 is exactly P (MQAR ~0.9% overall, relation 0, copy 0) — the shared flock read contributes nothing without the pointer.
3. **The pointer copy head is the only retrieval driver**, but partial and **seed-noisy** (MQAR 0.22–0.67 across arm/seed/distance), with a non-monotone distance profile (200 often above 16/64) that the pre-registration's branch-2 diagnosis would target.
4. **No Lorentz advantage.** The parity pair is contradictory across seeds: P+ptr > C on seed 1 (MQAR and open-relation), C > P+ptr on seed 2 (200 and relation). Within seed noise at n=2; no Lorentz claim.
5. **Exact copy is unsolved: 0/33 in every arm** (0/21 on the pure-retrieval cell) — the single-token pointer cannot emit a copied run.
6. **Teacher-forced cell scoring reproduces the same accuracies** (`teacher_forced_unscored: 0`), so emission is not hiding higher read accuracy on this instrument.

**Decision routing (per the pre-registration).** No pass; none reaches 0.9 at 16; some arms exceed 0.5 (so branch 3 is not triggered by the letter). The clear successor diagnosis: (a) cut/deprioritize the flock-read mechanism at this scale — it added nothing; (b) the pointer needs multi-token copy emission and seed stability before the gate is plausible; (c) the copy task is untouched by the current mechanism. A transformer control at the same size/data remains available to separate "read defect" from "curriculum/scale defect" (decisions 2–3 in the card). Nothing about English chat quality is claimed; the §8 panel stays sealed and unused.

**Artifacts (preserved).** `/Volumes/UOR-Workspace/uor-r4-lab/opencode-a1-run-20260930/`: `corpus-v2.1`, `smoke2`, `arm-{P-2,Pptr-2,F7-2,C-2,Pptr-s2,C-s2}`, `eval-*`, `cells-*`, `logs/`. All report roots sealed; `attempt.json`/`manifest.json` per root. Total: ~2.6 h wall, ≤ 8 threads, ≤ 4 GiB peak RSS, ~2 GiB cache growth, no paid compute.

— Lab 2

### comment 5922486852 — 2026-10-01T00:51:35Z — Casey-allard

**A1 transformer control — interim: the plain transformer also fails to retrieve; the pre-registered 2× control is now running.**

| Arm (steps) | gate | MQAR 16 / 64 / 200 | rel open | copy |
|---|---|---|---|---|
| **T** transformer, full (1880) | false | 0.027 / 0.000 / 0.028 | 0.000 | 0/33 |
| **T+ptr** transformer + pointer 32 (1856) | false | 0.216 / 0.194 / 0.500 | 0.481 | 0/33 |

For comparison, the geometric arms (2,590/2,566 steps): P 0/0/0.028, rel 0.00; P+ptr 0.405/0.25/0.667, rel 0.385; C 0.378/0.167/0.528, rel 0.288.

**Read:** ordinary transformer attention with the same data does **not** retrieve either (T ≈ 0, like P and F7), and the pointer lifts the transformer about as much as it lifts the geometric stack (T+ptr ≈ C within noise). So the retrieval failure is **not a geometric-read defect at this scale** — it looks like a curriculum/scale defect, which is the pre-registration's branch-3 reading ("adjust the data, not the read"). The formal **kill** needs the control at **2× the steps**; the wall left T at 1,880 (the transformer is ~0.6× the geometric throughput), so I have now launched **T at 5,180 steps** (~2 h solo, 6 threads, `arm-T-2x`) to complete the pre-registered verdict. Copy remains 0/33 in all eight arms.

### comment 5923390398 — 2026-10-01T02:16:50Z — Casey-allard

**A1 retrieval run — FINAL verdict (Lab 2, self-directed; Lab 1's pre-registration on #1511).**

**Setup.** `main` `302e0ad7`; `rrarra` width 128, heads 4, mlp ~502, context 256, **2.10M params**, from scratch; corpus chat-v0 + M-world v2.1 (83.2M tokens; 35,038 train / 224 dev responses). Gate: `a1_gate` on the M-world v2 development split (300 conversations, seed 9101) — MQAR ≥ 0.9 at 16/64/200 AND open-relation ≥ 0.9.

**Complete results (all roots sealed).**

| Arm (steps) | gate | MQAR 16 / 64 / 200 | rel open | copy | responsive |
|---|---|---|---|---|---|
| P geometric, full (2590) | false | 0.000 / 0.000 / 0.028 | 0.000 | 0/33 | 0.53 |
| F7 geometric, `flock:16:7` (2590) | false | 0.000 / 0.000 / 0.028 | 0.000 | 0/33 | 0.58 |
| P+ptr geometric + pointer 32 (2566) | false | 0.405 / 0.250 / 0.667 | 0.385 | 0/33 | 0.63 |
| C Dot + pointer 32 (2590) | false | 0.378 / 0.167 / 0.528 | 0.288 | 0/33 | 0.67 |
| P+ptr seed 2 (2590) | false | 0.378 / 0.194 / 0.278 | 0.231 | 0/33 | 0.65 |
| C seed 2 (2590) | false | 0.216 / 0.250 / 0.556 | 0.519 | 0/33 | 0.66 |
| T transformer, full (1880) | false | 0.027 / 0.000 / 0.028 | 0.000 | 0/33 | 0.59 |
| T+ptr transformer + pointer 32 (1856) | false | 0.216 / 0.194 / 0.500 | 0.481 | 0/33 | 0.66 |
| **T transformer at 2× steps (5180)** | **false** | **0.000 / 0.000 / 0.028** | **0.000** | 0/33 | 0.72 |

**Findings.**
1. **No arm passes the gate**, on either architecture or any seed.
2. **Flock selection alone is inert**: F7 ≡ P exactly. The shared flock read contributes nothing at this scale/curriculum.
3. **The pointer copy head is the only retrieval driver**, partial (MQAR 0.22–0.67) and seed-noisy, with a non-monotone distance profile.
4. **No Lorentz advantage**: the seed-1/seed-2 parity pair flips; within noise.
5. **Exact copy is 0/33 in all eight arms** — no mechanism here emits a copied run.
6. **The plain transformer control also fails to retrieve** (MQAR ≈ 0.03 at 1,880 steps), so the failure is not specific to the geometric read.

**Pre-registered kill — ESTABLISHED.** The card: *"the transformer control also fails at 2× the steps → A1 reports a negative, and Track A retrieval moves to the log + pointer design without further read tuning."* The 2× control (T at 5,180 steps) **fails completely** (MQAR 0.000/0.000/0.028, open-relation 0.000, copy 0/33) — exactly as the 1× control and every geometric arm did. **A1 is therefore a scoped negative at this ~2.1M-parameter, ~2.6k-step curriculum: ordinary transformer attention and the geometric stack fail alike, so the failure is curriculum/scale, not the geometric read.** Retain the one real component result: the pointer copy head is the only mechanism that moves retrieval at all (up to 0.67 MQAR at distance 200), partial and seed-noisy, with exact copy unsolved. **Successor: adjust the data/curriculum (not the read); Track A retrieval moves to the log + pointer design.** No English-chat claim; the §8 panel remains sealed and unused.

**Artifacts.** `/Volumes/UOR-Workspace/uor-r4-lab/opencode-a1-run-20260930/` — `corpus-v2.1`, `smoke2`, `arm-{P-2,Pptr-2,F7-2,C-2,Pptr-s2,C-s2,T-2,Tptr-2,T-2x}`, `eval-*`, `cells-*`, `logs/`; every root sealed with `attempt.json`/`manifest.json`.


### comment 5925977174 — 2026-10-01T06:27:13Z — Casey-allard

D19 roadmap reconciliation: The attempted recipes have their final scoped negative and preserved roots on this issue and #1552; D18 outcome D stops further A1 training. This closes the run task, not attention research. Lorentz-scored pointer and full capacity/load coverage were not established by this attempt. Failure of both geometric and ordinary recipes leaves shared data/optimization/instrument explanations possible; it does not isolate one cause or refute a family. Successor #1552.

Disposition only at this stated scope; see #1563 and protected PR #1564.


---
_Comment/review ids in this file: 5919775192, 5919895109, 5920550431, 5921783135, 5922486852, 5923390398, 5925977174_

