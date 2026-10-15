# #1044 — Qualify position-preserving R4 K/V binding (#1043)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-02T05:16:49Z
- **Updated:** 2026-10-09T14:16:50Z
- **Closed:** 2026-09-02T05:17:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1044
- **Merged:** 2026-09-02T05:17:55Z
- **Merge commit:** 77536d9ea1434dc7b92b3025bff32577a823252d
- **Base / Head:** main / UOR-Foundation:issue-1043-position-kv-binding
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Implements the one frozen `R4PositionPreservingCausalKVBindingV1` campaign for #1043 and records its immutable terminal result.

The mechanism preserves one causal K/V slot per position, exposes full and incremental cache-backed execution, and runs the same fitted weights through ordinary stable-softmax attention and coherent R4/H4 frame transport. Preparation, preflight, one 2,730-step fit, one reveal, scoring, replay, and fresh-process verification completed under the selected four-thread Apple Accelerate CPU plan.

Terminal: `INVALID_POSITION_KV_BINDING`

- result CID: `blake3:96a382d2f6118fbd2883fbee8b383764de0d6098efbf82bf863412c6996d80d0`
- artifact CID: `blake3:e34c211687076662740bced155b58352d325c79c0543d0f9948b9412b120527c`
- exact work/leakage/replay: pass
- R4/plain attention delta: `1.6689300537109375e-06 <= 2e-6`
- R4/plain logits: `2.193450927734375e-05 > 2e-5`
- R4/plain top-1: identical over `257,136` decisions
- R4 full/incremental logits: `1.239776611328125e-05 <= 2e-5`, top-1 identical

A numerical audit found expected f32 contraction reassociation, not a transport, cache, mask, or causal defect. The threshold and implementation identity remain frozen; the result is not retuned or reclassified.

A read-only construction diagnostic showed that a parity-only recovery has no decision value: fitted MQAR was `30/87,360`, English supplied-history was `578/8,190`, and no-history abstention was `2,730/2,730`. The artifact did not learn associative retrieval on construction, so this PR records the result and directs a future, separate role-tagged associative-first curriculum ladder before any new sealed fit. No generation, recurrence, lowering, correctness, or reasoning work is authorized.

## Scope

- adds the exact position-preserving K/V model, data lifecycle, frozen campaign, CLI commands, and focused tests
- binds implementation/data/preflight/fit/reveal/scoring/result identities
- publishes the byte-identical structured result and updates current research/programme/roadmap/glossary mirrors
- preserves the original production root, artifact, single reveal, thresholds, and invalid result
- does not change Rust runtime, WASM/browser, production serving, #1017, #954, or release behavior

## Verification

- focused #1043 Python suite: `25 passed`
- Python compilation: pass
- claim-wording gate: pass
- `cargo fmt --check`: pass
- workspace all-target/all-feature clippy: pass
- graph-format no-default and alloc ladders: pass
- workspace test ladder: all relevant suites passed; one unrelated server fixture transiently returned `BUSY` on its unique temporary producer lock, and its exact prebuilt test plus lock drop/reacquire regression passed immediately in isolation
- terminal fresh-process verifier: pass; reproduced result CID exactly
- structured result copy: byte-identical to the create-once production result

Closes #1043


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

