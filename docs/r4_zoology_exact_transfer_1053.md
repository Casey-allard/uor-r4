# Source-positive Zoology cell on exact #1045 bytes (#1053)

- **Executed status:** `STOCK_CELL_TRANSFER_MISS`; the single frozen run ended
  at epoch 64 with `984/8,192 = 12.01171875%` development top-1. Fresh-process
  verification passed. The result and optimizer-clock interpretation are
  appended below; the original pre-run contract is preserved.
- **Pre-run status:** `NOT_RUN`.
- **Authority:** [issue #1053](https://github.com/UOR-Foundation/uor-r4/issues/1053),
  under #973; follows the [positive #1050 reproduction](r4_zoology_release_reproduction_1050.md).
- **Question:** can a freshly initialized copied causal-softmax cell learn the
  exact open #1045 assignment-disjoint population with the #1050-positive
  training semantics?
- **Claim boundary:** this is an exact-byte transfer decision. Ordinary causal
  softmax attention already works at the released #1050 configuration. A miss
  here does not revoke that result or falsify attention.

## Bound inputs and one-arm contract

The source-positive reference result is
`blake3:bd16d012c01262ffb8c5197e4cf316c6fee1d722cf0700a0048386180a8122e0`.
Only its evidence identity is inherited: this run must not open its fitted
weights. The copied architecture retains the credited
[HazyResearch/Zoology ICLR24 source](https://github.com/HazyResearch/zoology/tree/de4e258784224e09909c257ff3ea040f089ed660)
and the CPU/query-only-projection adaptations documented in #1050.

The [#1045 record](r4_role_tagged_associative_curriculum_1045.md) binds the open
8,192 construction rows and 1,024 assignment-disjoint development rows. The
recorded split identity is
`blake3:d36937f974e5e96dc697b219db8a7eb448dff7192abdf88bf6b21000f58b1f48`.
The #1053 preparation must bind and validate the exact serialized population
before training; it must not generate substitute rows or reopen sealed data.
Role bytes are validated for provenance but are not model inputs.

The only primary arm is frozen as follows:

| Component | Frozen value |
| --- | --- |
| Initialization | independent seed `123`; no fitted predecessor weights |
| Cell | width 64, two blocks, one head; copied causal softmax |
| Required data-shape changes | vocabulary 4,096; context 120; eight K/V pairs |
| Learning rate | `0.00046415888336127773` (`0x1.e6b4b396428e5p-12`) |
| Optimizer | AdamW, weight decay `0.1` |
| Batch | `512` |
| Schedule | cosine, at most 64 epochs |
| DataLoader semantics | train and development shuffled; shared Torch RNG; `num_workers=0` |
| Epoch order | train, then development, then strict `accuracy > 0.99`; scheduler step only after a miss |
| Maximum training work | `8,192 × 8 × 64 = 4,194,304` query presentations |
| Maximum primary development work | `1,024 × 8 × 64 = 524,288` query presentations |

Development is open and evaluated each epoch for the declared stop decision;
it is not a sealed terminal population. There is no learning-rate sweep,
second arm, checkpoint selection by a different metric, or same-issue tuning.

## Required lifecycle repairs and focused admission

Before admission, the new implementation/dependency commitment must include
`pyproject.toml` and `uv.lock`. A restored checkpoint that already ends in a
strict passing epoch must finalize that pass without another optimizer step.
Focused tests must prove both behaviors. The #1050 implementation and evidence
remain immutable; these repairs apply only to this successor.

Admission requires the named source-cell, exact-loader, causal-prefix,
query-only loss/gradient projection, shared DataLoader RNG, dependency-CID,
and passing-checkpoint-finalization checks. A failed mechanics check returns
`INVALID_TRANSFER_MECHANICS` and stops this issue; any repair campaign requires
a separately scoped issue.

Measure one-, four-, and eight-intra-op-thread CPU plans with one training
process and zero DataLoader subprocesses. Select the measured-fastest stable
eligible plan, recording its backend, timing, RSS, and thread settings. CUDA
and MPS are forbidden. A plan is eligible only when its safety-adjusted
primary projection is at most `900 s` and peak RSS is at most `8 GiB`.
No eligible plan means `NOT_RUN_PREFLIGHT`, not a model failure. CPU work and
memory are measured, not inferred from thread count.

The `900 s` limit governs the safety-adjusted admission projection, not a
strict physical cutoff. An additional safeguard checks cumulative elapsed
time before starting each epoch; an epoch already in progress may finish
beyond that point. An early stop is `INCOMPLETE_EPOCH_BOUNDARY_BUDGET`, not a
transfer miss, and does not authorize the binding control.

## Ordered decision and falsifier

The metric to move is #1045 development top-1, previously `7,137/8,192 =
87.121582%`, to strict `>99%`. Every one of the 8,192 development query targets
is directly scoreable from its admitted causal K/V binding; the structural
ceiling is therefore 100%. The sole primary run stops on its first strict pass
or after epoch 64, subject to the epoch-boundary resource safeguard above.

Only after a primary pass, evaluate the existing exact data-level
binding-permuted development control once, using the same fitted artifact and
no retraining. This control adds exactly 8,192 query evaluations. It must
reduce top-1 by at least 50 percentage points relative to the passing primary
evaluation.

| Outcome | Terminal and resulting action |
| --- | --- |
| Primary `>99%`, binding-permuted drop `≥50pp` | `STOCK_CELL_PASSES_EXACT_BYTES`; scope a separate coherent-R4 K/V transport/replacement parity issue |
| Primary never `>99%` by epoch 64 | `STOCK_CELL_TRANSFER_MISS`; stop without R4 changes; record a serialization/population-transfer miss |
| Primary passes, control drop `<50pp` | `NONASSOCIATIVE_SHORTCUT`; reject the score as binding evidence |
| C0/lifecycle mechanics fail | `INVALID_TRANSFER_MECHANICS`; do not train or repair within this campaign |
| No CPU plan fits the frozen limits | `NOT_RUN_PREFLIGHT`; do not train |

No outcome authorizes English, generation, reasoning, R4 geometry, W8/modulo-256
softmax, integer/table lowering, product work, width/head changes, additional
controls, or broad QA in this issue. The intended later CPU-native geometric
runtime is unchanged; this offline control retains ordinary floating-point
matrix operations and softmax.

## Pre-run evidence ledger

This ledger records the state before execution; completed evidence is appended
below rather than silently replacing this contract.

| Evidence | Pre-run state |
| --- | --- |
| Exact input/population identities and read audit | `NOT_RUN` |
| Implementation/dependency and preparation CIDs | `NOT_RUN` |
| Focused loader/model/lifecycle checks | `NOT_RUN` |
| 1/4/8-thread CPU preflight and CID | `NOT_RUN` |
| Single primary arm, artifact/state/result CIDs | `NOT_RUN` |
| Binding-permuted control and CID | `NOT_RUN_PRIMARY_REQUIRED` |
| Fresh-process artifact/result verification | `NOT_RUN` |
| Python format/baseline lint, claim wording, Rust format, diff integrity | `NOT_RUN` |
| Role/H4/R4/W8/teacher/provider/sealed/future-value model reads | required zero; `NOT_RUN` |
| English, generation, reasoning, geometry, lowering, product work | `NOT_RUN_OUT_OF_SCOPE` |
| Broad workspace/release QA | `NOT_RUN_OUT_OF_SCOPE` |

Only the checks explicitly activated by #1053 are evidence for this issue.
The protected PR/queue's compatibility acknowledgements are transport only:
they perform no verification and must not be reported as test passes.

## Executed evidence (2026-09-02)

The single frozen run completed without a passing development epoch. Its
terminal is `STOCK_CELL_TRANSFER_MISS`, not an ordinary-attention falsifier.
The final epoch-64 artifact scored `984/8,192 = 12.01171875%` development
top-1 with NLL `6.79664158821106` nats. The best observed development score
was `990/8,192 = 12.0849609375%` at epochs 49 and 50; no best-checkpoint
selection replaced the final artifact. The final training epoch's online
metrics were `33,447/65,536 = 51.03607177734375%` and NLL
`4.135751157999039`; these are during-update metrics, not a post-fit training
reevaluation.

Primary wall time was `324.06136883283034 s`; total run wall was
`325.01393954106607 s`. Exactly 1,024 optimizer updates consumed 4,194,304
training query presentations, and 64 development passes consumed 524,288
query presentations. The binding-permuted model evaluation remains
`NOT_RUN_PRIMARY_MISS`, with zero control query decisions. Its input artifact
was prepared and validated, but no control result is claimed.

### Admission and lifecycle evidence

The implementation was frozen at commit `ec7b0770`. All 31 activated focused
tests passed across `test_zoology_transfer`, `test_zoology_transfer_contract`,
`test_zoology_release`, `test_zoology_control_model`, and
`test_zoology_control_data`. These cover the copied loader/model, causal and
query-projection mechanics, DataLoader RNG, dependency commitment, and
checkpointed-pass finalization. The new commitment includes `pyproject.toml`
and `uv.lock`; no #1050 evidence or historical implementation was changed.

C0 passed source loader/model goldens, exact initialization replay,
causal/query-projection parity, and its disposable `128/128` overfit. The
complete C0/CPU preflight took `29.601191874826327 s`. Each CPU plan repeated
its measured loss sequence deterministically; cross-thread byte equality is
not claimed.

| CPU threads | Stable | Peak RSS | Safety-adjusted primary projection |
| ---: | :---: | ---: | ---: |
| 1 | yes | 1,172,389,888 bytes | 540.387790 s |
| 4 | yes | 1,113,374,720 bytes | 367.999280 s — selected |
| 8 | yes | 1,200,177,152 bytes | 377.025853 s |

All plans fit the `900 s`/`8 GiB` limits. Four intra-op threads were fastest
on this M1 using Apple Accelerate/OpenMP, with one inter-op thread, one
training process, and zero DataLoader subprocesses. CPU time during its
measured training units corresponded to `1.886–1.939` active cores on average;
the configured four-thread count is not a claim of full four-core occupancy.
CUDA and MPS were not used.

Fresh-process verification revalidated the input/implementation/artifact
bindings, reproduced the final development logits CID, NLL and top-1, and
confirmed that the primary miss prevented control inference. Python format,
baseline Ruff `E4/E7/E9/F`, Rust format, claim wording, and diff integrity
passed. Broad workspace and release QA were not run; queue transport
acknowledgements add no verification evidence.
Verification added 8,192 development inference decisions and zero optimizer
updates, separate from the primary's 524,288 development presentations.
An independent read-only audit also matched all 20 exported tensors to the
checkpoint, validated the tied embedding/head and evaluation RNG, and found
exactly 1,024 AdamW steps with scheduler `last_epoch=64` and no passing epoch.

### Input, artifact, and result identities

The loader validated the exact #1045 split's row/assignment/KV-pair
disjointness. It read the one open MQAR payload and five public envelopes;
10,920 unique open rows (1,310,400 role tokens) were validated. Role-adapter
revalidation covered 9,216 rows and control-role revalidation 1,024 rows.
Validation of stored role bytes is provenance work, not a learned role input.
Model role, geometry, future-value, fitted-predecessor-weight, teacher,
provider, sealed, English, natural-text, tokenizer, and UOR-byte reads were
zero in their respective recorded ledgers.

| Evidence | CID |
| --- | --- |
| Source population | `blake3:54982556bc986ad8aa59bb408945fad85a5990b2afe29eb1d1b11d5db19e44c9` |
| Exact #1045 split | `blake3:d36937f974e5e96dc697b219db8a7eb448dff7192abdf88bf6b21000f58b1f48` |
| Serialized primary dataset | `blake3:96f154042f0fd920c7f6f3b1b650a6ce20f11c401f9ae0c81734f47ae231b7f1` |
| Primary tensor bytes | `blake3:baef4fd29bedddc5c9cd826b78101c5c412db0e883241e04e478e4e3baf1d8b1` |
| Prepared control dataset; inference not run | `blake3:34dad69bfbeda87ea7e4d5d7af2fd8434dc5ab33cc055df4a0965f0b7b96b693` |
| Implementation/dependency commitment | `blake3:e01665c060f03cd2dac1dadd8df9b8cfdcfea94d99eab6732a878f7fd97babdf` |
| Implementation tree | `blake3:fb293b15f3f098ec60d2a20650c16edf71d4629a28c35a01e5f1271f199e700a` |
| Preparation | `blake3:1de63b0377a0f64f3181de37e706306dfaa8242c40f667faf47f4aba812272f2` |
| Preflight | `blake3:93f11afcc7734d6305e965646a88f54a6c7d1db11e7d7891f50f1f30845a1cf9` |
| Primary run | `blake3:87b39d0a63516bdd9fec7b5fe2e2fc5fcf6d6d48d1afe3e00832472ab386fb3d` |
| Model artifact, 1,217,024 bytes | `blake3:4857d0fdaba7d3cbc64acba39ffb43200c2d6d78ae312aded1c5c89389505fed` |
| Model state | `blake3:0ded701c7eca62c7fde69f619d63921079519358a31ab1b72bab60b6a34b27c1` |
| Final development logits | `blake3:932f49e2dc46cdce8eef57b6bec3e852823ab22ae04cf8c085390c83e19cc622` |
| Result | `blake3:e2d1deb55a4612015ba924a94051beacd517f3c062c714c4972ba954f57621a1` |

### Interpretation: the optimizer clock was not matched

The pre-run phrase “serialization/population-transfer miss” is too narrow as
a causal diagnosis. The result establishes a miss of this bounded population
and training recipe; it does not isolate bad serialization. Retaining a
64-epoch cosine schedule while reducing the number of rows changes both
optimizer exposure and the rate of learning-rate decay in update units.

| Training clock | #1050 source-positive run | #1053 transfer |
| --- | ---: | ---: |
| Rows per epoch | 100,000 | 8,192 |
| Updates per epoch at batch 512 | 196 | 16 |
| Updates at observed stop | 3,920, strict pass at epoch 20 | 1,024, miss at epoch 64 |
| Training query presentations | 8,000,000 | 4,194,304 |
| Learning rate around update 1,024 | approximately `0.0004572` | final-epoch `0.00000027955`, then zero |

#1053's entire optimizer budget spans only `1,024 / 196 = 5.22449` source
epochs. #1050 was still near four-choice chance then; its first large binding
transition occurred between 2,744 and 2,940 updates, and its strict pass took
3,920. This is a material unresolved optimizer-clock/annealing mismatch, not
proof that the mismatch alone caused #1053's failure. Population size,
serialization, eight versus four K/V pairs, and context length also differ.
The independently verified #1050 ordinary-attention positive remains intact.

The recommended next action is one separately frozen optimizer-clock
correction: preserve these exact data, model, initial learning rate, and fresh
initialization, advance the cosine clock at the source's 196-update cadence,
and cap the primary arm at the observed source-positive 3,920 updates. Match
optimizer time explicitly, not nominal epoch labels. Matching updates does
not match query presentations or total work: this population has eight
queries per row rather than four. A new contract must report that larger
query dose and measured CPU budget before execution. This is a recommendation,
not a rerun, new issue, or changed verdict within #1053.

Coherent-R4 replacement, English, generation, reasoning, modulo-256 softmax,
and lowering remain `NOT_RUN`; no geometric mechanism is rejected or promoted
by this transfer miss.

## Successor outcome (#1055, 2026-09-02)

The separately frozen optimizer-clock correction completed 3,920 updates on
these unchanged data and reached `5,594/8,192 = 68.2861328125%` development
top-1, up `56.2744140625` percentage points from this record. It retained the
gain but missed its strict 99% target; no binding-control inference ran.
Fresh-process replay passed. See the [#1055 record](r4_zoology_optimizer_clock_1055.md)
for the complete, still-improving late curve and the recommendation for a
separately authorized bounded continuation of that checkpoint. This is not a
measured convergence point or evidence that more unique data caused the gain:
the same 8,192 training rows were reused. No continuation was launched, and
this appendix does not change #1053's frozen result.
