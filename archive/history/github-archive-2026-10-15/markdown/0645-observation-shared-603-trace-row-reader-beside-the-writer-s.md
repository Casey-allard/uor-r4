# #645 — observation: shared #603 trace-row reader beside the writer's lane layout

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-12T18:38:12Z
- **Updated:** 2026-08-13T05:37:32Z
- **Closed:** 2026-08-13T05:37:32Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/645
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Context

#603 gave TeacherTrace profiles a documented row/lane layout written by `observe_sharded_traced` (shards + `.prob` + `.trace` sidecars); `merge_trace_rows` concatenates rows but no production-side *reader* decodes a row back into (layer, head, support entries, marker-aware absence) — #605's fitting harness had to implement its consumer-side decoder in `crates/uor-r4-graph-compiler/src/route_fit.rs` against the documented layout.

## Problem

Two independent decoders of the same lane layout (the #605 one plus any future consumer) is drift surface: a writer-side layout change would be caught only by whichever consumer's tests notice.

## Goal

One shared row reader in `crates/uor-r4-graph-compiler/src/observation.rs` (beside the writer, same file discipline as the SUPPORT_ABSENT_MARKER convention), consumed by route_fit and any future reader; round-trip tested against the writer on the registered profiles; #605's local decoder replaced by it.

## Non-goals

No wire-format change; no new profile; no serving-path involvement.

## Verification

Round-trip write→read property tests per profile; route_fit tests keep passing unchanged (byte-identical manifests); workspace gates.

Discovered during #605 (PR #644).

## Comments (1)

### comment 5275513031 — 2026-08-13T03:15:20Z — Casey-allard

PR #658 (branch `i645-trace-reader`) adds one shared reader beside the writer.

`TraceRowLayout` + `TraceRow`/`read_row` land in `observation.rs` next to the `TraceCapture` assembler (pure function of TraceProfile + capture geometry; marker-aware, length-checked). The three drift-prone copies are consolidated onto it: `TraceCapture::new` derives row_bytes from it, and `route_fit::load_route_trace_corpus` replaces its inline width math + manual offset decode with `read_row` (dead `read_f32`/marker import/`support` local removed).

Round-trip test covers front-loaded support + trailing absence markers + GQA kv width + wrong-length rejection; the existing observe/route_fit end-to-end tests are the behaviour-parity gate (95 lib + 49 integration, 0 failed). Workspace builds, graph-compiler clippy clean, register gates R1/R2/R3/R4/R6 pass. Additive public API, byte format unchanged.


---
_Comment/review ids in this file: 5275513031_

