# #597 — model-source: bind all open-weight model semantics in a versioned source snapshot manifest

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-10T21:57:01Z
- **Updated:** 2026-08-12T03:53:46Z
- **Closed:** 2026-08-12T03:53:46Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/597
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

The repository can compile a pinned Hugging Face directory, but its durable provenance is split between model descriptors, the downloader, and compiled manifests. Open-weight recompilation needs one immutable identity for the complete source snapshot so observations, artifacts, and certificates cannot silently mix weights with different config, tokenizer, chat-template, or licensing semantics.

# Current evidence

- `models/smollm2-135m-instruct.json` records `source_kappa_scope = "model.safetensors"`, not a snapshot digest, and the 360M descriptor has no source κ.
- `src/model.rs:474-559` (`SourceDownload`/`download_source`) downloads selected `*.safetensors`, JSON, tokenizer/model files, license, and README files at a pinned revision but does not emit a canonical manifest for every downloaded file.
- `crates/uor-r4-model-source/src/lib.rs:1090-1158` (`HuggingFaceLlamaOracle::load_inner`) hashes only `model.safetensors`; `src/model.rs:47-72` (`ModelManifest`) describes compiled objects, not source semantics.
- Merged PR #322 recorded a pinned source κ for the 135M rehearsal, while #450/#452 made teacher and corpus κs visible; neither commits the complete source snapshot.

# Problem

There is no versioned, content-addressed source snapshot manifest binding every downloaded file and the adapter/compiler semantics that interpret it.

# Goal

Produce a canonical source manifest and root CID/κ that uniquely identifies the exact open-weight model snapshot used for source execution and compilation.

# Scope

- Define a versioned manifest for repository, immutable revision, license, adapter/compiler version, source-execution mode, and every file path, byte length, and digest.
- Include `config.json`, tokenizer files, special-token/chat-template data, tensor index/shards, README/license material, and any other file admitted by the downloader.
- Make the source manifest available to compile orchestration, observation manifests, artifact provenance, and reports.
- Add deterministic canonical serialization and root-address tests.

# Non-goals

- Do not add sharded tensor loading or new source dtypes; those belong to the dependent ingestion issue.
- Do not change deployed runtime arithmetic or artifact wire semantics.
- Do not re-pin existing fixtures automatically.

# Dependencies and blockers

- Coordinates with #515 for claim/ledger wording and preserves #531 as the owner of corpus-scale saturation.
- Cross-repository publication through #589 remains a future integration and is not part of this issue.

# Proposed implementation shape

Extend the existing model-source/download and compile provenance surfaces rather than introducing a second model descriptor. Keep the manifest host-side and versioned; include its CID in downstream provenance without making the runtime depend on source files.

# Acceptance criteria

- [ ] A schema-versioned source manifest is emitted for a pinned local snapshot.
- [ ] The manifest binds every admitted file, including all tokenizer/config/index/shard/license files, by canonical path, length, and digest.
- [ ] The root CID/κ changes when any bound file, revision, adapter version, or source mode changes.
- [ ] A compile report and observation manifest expose the source-manifest CID.
- [ ] Existing legacy artifact readers and fixtures remain readable and unchanged.

# Verification

- Unit tests for canonical ordering, digest coverage, tamper detection, and schema rejection.
- A deterministic double-build test asserting identical manifest bytes and CID.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`, `cargo test --workspace --offline`, and `python3 scripts/check_claim_wording.py`.

# Compatibility and migration

This adds provenance fields and a new manifest schema. Existing model descriptors and artifacts remain readable; migration must document which old κs were weight-only and must not silently relabel them as full-snapshot identities.

# Claim status

Definition, implementation, and empirical determinism criterion. It does not establish model-quality parity or exact transformer equivalence.


## Comments (3)

### comment 5260314367 — 2026-08-12T00:05:17Z — Casey-allard

Ecosystem-rebase input (#589/#620): adoption decision recorded on #620 — manifest root via `uor_addr::json::address_blake3` (canonical JSON, already in the dep graph, κ label format byte-compatible with our existing `blake3:<hex>` strings); per-file leaves stay raw blake3 digests. Upstream safetensors structural addressing requested at https://github.com/UOR-Foundation/uor-addr/issues/14 for the longer term.

### comment 5261845002 — 2026-08-12T03:17:05Z — Casey-allard

## Delivery record (PR #633 in queue)

Implemented per the acceptance criteria: schema-versioned manifest (`uor-r4-source-manifest/1`) binding repository/revision/license/compiler-version/source-mode + every admitted file (path, bytes, blake3), root κ via `uor_addr::json::address_blake3` (the #620 decision), emitted by `download_source` and threaded end-to-end (uor-r4-api CompileRequest → compiler/cover flags → CoverReport + ObservationManifest, server auto-binds when the snapshot carries a manifest). Both report fields are `serde(default)` optionals — legacy artifacts/fixtures byte-identical and readable. License populated from the pinned `models/*.json` descriptor. Tests: canonical ordering, digest coverage, tamper, schema rejection, deterministic double-build, every plumbing seam. Migration note in docs/MODEL_LIFECYCLE.md: pre-#597 weight-only κs (`source_kappa_scope = \"model.safetensors\"`) are NOT relabeled.

**Honest bound:** the bare CLI (`r4 graph-compile`/`graph-observe`) forwards user flags verbatim by design — a CLI user passes `--source-manifest-kappa` explicitly; the server workflow and typed API populate automatically. All four gates + full suite (125 blocks) green. Closes when #633 merges; #598 (shard ingestion, which consumes this manifest) is next in the program.

### comment 5262074867 — 2026-08-12T03:53:45Z — Casey-allard

PR #633 merged — delivered per the record above; closing. #598 (shard ingestion consuming this manifest) begins now.


---
_Comment/review ids in this file: 5260314367, 5261845002, 5262074867_

