# ORDER: one-model-end-to-end, claude-269 (mixed copy keys in cpu-train and fbs-toks)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `3641753d922e06fa0835e61b8393e07f42be2e1f` (main with claude-268 merged). Draft PR #3627.

## Goal

claude-268 gave each copy head its own key in the float read, the export and the integer session. Its two other consumers still read one key for every head, so for now the integer crate's public readers refuse a mixed record (`one_copy_key`). This unit gives both consumers a key per head and lifts that refusal.

**Plan step (the claim check):** order step 4, one model trained as served end to end. **What it changes in the model:** nothing in training. cpu-train's served forward, the trainer's audit of the integer serve, reads each copy head by its own key, as the session does. The mixed pair's GPU run needs that audit.

## Design

- **cpu-train `hamming.rs`:** `CopyHeads.pair` becomes one optional mask per copy head, from `record.head_copy_key(h)`, as the integer `CopyHeads` does.
- **cpu-train `served_forward.rs`:** the manifest's Hamming entry gains `copy_keys`, left out when none.
- **`fbs-toks`:** `require_one_token_copy_codes` also refuses a header with `copy_keys`, since `--copy-codes` audits the one-token key only. Its other modes drive the session, which serves mixed keys.
- **integer `stack/hamming.rs`:** drop `one_copy_key` from `read_hamming_uorlut01` and `served_copy_codes_uorlut01`.
- DeepSeek gets a `[claude]` note on the bench naming the cpu-train change (Casey's direction of 3 November).

## Cancel probe (run first)

1. Is the refusal still on main? `one_copy_key` is called in both readers at the base: yes.
2. Does another open PR touch these files? On 3 November, after the claim, no open PR other than #3627 changes cpu-train's `hamming.rs`, `served_forward.rs` or `fbs.rs`, `bin/fbs-toks.rs`, or `stack/hamming.rs`/`tests.rs`. DeepSeek's last cpu-train changes (#3590, #3611, #3612) are merged.

Result: go. Classification: instrument (the unit's own preconditions).

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-269/**`, `docs/research/notebook/float-training.md`, `docs/roadmap/items/one-model-end-to-end.md`
- `crates/uor-r4-cpu-train/src/{hamming.rs,served_forward.rs,fbs.rs}`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `crates/uor-r4-integer/src/stack/{hamming.rs,tests.rs}`

## Done when

1. (a) cpu-train's served forward matches the integer session on a mixed artifact, at `[2,1]` and `[1,2]`, as its key-2 parity tests do. (b) cpu-train's `copy_key_two_tells_two_facts_after_the_same_token_apart` gains a mixed case. (c) `fbs-toks --copy-codes` refuses a mixed header. (d) The public readers return a mixed record's `copy_keys`.
2. A negative gate for (a): with cpu-train's per-head key set aside, (a) fails.
3. `cargo test -p uor-r4-cpu-train` and `cargo test -p uor-r4-integer --features serving-wasm` pass with `--no-fail-fast`, apart from the seven fixture-bundle targets that fail at any head (claude-268).
4. `loop gates --full`, `loop report --check` and `loop pr-check --local` pass, and a fresh-context review approves at the exact head.

## Stop and decide (never stop and wait)

- If cpu-train's served forward cannot match the session on a mixed artifact, stop the build and record why. Do not ship a key the audit reads differently.
- If a struct literal outside scope breaks, record it and decide.
