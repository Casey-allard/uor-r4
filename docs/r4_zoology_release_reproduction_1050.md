# Released-configuration Zoology width-64 attention reproduction (#1050)

- **Status:** `SOURCE_REPRODUCTION_POSITIVE`
- **Authority:** [issue #1050](https://github.com/UOR-Foundation/uor-r4/issues/1050),
  native child of #973 and successor to #1049
- **Source:** HazyResearch/Zoology ICLR24 release
  [`de4e258784224e09909c257ff3ea040f089ed660`](https://github.com/HazyResearch/zoology/tree/de4e258784224e09909c257ff3ea040f089ed660),
  Apache-2.0
- **Claim boundary:** ordinary one-head causal softmax learned independently
  generated held-out-row multi-query associative recall at the released T=64
  configuration; no R4, geometric-attention, English, generation, reasoning,
  modulo-256 softmax, exact lowering, product, or release claim

## Why this reproduction was necessary

#1049 copied the source cell successfully but trained it on only 8,192 rows,
batch 64, and one rounded learning rate. It learned to select one of the four
values in a row on `4,089/4,096` development queries, but key-specific binding
remained at four-choice chance. The executable Figure-2 source instead uses
100,000 training rows, 3,000 test rows, batch 512, and the maximum over four
`np.logspace(-4,-2,4)` learning rates.

#1050 therefore reproduced that smallest reported positive boundary before
returning to UOR bytes. It did not tune #1049 or change R4.

## Frozen executable-source contract

The pinned source files are the released
[`figure2.py`](https://github.com/HazyResearch/zoology/blob/de4e258784224e09909c257ff3ea040f089ed660/zoology/experiments/paper/figure2.py),
[`config.py`](https://github.com/HazyResearch/zoology/blob/de4e258784224e09909c257ff3ea040f089ed660/zoology/config.py),
[`train.py`](https://github.com/HazyResearch/zoology/blob/de4e258784224e09909c257ff3ea040f089ed660/zoology/train.py),
and
[`data/utils.py`](https://github.com/HazyResearch/zoology/blob/de4e258784224e09909c257ff3ea040f089ed660/zoology/data/utils.py).
The frozen arm uses:

- vocabulary 8,192, sequence length 64, and four K/V pairs;
- 100,000 train rows from seed 0 and 3,000 test rows from seed 10;
- batch 512, width 64, two layers, one head, learned positions, and Identity
  state mixer;
- released double initialization, tied embedding/head, and 0.1 attention and
  embedding dropout;
- seed 123, AdamW weight decay 0.1, and 64-epoch cosine decay;
- real train and test `DataLoader(..., shuffle=True, num_workers=0)` iteration
  on their shared Torch RNG trajectory;
- source ordering `train -> test -> strict accuracy > 0.99 -> scheduler step
  only on a miss`;
- exact locked NumPy learning-rate values, bound by decimal and `float.hex()`.

Issue #1050 deliberately ran source index 1 first, followed only on a miss by
indices 0, 2, and 3:

1. `0.00046415888336127773` (`0x1.e6b4b396428e5p-12`);
2. `0.0001` (`0x1.a36e2eb1c432dp-14`);
3. `0.002154434690031882` (`0x1.1a62d511f2b4fp-9`);
4. `0.01` (`0x1.47ae147ae147bp-7`).

The credited #1047 model port has two declared integration exceptions: CPU
placement replaces the source trainer's unconditional CUDA placement, and
query hidden states are gathered before the tied vocabulary projection. The
latter avoids materializing a batch-512 full-position vocabulary tensor; a
direct full-versus-selected loss and gradient test passed. This is an exact
source-configuration and training-semantics reproduction, not byte-identical
upstream CUDA execution.

## Deterministic population and C0

The released population independently rebuilt to identical container and
tensor identities:

- data file CID:
  `blake3:f6dd39f9e0554df7409ee051e353798b89de8047d9f3ce32b983fa83623754b8`;
- tensor CID:
  `blake3:c8afd0c69ccde4fb4e0c4a7a225e70c5b8eaceec113c40c44dfedd9ec5a77d34`.

The two seeded splits contain 100,000 and 3,000 unique full rows with zero
exact full-row overlap. They are not assignment-disjoint: the held-out split
contains 11,998 unique K/V pairs, 294 of which also occur in training. This
split is held out from gradient updates but is evaluated every epoch to drive
the source early stop; it is not sealed terminal evaluation. This record
therefore makes only the held-out-row claim above. A post-result read-only
partition scored `291/294` recurring-pair queries and `11,609/11,706` queries
whose exact K/V pair was absent from training. The partition did not change
the frozen verdict.

Preparation bound implementation CID
`blake3:45f3d99fafec3cba380425f5be82846d6d8ae8c9a1ede426b90b90478991d4d9`,
tree CID
`blake3:d81405407eb4e6e4f420fd8cfde057ff23cf509cd828d0004db8d33f27394cbf`,
and preparation CID
`blake3:bdb9dd01ea0e115eaff54c0536833f2d13cae3d32c9c60cf90070225f031a335`.

C0 passed literal loader and model goldens, exact initialization replay,
causal-prefix and query-only projection parity, deterministic population
replay, and the disposable 32-row overfit at `128/128` queries. The focused
full-versus-query-only loss and gradient test also passed. #1049 reverified
successfully from the unchanged historical package before #1050 ran.

## CPU preflight

Fresh batch-512 subprocesses measured one, four, and eight intra-op threads.
All used Apple Accelerate/OpenMP with one worker and one inter-op thread; CUDA
and MPS were forbidden.

| Threads | Stable | Peak RSS | Safety-adjusted 64-epoch projection |
| ---: | :---: | ---: | ---: |
| 1 | yes | 943,357,952 bytes | 3,390.535 s — ineligible |
| 4 | yes | 983,302,144 bytes | 2,153.006 s — selected |
| 8 | yes | 983,433,216 bytes | 2,483.051 s |

Four threads were measured faster than eight on this M1; selecting four was a
performance decision, not a single-core limitation. Preflight CID is
`blake3:03000b3e10f7181811cf1f4439ff4b2e70b906fe1de835defeb90f90ef527019`.

## Executed result

The first learning-rate arm crossed the strict source threshold at epoch 20:

- test top-1: `11,900/12,000 = 99.1666667%`;
- test NLL: `0.05124610455830892` nats;
- training presentations: 8,000,000 query targets;
- test presentations: 240,000 query targets;
- arm wall: `577.834602 s`;
- total run wall: `578.211792 s`.

The binding transition was visible rather than a marginal threshold event:
test top-1 moved from `26.9167%` at epoch 14 to `49.8333%` at epoch 15,
`72.6500%` at epoch 18, `81.7583%` at epoch 19, and `99.1667%` at epoch 20.
The source rule stopped immediately. The remaining three rates were not run
because the predeclared source early stop fired; they are not failed arms.

Evidence identities are:

- arm CID:
  `blake3:77357ec8e51531c1b21f16270956cd95cf1f68e8ccee886c8fe57c1e99589874`;
- model artifact CID:
  `blake3:163cf3e5375b3e721fa7a826acdb2dfc809e5989209b03fb2a3eea3e3d5459e9`;
- model state CID:
  `blake3:600bdc76cefff79f4be8709197b15252cb531892fad0db2156b36b865c01877e`;
- result CID:
  `blake3:bd16d012c01262ffb8c5197e4cf316c6fee1d722cf0700a0048386180a8122e0`.

Fresh-process structural verification passed. Future-value, role, R4 geometry,
UOR-byte, teacher, and provider reads were all zero.

## Reproduction boundary found in final audit

The observed uninterrupted run, emitted model, checkpoint-to-artifact tensor
bytes, fresh artifact inference, and result envelope all reverified. Two
lifecycle details remain deliberately unclaimed:

- the implementation CID enumerates the Python, test, notice, and license
  files used by the run, but not `pyproject.toml` or `uv.lock`; those dependency
  files ship in the Git revision but are not recursively covered by that CID;
- the rolling checkpoint restores ordinary incomplete epochs exactly, but a
  final audit found a narrow crash window after saving a passing epoch and
  before writing its result. Restarting in that window could advance one extra
  epoch, so arbitrary-interruption/first-pass resume determinism is not claimed.

Neither condition occurred in this run or changes its observed positive
result. The exact-#1045 successor must bind the dependency files and finalize a
checkpointed passing epoch without taking another optimizer step.

## Interpretation and next action

This result establishes that the copied ordinary causal-softmax mechanism can
learn held-out-row key-to-value binding at its released scale. It rules out a
broken copied cell and localizes #1049's miss to the bundle of reduced-versus-
released calibration-contract differences: population and exposure, batch,
shuffle/RNG, learning-rate identity, evaluation ordering, and scheduler
semantics. No ablation isolates which difference caused the miss, and #1049's
immutable result does not change.

The next issue is a single transfer decision: initialize the same positive
cell independently and train it on the exact open #1045 bytes, using the
source-positive batch, shuffle, scheduler, and strict one-evaluation rule.
Require assignment-disjoint development binding before any destructive
binding control. Do not add English, generation, R4 geometry, or W8 lowering
to that issue. A positive C2 authorizes a separately scoped coherent-R4
replacement/transport comparison; a negative localizes the remaining gap to
#1045 serialization/population transfer rather than ordinary attention.

Modulo-256 remains the intended later substrate for categorical roles,
cyclic wheel/table operations, packed projections, and deterministic runtime
lowering. It is not the probability field used by this offline softmax cell.

## Final ledger

| Item | State |
| --- | --- |
| Native #973 parent and #1049 predecessor | bound |
| Exact 100,000/3,000 population | reproduced byte-for-byte |
| Literal-source C0 | passed |
| 1/4/8-thread CPU admission | passed; four threads selected |
| First frozen learning-rate arm | passed at epoch 20 |
| Remaining three learning-rate arms | not run; predeclared source early stop fired |
| Exact-#1045-byte C2 | `NOT_RUN_NEW_ISSUE` |
| R4/geometric comparison | `NOT_RUN` |
| English/generation/reasoning | `NOT_RUN` |
| W8/exact lowering | `NOT_RUN` |
