# REPORT: cpu-training-pilot, deepseek-211

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[compressed prompt VSA sheet](../../../../harness/prep/cpu-training-pilot-prompt-vsa-compression-2026-11-06.md).
Base: `4d1e705aa`.

## Status

Complete as **REJECT** for the selected lossy compressed signature.

TRAIN-only calibration selected the 1024-bit modulo mapping. It made prompt
selection about **3.67x** faster than the exact 4,096-bit index on both
artifacts, but it also lost one exact reply, five correct openings and 50
correct body positions on each artifact. Under the registered
equal-or-better-quality bar, exact binary VSA remains the serving default.

This closes the fixed 256/512/1024 modulo and 1024 SplitMix64 candidate set as
tested. It does not reject every learned, collision-avoiding or geometrically
structured compressed signature.

## What did not work

- Calibration ranked 1024-bit modulo first, then 1024-bit SplitMix64. On the
  TRAIN inner split it agreed with exact selection on only **81/87** held-out
  prompts; 512 modulo reached **73/87**, 1024 SplitMix64 **77/87**, and 256
  modulo **63/87**.
- On fresh seed 2778 and both artifacts, compressed retrieval changed from
  exact **49/138/1061** to **48/133/1011** for exact replies, correct openings
  and correct body positions. Both arms emitted all **445** reachable replies
  with zero premature BOS.
- The selection-stage speedup did not materially reduce whole-process cost.
  Exact selector processes took **3.54-3.56 s** on seed 1 and **3.51-3.52 s**
  on seed 2; compressed processes took **3.71-3.72 s** and **3.67-3.68 s**,
  because calibration and the unchanged CPU/panel setup dominate this bounded
  check.
- Attempt 1 stopped before panel generation because the standalone generator's
  checked-in lockfile was stale under `--locked`. It is retained as a sealed
  control fault. Attempt 2 used an isolated offline generator copy and the same
  source, seed, cell, protocol and tokenizer.
- Independent review remained unavailable under the current delegation policy.
  FIT and GO are explicitly a local exact-head self-review.

## What changed

- Added an opt-in `PromptReplySignatureConfig` research constructor with
  vocabulary, modulo and SplitMix64 mappings. `from_entries`,
  `from_sidecar`, `load` and serving still use the exact vocabulary mapping.
- Extended `cpu-prompt-reply-serving-check` with a fixed TRAIN-only candidate
  calibration, selected-configuration reporting, exact/compressed arms and
  exact-arm CPU row parity.
- Added collision, determinism, sidecar round-trip, calibration and fixed-split
  tests.
- Preserved the `uor-r4.stack-prompt-reply-readout/1` schema and bytes.

No `UORLUT01`, serving kernel, CUDA, float-training or Claude-owned source
changed. The source implementation is 518 additions and 51 deletions across
three code files; documentation and evidence add the rest.

## Measurements

Fresh panel:
`local/deepseek/prompt-vsa-compression/2026-11-06-attempt-2/panels/dd-2778`,
seed 2778, `dev,dev`, protocol 2, 128 conversations, **445**
reference-reachable replies per artifact.

- tokens:
  `4e65cd38cbd5ff0d9e4b8934cdbb70a0ccc6bff12c21b08a85d7b719d8ebecb4`;
- mask:
  `f36f4e12bfbea3c5adef9db9bb79d33e450e6622292ab4b9fffc55376fc76a4b`;
- manifest:
  `d95577dbfb14f63cb17643e4d939d6c190c8ec5735e0bc389f4ef1baac6ccb1a`.

Calibration used **366** inner library entries and **87** held-out TRAIN
prompts. Three alternating fresh-process runs per artifact and arm repeated
after timing removal.

| Artifact | Arm | Exact | Opening | Body | Emitted | Premature | Median prompts/s |
|---|---|---:|---:|---:|---:|---:|---:|
| seed 1 | exact 4,096 | 49 | 138 | 1061 | 445 | 0 | 45,928.37 |
| seed 1 | compressed 1,024 modulo | 48 | 133 | 1011 | 445 | 0 | 168,688.40 |
| seed 2 | exact 4,096 | 49 | 138 | 1061 | 445 | 0 | 45,045.05 |
| seed 2 | compressed 1,024 modulo | 48 | 133 | 1011 | 445 | 0 | 165,797.32 |

Aggregate selection medians were **45,928.37 / 166,916.73 prompts/s**
(**3.63x**). Exact and compressed decisions were artifact-invariant and
repeated exactly:

- exact decision SHA:
  `947ff38b7840b4194de5d56936033b24a01248d5c270c85cc103f3ae779b9353`;
- compressed decision SHA:
  `52c9232111706cf89e4e9aa22dacca81ce7400fd5e8d1f809b913a1f39b690e1`.

Sidecars remained byte-identical to deepseek-207:
`2a298b44...b6424` and `6504dcb0...e63b7`. Peak RSS was **210,620 KiB**.
Attempt 2 retained **140,401 bytes**; sealed `SHA256SUMS`:
`37c76996e88e31869853c5243259d8f9b74782bd730f5634f0e088cb771d8554`.

Compact evidence: [result.json](evidence/result.json).

## Interpretation

Modulo folding creates token collisions. Those collisions change set
intersection and union counts, so compressed Jaccard no longer represents the
exact prompt set and can reorder close TRAIN candidates. The 1024-bit candidate
was the best TRAIN proxy, but that proxy did not preserve the fresh panel's
quality.

The speed gain is real for isolation of the selector loop, but its absolute
effect is small: the exact selector is already fast, and the measured process
wall time did not improve. Compression is therefore not a useful serving
tradeoff on this panel. A future compressed representation needs to preserve
rank order, for example through learned token codes or a geometric collision
resolution path, rather than only reducing bit width.

## Checks run at the head

- implementation review: local exact-head self-review at `16f35277a`;
- prompt-reply tests: **8 passed**;
- calibration tests: **2 passed**;
- serving library: **477 passed, 1 expected ignore**;
- OpenAI loopback: **17 passed**;
- CPU library: **394 passed, 4 expected ignores**;
- formatting, claim wording, policy and D23/D25/D32 ratchets passed with no new
  error;
- policy retained advisories for the existing item-scope omission of
  `prompt_reply.rs`, the initial claim/ORDER commit grouping, and the standard
  prep-sheet `Stop rule` heading; none is a hard policy failure;
- source, binary, generator, tokenizer, artifacts, TRAIN and sidecar pins
  matched;
- exact arm matched the CPU row oracle on every attempt-2 row;
- all three runs per artifact and arm repeated after timing removal;
- attempt 1 and attempt 2 seals verified.

## Delivery

Pending review, protected queue merge and fresh-main verification.

## Skills used

No system-provided Codex skill applied.

## Still open

- learned or geometry-aware compressed signatures that preserve exact
  retrieval ordering;
- fresh response generation beyond learned TRAIN replies;
- artifact/export-bundle propagation of the prompt-reply sidecar;
- Claude's independent scale, export and serving track.

## Plain summary

The compressed prompt index was roughly 3.7 times faster inside the selector,
but it lost exact replies, openings and body correctness on both artifact
seeds. The exact 4,096-bit binary index remains the default and the compressed
candidate is rejected. Future work should make compression rank-preserving
instead of folding different tokens onto the same bit.
