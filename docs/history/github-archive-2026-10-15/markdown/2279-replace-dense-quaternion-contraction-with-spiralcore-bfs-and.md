# #2279 — Replace dense quaternion contraction with SpiralCore BFS and retained FBS geometry

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T03:45:46Z
- **Updated:** 2026-10-13T04:13:03Z
- **Closed:** 2026-10-13T03:55:27Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2279
- **Merged:** 2026-10-13T03:55:27Z
- **Merge commit:** c6cd409daab2cd35898cdb9750db1677e15f668c
- **Base / Head:** main / UOR-Foundation:codex/spiralcore-fbs-projection-20261012
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The quaternion projection still summed learned Hamilton coefficients across every input block after its GEMM was removed. Replace that dense contraction with retained Atomic Ternary FBS children/frontier and actual SpiralCore BFS signed transport: each output selects and transports one complete 8D carrier, with no learned input/output coefficient grid.

`projection=spiralcore_fbs` uses distinct F32 routing-point parameters and F32/BF16 CUDA carriers. Its selected-input adjoint is exact; routing credit is an explicitly labelled geometric root-edge secant surrogate, with complete sibling rerouting. Artifact bindings authenticate compiler/kernel/source geometry, retained children, BFS histories and shapes. CPU/Metal fallback, stale/undeclared bindings, unsupported snaps and native dense-head fallthrough refuse execution. Old quaternion contraction/expansion entry points refuse; their source, adjoints and tests are preserved as a restorable indexed patch. The local G2 Hamilton gate and peer work remain.

The owner explicitly authorized this bounded pod development/build/finite kernel qualification and private HF setup storage. Training remains held. Update AGENTS, contributor/session instructions, readable/machine policy, operator plan, milestone cache, status/roadmap, mechanism briefs and canonical record. The private HF store now contains the verified build-only setup recipe, qualification logs and checksum manifest.

Validation on CUDA12.8/Ubuntu24.04.3/Rust1.97.1, one RTX5090:

- Four GPU-only controls pass: F32/BF16, all27 retained terminals, signed carrier orientation, exact selected-input adjoint, analytic secant, shared2-row/3-output credit and numerical/shape/CPU refusal. Missing CUDA fails; references execute on GPU.
- Three static geometry/refusal tests, two parameter-preparation/artifact-binding tests and the retired-entry refusal pass. CUDA example check and retained parity target compile-only pass; its CPU-reference/model suite was never executed.
- Product source is identical to the production-qualified freeze `0a889bb19`; four device tests ran at `c2ebb1e32` (later test-target change removes an orphaned comment only). Touched-file rustfmt, claim wording, authority JSON and non-patch whitespace checks pass. Source ratchets pass1649 softmax sites and93 dense-product sites, down from95; these partial ratchets do not certify the full path.
- Old-source restoration reproduces the named pre-replacement source bytes. iCloud QA604672B/MD5 `0e60182d9425c7f5b30d23a9d5b559eb` and five-file HF SHA256 download round-trip pass. Failed attempts remain preserved. Pod released/deleted,1148s at$0.536/h≈$0.170924 before billing granularity.

This qualifies the finite operator, not useful language, common-writer completion, surrogate learning fitness, whole-model compliance or D11 serving. OP3a/full lawful path and explicit applicable training-hold release remain dependencies. No optimizer updates, CPU model reference, corpus run or quality measurement. Unified line count7/3→8/3, no headline reset; M1 memory20/40/reply37/232 and M2 saved437/512/opened0/128 remain unchanged, genuinely fresh NOT_RUN.

References #2029, #2030, #2028.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6137866931 — 2026-10-13T03:54:33Z — Casey-allard

EXACT-HEAD REVIEW — PASS (owner-permitted author self-review)

Reviewed head `61ec567a603b578789c081132636aea01afcca63`, tree `4da4cb4a5c95703a3d59ba3df810e9dcdf51c295`. Runtime source under `crates/` has no difference from the production-qualified freeze `0a889bb19`; kernel-test source is identical to `c2ebb1e32`. Current-head formatting, whitespace excluding literal restorable patches, authority JSON/claim wording and source ratchets pass. Local source-identical library/example check and six static/artifact/refusal tests pass; the device evidence is the actual production CUDA build and four GPU-only controls, not the earlier PTX-disabled type check or compatibility acknowledgements.

Traced: native mode remaps fresh projection variables into distinct query shapes; params refuses CPU/oldAll; custom forward/adjoints dispatch directly to CUDA; static metadata contains retained complete children/frontier and actual BFS compositions; runtime moves one carrier and preserves signed operator state. No all-input learned coefficient sum, hidden GEMM, normalized exponential or host activation/reference arithmetic exists in this operator. GPU references use explicit 8-coordinate gather/sign and geometric contrast/edge operations. Device status copies are validation only. Local G2 source is preserved. Native dense head fallthrough and old contraction/expansion APIs refuse. Save/load binds actual compiler/kernel/geometry/children/shapes and rejects unsupported/stale artifacts.

Scientific scope: the query secant is an explicitly declared surrogate, with no descent/language fitness claim; this couples the FBS binder abstraction rather than claiming the full marked FBS object. Device tests cover their stated shapes/dtypes/controls. No optimizer, CPU model reference, training, full-model generation, new comparator or quality panel ran. Remaining full-path/source ratchet sites and OP3a/common writer are unresolved dependencies. All training holds remain.

Preservation reviewed: indexed inactive patch applies at the pinned base and restores named old-source bytes; historical artifacts/measurements and peer merges retained. CUDA source/test executable hashes, failed attempts, actual costs and iCloud/HF download verification are recorded. Pod is released/deleted. The private HF setup recipe is build-only and carries no training authorization. Count7/3→8/3 reflects this source unit, no headline reset.

Delivery may enter the protected queue. Queue acknowledgement names execute no model tests. Verify actual merge commit and whole-tree/delivered-source identity on fresh origin/main before claiming delivery, then remove only this redundant branch/worktree and preserve closeout receipts.


### comment 6137960295 — 2026-10-13T04:02:42Z — Casey-allard

PASS: protected merge present on fresh origin/main; complete merge tree equals independent merge of peer base and reviewed head; all production-qualified crates identical. The merged tree also contains the intervening peer #2280 harness claim-store repair.



---
_Comment/review ids in this file: 6137866931, 6137960295_

