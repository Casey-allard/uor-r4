# #1595 — Add opt-in device=cpu|metal to the stack train and dialogue-train commands

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T03:12:24Z
- **Updated:** 2026-10-09T13:17:14Z
- **Closed:** 2026-10-02T13:59:58Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1595
- **Merged:** 2026-10-02T13:59:58Z
- **Merge commit:** 46fd32b16fd726f4fdba13afecaa2bef667c24f1
- **Base / Head:** main / UOR-Foundation:codex/device-select-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What and why

The geometric-stack trainer hard-codes `Device::Cpu` at two production sites — `train` (formerly `examples/geometric-stack.rs:1616`) and `dialogue_train` (`:3578`) — so no run could use the GPU. This wires the repository's existing strict parser, `baseline_protocol::device`, in as an **opt-in `device=cpu|metal` argument** on both commands.

Purpose: establish whether the GPU can train this stack at our sizes (owner direction, 2 October). This PR is the *enabler* for that measurement, not the measurement.

**Default is `cpu`.** Every existing run is behaviourally unchanged; `device_arg` returns `Device::Cpu` when `device=` is absent, and `device` is absent from both commands' required-argument sets.

## Exact-head evidence

**Reviewed head:** `245ae80c96b63fdee574619980b1acf6bfca25eb` (one commit on `origin/main` `35482908`; rebased, conflicts resolved by keeping both this change and main's `protocol` field).

**Compile** (executed, not a queue acknowledgement):
```
cargo check -j 6 --features cpu-accelerate -p uor-r4-training --example geometric-stack   → Finished (32.78s)
cargo build -j 6 --release --features cpu-accelerate -p uor-r4-training --example geometric-stack → Finished (2m37s)
```

**Actual behaviour of the new argument**, four cases executed against the release binary:

| Case | Result |
|---|---|
| `device=bogus` | `Error: Invalid("device must be cpu or metal; no implicit fallback")` |
| `device=metal`, feature off | `Error: Invalid("Metal requested without compiled Metal feature")` |
| `device=cpu` | accepted; proceeds to the next stage |
| no `device=` | **identical output to `device=cpu`** — the default is bit-for-bit the old path |

The last row is the compatibility claim, measured rather than asserted: with the argument absent, the run reaches exactly the same stage and error as an explicit `device=cpu`.

## Scope, and what this does NOT establish

This adds the **selector only**. It does **not** report a Metal result, and it makes no speed or capability claim.

Independently measured at an earlier head and reported on #1512: `metal_stack_ops_parity` passes 14/14 with the device verifiably engaged, and the op-level benchmark reports SwiGLU 16.63×, RMSNorm 18.67×, CrossEntropy 14.58× over CPU — **but no model in this project can train on Metal**, because `RecurrenceCore` and `FusedRead` have Metal forward kernels and **no Metal backward kernel**, so the first backward pass cannot execute. That is why a companion commit on my working branch adds a fail-fast `refuse_metal_training` guard; I kept it out of this PR to keep the diff to the single named change, and it will follow separately.

Whole-model contextual classification, trunk, NoRead/age/softmax, values/output remain floating point. No general language, reasoning, energy or complete integer-serving claim.

## Checks run
- `cargo fmt` applied; `cargo check` and `cargo build` as above.
- Four executed behaviour cases for the new argument.
- No existing test is modified by this change (it adds a field, a parser helper and two call sites; the `Settings`/`DialogueSettings` constructions and both `allowed` key lists are extended).

References #1512, #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

