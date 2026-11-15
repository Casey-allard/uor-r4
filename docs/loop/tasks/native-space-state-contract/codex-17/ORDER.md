# ORDER: native-space-state-contract, codex-17

Lab: codex. Item: [native-space-state-contract](../../../../roadmap/items/native-space-state-contract.md).
Parent: [codex-16](../codex-16/ORDER.md). Reviewed head: `e872c9d0a`.

## Why this unit exists

codex-16 found that the native Q31 path divided the trained term out of the carried state, so every
earlier objective run (codex-6, codex-7 and the codex-12 decision) measured a dead path. The path
is now live. This unit re-runs the **one shared gate** and the **per-lane gates** on the same
fixture, in one place, so the codex-12 A/B/C choice is made on live evidence instead of on the
withdrawn numbers.

## Claim record

`loop claim native-space-state-contract --lab-topic q31-objective-compare` was refused with exit 2
because `Cargo.lock` in this item's scope overlaps claude-391's PR #4009. This unit adds no
dependency and does not touch `Cargo.lock`; it continues under the owner's D85 direction.

## Method

- One fixture: `synthetic_fbs_trunk(20_261_115)`, tokens `[3, 5, 8, 13, 21, 34, 55, 89]`, the
  identity state's traced carries as values, the next token's per-lane writer block (Q12 shifted to
  Q31) as targets, 64 shipped `step_reference` training steps.
- **Shared gate (`/12`):** one gate trained on every `(value, target)` pair pooled across both
  layers and all lanes — exactly the codex-6/7 objective — served through
  `export_fbs_trunk_uorlut01_with_q31`.
- **Per-lane gates (`/13`):** K gates, lane `i` trained only on lane `i`'s records, served through
  `export_fbs_trunk_uorlut01_with_lane_gates` (codex-15).
- Both start from the same identity state and therefore the same records; both are served with the
  same `IntegerStackModel` and measured with the same token-level co-primary (next-token rank under
  the served `query_q12`, and the served tokens).
- The comparison test asserts only that both were trained and served; the signs go in the REPORT.

## Files in scope

- `crates/uor-r4-integer/tests/q31_lane_objective.rs` (extend; a `TEST-CHANGE:` line states why)
- `docs/loop/tasks/native-space-state-contract/codex-17/**`

Out of scope: source changes, `Cargo.toml`/`Cargo.lock`, the facade, CUDA/GPU, and any claim about
language, capability or model quality.

## Done when

1. The comparison test reports, in one run: pooled alignment before → after, per-lane alignment
   before → after, served tokens for each, and next-token rank for each.
2. It also reports the shared-gate model's per-lane breakdown, so the total is not hiding a lane.
3. The REPORT records the comparison with every sign, negative first, and names what the live
   evidence says about A (per-lane) versus the shared gate.
4. The test passes; the existing objective, serving and library tests still pass; formatting,
   touched-file Clippy, `git diff --check`, the harness contract rule, the REPORT check and the
   exact-head review pass.

## Cancel

Stop if the comparison needs a source change, a dependency, a GPU, or a fixture change. Do not tune
the fixture after seeing the result.

STOP and report when done.
