# #555 — feat(#510): R5 tranche — graph-compiler observation.rs host-I/O to SourceUnavailable (23→4)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T20:34:08Z
- **Updated:** 2026-10-09T14:17:26Z
- **Closed:** 2026-08-09T20:53:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/555
- **Merged:** 2026-08-09T20:53:29Z
- **Merge commit:** f578a9f4decde1ca9dbb8f6f510d000c1223fc93
- **Base / Head:** main / UOR-Foundation:r5/gc-obs
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

R5 rearchitecture (#510): second host-I/O module, the largest (19 `io::Result` sites). Convert the observation shard-spill/merge pipeline off `io::Result` / `Result<_, String>` to `Result<_, SourceUnavailable>` — reads (manifests, checkpoints, sidecars) and writes (shard files, `state.bin`) are the host-ingestion/persistence boundary. **The module's I/O surface is now fully sanctioned (0 audit sites).**

## Highest-leverage move
The two private helpers `invalid_input` / `invalid_data` now return `SourceUnavailable` instead of `io::Error`, so nearly every `return Err(invalid_*(...))`, `.ok_or_else(|| invalid_*(...))?`, and `.map_err(|e| invalid_*(...))` site converts unchanged. Raw `?` on fs/serde calls auto-converts via `SourceUnavailable`'s `From<io::Error>`/`From<serde_json::Error>`; the three bare `Err(error)` io-propagations use `Err(error.into())`; the seven `.map_err(|e| e.to_string())?` in `observe_sharded` collapse to `?`.

## Converted
`file_kappa`; `ObservationManifest::load`/`store`; the entire `ObservationShardWriter` impl (`open`, `set_partition_rule`, `set_input_cid`, `restore_partition_counts`, `write_record`[`_in_partition`/`_with_probability`/…], `flush`, `finish_shard`, `finalize_all`); `merge_shards`; `merge_probability_metadata`; `reconcile_probability_shard`; `reconcile_probability_pair`; `observe_sharded` (was `Result<ObserveSummary, String>`).

## WASM
`SourceUnavailable` is itself `#[cfg(not(target_arch="wasm32"))]`, and unlike `observation_text` this module has a wasm consumer (`probability_calibration` uses `message_bits_per_token` + `ProbabilityMetadata`). So the module can't be gated wholesale. The pure surface (`ProbabilityMetadata`, `message_bits_per_token`, `message_information_bits`, `sample_id`, `shard_of`, `shard_file_name`, `RecordPartition`, `PartitionCounts`, `ShardEntry`, `ObservationManifest` struct + `new`/`shard_count`) stays ungated. All I/O — the `SourceUnavailable`-referencing fns, the `ShardHandle`/`ObservationShardWriter`/`ObserveSummary` types, `PartitionCounts::add`, and the host-only imports (`std::fs`, `std::io`, `Path`/`PathBuf`, `compiler`, `TeacherOracle`, `Progress`) — is gated `#[cfg(not(target_arch="wasm32"))]`. wasm32 build is clean with no dead-code warnings.

## Ripple
`lib.rs` `observe()` and graph-cli `observe_command` map `SourceUnavailable` → `String` at their still-`String` boundaries.

## Verification
audit-limits graph-compiler **23 → 4** (observation.rs 0 sites). Only `lib.rs` (the 4 CLI-entry sites) remains. Full workspace build + wasm32 graph-compiler + clippy + fmt clean; 86 lib + 9 observe integration tests, BDD 100/100, proof-model, audit-deferral clean.

Part of the #510 bottom-up sweep — the final graph-compiler cluster (host-I/O).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

