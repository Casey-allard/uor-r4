<!-- freshness: sources=crates/uor-r4-discrete,crates/uor-r4-integer,crates/uor-r4-integer/src/stack/format.rs checked=abfad060460503bbd468f5b8c74b892b2aaca800 -->
# The language model

The core product is a language model: a program that reads text and predicts what comes next, so it
can eventually hold a conversation. UOR-R4 tries to do that with geometry and exact memory rather
than a transformer's dense arithmetic. It is meant to ship as one `UORLUT01` integer file that an
ordinary laptop or browser can run without floating-point arithmetic. **M1 is complete, but that
product does not ship yet.** No model holds a useful conversation and no alpha release exists.

Labels: **proof** (derived from definitions or code), **measurement** (a recorded run), and
**hypothesis** (untested). See [`docs/formal_vocabulary.md`](../formal_vocabulary.md) and the
[glossary](../start/glossary.md).

```mermaid
flowchart LR
  T["Train one native integer state on GPU"] --> X["Serialize the trained state losslessly"]
  X --> A["Verify raw-word replay and audit integer operations"]
  A --> S["Serve on the device<br/>uor-r4-integer"]
  S --> C1["Command line"]
  S --> C2["WebAssembly"]
  S --> C3["Native desktop"]
```

*D85 design. A device must reproduce the training path’s route, carry, read, memory-address and token decisions from the same prompt and starting state.*

## What the model is meant to be

**A geometric autoregressive state model with exact addressed memory.** Prime addresses, zeta
phases, E8/H4/icosian rotations and SpiralCore/FBS routing replace transformer machinery. Exact
identity remains separate from geometric address. Conversation with durable memory comes first,
then coding and reasoning, on one model.

**Fast and local.** The lead metric is tokens per second on ordinary hardware, with context and
decode reported separately. A small quality gap is acceptable if the model is much cheaper.

**Integer at serving.** D11 bans floating point and transcendental arithmetic and excludes dense
matrix multiplication. Since D80 and D81 it allows the hardware integer multiply and divide. D85 requires the shared integer or fixed-point representation throughout training and serving. Floating point serves as a reference or diagnostic outside that path. [Charter §3.7](../../CHARTER.md#3-technical-constraints)

## The current model experiment

The written-state experiment asks how much geometric routing and memory improve on a count model trained on the same text at equal bytes. On D87 DEV-tune, the order-2 modified Kneser–Ney scorer measured **2.004736 bits per byte overall** and **2.194312 on novel targets**. The frozen reference still needs its DEV-test score. The proposed model must then improve on that reference and pass the grounded-reply controls with memory removed or speakers shuffled. [Written-state decision](../loop/tasks/written-state-count-model/claude-409/DECISION.md) · [Scorer measurement](../loop/tasks/written-state-count-model/claude-421/REPORT.md)

## What exists now

The M1 closing artifact is antigravity-110's discrete `trunk-joint-pointer-v1-d11`, a 19-table
`uor-r4.lut-discrete-trunk/1` model saved as two seeded `UORLUT01` files. It is not the separate
float-trained FBS `lut-stack/5` trunk.

| Test | Seed 1 | Seed 2 | Controls |
|---|---:|---:|---|
| DEV integer top-1 | **7,002 / 14,287** | **7,001 / 14,287** | unigram 1,226; T2 6,300 |
| Fresh V2 integer top-1 | **7,215 / 14,479** | **7,228 / 14,479** | unigram 1,279; T2 6,582 |
| p-f held-out phrasing | **7,850 / 15,953** | **7,872 / 15,953** | unigram 1,409; T2 7,375; band 81 |
| Apple M1 decode | **10,876 tok/s** | **10,566 tok/s** | median; 192-token prefill, 64-token decode |

The speed row is the bar-f remeasurement; the older close-grade bar (c) reports 11,026 / 10,573 tok/s. Both seeds have exact 14,287-row DEV export/reference agreement and pass B4. These results close M1
([#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029)), with the older close grade for
bars (a)–(e) and claude-211's panel score for bar (f) ([M1 close](../labs/m1-close-saved-model/README.md), [Fresh V2](../labs/m1-fresh-panel-transfer/README.md), [bar-f](../loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md)).

**The limitation is part of the result:** 62.5% of p-f's reference replies are verbatim TRAIN
replies. M1 shows reading new prompt phrasings into known reply kinds, not new reply kinds or useful
conversation.

## Two model paths, not one joined model

- **The discrete path** produced the M1 artifact. It trains integer geometric state and exports
  through table-native `UORLUT01` schemas.
- **The float-trained FBS trunk** supplied the October complete-reply experiments. Its `lut-stack/5` export preserved exact rotations and met the export-gap bar on DEV.

Claude-217's cycle-step experiment ends every registered loop but leaves content underpowered, so
greedy remains the decode. Exact turn memory keys on exact first token ids and misses reworded
prompts ([claude-217](../loop/tasks/one-model-end-to-end/claude-217/REPORT.md)). Claude-218 found
trunk-state keys offline; [claude-219](../loop/tasks/one-model-end-to-end/claude-219/REPORT.md)
then served the point key for the first reply token in free generation on fresh p-v. It improves
templates by +13/+8/+12 and exact replies by +9/+8/+12 at three seeds, but reaches only 12–18% of
the forced-opening lift. [Claude-220](../loop/tasks/one-model-end-to-end/claude-220/REPORT.md) then served the point key inside one float model on fresh p-w: templates improve by +41/+40/+42 at three seeds with exact claude-219 parity, while recalling TRAIN replies rather than generalizing. [Claude-222](../loop/tasks/point-key-integer/claude-222/REPORT.md) has built the integer point table, search and `/8` record, with 380 library tests and a full serving audit. [Claude-224](../loop/tasks/point-key-integer/claude-224/REPORT.md) has built the panel decode and grade. [Claude-227](../loop/tasks/integer-retention-rates/claude-227/REPORT.md) passed retention GPU parity within the 3% bar; [Claude-228](../loop/tasks/point-key-integer/claude-228/REPORT.md) built the run's reference/read-back/BOS checks; [Claude-229](../loop/tasks/point-key-integer/claude-229/REPORT.md) found an instrument fault: the integer session uses the native writer instead of the trained whole-table nearest readout, so no bar result or throughput is reported. [Claude-231](../loop/tasks/integer-nearest-readout/claude-231/REPORT.md) built the nearest readout and export on native and browser paths; [claude-232](../loop/tasks/integer-nearest-readout/claude-232/REPORT.md) then missed greedy parity in every cell.

The November FBS experiments measured recall in controlled conversations. [Claude-358](../loop/tasks/one-model-end-to-end/claude-358/REPORT.md)'s three `p` models state a value told earlier on 314, 320 and 325 of 578 recall questions. The pure-vector baseline, [claude-376](../loop/tasks/one-model-end-to-end/claude-376/REPORT.md), served them over 2,218 rows: 377, 380 and 441 exact replies as exported, 171, 173 and 211 with the exact turn memory dropped. Almost every fact the model gets right comes from the turn memory's table. A later run carried state across training windows. In the vector-read comparison, Flock-g learned to select the requested fact, but its best held-out answer accuracy was 0.023 against the 0.90 bar. [Cross-window result](../loop/tasks/one-model-end-to-end/claude-380/REPORT.md) · [Read comparison](../loop/tasks/one-model-end-to-end/claude-386/REPORT.md)

## The artifact format and serving boundary

`UORLUT01` begins with an 8-byte magic string and a JSON header, then aligned integer data sections.
The reader checks offsets and lengths and computes the file's SHA-256 as it loads, so a release can
pin that identity ([`format.rs`](../../crates/uor-r4-integer/src/stack/format.rs)). Table-native
schemas include `lut-discrete/1` and `lut-discrete-trunk/1`. The separate FBS `lut-stack/5` is a
4-bit float-to-integer stack with exact rotations, not a table-native discrete schema; older dense
`lut-stack/1`, `/2` and `/3` schemas are not a description of today's M1 artifact.

The serving engine is [`crates/uor-r4-integer`](../../crates/uor-r4-integer/). It serves native and
WebAssembly artifacts, with save/restore formats such as `UR4SESS1`. An engine audit proves the
absence of refused instructions, not model quality. M4 still requires the accepted model's full
quality, CLI, session and audit path; M7's distribution check is separate.

## The eight milestones in plain words

| # | Meaning | Current state |
|---|---|---|
| M1 | **It predicts text.** Six bars through the saved integer model, including fresh and held-out-phrasing transfer | **Complete** |
| M2 | **It answers from memory.** On the same model, ≥256/512 complete, correct saved replies and ≥52/128 fresh opposite-role pairs with both replies complete and distinct | In progress |
| M3 | **It remembers across sessions.** Provenance, correction, abstention, reset and restart | In progress |
| M4 | **It is served.** Accepted model with lossless native serialization and raw-word parity; held-out quality **≤0.90 BPB**; M1–M3 bars through `uor-chat`; bit-identical continuation after restore; full CLI/session/audit path | Qualification requires the selected M3 model and its measured [M4 artifact](../roadmap/items/m4-export-uorlut01-cli.md) |
| M5 | **It is cheap on a laptop.** Latency, memory, energy and selected access measured | [Protocol registered](../roadmap/items/m5-laptop-full-path-measurement.md), accepted-model measurements open |
| M6 | **It reasons and codes.** Frozen tasks with executable oracles and causal ablation | [Reference verifier](../roadmap/items/m6-qualify-reasoning-coding.md) checked 80 programs on 320 vectors, model qualification open |
| M7 | **It runs everywhere the same.** Library, CLI and browser parity plus a local-by-default Studio with optional user-configured web API | Runtime/harness on synthetic stand-in; accepted model open |
| M8 | **It is released.** One versioned, checksum-pinned artifact meeting M1–M7 | [Packaging and independent reproduction](../roadmap/items/m8-release-packaging.md) depend on the accepted artifact |

M4's quality work includes a lawful likelihood (`PRLL`), but GPU held-out scoring remains open. M1
does not select the M2/M3/M4 model automatically, and an engine audit alone closes no milestone.

## Historical models and open questions

The four 214M-parameter checkpoints on Hugging Face predate today's rules and used softmax and
matrix multiplication. They are historical evidence, not the D11 product. K16's 65 and 463 of
14,287 were also important negative results, but they predate the M1 close.

Open questions now centre on complete generation rather than M1:

- Can exact memory key a reworded known question so a new phrasing starts the right reply?
- Can one joined model produce complete correct replies and durable memory, then pass M4's full
  serving path?
- What are the full-path latency, memory traffic and energy per token? M5 has not measured them.
- Can library, CLI and browser produce bit-identical tokens and sessions on the accepted model?

## Go deeper

- [Model architecture](../model/architecture.md), [training](../model/training.md), and [serving and export](../model/serving-and-export.md).
- [Releases and artifacts](./releases-and-artifacts.md) for format, checksums and the future M8 release.
- [M1 close](../labs/m1-close-saved-model/README.md) and [bar-f report](../loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md).
- Code: [`crates/uor-r4-discrete/`](../../crates/uor-r4-discrete/) and [`crates/uor-r4-integer/`](../../crates/uor-r4-integer/).
- The one [PLAN](../../PLAN.md) and the long-form [project track](../integration/project-track.md).
