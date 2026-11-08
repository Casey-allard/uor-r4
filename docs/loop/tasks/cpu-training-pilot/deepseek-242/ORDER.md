# ORDER: cpu-training-pilot, deepseek-242 (first-reply Hamming value trace)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `bfa5de6ce`.

## Goal

Measure which earlier token positions each served Hamming read and head keeps
when the model predicts the first reply token, on the sealed Claude-322 integer
artifacts. For every `m2-fresh256` row, identify the asked and paired-other
value token spans in the rendered prompt and report whether every read/head
keeps neither, the asked span only, the other span only, or both.

This is an independent CPU diagnostic supporting Claude-326 and Claude-328. It
does not train, update an artifact, touch Claude's open branch, or claim that a
query-conditioned read works.

## Cancel probe

- Claude-327 reports the first-position Hamming trace as still open because it
  required new code.
- Claude-328's open branch modifies the float trainer and its recall probe; it
  does not modify `crates/uor-r4-cpu-train`.
- The sealed Claude-322 integer artifacts are available locally with hashes
  `86e51d5e...` and `8838cd10...`, and the integer session exposes
  `set_hamming_trace`, `take_hamming_trace`, `mark_reply_start` and
  `step_native`.
- No committed report traces asked-value and other-value prompt spans through
  each served Hamming head at the first reply decision.

## Design

Add one opt-in CPU binary, `cpu-first-reply-hamming-trace`, without changing
the serving or training paths.

For each panel row and both registered renderings:

1. Render the exact protocol-v2 prompt used by `served-reply-panel`.
2. Encode the shortest accepted answer and the paired other answer, excluding
   terminal EOS, and locate every full token span in the prompt. Refuse a row
   whose value span is absent; retain repeated spans and count a kept position
   as value membership when it lies in any matching span.
3. Prime the integer session over every prompt token before the final prompt
   token with tracing enabled, discard those priming traces, mark the reply
   start, step the final prompt token, and retain only the reads produced by
   that first-reply decision.
4. Record every read depth, head, kept position and value-span membership.
5. Report per artifact, rendering, stratum and head:
   - rows keeping neither, asked only, other only, and both value spans;
   - kept positions for the asked and other first tokens separately;
   - the model's chosen first token and whether it equals either reference's
     first token;
   - exact input, artifact and executable hashes.

## Files in scope

- `crates/uor-r4-cpu-train/Cargo.toml`
- `crates/uor-r4-cpu-train/src/bin/cpu-first-reply-hamming-trace.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-242/**`
- `docs/labs/cpu-training-pilot-first-reply-hamming-trace-2026-11-08/resource-ledger.json`
- `docs/research/notebook/cpu-training.md`

## Done when

1. The binary builds offline and locked. Tests cover exact renderer parity,
   value-span extraction and ambiguity refusal, first-step trace isolation,
   read/head tallying, EOS exclusion and fail-closed metadata hashes.
2. Both sealed artifacts run on all 256 panel rows under both registered
   renderings without training.
3. The REPORT records per-head asked/other/neither/both counts, chosen-token
   agreement and a resource receipt.
4. REPORT, notebook row, report check, local policy, full gates and a fresh
   exact-head review pass.

## Stop and ask if

This is attribution evidence, not a capability result. Do not alter a forced
prefix, train a model, modify Claude-owned files, claim that keeping a value
position proves selection, or substitute this trace for Claude-328's trained
query-recall probe.
