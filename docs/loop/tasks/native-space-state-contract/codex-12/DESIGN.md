# DESIGN: the next one-space token-objective configuration

Status: decision record for codex-12. No implementation in this unit.

## Evidence

Two configurations of the first token objective are measured negatives (host, synthetic trunk
seed `20_261_115`, vocabulary 256, width 32, two recurrence layers, 8 lanes each, sequence
`[3, 5, 8, 13, 21, 34, 55, 89]`, 112 records = 7 transitions x 2 layers x 8 lanes, 64 training
steps):

| Configuration | Target | Dot before | Dot after | Served tokens |
|---|---|---|---|---|
| codex-6 | writer block, `q12 << 16` (writer's 1/8 range) | 2,476,676,547,100,606,464 | 2,142,903,011,938,598,912 | unchanged |
| codex-7 | writer block, normalized to unit Q31 scale | 18,514,432,582,351,337,035 | 17,116,574,668,483,326,999 | unchanged |

Both moved the gate and both lowered the independent dot. The state is **one** 4-word Hamilton
gate (`Q31HamiltonModel`, 48 bytes with its cursor), applied to every lane of every layer. The
records carry 112 different target directions. A single rotation cannot align them; the update
settles on a compromise. Changing the target scale did not help, so the scale is not the suspect;
the state's shape is.

## Options

- **A. Per-lane gates, shared across layers.** One 4-word gate per lane (`K = width / 4`), used by
  both recurrence layers. Synthetic `K = 8`; a real width-256 stack `K = 64`. Removes the
  cross-lane conflict with the smallest state growth.
- **B. Per-layer-lane gates.** One gate per `(layer, lane)`, synthetic `K = 16`, a 6-layer
  width-256 stack `K = 384`. Removes the cross-layer conflict too, at much larger state.
- **C. A different objective or update.** Keep one gate and change what is trained, e.g. a served
  token objective, or a loss whose sign is checked against the independent dot. No state growth,
  but the gate's effect on the readout is indirect and a new objective must itself be justified.
- **D. Stop the line.** Rejected: two negative configurations close configurations, never the
  mechanism (CHARTER §4.1), and the owner's direction (D86) keeps one-space training as the goal.

## Decision

**A first.** It is the smallest state change that removes the measured conflict, it keeps the
existing typed operator, and it gives a clean per-lane measurement. If A is negative on every
lane, B; if B is negative, C. The decision is reversible and is logged for Casey.

## State and schema for A

- The state becomes `K` gate quaternions (four little-endian Q31 words each) plus the same 16-byte
  cursor (epoch `u64`, step shift `u32`, version `u8`, 3 zero bytes): `32 K + 16` bytes.
  Synthetic `K = 8`: 272 bytes; width-256 `K = 64`: 2,064 bytes.
- A new schema `uor-r4.lut-stack/13`, not a reinterpretation of `/12`: an engine built before
  per-lane gates would serve one gate and silently drop the others, so it must refuse the
  artifact by schema.
- The table `q31.gates` holds the whole state as little-endian `u32` words (`8 K + 4` words), and
  the header record `q31_lanes = { state, sha256, lanes, epoch, step_shift, version }` binds the
  digest, `K` and the cursor. The digest is over the exact `32 K + 16` bytes.
- **Cursor rule (decision-review F1):** there is exactly one shared `(epoch, step_shift)` in the
  16-byte cursor. A training **round** applies one `step_reference` batch to every lane and then
  advances the shared epoch once; a lane does not own a cursor. The type is a new
  `Q31LaneGates { gates: Vec<Q31Quaternion>, step_shift: u32, epoch: u64 }`, not `K` separately
  serialized `Q31HamiltonModel`s, so the byte shape stays `32 K + 16`.
- The parser validates: `K = shape.width / 4`, the record/table names, the length, the digest,
  every gate in the closed Q31 domain through `Q31HamiltonModel::from_bytes`, and the cursor
  consistency. A `/13` header without the record, or a `/12` header with one, is refused.

## Operator and serving

- Training and serving keep the same `q31_hamilton_rotate`, called with the lane's gate. No new
  arithmetic definition.
- `Q31ServingSession` becomes a vector of gates (`K`) plus the cursor; `transition(lane, value)`
  selects `gates[lane]`; the trace exposes the lane as it does today. The composite session
  identity binds the artifact hash and the whole `K`-gate state.
- The AOT CUDA engine already accepts one gate per batch item (`hamilton_rotate` grids over the
  item count), so training `K` gates needs no new kernel: one grouped call with a gate per item,
  or `K` calls of the existing entry. A batched-kernel optimization is later, not required.
- A model without the section stays on the existing path; a `/13` artifact is refused by older
  engines and a `/12` artifact by the per-lane parser.

## Training and the independent measure

- Records are grouped by lane. Each gate is trained only on its own lane's carries and
  next-token blocks, with the existing `Q31HamiltonModel::step_reference` and a shared cursor.
- The measure is the same **in-sample independent** integer dot (a different loss than the
  training L1; it is computed on the same records, not held out), now **per lane**:
  `sum over the lane's records of rotate(gate_lane, before) · target`, before and after, plus the
  served token ids. A lane's sign is reported, not asserted; the word "independent" here means
  "not the training loss", not "held out".
- Bar for the configuration: the aggregate dot rises and at least one lane's dot rises while none
  falls, with the artifact carrying exactly the trained `K` gates and a fresh `uor-chat` process
  matching the in-process run. A cross-lane fall is a negative for A.
- **Token-level co-primary (decision-review F2):** a dot-only improvement is **not** token
  progress. The GPU qualification sheet must carry a token-level co-primary measure (served token
  ids and, where a panel exists, next-token accuracy) beside the per-lane dot, and a
  dot-positive/token-flat A is reported as proxy-only, not as a model or M4 result.

## Cost, risks and the next units

- Cost: one format/schema/parser/serving unit, one training/grouping unit, one GPU qualification
  unit. State growth is 5.7x for `K = 8` and 43x for `K = 64` over 48 bytes.
- Risks: a lane with few records overfits (the tiny fixture has 14 records per lane); the cursor
  must be shared consistently; the served readout may still not move even when the per-lane dot
  does, because the alignment is a proxy for the token objective, not the token objective itself.
  That last risk is measured and reported, not hidden.
- If A fails, B tests the cross-layer conflict; C tests whether the alignment objective is the
  wrong objective. The D86 donor data is the eventual real corpus; per-lane gates are a
  **prerequisite of the state shape**, not a sufficient condition for consuming that corpus. The
  corpus path also depends on the token objective and D86's hard-target framing (answer texts,
  no logits), not on gate arity.

## Implementation order (next units)

1. `/13` schema, parser, exporter and serving with `K` gates, with a bit-identical replay fixture
   and a planted fault (a per-lane gate changed).
2. The token-objective test rerun per lane on the synthetic trunk; the per-lane dots and served
   tokens recorded as one notebook row.
3. If A's per-lane dot is positive, the GPU qualification under a sheet and a non-author FIT,
   with a **token-level co-primary** measure; a dot-only positive is proxy-only and does not
   advance to a model claim. If the dot is negative, B.
