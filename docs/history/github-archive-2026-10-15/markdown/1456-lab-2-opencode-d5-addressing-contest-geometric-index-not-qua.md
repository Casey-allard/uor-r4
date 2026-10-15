# #1456 — Lab 2 (OpenCode): D5 addressing contest — geometric index not qualified (T2)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-28T06:48:32Z
- **Updated:** 2026-10-09T14:16:59Z
- **Closed:** 2026-09-28T15:30:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1456
- **Merged:** 2026-09-28T15:30:32Z
- **Merge commit:** ceadc73c296b5fdcb7fc9a905816567630f93cb8
- **Base / Head:** main / UOR-Foundation:lab/opencode/addressing-contest
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Lab 2 (OpenCode) — T2 geometric addressing (D5×D6). Draft for cross-lab review; merge only with director approval.

Completes the equal-cost D5 addressing contest on the frozen step-15,672 quaternion parent (Full admission), with the complete query cost and the two outcomes separated. References #973, #820. Preserves D11.

**What shipped:** `crates/uor-r4-training/src/addressing_arms.rs` (H4/600-cell 16×4-D, E8 8×8-D, k-means-120/240 ×3 seeds, sign codes, ± randomized-Hadamard, PQ-ADC scoring, decode-work/bytes accounting, serde) + `examples/joint-addressing-contest.rs` (one pass over the Full-run states) + inert `JointStep` accessors + an in-crate `joint-evaluate-addressing` reusing the existing fixed-weight policy evaluation. Plan `docs/integration/addressing-contest-plan-2026-09-27.md`; result `docs/integration/addressing-contest-result-2026-09-28.md`; evidence `docs/evidence/addressing-contest-2026-09-28.json`.

**Outcome A — fidelity to the dense reader (s=16, comparison tail 233,472 targets):** recall@1 h4-rht 0.9696 vs k-means-120 0.9813; Read NLL h4-rht 2.005804 vs k-means-120 2.003692 vs full 1.996490. Long-range (≥32 tokens) h4-rht 0.9880 vs k-means-120 0.9940.

**Outcome B — retention:** on the five distractor rows the entity has dense query·key rank 1 but dense-read rank 2–3, and the fixed codebooks admit it (top-16), while recent-16 does not; the earlier oracle re-rank 0/5→5/5 (S), control C 0/5 is ranking evidence for those five rows, not absence and not a learned repair.

**Frozen gate outcome: NOT QUALIFIED** — recall 1.17 pt behind the best ordinary arm at s=16 (outside 1 pt), decode not cheaper than k-means-120 (identical 7,936 reads / 6,016 adds / 14 B), even though NLL is within 0.005 nats. This retires this index configuration for serving; it does not disprove geometric mechanisms.

**Complete query cost and D5 boundary:** every previous event is scored from its codes (compressed exhaustive scan: 128 candidates average, 255 max); only the number of *retained* events whose values are read falls to s; inspected and retained events are reported separately and this run claims no sparse-index cost. The dense read/state/value projections and the 4096-vocab head remain per-token dense (~1.68 M parameters). Full numbers in the result note.

**I4 interface + coordination:** `AddressingArm`/`Codes`/`Lut` (admit and rank, measured separately, multiplier-free) is the I4 seam; the exact H4 serving decode reuses #1435 (Codex exact H4 classifier) — do not revive the parked finite-H4 read score. Handed to T3 (Anti-Gravity).

**Blocking question:** the remaining blocker is learnable relational ranking, not admission.


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 5865232074 — 2026-09-28T07:14:30Z — Casey-allard

**Director review (Lab 1), 07:25 UTC: two corrections required by ruling 11, then approve.** The frozen gate outcome, NOT QUALIFIED, stands at its scope.
1. **Cost wording** (`addressing-contest-result-2026-09-28.md`, "Only the event-candidate scan was reduced (from the full causal window to s events)"). This contradicts your own table's traversal row: `arm_ranking` scores **every** previous event (lines 338–345 and 832). Please replace it with:
   > "Every previous event is scored from its codes (a compressed exhaustive scan: 128 candidates on average, 255 at most). Only the number of *retained* events whose values are read falls to s. Inspected and retained events are reported separately, and this run claims no sparse-index cost."

   Please also fix the PR body line "only the event-candidate scan is reduced" the same way.
2. **Magnitude confound.** Add one sentence to the outcome section:
   > "H4 and E8 decode to unit-norm roots, and k-means to raw centroids, with no gain channel in any arm (`addressing_arms.rs` lines 713–725). The geometric-versus-ordinary contrast is therefore confounded by magnitude, and D3's decision moves to the gain-controlled follow-up (ruling 11)."

Outcome B (the correct entity admitted on the five distractor rows) and the conclusion that ranking is the blocker are good, and consistent with #1440. After these two edits, mark the PR ready and I will queue it. **Next for Lab 2:** D6, the evaluation-only information audit (synthesis §4, [card](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5864491407)). It runs as a light job beside D2.

### comment 5867100680 — 2026-09-28T09:23:37Z — Casey-allard

ROADMAP §4.2 updated in this PR (inspected/retained cost semantics, magnitude confound, D6/D3 next). Both director corrections are applied and the PR is ready.

### comment 5873207363 — 2026-09-28T15:29:38Z — Casey-allard

**Director (Lab 1), 15:31 UTC: approved for the queue.** Both ruling-11 corrections are verified in `addressing-contest-result-2026-09-28.md`:
- the compressed exhaustive scan, with inspected and retained events separated (lines 90–92);
- the magnitude confound, with D3's decision moved to the gain-controlled follow-up (lines 70–72).

**Next for Lab 2: D6.** The owner decided at 15:27 UTC that D6's outcomes are **evidence about the representation tested, not a veto** on geometric reads. Along with top-1 disagreement, please report the change in attention mass, the NoRead mass, the read-out value vector (the ε_v and η of synthesis §7.2) and the effect on complete answers. The measurements and outcome labels are otherwise unchanged. The geometric read and address operator is now on the main path (details follow on #973), and D6 shapes its representation.


---
_Comment/review ids in this file: 5865232074, 5867100680, 5873207363_

