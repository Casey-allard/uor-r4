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
scope: [docs/research/notebook/m4-serving.md, docs/research/notebook/data-tokenizer.md, crates/uor-r4-discrete/**, crates/uor-r4-integer/Cargo.toml, crates/uor-r4-integer/src/discrete_d11.rs, crates/uor-r4-integer/src/discrete_count.rs, crates/uor-r4-integer/src/discrete_d11/rival_admission.rs, crates/uor-r4-integer/src/discrete_d11/pair_rank.rs, crates/uor-r4-integer/src/all_starts.rs, crates/uor-r4-integer/src/all_starts/original.rs, crates/uor-r4-integer/src/all_starts/rival.rs, crates/uor-r4-integer/src/paired_role.rs, crates/uor-r4-integer/src/paired_copy.rs, crates/uor-r4-integer/src/contextual_source.rs, crates/uor-r4-integer/src/bin/uor-chat.rs, crates/uor-r4-integer/src/bin/pack-contextual-source.rs, crates/uor-r4-integer/src/bin/uor_chat/**, crates/uor-r4-integer/tests/discrete*, docs/labs/kn-reference/**, docs/loop/tasks/written-state-count-model/**, docs/roadmap/items/written-state-count-model.md]
done_class: gpu
---

## Why it matters

The November 16 decision puts the current training and serving experiment on one integer state. Counts supply the baseline. The later controls test whether a geometric address carries information beyond token context at equal table bytes. M2 is the next capability milestone.

## What is missing

P0 freezes the split, reference models and scoring instruments. PR #4057 measured 2.004736 bits per byte on DEV-tune with the order-2 modified Kneser–Ney reference. PR #4058 merged at `11c2901d9`. The DEV-tune sweep selected order 10, the maximum its key could represent. On sealed DEV-test the reference scored **1.418136 bits per byte on all targets** and **1.743746 on the novel subset**. The corresponding bar Q thresholds for a candidate are 1.134508 and 1.394996 bits per byte. The geometric model’s P1–P5 comparisons remain open under their original gates. [Calibration](../../loop/tasks/written-state-count-model/claude-421/REPORT.md) · [Sealed reference and thresholds](../../loop/tasks/written-state-count-model/claude-422/REPORT.md) · [Merged PR #4058](https://github.com/UOR-Foundation/uor-r4/pull/4058)

D31 delivered the complete 1% C1 count file and the full reserved-TRAIN A1 fitted state through the ordinary text client. An independent blind CUDA run reproduced all 192 causal decisions and full Q48 score hashes. Both read modes scored 0/6 fixed-gold replies through EOS, and changing the requested person left the first-step distribution unchanged when fact order was fixed. The item stays ready for a full-dose P1 candidate within the 4 GB cap, scored after frozen fitting and readback on the registered panels. DEV-test stayed sealed during D31. The P1(d) ft/dev floor, P2 equal-byte comparison, P3 reply gates and P4 M1-class measurements remain open. [D31 result and next comparison](../../loop/tasks/written-state-count-model/codex-31/REPORT.md) · [First wrong token](../../loop/tasks/written-state-count-model/codex-31/client/FIRST-WRONG.md)

## Done when

Follow the accepted phase table, including its stop and fallback branches. P1 checks the GPU count stream against the CPU oracle and serving read-back. P2 tests the geometric address against three equal-byte controls. P3 scores complete replies and error-written rows. P4 requires the lossless served artifact and raw-word replay. P5 tests the scored geometric core under its registered cost and quality limits. Record each phase at its own evidence level and keep the original bars, controls and owner-only proposals in their decision record. [Phase gates](../../../PLAN.md#approved-sequences) · [Decision and thresholds](../../research/reviews/pi-review-2026-11-16/DECISION.md)

## How to start

Record each model or scoring result in the [serving notebook](../../research/notebook/m4-serving.md), and split or census provenance in the [data notebook](../../research/notebook/data-tokenizer.md). Read the last merged phase report and the native-state contract, then take the first unmet gate in the phase table. The P0 scorer or a trained control alone cannot advance a model milestone. [Native-space contract](../../loop/tasks/written-state-count-model/claude-409/NATIVE-SPACE.md)

## Rules that apply

Use the shared native operators and lossless representation specified in D85. Pre-register each deciding GPU phase with a non-author FIT. Preserve the dataset's protected-asset rules and report finite checks, reference scores and trained-model evidence separately. [Charter](../../../CHARTER.md)

## Sources

[Decision card](../../loop/tasks/written-state-count-model/claude-409/DECISION.md) · [D87 corpus receipt](https://github.com/UOR-Foundation/uor-r4/pull/4028) · [Phase order](../../../PLAN.md#approved-sequences)

The [18 November program assessment and closure plan](../../loop/tasks/written-state-count-model/codex-9/CLOSURE-PLAN.md) reconciles the P0 amendments, Wick selector and writer diagnostics, LexLean correspondence and remaining M2–M8 product gates. Its proposed experiments include a crossed role/value-boundary test in parallel with the P1 frozen count baseline.

The [first implementation and Wick decision](../../loop/tasks/written-state-count-model/codex-10/REPORT.md) replays the 1% count stream and packages a learned occurrence reader for ordinary text. The crossed writer and held copy-eligibility tests expose transfer and abstention errors. This record predates the D31 1% full-support P1 prototype.

The [fresh instrument and transfer retest](../../loop/tasks/written-state-count-model/codex-16/REPORT.md) independently verifies correct and rival source behavior and finds distinct requested-role ties and value-start errors on unmarked questions. The role-discrimination and boundary tests proceed under the existing milestone and P1–P5 criteria.

The [D32 whole-user and candidate comparison](../../loop/tasks/written-state-count-model/codex-32/REPORT.md) recovered eight additional requested spans and one additional exact crossed reply. All excluded endpoints still failed. The wider scope also released two unsupported copies. The count identity, equal-byte and broader language gates remain open.

D33 met the original-token count identity gate P1(a) on C1. All eleven raw and eleven adjusted order summaries matched between the archived GPU writer, independently reconstructed TRAIN counts and the count-enabled normal D11 loader. The single model is 2,316,338,762 bytes, plus a 265,393-byte tokenizer. The full-dose pruned model, R-only refit and paired P1(b) score remain the next count dependency. [D33 three-way readback and byte evidence](../../loop/tasks/written-state-count-model/codex-33/REPORT.md)

D34 recovered the eight previously missing detached-name endpoints on the 16 exposed crossed prompts. Correct requested spans and bound assertions reached 16/16; excluded assignment stayed 0/16 and exact replies rose from 2/16 to 4/16. Each of the twelve remaining correct starts was eligible and tied or lost to a rival. The next role and value fits require disjoint matched prompts, an explicit incidental class and fixed-writer checks. [D34 originals, parity and consequences](../../loop/tasks/written-state-count-model/codex-34/REPORT.md)

D35 tested signed differences between candidate words with the original exact writer. Candidate-prefix ranking scored 14/224 ordinary exact versus 6 for baseline, with 93 new wrong-source rows. On 216 matched short objects it fell from 32 to 20 and scored 42 with supplied spans, tied with baseline and below reversal at 48. Keep the baseline. A label-free occurrence-role test and the deterministic whole-context count builder are the next distinct dependencies. [D35 cold client replay and source choices](../../loop/tasks/written-state-count-model/codex-35/REPORT.md)

D36 materialized the entire frozen C stream and applied the preregistered whole-context rule at the deployment-derived count cap. Threshold 318 occupied 3,912,334,481 compact bytes. The high-order groups retained 30.52% of raw support and 7.80% of adjusted support. The diagnostic D11 with its tokenizer occupies 3,999,195,913 bytes. Its ordinary untraced Linux process peaked at 4,702,887,936 bytes on Wick. The resident and warm-serving gates remain open. P1(b) still needs an R-only fit against the exact deployed table, a versioned Pick comparison on DEV-tune, and the registered paired DEV-test. The sealed R* archive kept its target-mass digest. Reproducing its canonical target tape is the identified dependency for the paired calculation. P2 follows P1(a,b). [D36 result](../../loop/tasks/written-state-count-model/codex-36/REPORT.md) · [Implementation and gates](../../loop/tasks/written-state-count-model/codex-36/NEXT.md)
