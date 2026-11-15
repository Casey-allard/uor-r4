<!-- freshness: sources=PLAN.md,docs/roadmap/items/point-key-integer.md checked=d5eb4a55656726b04ea79b06968f91905933c925 -->
# Where UOR-R4 is today

**In one paragraph.** UOR-R4 is pre-alpha: **1 of 8 milestones is complete**, and no model holds a
useful conversation. A saved discrete geometric model passed M1's six bars on two seeds, through
its integer export. That is a real language-base result. It does not establish complete replies,
new kinds of answer, durable conversation, or a shippable product. This page gives the measured
numbers first, then the limits and what is still open.

![A horizontal bar chart on one scale from 0 to 14,287 development-panel positions. The saved M1 model scores 7,002 and 7,001, above the next-word table at 6,300 and the unigram table at 1,226.](../figures/start/status-vs-baselines.svg)

*Correct first guesses on DEV. The saved M1 model beats both plain count tables at both seeds.*

Terms are in the [glossary](./glossary.md). Claims are labelled **proof**, **measurement** or
**hypothesis**. A passing check shows code does what it says; it is not a trained capability, and
`NOT_RUN` never means passed.

## M1 is complete

The closing artifact is antigravity-110's `trunk-joint-pointer-v1-d11`, a discrete 19-table
`uor-r4.lut-discrete-trunk/1` model exported as two seeded `UORLUT01` files. It is not the separate
float-trained FBS trunk. The older close grade covers bars (a)–(e); claude-211's p-f panel score
adds bar (f), and [M1 #2029](https://github.com/UOR-Foundation/uor-r4/issues/2029) is closed.

| Measurement | Seed 1 | Seed 2 | Controls and scope |
|---|---:|---:|---|
| DEV integer top-1 | **7,002 / 14,287** | **7,001 / 14,287** | unigram 1,226; T2 6,300; bar (d) target 6,324 = T2 + 24 |
| Fresh V2 integer top-1 | **7,215 / 14,479** | **7,228 / 14,479** | unigram 1,279; T2 6,582; paired band 82 |
| Held-out-phrasing p-f | **7,850 / 15,953** | **7,872 / 15,953** | unigram 1,409; T2 7,375; paired band 81 |
| Apple M1 decode | **10,876 tok/s** | **10,566 tok/s** | bar-f remeasurement; median, 192-token prefill and 64-token decode |

The older close-grade bar (c) reports 11,026 / 10,573 tok/s; the table keeps the separate bar-f remeasurement. Both seeds have exact 14,287-row DEV export/reference agreement and pass the planted-fault B4 control.
Sources: [M1 close](../labs/m1-close-saved-model/README.md), [Fresh V2](../labs/m1-fresh-panel-transfer/README.md), and [bar-f report](../loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md).

**What bar (f) proves, and its limit.** At K = 1,024, no p-f prompt value-masked template occurs
in TRAIN. The model reads new prompt phrasings into known kinds of reply. But **62.5% of p-f's
reference replies are verbatim TRAIN replies**. That is reference-data overlap, not a measured
generated-copy rate, and not new reply kinds or useful conversation.

## What works at their measured scopes

| Part | Evidence | Scope | Source |
|---|---|---|---|
| Discrete geometric language base | M1's six bars on two saved integer artifacts | Next-token ranking on DEV, Fresh V2 and p-f, not free conversation | [M1 close](../labs/m1-close-saved-model/README.md), [Fresh V2](../labs/m1-fresh-panel-transfer/README.md), [bar-f](../loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md) |
| Integer serving (D11) | Exact 14,287-row DEV export/reference agreement and planted-fault control; integer multiply and divide are allowed, floats remain banned | The M1 pair and the separate FBS `lut-stack/5` export | [serving dossier](../mechanisms/lut-serving.md) |
| Exact-read oracle | True-address 80/80 against 0/80 no-memory and 20/80 sieve | Exact-read oracle only, not learned address formation or M3 | [exact memory](../mechanisms/exact-memory.md) |
| Object-store consumer | Consumer built; frozen 384-item host-control panel passes | On-model M3 run needs a selected M1 or M2 model; host controls are not M3 qualification | [consumer record](../labs/object-store-consumer-m3/README.md) |
| Browser worker and Studio | Synthetic parity harness and live M1 top-8 next-token prediction, with self-hosted assets and stack-bound tokenizers | Language base, not chat; no qualified bundled product or M7 accepted-model parity | [web client](../products/web-client.md) |
| Desktop shell | Native Tauri/Rust backend and durable session restore on the seed-5 synthetic artifact | No qualified trained product bundle | [desktop client](../products/desktop-client.md) |

## The separate FBS trunk: replies in conversation (since 31 Oct)

The float-trained FBS trunk is a second model path; it has not joined the saved discrete M1 model.
Since M1 closed, the work on it has been the second open question: can a model answer in a
conversation? Every number below is a measurement at its named scope.

**Human-written text.** On a held-out panel of 1,460,697 rows of public conversation and
encyclopedia text, a batch of 64 with twice the training text gets 331,698 and 342,250 next tokens
right at two seeds, against T2's 256,660: ahead at both seeds
([claude-301](../loop/tasks/one-model-end-to-end/claude-301/REPORT.md), 6 Nov). Width 256 gets
about 125,000 more rows right than width 128; width 512 did not fit the GPU's memory (claude-317,
[notebook](../research/notebook/float-training.md)).

**Recall in controlled conversations.** Since 10 Nov the panels are conversations where a value is
told and later asked for. [Claude-358](../loop/tasks/one-model-end-to-end/claude-358/REPORT.md)'s
three models (p-s1 to p-s3, 13 Nov) state the told value on **314, 320 and 325 of 578** recall
questions. One of them, p-s2, serves on an Apple M1 at 1,567 to 1,636 tok/s decode and 4,309 to 4,499 tok/s
context prefill.

**What the vectors carry alone.** [V0, claude-376](../loop/tasks/one-model-end-to-end/claude-376/REPORT.md)
(14 Nov) served those three models over 2,218 conversation rows twice: as exported, and with their
exact turn memory dropped. Exact replies are **377, 380 and 441** as exported and **171, 173 and
211** without the memory. Almost every fact the model gets right comes from the turn memory's
table, which is keyed on the turn's exact token ids and counted from TRAIN. The vector state alone
does not yet carry the facts.

**Additions tried since 31 Oct** ([mechanism board](../mechanisms/README.md#results-since-31-oct-with-no-board-row-yet)):

| Addition | Result | Served? |
|---|---|---|
| E8 address memory, a learned table read by the state's E8 cell | Exact replies of 1,108: 188 → 218 and 225 → 254 at two seeds, half a reply under its bar: not called ([claude-346](../loop/tasks/one-model-end-to-end/claude-346/REPORT.md)); not called either when served in integers ([claude-366](../loop/tasks/one-model-end-to-end/claude-366/REPORT.md)) | As a research schema only |
| Pair experts, counted from TRAIN (not trained) | Good everyday replies of 239: 137, 135, 134 → 170, 171, 170 held out ([claude-367](../loop/tasks/one-model-end-to-end/claude-367/REPORT.md)) | Opt-in in `uor-chat`; a reply served this way is a stored TRAIN reply, not the model's |
| Pair rows, trained and addressed by the turn's token pairs | Exact replies of 2,218: 431 → 527 and 413 → 521 at two seeds, mostly the turn's own words carried into the reply ([claude-370](../loop/tasks/one-model-end-to-end/claude-370/REPORT.md)); two weighings did not add (claude-373, claude-374) | No: the export refuses them |

**The native-space route (D85, 13 Nov).** The owner set the default route to one persistent
integer geometric state that training and serving share, with the same operators and bit-identical
replay, instead of float master weights and an export gap. The first Q31 vector state has trained
on CUDA in a 64-step contract test, CPU and CUDA equal after every step
([codex-5](../loop/tasks/native-space-state-contract/codex-5/REPORT.md)); that is a contract test,
not language-model training. The integer crate now restores that trained state with no numerical
conversion and runs the same shared transition: after 64 trained steps, the restored state and its
transition results match as raw words. No token generation is claimed ([codex-8](../loop/tasks/native-space-state-contract/codex-8/REPORT.md)). The discrete path's CUDA training, CPU training and integer serving now
agree on the same fixture bytes, on two fixtures ([claude-372](../loop/tasks/one-model-end-to-end/claude-372/REPORT.md)).

**In progress (14 Nov).** Carrying the state across training windows, V1
([#3979](https://github.com/UOR-Foundation/uor-r4/pull/3979)); today training carries no state
from one window to the next. Vector-native reads, V2(a)
([#3978](https://github.com/UOR-Foundation/uor-r4/pull/3978)). Neither has a result yet.

The point-key and nearest-readout units of late October (claude-217 to claude-232) are recorded
in the [float-training notebook](../research/notebook/float-training.md) and the
[open questions](../research/open-questions.md).

## What does not work yet

- **Useful conversation.** M1's next-token bars do not establish coherent, correct complete replies.
- **Facts from the vectors alone.** Without the exact turn memory, the current models answer almost
  no fact questions (V0, claude-376); the memory's table supplies them.
- **New reply kinds.** p-f has held-out prompt phrasings but 62.5% of its reference replies come
  from TRAIN. The FBS generation results make that limitation visible.
- **A joined model.** The discrete M1 path and FBS trunk remain separate. Not every FBS part is
  served in one integer session (pair rows are refused by the export), and accepted-model
  text/chat distribution remains open.
- **M2–M8.** Grounded replies, durable memory, full accepted-model serving, laptop cost,
  reasoning/coding, distribution and alpha release are not complete.

## Historical result: K16

The 19 Oct K16-A model scored 65 and 463 of 14,287, below unigram 1,226 and T2 6,300. K16-R
collapsed to 19 or 20 occupied cells. These were important negative results, but they are not the
current best model and must not be used to erase the later M1 close
([K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md)).

Four older 214M Hugging Face checkpoints predate today's rules: they used softmax and matrix
multiplication and are historical evidence, not the D11 product.

## Where it stands

| Measure | Current value | Source |
|---|---|---|
| Milestones complete | **1 of 8 (M1)** | [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029) |
| Saved model top-1 on DEV | 7,002 / 7,001 of 14,287 | [M1 close](../labs/m1-close-saved-model/README.md) |
| Fresh V2 / p-f | 7,215 / 7,228; 7,850 / 7,872 | [Fresh V2](../labs/m1-fresh-panel-transfer/README.md), [bar-f report](../loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md) |
| Useful conversation | Not established | [V0, claude-376](../loop/tasks/one-model-end-to-end/claude-376/REPORT.md) |
| Recall of a told value | 314 / 320 / 325 of 578 questions, three seeds | [claude-358](../loop/tasks/one-model-end-to-end/claude-358/REPORT.md) |
| Exact replies, with / without the turn memory | 377–441 / 171–211 of 2,218, three seeds | [V0, claude-376](../loop/tasks/one-model-end-to-end/claude-376/REPORT.md) |
| Next capability milestone | M2: grounded complete replies | [PLAN](../../PLAN.md) |
| Work in progress | One native integer state for training and serving (D85); carrying it across training windows (V1) and vector-native reads (V2(a)) | [PLAN](../../PLAN.md), [#3979](https://github.com/UOR-Foundation/uor-r4/pull/3979), [#3978](https://github.com/UOR-Foundation/uor-r4/pull/3978) |
| Products shipped | None; M1 is not an alpha release | [products](../products/README.md) |

## Go deeper

- [Where we're going](./where-were-going.md): milestones and the next route.
- [The model, end to end](../model/architecture.md) and [training](../model/training.md).
- [Open questions](../research/open-questions.md): what is unknown and what would settle it.
- Records: [M1 close](../labs/m1-close-saved-model/README.md), [bar-f](../loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md), [claude-217](../loop/tasks/one-model-end-to-end/claude-217/REPORT.md), and [K16 history](../labs/k16-outcome-call-2026-10-19/README.md).
- The one plan: [PLAN.md](../../PLAN.md) and rendered [STATE.md](../../STATE.md).
