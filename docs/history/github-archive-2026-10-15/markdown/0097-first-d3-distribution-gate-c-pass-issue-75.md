# #97 — First D3-distribution Gate C pass (issue #75)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-23T08:00:25Z
- **Updated:** 2026-07-23T08:38:46Z
- **Closed:** 2026-07-23T08:38:46Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/97
- **Merged:** 2026-07-23T08:38:45Z
- **Merge commit:** 9bb75c2006c427ea74f86d1883157305f0f4ac64
- **Base / Head:** main / UOR-Foundation:issue-75-d3-gate-c
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The M.V.G. checkpoint's judge on the real D3 distribution, plus the tokenizer byte-sizing fix that natural text exposed.

**Tokenizer fix** (scenarios.rs): Tokenizer.encode sized the buffer by char count, but byte-fallback UTF-8 emits up to one token per byte — natural (non-ASCII) text crashed with 'token buffer full'. Sized by text.len() with the provable worst case noted.

**D3 first pass (declared n; SmolLM2-135M teacher, add-one, all positions ExactContext status):**

| partition | Rule 1+2 top-1 | Rule 1+2 bits | baseline top-1 | baseline bits |
|---|---|---|---|---|
| natural (400-article Simple Wiki slice, 17,457 held-out) | 15.04% | 13.30 | 15.04% | 18.76 |
| continuity (200k teacher-generated, 40,342 held-out) | 14.76% | 14.62 | 14.76% | 20.63 |

**M.V.G. contract verdicts:** Row 1 — floor PASS (bit-identical agreement, win/loss +0/-0 on every position), +5pp stretch FAIL on both. Row 2 — PASS on both (-5.46 / -6.01 bits below baseline). Deployed quality gate: PASS on both partitions at full f64 precision. Witness replay 64/64 per run.

**Findings the checkpoint needs to hear:** (1) D3-scale SmolLM2 agreement is ~15% vs the fixture's 31.7% — the teacher difference (SmolLM2 vs legacy stories15M), not a harness regression; the fixture anchor and D3 numbers are not comparable beyond shape. (2) Rule 1 (graph residuals alone) collapses on natural text (0.19%) — 8-token contexts repeat far less on natural text, so region residuals do not transfer; the recovery is entirely exact-context precedence, exactly as on the fixture. (3) 100% ExactContext status on both partitions — Graph/Novel paths still corpus-unmeasured at scale.

Observation pipeline evidence: 90,019 records (400/400 articles, 30 min wall), partition 72,562/17,457 matching the blake3 rule exactly (320/80 articles), merged kappa blake3:43d961d7... with independent byte-replication, determinism double-run proven. Verified: 49 suites, clippy -D warnings, fmt clean.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

