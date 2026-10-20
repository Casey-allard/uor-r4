# #1453 — Whole-project synthesis, the approved track, and the external audit integration (References #820, #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-28T05:48:44Z
- **Updated:** 2026-10-09T13:09:01Z
- **Closed:** 2026-09-28T06:12:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1453
- **Merged:** 2026-09-28T06:12:36Z
- **Merge commit:** 2f0351998454e3a099771c29575490bed7b9e588
- **Base / Head:** main / UOR-Foundation:lab/claude/synthesis-track
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab:** Claude, director.

**Owner requests:**
- synthesize all results, so that the correct track is chosen from evidence;
- then (2026-09-28) integrate the external Codex audit, correcting what the evidence supports.

**Owner decisions recorded:**
- the track: "Approve, start with D2";
- data: existing corpora plus generated data;
- SIMD: decide after D4.

**Records:**
- `docs/integration/whole-project-synthesis-2026-09-28.md`: the synthesis, the approved track, the pre-registered measurements, and §7 on the audit integration.
- `docs/evidence/external-codex-audit-2026-09-28/`: the Codex packet verbatim, 18 files, with a claim ledger and the director's reproductions.
- `ROADMAP.md`:
  - D0 recorded;
  - §2a, the approved track;
  - ruling 11, the audit corrections;
  - B1 closed, and T4 transferred to Lab 1;
  - new dead-path rows and a director log entry.

**Findings** (unchanged, with scope corrected by the audit):
1. **Nothing talks yet.**
   - The measured limits are capacity (0.63–7M non-embedding parameters), missing conditioning (74.6% of response targets lacked their prompt) and exposure.
   - That these are the *cause* is a working hypothesis, tested by D0 and D5.
2. **Geometry as a drop-in score or mixer ties or loses in every parameterisation tested.**
   - #1438's cause is **unresolved**: magnitude, snapping and a learned MLP changed together, and its norm control never ran.
3. **Geometry as exact discrete structure wins where the task has that structure:** A5 automata, and exact identity (KVAR).
4. **What fails is the learned interface to exact structure, and read ranking.**
5. **The hard artifact's losses come from the weight representation; its energy cost comes from instruction count.**

**The track:**
- a 10–30M recurrent core, **the stack (D0 decided: 1.998113 against 2.011149)**;
- M1 AERM, architectural exact relational memory;
- M2, conditioned anti-echo training;
- M3, a geometry-coded D11 artifact;
- M4, event-gated state, deferred.

**External audit: what was verified and changed** (ledger in the evidence README):
- **Accepted, verified in source:**
  - T2's harness scores **every** previous event while charging only the s retained. It is a compression-fidelity screen, not an index cost.
  - T2's H4 and E8 arms decode to unit roots, k-means to raw centroids, so the contrast is confounded by magnitude.
  - #1438's reader drops per-lane magnitude before a learned MLP.
- **Reproduced:**
  - both supplied probes, with identical checks and script SHA-256;
  - the 2I orbit factorization, recomputed independently (230,400 pairs, maximum error 1.2e-13).
- **Added:**
  - **D6**, an evaluation-only information audit (Lab 2): arms O/U/D/G/K and four named outcomes. Only GAIN re-admits a geometric read.
  - **D7**, conditional score-to-address compilation, as AERM's approximate channel with exact postings.
  - A four-class failure trace, and counterfactual coverage for D2.
- **Adjusted:** D6 goes to Lab 2, which owns #1438's reader and T2's accessor, not Lab 1.
- **Not adopted:** certificate pruning (0/192 certified), orbit codebooks, and new kernel fits before D6.

| ID | What | Owner | Status |
|---|---|---|---|
| D0 | The cycle-4 reading | Cloud | **Decided: the stack** |
| D1 | Transport attribution at matched width | Cloud | Queued |
| D2 | The AERM probe against a dense-read core | Lab 1 | Next, after #1433's endpoints |
| D3 | The T2 contest, then a gain-controlled follow-up | Lab 2 | Frozen run complete locally |
| D4 | Geometry-coded QAT against #1433 | Lab 3 | After #1433 |
| D6 | The information audit, evaluation only | Lab 2 | Next for Lab 2 |
| D7 | A geometric address channel | Labs 1 and 2 | Conditional on D6 and D3 |
| D5 | The milestone candidate | All | After D0–D4 and D6 |

**Checks:**
- `python3 scripts/check_claim_wording.py` passes.
- The relative links resolve, and the table columns are consistent.
- Documentation and evidence only; no code changes.

References #820, #973, #962, #963, #964

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

