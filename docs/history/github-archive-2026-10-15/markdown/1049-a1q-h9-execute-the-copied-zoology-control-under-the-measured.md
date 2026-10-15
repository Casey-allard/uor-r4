# #1049 — A1Q-H9: execute the copied Zoology control under the measured CPU wall

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-02T08:40:18Z
- **Updated:** 2026-09-02T09:26:39Z
- **Closed:** 2026-09-02T09:26:18Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1049
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #973. Successor to #1047 and its merged PR #1048.

## Objective

Execute the already copied and source-parity-qualified ICLR24 HazyResearch/Zoology one-head width-64 causal-softmax MQAR cell. #1047 stopped `NOT_RUN_PREFLIGHT` only because its all-core 8-thread projection was `959.212581 s` after the frozen safety factor, 59.2 seconds above a 900-second resource wall. This issue changes the execution wall to 1200 seconds and changes no scientific mechanism.

## Frozen predecessor

- #1047 merge: `677fb133b6d6a01fe384450b66beabbbd1b8f9a5`
- #1047 implementation tree: `blake3:c848c05ae53bc3adc0a8f7099ceed43657b6348e4e00fe3aaef5cf1368cc38de`
- #1047 preflight: `blake3:78158700e632d303bf674ed544f997a0e14eb89947470f5032e6acc75c830c9b`
- #1047 result: `blake3:b453abccc6ae0db9cc186c791aba268555dc0e75fe687c994e940254b0ac9ef6`
- C0 source mechanics and 32-row `128/128` overfit: PASS
- fastest measured plan: CPU-only, all 8 host cores, batch 64

## Frozen change boundary

Allowed changes are issue/policy/result provenance for this successor and `HARD_WALL_SECONDS: 900 -> 1200`. Preserve the exact source/model equations, initialization and RNG namespaces, source and #1045 populations, batch 64, AdamW settings, 64-epoch cosine schedule, query-only scoring, thresholds, two-consecutive rule, binding-permuted control, CPU-only 1/4/8-thread preflight, read ledger, and stop decisions. CUDA and MPS remain forbidden. No width, head, learning-rate, loader, serialization, optimizer, or data change is allowed.

Modulo-256/W8 remains available for discrete role provenance, wheel/table operations, and later lowering. It is excluded from this ordinary real-valued softmax control and is not a substitute for probability normalization.

## Run contract

    metric to move:       #1045 exact-byte assignment-disjoint dev top-1, 87.121582% -> >=99%
    reachability ceiling: every query has one physically admitted source K/V; 100% reaches attention
    cheap instrument:     exact source C0 plus measured 1/4/8-thread CPU timing
    proceed only if:      C0 passes; projected C1+C2 <=1200s; peak RSS <=8GiB
    if positive:          require binding-permutation drop >=50pp, then align R4 addressing to the demonstrated cell
    if negative:          follow the unchanged C1/C2 stop code; do not tune this issue
    cost estimate:        <=1200s execution wall after a roughly 40s CPU preflight

C1 remains the scaled source-native V8192/T64/four-pair control at 8192 construction and 1024 development rows. C2 remains the exact #1045 V4096/T120/eight-pair assignment-disjoint bytes at 8192 construction and 1024 development rows. Each requires two consecutive construction >=99.5% and development >=99%. C2 is capped at 4,194,304 query presentations. Binding permutation runs only after C2 passes and must drop at least 50 percentage points.

## Definition of done

- New create-once implementation/preparation/preflight/result identities bind the successor and predecessor.
- C0 and the measured all-core admission plan reproduce.
- At most one C1 and, if reached, one C2 run execute under the 1200-second hard wall.
- Exact metrics, controls, artifacts, read/work ledger, decision, and CIDs are recorded in docs and issue comments.
- Focused tests and claim wording pass.
- Delivery lands through PR and protected merge queue.

A capacity stop is not an attention negative. A C1/C2 result is ordinary causal-softmax evidence only; R4/geometric attention, English, generation, reasoning, quantization, exact lowering, product, and release remain out of scope.

## Comments (2)

### comment 5507390333 — 2026-09-02T09:24:57Z — Casey-allard

#1049 executed once under the frozen 1,200-second wall and completed `SCALED_SOURCE_CALIBRATION_MISS`.

- C0 source mechanics / deterministic replay / causal-query parity / `128/128` overfit: PASS
- selected plan: CPU-only, all 8 host cores, batch 64, Apple Accelerate/OpenMP
- C1: 64 epochs, `2,097,152` query presentations, `195.201318 s`
- construction: `32,758/32,768` (`99.969482%`), NLL `0.0205207`
- best development: `999/4,096` (`24.389648%`)
- final development: `980/4,096` (`23.925781%`), NLL `9.174184`
- consecutive passes: 0
- C2 / binding-permuted: `NOT_RUN_C1_MISS`
- all forbidden/future/sealed/provider/teacher/cache/transport/role/H4 reads: 0
- artifact CID: `blake3:aa0980621f7cae3ce392003ee0230fe536d3c842f342fa9129fe8d77c45882dc`
- result CID: `blake3:9b36540d81d0967a3f7e2ccabed80900d31c904b6c747d9ba0d539b325b13373`

Read-only diagnosis: `4,089/4,096` predictions selected one of the four admitted row values, but `3,109` selected the wrong admitted value and only `980` matched the queried key. The scaled cell learned value-set extraction, not key-specific binding.

The pinned executable release differs in three load-bearing ways: 100,000 vs 8,192 construction rows, batch 512 vs 64, and best-of-four rates vs one. #1050 freezes one exact executable-source reproduction, staged to run the already-bound rate first and stop on the first strict `>99%` positive.

Delivery: PR #1051. Focused tests 25/25, fresh-process result verification, Python compile, fmt, claim wording, diff check, and independent review all pass. `data.py` and `model.py` remain byte-identical to protected #1047/main.


### comment 5507411128 — 2026-09-02T09:26:39Z — Casey-allard

Delivered through protected merge queue in PR #1051 at merge commit `7c0c66e5e69d8523101b0ec6088e5a254b0c93fa`. Queue-side full CI completed successfully. Issue closed by the merged PR; successor #1050 remains the sole active copied-control reproduction.


---
_Comment/review ids in this file: 5507390333, 5507411128_

