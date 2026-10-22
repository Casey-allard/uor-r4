---
id: m3-legacy-chatsession-persistence
title: Retire or complete legacy width-576 ChatSession persistence
milestone: M3
area: tooling
status: done
open_to: anyone
needs_gpu: false
depends_on: []
scope: [crates/uor-r4-integer/**, docs/labs/m3-legacy-chatsession-persistence/**]
---

## Why it matters

Session save and restore is part of M3 durable memory. Today the repository has two session paths: the legacy `ChatSession` (schema `uor-r4.integer-session/1`) and the `IntegerStackSession`, whose reload is exact. The legacy path silently drops state, and the capability API says so in its own text ("Durable-memory capability is not established"). This item clears the tracked defect so session persistence has one tested path. Background: [serving and export](../../model/serving-and-export.md).

## What is missing

The legacy `ChatSession` does not serialize its three width-576 value stores: `persistent_values_576`, `dialogue_values_576` and `l2_pages_576` (fields in `crates/uor-r4-integer/src/model.rs`). A restored width-576 session reads empty values and diverges at its first step. The test `restored_dialogue576_session_continues_identically` in `crates/uor-r4-integer/tests/capability_api_tests.rs` is ignored for that reason, and it also needs the local `dialogue-child-bundle-1` fixture. If the path is reactivated, the three stores must serialize. Otherwise the ignored test is retired in favour of the `IntegerStackSession` reload gate (`#1476`).

## Done when

Either the three value stores serialize and the test passes, or the ignored test is retired in favour of the `IntegerStackSession` reload gate. In both cases the "omits the width-576 value stores" wording is removed or updated where it appears (the test file header, `capability_api.rs`, the test's ignore message).

## How to start

1. Decide the direction first, on `#2031`, in a short design note: reactivate or retire. Read `crates/uor-r4-integer/src/session.rs` (`SerializedChatSession`, `SerializedSessionState`, schema constant `SESSION_SCHEMA_V1` near line 475) and `crates/uor-r4-integer/src/capability_api.rs` (the durable-memory text near line 55 and `restore_session` near line 328). Check whether any shipped path still builds a width-576 `ChatSession`. If none does, retiring is the smaller, safer choice.
2. Note that `crates/uor-r4-integer/src/session_codec.rs` is the codec for the `IntegerStackSession` (`SerializedStackSession`), not the legacy `ChatSession`. Use it as the model for how the stack path is exact: `session_save_and_restore_produces_bit_identical_logits` and `session_pause_save_fresh_restore_greedy_continuation` in `crates/uor-r4-integer/src/stack/tests.rs`. Confirm these are the reload gate that `#1476` means, and name them in the note.
3. If retiring: remove the ignored test, update the header in `crates/uor-r4-integer/tests/capability_api_tests.rs` and the durable-memory string in `capability_api.rs`, and keep `uor-r4.integer-session/1` tests that need no 576 fixture. Run `~/.cargo/bin/cargo test -p uor-r4-integer` and show that the default (non-ignored) suite still passes.
4. If completing: add the three stores to the serialized state with a schema bump, using integer-only fields, with a version and tamper check as in `crates/uor-r4-integer/tests/adversarial_m3_session_tamper_prng.rs`. First write a failing round-trip test on the synthetic byte-vocabulary bundle (no fixture needed). Then run the ignored test with `~/.cargo/bin/cargo test -p uor-r4-integer --test capability_api_tests -- --ignored` where the fixture exists.
5. Open a PR that says `Closes #<this issue>` and `References #2031` (the M3 milestone issue stays open; this defect also folds `#1476`). Record the result as one row in `docs/research/notebook/m3-durable-memory.md`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- issue#2031 (folds #1476)
- [`crates/uor-r4-integer/src/session.rs`](../../../crates/uor-r4-integer/src/session.rs)
- [`crates/uor-r4-integer/src/stack/tests.rs`](../../../crates/uor-r4-integer/src/stack/tests.rs)
