# #634 — feat(#598): indexed safetensors shards + declared BF16/F16/F32 codecs

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-12T04:21:15Z
- **Updated:** 2026-10-09T14:17:06Z
- **Closed:** 2026-08-12T07:59:55Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/634
- **Merged:** 2026-08-12T07:59:55Z
- **Merge commit:** f1127854d00853b98cc20d0ae0f34b401e44adba
- **Base / Head:** main / UOR-Foundation:i598-shard-ingestion
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

feat(#598): indexed safetensors shards + declared BF16/F16/F32 codecs

One deterministic validation boundary (SafetensorsSnapshot::open) for
both layouts: model.safetensors.index.json resolved so every tensor
maps to exactly one shard, with 9 focused error variants (missing
shard/tensor, duplicate incl. raw-JSON double-mapping, unexpected,
shape, byte-length, dtype inconsistency, unsupported dtype by name -
GPTQ/AWQ-style rejected, never approximated). Full header validation
pass precedes any decode. The single-file path is the synthesized
1-shard case of the same code path - no fork; single-file kappa stays
byte-identical to blake3(model.safetensors), verified by synthetic
test. codec module: exact-widening bf16/f16/f32 with 43 hex bit-pattern
table rows (normals, subnormals, +-0, +-inf, payload NaNs). Eager
flattened load retained; runtime untouched; no deps added. 21 new
tests. docs/MODEL_LIFECYCLE.md gains the sharded-snapshot section.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

