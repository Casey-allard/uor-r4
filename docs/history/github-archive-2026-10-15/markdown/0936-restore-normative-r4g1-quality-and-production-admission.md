# #936 — Restore normative R4G1 quality and production admission

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-25T05:46:40Z
- **Updated:** 2026-10-09T14:16:56Z
- **Closed:** 2026-08-25T06:10:40Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/936
- **Merged:** 2026-08-25T06:10:40Z
- **Merge commit:** e346816cb40b089583cbd3fff2cdd84924362c65
- **Base / Head:** main / UOR-Foundation:issue-933-normative-r4g1-quality
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Closes #933.

This restores one normative production candidate/token owner (`R4G1Runtime`), makes SKMX/PSIB reachable through the shared production adapters, and binds production admission to a typed, full-census deployed-quality report plus its exact comparator store and raw evidence.

The empirical verdict is **RATIFY only for the exact measured envelope**:

- canonical population: 72,130 positions
- normative: 21,293 / 72,130 = 29.5203%
- same-position TLA: 20,284 / 72,130 = 28.1214%
- paired difference: +13.988 permille, 95% CI [11.057, 16.919]
- sections absent: 18,806 / 72,130 = 26.0723%
- bounded-lane difference: +34.479 permille, 95% CI [31.681, 37.277]
- shuffled-label control: -229.003 permille, 95% CI [-232.359, -225.647]
- internal absent identity: 72,130 checks, 0 mismatch
- cross-surface: 72,138 checks, 0 mismatch
- witness replay: 64 / 64, 0 failure

This is not a universal 30% floor, live-teacher parity, instruction following, reasoning, factuality, semantic abstention, or free-running coherence.

## Production hardening

- Bind schema-2 admission to the exact deployed report, positions, nested comparator, graph, tokenizer, corpus, selector, decode, and raw evidence.
- Package and verify `tless_store.bin`; recompute the TLA comparator definition from its exact bytes and version.
- Parse the comparator store at the shared production boundary, rejecting malformed bytes even when an attacker rebinds every digest.
- Keep package, native load/install, strict empty-store startup, and static/WASM inventory on the same fail-closed envelope.
- Add planted substitution, mutation, malformed-store, version, position, and browser-inventory negatives.

Canonical graph CID: `blake3:ff82dfd5f04eac7e944443b1ea4cc9fe93a007b3b8f07286876d52709a98bc49`

Full report CID: `blake3:88ee8210e1f4c48dc26999f5685350b2d2343676cdbd6f9b1aee7c7f1c66146f`

Hardened manifest raw BLAKE3: `c2025e9e507e8367993d78bd83ef099ce5851c838d3cc5cf01eda5560986ad33`

## Reproducibility and admission evidence

Independent 4-worker and 8-worker clean emissions reproduced the graph, controls, 6k/18k/full reports, parity artifact, witness artifact, teacher/store/tokenizer inputs, and hardened release manifest byte-for-byte.

Strict production startup from an empty model store admitted the exact schema-2 bundle through the ordinary release loader. Earlier bypass-loader and pre-hardening results are diagnostics only.

## Local verification

- `cargo test --workspace --no-fail-fast --offline`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo fmt --check`
- graph-format no-default and alloc ladders
- graph-runtime no-default ladder
- wasm32 library check
- claim-wording and 33-ID conformance checks
- non-vacuous canonical κ reproduction: 3 / 3
- BDD: 124 / 124

The local `cargo-audit` subcommand was unavailable, so local audit is **NOT_RUN**, not PASS. The repository's required `cargo audit` check in the protected merge queue is the binding verdict.

The BDD live-teacher parity fixtures were absent. Those fixture-gated scenarios vacuously skipped, so the 124 / 124 BDD result is **not** live-teacher parity evidence; #932 remains the downstream observable exact-parity work.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

