# The research lines

A research line is one sustained question the project worked on, such as "how does the discrete model read earlier text". The audit sorted all 1,173 recorded pieces of work into 20 lines. This page lists them, shows when each ran, and says in a word where each stands. Seventeen lines have at least three records and get their own page. None of them has produced a model that holds a useful conversation: the project is pre-alpha and 0 of its 8 milestones are complete (`docs/integration/NOW.md`).

![A timeline from July to October 2026 of the 12 largest of the 20 research lines, with one dot per dated record; the 8 smaller lines are not shown](../../figures/research-lines-timeline.svg)

*When each line ran. The September lines end as the October discrete line begins; the tooling and operations lines run alongside.*

## How to read the table

- **Records** are catalog entries. They mix lab records, lane results, reviews and plans, so they show where effort went and not how many experiments ran. Source: `archive/catalog.jsonl`, counted with `jq` on the field `line`.
- **Results** are the records whose outcome is positive, negative, inconclusive or diagnostic-stop. The rest are plans, tooling, process, review, operations and blocked runs. A *blocked* record is a run that could not start; it is not a negative.
- **Span** is the first to last dated record in 2026. Some records carry no date and are left out of the span.
- A positive record reports that the target the record itself set was met, at the scope it states. Many are narrow, scored on a panel the authors wrote, and none is a language result.
- **State** follows the project's vocabulary. *Active* means on the current model path. *Parked* means the tested forms failed or broke a rule and the mechanism stays open (decision D22). *Closed* means the line stopped (a line label only: its mechanisms keep their board status); a negative closes a configuration and never a mechanism. *Preserved* is evidence kept inactive.

## All lines

| Line | Span | Records | Results | State |
|---|---|---:|---:|---|
| [`native-geometric-sept`](native-geometric-sept.md): the September native geometric model | 31 Jul to 28 Sep | 294 | 224 | Closed; whole model parked |
| [`m1-discrete-attention`](m1-discrete-attention.md): discrete attention for the language base | 7 Aug to 19 Oct | 199 | 73 | Active; M1 not met |
| [`harness-governance`](harness-governance.md): research tooling and rules | 24 Jul to 19 Oct | 185 | 0 | Active support, slimmer since 19 Oct |
| [`m2-exact-memory`](m2-exact-memory.md): grounded replies from exact memory | 3 Sep to 12 Oct | 151 | 93 | Held; fresh test failed (0 of 128) |
| [`geometric-stack-pretraining`](geometric-stack-pretraining.md): the geometric stack | 20 Sep to 11 Oct | 84 | 50 | Parked as a whole model |
| [`r4g1-transformerless`](r4g1-transformerless.md): compiling a teacher into tables | 21 Jul to 27 Sep | 74 | 40 | Parked |
| [`m4-serving`](m4-serving.md): integer serving | 7 Aug to 18 Oct | 42 | 23 | Engine active; no lawful model exported |
| [`m1-float-rung0`](m1-float-rung0.md): the float route to rung 0 | 31 Aug to 13 Oct | 38 | 18 | Preserved; parts ready, never trained |
| [`m3-durable-memory`](m3-durable-memory.md): durable memory and the object store | 3 Oct to 18 Oct | 33 | 3 | Store ready; nothing consumes it |
| [`compute-ops`](compute-ops.md): GPU and pod operations | 3 Sep to 18 Oct | 15 | 0 | Active support |
| [`eval-panels`](eval-panels.md): evaluation panels and graders | 7 Aug to 14 Oct | 14 | 5 | Panels frozen; reply headline suspended |
| [`geometric-intelligence-aug`](geometric-intelligence-aug.md): the August geometric programme | 25 to 30 Aug | 9 | 7 | Closed |
| [`data-tokenizer`](data-tokenizer.md): data and tokenizer | 16 Aug to 19 Oct | 8 | 1 | Active tokenizer; 16,384-piece trainer and donor corpus prepared, not run |
| [`m7-distribution`](m7-distribution.md): API, browser runtime and Studio | 3 Sep to 15 Oct | 8 | 0 | Runtime delivered; no model to serve |
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
- **The only active modelling line is `m1-discrete-attention`.** Its best trained artifact is below a word-count table (65 and 463 of 14,287 development rows against 1,226).

## Where to go next

- [How research is done now](../README.md), the rules for notebooks, sheets and releases.
- [What the project knows](../findings.md), [what is still open](../open-questions.md) and [how the project got here](../history.md).
- [Mechanism board](../../mechanisms/README.md): the 27 mechanisms with ACTIVE, READY or PARKED status.
- [Archive catalog](../../../archive/CATALOG.md): every dated record, with one generated page per line under `archive/catalog/`.
