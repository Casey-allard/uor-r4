<!-- freshness: sources=CHARTER.md,PLAN.md,docs/roadmap/items/point-key-integer.md -->
# Where UOR-R4 is going

**In one paragraph.** The goal is a fast language model that runs on your own laptop or browser,
holds a conversation with durable memory, and later reasons and writes code, all through geometry
and exact memory rather than a transformer. **M1 is complete: 1 of 8 milestones.** M2 is the next capability milestone:
complete correct replies grounded in exact memory. The immediate obstacle is new-phrasing reply
opening, not the already-passed next-token count-table bars. This page is a plan, not a promise.

![The project road: M1 marked complete on the saved discrete model, while the current end-to-end line works toward grounded replies, durable memory and the remaining milestones.](../figures/start/roadmap.svg)

*M1 is complete. Current work targets new-phrasing reply opening and one useful model end to end.*

Terms are in the [glossary](./glossary.md). Labels are **proof**, **measurement** or **hypothesis**.

## The eight milestones

| | Milestone | In plain words | Status | Issue |
|---|---|---|---|---|
| M1 | Language base | A saved model predicts the next token better than count tables through its integer export | **Complete, 30 Oct 2026** | [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029) |
| M2 | Grounded reply from exact memory | Complete correct replies using facts stored earlier, on development and fresh prompts | In progress | [#2030](https://github.com/UOR-Foundation/uor-r4/issues/2030) |
| M3 | Durable conversation memory | Provenance, correction, abstention, reset and restart on the selected model | In progress | [#2031](https://github.com/UOR-Foundation/uor-r4/issues/2031) |
| M4 | One served model | The accepted model through D11 and the command line, with held-out quality at or below **0.90 BPB**, export-gap and session bars | Not complete; accepted model path not started | [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032) |
| M5 | Laptop cost | Energy, memory, latency and selected parameter access on an M1-class laptop | Not started | [#2033](https://github.com/UOR-Foundation/uor-r4/issues/2033) |
| M6 | Reasoning and coding | Unseen multi-step and executable coding tasks on the same model, with causal ablation | Not started | [#2034](https://github.com/UOR-Foundation/uor-r4/issues/2034) |
| M7 | API, browser and Studio | Bit-identical tokens and sessions across library, CLI and browser, with truthful local-by-default Studio, with optional user-configured web API | Runtime/harness on synthetic stand-in; accepted model open | [#2035](https://github.com/UOR-Foundation/uor-r4/issues/2035) |
| M8 | Alpha release | One versioned, checksum-pinned model meeting M1–M7, independently verified | Not started | [#2036](https://github.com/UOR-Foundation/uor-r4/issues/2036) |

A milestone closes only when its full bar is met on a saved model. M1's close does not imply M2 or
useful conversation.

## What M1 established

The saved discrete model passes six bars at both seeds: a self-contained D11 artifact; exact
14,287-row DEV export/reference agreement and a planted-fault control; tokens per second; DEV integer top-1 above
unigram and T2; Fresh V2 transfer; and held-out-phrasing transfer.

- DEV: **7,002 / 7,001** of 14,287 against unigram 1,226 and T2 6,300.
- Fresh V2: **7,215 / 7,228** of 14,479 against unigram 1,279 and T2 6,582.
- p-f: **7,850 / 7,872** of 15,953 against unigram 1,409 and T2 7,375.
- Apple M1 decode: **10,876 / 10,566 tok/s**, measured with a 192-token prefill and 64-token decode.

The limitation is equally important: 62.5% of p-f's reference replies are verbatim TRAIN replies.
M1 reads new prompt phrasings into known reply kinds; it does not show new reply kinds or useful
conversation. See [CHARTER §4.8](../../CHARTER.md#4-evidence), the [M1 close](../labs/m1-close-saved-model/README.md), [Fresh V2](../labs/m1-fresh-panel-transfer/README.md), and the [bar-f report](../loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md).

## The current order of work

1. **Localize the parity MISS, then complete `integer-nearest-readout` and rerun `point-key-integer`.** Claude-218 found trunk-state keys that
   improved openings offline; [claude-219](../loop/tasks/one-model-end-to-end/claude-219/REPORT.md)
   showed a partial free-generation gain on fresh p-v, and [claude-220](../loop/tasks/one-model-end-to-end/claude-220/REPORT.md)
   then served the point key inside one float model on fresh p-w: templates improve by +41/+40/+42 at three seeds with exact claude-219 parity, while recalling TRAIN replies rather than generalizing. [Claude-222](../loop/tasks/point-key-integer/claude-222/REPORT.md) has built the integer point table, search and `/8` record, with 380 library tests and a full serving audit. [Claude-224](../loop/tasks/point-key-integer/claude-224/REPORT.md) has built the panel decode and grade; [claude-227](../loop/tasks/integer-retention-rates/claude-227/REPORT.md) passed retention GPU parity within the 3% bar; [Claude-228](../loop/tasks/point-key-integer/claude-228/REPORT.md) built the run's reference/read-back/BOS instruments (390 integer-library tests with one ignored and four negative gates), which held in row 0; [Claude-229](../loop/tasks/point-key-integer/claude-229/REPORT.md) found an instrument fault: the integer session uses the native writer instead of the trained whole-table nearest readout, so no bar result or throughput is reported. [Claude-231](../loop/tasks/integer-nearest-readout/claude-231/REPORT.md) built the nearest readout and export on native and browser paths; [claude-232](../loop/tasks/integer-nearest-readout/claude-232/REPORT.md) then missed greedy parity in every cell, so the mismatch must be localized before the p-x rerun ([item](../roadmap/items/point-key-integer.md)).
2. **Serve every required part in integers.** Finish the served read, memory and readout rather
   than treating an export or an audit alone as the whole model.
3. **Train and score one model end to end.** Measure complete generated replies on untouched
   phrasings with tokens per second, then export and serve that same model.
4. **Qualify M2 and M3, then M4.** Complete correct replies and durable memory precede the accepted
   model's full serving contract. After that come laptop cost, reasoning/coding, distribution and
   the alpha release.

The preceding claude-217 result explains why this order matters. Its cycle step ends every
registered loop, but its content decision is underpowered and greedy remains the decode. On p-s,
teacher-forced reply openings are 17.4%, 13.7% and 18.6% at three seeds, against about 80% on later tokens.
Stopping a loop does not repair a reply that began wrong
([report](../loop/tasks/one-model-end-to-end/claude-217/REPORT.md)).

## What comes after the first useful reply

- **M3:** qualify durable memory: provenance, correction, abstention, reset and restart, with
  token-identical restored sessions.
- **M4:** select the accepted model and qualify the full `UORLUT01` export, CLI, quality, session
  and audit path. A lawful likelihood (`PRLL`) exists, but GPU held-out scoring remains open.
- **M5:** measure real laptop cost rather than infer efficiency from arithmetic counts.
- **M6:** add reasoning and coding on the same model with executable checks and causal ablation.
- **M7:** prove library/CLI/browser parity on the accepted model and ship a truthful local-by-default Studio, with optional user-configured web API.
- **M8:** cut one versioned, checksum-pinned release and verify it independently on consumer hardware.

## The three products at the end

| Product | What it will be | What exists today |
|---|---|---|
| [The model](../products/language-model.md) | One integer `UORLUT01` model for local serving | The M1 pair and a separate FBS export, but no conversational product release |
| [Web client](../products/web-client.md) | The Studio running that file locally in WebAssembly | Synthetic parity harness plus public M1 top-8 next-token prediction; no chat or qualified bundle |
| [Desktop client](../products/desktop-client.md) | A Tauri app sharing the interface with durable memory and offline use | Native shell and durable session restore verified on a synthetic artifact; no qualified bundle |

Nothing ships until a trained model passes the remaining product bars. Serving an exported D11
artifact on the user's device is the product and is allowed. GPU remains the qualification
authority; only the owner-authorized D84 CPU program runs development research locally
([charter](../../CHARTER.md), sections 3.5-3.6).

## The biggest unknown

Can the model start a correct reply when the user's phrasing is new, rather than only retrieve or
continue a familiar template? M1 answered the narrower next-token count-table question. It did not
answer that broader generation question. Claude-220 serves the point key inside one float model
on fresh p-w: templates improve by +41/+40/+42 at three seeds with exact claude-219 parity, while
recalling TRAIN replies rather than generalizing. The integer table, client-sized search and
whole-model score are still unproven; useful conversation has not been established.

## Where it stands

| Item | State | Source |
|---|---|---|
| Milestones complete | **1 of 8 (M1)** | [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029) |
| Next milestone | M2: grounded complete replies | [PLAN](../../PLAN.md) |
| Immediate problem | Localize claude-232's parity MISS, then recheck parity before rerunning p-x | [claude-232](../loop/tasks/integer-nearest-readout/claude-232/REPORT.md), [item](../roadmap/items/point-key-integer.md) |
| M4 | Lawful likelihood built; full accepted-model serving and held-out score open | [serving and export](../model/serving-and-export.md) |
| Products shipped | None | [products](../products/README.md) |

## Go deeper

- [Where we are](./where-we-are.md): the measured current state and its limits.
- [Transformer vs geometry](./transformer-vs-geometry.md): replacements and what each has shown.
- [PLAN](../../PLAN.md) and rendered [STATE.md](../../STATE.md): the live plan and generated status.
- [Products](../products/README.md) and [releases](../products/releases-and-artifacts.md).
- [Charter](../../CHARTER.md): the rules every stage must follow.
