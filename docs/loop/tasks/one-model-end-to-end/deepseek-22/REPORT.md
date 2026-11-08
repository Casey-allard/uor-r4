# REPORT: one-model-end-to-end, deepseek-22

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Prep:
[`one-model-end-to-end-reply-sft-2026-11-07.md`](../../../../harness/prep/one-model-end-to-end-reply-sft-2026-11-07.md).
Laptop only; no GPU, pod, model step or spend.

## What did not work, first

- **The historical `dialogue-recall-corpus` route is not buildable on current `main`.**
  Its source imports `stack_dialogue`, `stack_tracking` and `binding_probe`; all three
  modules are excluded from the current library graph with `#[cfg(any())]`. It is not a
  Cargo target, and registering it fails before generation on those unresolved modules.
  Restoring the whole historical dialogue stack would expand this unit into unrelated
  retired code.
- **The first M-World context choice was invalid.** At `context=256`, conversation 14 had
  257 tokens and the generator correctly refused the store. The accepted build uses
  `context=384`, while training remains at the model's 192-token window; the longest
  generated episode is 262 tokens.
- **`m2-reply512` is still unavailable.** Recursive store listing finds no
  `m2-reply512`, `saved437` or `dev512` data files. Repository/archive search finds the
  recorded hashes and audits, not the bytes. It remains `NOT_RUN`; no replacement was
  invented.
- **This unit does not train.** It produces and verifies the data and a complete short-SFT
  sheet. FIT and D67 review must pass before dispatch.
- **The first fresh review rejected the initial packet.** TEST FITNESS was `NOT_FIT`, D67
  was `NO GO`. The missing init/rank tools, placeholders and truncated pins, incomplete
  cost, writer-head continuation guard, single-row success threshold, and an exclusion
  check that only saw generated user turns were the blockers/majors. This revision removes
  those packet defects; the tools and a new review remain required before dispatch.

## What changed

- Recorded the lawful substitute after the historical-generator probe: the registered
  `m-world-recall-corpus`, with real oracle-memory turns and a response mask over every
  assistant response and EOS.
- Generated 20,000 deterministic M-World v2 conversations at seed 1 and protocol 2:
  2,670,958 tokens, 825,071 response tokens and 71,895 response/EOS runs. The optional
  recall blocks contain 11,578 value-bearing lines and 1,479 `Memory: none.` lines.
- Added `m2-sft-exclusions`, a deterministic Rust builder for the 96-string exclusion panel.
  It reproduces the prior panel byte-for-byte. The generator rejected 0 exact user-turn
  draws; the excluded token and mask files are byte-identical to the unexcluded build.
- Added `corpus-panel-overlap`, which decodes every document and every mask-1 assistant
  response. It checks all 96 plan/query/accepted strings, exact accepted responses, and
  word 6-grams. The mixed store has 0 in every category.
- Used the existing OASST2-only reply-masked TRAIN root for natural continuation:
  3,773,419 tokens, 3,234,805 scored tokens and 7,784 EOS.
- Mixed OASST2 then M-World with `mix-chat-corpus` into 25,071 rows, 6,444,377 tokens and
  4,059,876 scored tokens. EOS appears 79,679 times at 19,625 per million scored targets,
  with median interval 11 scored tokens. This is 26 times D3's 756 per million and close
  to DEV's 85,812 in the M-World component alone (87,139 per million), while retaining the
  human store as 80% of rows.
- Wrote the short-SFT pre-registration sheet: seed 1 only, 1,024 steps from (b64L), batch
  64 in micro-batches of 16, fresh optimizer/schedule, panel scoring every 256 steps,
  continuation guard, and explicit stop/scale conditions.
- Uploaded the three mixed-store files under
  `results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/inputs/` at HF dataset commit
  `5c8765455433a669df13bd2a1677c051383d5397`, then downloaded and re-hashed all three.
- The revised sheet pins both seed checkpoints, exact source and DEV hashes, the panel
  hashes, the base rank baselines, the base OASST2-held-out guard (132/512; floor 93),
  multi-row outcomes and precedence, and a complete time/RAM/storage/retry envelope.
- No corpus, checkpoint or generated manifest is committed.

## Measurements

The pinned tokenizer is `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.

| store | tokens | scored | EOS | EOS / million |
|---|---:|---:|---:|---:|
| OASST2 TRAIN | 3,773,419 | 3,234,805 | 7,784 | 2,406 |
| M-World oracle | 2,670,958 | 825,071 | 71,895 | 87,139 |
| **SFT mix** | **6,444,377** | **4,059,876** | **79,679** | **19,625** |

The mix's maximum scored run is 6,135 tokens, from the long natural OASST2 tail; its median
is 11. That tail is why token exposure alone is not the EOS lever. M-World supplies 90.2%
of all EOS targets and 80% of the rows.

| artifact | SHA-256 |
|---|---|
| SFT `tokens.u16` | `3a2c397e4b1a13c488096f931cf377efaeed51d2de17c231ac72d4b6291899e3` |
| SFT `response_mask.u8` | `67af9f09152510edba99795f3ff247a6f17eb80c3907836cd7c0af6d7cddb28b` |
| SFT `manifest.json` | `1c7aee4dfaffdde07f9c64a5b887b504ee21355038b6ae00897a8a97407d9bbe` |
| M-World `tokens.u16` | `3b7b556c8e45c8d90c28bc8dec74b4082620a15d87cc055a401f21630c3b7c35` |
| M-World `response_mask.u8` | `9665387299077f0560a657a2f5f8ace8605acc7292a27e7cf759d3b73b5b5543` |
| M-World `manifest.json` | `427279468476f307ded50ff5f9fe544f5c6840be76db53dd8539019ee14db9b8` |
| M2 exclusion panel | `e0a96dd64c6da637f077c0833fb96408028102082467e1a0dbbeed1edf285c98` |
| EOS census | `607bd3f50b2bc2bc929af28a70f8a98f365a7c500b67512f323f372c571b9dc6` |
| panel-overlap audit | `34a9646195cd2dd929d5ffcc779bbcbfe5157f55554f9894981ee225ba20f053` |
| OASST2 held-out base 512-row grade | `744a6a8d3843ae6b02fc1a021dd5273975f2cc788fae03ebe904165401c4e08e` |

The base checkpoint in the sheet is Claude-301 seed 1 `step-016384`:
`model.safetensors 6fd2d764…`, `config.json 1045acb6…`,
`hamming_read.safetensors 672de97f…`.

## Checks run at the head

- `cargo build --offline --locked -p uor-r4-training --bin m-world-recall-corpus
  --bin mix-chat-corpus`: passed.
- Registered the historical generator, observed its unresolved historical-module errors,
  then restored the Cargo file; no dead target remains.
- Two independent 20,000-conversation runs (with and without the M2 exclusion) produced
  byte-identical `tokens.u16` and `response_mask.u8`; the exclusion manifest records 96
  turns and 0 rejected draws.
- `mix-chat-corpus` accepted both source manifests and emitted 6,444,377 tokens with two
  exact source spans.
- `corpus-eos-exposure` read the mixed token store and mask: 79,679 EOS, all scored and
  all terminated; 0 scored targets left without an EOS.
- `cargo test --offline --locked --bin m2-sft-exclusions`: 2 passed.
- `cargo test --offline --locked --features cpu-parity --bin corpus-panel-overlap`:
  3 passed.
- `m2-sft-exclusions` reproduced the 96-string exclusion file at SHA `e0a96dd6…`.
- `corpus-panel-overlap` decoded 25,071 documents and 79,679 assistant replies:
  0 exact panel/accepted matches and 0 shared 6-grams.
- `cpu-checkpoint-grade` on the sealed seed-1 artifact and the deterministic 512-row
  OASST2 held-out sample recorded 132 top-1 rows, sample fingerprint `b724656f…`.
- SHA-256 values above were recomputed from the generated files.
- `uor-harness prep-check`: the sheet is complete; verdict remains `PENDING` pending the
  fresh review recorded below.
- `git diff --check`: passed.

## Skills used

No system-provided Codex skill applied. The repository charter, loop, corpus writers,
tokenizer protocol and compute rules were used directly.

## Still open

- Fresh non-author FIT and D67 review of the sheet.
- Run seed 1 under the sheet. Scale to seed 2 only after the sheet's positive row: teacher-
  forced EOS top-1 >= 16/256 with median rank <= 3, free EOS stops >= 16/256, at least
  four complete accepted rows, and the 512-row continuation guard >= 93.
- Export and score every 256-step checkpoint; a training loss alone does not decide this
  unit.
- `m2-reply512` remains unavailable.

## Plain summary

There is now a 6.44M-token reply-masked SFT store with 79,679 EOS targets, a short
answer median of 11 tokens, no exact overlap with the 96 M2 plan/query/answer strings, and
a pinned human continuation store. The next run starts from the existing (b64L) weights,
not a fresh model, and is staged to stop quickly if EOS rank does not move.
