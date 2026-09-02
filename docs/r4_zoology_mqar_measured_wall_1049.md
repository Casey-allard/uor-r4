# Measured-wall Zoology MQAR execution (#1049)

- **Status:** `SCALED_SOURCE_CALIBRATION_MISS`
- **Policy:** `ZoologyMQARControlV2MeasuredWall`
- **Authority:** [issue #1049](https://github.com/UOR-Foundation/uor-r4/issues/1049),
  child of #973 and resource-policy successor to #1047
- **Source mechanism:** credited HazyResearch/Zoology ICLR24 release
  `de4e258784224e09909c257ff3ea040f089ed660`, Apache-2.0
- **Claim boundary:** ordinary one-head causal softmax on open MQAR only; no R4,
  geometric-attention, English, generation, reasoning, quantization, exact
  lowering, product, or release claim

This is the frozen execution continuation for #1047. It changes no scientific
mechanism. #1047 passed literal-source loader/model goldens, deterministic
mechanics, causal/query-only projection parity, and the disposable 32-row
overfit. Its C1/C2 run was `NOT_RUN_PREFLIGHT` because the fastest all-core CPU
plan projected `959.212581` seconds after the required safety factor, above its
900-second wall.

## Bound predecessor

- protected #1047 merge:
  `677fb133b6d6a01fe384450b66beabbbd1b8f9a5`;
- implementation tree:
  `blake3:c848c05ae53bc3adc0a8f7099ceed43657b6348e4e00fe3aaef5cf1368cc38de`;
- preflight:
  `blake3:78158700e632d303bf674ed544f997a0e14eb89947470f5032e6acc75c830c9b`;
- result:
  `blake3:b453abccc6ae0db9cc186c791aba268555dc0e75fe687c994e940254b0ac9ef6`.

#1047 produced no fitted artifact and no scientific result. Its only binding
observation is that the all-core batch-64 plan needs a larger execution wall.

## Exact change boundary

The successor changes:

1. issue/result policy provenance from #1047 V1 to #1049
   `ZoologyMQARControlV2MeasuredWall`; and
2. the combined C1+C2 hard wall from 900 to 1,200 seconds.

It preserves the V1 source-attribution CID, source and #1045 population CIDs,
RNG and row/shuffle namespaces, model equations, width 64, two layers, one
head, learned positions, initialization, dropout, tied head, batch 64, AdamW
settings, 64-epoch cosine schedule, query-only scoring, thresholds, two-pass
qualification, binding-permuted control, CPU-only 1/4/8-thread preflight, and
all read/work prohibitions. CUDA and MPS remain forbidden.

The inherited `uor-r4/1047/...` namespaces are intentionally unchanged. A new
namespace would change row identities or optimization ordering and would be a
scientific intervention rather than a resource-policy continuation.

## Modulo-256 boundary

W8 remains the intended substrate for discrete roles, modular wheel/table
operations, packed shift-add lowering, and later route/runtime representation.
It is excluded from this ordinary real-valued softmax control. Since
`Z/256Z` is not a field and lacks general inverses, it does not directly supply
continuous probability normalization.

## Ordered controls

The runner repeats C0 to bind the successor implementation, then measures the
same 1-, 4-, and 8-thread batch-64 CPU plans. It proceeds only when C0 passes,
the projected C1+C2 cost is at most 1,200 seconds, and peak RSS is at most
8 GiB.

C1 remains the source-native `V=8192`, context-64, four-pair population with
8,192 construction and 1,024 development rows. C2 remains the exact #1045
`V=4096`, context-120, eight-pair population with 8,192 construction and 1,024
assignment-disjoint development rows. Each rung requires two consecutive
evaluations with construction top-1 at least 99.5% and development top-1 at
least 99%. C2 remains capped at 4,194,304 query presentations.

Only after C2 passes does the data-level binding-permuted control run; it must
reduce development top-1 by at least 50 percentage points.

## Frozen decisions

| Observation | Verdict | Next action |
| --- | --- | --- |
| C0 or source parity misses | `INVALID_CONTROL_PORT` | Repair parity only in a new issue; no scientific inference. |
| No CPU plan fits 1,200 seconds / 8 GiB | `NOT_RUN_PREFLIGHT` | Stop without an attention verdict. |
| C1 misses | `SCALED_SOURCE_CALIBRATION_MISS` | Stop before C2; decide separately whether the full released calibration merits its cost. |
| C2 misses construction or two-pass qualification | `STOCK_CELL_EXACT_QUALIFICATION_MISS` | Isolate exact-byte fit/temporal qualification; do not modify R4. |
| C2 construction passes but assignment-disjoint development misses | `STOCK_CELL_TRANSFER_MISS` | Isolate serialization versus assignment-disjointness; do not modify R4. |
| C2 passes but binding-permuted drop is below 50pp | `NONASSOCIATIVE_SHORTCUT` | Reject the score as associative-attention evidence. |
| C2 passes with binding-permuted drop at least 50pp | `STOCK_CELL_PASSES_EXACT_BYTES` | Align the R4 cell to the demonstrated one-head width-64 boundary. |

No outcome authorizes same-issue tuning or rerun.

## Run contract

    metric to move:       #1045 assignment-disjoint dev top-1, 87.121582% -> >=99%
    reachability ceiling: every query has one physically admitted source K/V
    cheap instrument:     exact C0 plus measured 1/4/8-thread CPU timing
    proceed only if:      projected C1+C2 <=1200s and peak RSS <=8GiB
    if positive:          require binding-permuted drop >=50pp; then align R4
    if negative:          follow the unchanged stop code; do not tune #1049
    cost estimate:        about 40s preflight; <=1200s C1+C2 execution wall

## Pre-run publication ledger

| Item | Frozen state |
| --- | --- |
| #1047 resource predecessor | bound |
| V1 source attribution and scientific namespaces | preserved |
| #1049 implementation/preparation | `NOT_CREATED` |
| C0 and CPU preflight | `NOT_RUN` |
| C1 source-native calibration | `NOT_RUN` |
| C2 exact #1045 bytes | `NOT_RUN` |
| Binding-permuted control | `NOT_RUN` |
| Result/artifacts | `NOT_CREATED` |

## Executed result

The create-once preparation bound implementation tree
`blake3:3a84949c9767f6cdad2a468c5aeb3bc226beea6d1889ff839bff938895e977ad`,
implementation CID
`blake3:fbdc38c477a65da3a5948eea9c150ffa8da0cb52c00c5853e5f0b2974d903957`,
and preparation CID
`blake3:ffb57c86fa725a5fe021a5e904310f1b69dc23d682b5a25c20f95d77527c4c2e`.
The source-derived `data.py` and `model.py` remained byte-identical to the
protected #1047 merge.

C0 repeated successfully. Literal released-source loader and model goldens,
deterministic initialization replay, causal-prefix and query-only projection
parity, and the disposable 32-row `128/128` overfit all passed. Preflight CID
`blake3:75a6c566b249a6454fc0edd3f9bd2dcdb2cbbc76c2c3c3fbe66a5b7eb5c015`
selected the all-core CPU-only 8-thread, batch-64 plan. Its projected combined
wall was `957.947823 s`, peak RSS was `603,930,624` bytes, and both fit the
frozen limits. PyTorch reported Apple Accelerate BLAS and OpenMP; CUDA and MPS
remained disabled.

C1 then completed all 64 epochs and `2,097,152` query presentations in
`194.690164 s`; total run wall was `195.201318 s`. It nearly memorized the
construction population, reaching `32,758/32,768` (`99.969482%`) with NLL
`0.0205207`. Assignment-disjoint development did not learn key-specific
binding. Its best top-1 was `999/4,096` (`24.389648%`), first reached at epoch
10 with NLL `6.724377`; final top-1 was `980/4,096` (`23.925781%`) with NLL
`9.174184`. No evaluation passed the 99% development threshold, so consecutive
passes remained zero.

The immutable verdict is `SCALED_SOURCE_CALIBRATION_MISS`. C2 and its exact
#1045 bytes are `NOT_RUN_C1_MISS`; the binding-permuted control is also
`NOT_RUN_C1_MISS`. Future-value, sealed-input, provider, teacher, cache,
transport, role-model, and H4-model reads were all zero. The C1 artifact is
bound to
`blake3:aa0980621f7cae3ce392003ee0230fe536d3c842f342fa9129fe8d77c45882dc`
with state CID
`blake3:aaff1b1e919b49928181e6f29910f275a8fc32f5050a2276b0c9758cab73febc`.
The result CID is
`blake3:9b36540d81d0967a3f7e2ccabed80900d31c904b6c747d9ba0d539b325b13373`.
Fresh-process structural verification passed.

## Post-result diagnosis and next decision

A read-only score diagnosis, performed after the frozen verdict and not used
to reclassify it, found that the artifact predicted one of the four values
physically admitted in the row on `4,089/4,096` development queries
(`99.829102%`). Only `980` were the value bound to the queried key; `3,109`
were another admitted value and seven were outside the admitted set. Its four
admitted-slot prediction counts were `968`, `1,034`, `1,036`, and `1,051`.
The cell therefore learned value-set extraction while key-to-value assignment
remained at the four-choice chance boundary. This is a binding failure at this
training scale, not evidence that the copied softmax operation or causal
mechanics are broken.

The scaled construction set contains only `32,768` K/V presentations: an
average of `8.002` appearances per possible key and `8.000` per possible
value. It covered `4,093/4,095` keys and every value, but only `7/4,096`
development K/V pairs occurred exactly in training. The released Figure 2
contract uses 100,000 construction rows and 3,000 test rows, or 400,000 K/V
presentations—`12.207` times this construction exposure and approximately
`97.68` appearances per key/value. The paper reports that width-64 attention
solves MQAR perfectly at all tested sequence lengths:
[released Figure 2 configuration](https://github.com/HazyResearch/zoology/blob/de4e258784224e09909c257ff3ea040f089ed660/zoology/experiments/paper/figure2.py),
[paper Section 4.3](https://arxiv.org/abs/2312.04927).

That source-backed gap gives one exact released-control reproduction decision
value. [Issue #1050](https://github.com/UOR-Foundation/uor-r4/issues/1050)
freezes that reproduction. #1049 differs from the pinned executable release in three load-bearing
ways: 8,192 rather than 100,000 construction rows, batch 64 rather than 512,
and one learning rate rather than the released four-rate sweep whose maximum
is reported in Figure 2. The next issue must treat commit `de4e258...`
  executable source as authority: exact 100,000/3,000 populations, batch 512,
seed 123, the four predeclared rates `1e-4`, `4.6415888336e-4`,
`2.1544346900e-3`, and `1e-2`, and the source's single-evaluation strict
`>99%` early stop. The paper appendix differs from that executable source on
batch size, warmup, and state-mixer prose; the contracts must not be mixed.

Run the already-bound `4.6415888336e-4` arm first and stop positive if it
clears the source gate. Only if it misses do the other three frozen rates run.
Straight row-count scaling from the observed all-core wall estimates
`35–40 min` raw and about a 50-minute admission budget per arm; exact batch-512
timing and memory must replace that estimate in preflight. A positive may
authorize returning to C2. The clean falsifier is all four released rates
failing strict `>99%` test accuracy by epoch 64 under exact executable-source
parity; a one-rate miss is not a Figure 2 falsifier. Neither branch changes the
current W8 boundary: modulo-256 remains a later discrete role/table/lowering
substrate, not the probability domain for this offline softmax control.

## Final publication ledger

| Item | Observed state |
| --- | --- |
| #1047 resource predecessor | bound |
| V1 source attribution and scientific namespaces | preserved |
| #1049 implementation/preparation | created and verified |
| C0 and CPU preflight | passed; 8-thread batch-64 plan admitted |
| C1 source-native calibration | completed; development gate missed |
| C2 exact #1045 bytes | `NOT_RUN_C1_MISS` |
| Binding-permuted control | `NOT_RUN_C1_MISS` |
| Result/artifact | created and verified |
