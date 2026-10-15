# #1849 — Retain complete frozen prediction traces within the report allowance

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T17:57:27Z
- **Updated:** 2026-10-09T13:17:44Z
- **Closed:** 2026-10-07T18:05:13Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1849
- **Merged:** 2026-10-07T18:05:13Z
- **Merge commit:** 69dba2679ae87507351fa836504c3d6013655772
- **Base / Head:** main / UOR-Foundation:codex/native-transfer-report-capacity-20261007
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The frozen 32-case evaluator retains all legal-token scores and complete action traces. At its 32-token output limit those reports exceeded the old 1 GiB ceiling. Allow an explicit report budget up to 4 GiB and default to 3 GiB; retain the existing failure/seal reserve and every trace. No model, candidate, label, data, or serving behavior changes.

References #820.

Validation at exact head ec5d5fd2f1c4494d0bd66ea50d2f35afa2d9d039: independent source review, touched formatting, release example check, five focused tests (zero ignored), and release build passed on one spare CPU thread of an existing Linux x86_64 pod. The conservative serializer projection was 2.323 GiB; the completed fresh report used 1.922 GiB. Model results and their limitations are recorded separately on #820 and in #1850.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6043836483 — 2026-10-07T18:04:16Z — Casey-allard

Exact-head self-review at ec5d5fd2f1c4494d0bd66ea50d2f35afa2d9d039. This changes only the default/accepted report allowance, retaining the same output claim, full traces, 1 MiB seal reserve, serving path and data. Independent source/serializer review passed; conservative worst-case trace projection is ~2.323 GiB, below the declared fresh-run 3 GiB allowance. Owning resource projection increases new storage to 6 GiB on the existing remote volume; no extra GPU rental.

Executed on Linux x86_64 wp23upwpazsjes, Rust 1.97.1, clean git archive of this exact committed head, one spare CPU thread, CARGO_TARGET_DIR=/root/codex/target, CARGO_INCREMENTAL=0, UOR_BUILD_SOURCE_COMMIT pinned to this head:

- rustfmt --edition 2021 --check crates/uor-r4-core/examples/native-bank-generalization.rs: PASS.
- cargo check --locked --offline --release -p uor-r4-core --example native-bank-generalization: PASS.
- cargo test --locked --offline --release -p uor-r4-core --example native-bank-generalization: PASS, 5 executed / 0 failed / 0 ignored, including target-free native snapshot, source occurrence preservation, scope/missing-source rejection and output ancestry checks.
- cargo build --locked --offline --release -p uor-r4-core --example native-bank-generalization: PASS.
- git diff --check: PASS.

Receipt: /workspace/uor-r4/codex/frozen-native-transfer/checks-capacity-ec5d5fd2f-attempt1/receipt.json. No CUDA kernels changed; these are core instrument checks. Earlier exact-source evaluator replay at 3c36bc2a2 reproduced 8/8 original complete replies and all saved native trace comparisons. The fresh 32-case model result is a separate measurement, not a test-suite claim.



---
_Comment/review ids in this file: 6043836483_

