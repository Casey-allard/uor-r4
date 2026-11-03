# REPORT: fresh complete-reply evidence, deepseek-13

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](ORDER.md). Base:
`686d9c872aa3b7cb650f2ffae73d93708829827f`.

## What did not work

- **Fresh prompt templates do not fix reply collapse.** The public M1 model
  produced 22 distinct non-empty texts across 199 non-empty replies on the new
  218-prompt panel. The most common text was still
  " Got it, I'll remember that."
- **No reply is an exact reference match.** `0 / 218` generated replies
  equalled their reference reply after excluding EOS.
- **Most replies finish, but not all do.** 184 replies stopped at EOS, 12
  reached the 16-token cap and 22 hit the context limit. Nineteen
  context-limit replies were empty; the other three emitted a partial reply
  before exhausting the context.
- **Seven replies repeated a 4-gram and five decoded texts contained a
  replacement character.** Both are small shares, but neither is chat-ready.
- **The generated control shares reply templates.** The panel's prompt
  templates are new, but 104 of 218 reply templates are present in the
  `train,train` generated control at K = 1,024. This is not a canonical TRAIN
  or human-reference comparison.
- **This is still not semantic correctness.** The panel tests new prompt
  templates in a deterministic synthetic dialogue world. It does not test
  unseen words, human reference wording or useful general conversation.
- **The teacher-forced reading is capped.** `max-targets=1024` samples the
  panel's 2,533 response targets. The complete-reply pass covers all 218
  prompt/reply pairs, but the teacher-forced totals are not a full-panel
  reading.

## What changed

No model or serving code changed. This unit generated a new, previously unused
seed-2201 `dev,dev` held-out-phrasing panel and an independent same-seed,
same-size `train,train` generated control with the pinned public M1 tokenizer.
It then ran the deterministic public M1 complete-reply grader twice.

The generator is deterministic: a second panel draw was byte-identical in
tokens, mask and manifest. The two complete-reply reports were byte-identical.

## Measurements

Public seed-1 19-table artifact:
`04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`.

| Reading | Result |
|---|---:|
| Panel conversations | 64 |
| Paired prompts and replies | 218 |
| Maximum new tokens | 16 |
| EOS-complete replies | 184 |
| Token-limit stops | 12 |
| Context-limit stops | 22 |
| Exact-reference replies | 0 |
| Distinct non-empty texts | 22 |
| Empty-text replies | 19 |
| Replies containing a repeated 4-gram | 7 |
| Replies containing a decoder replacement character | 5 |
| Generated new / total 4-grams | 201 / 1,517 |
| Generated new / total 8-grams | 721 / 721 |
| Prompt templates seen in control at K = 256 / 1,024 | 0 / 218; 0 / 218 |
| Reply templates seen in control at K = 256 / 1,024 | 116 / 218; 104 / 218 |
| Reference replies seen in generated control | 86 / 218 |
| Teacher-forced admitted / top-1 / top-8 | 665 / 505 / 543 of 1,024 sampled targets |
| Full CPU trunk parity | 32,768 / 32,768 Q12 coordinates exact |

Against deepseek-12's 58-prompt panel, completion fell from 87.9% to 84.4%.
The raw count of distinct non-empty texts rose from 11 to 22, but the number
of non-empty replies rose from 54 to 199, so diversity per non-empty reply
fell from 20.4% to 11.1%. Repeated-4-gram shares stayed near 3.2%, and
new-4-gram shares stayed near 13.3%. The exact-reference count remained zero.

Panel identity:

- tokens:
  `73c2375664cf5546ae62fc0861c4f7fbf3b99607fb70e8eccf37ba7a1654ec9c`;
- response mask:
  `f038e75405f2a8c6f7f45cc7603e935b9991d855507a1c32cf0f766be5f945e6`;
- manifest:
  `73fd6f141b4f82dc61599452c6ccd9b2de55c9c1ef40a71149d38f3911051bdd`.

Generated-control identity:

- tokens:
  `648968ddbdbe5329dc3a063c34f3d950b56f56c66d1ae2f761269a9bf9847ddd`;
- response mask:
  `47bf864e0d751391ccaf210ef87ee75817665adf62e642906019b9fba0b25d07`;
- manifest:
  `aa723193aeb4f64a7f56872c0bb14465b8027cf392827743782c37d5b99ffe67`.

The full source report SHA-256 is
`dede2591d9049160b3df5e54aa987df766152805b8a5e8a1d47f475eac185dc6`.
The committed summary is
[evidence-deepseek-13-fresh-replies.json](../../../../labs/one-model-end-to-end/evidence-deepseek-13-fresh-replies.json)
(SHA-256
`e2c2fe342af76becee79e37ba2df57e6a7e6a42cf6b0e056dce006d721dc1343`).

## Checks run at the head

- `cargo test --offline --locked --features cpu-parity --lib --bin
  m2-evidence -j2`: 16 library and 3 CLI tests passed.
- Strict package Clippy with `--all-targets --no-deps -- -D warnings` passed;
  dependency dead-code warnings remain.
- Edition-2021 rustfmt and `git diff --check` passed.
- The panel writer completed twice; token files, masks and manifests were
  byte-identical.
- Two complete-reply runs completed with identical 218-row reports.
- `uor-harness loop report --check` and the roadmap render check passed.

## Skills used

No available Codex skill applied. The existing panel writer, M2 evidence lane,
public artifact and delivery rules were read directly.

## Still open

- A trained full-path FBS artifact that carries learned Hamming, turn memory
  and point-key behavior.
- Count-table, memory-off and no-read controls.
- A human-authored semantic panel and a fresh-reply-wording judge.
- GPU qualification, throughput qualification and a useful local chat model.

## Plain summary

The public M1 model's non-chat readiness negative survives a larger panel whose
prompt templates are new. It still produces a small set of near-fixed phrases,
never matches a reference reply and emits five replacement-character texts.
Completing a reply is not the same as answering it.
