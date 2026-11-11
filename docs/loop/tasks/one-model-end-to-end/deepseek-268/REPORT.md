# REPORT: one-model-end-to-end, deepseek-268 (scenario corpus generator)

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Laptop only; no model, pod or spend.

**In one line:** the deterministic scenario generator now emits aligned
TRAIN, DEV and Fresh stores that the existing trainer corpus ABI accepts, but
no model was trained and no language or capability claim is made.

## What did not work, first

- **The first real `mix-chat-corpus` run refused the store.** Each split
  manifest carried the tokenizer ids, CID and vocabulary but omitted
  `tokenizer.path` and `tokenizer.sha256`, so the mixer reported:
  `local/deepseek/scenario-corpus-v1/train: tokenizer SHA-256 differs from the
  output tokenizer`. The top-level scenario manifest was correct; the
  trainer-facing split manifest was not.
- **The first focused test covered only a mask mutation.** The ORDER requires
  refusal of byte, label, mask and split mismatches, so the test now mutates
  token bytes, a mask, both manifest levels, two scenario-label files, and
  removes one split file. Every case must fail the regeneration check.
- **No model was trained or scored.** This unit produces inputs. It says
  nothing about whether the model learns, remembers, attends, replies or
  generalizes.

## What worked

- One seed generates all ten scenario families: memorise, paraphrase,
  selection, update, abstention, copy, general dialogue, no-memory control,
  wrong-memory control and stale-memory control.
- Each `ScenarioRow` binds its scenario id, family id, split, archetype,
  question-template family, entity, attribute, expected value, distractor,
  scoreability, expected EOS, reference reply and exact turns.
- Entity names, scenario ids, value tuples and question-template families are
  disjoint between TRAIN, DEV and Fresh.
- Every row ends in an assistant reply whose tokens, including EOS, are marked
  in `response_mask.u8`. Every scored question has one authoritative value;
  the intentionally unscored general-dialogue rows account for the remaining
  3.2%.
- The writer decodes the masked stream and requires every assistant turn to
  appear there in full. The validator also fixes which archetypes carry a
  concrete value and which must abstain.
- Each split writes `tokens.u16`, `response_mask.u8`, `manifest.json` and
  `scenarios.jsonl`; the root writes `scenario-manifest.json`. The split and
  root manifests now carry the tokenizer path, SHA-256, CID and dialogue ids.
- `scenario-corpus check` regenerates into a temporary directory and compares
  all thirteen generated files byte for byte.

## What changed

- `crates/uor-r4-training/src/scenario_corpus.rs` adds the deterministic
  scenario model, generator, split writer, trainer-facing manifests and
  regeneration checker.
- `crates/uor-r4-training/src/bin/scenario-corpus.rs` exposes `generate`,
  `check` and `census`.
- `crates/uor-r4-training/tests/scenario_corpus.rs` pins deterministic bytes,
  holdouts, value/EOS alignment and corruption refusal.
- `Cargo.toml`, `lib.rs`, the item scope and the data-tokenizer notebook expose
  and record the unit. No model, trainer, exporter or serving behavior changed.

## Measurements

Production tokenizer:
`d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`
(vocabulary 4,096). Seed `20261111`, context 256.

| Split | Rows | Tokens | Response tokens | Scored questions |
|---|---:|---:|---:|---:|
| TRAIN | 20,000 | 1,739,599 | 373,321 | 19,375 / 20,000 (96.8%) |
| DEV | 1,024 | 99,767 | 21,941 | 992 / 1,024 (96.8%) |
| Fresh | 1,024 | 99,329 | 22,602 | 992 / 1,024 (96.8%) |

TRAIN identity:

- tokens:
  `3412e1b6ccb46db986959aa08b60490d46b3a2a3524fad476cf1fedd838259d8`
- response mask:
  `b0f745fbea8c11f7e9c2993ca03a4eb3ae5ee101b2b0ca0c23e544cac9cb92e0`
- scenarios:
  `9799855ae6aaa5dcfdff514ca68bd142b11226be449538035a3576a2902d9c43`
- trainer manifest:
  `d660f4e695363bbf121391841d17b147e8aec30071d5e59de4008b646c57ae57`

The token and mask hashes match the pre-repair corpus. The trainer manifest
changed because it gained the missing tokenizer binding.

`mix-chat-corpus` accepted the TRAIN split with one input and label `scenario`:
20,000 rows, 1,739,599 tokens, 373,321 response tokens, 0 drops and 0 special
token occurrences. Output manifest tokenizer path and SHA-256 match the split.

## Checks run at the head

- `cargo test --offline -p uor-r4-training --test scenario_corpus`:
  4 passed, 0 failed.
- `cargo test --offline -p uor-r4-training --lib`:
  123 passed, 0 failed.
- The mutation test rejects seven modified files and one missing split file.
- `scenario-corpus check` on the production corpus: `PASS`.
- `mix-chat-corpus` on TRAIN: accepted and wrote the smoke store.
- `rustfmt --check` on the three changed Rust files: pass.
- `cargo clippy --offline -p uor-r4-training --test scenario_corpus --bin
  scenario-corpus`: exit 0; no warning names either changed Rust file. The
  command reports existing warnings elsewhere in the dependency graph and
  `uor-r4-training`.
- `git diff --check`: pass.
- `uor-harness loop report --check --order .../ORDER.md`: pass.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: `PASS`. Its scope
  advisory lists the five new scope entries because the gate reads item scope
  at the merge base; the head updates those entries. Its file-map advisory is
  pre-existing and names no file in this unit.

## Skills used

`working-discipline` kept the loop documents, real mixer and held-out data in
the verification path. `solution-integrity` caught the missing trainer-facing
tokenizer binding and required the negative mutations to demonstrate refusal.

## Size budget

The generator, CLI and focused test are 1,293 changed lines before this report,
above the ORDER's 1,200-line stop line. The implementation and reporting are
separate commits so the code artifact can be reviewed independently. No bar,
item status, model, trainer, exporter or serving path changed.

## Still open

- Train a model on the corpus and measure it only through the registered
  complete-reply and fresh-phrasing instruments.
- Prove that aligned labels improve learning over the same token budget on the
  current corpus; construction alone cannot show that.
- Publish the corpus or a checksum-pinned sealed bundle if a later run needs
  it. The generated files remain ignored under `local/` in this unit.

## Plain summary

The project now has a deterministic source of small, labelled dialogues in
which the statement, wording, answer and control condition cannot drift apart.
Rust tests prove the splits are disjoint and corrupt files are refused, and
the existing trainer mixer accepts the result. This is a reliable input, not
evidence that a model can use it.
