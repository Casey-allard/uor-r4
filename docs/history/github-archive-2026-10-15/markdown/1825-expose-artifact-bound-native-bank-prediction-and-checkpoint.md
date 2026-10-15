# #1825 — Expose artifact-bound native bank prediction and checkpoint replay

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T03:57:23Z
- **Updated:** 2026-10-09T13:17:40Z
- **Closed:** 2026-10-07T04:12:43Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1825
- **Merged:** 2026-10-07T04:12:43Z
- **Merge commit:** dd6f4f4b0a5ee0e6c83b7b8ff02d6dc066ebc4cc
- **Base / Head:** main / UOR-Foundation:codex/native-bank-generator-20261007
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Native full-bank prediction currently lives inside an experiment evaluator. Add an artifact-bound core API that takes a pinned, ordered Context/Source bank and performs integer geometric Copy/Generate scoring with actual emitted-token feedback. Callers no longer need a target sequence, selected-source label, training graph or evaluator JSON to invoke this numerical path.

The API authenticates source/Generate/bridge/cue/prefix artifacts, preserves occurrence order and source provenance, rejects unresolved/conflicting/evicted sources distinctly, accounts for compiled emission-view token counts, and separates EOS, output/context limits and retained trace limits. It preserves the full alias-aware vocabulary competition. Host construction and replay allocate; this does not qualify complete D11 cost, chat capability or model quality. The existing 128-token/4,096-ID limits remain.

Add an eight-row retained-artifact replay example, plus a complete-file report verifier. Saved canonical prefixes are explicit evaluation conditions; separate free generation starts empty and uses actual chosen IDs. The saved reference lacks full per-atom token masses, so their direct parity remains unavailable. Source/context pairing and snapshot completeness require an authenticated caller; this PR does not edit GroundedSession or add an actual-store adapter.

Validation at final head 267585e6: changed-file formatting, release library/example check, five focused core tests, three example/path tests and release build pass on Linux x86_64. Authenticated checkpoint replay passes all eight declared rows: 84 canonical-prefix positions and 256 actual own-prefix positions. Complete BLAKE3 report verification passes for the new replay, retained zero-update baseline and completed first clipped fit. Package-wide formatting reports pre-existing differences in the unrelated addressed-attention-arith-fit binary; those files are untouched. Source review found and repaired a sealed-output ancestry defect before execution.

References #820. No training run or geometric-advantage claim is introduced.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6030753845 — 2026-10-07T04:11:29Z — Casey-allard

Exact-head self-review: `267585e6beed43b5cb6464a5796a357d9e6243be`, clean committed tree, Linux x86_64 on shared pod `ianty3o30vhgew`; CPU-only integer component replay, no D19 floating session or CUDA parity claim. Independent source review approved module SHA `9f18ab5ee6928e127e60102235b4ddd6fbd5c6eb149e5f035797b716d6734b95`. Adversarial review found the replay's sealed-ancestor/symlink defect; repaired driver SHA `d92be8b0058203c5f783528b7e32a1796444694882594d9659bb791d9642c158` received separate scoped review before compilation.

Actual checks at this head:

- Changed-file rustfmt check and `git diff --check`: PASS. Package `cargo fmt --check -p uor-r4-core` was also attempted at the preceding API commit and fails on unchanged main formatting in addressed-attention-arith-fit; no package-wide PASS is claimed.
- `cargo check --release -p uor-r4-core --lib --example native-bank-generate-parity`: PASS.
- `cargo test --release -p uor-r4-core --example native-bank-generate-parity`: 3 PASS, including sealed ancestry, symlink and no-mutation path checks.
- `cargo test --release -p uor-r4-core --lib native_bank_generate::`: 5 PASS, covering actual feedback/EOS/context caps, provenance/status/sparse token admission, occurrence order, Copy sums/ties/aliases and artifact/budget checks.
- `cargo build --release -p uor-r4-core --example native-bank-generate-parity`: PASS. Executable SHA256 `fdc0c3f803c59e3ea6bb92f2e4c893c1eca64d225b66181f0a6b123296b1492c`.
- Authenticated retained checkpoint replay: 8/8 declared construction rows PASS, 84 canonical-prefix and 256 actual-feedback positions. It compares full raw Copy/order/component evidence, retained state, all Generate scores via saved digest, bridge and canonical reducer summary. Direct comparison of omitted per-atom pool masses remains unavailable.
- Independent complete-file BLAKE3 verification of replay, zero-update admission and completed seed1001/clipped fit: PASS, exit 0 for each.

Replay report SHA256 `32a37e20569b504ec2384ae8b7fffa644b1b01cc452d7f88606b9b8fe6637922`; receipts and logs under `/workspace/uor-r4/codex/native-bank-generator-267585e6/`, executable retained under `/workspace/bin/267585e6beed43b5cb6464a5796a357d9e6243be-cpu/`. Shared `/root/codex` is 4.5 GiB, below the 10 GiB ceiling. Build used the shared target, four Cargo jobs; replay used one Rayon thread. No laptop build or model artifact download occurred.

Review conclusion: suitable to merge as a reusable artifact-bound numerical API with construction parity. This does not establish chat, transfer, native whole-path opcode/energy qualification or actual-store snapshot completeness. Raw Context/source authentication remains a caller obligation; no GroundedSession seam changed. Existing failed model results remain retained.



---
_Comment/review ids in this file: 6030753845_

