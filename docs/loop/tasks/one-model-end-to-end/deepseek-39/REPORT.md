# REPORT: one-model-end-to-end, deepseek-39

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Base: `fc2d2b143`.

**The SFT mix is now CPU-readable and the geometric opening selector moves the
first token.** Rebuilding the manifest over the sealed source stores preserves
the exact token and mask bytes. An exploratory CPU probe then selected the
right first token on 26/64 held-out OASST2 dialogue examples, against 0/64 for
the model and 0/64 for the shuffled-target control. This is development
evidence, not a fresh-panel or GPU-qualified chat result.

## What did not work, first

The CPU opening-copy probe rejected the sealed reply-SFT mix with
`corpus manifest vocabulary differs`: its manifest lacks the
`tokenizer.vocab_size`, `tokenizer.bos_id` and `max_tokens` fields required by
`uor_r4_cpu_train::validate_corpus_manifest`.

The 64-example transfer probe also closed without complete replies. Every arm
hit the 64-token cap, and no reply matched its reference. The first-token gain
therefore does not by itself produce a usable conversation.

## What changed

`mix-chat-corpus` now refuses incompatible source tokenizer identity, CID,
BOS, vocabulary, response counts and content hashes. It records the output
`vocab_size`, BOS/EOS/UNK ids, tokenizer CID, dialogue protocol and the maximum
input `max_tokens`. The source token bytes, masks and ordering are unchanged.

Added a real-binary integration test that invokes `mix-chat-corpus` on two
synthetic UORT stores, checks the token/mask bytes, and passes the emitted
manifest through `uor_r4_cpu_train::validate_corpus_manifest`. A second test
proves mismatched tokenizer hashes, vocabulary widths and CIDs are each refused
before the output is claimed. Its source stores use different `max_tokens`
values, proving the output takes the maximum covering both.

## Measurements

The fixed producer was rerun on the same two source stores used by the sealed
SFT mix. Its output retained the exact sealed content:

- tokens SHA-256:
  `3a2c397e4b1a13c488096f931cf377efaeed51d2de17c231ac72d4b6291899e3`;
- mask SHA-256:
  `67af9f09152510edba99795f3ff247a6f17eb80c3907836cd7c0af6d7cddb28b`;
- token count: 6,444,377; scored targets: 4,059,876;
- repaired manifest SHA-256:
  `a9fba3b4fdb7611625668d8153b9ec7c60ecf57be8adb7c326b2b811524fb809`;
- recorded `max_tokens`: 1,048,576; tokenizer vocabulary 4,096; BOS 0; EOS 1;
  UNK 2; protocol `uor-r4.literal-role-dialogue/2`.

The exploratory CPU transfer run used the sealed selected step-256 artifact,
eight disjoint first-8 training keys and eight held-out keys, eight examples
per key, 32 workers and a 64-token reply cap:

| Arm | First-token top-1 | Complete replies |
|---|---:|---:|
| model only | 0/64 | 0/64 |
| point key | **26/64** | 0/64 |
| target medoid | **26/64** | 0/64 |
| target prototypes (k=4) | **26/64** | 0/64 |
| eight-neighbour vote | 23/64 | 0/64 |
| shuffled targets | 0/64 | 0/64 |

The point, medoid and prototype arms beat the shuffled controls by 26 paired
rows, above the two-sigma threshold of 11. CPU/serving parity was exact on all
arms: 0 token mismatches, 0 query-coordinate mismatches and 0 kept-cell
mismatches over 1,791 positions. The probe took 5.522 seconds and peaked at
418,412 KiB RSS.

The held-out examples here are drawn from the reply-SFT training distribution,
not `m2-fresh256`. They show that the integer trunk's opening point contains
usable prompt signal and that geometric nearest selection can expose it. They
do not establish fresh-phrasing generalization or useful chat.

## Checks run at the head

- `cargo test -p uor-r4-training --test mix_chat_corpus_manifest --offline`:
  2 passed.
- `cargo test --offline -p uor-r4-cpu-train --lib`: 455 passed, 0 failed, 4
  ignored.
- The real producer run over OASST2 and M-World reproduced the sealed token and
  mask byte for byte and emitted the complete manifest above.
- The repaired manifest passed `uor_r4_cpu_train::validate_corpus_manifest`
  through the integration test.
- The CPU point-transfer probe completed with exact CPU/serving parity on all
  arms.
- `cargo check --offline --locked -p uor-r4-training --bin mix-chat-corpus`
  passed.
- Targeted `rustfmt --check` passed for both changed Rust files. The repository
  still has unrelated pre-existing formatting drift under `cargo fmt --all`;
  this unit did not reformat unrelated files.
- `uor-harness loop gates --full`: all gates green.

## Skills used

No system-provided Codex skill applied. The repository's corpus, D84 and loop
rules are used directly.

## Still open

- The 26/64 transfer reading is not a fresh-panel result. It needs an
  `m2-fresh256`-adapted CPU panel run before it can change the training plan.
- The point arm improves only the opening; all 64 replies still hit the cap.
  EOS and complete-reply selection remain separate blockers.
- `m2-reply512` remains unavailable.

## Plain summary

The tool that mixed the reply-training corpus left required vocabulary and BOS
metadata out of its manifest, so the laptop trainer could not open the exact
corpus. That producer contract is repaired. A small CPU probe then shows the
geometric point key can choose the first word on held-out training documents,
but it still cannot produce a complete answer; fresh-panel testing and
termination remain open.
