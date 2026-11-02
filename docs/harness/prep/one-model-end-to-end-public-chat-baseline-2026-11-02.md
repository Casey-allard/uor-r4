# Prep sheet: public integer-chat baseline, 2 November 2026

Lab: deepseek. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md).
Order: [deepseek-6](../../loop/tasks/one-model-end-to-end/deepseek-6/ORDER.md).
Source base: `b44402e0c`.

TEST FITNESS: NOT_FIT

## Registration-integrity finding

A fresh-context reviewer found that this sheet is not a pristine
pre-registration. Before the sheet was finalized, a seed-1 smoke through the
same artifact and tokenizer observed outputs for two entries now selected in
the panel:

- `conv-talk-01` (`Hello! How is your day going?`) emitted
  `" Whilk from you."`, ids `[1044, 316, 77, 583, 351, 16, 1]`, stop `eos`,
  history 27, and 0.0438 s.
- `conv-mem-01` was run as
  `The new puppy at our house is named Pickle.` followed by
  `What did we name the new puppy?`. Both replies were
  `" Whilk from you."`, at 135.27 and 162.43 ids/s.

Those observations predate the claim that all eight entries were fixed before
outputs were read. They are not a complete fixed-panel run, but they invalidate
the present experiment's pristine-registration claim. The fixed eight-entry
panel was not subsequently run under this sheet.

## Question

What does the current public integer FBS model actually emit through the
product `uor-chat` path, and is it ready for a working conversational-model
demonstration? The measurement covers both published seeds, six prompt
categories, two multi-turn memory probes and one deterministic repeat.

This is an ungraded integration baseline. It does not establish
answer-correctness, complete-reply, fresh-phrasing or language capability.

## Base artifact

- Seed 1 artifact:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut`,
  SHA-256 `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`.
- Seed 1 tokenizer:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/tokenizer-seed-1.json`,
  SHA-256 `1ca1ad2fe36ca43b4cc7deeb46e49c275f5172720d2e2fe30fafca2c17ce1514`.
- Seed 2 artifact:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut`,
  SHA-256 `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`.
- Seed 2 tokenizer:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/tokenizer-seed-2.json`,
  SHA-256 `5c09890c37475e1fdc2ae5cb6e70c4613200a0bd6c59a1eafe7f3b4a4311f72c`.
- Decoder: greedy, temperature 0, maximum 16 response tokens, 16 worker
  threads.

## Data split

Panel:
`/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-panels/conversational-v2.json`,
160 prompts in six categories. The eight selected entries are fixed before
outputs are read:

| Panel id | Category | Turns |
|---|---|---|
| `conv-clar-01` | clarify/on-topic | 1 |
| `conv-mem-01` | multi-turn memory | 2 |
| `conv-do-01` | instruction | 1 |
| `conv-do-02` | instruction | 1 |
| `conv-talk-01` | small talk | 1 |
| `conv-story-01` | story continuation | 1 |
| `conv-unk-01` | unknowable | 1 |
| `conv-mem-02` | multi-turn memory | 3 |

The six one-turn prompts run with `--say`. The two memory probes each run as
one REPL process so prior turns remain in the session.

## Steps

For each seed, use the pinned tokenizer:

```sh
target/debug/uor-chat --stack ARTIFACT --tokenizer TOKENIZER \
  --threads 16 --temperature 0 --max-tokens 16 --say PROMPT
```

Run that command for `conv-clar-01`, `conv-do-01`, `conv-do-02`,
`conv-talk-01`, `conv-story-01` and `conv-unk-01`.

Run `conv-mem-01` and `conv-mem-02` separately as REPL stdin:

```sh
target/debug/uor-chat --stack ARTIFACT --tokenizer TOKENIZER \
  --threads 16 --temperature 0 --max-tokens 16
```

Send each entry's `user_turns` in order, then `/quit`. Capture the complete
transcript, including `[served]` timing lines. Repeat the seed-1 commands once
and compare reply ids and text.

## Readings

For every seed:

- exit status and complete reply JSON/REPL transcript;
- exact response text and response ids;
- stop reason, history depth, seconds and response ids/s;
- whether the memory replies contain `Pickle` and `tomato` respectively,
  case-insensitively.

Seed 1 runs a second time. Its reply ids and text must match the first pass.

## Bar and decision

- **Instrument pass:** both seeds load the artifact and tokenizer through
  `uor-chat`, all fixed prompts complete, every reply stops by `eos` or the
  16-token cap, and the seed-1 repeat is exact in reply ids and text.
- **Conversational readiness:** the public artifact is **not ready for a
  useful-chat demonstration** if either memory probe fails its fixed content
  anchor on either seed. Without that failure, the public artifact is only
  **ready for qualitative inspection**, never qualified.
- The report gives every transcript, including failures, and makes no broader
  language claim.

## Cost and stop rule

Local Ryzen 9 9950X only, 16 threads, `$0` external compute. Stop on a load or
protocol failure, a command timeout above 60 seconds, or a non-deterministic
seed-1 repeat. The expected run is under one minute per seed.

## Minimum test power

Two independent public model seeds, eight fixed prompt entries and eleven
model replies per seed (six one-turn replies plus two- and three-turn memory
probes), with one complete seed-1 repeat. This is enough to test product
connectivity, deterministic serving and the two fixed memory anchors. It is not
enough for answer-quality or language-capability evidence.

## Claim boundary

This test can show that the product serving path is connected and can reveal
obvious conversational failure. It cannot show that an answer is correct, that
the model generalizes, or that the model is ready to ship.

## Verdict

TEST FITNESS: NOT_FIT

**Reason:** two selected entries were observed before the registration was
finalized, so this is not the unobserved comparison claimed by the ORDER.
`PRE-RUN: FAIL`.

This verdict is an integrity verdict, not a product verdict. The focused
`uor-chat` harness check passed, and the observed smoke shows that the product
path loads the integer artifact and returns transcript records. No fixed-panel
model run was performed after this finding.

A follow-up must be registered honestly as a post-smoke fixed-panel baseline,
pin the panel file by SHA-256 as well as both artifacts and tokenizers, name the
exact worktree-built `uor-chat` binary and output root, and explicitly disclose
the observations above. It cannot claim that its selected inputs or outputs
were unread before design.

The follow-up is
[`one-model-end-to-end-public-chat-post-smoke-2026-11-02.md`](one-model-end-to-end-public-chat-post-smoke-2026-11-02.md).
It also selects protocol 2 after the project found that the original smoke's
default protocol-1 tokenization was the wrong training protocol.
