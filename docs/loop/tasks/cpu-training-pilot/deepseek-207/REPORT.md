# REPORT: cpu-training-pilot, deepseek-207

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[prompt-reply serving sheet](../../../../harness/prep/cpu-training-pilot-prompt-reply-serving-2026-11-05.md).
Base: `c0e6b6757`.

## Status

Complete as **KEEP**. The persisted prompt/reply sidecar reproduces the
deepseek-204 CPU selections on both public artifacts, and the OpenAI loopback
path emits the complete retrieved reply with EOS or falls back when the request
cannot fit it.

This is a serving-contract result for exact-memory retrieval. It is not fresh
response generation, general language understanding, GPU qualification or a
cross-model capability comparison.

## What did not work

- Attempt 1 at implementation `6a48ad613` showed the selection result but did
  not bind the sidecar's ignored BOS/EOS/UNK ids to the loaded protocol. Its
  output remains at `2026-11-05`.
- Attempt 2 added control-id validation but the exact API build found a
  CLI-only protocol scope fault. Its selected rows remain at
  `2026-11-05-attempt-2`.
- Attempt 3 fixed the CLI scope but the low-level serving attachment still did
  not independently verify the tokenizer hash and protocol version. Its
  selected rows remain at `2026-11-05-attempt-3`.
- The integer crate's blanket all-target Clippy reports pre-existing
  diagnostics outside this unit. No diagnostic names a new prompt-reply file.

## What changed

- Added `uor-r4.stack-prompt-reply-readout/1`, binding artifact SHA-256,
  tokenizer SHA-256, dialogue protocol, ignored control ids and complete
  prompt/reply pairs.
- Added strict parsing, size limits, vocabulary checks, exact rational Jaccard
  ordering, lower-ordinal ties and protocol-control filtering.
- Added `cpu-prompt-reply-sidecar` to build the persisted table from the TRAIN
  corpus.
- Added `cpu-prompt-reply-serving-check` to compare every served selection with
  the independently derived CPU arm.
- Added `uor-chat --prompt-reply` for loopback serving. A fit response is
  emitted with EOS; an oversized response falls back to greedy model
  generation.
- Extended OpenAI health, model, capabilities, response and streaming metadata
  with the prompt-reply path and sidecar SHA-256.
- No `UORLUT01`, Hamming/read kernel, CUDA, float-training or Claude-owned
  source changed.

## Measurements

Attempt 4 output:
`local/deepseek/prompt-reply-serving/2026-11-05-attempt-4`.

The sidecar contains 453 TRAIN entries and is 62,846 bytes:

| Artifact | Exact | Opening | Body | Emitted | Premature | Prompts/s |
|---|---:|---:|---:|---:|---:|---:|
| seed 1 | 39 | 131 | 948 | 471 | 0 | 9,818.64 |
| seed 2 | 39 | 131 | 948 | 471 | 0 | 10,061.95 |

All **471/471** selections matched the CPU arm per row by selected ordinal and
complete-reply SHA-256. Both artifacts shared decision SHA-256:

`a142efc85caae824c088441ff654e3c78ad828c6aab408ff1fb22d46d4fd558c`

Sidecar SHA-256:

- seed 1:
  `2a298b44ea34d1a8e2d40191951d3da9b3300e2ac7de65e6c2ec13950a0b6424`;
- seed 2:
  `6504dcb047ce7157dd5e968c29441505cf48d993f01ec63222125634c61963b7`.

Both sidecars and reports repeated exactly after removing timing fields.
Oversized-response fallback is covered by the API test at a three-token cap.

Resources:

- release build: **19.09 s**, 1,284,164 KiB peak RSS;
- seed 1 / seed 2: **3.79 / 3.76 s**, 205,824 / 205,460 KiB peak RSS;
- seed 1 / seed 2 repeats: **3.80 / 3.76 s**;
- retained evidence: **376,443 bytes**;
- sealed `SHA256SUMS`:
  `581d7590feae3fee30e3009b0a400bf4b72efda1e803c25ed548d94cd19c6cfc`;
- sidecar builder binary:
  `f8ab1cf1829fa998574c1251afd1e28a20c7d0a93bd98fc7a718be046ce42d56`;
- serving-check binary:
  `0a0bf46c1d6235afffd300c353bcee4fd773b3cf9abd3145e464bb3517b18c12`.

Compact evidence: [result.json](evidence/result.json).

## Interpretation

The exact-memory completion now reaches the actual loopback boundary. It is
fast because a hit returns a recorded response rather than stepping the model
for every output token. That speed statement applies only to retrieval hits;
the implementation deliberately falls back to the ordinary integer path when
the response does not fit the request.

The result does not make prompt retrieval a geometric vector model yet. The
sidecar is a strict discrete token-set index. A later VSA or artifact-bound
vector signature may replace literal token-set intersection only if it
preserves the row-level result and its serving identity.

## Checks run at the head

- serving library: **473 passed, 1 expected ignore**;
- OpenAI loopback: **19 passed**, including complete hit, cap fallback,
  metadata and all existing streaming/route controls;
- CPU release suite: **394 passed, 4 expected ignores**, plus binary tests;
- focused prompt-reply parser/selection tests: **4 passed**;
- strict `uor-r4-cpu-train` Clippy and targeted rustfmt: passed;
- integer serving library tests: passed; blanket all-target Clippy retains
  pre-existing failures outside the touched files;
- source, tokenizer, TRAIN, panel and artifact pins: matched;
- sidecar round-trip and 64 MiB bound: passed;
- 471/471 per-row CPU/serving parity on both artifacts;
- both sidecars and reports repeated exactly;
- sealed evidence manifest read-back: passed.

## Skills used

No system-provided Codex skill applied. Repository D22/D67, D84 CPU research,
integer serving, exact-evidence and protected-delivery procedures were used.

## Still open

- replace literal token-set indexing with an artifact-bound discrete/VSA
  signature that preserves this parity;
- preserve sidecar identity through exported deployment bundles;
- fresh response generation beyond responses already present in TRAIN;
- Claude's independent scale, export and serving track.

## Plain summary

The prompt-retrieval result is now an artifact-bound integer serving sidecar.
It reproduces the CPU selection exactly on both artifacts, returns complete
responses through the loopback boundary, falls back safely when needed, and
runs at about ten thousand prompt selections per second.
