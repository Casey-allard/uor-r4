# #2495 — D19: exact Showcase-compatible byte object store and occurrence bindings

- **Type:** Pull request
- **State:** open
- **Author:** @Casey-allard
- **Created:** 2026-10-15T02:23:00Z
- **Updated:** 2026-10-15T02:23:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2495
- **Merged:** not merged
- **Merge commit:** fce29553881c02b224d41028148d9b5fcbf57250
- **Base / Head:** main / UOR-Foundation:codex/d19-object-store-impl
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline

Line: D19 durable-memory infrastructure · count 0/3 unchanged · headline 437/512 → 437/512 (retained M2 dev record, no new model measurement).

## What and why

Implement `uor-r4-object-store` under the approved #2487 contract and PM's five requirements. It preserves Showcase records and exact ordered reconstruction while adding deterministic exchange, bounded reference reads, atomic import/persistence, and object-version/tokenizer/occurrence bindings. The only shared tokenizer changes are two allocation-free metadata accessors; encoding and decoding behavior is unchanged.

DeepSeek owns the production DAG reader. Its `Backend::node/stored` adapter was compiled against published source `b32651b82`; PM provisionally ratified the seam in #2028 comment6182842815 while DeepSeek is offline. DeepSeek confirmation is still owed on return; this does not declare either API stable or mutate its branch.

## Result and scope

KEEP as a source/data-structure prototype; NOT YET PROMOTED into the model. Twenty object-store byte controls and thirteen existing tokenizer tests pass, plus focused Clippy/fmt and the exact-source reader adapter. Original supplied ZIP/fixture hashes are pinned and regenerated source bytes match the original record answers. Raw/retained/new/visited bytes are separate counts: unique data expands (32,768 to70,972 record bytes), repeated content deduplicates (7,776 to380); this is not a universal compression or throughput claim.

D24 data-preparation/storage scope, explicitly no FIT required by PM [6181310800](https://github.com/UOR-Foundation/uor-r4/pull/2487#issuecomment-6181310800). No model, GPU, pod, rental or training; seed-2/M2 HELD, already released seed-1 untouched. No model callers, head, writer, schedule or headline changed.

## Validation

- `CARGO_TARGET_DIR=local/object-store-target cargo test -p uor-r4-object-store -p uor-r4-tokenizer --offline`
- `CARGO_TARGET_DIR=local/object-store-target cargo clippy -p uor-r4-object-store --all-targets --offline -- -D warnings`
- `cargo fmt -p uor-r4-object-store -p uor-r4-tokenizer --check`
- Byte-only usage and restart example; actual #2489 adapter check, source and lock retained in record.
- `python3 scripts/check_claim_wording.py`; `uor-harness ratchet d23 --quiet`; `uor-harness ratchet d25 --quiet`; shard integrity check.
- Canonical record: docs/labs/d19-object-store-2026-10-15/README.md; newest-first current-state entry; README repository map and PROJECT_MAP updated. STATUS/ROADMAP and #2028 milestone table unchanged because no model capability changed.
- Exact-head review/check receipt posted on this PR; protected merge only. After merge: verify fresh origin/main, preserve evidence, release/delete owned branch/worktree, run hygiene.

References #2028, #2030, #2487, #2489

Harness check: 49 rules, 15 findings (0 errors, 14 warnings and 1 informational finding). Warnings cover shared stale/released peer claims, offline presence, historical unknown receipt/unfit sheet, unstarted panels and an M1 headline parser mistaking a gate-index fraction for a score. No peer state was rewritten to make these disappear. The specific source-data approval above applies; no new model FIT or hold release is claimed.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6182967991 — 2026-10-15T02:23:51Z — Casey-allard

**Exact-head source review: APPROVE — `c8441c0c648b221b5dc4de4efb5ec171d1f6ac22`.** Independent Codex review agent inspected the committed core, bindings, allocation-free tokenizer accessors and traversal preflight; no blocking findings. This is a source review, not a FIT verdict or training release.

Author-executed checks at this exact head: **20/20 object-store integration controls and 13/13 existing tokenizer tests PASS**, zero ignored; focused Clippy (`--all-targets -- -D warnings`) and fmt PASS. Earlier same-source byte example/restart and actual #2489 reader adapter PASS; retained adapter identities/lock/log are in the dated record. The independent reviewer did not claim to execute those tests.

Review covers record/hash validation, canonical ternary frontier and ordered multiplicity, limits before expansion/allocation, transactional import/fold/persistence, missing-frontier semantics, partial-vs-whole integrity types, exact original-byte ingestion and version/tokenizer/occurrence binding. Two explicit limits remain: imported binding metadata requires its own trusted provenance, and short BLAKE2b-96 addresses are not collision-free or hostile-network security certification.

Harness: 49 rules, 15 reported findings, **0 errors / 14 warnings / 1 info**. Source word gate, D23/D25 ratchets and ledger shard integrity passed. PR lint has 0 errors and one registry-line warning because this owner/PM-assigned D19 infrastructure line is not the retained M2 model line; it was not renamed to imply model progress. All holds unchanged; no model/GPU/rental/compute occurred.



---
_Comment/review ids in this file: 6182967991_

