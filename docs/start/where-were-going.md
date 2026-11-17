<!-- freshness: sources=CHARTER.md,PLAN.md,docs/roadmap/items/point-key-integer.md checked=d5eb4a55656726b04ea79b06968f91905933c925 -->
# Where UOR-R4 is going

**In one paragraph.** The goal is a fast language model that runs on your own laptop or browser,
holds a conversation with durable memory, and later reasons and writes code, all through geometry
and exact memory rather than a transformer. **M1 is complete: 1 of 8 milestones.** M2 is the next capability milestone:
complete correct replies grounded in exact memory. The immediate obstacle is that the current
models' facts come from an exact table rather than their learned state; since 13 Nov (D85) the
route is one native integer state shared by training and serving. This page is a plan, not a
promise.

![The project road: M1 marked complete on the saved discrete model, while the current end-to-end line works toward grounded replies, durable memory and the remaining milestones.](../figures/start/roadmap.svg)

*M1 is complete. Current work targets replies that come from the model's own state, and one useful model end to end.*

Terms are in the [glossary](./glossary.md). Labels are **proof**, **measurement** or **hypothesis**.

## The current experiment

The 16 November research decision starts with a count model on D87, a corpus for which training, tuning and test streams are separately sealed. The tuning sweep selected order 10. The frozen reference has now been scored on DEV-test. The remaining comparison measures what geometric routing and memory add at equal bytes. The proposed scale is one billion cells in at most 4 GB and at least 1,000 decoded tokens/s. The experiment also calls for grounded replies to beat both the no-memory and shuffled-speaker controls. These are targets. [Decision and thresholds](../loop/tasks/written-state-count-model/claude-409/DECISION.md) · [Split construction](../loop/tasks/written-state-count-model/claude-420/REPORT.md) · [Plan](../../PLAN.md)

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

## The native training and serving contract

Since 13 Nov the route is D85: one persistent native geometric state, in one declared integer or
fixed-point representation, that training and serving share, with the same typed operators and
bit-identical replay. Float master weights and a float-to-integer export gap become reference work.
[PLAN](../../PLAN.md#native-space-cycle) gives the four steps:

1. **Declare the state and its operators** (`native-space-state-contract`): every carried field,
   each transition's shared training and serving operator, and a bit-identical replay fixture.
   The first Q31 vector state has trained on CUDA in a 64-step contract test, with CPU and CUDA
   equal after every step ([codex-5](../loop/tasks/native-space-state-contract/codex-5/REPORT.md)).
2. **Keep gradients and optimizer updates in that state**, not in a parallel float master.
3. **Native CUDA training kernels** with the same transition identities the integer runtime serves.
4. **One trained end-to-end model,** serialized losslessly and served with no numeric conversion,
   then scored on complete replies with tokens per second.

On the current FBS models the measured gap is clear. Served without their exact turn memory, the
models give 171 to 211 exact replies of 2,218 against 377 to 441 with it, and almost every fact
comes from the memory's table ([V0, claude-376](../loop/tasks/one-model-end-to-end/claude-376/REPORT.md)).
Cross-window state and three optional vector reads were implemented in November. In the two-seed test Flock-g learned which fact to select, while its best held-out answer accuracy was 0.023 against the 0.90 bar. [Cross-window report](../loop/tasks/one-model-end-to-end/claude-380/REPORT.md) · [Read comparison](../loop/tasks/one-model-end-to-end/claude-386/REPORT.md). M2 and M3, then M4, follow one
useful model; after that come laptop cost, reasoning and coding, distribution and the alpha release.

The late-October point-key and nearest-readout units (claude-217 to claude-232) are recorded in
the [open questions](../research/open-questions.md) and the
[float-training notebook](../research/notebook/float-training.md).

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

Can the model's own learned state carry what a reply needs, the told values and the facts, rather
than an exact table keyed on the turn's token ids? M1 answered the narrower next-token
count-table question. On recall in controlled conversations, claude-358's models state a told
value on 314, 320 and 325 of 578 questions at three seeds, but V0 shows the facts they get right
come almost entirely from the turn memory's table. Useful conversation has not been established.

## Where it stands

| Item | State | Source |
|---|---|---|
| Milestones complete | **1 of 8 (M1)** | [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029) |
| Next milestone | M2: grounded complete replies | [PLAN](../../PLAN.md) |
| Immediate problem | Facts come from the exact turn memory's table, not the learned state: 171–211 exact replies of 2,218 without it, 377–441 with it | [V0, claude-376](../loop/tasks/one-model-end-to-end/claude-376/REPORT.md) |
| Work in progress | The selected order-10 reference measured 1.418136 bits per byte overall and 1.743746 on novel targets on the sealed DEV-test. The geometric model comparison remains open | [Phase order](../../PLAN.md#approved-sequences), [P0 calibration](../loop/tasks/written-state-count-model/claude-421/REPORT.md), [sealed reference](../loop/tasks/written-state-count-model/claude-422/REPORT.md) |
| M4 | Lawful likelihood built; full accepted-model serving and held-out score open | [serving and export](../model/serving-and-export.md) |
| Products shipped | None | [products](../products/README.md) |

## Go deeper

- [Where we are](./where-we-are.md): the measured current state and its limits.
- [Transformer vs geometry](./transformer-vs-geometry.md): replacements and what each has shown.
- [PLAN](../../PLAN.md) and rendered [STATE.md](../../STATE.md): the live plan and generated status.
- [Products](../products/README.md) and [releases](../products/releases-and-artifacts.md).
- [Charter](../../CHARTER.md): the rules every stage must follow.
