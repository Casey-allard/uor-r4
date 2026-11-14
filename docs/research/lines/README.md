<!-- freshness: sources=PLAN.md -->
# The research lines

A research line is one sustained question the project worked on, such as "how does the discrete model read earlier text". The 19 October audit sorted all 1,173 recorded pieces of work into 20 lines. This page lists them, shows when each ran, and says in a word where each stands. Seventeen lines have at least three records and get their own page. None has produced a model that holds a useful conversation: the project is pre-alpha and **1 of its 8 milestones is complete (M1)** ([PLAN](../../../PLAN.md), [STATE](../../../STATE.md)).

![A timeline from July to October 2026 of the 12 largest of the 20 research lines, with one dot per dated record; the 8 smaller lines are not shown](../../figures/research-lines-timeline.svg)

*When each line ran. The September lines end as the October discrete line begins; the tooling and operations lines run alongside.*

## How to read the table

- **Records** are catalog entries. They mix lab records, lane results, reviews and plans, so they show where effort went and not how many experiments ran. Source: `archive/catalog.jsonl`, counted with `jq` on the field `line`.
- **Results** are the records whose outcome is positive, negative, inconclusive or diagnostic-stop. The rest are plans, tooling, process, review, operations and blocked runs. A *blocked* record is a run that could not start; it is not a negative.
- **Span** is the first to last dated record in the catalog in 2026, not a current-status cutoff. Some records carry no date and are left out of the span; newer results appear after the table.
- A positive record reports that the target the record itself set was met, at the scope it states. Many are narrow, scored on a panel the authors wrote, and none establishes useful conversation.
- **State** follows the project's vocabulary. *Active* means on the current model path. *Parked* means the tested forms failed or broke a rule and the mechanism stays open (decision D22). *Closed* means the line stopped (a line label only: its mechanisms keep their board status); a negative closes a configuration and never a mechanism. *Preserved* is evidence kept inactive.

## All lines

| Line | Span | Records | Results | State |
|---|---|---:|---:|---|
| [`native-geometric-sept`](native-geometric-sept.md): the September native geometric model | 31 Jul to 28 Sep | 294 | 224 | Closed; whole model parked |
| [`m1-discrete-attention`](m1-discrete-attention.md): discrete attention for the language base | 7 Aug to 19 Oct | 199 | 73 | M1 complete, 30 Oct |
| [`harness-governance`](harness-governance.md): research tooling and rules | 24 Jul to 19 Oct | 185 | 0 | Active support, slimmer since 19 Oct |
| [`m2-exact-memory`](m2-exact-memory.md): grounded replies from exact memory | 3 Sep to 12 Oct | 151 | 93 | In progress; the earlier fresh test failed (0 of 128), and D67 removed the blanket hold |
| [`geometric-stack-pretraining`](geometric-stack-pretraining.md): the geometric stack | 20 Sep to 11 Oct | 84 | 50 | Parked as a whole model |
| [`r4g1-transformerless`](r4g1-transformerless.md): compiling a teacher into tables | 21 Jul to 27 Sep | 74 | 40 | Parked |
| [`m4-serving`](m4-serving.md): integer serving | 7 Aug to 18 Oct | 42 | 23 | Engine active; M1 and FBS exports served, accepted M4 path open |
| [`m1-float-rung0`](m1-float-rung0.md): the float route to rung 0 | 31 Aug to 13 Oct | 38 | 18 | Historical line; Hamilton/FBS trained, recurrence READY with fit/export measured, M1 complete on the discrete path |
| [`m3-durable-memory`](m3-durable-memory.md): durable memory and the object store | 3 Oct to 18 Oct | 33 | 3 | Store ready; consumer built; on-model M3 run needs a selected M1 or M2 model |
| [`compute-ops`](compute-ops.md): GPU and pod operations | 3 Sep to 18 Oct | 15 | 0 | Active support |
| [`eval-panels`](eval-panels.md): evaluation panels and graders | 7 Aug to 14 Oct | 14 | 5 | Panels frozen; reply headline suspended |
| [`geometric-intelligence-aug`](geometric-intelligence-aug.md): the August geometric programme | 25 to 30 Aug | 9 | 7 | Closed |
| [`data-tokenizer`](data-tokenizer.md): data and tokenizer | 16 Aug to 19 Oct | 8 | 1 | Active tokenizer; 16,384-piece trainer and donor corpus prepared, not run |
| [`m7-distribution`](m7-distribution.md): API, browser runtime and Studio | 3 Sep to 15 Oct | 8 | 0 | Synthetic parity harness and public M1 top-8 prediction in the Studio; accepted parity open |
| [`spiralcore-wire`](spiralcore-wire.md): SpiralCore wire transport | 18 Oct | 7 | 0 | Side exploration, off the model path |
| [`m5-energy`](m5-energy.md): laptop energy and cost | 25 Sep to 18 Oct | 4 | 1 | Not started; no joules per token measured |
| [`m1-prime-zeta`](m1-prime-zeta.md): the prime-zeta codebook | 14 Oct | 3 | 2 | Active as baseline start |
| `vsa-native-prose` | 9 Oct | 2 | 1 | No page (fewer than three records) |
| `m6-reasoning-coding` | 22 Aug | 2 | 1 | No page (fewer than three records) |
| `e8-orbit-memory` | 11 Oct | 1 | 1 | No page (fewer than three records) |
| **Total** | 21 Jul to 19 Oct | **1,173** | **543** | |

The three short lines are listed in the [generated archive catalog](../../../archive/CATALOG.md) with all their records. The `vsa-native-prose` records are in [`archive/labs/vsa-native-test-2026-10-09/README.md`](../../../archive/labs/vsa-native-test-2026-10-09/README.md), the E8 record is in [`archive/labs/e8-orbit-memory-2026-10-11/README.md`](../../../archive/labs/e8-orbit-memory-2026-10-11/README.md), and the `m6-reasoning-coding` record is [`archive/w33_geometry_qualification_845.md`](../../../archive/w33_geometry_qualification_845.md). The VSA and E8 orbit memory mechanisms have pages on [the mechanism board](../../mechanisms/README.md).

## What the whole table shows

- **Two lines hold 42% of the records.** `native-geometric-sept` (294) and `m1-discrete-attention` (199) together hold 493 of 1,173 (arithmetic from the table).
- **The "Results" column counts every recorded result, including ones scored on authored or exposed panels.** Read it with [what the project knows](../findings.md): development is not fresh, and a pass on a built panel is not a language result.
- **Tooling and operations records are large.** `harness-governance`, `compute-ops` and `spiralcore-wire` hold 207 records and no model result. Part of that came from owner decisions that forced a rebuild in October (`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md`, section 3).
- **`m1-discrete-attention` has now closed at M1 scope.** The saved model scores 7,002 and 7,001 of 14,287 on DEV, above T2's 6,300 and unigram's 1,226. K16's older 65 and 463 are history, not the current best.

## Since 27 Oct

**M1 closed 30 Oct; current research through 31 Oct 2026.** The table above stops at 19 Oct. The discrete path has since closed M1, while the float-trained FBS trunk carries current complete-reply research. See [the pipeline map](../../model/pipeline-map.md) for both paths and [the M1 research line](./m1-discrete-attention.md) for the close. The project is pre-alpha: 1 of 8 milestones is complete. Claude-220 serves the point key inside one float model on fresh p-w, improving templates by +41/+40/+42 at three seeds with exact claude-219 parity; the gain recalls TRAIN replies, not new wording. [Claude-222](../../loop/tasks/point-key-integer/claude-222/REPORT.md) has built the integer point table, search and `/8` record, with 380 library tests and a full serving audit. [Claude-224](../../loop/tasks/point-key-integer/claude-224/REPORT.md) has built the panel decode and grade; [claude-227](../../loop/tasks/integer-retention-rates/claude-227/REPORT.md) passed retention GPU parity within the 3% bar; [claude-228](../../loop/tasks/point-key-integer/claude-228/REPORT.md) built the run's reference/read-back/BOS checks; [claude-229](../../loop/tasks/point-key-integer/claude-229/REPORT.md) found an instrument fault: the integer session uses the native writer instead of the trained whole-table nearest readout, so no bar result or throughput is reported. [Claude-231](../../loop/tasks/integer-nearest-readout/claude-231/REPORT.md) built the nearest readout and export on native and browser paths; [claude-232](../../loop/tasks/integer-nearest-readout/claude-232/REPORT.md) then missed greedy parity in every cell, so the mismatch must be localized before the p-x rerun.

## Where to go next

- [How research is done now](../README.md), the rules for notebooks, sheets and releases.
- [What the project knows](../findings.md), [what is still open](../open-questions.md) and [how the project got here](../history.md).
- [Mechanism board](../../mechanisms/README.md): the 28 mechanisms with ACTIVE, READY or PARKED status.
- [Archive catalog](../../../archive/CATALOG.md): every dated record, with one generated page per line under `archive/catalog/`.
