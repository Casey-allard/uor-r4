# Role-tagged associative-first causal attention (#1045)

- **Status:** executed; stopped at R1 with `OPEN_MQAR_NOT_LEARNED`
- **Policy:** `R4RoleTaggedAssociativeCurriculumV1`
- **Authority:** #1045, child of #973 and programme root #820
- **Predecessor:** #1043, terminal `INVALID_POSITION_KV_BINDING`
- **Claim boundary:** ordinary learned causal-softmax associative attention first;
  English transfer, language preservation, geometric attribution, generation,
  reasoning, compression, and exact lowering remain separate decisions

This is the durable architecture decision and evidence record for #1045.  The
GitHub issue comment posted before implementation freezes the complete open run
contract.  This file records the same decision in-tree and will be appended with
executed results; it does not revise #1043's immutable terminal campaign.

## Decision

Use UOR-Framework's byte-native substrate where modulo-256 semantics are
correct, but retain ordinary numerically stable softmax while establishing the
learning mechanism.

The repository pins UOR-Framework commit
`51c01382200b0179d6640b07e9c8119364ab69a1`, current Framework `main` when
this contract was frozen.  `W8` supplies real wrapping arithmetic in
`Z/256Z`, typed grounding, and deterministic table-friendly identities.  It
does not supply learned Q/K/V attention, vector softmax, log-sum-exp, or an
associative learner.

This is an algebraic boundary, not merely a missing implementation.  Only odd
residues are invertible in `Z/256Z`; no residue `x` satisfies
`2x = 1 (mod 256)`.  Consequently even an equal two-way normalized attention
distribution cannot be represented by division in one W8 residue.  Wrapping
exponentiation also lacks the order and normalization semantics of real
exponentials.

The accepted use in #1045 is therefore a categorical `uint8` role carrier:

| byte | role |
| ---: | --- |
| `0` | `TEXT` or filler |
| `1` | `KEY` |
| `2` | `VALUE` |
| `3` | `QUERY` |

The byte is an identity, not a claim that residue distance or wheel adjacency
encodes semantic distance.  The later lowering candidates, after learning
works, are the exact 256-class stage-code domain, table/coded projections,
packed shift-add terms, binary-VSA bytes, and bytecode operands.  W8 wrapping
is not substituted for 288-wide Hamming totals, signed score accumulators,
saturating `ScoreQ`, Q29 phase arithmetic, counters, or noncommutative H4
multiplication.

## Why the predecessor is not an attention falsifier

#1043 mixed natural language, MQAR, English history, and easy no-history
abstention from its first optimizer update.  It presented only 10,920 MQAR
sequences once: 87,360 supervised query decisions.  Construction MQAR ended at
`30/87,360`, while no-history ended at `2,730/2,730`.  The terminal result is
valid for that frozen recipe, but the recipe did not reproduce the exposure or
curriculum of a known working learned-attention baseline.

#1045 copied the relevant Zoology *curriculum and optimizer shape* before
introducing another UOR-specific mechanism: two causal-softmax layers,
query-only cross-entropy, AdamW, cosine decay, repeated associative-only epochs,
open development accuracy above 99%, then later populations.  It was not a
stock Zoology reproduction.  In particular, it retained the inherited UOR
four-head RoPE/SwiGLU cell, a 48-wide initialized language model, four-slot
records with filler noise, an 8,192-row construction set, and a different
vocabulary/context.  The released Zoology Figure 2 experiment instead uses its
own one-head learned-position attention cell, identity state mixer, two-token
K/V serialization, zero filler, 100,000 construction examples, and a declared
width/learning-rate grid.  Tinylang independently supports visible causal
key/value/query serialization and query-only supervision.  No attention-link
auxiliary loss, alternate optimizer, or hyperparameter sweep was added here.

Primary precedent:

- [Released ICLR24 Figure 2 configuration](https://github.com/HazyResearch/zoology/blob/de4e258784224e09909c257ff3ea040f089ed660/zoology/experiments/paper/figure2.py)
- [Released Zoology MQAR loader](https://github.com/HazyResearch/zoology/blob/de4e258784224e09909c257ff3ea040f089ed660/zoology/data/associative_recall.py)
- [Released Zoology causal attention](https://github.com/HazyResearch/zoology/blob/de4e258784224e09909c257ff3ea040f089ed660/zoology/mixers/attention.py)
- [Released Zoology optimizer and early stop](https://github.com/HazyResearch/zoology/blob/de4e258784224e09909c257ff3ea040f089ed660/zoology/train.py)
- [Tinylang associative serialization and loss](https://github.com/aryamanarora/tinylang/blob/50c9434001e1880cb94eb3f914ef9666b67c477a/tinylang/language/ar.py#L31-L101)

## Mechanism

The new policy wraps the frozen #1043 position-preserving mechanism without
editing it.  It retains its two layers, four heads, learned Q/K/V/O, RoPE,
max-subtracted causal softmax, exact per-position K/V cache, coherent R4-frame
execution, and tied vocabulary output.

A zero-initialized `4 x 48` role embedding is added to the token embedding.  It
adds 192 parameters.  The TEXT row remains fixed at zero; therefore an all-TEXT
input at initialization exactly preserves the ordinary artifact's behavior.
Roles are compiled into K/V values, while cache dimensions and the 23,040-f32
state budget stay unchanged.  H4 frames remain derived from token IDs only.

Every role must be reconstructible from the physically admitted serialization
at that position.  Labels, answer values, future tokens, hidden assignment
metadata, #1043's failed fitted artifact, and `evaluation/sealed/*` are
forbidden model inputs.  Matched and unsupported queries receive the same
QUERY role; there is no answer-revealing ABSTAIN bit.

Training projects to vocabulary logits only at labelled query/readout
positions.  This is the same masked query-only cross-entropy as a full
`[batch,time,vocabulary]` tensor, without computing unused vocabulary rows.

## Open ladder and decisions

The runner stops at the first miss.

### R0 — mechanics and overfit

Use 32 open matched MQAR sequences for at most 256 updates.  Require exact role
reconstruction, prefix invariance under future-suffix/label mutation, zero
future/forbidden/provider/teacher reads, falling loss, and 100% query top-1.

Failure: `OPEN_MECHANICS_OR_OPTIMIZER_FAILURE`; do not launch R1.

### R1 — assignment-disjoint MQAR

Rank the 10,920 open construction rows under a new #1045 BLAKE3 namespace and
use exactly 8,192 train rows, 1,024 development rows, and the remainder only for
open controls.  Validate that no key/value assignment crosses partitions.

Train only eight-record/eight-query MQAR for at most 64 complete epochs.  Use
AdamW (`lr=4.64e-4`, default betas/epsilon, `weight_decay=0.1`), gradient clip
1.0, and epoch-level cosine annealing to zero.  Evaluate every epoch and stop
after two consecutive epochs with train at least 99.5% and development at least
99.0%.  The exposure cap is 4,194,304 query presentations.

After a primary plain-attention pass, run role-off, attention-off, current-only,
value-permuted, and binding-permuted controls.  Native development must be at
least 99%; current-only, value-permuted, and binding-permuted must each lose at
least 50 percentage points.  A role-off drop of at least 25 points is required
only to attribute benefit to role tags; its absence does not erase a valid
ordinary-attention result.

Failure: `OPEN_MQAR_NOT_LEARNED`; next action is the minimal stock Zoology cell
and loader as an integration control.  Success advances to English but does not
authorize generation.

### R2 — English transfer

Train supported English-history rows first with an `H,H,H,M` update cycle and
assignment/template-disjoint open development.  Cap at 2,048 updates.  Require
history at least 90%, binding-permuted drop at least 35 points, and MQAR at
least 98%.

Only after that pass, add shape-matched unsupported queries containing unrelated
visible facts.  Require supported and UNKNOWN accuracy each at least 90%, zero
unsupported assigned-value guesses, and MQAR at least 98%.  Empty-history
abstention is not accepted as evidence.

Failure: `OPEN_ENGLISH_TRANSFER_MISS`.

### R3 — natural-language preservation

Measure the fitted model against initialization on 2,048 open construction
windows and 256 open development windows.  If it is already within +0.05 nat
NLL and -1 percentage point top-1, do not train more.  Otherwise permit at most
512 low-learning-rate natural replay updates.  Every passed binding capability
may regress by at most one point.

Before R2 and any R3 mixed preservation, record component gradient norms, dots,
and cosines on 16 fixed open calibration batches, globally and by parameter
group.  A protected objective with nonpositive median `g_i dot g_total` or more
than 25% nonpositive cases stops `OPEN_OBJECTIVE_CONFLICT`; optimizer invention
is outside #1045.

Only after every open rung passes may a small mechanics fixture compare
full/incremental and plain/coherent-R4 execution.  It uses exact top-1 and a
scale/ULP-aware forward-error report; it does not modify #1043's frozen
threshold or claim intrinsic H4 superiority.

## Long-run contract

    metric to move:       open assignment-disjoint MQAR top-1, from 30/87,360 to >=99%
    reachability ceiling: every label admits its prior K/V state; possible movement is 99.966pp
    cheap instrument:     R0 role/leak/overfit plus CPU thread-and-batch timing
    proceed only if:      R0 passes; projected R1 <=1,800s and peak RSS <=16GiB
    if positive:          advance to English transfer and preservation
    if negative:          stop and port the stock Zoology MQAR integration control
    cost estimate:        R0/preflight <=5m; R1 <=30m; later rungs conditional

The preflight measures Apple Accelerate CPU plans with 1, 4, and 8 intra-op
threads and practical query-only batches.  It chooses measured throughput, not
nominal core count.  CUDA and MPS are forbidden.

## Input and output ledger

Allowed inputs:

- `inputs/ordinary-initialization.safetensors`
- `inputs/r4-group-address-geometry.json`
- `inputs/h4-spin-frame-sidecar.json`
- the public #1043 data manifest and terminal commitment hashes (never the
  sealed payloads)
- the manifest-bound tokenizer and open `construction/*`
- source code and the frozen #1045 contract

Forbidden inputs and actions:

- #1043 `artifact/model.safetensors`
- #1043 `evaluation/sealed/*`
- any terminal reveal/rescore/retry
- teachers, providers, Ollama, Gemma, CUDA, or MPS
- generation, reasoning, recurrence compression, softmax replacement,
  quantization, E8/table lowering, browser/WASM, release, or product claims

## Executed evidence

The implementation tree was frozen as
`blake3:4f2f6b814772b62ac2d4ab4da464df4cbd48c930cd7871a9fb464ba8efdd8066`.
Preparation produced
`blake3:d26d2d8a607618210eafc439cf6885ad6baa46b01a19363a0be64fe1b749ee7c`;
the open assignment-disjoint split contained 8,192 construction rows, 1,024
development rows, and 1,704 control rows under
`blake3:d36937f974e5e96dc697b219db8a7eb448dff7192abdf88bf6b21000f58b1f48`.

R0 passed twice deterministically at `256/256` query top-1 with NLL
`0.03103681` and zero leakage.  The Apple-CPU preflight
`blake3:25f1db6d37a449b1b719938d3aaeed064976ad832bb76453af964086fb0b9e4b`
measured the declared 1-, 4-, and 8-thread plans and selected
`cpu-accelerate-8t-b64`: eight CPU threads, one worker, batch 64, projected
1,714.482 seconds and approximately 669 MiB RSS.  CUDA and MPS remained
forbidden.

R1 completed all 64 epochs and 4,194,304 query presentations in 966.749
seconds.  The final construction evaluation reached `65,500/65,536`
(`99.945068%`, NLL `0.2030258`), so the construction gate passed.  The final
assignment-disjoint development evaluation reached only `7,137/8,192`
(`87.121582%`, NLL `1.1712778`).  The best open development checkpoint was
epoch 53 at `7,162/8,192` (`87.426758%`, NLL `1.1540178`).  It therefore never
produced the required two consecutive 99% development passes.

The independently verified result is
`blake3:d920ad7b7f373c55cb564e27b3ddb1af8949a20c432e0d7cd2b39f1f69999557`;
the fitted artifact is
`blake3:92bb13caf71c9ef44885a9da39023d080de075118b5902b716d2ca9b0f61f611`
(1,011,656 bytes).  The work ledger records zero forbidden reads, future
reads, provider calls, teacher calls, failed source-artifact reads, and sealed
input reads.

Verdict: `OPEN_MQAR_NOT_LEARNED`.  This is not a dead optimizer or dead
attention path: the near-perfect construction fit shows that the plain
causal-softmax mechanism can represent the seen assignments.  It did not meet
the frozen generalization criterion, and it is not coherent-R4/geometric
attention evidence (`transported_r4_blocks = 0`).  English transfer, natural
preservation, generation, reasoning, and lowering remain `NOT_RUN`.

The primary miss stopped the destructive controls as declared:
`native = NOT_RUN_PRIMARY_MISS`; current-only, role-off, value-permuted, and
binding-permuted are `NOT_RUN_NATIVE_MISS`.  The frozen inherited mechanics
could not construct an attention-disabled arm, so
`attention_off = UNAVAILABLE_FROZEN_MECHANICS`.  That unavailable control does
not alter the negative primary verdict, but it prevents an attention-specific
attribution claim.

The predeclared divergent action is now binding: do not tune or rerun #1045.
Port the released ICLR24 Zoology MQAR loader and causal-softmax cell as a new,
credited integration control.  First establish that exact stock path under a
bounded CPU-only contract; then put the same stock cell on the #1045 open
serialization to distinguish a UOR-cell limitation from a data-contract
limitation.  Only a generalizing positive can advance to English transfer.
