# REPORT: complete-reply count-table control, deepseek-14

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](ORDER.md). Base:
`640367b302af5a0aecd09d78fc66c9c1c23437c6`.

## What did not work

- **Neither arm answered the prompt.** The public M1 model and the count table
  both produced zero exact-reference replies across 218 paired prompts.
- **The count table completed more replies by collapsing harder.** It reached
  EOS on 196 replies but produced only four distinct non-empty texts across all
  199 non-empty responses. The model reached EOS on 184 and produced 22
  distinct non-empty texts.
- **The model's extra text forms are not correctness.** All 199 non-empty
  model paths differed from the count-table path at the first token and as a
  whole reply. That measures departure from the naive control; it does not
  establish that the departure is useful.
- **The model still repeated and corrupted more text.** Seven model replies
  repeated a 4-gram and five contained a decoder replacement character. The
  count table had neither.
- **Canonical TRAIN is unavailable on this host.** The control corpus is the
  declared seed-2201 `train,train` generated control used by deepseek-13, not
  canonical TRAIN.
- **Memory-off and no-read remain unread.** The public 19-table M1 artifact
  carries neither a serving turn memory nor the Hamming mechanism those arms
  would ablate.

## What changed

`m2-evidence` now builds and records a deterministic successor count table
when a reference corpus is supplied with complete-reply generation.

- Only adjacent tokens inside one reference document contribute. The final
  token of a document never transitions to the next document.
- The commonest successor wins; equal counts break to the smaller token id.
- An unseen previous token uses the corpus-wide commonest successor with the
  same tie rule.
- The count table and model use the same paired prompt, document prefix,
  context limit, 16-token limit and tokenizer EOS.
- The report records the source role, 8,467 in-document transitions, 560
  previous tokens, full count-table reply metrics, and first-token/whole-reply
  differences from the model.

## Measurements

Public seed-1 19-table artifact:
`04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`.

| Reading | Public M1 model | Count table |
|---|---:|---:|
| Replies | 218 | 218 |
| EOS-complete | 184 | 196 |
| Token-limit stops | 12 | 0 |
| Context-limit stops | 22 | 22 |
| Exact-reference replies | 0 | 0 |
| Distinct non-empty texts | 22 | 4 |
| Empty-text replies | 19 | 19 |
| Repeated-4gram replies | 7 | 0 |
| Replacement-character replies | 5 | 0 |
| New / total 4-grams | 201 / 1,517 | 796 / 1,583 |
| New / total 8-grams | 721 / 721 | 787 / 787 |

First-token differences: 199 / 218. Whole-reply differences: 199 / 218.

Teacher-forced scoring and CPU parity remained unchanged:

| Check | Result |
|---|---:|
| Admitted / top-1 / top-8 | 665 / 505 / 543 of 1,024 |
| CPU Q12 query coordinates | 32,768 / 32,768 exact |
| Writer token / top-8 / admitted-cell mismatches | 0 / 0 / 0 |

The two complete evidence reports were byte-identical. Their SHA-256 is
`17db3a53369bf10c04497290a03c3fc0c5f4dab22bd01a36b8f42a45019e62cb`.
The committed compact summary is
[evidence-deepseek-14-count-table.json](../../../../labs/one-model-end-to-end/evidence-deepseek-14-count-table.json).

## Reading

The count table is a stronger completion arm than the public model on this
panel: 196 against 184 EOS completions. It is a far weaker diversity arm:
four texts against 22. Both fail exact-reference correctness. The model departs
from the naive successor control on every non-empty reply, but that departure
contains no demonstrated answer quality and includes more loops and decode
corruption.

This closes the count-table-control instrument gap for the public panel. It
does not turn the public M1 model into a useful chat result and does not provide
the memory-off/no-read controls.

## Checks run at the head

- `cargo test --offline --locked --features cpu-parity --lib --bin
  m2-evidence -j8` in a fresh target: 18 library and 3 CLI tests passed.
- Strict package Clippy with `--all-targets --features cpu-parity -- -D
  warnings`: passed.
- Package `cargo fmt --check`, JSON validation and `git diff --check`: passed.
- Two public complete-reply runs from the final release binary were
  byte-identical.
- Existing teacher-forced, writer and 19-table CPU trunk parity checks
  remained exact in both reports.
- `loop report --check`, `loop render --check` and `loop gates --full`: passed.

## Skills used

No available Codex skill applied. The existing complete-reply instrument,
corpus format and repository delivery rules were read directly.

## Still open

- A trained full-path FBS artifact carrying learned Hamming, turn-memory and
  point-key behavior, so the other controls can be implemented.
- Semantic reply correctness and genuinely fresh reply wording.
- A configuration that improves a held-out capability bar.
- GPU qualification of the serving and training track.

## Plain summary

The evidence lane can now compare the public model with a simple successor
count table on exactly the same 218 prompts. The count table completes more
replies but collapses to four texts; the model produces more text forms but
still matches no reference and repeats or corrupts more. Neither is useful
chat. This supplies the missing count-table control while leaving memory-off
and no-read honestly unread on an artifact that lacks those mechanisms.
