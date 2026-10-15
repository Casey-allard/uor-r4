# #556 — feat(#510): R5 tranche — graph-compiler lib.rs CLI entry to total; graph-compiler at 0 (4→0)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T21:15:03Z
- **Updated:** 2026-10-09T14:17:26Z
- **Closed:** 2026-08-09T21:33:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/556
- **Merged:** 2026-08-09T21:33:23Z
- **Merge commit:** b9ea9adf73fefdee11b995245119e80656aa1952
- **Base / Head:** main / UOR-Foundation:r5/gc-libentry
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

R5 rearchitecture (#510): the final graph-compiler cluster. Convert the four CLI-entry fns in `lib.rs` off `Result<_, String>`. **graph-compiler is now fully sanctioned — 0 audit-limits sites.**

## Surfaces
- `parse_options -> Option<GraphCompileOptions>` and `parse_observe_options -> Option<ObserveOptions>` — pure argument parsing. A flag with no value, an unparseable or zero numeric value, an unknown flag, or (observe) neither `--source` nor `--checkpoint` means the requested options are not a product of those arguments → `None`. Uses `args.get(i+1)?` and `value.parse().ok()?`.
- `compile` / `observe -> Result<(), SourceUnavailable>` — host-facing CLI entries. Their failures are all "a declared input or host resource could not be ingested": host fs reads (via `SourceUnavailable`'s `From<io::Error>`, or `map_err` for path context), invalid artifact container / incomplete corpus / non-UTF-8 paths (`ok_or_else` + `SourceUnavailable::new`), the `emit_r4g1` `ObservedBound` (mapped into `SourceUnavailable::new`), and the now-total `observe_sharded` / HF-oracle load (propagate via `?` as `SourceUnavailable` identity). serde_json via `From<serde_json::Error>`. A `parse_*` rejection becomes a `SourceUnavailable` at the compile/observe boundary.

## WASM
`compile`/`observe`/`parse_options` were already `#[cfg(not(target_arch="wasm32"))]`; the new `SourceUnavailable` and the now-gated-only `compiler` import are gated to match, so the wasm build stays clean (`parse_observe_options` is total `Option` and needs no gating).

## Ripple
The `r4` binary (`src/main.rs`) and `uor-r4-api` (`compile.rs`) map `SourceUnavailable` → their `String` surfaces at the call boundary.

## Verification
audit-limits graph-compiler **4 → 0**. Full workspace build + wasm32 graph-compiler + clippy + fmt clean; 86 lib + 9 observe integration + 22 graph-cli tests, BDD 100/100, proof-model, audit-deferral clean.

**Milestone: graph-compiler (the leaf crate) is fully R5-sanctioned.** Next crate up the stack: graph-certify.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

