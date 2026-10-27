---
id: m3-qualify-memory
title: Qualify durable memory on the selected native model
milestone: M3
area: evaluation
status: ready
open_to: lab
needs_gpu: true
depends_on: [m1-close-saved-model, object-store-consumer-m3, m3-memory-turn-discriminator-gate, m3-policy-precedence-and-sieve-tests]
notebook: docs/research/notebook/m3-durable-memory.md
scope: [crates/uor-r4-integer/**, crates/uor-r4-object-store/**, docs/mechanisms/exact-memory.md, docs/mechanisms/object-store.md, docs/harness/prep/m3-qualify-memory*, docs/labs/m3-qualify-memory/**, docs/loop/tasks/m3-qualify-memory/**, docs/research/notebook/m3-durable-memory.md]
---

## Why it matters

M3 is the claim that a conversation can be stored and come back exactly, with answers staying grounded in what was said (issue #2031). The memory pieces exist but have only been tested on fixtures, so no result yet says that a trained model uses its memory. This item turns them into one measured verdict on the selected native model, and `m4-export-uorlut01-cli` depends on it. Background: [exact memory](../../mechanisms/exact-memory.md).

## What is missing

m3-memory is not qualified and its status is paused (`STATUS.md:18`: evaluator and session built, not qualified; waits on an M1 or M2 model). The evaluator and session exist (`stack_grounded_session`, the typed-script multi-turn evaluator, `IntegerStackSession`, and the `UR4SESS1` session file) but have never run on a trained M1 or M2 model. Note that `crates/uor-r4-training/src/stack_grounded_session.rs` describes itself (line 6) as "a float development path, not a D11 serving kernel": the integer-path verdict must come from `IntegerStackSession`, not from that module alone.

No such model is selected yet. M1 has no passing result, and M2 is held with 437/512 saved development replies, 0/128 on a transfer check, and a fresh test that is `NOT_RUN` (`STATUS.md:17`). `ROADMAP.md:16` states the M3 bar only as "Memory persists across sessions and answers stay grounded". No per-check count or threshold is registered; registering them is step 3 below.

## Done when

On the selected artifact:

- Provenance, contradiction and correction, abstention on unstated or retracted facts, explicit reset and process restart all pass.
- Saved and restored sessions continue identically token for token (`IntegerStackSession` plus `UR4SESS1`).

The pass threshold for each check is not yet registered.

## How to start

1. **Read the code and run the host tests.** Open `crates/uor-r4-integer/src/session_codec.rs` (`UR4SESS1` `encode` and `decode`), `crates/uor-r4-integer/src/stack/session.rs` (`IntegerStackSession`), `crates/uor-r4-training/src/stack_grounded_session.rs` and `crates/uor-r4-training/src/grounded_session_eval.rs`. Run `~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm`; without that feature the serving modules (`session_codec`, `stack`) are not compiled, and the last recorded run was 165 passed (`docs/labs/wasm-serving-runtime-2026-10-18/README.md:30`). Confirm which file holds the typed-script multi-turn evaluator; `grounded_session_eval.rs` (frozen-script diagnostics, schema `uor-r4.grounded-script/1`) is the likely one.
2. **First deliverable: a coverage note on the issue.** List each of the six behaviours (provenance, contradiction and correction, abstention, reset, process restart, token-for-token restore) against the evaluator case that tests it, or mark it missing. State whether `m3-memory-turn-discriminator-gate` and `m3-policy-precedence-and-sieve-tests` change the evaluator. Confirm the evaluator scores with exact typed-script graders and no softmax, squashing or matrix multiply.
3. **Draft the pre-registration sheet.** Name the selected artifact by hash (blocked until M1 or M2 yields one). Freeze the typed-script panel as files, list each with its SHA-256, and keep it separate from any development scripts. Register the per-check thresholds, seeds, GPU, stop rule and cost. Ask a non-author for a TEST FITNESS: FIT review.
4. **Write the finite restore control first, as a failing test.** In `crates/uor-r4-integer/tests/`, build a synthetic stack (see `crates/uor-r4-integer/src/bin/mk-synthetic-stack.rs`, a binary that needs `--features serving-wasm`; run the new test with the same feature), run a few turns, save with `UR4SESS1`, restore in a fresh process, and require the continued token stream to equal an uninterrupted run.
5. **Wait for the owner.** M2 and training are held, and one typed owner release covers one pre-registered sheet; Casey types it. After release, run the sheet on a GPU (outside contributors attach hardware specs to the PR).
6. **Open a PR** that says `Closes #<this issue>` (or `References` if partial), and record the verdict as one row in `docs/research/notebook/m3-durable-memory.md`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`ROADMAP.md:16`](../../../ROADMAP.md)
- [`STATUS.md:17-18`](../../../STATUS.md) (M2 and M3 rows)
- `docs/labs/wasm-serving-runtime-2026-10-18/README.md:30` (the host test command and count)
- issue#2031

