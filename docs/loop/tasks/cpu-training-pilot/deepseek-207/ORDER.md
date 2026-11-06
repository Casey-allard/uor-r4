# ORDER: cpu-training-pilot, deepseek-207 (prompt-reply serving)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `c0e6b6757`.

## Goal

Package the deepseek-204 prompt-set completion result as an artifact-bound
integer sidecar and attach it to the loopback serving path. A hit emits one
complete TRAIN reply followed by EOS; a miss, oversized response or
insufficient token cap falls back to the existing greedy model path.

This is serving integration for exact-memory retrieval, not fresh response
generation or general language understanding.

## Fixed construction

The sidecar stores TRAIN prompt/reply token pairs. Prompts are ranked by exact
unigram-set Jaccard using integer cross-multiplication; equal scores choose the
lower TRAIN ordinal. Protocol control IDs are ignored on query construction.
The sidecar binds the artifact SHA-256, tokenizer SHA-256 and dialogue protocol
version, and is limited to 64 MiB.

`uor-chat --stack ... --prompt-reply SIDECAR --serve ...` loads and validates
the sidecar before opening loopback. The stored response is emitted only when
its tokens plus EOS fit both the request token cap and model context.

## Arms and checks

1. persisted sidecar selection;
2. same in-memory CPU selection from the sealed panel;
3. normal model fallback when the token cap is too small;
4. invalid artifact/tokenizer/protocol/control-token sidecars.

## Bar

For both public artifacts:

- all 471 panel decisions match the CPU prompt-retrieval arm by selected
  ordinal and complete-reply SHA-256;
- treatment remains **39** exact, **131** correct openings, **948** correct
  body positions and zero premature BOS;
- all 471 reachable prompts emit a complete reply;
- the sidecar round-trips from persisted bytes and repeats exactly;
- selection throughput is at least **500 prompts/s**;
- OpenAI API tests prove complete-response emission, cap fallback and identity
  metadata;
- no `UORLUT01`, serving arithmetic, CUDA, float-training or Claude-owned
  source changes.

## Cost

Local CPU only on the Ryzen 9 9950X. The run is capped at 30 minutes, 2 GiB peak
RSS and 32 MiB retained evidence.

## Done when

1. Sidecar format, builder, serving attachment, CLI flag and focused tests are
   implemented.
2. FIT and GO precede the persisted two-artifact serving check.
3. REPORT, compact evidence and program records are merged to main.
4. The claim is released after fresh-main verification.

## Stop rule

Stop on source, artifact, tokenizer, panel, sidecar, selection, API, parity,
determinism or output-root drift; fewer than 200 reachable rows; a vacuous or
size-invalid sidecar; throughput below 500 prompts/s; or any need to change
`UORLUT01`, serving kernels, CUDA, float training or Claude-owned source.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-207/ORDER.md
