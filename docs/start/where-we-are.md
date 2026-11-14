<!-- freshness: sources=PLAN.md,docs/roadmap/items/point-key-integer.md -->
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

## The separate FBS trunk and the current failure

The float-trained FBS trunk is a second model path; it has not joined the saved discrete M1 model.
Its `lut-stack/5` export keeps the rotations exact and has its DEV export gap inside the bar. The current work is complete replies on new
phrasings, not another replay of M1.

[Claude-217](../loop/tasks/one-model-end-to-end/claude-217/REPORT.md) found that cycle and echo
decode steps end every registered loop, but every content decision is **underpowered**, so greedy
remains the decode. On held-out p-s:

- Exact-reference replies are only **3, 1 and 4 of 256** at three seeds, against 38–40 on DEV.
- Teacher-forced reply openings are **17.4%, 13.7% and 18.6%**, against **80.8%, 80.2% and 79.9%**
  on later tokens.
- Exact turn memory keys on the turn's exact first token ids. It adds 322–337 of 1,226 openings on
  DEV but only 14–16 of 1,400 on p-s, so a reworded known question can find no stored key.

[Claude-218](../loop/tasks/one-model-end-to-end/claude-218/REPORT.md) then found trunk-state memory
keys that improved reply openings offline. [Claude-219](../loop/tasks/one-model-end-to-end/claude-219/REPORT.md) then tested them in free
generation on fresh p-v. Serving the point key for the first reply token where exact memory misses
improves template replies by **+13, +8 and +12** and exact replies by **+9, +8 and +12** at three
seeds. Keyed openings are right on 27–31% of cases, so this is a partial gain, not useful chat. The
sign key is inconclusive at this panel size. [Claude-220](../loop/tasks/one-model-end-to-end/claude-220/REPORT.md)
then served the point key inside one float model on fresh p-w: templates improve by +41/+40/+42 at
three seeds with exact claude-219 parity, while recalling TRAIN replies rather than generalizing.
[Claude-222](../loop/tasks/point-key-integer/claude-222/REPORT.md) has built the integer point table,
search and `/8` record, with 380 library tests and a full serving audit. [Claude-224](../loop/tasks/point-key-integer/claude-224/REPORT.md) has built the panel decode and grade. [Claude-227](../loop/tasks/integer-retention-rates/claude-227/REPORT.md) passed retention GPU parity within the 3% bar; [Claude-228](../loop/tasks/point-key-integer/claude-228/REPORT.md) built the run's reference/read-back/BOS instruments (390 integer-library tests with one ignored and four negative gates), which held in row 0; [Claude-229](../loop/tasks/point-key-integer/claude-229/REPORT.md) found an instrument fault: the integer session uses the native writer instead of the trained whole-table nearest readout, so no bar result or throughput is reported. [Claude-231](../loop/tasks/integer-nearest-readout/claude-231/REPORT.md) built the nearest readout and export on native and browser paths. [Claude-232](../loop/tasks/integer-nearest-readout/claude-232/REPORT.md) then missed greedy parity in every cell: DEV first-token agreement is 291, 285 and 284 of 310 at three seeds, and p-w agreement is 911, 777 and 64 of 1,202. The mismatch must be localized before the p-x rerun.

## What does not work yet

- **Useful conversation.** M1's next-token bars do not establish coherent, correct complete replies.
- **New reply kinds.** p-f has held-out prompt phrasings but 62.5% of its reference replies come
  from TRAIN. The FBS generation results make that limitation visible.
- **A joined model.** The discrete M1 path and FBS trunk remain separate. Not every FBS part is
  served in one integer session, and accepted-model text/chat distribution remains open.
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
| Useful conversation | Not established | [latest free-generation evidence](../loop/tasks/one-model-end-to-end/claude-220/REPORT.md) |
| Next capability milestone | M2: grounded complete replies | [PLAN](../../PLAN.md) |
| Immediate gate | Localize claude-232's parity MISS layer by layer, then recheck parity before the p-x rerun | [claude-232](../loop/tasks/integer-nearest-readout/claude-232/REPORT.md), [point-key item](../roadmap/items/point-key-integer.md) |
| Products shipped | None; M1 is not an alpha release | [products](../products/README.md) |

## Go deeper

- [Where we're going](./where-were-going.md): milestones and the next route.
- [The model, end to end](../model/architecture.md) and [training](../model/training.md).
- [Open questions](../research/open-questions.md): what is unknown and what would settle it.
- Records: [M1 close](../labs/m1-close-saved-model/README.md), [bar-f](../loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md), [claude-217](../loop/tasks/one-model-end-to-end/claude-217/REPORT.md), and [K16 history](../labs/k16-outcome-call-2026-10-19/README.md).
- The one plan: [PLAN.md](../../PLAN.md) and rendered [STATE.md](../../STATE.md).
