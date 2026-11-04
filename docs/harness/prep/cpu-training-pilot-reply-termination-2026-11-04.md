# CPU reply termination diagnostic

**Status:** FIT

TEST FITNESS: FIT

## Question

At every generated decision, why does the served writer choose a non-BOS token?
Classify whether BOS is absent from the admitted cells, admitted but ranked
after another token, ranked first but suppressed by a decoder defect, or mixed.

This is a bounded diagnostic with no new training configuration or
optimization hypothesis. The exact 32-update replay only reconstructs the
pinned deepseek-109 artifacts. It does not change serving.

## Compute class

CPU development research under the owner-authorized D84 program in
`uor-r4-cpu-train`. GPU remains the qualification authority.

## Base artifacts and inputs

| Input | Identity |
|---|---|
| seed 1 FBS artifact | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| seed 2 FBS artifact | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |
| seed 1 trained artifact | `c856ce21cf46a4be5a0cc02e033d9c7d0cd674bc4ab96a1185a1b9ccf3b88de4` |
| seed 2 trained artifact | `10f91f7309c3ed36e3ce76878b2c75c07bae0102b3933571ccbd8f3b40b350e6` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |
| hardware | AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads |

## Data split

Replay the exact deepseek-109 training selection:

- 16 repeated opening keys;
- 102 training and 114 evaluation examples;
- 32 attempted Q24 updates and 32 Rayon workers;
- frozen, trained, memory-off and trained-memory-off complete-reply decode;
- 16 generated tokens per prompt.

Direct-target arms are not diagnostic subjects: this unit asks how the served
writer treats BOS before any target override.

## Steps

1. Build a fresh release `cpu-memory-panel-driver`.
2. Pass eleven focused reply tests, the writer diagnostic test and the full CPU
   suite before public replay.
3. Run the exact deepseek-109 training replay with:

```text
env RAYON_NUM_THREADS=32 target/release/cpu-memory-panel-driver \
  ARTIFACT DEV_TOKENS DEV_MASK DEV_MANIFEST OUT \
  --expected-artifact-sha256 ARTIFACT_PIN \
  --expected-tokens-sha256 TOKENS_PIN \
  --expected-mask-sha256 MASK_PIN \
  --expected-manifest-sha256 MANIFEST_PIN \
  --expected-trained-sha256 TRAINED_PIN \
  --steps 32 --keys 16 --per-key 16 --key-tokens 8 --reply-turn 8 \
  --reply-cap 16 --termination-diagnostic --seed 1
```

4. Repeat seed 1 in a fresh process.

For every generated step, record:

- generated token and underlying writer token;
- whether BOS's cell is admitted;
- BOS rank among all admitted writer candidates and whether it is top-8/top-1;
- BOS L1 distance, chosen-token L1 distance and their difference.

## Bar

Instrument:

- trained artifact hashes equal their deepseek-109 pins;
- each arm has 114 prompt rows and 1,824 generated decisions;
- reply-sequence digests equal the corrected deepseek-110 pins:
  - seed 1 frozen/memory-off/trained-memory-off
    `61e90d0e08997f9300b84718a951c2119bfe1b52d5da2f4575a4876f9aa51d49`;
  - seed 1 trained
    `ea95ef96a4adad3b3964912591ae0b9b9dfecf484aaae85cad9cb4dcbc769eec`;
  - seed 2 all four arms
    `09920e95529cec1e4c87975d25cd9225a339c30e11a78853262fa918ed2d767c`;
- chosen BOS is zero in all arms, because deepseek-110 had zero BOS stops;
- five served teacher-forced arms retain exact CPU/serving parity;
- all diagnostic distances are non-negative and
  `bos_minus_chosen_distance = bos_distance - chosen_distance`.

Partition every arm's steps into mutually exclusive counts:

```text
never_admitted       = decisions - bos_admitted
admitted_not_top1    = bos_admitted - bos_top1
top1_suppressed      = bos_top1 - chosen_bos
chosen_bos           = chosen_bos
```

Declare **Never admitted** only when `never_admitted > 0` and the other three
counts are zero. Declare **Admitted but never first** only when
`admitted_not_top1 > 0` and the other three are zero. Declare **Decoder defect**
only when `top1_suppressed > 0` and the other three are zero. Otherwise report
**Mixed** with all four exact counts; never force a mixed arm into one label.

`NoDecisions` is an instrument failure for this run: every registered prompt is
non-empty and has a 16-token cap. `AllChosenBos` is also an instrument/hash
failure here because the deepseek-110 pins require `chosen_bos = 0` for every
arm. The implementation classifies both explicitly instead of mislabelling them
as mixed.

## Minimum test power

Two model seeds, 114 repeated-key prompts, four query-movement arms and 16
generated decisions per prompt: 14,592 diagnostic steps.

## Outcomes

- Never admitted: fix termination-cell admission.
- Admitted but never first: test a served BOS movement objective.
- Top-1 but suppressed: fix the decoder as an instrument fault.
- Mixed: split the next work by admission versus ranking counts.

## Cost and stop

Local Ryzen 9 9950X only. Two exact training replays plus four diagnostic arms
per seed are expected below five CPU minutes. Stop before seed 2 if seed 1
projects beyond 30 minutes. Stop on generation/hash drift, missing parity,
nondeterministic classification or any post-observation tuning.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030
sheet: docs/harness/prep/cpu-training-pilot-reply-termination-2026-11-04.md
head: 4f25240d9d3ba9e30bc24709b8ea04cfba835aa8
reviewer: Codex fresh-context non-author review
note: bounded two-seed D84 CPU termination diagnostic; four query-movement arms;
114 prompts and 16 generated decisions per prompt; pinned deepseek-109 trained
hashes and deepseek-110 reply digests; writer diagnosis records BOS admission,
admitted-candidate rank, top-8/top-1 status and exact L1 margins; mutually
exclusive NeverAdmitted, AdmittedNotTop1, Top1Suppressed, AllChosenBos,
NoDecisions and Mixed classifications are implemented and tested; 30-minute
stop and seed-1 repeat registered
date: 2026-11-04

This FIT clears only the registered configuration at this exact head. Any change
to source, hashes, selection, arms, cap, controls or classification rules
requires re-review.
