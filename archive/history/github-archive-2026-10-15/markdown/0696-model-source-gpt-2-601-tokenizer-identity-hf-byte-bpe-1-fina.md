# #696 — model-source: GPT-2 #601 tokenizer identity (hf-byte-bpe/1) — final open item of #657

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-14T04:23:30Z
- **Updated:** 2026-08-14T05:26:13Z
- **Closed:** 2026-08-14T05:26:13Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/696
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Item 4 (and the last remaining open item) of the #657 GPT-2 enablement tracker. Items 1 (compile-path dispatch, #671), 2 (#603 trace-capture, now implemented — `forward_capturing_trace` + real `trace_capture_geometry`/`step_with_trace_capture`), 3 (#602 learned-absolute operator, #672), and 5 (compile→certify canary, #670/#693) have all landed. GPT-2 now compiles end-to-end to an R4G1 artifact with source-parity rows. The one surface still deliberately absent is the tokenizer identity.

## Problem
`HuggingFaceGpt2Oracle::tokenizer_address` (crates/uor-r4-model-source/src/gpt2.rs:1174) returns the generic `"huggingface-tokenizer"` — byte-for-byte the same string the Llama oracle reports. So a compiled GPT-2 bundle's #601 record does not distinguish the GPT-2 byte-level BPE tokenizer (vocab.json + merges.txt) from any other HF tokenizer. GPT-2 uses exactly the `hf-byte-bpe/1` family already registered in `uor_r4_core::transformerless::hf_bpe` (and named as the sole known family in `SourceIngestKind::UnknownTokenizerAdapter`'s message), so the identity exists in the registry but the oracle does not declare it.

The struct doc (gpt2.rs:1115–1122) still lists this — alongside the #602/#603/dispatch surfaces that have since been implemented — as deliberately absent; that note is now stale and should be trimmed to the tokenizer item (then removed when this lands).

## Recommendation
Wire `tokenizer_address` (and/or the richer #601 surface the certificate consumes) to record the GPT-2 tokenizer distinctly as `hf-byte-bpe/1`, bound to the pinned `vocab.json` + `merges.txt` CID from the #669 descriptor (models/gpt2-124m.json), so a compiled bundle binds the exact tokenizer rather than the generic id. Refuse-by-name discipline unchanged: unknown `(family, version)` stays rejected, never approximated.

## Regression gates
- SmolLM2 / Llama `tokenizer_address` and compiled bytes unchanged (the Llama path must not shift).
- The #670 presence-gated canary continues to pass; extend it to assert the report/certificate carries the GPT-2 tokenizer identity row if the surface reaches the cover report.
- Full gate suite (fmt, clippy --workspace --all-targets, workspace tests, register conformance).

## Acceptance
A compiled GPT-2 bundle's #601 tokenizer record names `hf-byte-bpe/1` with the pinned vocab/merges CID, distinct from the Llama tokenizer; SmolLM2 byte-unchanged. Closes the last open item of #657.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

