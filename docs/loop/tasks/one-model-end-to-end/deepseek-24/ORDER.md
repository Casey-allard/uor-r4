# ORDER: one-model-end-to-end, deepseek-24 (packet float/integer parity)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `d49994fdc`.

## Goal

Close the `NOT_RUN` same-scorer parity gap in deepseek-21. One GPU instrument
must render each `m2-fresh256` packet once, teacher-force the same shortest
accepted reference answer plus EOS through the float checkpoint and its
exported integer artifact, and report exact chosen-token, target-rank and
EOS-rank differences.

## Cancel probe

- A public float packet reader may bind only the current no-turn-memory,
  no-reply-cut `(b64L)` shape and must refuse any other checkpoint before model
  work.
- The float path must use the same protocol-v2 `Assistant:` prompt, Q12 served
  query, shared table and lower-token tie break as the integer reader.
- Integer ranks must use the actual served order, including a point-key
  override when present; the current target has no memory.

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/Cargo.toml`
- `crates/uor-r4-float-train/src/bin/packet-parity.rs`
- `crates/uor-r4-cpu-train/src/fbs.rs`
- `crates/uor-r4-cpu-train/src/lib.rs`
- `docs/labs/one-model-end-to-end/instrument/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-24/**`
- `docs/research/notebook/float-training.md`

## Done when

1. `packet-parity` builds with a stub-`nvcc` type check and accepts one float
   checkpoint plus its `UORLUT01` artifact.
2. It verifies the panel manifest, tokenizer, target order and one EOS per row.
3. It writes a new report with per-row and aggregate rank disagreement counts.
4. Pure comparison and target-misalignment tests pass.
5. The exact instrument runs on the first trained checkpoint (or the base
   checkpoint as a control), and the REPORT records the actual parity numbers.
6. Delivery follows the full PR, fresh exact-head review, merge queue and
   post-merge verification path.

## Stop and ask if

Ask means decide, record and continue. A GPU run follows the shared caps and
never touches Claude's pods. No softmax, dense matmul or transformer fallback is
introduced; the comparison reads existing geometric decisions only.
