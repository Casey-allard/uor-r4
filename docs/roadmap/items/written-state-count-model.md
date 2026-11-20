---
id: written-state-count-model
title: Test the written-state geometric count model against the frozen reference
milestone: research
area: training
status: ready
open_to: lab
needs_gpu: true
depends_on: [m2-donor-corpus-generation]
notebook: docs/research/notebook/m4-serving.md
scope: [docs/research/notebook/m4-serving.md, docs/research/notebook/data-tokenizer.md, crates/uor-r4-discrete/**, crates/uor-r4-integer/src/discrete_d11.rs, crates/uor-r4-integer/src/discrete_d11/rival_admission.rs, crates/uor-r4-integer/src/all_starts.rs, crates/uor-r4-integer/src/all_starts/original.rs, crates/uor-r4-integer/src/all_starts/rival.rs, crates/uor-r4-integer/src/bin/uor-chat.rs, crates/uor-r4-integer/tests/discrete*, docs/labs/kn-reference/**, docs/loop/tasks/written-state-count-model/**, docs/roadmap/items/written-state-count-model.md]
done_class: gpu
---

## Why it matters

The November 16 decision puts the current training and serving experiment on one integer state. Counts supply the baseline. The later controls test whether a geometric address carries information beyond token context at equal table bytes. M2 is the next capability milestone.

## What is missing

P0 freezes the split, reference models and scoring instruments. PR #4057 measured 2.004736 bits per byte on DEV-tune with the order-2 modified Kneser–Ney reference. PR #4058 merged at `11c2901d9`. The DEV-tune sweep selected order 10, the maximum its key could represent. On sealed DEV-test the reference scored **1.418136 bits per byte on all targets** and **1.743746 on the novel subset**. The corresponding bar Q thresholds for a candidate are 1.134508 and 1.394996 bits per byte. The geometric model’s P1–P5 comparisons remain open under their original gates. [Calibration](../../loop/tasks/written-state-count-model/claude-421/REPORT.md) · [Sealed reference and thresholds](../../loop/tasks/written-state-count-model/claude-422/REPORT.md) · [Merged PR #4058](https://github.com/UOR-Foundation/uor-r4/pull/4058)

## Done when

Follow the accepted phase table, including its stop and fallback branches. P1 checks the GPU count stream against the CPU oracle and serving read-back. P2 tests the geometric address against three equal-byte controls. P3 scores complete replies and error-written rows. P4 requires the lossless served artifact and raw-word replay. P5 tests the scored geometric core under its registered cost and quality limits. Record each phase at its own evidence level and keep the original bars, controls and owner-only proposals in their decision record. [Phase gates](../../../PLAN.md#approved-sequences) · [Decision and thresholds](../../research/reviews/pi-review-2026-11-16/DECISION.md)

## How to start

Record each model or scoring result in the [serving notebook](../../research/notebook/m4-serving.md), and split or census provenance in the [data notebook](../../research/notebook/data-tokenizer.md). Read the last merged phase report and the native-state contract, then take the first unmet gate in the phase table. The P0 scorer or a trained control alone cannot advance a model milestone. [Native-space contract](../../loop/tasks/written-state-count-model/claude-409/NATIVE-SPACE.md)

## Rules that apply

Use the shared native operators and lossless representation specified in D85. Pre-register each deciding GPU phase with a non-author FIT. Preserve the dataset's protected-asset rules and report finite checks, reference scores and trained-model evidence separately. [Charter](../../../CHARTER.md)

## Sources

[Decision card](../../loop/tasks/written-state-count-model/claude-409/DECISION.md) · [D87 corpus receipt](https://github.com/UOR-Foundation/uor-r4/pull/4028) · [Phase order](../../../PLAN.md#approved-sequences)

The [18 November program assessment and closure plan](../../loop/tasks/written-state-count-model/codex-9/CLOSURE-PLAN.md) reconciles the P0 amendments, Wick selector and writer diagnostics, LexLean correspondence and remaining M2–M8 product gates. Its proposed experiments include a crossed role/value-boundary test in parallel with the P1 frozen count baseline.

The [first implementation and Wick decision](../../loop/tasks/written-state-count-model/codex-10/REPORT.md) replays the 1% count stream and packages a learned occurrence reader for ordinary text. The crossed writer and held copy-eligibility tests expose transfer and abstention errors. The deployable full-support P1 mixer, P2 predictive comparison and M2 free-reply milestone remain open.

The [fresh instrument and transfer retest](../../loop/tasks/written-state-count-model/codex-16/REPORT.md) independently verifies correct and rival source behavior and finds distinct requested-role ties and value-start errors on unmarked questions. The role-discrimination and boundary tests proceed under the existing milestone and P1–P5 criteria.
