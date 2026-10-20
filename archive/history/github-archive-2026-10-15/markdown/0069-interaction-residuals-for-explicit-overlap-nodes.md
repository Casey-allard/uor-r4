# #69 — Interaction residuals for explicit overlap nodes

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-23T00:05:20Z
- **Updated:** 2026-07-23T15:46:02Z
- **Closed:** 2026-07-23T14:30:23Z
- **Labels:** graph-compiler
- **Milestone:** Phase 4: Transitions and residual emission
- **Assignees:** auser, Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/69
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Depends on: #64 and the cover fineness sweep. The PDF's original double-counting mitigation expects interaction residuals attached to canonical overlap nodes when overlapping memberships carry non-independent evidence. The induced cover currently has no explicit overlap nodes (only E_o edges). Evaluate whether any high-co-activation region pairs (from the E_o builder) justify materializing an overlap node with an interaction residual ΔI(pair,v) = lnP(v|pair) − lnP(v|p1) − lnP(v|p2) + lnP(v|parent), gated on held-out gain vs its bytes. Acceptance: overlap nodes exist only where the held-out bits/token improvement exceeds a documented threshold (else this issue closes as 'evaluated, not justified' with the numbers).


---

## Definition of Done
- A principled evaluation decides whether any high-co-activation region pairs justify explicit overlap nodes with interaction residuals.
- Overlap nodes exist in artifacts only where held-out gain exceeds the documented byte threshold — or the candidate is closed as evaluated-and-rejected with numbers.

## Acceptance criteria
- [ ] Candidate pairs are selected by a documented co-activation criterion (not hand-picked).
- [ ] Per-candidate held-out bits/token delta vs artifact bytes is reported.
- [ ] If adopted: Theorem 10 non-duplication tests cover interaction entries (unique canonical contribution IDs).
- [ ] If rejected: the measured table proving non-justification is attached.

## Comments (1)

### comment 5060442106 — 2026-07-23T15:46:02Z — auser

## Final verdict: EVALUATED, NOT JUSTIFIED — closing

Explicit overlap nodes with interaction residuals are **rejected for artifact storage** on the D3 distribution. This supersedes the zero-delta result reported in #99: the metric behind that table was flagged as fake in review commit `662865f` (never landed; cleanup tracked in #102). The pinned, reproducible measurement is now in-tree as `crates/uor-r4-core/tests/interaction_residuals.rs` (PR #101).

### Method

16 strongest co-activation region pairs selected from E_o neighbor edges, ranked by joint-observation mass. Each candidate scored with quantized log-probability interaction residuals

ΔI(pair, v) = ln P(v|pair) − ln P(v|p1) − ln P(v|p2) + ln P(v|parent)

side-by-side against the chain-telescoped baseline (#64) on the D3 held-out split, under both Rule 1 (precedence) and Rule 1+2 (compiler).

### Result (all 16 candidates)

| Rule | Effect |
| :--- | :--- |
| Rule 1 (precedence) | **+0.64 … +2.25 bits/token worse** across all candidates |
| Rule 1+2 (compiler) | **+0.02 … +0.29 bits/token worse** *and* top-1 agreement drops up to **−0.53pp** |

The agreement drop breaks the EXCT argmax-identity the deployed gate relies on (#68) — so even the mildest candidate is disqualifying, before the bits regression is considered.

Decision threshold per this issue's DoD: ≥0.05 bits/token improvement on either rule without agreement loss. **No candidate clears it.** Best candidate still degrades scoring.

### Anchors (asserted by the harness)

- Artifact κ = `blake3:de04eec8be0ce001c1493acee1b28f83976a74f85519855e1f23e8676d713704`
- Rule 1: 1.2185% agreement / 17.7299 bits/token
- Rule 1+2: 31.7086% agreement / 9.8612 bits/token
- TLA3 baseline: 31.71% / 11.8781 bits/token

### Why it fails

The interaction residual double-counts information the EXCT refinement chain already carries: conditioning on a region *pair* re-attributes probability mass that chain telescoping (#64) has already assigned, moving the candidate distribution *away* from the teacher.

Reproduce: `cargo test -p uor-r4-core --release --test interaction_residuals -- --ignored --nocapture` (~19s; requires `/tmp/ref/out/model.bin`).

Negative result recorded; closing.



---
_Comment/review ids in this file: 5060442106_

