# #1050 — A1Q-H10: reproduce the exact released Zoology width-64 attention control

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-02T09:19:44Z
- **Updated:** 2026-09-02T13:28:53Z
- **Closed:** 2026-09-02T13:28:05Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1050
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #973. Successor to #1049 and the credited source port in #1047.

## Objective

Reproduce the exact executable-source ICLR24 HazyResearch/Zoology Figure 2 attention control at the smallest published positive boundary before returning to UOR bytes. #1049 passed literal-source mechanics and learned the four admitted values, but its scaled 8,192-row, batch-64, one-rate calibration remained at four-choice key/value binding chance. The pinned release reports perfect MQAR for width-64 attention and uses materially more data, batch 512, and a four-rate sweep.

This is ordinary causal-softmax source calibration only. It is not R4/geometric attention, English, generation, reasoning, quantization, exact lowering, product, or release work.

## Authority and exact source contract

Executable authority: HazyResearch/Zoology commit `de4e258784224e09909c257ff3ea040f089ed660`, especially `zoology/experiments/paper/figure2.py`, `zoology/config.py`, and `zoology/train.py`.

Freeze the T=64 / four-KV / d_model=64 attention arm exactly:

- vocabulary 8,192;
- 100,000 construction rows and 3,000 test rows;
- released row generation and ordering, train seed 0 and test seed 10;
- batch 512;
- two TransformerBlocks, one width-64 attention head, learned positions;
- Identity state mixer, tied embedding/head, released initialization and dropout;
- seed 123, AdamW weight decay 0.1, cosine decay, at most 64 epochs;
- four predeclared rates: `1e-4`, `4.6415888336e-4`, `2.1544346900e-3`, `1e-2`;
- source early stop after one evaluation with test accuracy strictly greater than 0.99.

The paper appendix conflicts with the pinned executable on batch size, warmup, and state-mixer prose. Do not combine those contracts: the pinned executable source is authority for this reproduction.

## Ordered execution

1. Bind exact source/population/model/optimizer identities and repeat literal-source C0 parity.
2. Preflight batch 512 on the CPU with 1/4/8-thread plans; select the fastest stable eligible plan. CUDA and MPS are forbidden.
3. Run the already-bound `4.6415888336e-4` arm first.
4. If that arm exceeds 99% test accuracy, stop positive without running the other rates.
5. Only if it misses, run the other three frozen source rates. Stop when one passes or after all four miss.

No construction threshold, two-consecutive rule, binding permutation, UOR C2 bytes, or invented intervention may be added to the source reproduction.

## Run contract

    metric to move:       exact released 3,000-row test top-1 -> >99%
    reachability ceiling: every query reaches ordinary causal softmax; source reports a width-64 positive
    cheap instrument:     exact source C0 plus measured batch-512 1/4/8-thread CPU timing and RSS
    proceed only if:      C0 passes; one arm projects <=3000s and peak RSS <=8GiB
    if positive:          stop the sweep; authorize a fresh exact-#1045-byte C2 transfer issue
    if negative:          run remaining frozen source rates; if all miss, audit port/source parity and stop without changing R4
    cost estimate:        35-40 minutes raw / <=50-minute budget per arm; <=3.31h worst-case serial, likely shorter by early stop

## Falsifier

Only all four released learning-rate arms failing strict `>99%` test accuracy by epoch 64 under exact executable-source parity falsifies the local reproduction. A one-rate miss does not.

## Modulo-256 boundary

W8 remains the intended substrate for discrete roles, modular wheel/table operations, packed shift-add lowering, and later route/runtime representation. `Z/256Z` is not the probability field for this offline real-valued softmax reproduction.

## Definition of done

- Exact executable-source contract and the #1049 result are recursively bound.
- C0 and batch-512 all-core resource admission pass, or a truthful `NOT_RUN_PREFLIGHT` is published.
- One to four frozen arms execute in the declared order with early stop.
- Per-arm artifacts, top-1/NLL, work/read ledgers, timings, CIDs, and the final decision are recorded.
- Focused tests, formatting, and claim wording pass.
- Delivery lands through PR and the protected merge queue.


## Comments (2)

### comment 5509826647 — 2026-09-02T12:52:30Z — Casey-allard

Implementation freeze before execution

- Native parent edge to #973 is now present; #1050 remains assigned and unblocked.
- Clean branch/worktree is based on protected `main` at `7c0c66e5`; implementation commit is `06ff97e4`.
- #1049's package and verifier remain byte-unchanged and its result reverified in a fresh process.
- The new sibling lifecycle uses the exact released 100,000/3,000 population, batch 512, width 64, two layers, one head, Identity state mixer, shared train/test DataLoader RNG trajectory, train -> test -> strict `>0.99` stop -> scheduler-step-on-miss ordering, and locked `np.logspace` float identities.
- Frozen execution order is source indices `(1,0,2,3)`: `0.00046415888336127773`, `0.0001`, `0.002154434690031882`, `0.01`; first positive stops the sweep.
- Declared integration exceptions are CPU placement and query-only tied-head projection. A direct loss/gradient parity test passed; this is exact source configuration/mechanics, not byte-identical upstream CUDA execution.
- Preflight will repeat C0 and measure fresh 1/4/8-thread, batch-512 CPU subprocesses. Proceed only at <=3,000 seconds projected per arm and <=8 GiB RSS. CUDA and MPS are forbidden.
- If one arm passes: stop and scope exact-#1045-byte C2 separately. If all four miss: audit source-port parity and do not change R4.

Cleanup before execution removed four clean merged issue worktrees plus regenerable Cargo output, reclaiming roughly 5-6 GiB. Model, corpus, predecessor evidence, locked CPU environment, research material, and unrelated dirty-main files were preserved.


### comment 5510307539 — 2026-09-02T13:28:53Z — Casey-allard

Terminal result: SOURCE_REPRODUCTION_POSITIVE.

The first frozen source arm, learning rate 0.00046415888336127773, crossed the strict >99% source rule at epoch 20: 11,900/12,000 = 99.1666667% top-1, NLL 0.05124610455830892, arm wall 577.834602 seconds. The predeclared early stop fired, so the remaining three rates were not run.

Evidence:
- preparation CID blake3:bdb9dd01ea0e115eaff54c0536833f2d13cae3d32c9c60cf90070225f031a335
- preflight CID blake3:03000b3e10f7181811cf1f4439ff4b2e70b906fe1de835defeb90f90ef527019
- arm CID blake3:77357ec8e51531c1b21f16270956cd95cf1f68e8ccee886c8fe57c1e99589874
- artifact CID blake3:163cf3e5375b3e721fa7a826acdb2dfc809e5989209b03fb2a3eea3e3d5459e9
- state CID blake3:600bdc76cefff79f4be8709197b15252cb531892fad0db2156b36b865c01877e
- result CID blake3:bd16d012c01262ffb8c5197e4cf316c6fee1d722cf0700a0048386180a8122e0

Independent audit and fresh verification passed. The source test split is held out from gradient updates but drives early stopping; it has zero full-row overlap and 294 recurring K/V assignments. Post-result partition accuracy was 291/294 on recurring pairs and 11,609/11,706 on pairs absent from training.

This establishes copied ordinary causal-softmax binding on independently generated rows. It rules out a broken copied cell and localizes #1049 to bundled reduced-versus-released training-contract differences without isolating a causal factor. It is not R4/geometric attention, English generation, reasoning, modulo-256 softmax, or exact lowering.

Delivered by PR #1052, protected merge 0cffa3fb0a17a891383aa76d34104b9a906237fc. The binding next action is one fresh exact-#1045-byte C2 transfer issue.


---
_Comment/review ids in this file: 5509826647, 5510307539_

