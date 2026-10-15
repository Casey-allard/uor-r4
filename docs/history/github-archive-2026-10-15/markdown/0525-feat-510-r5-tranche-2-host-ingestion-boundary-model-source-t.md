# #525 — feat(#510): R5 tranche 2 — host-ingestion boundary; model-source to sanctioned surface

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T02:58:45Z
- **Updated:** 2026-10-09T13:08:53Z
- **Closed:** 2026-08-09T03:17:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/525
- **Merged:** 2026-08-09T03:17:24Z
- **Merge commit:** 25f235f87fa8cef781fb67ab8df84942caf76a81
- **Base / Head:** main / UOR-Foundation:feat/510-r5-t2-model-source
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Fourth R5 step (issue #510): opens the **host-ingestion boundary** and converts the teacher loader (`uor-r4-model-source`) off ad-hoc `Box<dyn Error>` / `String` results. The graph substrate (graph-format + graph-runtime) is now fully on the sanctioned surface via #518–#524; this begins the host-tooling layer.

## Decision: a host-boundary sanctioned type
`model-source` is a std, float-heavy teacher (a run.c forward-pass port) that ingests external HuggingFace safetensors. Its errors — missing model dir, safetensors dtype mismatch, corrupt `config.json` — have no honest representation in the graph substrate's `NotAProduct` (which wraps a graph-*wire* `FormatError` like `BadMagic`); `model-source` doesn't even depend on graph-format. So rather than pollute the wire contract, this introduces one host-side sanctioned error.

`SourceUnavailable { reason }` — the host-side analogue of `NotAProduct`. It reports exactly one condition: *a declared external source artifact could not be ingested into a valid teacher at construction*. `From<io::Error / serde_json::Error / safetensors::SafeTensorError>` keeps `?` ergonomic and **preserves the operator diagnostic** (which file, which tensor, what dtype). It is registered in the `xtask audit-limits` allowlist alongside the substrate's three.

## Conversions
- `HuggingFaceLlamaOracle::{load, load_with_sequence_length, load_inner}`, `append_tensor`, `append_layers`, `progress::read_file` → `Result<_, SourceUnavailable>`.
- `RepresentationSource::read_embedding_rows` → `Option<()>` (None = the caller's range/buffer cannot be served — a property of the caller's chosen instantiation). All implementors updated (LlamaOracle, HuggingFaceLlamaOracle, core's recorded representation, compiler test/observation fakes); callers use `.unwrap()`/`.expect()`, unchanged.

## Verification
- `audit-limits`: model-source now reports **zero** unsanctioned errors.
- Build / clippy (`-D warnings`) / fmt clean; all workspace test targets compile; `recorded_representation` and `observe` integration tests green.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

