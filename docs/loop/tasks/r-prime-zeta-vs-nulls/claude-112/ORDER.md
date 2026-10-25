# ORDER: r-prime-zeta-vs-nulls, claude-112

Lab: claude. Item: [r-prime-zeta-vs-nulls](../../../../roadmap/items/r-prime-zeta-vs-nulls.md) (#2881). Base: `main` at claim.

## Goal

The item's done-when reads:

> Zero-update reach and capacity numbers for prime-zeta, random-injective and quota-only-injective codes on the same panel, recorded as a notebook row. The bar is not yet registered; registering it, and defining "reach" and "capacity", is part of the pre-registration sheet.

**What exists.**
- The null modes, the `prime-zeta-codebook` CLI (7/7 bin tests), and the draft sheet `docs/harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md`.
- That draft has seven arms: P8, Q0–Q2 and R0–R2. Its tables are pinned and rebuild bit-exact on the host.
- Kimi's non-author review (23 Oct) found it NOT_FIT on two grounds:
  - (a) "owner/PM registration of the bar and spend";
  - (b) an outcome rule that was not total.
- (b) was resolved in antigravity-36. (a) is now the lab's own call: D68 for the bar, and D67 for the run.

**This unit:**
1. **A D68 card** registering the draft's bar, with the residual clause.
2. **The final sheet** (`r-prime-zeta-vs-nulls-2026-10-25.md`), with the registration under D67 and D68, pins and the grader. Then a fresh non-author FIT, and a D67 pre-run review.
3. **The grader** as a detached host instrument in `docs/labs/r-prime-zeta-vs-nulls/instrument/`. It takes a read-only path dependency on `uor-r4-discrete` and reuses `address_key_grade::paired_psucc` and `clears` (merged in #3230): admission rows paired by ordinal, and the exact integer floor F = max(24, 2√(b+c)). It is placed outside the crate, because any crate path in this item's scope would lock every discrete item for every lab.
4. **The zero-update GPU run:** 7 arms, about $0.20–$0.50 on one RTX 5090. Then the grading, the REPORT and the notebook row.

## Cancel probe (run first)

| Check | Result |
|---|---|
| The null modes and CLI are on `main`, and `prime-zeta-codebook` bin tests pass | **Holds:** 7 passed, 0 failed |
| The draft's 7 pinned tables rebuild bit-exact on the host | **Holds:** `83c1380a`, `33b4e459`, `57225bbb`, `e03d6aef`, `229e93f5`, `36fec632` and `25ec9bd2` match the draft's `file_sha256` pins |
| The reach measure the draft uses (`attention-dev-b3` per-row `target_admitted`) can be graded with `paired_psucc` | **Holds:** `B3Summary` rows; `address_key_grade` is on `main` |
| No open PR holds this item | **Holds:** `loop claim` passed |

## Files in scope

- `docs/harness/prep/r-prime-zeta-vs-nulls*`
- `docs/labs/r-prime-zeta-vs-nulls/**`, the instrument included
- this task folder

## Done when

- The bar is registered by a D68 card with a fresh decision review.
- The final sheet has a non-author FIT and a D67 pre-run review.
- The instrument has tests that fail without it.
- The 7 arms run with their instrument gates (G-Rebuild, P8 exactness) first, and are graded by the registered rule.
- The REPORT is claims-checked.
- The notebook row is written. If the installed guard refuses `docs/research/notebook/m1-prime-zeta.md`, note it in the REPORT and carry the row in a unit that holds the page.
- The item is set `done` (`Closes #2881`), since the done-when is the recorded numbers.

## Stop and ask if

Stop and decide, never stop and wait.

- An instrument gate fails. That is a fault (D31): fix it before any null reading.
- The sheet's design cannot run as drafted. Then the FIT reviews the change.
