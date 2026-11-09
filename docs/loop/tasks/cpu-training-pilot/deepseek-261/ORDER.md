# ORDER: cpu-training-pilot, deepseek-261 (CPU Hamming ask and carry)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `5643a45c5`.

## Goal

Make the independent CPU Hamming port execute the exported ask heads and
readout question carry exactly as `IntegerStackSession` does, so CPU training,
evaluation and audit paths no longer silently omit the asking model's query.

## Cancel probe

- The integer artifact reader exposes `AskImage` on every Hamming read.
- The integer `TypeHeads` asks with `table + ask`, through the key rotation,
  and the readout adds the last `h` rows' Q16-gained asked points at Q28.
- `crates/uor-r4-cpu-train/src/hamming.rs` never reads `read.ask`; its type
  heads use the hook point only.
- No open DeepSeek PR or claim touches this unit.

**Result (instrument, at claim):** pass. This is a real served-forward parity
gap, not a model result.

## Design

- Retain the existing non-ask query rotation.
- Add the ask table, key rotation and optional carry to the CPU type heads.
- Compute the asked point at Q28 exactly as the integer session does:
  `(table + ask)[x_t] << 16` plus the last `h` window rows' Q16 gains.
- Apply the carry only to the readout read; stream reads with asks use the
  same ask table without a carry.
- Use ask queries in the stream, readout and trace paths.
- Add CPU/integer parity tests for:
  - ask heads without carry;
  - ask heads with readout carry;
  - a mutated carry control that changes the CPU result;
  - non-ask artifacts remaining unchanged.

## Files in scope

- `crates/uor-r4-cpu-train/src/hamming.rs`
- `crates/uor-r4-cpu-train/src/fbs.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-261/**`
- `docs/research/notebook/cpu-training.md`

## Done when

1. CPU and integer steps match on ask artifacts with and without carry over a
   multi-row window.
2. Removing the ask or carry changes the same CPU result as it changes the
   integer result.
3. Existing no-ask Hamming and copy-key parity remains green.
4. Focused tests, formatting, report/policy checks and full gates pass.
5. REPORT and notebook row are committed; a fresh exact-head review approves;
   the PR merges and is verified on `main`.

## Stop and ask

Do not change the artifact format, trainer objective, model weights, a bar or
scope. Do not touch Claude's language-carry branch, pods or artifacts.
