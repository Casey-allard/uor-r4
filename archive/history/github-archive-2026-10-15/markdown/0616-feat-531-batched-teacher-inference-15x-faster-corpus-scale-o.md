# #616 — feat(#531): batched teacher inference — ~15x faster corpus-scale observe

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-11T02:38:38Z
- **Updated:** 2026-10-09T14:17:01Z
- **Closed:** 2026-08-11T02:57:51Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/616
- **Merged:** 2026-08-11T02:57:51Z
- **Merge commit:** 02be0235063ecc5a3a831031df69bc600eb5287d
- **Base / Head:** main / UOR-Foundation:r5/batched-observe
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## #531 — batched teacher inference for the corpus-scale observe

The teacher observe was memory-bandwidth bound: every token re-reads the full ~1.4 GB of teacher weights, so autoregressive inference of a 360M teacher runs at ~20 tok/s and thread-level parallelism only bought ~1.3×. This batches independent articles through one forward, so each weight read serves the whole batch.

**Measured on an M1 (360M teacher, Accelerate):** serial ~20 tok/s → batch 8 **3.6×**, batch 16 **7.3×**, batch 32 **15.1×** (~910k records/hour → **2M observe ~2.2h** vs ~28h serial). One shared weight copy, not N.

### What's here
- `matmul_batched` + `Llama::forward_batch(states[B], tokens[B], positions[B])`: the six weight matmuls (Q/K/V, output, MLP, vocab) run once over the batch; per-sequence rmsnorm/RoPE/attention (own KV cache + position)/SwiGLU/residual mirror `forward()` exactly. Off macOS it reuses the serial `dot_fast` (bit-identical); on macOS it's Accelerate `cblas_sgemm`. Teacher data, not the pinned legacy proof.
- `BatchedTeacher` trait (impl by the HF oracle) + `observe_text_corpus_batched`: fixed-group continuous batching — gather up to `--batch` articles in ordinal order, step them together to the group's longest, encode each active position via the shared `encode_position`, commit in ordinal order. Reuses the extracted `prepare_text_observation` / `finalize_text_observation` / `commit_article`, so checkpoint/resume/reporting are identical to the serial path.
- `observe-text --batch N` (Hugging Face teacher only; errors on `--checkpoint`). Supersedes `--workers` for throughput.

### Correctness
- `matmul_batched_matches_serial_fast` and `forward_batch_matches_serial_forward` — bit-identical to the serial path (off macOS).
- `batched_matches_serial_byte_for_byte` — a fake teacher whose logits match `FakeOracle` observed serially vs at batch 4: merged stream, κ, every shard, and `stories.jsonl` are byte-for-byte equal.

Full gauntlet green: build, `--all-features`, `cargo test --workspace --no-run` (incl `tests/bdd.rs`), `clippy -D warnings`, `fmt`, `check-model` / `audit-deferral` / `audit-limits`.

Refs #531.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

