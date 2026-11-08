# ORDER: one-model-end-to-end, deepseek-33 (same-scorer packet parity)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `208a29ce5`.

## Goal

Close the remaining same-scorer float/integer parity gap with an actual GPU
reading. Use the merged `packet-parity` instrument on the content-ratio run's
selected step-512 checkpoint and final step-2,048 checkpoint, under both
registered `per-turn/v1` and `one-user-turn/v1` renderings.

## Cancel probe

- The content-ratio run is sealed and read back at
  `results/deepseek/one-model-end-to-end-content-ratio-2026-11-08/runs/s1-content-ratio-20261108t055107z-adb12718d6c9ea88/`.
- `crates/uor-r4-float-train/src/bin/packet-parity.rs` is merged on `main`.
- No open DeepSeek packet-parity PR or live DeepSeek pod exists.
- The run uses only a DeepSeek-owned 5090 inside the four-pod/$8-per-hour caps.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/deepseek-33/**`
- `docs/labs/one-model-end-to-end/packet-parity-2026-11-08/**`
- `docs/research/notebook/float-training.md`

## Done when

1. The parity job fetches the sealed content-ratio run and re-hashes every input
   it uses before model work.
2. Step 512 and step 2,048 are each compared on the same prompt and shortest
   accepted reference through the float checkpoint and integer artifact.
3. Both renderings report chosen-token mismatches, rank mismatches, top-1
   counts, EOS-rank disagreements and maximum rank delta.
4. The evidence binds the source run, checkpoint files, artifact, tokenizer,
   panel files, binary hashes, CPU/GPU identity and job log; it is archived,
   read back and the owned pod is deleted.
5. One REPORT and notebook row record the actual readings without an M2,
   capability or useful-chat claim.

## Stop and ask

An instrument, checkpoint, artifact, hash, panel or readback failure stops the
run and trains nothing further. Never substitute `m2-fresh256` for
`m2-reply512`; the latter remains unavailable. Never touch another lab's pods,
leases, branches or files.
