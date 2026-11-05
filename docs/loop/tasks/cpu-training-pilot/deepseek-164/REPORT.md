# REPORT: cpu-training-pilot, deepseek-164

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `faf597e93`.

## Status

Complete. Released the finished longest-32 exact-copy claim and recorded final
delivery verification.

## What did not work

None at the bookkeeping level. The experiment itself remains a bounded
negative: deeper exact matches tied nested longest-4 on scored body positions.

## What changed

- Removed `deepseek/copy-depth32` and the temporary release claim.
- Recorded exact-head review, merge and fresh-main checks in deepseek-163.
- No model, source, serving, CUDA, float-training or Claude-owned file changed.

## Measurements

- reviewed head: `61952d866d09e992d9122bc74850ad10ec4f3778`;
- merge: `faf597e931e448c9be2c0d33de1f2d8b48ddc8db`;
- fresh-main CPU library tests: **288 passed**, 4 expected ignores;
- fresh-main binary tests: **82 passed**;
- published depth-32 evidence and no-corpus check passed.

## Checks run at the head

- Claims JSON parses and no unrelated claim is removed.
- Report, scope, policy and `git diff --check` pass.
- Fresh main contains the merged evidence and code.

## Size

Against base `faf597e93`, this release changes four bookkeeping paths
(**+85 / -7**).

## Skills used

No system-provided Codex skill applies. Project delivery procedures were used.

## Still open

Test a non-exact geometric copy/read, then combine selected content with
independent termination.

## Plain summary

The result and evidence are merged. This PR closes the coordination claim.
