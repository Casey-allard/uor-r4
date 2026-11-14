# REPORT: infra-loop-harness, codex-3

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- The first D85 documentation pass left the live `docs/rules/README.md` describing §3.7 as
  “Float only offline, with an export gap”, and left the same historical route in the geometry
  guide and project tracker.
- The new native-state item initially used `crates/uor-r4-harness/src/**`, which covers shared
  harness files; `loop claim` would refuse the item even though it was marked ready.
- The tracked PICKUP still named the old main, old CUDA branch and no CUDA PR.
- The model-surface check accepted a page that retained the canonical marker and status but
  added a contradictory sentence.
- The architect skill's revised paragraph still retained the pre-D80 "no multiplier
  instruction" wording, and the tracked PICKUP had duplicate step numbering and old test counts.
- The architect skill pointer and native-space pointer required each other, creating a
  circular skill dependency.
- The repair-chain ORDERs omitted `Setup row: 2`, so the review gate correctly refused to count
  an exact-head APPROVE.

## What changed

The native-state item now lists only the specific non-shared harness source files it may need.
The live-rule guide, geometry guide, project tracker and decision map now distinguish the D85
native default from the retained historical float-master/export-gap evidence. The tracked PICKUP
now records current `main`, the ready control-plane PR and the stacked CUDA PR.

The model-surface check now rejects the two explicit contradictory claims that the fresh review
planted; a regression fixture covers the retained-marker case. The architect skill now carries
the current D80 integer-multiply permission, and the tracked PICKUP has a single current next-step
list and the latest check counts. The redundant architect pointer edge was removed, so the skill
dependency graph is no longer cyclic. No new rule or harness abstraction was added. The historical
evidence and old decision rows were not edited. All three unmerged ORDERs now name the same
`Setup row: 2`.

Size: 8 code/test changes plus 26 documentation changes across six files; this REPORT and the
separate ORDER are additional record material.

## Measurements

- Focused native-space contract: 8 passed.
- Full harness package: 685 passed, 0 failed, 1 ignored.
- Native-space rule on the real repository: 1 selected, 0 failed.
- `git diff --check` passed.

No model, training, serving or performance result was measured.

## Checks run at the head

- `cargo test --locked -p uor-r4-harness native_space_contract` - 8 passed.
- `TMPDIR=/var/tmp/uor-harness.KQCj53 cargo test --locked -p uor-r4-harness` - 685 passed,
  0 failed, 1 ignored; all integration and doc tests passed.
- `UOR_LAB=codex local/harness-target/release/uor-harness check --only
  harness.native_space_contract` - 1 rule, 0 failed.
- `CARGO_TARGET_DIR=local/harness-target cargo fmt --package uor-r4-harness -- --check` - passed.
- `CARGO_TARGET_DIR=local/harness-target cargo run --locked -p xtask -- links` - ran; 7,512
  sources and 27,142 links checked, with 1,288 repository-reported broken links and exit 0.
- `UOR_LAB=codex local/harness-target/release/uor-harness loop report --check --order
  docs/loop/tasks/infra-loop-harness/codex-3/ORDER.md` - passed.
- `git diff --check` - passed.

## Skills used

- `working-discipline`: reran the actual checks at the repaired head and kept advisory findings
  separate from errors.
- `uor-research-architect`: kept the repair at the D85 documentation boundary.
- `scope-guard`: removed the shared-file glob instead of widening the item around it.

## Still open

- `pr-check` may still print its advisory base-scope warning for the historical infra item scope;
  D76 makes that warning non-blocking.
- The next implementation is integer serving of the shared serialized Q31 model state.
- No model capability or speed claim is made.

## Plain summary

The D85 control plane is now claimable for the next item, its live rule summaries say the same
thing as the owner decision, and the handoff points at the actual CUDA PR. The full harness package
passes; no new validation layer was added.
