# #1819 — Diagnose source-state transport and frozen Generate consumption

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T01:53:28Z
- **Updated:** 2026-10-09T13:17:59Z
- **Closed:** 2026-10-07T02:10:43Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1819
- **Merged:** 2026-10-07T02:10:43Z
- **Merge commit:** 3141474d5ddde13a2e16c26bfe3733f43e0cdeb1
- **Base / Head:** main / UOR-Foundation:codex/source-state-interchange-20261007
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The retained joint-fit checkpoint exports a source-independent bridge, so changing which memory occurrence is selected cannot diagnose Generate's ability to consume that occurrence. Extend `geometric-entry-margin` with a zero-update source-state interchange mode that holds the query frame, full Copy inventory/scores, and Generate parameters fixed while comparing the original bridge, identity bypass, retained-master categorical transport, and exact source-copy transport.

The diagnostic preserves physical occurrence identity separately from cumulative H4 state, validates the retained checkpoint and categorical artifact, rebuilds the categorical map from the original masters, and checks factual native/learning and saved-report parity before label-assisted interventions. Its four fixed development rows are teacher-forced mediator evidence, not a prediction or attention qualification. The original entry-credit mode remains the default.

Validation at exact head `e155e7c`: changed-file formatting, CUDA-feature release check/build and six focused tests passed on Linux x86_64. Workspace-wide `cargo fmt --check` reports existing formatting differences in untouched main files. Actual RTX 5090 CUDA execution completed in 26 seconds with exit 0, four factual native/saved-report parity checks and unchanged masters; exclusive output was sealed and verified. Exact-head source review and independent actual-report review completed.

The measured controls show distinct stored keys and selected-source collapse in the original quarter bridge. Categorical transport restores source-dependent poststates and Generate vectors, but the frozen decoder is inconsistently calibrated and final tokens remain wrong. This diagnoses the next learning boundary; it does not qualify attention or prediction. Report and exact receipts are linked in the PR comments.

References #820.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6029248605 — 2026-10-07T01:57:19Z — Casey-allard

Exact-head self-review: `e155e7c5fc117aed02e7230b29ee4e6bca811355`. Independent source critic rereviewed this exact committed file and found no remaining blocker after the NativeArtifactBinding/SourceActionBinding admission mismatch was repaired. Scope: fixed teacher-forced source-state mediator diagnostic; no fitting or attention/prediction qualification.

Linux x86_64, Rust 1.97.1, CUDA-feature release build with CUDA_COMPUTE_CAP=120; compilation/tests used four CPU threads and CUDA_VISIBLE_DEVICES="" on the shared pod. Clean source tree, one shared Codex target directory. Executed checks:
- Changed-example rustfmt check: exit 0.
- `cargo check --release --offline -p uor-r4-training --features cuda --example geometric-entry-margin`: exit 0 (56.62s).
- Focused example tests: exit 0, 6 passed (175s compile, no GPU execution). Includes legacy config, occurrence multiplicity/ambiguity, signed source-copy exact transport, tiny-gradient metric receipt and exclusive-output boundaries.
- Release example build: exit 0 (4.18s).
- Workspace-wide `cargo fmt --check`: failed on untouched inherited main files; not reported as PASS and those files were not reformatted.

Preserved log: `/workspace/uor-r4/codex/source-interchange-e155e7c/focused-checks.log`. Executable SHA256: `30f61aaaa1b67ed04833816af2476f115ff3c7cc2e7a49239d9c1d9ff748fe66`. Bound source bundle SHA256: `0d8f1fe39aea157a2f66f5055d49363fd2638b3bdcd24f96e988fc77f18f5b1f`. Actual GPU diagnostic is next; these source/CPU checks are not its result. PR will not be queued until that boundary is exercised and reviewed.


### comment 6029392575 — 2026-10-07T02:09:31Z — Casey-allard

Actual exact-head CUDA execution at `e155e7c5fc117aed02e7230b29ee4e6bca811355`: RTX 5090 on canonical pod `vqj6ghxyhnw4jx`, CUDA 12.8 / driver 570.195.03; 02:06:11–02:06:37Z, **26s, exit 0**. Source tree clean; executable SHA256 `30f61aaaa1b67ed04833816af2476f115ff3c7cc2e7a49239d9c1d9ff748fe66`. Pod SSH-ready in 30s; released and deleted after completion. Exclusive output sealed and verified by driver; report SHA256 `52466a5a49523e1a27650b409be2b9be798b97811b291cadb0d6eb51bbbc6b40`, canonical `/workspace/uor-r4/codex/source-interchange-e155e7c/actual-checkpoint-attempt1/`. Job receipt `/workspace/uor-r4/jobs/codex/20261007T020611Z-source-interchange-gpu0.log`.

All four fixed rows 0/8/4/12 pass factual native/learning and available saved-diagonal parity. No updates; original master identities unchanged. Every row has 11 physical candidates with 11 distinct full H4 key states. Quarter and identity controls are exactly insensitive to within-row selected-key intervention. Exact source-copy reaches every selected key exactly. Categorical transport changes the relevant poststates and full Generate score vectors; row 4's factual and distractor arms name the same physical occurrence, so their equality is expected, not a collision.

Categorical correct-target versus same-record-wrong-offset raw Generate differences (score units): +0.375, +0.3125, -0.875, +0.5625. Versus distractor: +0.4375, +0.1875, -0.5625, 0. This is source-sensitive but incompletely calibrated readout, not a predictive success. Score/mass and full probability differ: e.g. row 8's positive raw contrast against the wrong offset still has a slightly negative probability contrast. Frozen Copy dominates; argmax remains wrong in these interventions. Source-copy is a positive geometric transport control, not a trained decoder control.

Independent actual-report critic review agrees: no demonstrated carrier collision; original quarter export collapses selected-source influence; categorical is admissible as a **frozen source-sensitive intervention** for paired raw/clipped context/potential/Generate learning restart. Do not bypass bridge freshness checks, train unchanged quarter transport as if source-sensitive, or promote this four-row common-prefix diagnostic to attention/chat/transfer. The next fit must use actual token-alias loss and target-free native own-prefix/source-swap prediction measurements, with raw target scores, mass and full probability reported separately.



---
_Comment/review ids in this file: 6029248605, 6029392575_

