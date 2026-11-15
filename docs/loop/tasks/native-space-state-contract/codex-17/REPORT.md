# REPORT: native-space-state-contract, codex-17

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim native-space-state-contract --lab-topic q31-objective-compare` was refused with exit 2
  because `Cargo.lock` in this item's scope overlaps claude-391's PR #4009. This unit adds no
  dependency, does not touch `Cargo.lock`, and continued under the owner's D85 direction.
- **The alignment dot the earlier units reported is not the update's objective.** `step_reference`
  optimizes an L1 loss (`Q31ModelStep::loss_before`/`loss_after`, `q31_hamilton_train_step`); the
  dot is a different functional. On the fixed path both configurations *lower* their L1 loss while
  the dot falls, so "the alignment fell" (codex-6, codex-7, codex-12) never meant the update failed
  or that a gate configuration was worse. The dot is retired as the primary measure here.
- **Neither configuration moves a served token.** Shared gate and per-lane gates both serve
  `[125, 236, 138, 207, 8, 161, 92, 205]`, the untrained state's tokens; 0 of 8 moved.
- **Neither configuration improves the token-level rank.** Mean next-token rank starts at 123.57;
  the shared gate ends at 123.86 (2 of 7 transitions improved), the per-lane gates at 124.43 (1 of
  7). Both are marginally worse; the shared gate is marginally less worse.
- **Per-lane gates (A) are not supported by the live evidence.** The shared gate reaches a lower
  L1 on the same fixture (335,009,822,960 vs 336,129,862,535) and a better mean rank, with the same
  unchanged tokens.
- The token-level measure still has no signal to learn on this fixture: the mean rank is near
  chance, so a token-level *win* cannot appear here at all — only movement or its absence.
- The comparison test lives in `crates/uor-r4-integer/tests/q31_lane_objective.rs`, which the item
  scope does not list (`src/stack/**` only); `pr-check` reports the D76 advisory and passes.

## What changed

- `crates/uor-r4-integer/tests/q31_lane_objective.rs` gains one test,
  `the_shared_gate_and_the_per_lane_gates_are_measured_on_one_fixture`: the pooled single-gate
  objective (codex-6/7) and the per-lane objective (codex-12 A) trained on the same start-state
  records, served through `/12` and `/13` respectively, with the L1 objective, the dot, the served
  tokens and the next-token rank reported for both, plus the shared gate's per-lane breakdown.
- No source, dependency or fixture change; no existing assertion was relaxed.

Size: about 120 lines of test and the ORDER/REPORT.

## Measurements

Fixture: `synthetic_fbs_trunk(20_261_115)`, tokens `[3, 5, 8, 13, 21, 34, 55, 89]`, identity start
state, 64 shipped `step_reference` steps, 16 records per lane (two layers).

| Quantity | Untrained | Shared gate (`/12`) | Per-lane gates (`/13`) |
|---|---|---|---|
| L1 objective (its own measure) | `384,193,318,534` | `335,009,822,960` (−12.8%) | `336,129,862,535` (−12.5%) |
| Alignment dot | `3,673,307,283,483,328,512` | `3,192,833,317,191,155,712` | `3,236,691,098,920,681,472` |
| Served tokens | `[125, 236, 138, 207, 8, 161, 92, 205]` | same, 0 moved | same, 0 moved |
| Mean next-token rank | 123.57 | 123.86 (2 of 7 improved) | 124.43 (1 of 7 improved) |

Shared gate's per-lane alignment after training on the pooled records:
`[3,079,952,955,303,657,472, 1,249,974,901,779,464,192, −2,349,435,473,194,647,552, −28,738,899,498,500,096, 135,826,573,929,611,264, 868,852,771,816,079,360, −107,097,012,060,225,536, 343,497,499,115,716,608]`.

Per-lane alignment after training, lane 0..7:
`[3,087,280,004,029,218,816, 1,315,774,735,734,341,632, −2,429,816,936,107,278,336, −27,994,145,283,571,712, 152,883,339,764,957,184, 849,483,210,279,026,688, −80,633,699,113,697,280, 369,714,589,617,684,480]`.

No language, capability or GPU claim.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --test
  q31_lane_objective` — `4` passed (the three earlier tests plus the comparison).
- `cargo test --locked -p uor-r4-integer --features serving-wasm q31_serving` — `12` passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm --lib` — `663` passed, `1`
  ignored.
- `rustfmt --check` on the touched test file — clean; no pre-existing file reformatted.
- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm --lib --tests` — no
  warning in the added lines.
- `git diff --check`, the harness contract rule, the REPORT check and the exact-head review — at
  the head.

## Skills used

- `solution-integrity`: checked what the update actually optimizes before reading the dot's fall as
  a training result; the L1 column is the update's own quantity.
- `trace-back`: the earlier "alignment fell" was followed to its definition and to
  `q31_hamilton_train_step`'s loss.
- `scope-guard`: one test file, no source or fixture change.
- `test-freeze`: the file gains a test and relaxes none; the commit carries a `TEST-CHANGE:` line.

## Still open

- The token-level question needs a trunk whose writer rows encode a real next-token relation (the
  V11 state or the D86 donor), and a state large enough to move the served query. That is the
  GPU/FIT path; it must not compete with the donor pilot on wick.
- If gates are to matter at the token level, the objective must be defined on the readout rather
  than on per-lane writer blocks.
- codex-12's A/B/C order should be re-read against this unit: A is unsupported, and B (per-layer
  lanes) would add parameters without changing the measure's blindness.
- The D76 item-scope advisory for `crates/uor-r4-integer/tests/**`.

## Plain summary

On the fixed path, both the one shared gate and the per-lane gates cut their own L1 objective by
about 13%, so the shipped update works. But the alignment number the earlier units reported is a
different functional and was never the objective, and neither configuration moves a served token or
improves the next-token rank on this synthetic fixture. Per-lane gates are not better than the
shared gate on any live measure, so configuration A is unsupported; the token-level answer needs a
trained trunk and a readout-based objective.
