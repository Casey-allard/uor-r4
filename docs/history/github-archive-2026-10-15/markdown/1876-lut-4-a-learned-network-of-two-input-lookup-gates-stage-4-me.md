# #1876 — LUT-4: a learned network of two-input lookup gates (Stage 4 mechanism, no language gate yet) (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T05:02:06Z
- **Updated:** 2026-10-09T13:21:00Z
- **Closed:** 2026-10-08T05:03:46Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1876
- **Merged:** 2026-10-08T05:03:46Z
- **Merge commit:** e568e98427499bfbe01513b4e14b3547495b42c8
- **Base / Head:** main / UOR-Foundation:deepseek/lut4-module
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this adds

Stage 4 of `native-core-transition-plan.md` names the deepest limit: the serving scorer is purely additive — seven summed terms with no learned feature interaction — so its ceiling is n-gram/backoff quality. This adds the mechanism that supplies the missing interaction.

`crates/uor-r4-core/src/native_geometric/learner/lut4/` — a learned network of **two-input lookup gates**:

- **Served form is a truth-table read.** Each output reads two inputs through one of the sixteen two-input Boolean gates, exported as a 4-bit gate id plus its wiring. `Lut4Artifact::eval` is integer index operations and one table read per output — no multiplier, no float, no dense contraction.
- **Exact soft form for training.** Every Boolean function of two variables has a unique real multilinear extension over `{1, a, b, ab}` with integer coefficients, so the soft forward is the exact mixture over the sixteen gates (AND is `ab`, OR is `a + b − ab`, XOR is `a + b − 2ab`). Export takes the argmax gate and wiring; the soft-to-hard difference is the **discretisation gap**, reported alongside **gate utilisation** and entropy, as the plan requires.
- **Identity/residual initialisation** on square layers, so a fresh network starts as an exact pass-through.

## The two diagnosed failures, addressed and measured

Both come from the September audit (`docs/integration/review-2026-09-16/04-mathematics-audit.md:252`), which lists among the repository's correct self-diagnoses: *"LUT4 has a 64-input cone and 256-bit bottleneck."*

- **Cone.** Depth composes gates and each layer's two inputs are chosen by **learned** connectivity, so an output at depth `L` reaches up to `2^L` inputs. `Lut4Network::cone_sizes()` reports the reachable input count per output, and it counts only the slots the chosen gate actually reads — so an identity layer measures cone 1 rather than pulling in an input it ignores.
- **Bottleneck.** Width is per-layer and declared by the caller; nothing is forced through a fixed bus.

## A collapse the training found, and the fix

The connectivity utility can drive both slots for an output to the same input. At that point **no two-input gate is expressible** and training stalls at the best constant — measured on XOR as a loss pinned at exactly 0.25. Wiring is now kept distinct and bounded (`enforce_wiring`), and the XOR test is stable at 20,000 steps because of it. That is recorded in the module rather than silently patched.

## Verification at the exact head

`cargo test -p uor-r4-core lut4` — **6 passed / 0 failed**:

- every gate polynomial matches its truth table exactly, on all 16 gates × 4 input pairs;
- identity initialisation is an exact pass-through with cone 1 per output;
- depth widens the cone (`2^L` reach at the widest declared stack);
- the exported artifact equals the served table read;
- XOR is learned, its chosen gate is 6, the hard export computes XOR exactly on all four rows, and the discretisation gap is reported (< 5e-3 at saturation);
- utilisation and entropy report a collapse.

`cargo check -p uor-r4-core --all-targets` clean; `rustfmt` clean; the PR is the module plus its registration, nothing else.

## What this is not

**Not a language result.** The plan's gate for Stage 4 — improvement on *real* held-out text (not TinyStories-shaped), with the discretisation gap and utilisation reported — is the **next** step and has not been run. This lands the mechanism and its unit evidence only; the insertion as a learned nonlinear term over the shortlist feature vector, and the held-out measurement, follow.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

