# UOR-R4 course-correction package — 2026-10-18 (Kimi PM)

Basis: six DeepSeek specialist audits (saved in this directory): yazan-issues-audit.md,
roadmap-tracker-coherence.md, readme-front-door.md, mechanism-census.md,
contributor-experience.md, d37-reactivation-architect.md. Owner directions this session:
reopen handling of Yazan's issues (executed by Claude at 16:50Z), ratify the D5 split
(executed as D38, #2671/6dd23c09e), and the new lab self-direction rule (labs pick work,
notify PM, PM redirects).

## Diagnosis in one paragraph

The rung plan is coherent; the programme layer is not. The active science (K16-A/R
redirect) is specific, gated and staffed. But project-track.md declares three different
"active plans", ROADMAP.md still teaches the superseded D32 float ban, M1 has no
acceptance criterion that can be met (R14 voided the only written one), "dormant"
mechanism labels mostly mean "float version preserved" while the discrete version is
active, and the GitHub surface (blank description, no topics, no milestones, an issue
template that permits matmul) misrepresents the project to the outside collaborators
who have now arrived. Separately, the chief architect's first-principles analysis
suggests the K16 sheet's Bar (i) may be arithmetically unreachable due to codebook
aliasing — being tested now at $0 (W0 census, agent-20).

## Executed already (no further decision needed)

- D38 (D5-a residency / D5-b sparsity) recorded, merged (#2671, 6dd23c09e), verified,
  posted to #2657 and #2033.
- #2655/#2657 reopened by owner/Claude; audit concurs; #2656 stays closed.
- W0 stored-checkpoint alias census commissioned (host-only, $0, no release needed).

## Owner decisions requested (numbered; PM recommendation in italics)

1. **M1 definition-of-done.** R14 voided the 116/232 reply bar; M1 currently cannot
   close. Adopt the drafted DoD: G-Pre ceiling ≥ 4,894; G-Adm both seeds; G-Occ with
   >16 occupied cells and an actual R>0 row; ≤0.90 BPB on the chat held-out stream
   through the D11 export with reported export gap; a fresh pre-registered transfer
   panel. *Recommend adopt, as a decision entry.*
2. **G-Cap reachability gate on the FIT'd K16 sheet.** Add `distinct_vectors ≥ Bar (i)`
   (net of admission) as a pre-run instrument gate; below it the run is
   INSTRUMENT_UNREACHABLE (D31), not a negative. Also supplement the vacuous G-Pre
   with census quantities. *Recommend adopt — it changes a FIT'd bar, so it is yours.*
3. **Initializer for new lineages.** May new lineages use an injective codebook
   (PrimeZeta or random injective null) while the frozen lineage keeps xorshift?
   *Recommend yes for new lineages only.*
4. **Workstream sequence after K16 qualification.** W1 K16 run (~$2) → W2 read-code
   capture ($0.91, same lease, already FIT'd) → W3 BVG-K discrete batched-coordinate
   law (~$12, matched discrete speed competitor) → W4 one float-credit sheet only if
   W2 says H_law and W3 fails. *Recommend endorse.*
5. **Export-gap decision rule.** What gap disqualifies a float candidate — the
   two-seed band, a fixed BPB-equivalent, or "reported only"? *Recommend: the
   two-seed band on the same evaluator, fixed per sheet at FIT time.*
6. **Interim M1 headline legibility.** Keep SUSPENDED, or set an interim capability
   target (e.g. "injective codebook + Q4 admission ≥ 4,894")? *Recommend interim
   target so D21 can fire again.*
7. **GitHub surface settings.** Set repo description/topics, enable Discussions,
   disable empty wiki, create M1–M8 GitHub milestones, add lab:kimi/lab:antigravity
   and workstream:* labels, fix `status: on-hold` spacing. *Recommend execute; it is
   repo settings, low risk, reversible.*
8. **Serving-float planning horizon.** Keep "serving float is a possible future D11
   revision" open (build export-gap infrastructure now) or closed for this horizon
   (gap is reporting-only)? *Recommend reporting-only for now.*

## PM execution plan (no further decisions; phased PRs)

Phase 1 (immediate, docs/settings):
- Fix `.github/ISSUE_TEMPLATE/implementation.yml` matmul sentence (live D25/D30/D37
  violation) and retarget templates from closed #820 to #2028/#2029.
- NOW.md (90-second current plan page) linked from README/STATUS/ROADMAP.
- Correct stale rows: ROADMAP D32→D37 text, M7 row post-#2654, closed-bug citations,
  README D0–D34→D0–D38, stats block (10 open issues, 6,452 tests), milestone status
  vocabulary unified (M2 held / M3 paused).
- Mechanism page corrections from the census (read-binding, pointer-identity-key,
  finite-relational-head, exact-addressed-memory, flock-rank-reads, rank-margin D37
  note, mechanisms README header).
- Yazan audit must-dos not already done: correct PR #2654 body (pinned comment),
  annotate the RAPL receipt arithmetic/CI, label SIMD kernels as unintegrated
  fixtures, fix the stale Arm M header in simd.rs, make the WASM demo runnable or
  marked non-runnable, record the D29 serving-wasm interpretation.
- Contributor on-ramp FIX-2/3/4/5: truthful "what is open", HARD-RULES.md,
  OUTSIDE.md, SECURITY.md.

Phase 2 (front door, one PR): README hero rewrite (~110-word status), "What works /
what does not" pair, TOC, figure badge fixes (BUILT & TESTED → HISTORICAL · INACTIVE
on dormant float figures), milestones.svg M1 line, embed serving-pipeline.svg,
rename the fake hero art, CONTRIBUTING uor-harness/cli.py fixes, good-first-issue
seed issues, issue-template config.yml + question form, outside-contributor PR
checklist fork.

Phase 3 (plan repair, after decisions 1–6): project-track.md collapse to one active
plan (history moved to docs/history/), current-state.md rolling window + archive,
Dose-1/Dose-2 glossary, per-milestone DoD entries, #2028 table rewrite,
optimization-path record supersession note, project-track Rung-1 softmax text
annotated (architect risk #8).

## New lab operating rule (owner, this session)

Labs self-direct: pick work from NOW.md / milestone queues, post a one-line intent
note to #2029 (or the owning milestone) before starting, keep working unless the PM
redirects. PM (Kimi) redirects only when work is non-appropriate. Handoff-waiting is
no longer the default. All existing gates unchanged: no compute without FIT + PM
dispatch + owner release where a hold applies; D23/D25/D30/D29 absolute; D37 float
offline-only in a separate scanned module; D11 serving; archive-not-delete;
protected merge queue; merge-tree verification; heartbeat every 5 minutes.

## Per-lab bootstrap prompts (to post after Phase 1 lands)

### Claude
You are the Claude lab on UOR-R4. Refresh origin/main and read NOW.md,
docs/integration/DECISIONS.md D37–D38, and the newest current-state entries. Current
state: the K16-A/R rows_v1 qualification is completing at 21224b446+ on the retained
pod; you are the sole owner-delegated packet acceptance reviewer (owner 6251237195);
timing is the two checkpoint-0 cells only (owner 6251027125). New operating rule: do
not sit dormant waiting for handoff — after your acceptance gate, pick work from
NOW.md or the milestone queues, post a one-line intent note to #2029 before starting,
and continue unless Kimi PM redirects. Natural self-directed lanes for you:
rows-qualifier source ownership, Stage M2 Relation-vs-Route preparation review,
mechanism-brief corrections. Gates: no training dispatch (Kimi PM + owner release),
D23/D25/D30/D29 absolute, D37 float offline-only in a separate scanned module,
archive-not-delete, protected queue with merge-tree verification, heartbeat every 5
minutes.

### Codex
You are the Codex lab on UOR-R4. Refresh origin/main and read NOW.md, D37–D38, and
the newest #2029 comments. Current state: you own the K16 qualification execution —
C-R suite, the timing-companion build (same rlib for both executables), sealed
manifests and custody at 21224b446+, then the four rows_v1 configs and measured
D-Packet. Claude accepts; Kimi dispatches; the owner's training release stays
separate. New operating rule: after the sealed packet posts, do not wait for
assignment — pick the next item from NOW.md (BVG-K sheet finalization prep, the
read-code capture ride-along on the same lease), post a one-line intent note, and
continue unless Kimi PM redirects. Gates unchanged: qualification is not training;
no retry from failure without external review; preserve failed-attempt custody;
pods via uor-pod only, lease renewed before expiry under a live job.

### DeepSeek
You are the DeepSeek lab on UOR-R4. Refresh origin/main and read NOW.md and D37–D38.
Your standing lanes: independent exact-head reviews, static diagnosis (your last
three — E0063, control-8 fixture, verifier constants — were all correct and used),
the receipt cross-check of the sealed K16 packet in parallel (not a required seat
per owner 6251237195, but wanted), and PM cover whenever Kimi is STALE >30 min. New
operating rule: between review requests, self-direct from NOW.md — the tracker
hygiene and stale-documentation correction queues name concrete items — post a
one-line intent note, and continue unless Kimi PM redirects. No compute without the
FIT + dispatch + release chain; your cover ends when Kimi's presence is live.

### Antigravity
You are the Antigravity (AGY) lab on UOR-R4. Refresh origin/main and read NOW.md and
D37–D38. Current state: your consumer verifier at c02d31d4 has seven exact source
findings from Codex's confined review (two-cell timing amendment, rows-image
4-tuple, C-R5 native constants 14/1020/[641,73,256], negative-witness identity,
C-R6 four distinct roles, source/checkpoint identity join, authenticated staging
paths) — finishing those is your first item, parallel to qualification, not gated
by it. New operating rule: after the repair, self-direct from NOW.md (host-verifier
hardening, consumer schema documentation, exact-head reviews), post a one-line
intent note, and continue unless Kimi PM redirects. Confined-review discipline
unchanged: verify actual bytes by hash, no execution beyond your declared scope.

### Kimi (PM, for the record)
PM seat: sheet/packet/disposition/FIT/dispatch ownership, merge-queue supervision,
semantic-drift watch, lab redirection under the new self-direction rule, and the
course-correction execution plan above. No training/eval from this seat.
