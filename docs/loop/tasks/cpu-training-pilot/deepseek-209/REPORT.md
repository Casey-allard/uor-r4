# REPORT: cpu-training-pilot, deepseek-209

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[prompt VSA sheet](../../../../harness/prep/cpu-training-pilot-prompt-vsa-2026-11-06.md).
Base: `a4981948e`.

## Status

Complete as **KEEP**. The prompt-reply selector now represents each loaded
prompt as a 4,096-bit binary vocabulary vector and computes exact set Jaccard
with bitwise AND/OR popcounts. All 471 decisions on both artifacts match the
literal-set selector and the CPU row oracle.

Median selection throughput increased from **9,820.55** to **46,362.99
prompts/s**, a **4.72x** aggregate gain.

## What did not work

- The first candidate was an exact one-hot binary vocabulary vector, so it
  could preserve behavior but could not claim compression: every row still
  carries 64 machine words.
- No learned/random hyperplane compression was tested here; doing so would
  change the retrieval rule and needs a separate capability experiment.
- Independent review remained unavailable under the current delegation policy.
  FIT and GO are explicitly a local exact-head self-review.

## What changed

- The loaded `PromptReplyIndex` now holds both original prompt tokens, for
  byte-stable sidecar serialization, and a vocabulary-dimensional `u64`
  signature.
- Token `t` sets bit `t % 64` of word `t / 64`.
- Selection computes intersection and union using bitwise AND/OR and
  `count_ones`.
- Hamming symmetric difference follows directly as `union - intersection`.
- The persisted `uor-r4.stack-prompt-reply-readout/1` payload is unchanged.
- No `UORLUT01`, serving-kernel, CUDA, float-training or Claude-owned source
  changed.

## Measurements

Output: `local/deepseek/prompt-vsa/2026-11-06`.

Three alternating fresh-process runs per artifact:

| Artifact | Literal set median | Binary VSA median | Speedup |
|---|---:|---:|---:|
| seed 1 | 9,653.82 prompts/s | 46,449.70 prompts/s | 4.81x |
| seed 2 | 9,987.28 prompts/s | 46,276.28 prompts/s | 4.63x |
| aggregate | 9,820.55 prompts/s | 46,362.99 prompts/s | 4.72x |

All arms retained:

- **471/471** reachable/emitted rows;
- **39** exact replies;
- **131** correct openings;
- **948** correct body positions;
- zero premature BOS;
- decision SHA-256
  `a142efc85caae824c088441ff654e3c78ad828c6aab408ff1fb22d46d4fd558c`.

The sidecars remained byte-identical to deepseek-207:

- seed 1:
  `2a298b44ea34d1a8e2d40191951d3da9b3300e2ac7de65e6c2ec13950a0b6424`;
- seed 2:
  `6504dcb047ce7157dd5e968c29441505cf48d993f01ec63222125634c61963b7`.

Resources:

- release build: **22.56 s**, 1,295,864 KiB peak RSS;
- selector processes: **3.72-3.84 s**, maximum process RSS **207,104 KiB**;
- retained evidence: **153,616 bytes**;
- sealed `SHA256SUMS`:
  `a5c95721a0dbef051cc4894db1d9589b2a87e49cebd30e3407fd6bfe69c87894`;
- binary-VSA SHA-256:
  `2b862acb8d7c950acdc4e67e50c1290ad212754c9662ff33057bc78fc8e874e8`;
- literal-set baseline SHA-256:
  `984422962013c46dd1d7ca07aa04805b7ed7bab4804296e9389cad541dd04f98`.

Compact evidence: [result.json](evidence/result.json).

## Interpretation

The result is an exact representation change, not a model or retrieval-quality
change. AND/OR popcounts recover the same cardinalities as ordered set
intersection and union, and all row decisions verify the equivalence on the
sealed panel.

The speed gain comes from replacing per-pair ordered-set operations and query
allocation with fixed-width bit operations. It remains a dense one-hot vector
over the vocabulary rather than a learned low-dimensional embedding.

## Checks run at the head

- VSA prompt-reply tests: **5 passed**, including signature popcount and
  Hamming-symmetric-difference identities;
- serving library: **474 passed, 1 expected ignore**;
- OpenAI loopback: **19 passed**;
- CPU release suite: **394 passed, 4 expected ignores**, plus binary tests;
- 12 alternating selector processes completed;
- every run matched literal-set, binary-VSA and CPU row decisions;
- three runs per artifact/arm repeated after timing removal;
- baseline and VSA binary hashes matched their pins;
- targeted formatting, `git diff --check`, claim wording and repository policy
  checks passed with no new error;
- integer blanket all-target Clippy retains pre-existing diagnostics outside
  the touched file.

## Delivery

PR #3767 merged through the protected queue as
`b8e01c13bb07c69c9484a9a850489ce3c245d631`. Fresh main retained the reviewed
source and evidence hashes, passed the serving/API/CPU suites, and reproduced
the sealed seed-1 VSA report after timing and absolute-path removal.

## Skills used

No system-provided Codex skill applied.

## Still open

- a compressed or learned VSA signature that can reduce the 4,096-bit width
  while retaining the retrieval result;
- fresh response generation beyond learned TRAIN replies;
- artifact/export-bundle propagation of the sidecar;
- Claude's independent scale and serving track.

## Plain summary

The verified prompt selector now runs in a binary vocabulary vector space with
AND/OR popcount Jaccard. It makes exactly the same 471 decisions on both
artifacts and is about 4.7 times faster than the literal ordered-set
implementation.
