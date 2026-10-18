# UOR-R4 plan-coherence audit — 18 October 2026

## 1. Verdict

**There is a coherent plan, but it is one level down from where the owner is looking — and the four documents that claim to be the plan contradict it.**

The real plan exists and is defensible: one FIT'd K16-A/K16-R admission redirect on `rows_v1`, with named gates, named owners, a measured budget and pre-declared outcome branches. It lives in `docs/harness/prep/m1-discrete-rung0-admission-redirect-2026-10-15.md` and its successor draft `m1-learned-attention-link-R-DRAFT-2026-10-18.md`, and it is restated correctly in `ROADMAP.md:3-99` and the latest `#2029` comment (`6251527…`, 16:37Z).

What is broken is the layer above it:

- `project-track.md` — the document every agent is told is canonical — opens with an 18 October heading over ~13 October text (`project-track.md:16-22`), and contains **three separate sections that each call themselves the active plan** (lines 3, 133, 926).
- The "post-dose optimization path" (owner direction, same day) still says "**Dose 1 (current)**" and gates the optimization rung on "the dose producing a decision" — but that dose was stopped and its successor's decision branch never fired (`project-track.md:950-951`, `optimization-path-2026-10-18/README.md:25-30`).
- **M1 has no operative acceptance criterion.** Its own issue asks for 116/232; `R14` (`pm-rules.md:319-336`) voids every reply target. The other two criteria point at artifacts the discrete model has never produced.
- **D21's progress-control rule cannot fire on M1**, because it is defined on the headline R14 suspended. The tracker restates `count 0/3` while ~48 M1-line commits have merged since 14 October.

So: coherent rung, incoherent programme. The owner's fear is justified at the plan layer, not at the work layer.

---

## 2. The current plan skeleton, as it actually is

**Objective:** one native Rust autoregressive geometric model, integer/table serving (D11), useful conversation + memory + coding on a laptop. 0/8 milestones. Pre-alpha.

**Active substrate:** D32 discrete (`crates/uor-r4-discrete`), GPU-only, integer. D37 (18 Oct) re-admits float **offline only**, in a separately scanned geometric module, with a reported export gap; matmul and squashing stay banned everywhere; serving stays D11. **No hold is released by D37.**

**Active line:** M1 `#2029` — the K16 admission redirect on `rows_v1`: matched 2-arm × 2-seed contrast, `K16-A` control `8T+32A` vs `K16-R` candidate `8T+32R`, `V4096/B4/C192/K16`, `ReadSpec::Route`, `CanonicalVectorThenReadV1`, 16,404 batches (1,049,856 targets), fresh init, fresh transfer before any KEEP.

**Critical path right now, in order:**

| # | Step | Owner | State |
|---|---|---|---|
| 1 | Full one-session qualification at `21224b446`+ on retained pod `vast-56571294`: 25/25 finite controls + `admit_ptx`, C-R1..C-R6, `G-Pre` (checkpoint-0 `target_admitted_rows ≥ 4,894`, both seeds, re-measured baseline `u`), active-R rows checks, two checkpoint-0 A/R 100+1 timing records, measured all-in budget, sealed manifests | Codex | **NOT_RUN**. Three source faults fixed today: `#2667` (zero-gate Snapshot), `#2668` (12-entry PTX inventory), `#2670` (control-8 stale refusal string) |
| 2 | Seven consumer-verifier corrections (two-cell timing amendment, rows-image 4-tuple, C-R5 constants 14/1020/[641,73,256], negative-witness identity, C-R6 four distinct roles, source/checkpoint identity) | AGY | **NOT_ACCEPTED** at `c02d31d4` per Codex review (`#2029`, 16:36Z) |
| 3 | Packet acceptance | Claude (sole, owner `6251237195`) | pending ✓ |
| 4 | Regenerate four `rows_v1` configs + measured D-Packet | Codex | pending |
| 5 | **Training dispatch** — needs an owner release; preparation GO / training NO-GO | owner | held |
| 6 | Outcome branch: `K16-R` pass → register R-DRAFT (Relation + R); `K16_R_NULL` with A keeping admission → width-only successor; `ADMISSION_STILL_CAPPED_AT_K16` → full-vector address metric | PM | pre-declared in `m1-learned-attention-link-R-DRAFT-2026-10-18.md:6-8` |

**Budget:** ~1.2 h for four runs (measured, `current-state.md:83`); pod quote $0.908/h; **holds:** training, M2, Stage B; Q4 NOT_EXECUTABLE.

**What is *not* on the path but is presented as if it were:** the optimization rung, batched-coordinate scoring, spinor adapters, and every frozen float mechanism. None is sequenced against K16.

---

## 3. Contradiction and staleness table

### 3.1 Top-level documents vs live facts

| # | Claim | Where | Contradiction |
|---|---|---|---|
| C1 | "**Updated 14 October 2026**" under an 18 October header | `ROADMAP.md:51` | The same file's header block is 18 Oct and cites `#2658/#2662` merges |
| C2 | "**Current learning permission — D32:** … no floating-point forward, gradient, optimizer or matrix product" | `ROADMAP.md:34-35` | Directly contradicted by D37 (`DECISIONS.md:1740-1772`) and by `README.md:36-39`, `STATUS.md:50-53`. ROADMAP states the superseded ban as current |
| C3 | "D32 extends the discrete requirement to learning and numerical model instruments: no float forward, gradients or optimizer" | `ROADMAP.md:179-182` (§0) | Same contradiction; §0 is the "hard runtime rules" section, so this reads as a standing ban |
| C4 | "Thresholds marked **PROPOSED** on M1, M2 and M4 await owner confirmation" | `ROADMAP.md:138` | M1's row says "**no target** under R14" (`ROADMAP.md:143`); M2's bars are stated as fixed acceptance in `#2030`; R14 forbids re-scaling any of them. A "PROPOSED threshold" cannot coexist with "no target" |
| C5 | "Standing issues: … **open bugs** `#1542`, `#1476`, `#1718`, `#1738`" | `ROADMAP.md:154-155` | All four **closed 2026-10-18** (15:26–15:30Z). They were folded into `#2031` correctly; ROADMAP was not updated |
| C6 | ROADMAP experimental table lists "prior rung 0 **KEEP**", "exact weights at dense quality (**KEEP**)", "rung 1 half-MLP **KEEP and iterate**" | `ROADMAP.md:164`, `161-168` | The section header says these are inactive evidence, but the table uses live KEEP vocabulary instead of README's `dormant` / `archived-negative (D22 scope)`. To an outsider this reads as six current KEEPs |
| C7 | "Owner decisions **D0–D34**" | `README.md:266` | D37 exists and is cited 8 lines earlier in the same file |
| C8 | "decisions **D0 to D27**; **14 open issues**" | `README.md:371` | D37; **10** open issues |
| C9 | M1 "Current next dependency: D32 decomposition … baseline reuse … timing2 …" | `STATUS.md:158` | Superseded by the K16 redirect on 18 Oct. STATUS's own header (`:19-46`) says so, 100 lines above its own table |
| C10 | "Actual qualification build blocker: … failed E0063/exit101. Two standalone Route Snapshot constructors … repaired" | `STATUS.md:22-26` | Stops at `#2667`. Omits `#2668` and `#2670` — two further blockers and fixes. Same lag in `ROADMAP.md:19` |
| C11 | M3 **paused** (`README.md:99`), **in progress** (`ROADMAP.md:145`), **status:in-progress** (`#2031` label), **paused** (`#2028`) | four places | Four status words for one milestone |
| C12 | M4 **held** (`README.md:100`), **in progress; compute blocked** (`ROADMAP.md:146`), **blocked-on-owner** (`#2028`), **status:in-progress + lab:deepseek** (`#2032`) | four places | Four words, and `ROADMAP.md:146`/`README.md:100` attribute M4 to Claude "read source" while the label says DeepSeek |
| C13 | "Plan of record: `ROADMAP.md` **and** `project-track.md`" | `README.md:108-109` | Two plan-of-record documents. `AGENTS.md` calls project-track "canonical"; `ROADMAP.md:57-58` calls it "Long-form plan"; `docs/README.md` links D37 as authority |

### 3.2 The canonical plan vs itself

| # | Claim | Where | Contradiction |
|---|---|---|---|
| C14 | "The composed discrete flock/shared-writer path at `76198c3b5` passes **seventeen** admitted CUDA construction controls after Claude FIT (#2395): eleven Route and six flock. … **D1 seeded driver/evaluator and D2 measured cost are Codex-owned next for Kimi's PENDING learning sheet**" | `project-track.md:16-22` | `ROADMAP.md:36-38` and `STATUS.md:78-80`: **25** controls at `e7d926fbe` (#2400, eleven Route + six flock + **eight driver**); D1/D2 **landed**; the learning sheet is FIT, not PENDING. This is 13 October text under an 18 October heading |
| C15 | "**Rung 0 is suspended, not withdrawn** … the pre-registration `6124773996` **resumes unchanged once the stack it trains is compliant**" | `project-track.md:59-61` | D32 §5 freezes float sheets; D37 item 5 replaces that with "re-registration only" (`DECISIONS.md:1763-1767`). "Resumes unchanged" is the exact reading D37 forbids |
| C16 | "**This section is the active plan**" (D19, 1 October) | `project-track.md:133` | Line 3 is also current owner direction; line 926 is "**Active programme** — learn the model, then harden its execution" (D8, 24 September). Three self-declared active plans in one file |
| C17 | "**Dose 1 (current).** The paired Relation/frozen-zero falsification run …" | `project-track.md:950` | Dose 1 was stopped at 904/1065 before Q1; its successor ran and stopped at Q1 admission collapse. `current-state.md:128`, `189` |
| C18 | "**Optimization rung (gated on the dose producing a decision)**" | `project-track.md:951` | The dose produced no decision ("no registered efficacy hypothesis fires", `#2029` `6248429872`). The gate has no defined branch for "stopped without decision", so step 2 is unreachable and undefined |
| C19 | "the active discrete path (`crates/uor-r4-discrete`, **Dose 1**) remains exact integer" | `project-track.md:11-12` | Dose 1 is over; the active law is the K16 redirect |
| C20 | Rung ladder statuses "Complete / retained gate met / **KEEP**" for rungs 0–4 | `project-track.md:934-945` | These are the pre-D32 float ladder. They are not marked frozen the way the operator table in `README.md:136-160` is, and they still say "Complete". `README.md:154` marks the same rung `dormant` float sheet |
| C21 | "**Status as of 14 October 2026** … [D37] (18 October) re-admits float …" | `mechanisms/README.md:13-14` | Date stamp predates the content in the same paragraph. Also lists "eleven … seventeen … twenty-five" controls as a flat sequence without saying which supersedes which |

### 3.3 The D37 problem specifically

| # | Finding | Evidence |
|---|---|---|
| C22 | **No plan entry says which frozen mechanisms may return, in what order, or how they interact with K16.** D37 item 5 says only "each may come back only as a new pre-registration with a fresh FIT … and an owner release" | `DECISIONS.md:1763-1767` |
| C23 | The **only** ranked D37 candidate list is in a mechanism brief, under M4: C1 float-trained coupling over fixed Flock topology, C2 lawful soft-to-hard selection, C3 FBS ternary-tree murmuration — with C1's decisive measurement written as "conditional top-1 on the **Dose 1** slices" | `docs/mechanisms/starling-read.md:76-95` |
| C24 | A C1 pre-registration draft exists and is **PENDING FIT**, and its own §7 asks the PM to confirm a reading of D37 item 3 before registration | `docs/harness/prep/m4-c1-flock-coupling-DRAFT-2026-10-18.md:151-157, 198` |
| C25 | Frozen float sheets are identifiable but not enumerated as a re-registration queue: `m1-rank-margin-rung0-2026-10-13`, `m1-d29-vendor-finite-2026-10-13`, `m1-starling-rung0b-2026-10-13`, `m4-starling-rung0b-finite-2026-10-13`, `m1-finite-relational-head-2026-10-14` and the operator sheets. Five carry `TEST FITNESS: FIT` and are frozen | FIT verdicts in `docs/harness/prep/*.md`; D37 item 5 list |
| C26 | D37 item 6 pins "Dose 1 (`m1-learned-attention-link-2026-10-17.md`) stays integer-only" — but the *successor* (R-DRAFT) also states "D37 is not used; this sheet runs no float" | `DECISIONS.md:1768`; `m1-learned-attention-link-R-DRAFT-2026-10-18.md:15`. So **every** live M1 sheet is integer-only; D37 currently changes nothing on the critical path |

### 3.4 The optimization path

| # | Claim | Where | Status |
|---|---|---|---|
| C27 | "Owner direction (2026-10-18, in-session): the falsify-then-optimize sequence **is the current path**; all three active labs align to it" | `optimization-path-2026-10-18/README.md:3-5` | Merged 06:51Z. Superseded within hours by the Q1 diagnosis and the K16 redirect |
| C28 | "**Codex (cf1):** dose packet seal (with the fresh-input recipe), execution, outcome report" | `optimization-path-2026-10-18/README.md:41-42` | Current division (16:37Z): Codex = qualification + timing + operator + packet; **Claude = sole packet acceptance**; AGY = consumer verifier; Kimi = acceptance/dispatch. Neither the optimization record nor `project-track.md:953` reflects Claude's acceptance authority |
| C29 | "Batched-coordinate scoring — candidate training-time lever; new FIT'd pre-registration required" | `optimization-path-2026-10-18/README.md:34-36`; `project-track.md:952` | Draft exists, `TEST FITNESS: PENDING`, no registration wired to any milestone | 
| C30 | "the compute-cost thesis is architectural … proven in order, not asserted" | `optimization-path-2026-10-18/README.md:17-20` | Fine as a thesis, but no document states what measurement would falsify it or which rung produces that measurement |

### 3.5 Rules that no longer compute

| # | Rule | Where | Problem |
|---|---|---|---|
| C31 | D21: after **three consecutive merged PRs** on a line that do not move the headline → pivot card | `DECISIONS.md:1055-1066` | M1's headline is **suspended with no target** under R14. No PR can move it, so every PR counts and none resets; yet the tracker repeats "count 0/3 unchanged" (`ROADMAP.md:143`, `current-state.md:191` etc.) across ~48 M1-line commits since 14 Oct. The rule is either suspended (not stated) or unenforced (not stated) |
| C32 | R14: a target exists only when its instrument passes R12's four questions | `pm-rules.md:319-336` | The `v2` check table was delivered (`current-state.md:2185`) but **no target was ever set from it**; nothing records why. So M1 is permanently targetless by omission rather than by decision |
| C33 | D35 §4: seed-2 held for "the admission iteration"; the K16 redirect registers seeds 1 **and** 2 | `DECISIONS.md:1703-1711`; redirect sheet `:13` | Not a contradiction, but unresolved: D35 §1(a)-(e) re-issue requirements are never explicitly discharged or explicitly scoped away for the K16 seed-2. A reviewer cannot tell whether D35 binds it |
| C34 | PR template demands `Line: · count /3 · headline → ` on every PR | `.github/pull_request_template.md:7-9` | Same defect as C31: the field cannot be filled truthfully for M1 |

### 3.6 Tracker (GitHub) vs documents

| # | Finding | Evidence |
|---|---|---|
| C35 | `#2029` acceptance criterion 1 requires **116/232** replies and **34/40** memory | `#2029` body, "Acceptance Criteria" |
| C36 | `#2028` says M1 replies are "SUSPENDED under **D28**" | `#2028` body, M1 row | Wrong authority: D28 bans neural graders; R14 voids the target (`pm-rules.md:319-336`) |
| C37 | `#2028` and `#2029` call the stopped `rows_v1` run "**Paired `rows_v1` Dose 1**" | both bodies | ROADMAP/STATUS/README call it the successor / **Dose2**. "Dose 1" is simultaneously (a) the stopped original run, (b) the FIT sheet, (c) the rows_v1 run. D37 item 6 uses sense (b) |
| C38 | `#2032` says the murmuration read is "gated on **Rung 0** and owner hold release" | `#2032` body | Rung 0 is the frozen float sheet (C15). M4's chosen read is gated on a sheet that cannot run |
| C39 | M4 acceptance criterion 1 (zero prohibited instructions at serving) is already claimed PASS for native (`#2655`, 44 symbol ranges) and WASM (`#2654`, 7/7) | `#2032` body | Criterion is written against the *runtime*, not the *chosen model artifact*, so "partial acceptance met" is ambiguous and M4 could be argued open or blocked depending on the reader |
| C40 | M7 acceptance criterion 2 (compiled wasm passes the audit) is already satisfied | `#2035` body vs `not-started` | Status says not-started while a stated acceptance criterion is green |
| C41 | `#2031` claims `UR4SESS1` (`crates/uor-r4-integer/src/session_codec.rs`, from `#2654`) as M3 delivered infrastructure | `#2031` body | It is a D11 serving artifact (M4/M7). Reasonable as infrastructure, but it inflates M3's delivered list |

---

## 4. Issue disposition table

Ten open issues total (`gh issue list --state open`; `open_issues_count: 10`). **There is nothing to close.** The problem is the opposite: two mega-threads absorb all work (`#2029` 1,032 comments, `#2037` 1,293), and the other eight issues are near-static.

| # | Issue | Verdict | Concrete action |
|---|---|---|---|
| `#2028` | Roadmap tracker | **Current but wrong in two cells** | Fix M1 row: rename "Dose 1" → "rows_v1 successor (Dose 2)"; change "SUSPENDED under D28" → "SUSPENDED under R14, no target"; add `#2668`/`#2670`; add the 16:37Z owner division. Keep open |
| `#2029` | M1 | **Stale acceptance; current gate** | Rewrite Acceptance Criterion 1: mark the 116/232 reply bar **BLOCKED** per D28/R14 and state the operative bar (G-Pre 4,894 → G-Adm → G-Occ → export at ≤0.90 BPB) or state plainly that M1 has no closing instrument. Add the 16:37Z division. Keep open |
| `#2030` | M2 | **Stale body, correct label intent** | "Current State" is 13–14 Oct; add a dated line: no M2 compute since 14 Oct, hold unchanged. Relabel `status:in-progress` → `status:blocked`. Keep open |
| `#2031` | M3 | **Mislabelled** | Relabel to `status:blocked`/`status: on-hold`; re-attribute `UR4SESS1` as M4/M7 infrastructure. Good fold-in of the four closed bugs — keep that |
| `#2032` | M4 | **Stale gating logic** | Replace "gated on Rung 0" with the discrete line's actual gate (murmuration read finite FIT on the discrete carrier); fix the `lab:deepseek` label vs Claude read scope. Keep open |
| `#2033` | M5 | **Honest, thin** | Relabel body to say the RAPL/QUBO artifacts are *instruments*, not M5 measurements (its own text says this; the tracker table does not). Keep open |
| `#2034` | M6 | **Fine, thin** | Keep; add a one-line dependency on the M4 artifact identity |
| `#2035` | M7 | **Contradicts itself** | A stated acceptance criterion (wasm audit) is already green while status is `not-started`. Mark criterion 2 satisfied-at-runtime and scope it to the model bundle. Keep open |
| `#2036` | M8 | **Fine** | Keep |
| `#2037` | Compute board | **Overloaded** | 1,293 comments. Move per-pod history to `docs/labs/compute.md` or monthly archives; keep only live leases and the running total |

**Closed issues still cited as open:** `#1542`, `#1476`, `#1718`, `#1738` (`ROADMAP.md:154`), `#820` (both issue templates). `#820`, `#973`, `#1172`, `#1552` all closed 2026-10-09.

---

## 5. Tracker-hygiene recommendations (ready for PM decision)

### 5.1 The single highest-value fix: separate "plan" from "record"

`docs/integration/current-state.md` is **8,638 lines** and `project-track.md` is **1,458**. No outside researcher will read either, and the PM's own newest-15 entry review requires ~400 lines of dense prose.

- Cut `current-state.md` to a rolling 60-entry window; move older entries to `docs/integration/current-state-archive-YYYY-QN.md` (the repo already uses this pattern: `docs/history/current-state-2026-09-25-to-2026-10-02.md`).
- Reduce `project-track.md` to one "active plan" section (retitle lines 126 and 926 as `Historical: …`). Move the September/October ladders wholesale to `docs/history/`.
- Add one file the owner can read in 90 seconds: `docs/integration/NOW.md` — objective, active line, the 6 critical-path steps with owners, the current block, the budget, the holds, and the next decision. Link it from README/STATUS/ROADMAP.

### 5.2 Add a "definition of done" per milestone that survives R14

Draft text for M1, which the PM can accept or amend:

> **M1 closes when the saved discrete rung-0 model passes, on one artifact: (a) `G-Pre` admission ceiling ≥ 4,894; (b) `G-Adm` on both seeds; (c) `G-Occ` with >16 occupied cells and an actual R>0 row; (d) ≤0.90 BPB on the chat held-out stream through the D11 export, with a reported export gap; (e) a fresh, pre-registered transfer panel. The 116/232 reply bar is BLOCKED under D28/R14 and is not a closing condition.

Without this, M1 cannot close no matter what the model does.

### 5.3 Labels

| Action | Detail |
|---|---|
| **Add** | `lab:kimi`, `lab:antigravity` — both labs are active today (Kimi PM of record; AGY owns 7 verifier corrections) and neither has a label |
| **Add** | `workstream:k16-admission`, `workstream:serving-d11`, `workstream:instrumentation`, `workstream:governance` — the current work is invisible in the label space |
| **Rename** | `status: on-hold` → `status:on-hold` (space in a label name breaks `--label` queries and scripts) |
| **Retire or apply** | `roadmap:active` (8 uses), `roadmap:parked` (4), `roadmap:ready` (3), `roadmap:next` (0 on open issues) — the scheduling vocabulary exists but no open issue carries it |
| **Apply** | `roadmap:active` to `#2029`; `roadmap:dependency` to `#2032`, `#2033`; `status:held` (new) to `#2030`, `#2032` |

### 5.4 Milestones — currently non-functional

There is **one** GitHub milestone ("Native Rust geometric intelligence…", open 0 / closed 74, legacy closed issues). Every open issue has `milestone: none`. M1–M8 exist only as issue titles.

- Create eight milestones `M1 Language base` … `M8 Alpha release`; assign `#2029`–`#2036` respectively.
- Close or rename the legacy milestone so it does not read as the active one.
- Then GitHub's milestone view becomes the scope picture the owner is asking for.

### 5.5 Issue templates — one contains a prohibited instruction

`.github/ISSUE_TEMPLATE/implementation.yml:10` says:

> "**Rust training may use matmul**; final serving has no matrix products or transformer backbone."

This contradicts D25, D30 and D37 item 2 ("no dense matrix multiplication or GEMM, including wrapper laundering"). It is live text that tells any contributor that matmul in training is allowed. **Fix immediately.** The same template and `research-experiment.yml` point at `#820` as the roadmap authority, which closed on 9 October; retarget to `#2028`/`#2029`.

The PR template (`.github/pull_request_template.md`) is otherwise current and good — but its mandatory `count /3` field has the C31 defect and should be redefined or made conditional.

### 5.6 Naming: retire "Dose 1"/"Dose 2" as load-bearing terms

Add a one-row glossary to `NOW.md` and to `#2029`:

| Term | Means | Status |
|---|---|---|
| Paired Relation/frozen-zero dose (sheet `m1-learned-attention-link-2026-10-17`) | the FIT'd integer sheet | still the parent sheet; **run stopped** at 904/1065 |
| `rows_v1` successor | `20d158ae`, packet `01833fa7` | **stopped** at Q1, 4,096 batches, both arms |
| K16-A / K16-R redirect | `m1-discrete-rung0-admission-redirect-2026-10-15` | **the live line** |

---

## 6. What I could not verify

- **Project boards:** `gh project list` fails (`missing required scopes [read:project]`). I cannot confirm whether a board exists or is stale. Worth a token refresh if the owner wants a board-based view.
- **#2029's 1,032 comments:** I read the newest three and used the document bodies for everything else. A claim-by-claim audit of that thread is a separate job.
- **Whether D35 binds the K16 redirect's seed 2** (C33) is genuinely ambiguous in the sources; I could not find a comment that resolves it.

---

## 7. Three sentences for the owner

1. The science plan is real and specific — it is the K16-A/K16-R redirect, and it has an owner, a gate list, a budget and pre-declared failure branches.
2. The programme plan is not: the canonical document still describes 13 October as current, calls a stopped run "current", contains three self-declared active plans, and states a float ban that D37 repealed this morning.
3. The single most damaging hole is that **M1 has no acceptance criterion that can be met**, because R14 voided the only one written down — so the project cannot say what finishing line 1 looks like, and no amount of further K16 engineering will answer that.