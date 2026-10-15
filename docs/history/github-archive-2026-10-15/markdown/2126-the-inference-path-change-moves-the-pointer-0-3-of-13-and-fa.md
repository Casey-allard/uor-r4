# #2126 — The inference-path change moves the pointer 0 → 3 of 13 and FAILS its pre-declared condition (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T01:47:44Z
- **Updated:** 2026-10-10T03:23:21Z
- **Closed:** 2026-10-10T01:49:07Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2126
- **Merged:** 2026-10-10T01:49:07Z
- **Merge commit:** 265ca0191d872187dbcc7f4d9075b7464ad078d4
- **Base / Head:** main / UOR-Foundation:inference-path-fix
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Applied — serving-time knobs on UNCHANGED WEIGHTS
`PointerSelect::TopK(2)` (plan §2a: `TopK(1)` is the single-source pointer) and `set_pointer_gate_floor(0.9)` (§2b: `p_copy` is exactly 0 with no floor). The trace bin gained `select=` and `gate_floor=`. **No weight changed, no training run.**

## The two arms
| | **ARM A — control (default knobs)** | **ARM B — TopK(2), gate floor 0.9** |
|---|---|---|
| **both stored digits selected** | **0 of 13** | **3 of 13** |
| class split | **EMITTER 2, PARTIAL 1, READER 10** | **DELIVERED 3, PARTIAL 6, READER 4** |
| byte-for-byte vs sealed | **13 of 13** ✔ | **0 of 13** |
| two-token assertion | held | held |

**Arm A is the instrument's validity control and it holds with the same binary.** Arm B's `0 of 13` is expected, not a defect: the knobs change decoding by construction, so the treatment arm cannot reproduce replies produced without them.

## The pre-declared condition: FAILED on its primary
| condition | bar | measured | |
|---|---|---|---|
| all 13 show both digits selected | 13 of 13 | **3 of 13** | **FAIL** |
| `mem-040` still emits `84` | yes | **yes** (`26, 22` in order) | pass |
| `mem-008`/`mem-024` no longer emit `19` | yes | **yes, both** | pass |
| instrument conditions hold | yes | control 13/13; assertion held | pass |

**Reported as a miss, not averaged into a story: this setting is insufficient on its own.**

## The falsification did NOT fire in its stated form
It was: *if both digits are selected and the reply still lacks the value, the fix is in the wrong place and the emitter is the real problem.* **Where both digits were selected — the 3 DELIVERED rows — the value WAS emitted.** So on these 13 rows **the failure is coverage, not emission: selection and emission agree wherever selection happens.** That is what this run adds. It does **not** license widening the same change.

## The EMITTER rows: a SEPARATE effect with its own evidence
`mem-008`/`mem-024` both emitted `19` in arm A; **neither does in arm B** — the gate knob acting on the mechanism that implicated it, a **selected-then-dropped** failure. **Reported separately from the reader fix and not counted as its success:** `mem-008` arm B emits the distractor's `9` and the stored `8` but not the value's run; `mem-024` emits `23, 22` in order — the stored `54`. They keep their own piece and pass condition.

## What this does not settle
**Arm B's replies are a different distribution and their quality was NOT measured** — no judge was run, the gate floor of 0.9 forces copying at 90% weight, and the id sequences show heavy repetition (`2605` repeated). **This is not a capability improvement and must not be read as one.** It classifies 13 rows on this artifact only, says nothing about the reply half, and **this artifact has no memory reader** so nothing transfers to an addressed-memory path.

## Ledger
Token count **REFUTED** · minimal pairs/digit order **REFUTED** · value addressability **REFUTED** · **the pointer's single-source shape NOT REFUTED, NOT CONFIRMED** — the knobs move coverage 0 → 3 of 13, so the shape is *a* constraint and at this setting not the whole one · the learned read/emit path **SPLIT**, with selection and emission agreeing wherever selection happens.

## Delivery
Record [`docs/labs/inference-path-fix-result-2026-10-09/README.md`](docs/labs/inference-path-fix-result-2026-10-09/README.md); the knobs in the bin; newest-first [`current-state.md`](docs/integration/current-state.md) entry ending in **Next:**; bundle on cloud-store before any local deletion: `inference-path-fix-result-2026-10-09.tar`, 96,768 bytes, md5 `3d5876e595f3105fc4b63b38ea5a97a5`. `check_claim_wording.py` passes; `scripts/__pycache__/` not committed; one live branch.

**Next: do not widen this change and do not try a third knob** — find out **why coverage stops at 3** by reporting which window positions the kept sources occupy at the step where the value should be emitted. **That is a read, not a knob.**

**Criterion 1 remains NOT MET on both halves; 43/232 unchanged; v5 not re-run. References #2029.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

