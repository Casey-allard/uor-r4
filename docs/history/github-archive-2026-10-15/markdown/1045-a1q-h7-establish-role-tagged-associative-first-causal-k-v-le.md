# #1045 — A1Q-H7: establish role-tagged associative-first causal K/V learning

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-02T05:34:58Z
- **Updated:** 2026-09-02T07:16:21Z
- **Closed:** 2026-09-02T07:14:46Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1045
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Authority and outcome

Parent: #973. Programme root: #820. Direct successor to #1043's immutable `INVALID_POSITION_KV_BINDING` terminal.

Establish, on open construction/development data, whether the already implemented position-preserving causal R4 K/V softmax cell can learn supplied associative bindings when role ambiguity and mixed-objective competition are removed. This is a capability-development rung, not a sealed terminal campaign and not authorization for generation, reasoning, recurrence compression, or exact lowering.

## UOR-Framework substrate decision

The current repository already pins live UOR-Framework main `51c01382200b0179d6640b07e9c8119364ab69a1`. Its W8 surface supplies genuine `Z/256Z` wrapping arithmetic, typed byte coordinates, deterministic bindings/provenance, carry vocabulary, and table-friendly identities. It does not supply vector softmax, log-sum-exp, learned attention, associative retrieval, or a hidden optimized backend. `Z/256Z` is not a field, so arbitrary attention denominators are not generally invertible.

This issue therefore:

- retains ordinary numerically stable compiler-side softmax while learning;
- serializes causal role identities as a bounded byte-native field suitable for later W8 grounding/table lookup;
- does not insert modular wrapping into real-valued Q/K/V training, Hamming accumulators, ScoreQ, Q29 phase, or H4 multiplication;
- records the later high-value lowering path: existing uor-matmul exact Q/K/value execution, then E8/coded-table compilation only after retrieval works.

## Mechanism boundary

Implement one `R4RoleTaggedAssociativeCurriculumV1` over the #1043 position-preserving model:

- keep one exact causal K/V record per admitted prefix position;
- keep learned Q/K/V/O, RoPE, stable softmax, H4/R4 frame transport, and candidate-affecting tied output;
- add only an explicit causal role channel with TEXT, KEY, VALUE, and QUERY states;
- derive every role from the observed serialized prefix/structure, never from the answer, future token, evaluation label, or hidden assignment metadata;
- preserve full-prefix/incremental and coherent-R4/plain execution;
- initialize from the frozen ordinary source-free language artifact, never from #1043's failed fitted artifact.

The role carrier is categorical. Residue distance or wheel adjacency is not claimed to encode semantics.

## Open development ladder

Publish one analysis comment before code that fixes the exact dataflow, parameter/state ledger, phase schedule, thresholds, cost, controls, and files.

Then run only these ordered cheap rungs, stopping at the first miss:

1. mechanics and small construction overfit;
2. assignment-disjoint MQAR development transfer under associative-only training;
3. supplied-English binding transfer with history/no-history abstention;
4. add natural-language replay and measure both preservation and gradient conflict.

No unseen terminal population is created or opened in this issue. A later issue may freeze one only if all open rungs pass without label/future leakage.

## Controls and evidence

At each applicable rung report:

- role-enabled versus all-TEXT role-off;
- current-only/history-off;
- value/binding permutation;
- coherent-R4/plain parity and transport mismatch;
- exact full/incremental replay;
- component loss/top-1 and component-gradient cosine before mixed preservation;
- complete source/target/future/forbidden/provider/teacher/work ledger.

A synthetic pass without English transfer does not authorize generation. English transfer with language regression does not authorize generation. Gauge-equivalent R4/plain parity is legitimate implementation evidence, not intrinsic H4 superiority.

## Scope exclusions

Do not:

- rescore, tune, retry, or reinterpret #1043;
- alter #1043's frozen terminal artifacts or thresholds;
- create another sealed campaign before the open ladder passes;
- replace softmax, quantize Q/K/V, add recurrence compression, or begin E8/table lowering;
- start #954 correctness, #955 reasoning, browser/WASM, release, or product promotion;
- add broad permanent test gates or launch an unrelated corpus-scale run;
- use CUDA or a remote GPU.

CPU/Apple Accelerate may use the fastest deterministic eligible thread plan.

## Definition of done

- [ ] Publish the architecture/substrate analysis and immutable open-ladder contract.
- [ ] Implement the one role-tagged model/data/curriculum path with focused mechanics coverage.
- [ ] Execute the ladder only until its first decision-bearing result.
- [ ] Record exact open-development results and the next divergent action.
- [ ] Correct current programme documentation without overstating the claim.
- [ ] Deliver through one focused PR and the protected merge queue.


## Comments (3)

### comment 5505082160 — 2026-09-02T05:48:54Z — Casey-allard

## Frozen architecture decision and open run contract

This comment freezes the implementation and decision boundary before code, as required by the issue.

### Decision: use W8 where its algebra is correct

The repository already pins UOR-Framework commit `51c01382200b0179d6640b07e9c8119364ab69a1`, which is current Framework `main`. Its `W8` is genuine wrapping arithmetic in `Z/256Z`; it also provides typed grounding and deterministic table bindings. That is useful here for the **categorical role carrier and later table lowering**.

It is not a substitute for learned softmax. In `Z/256Z`, only odd elements are invertible. Even the equal two-item distribution would require an element `x` satisfying `2x = 1 (mod 256)`, which does not exist. Wrapping exponentiation also does not preserve the order or normalization of real exponential weights. Therefore this issue retains f32 max-subtracted causal softmax during compiler-side learning.

After retrieval works, the prioritized W8/uor-matmul lowering targets are: the exact 256-class stage-code domain, table/coded projections, packed shift-add term bytes, binary-VSA bytes, and bytecode operands. W8 wrapping is explicitly forbidden for Hamming totals (which reach 288), signed score accumulators, saturating `ScoreQ`, Q29 phase arithmetic, token counters, and noncommutative H4 multiplication.

### Why #1043 does not falsify ordinary attention

#1043 mixed natural language, MQAR, English history, and the easy no-history/UNKNOWN objective from update one. It supplied MQAR only `10,920 * 8 = 87,360` supervised query presentations; construction MQAR ended at `30/87,360` while no-history ended at `2,730/2,730`. That is an optimization/curriculum failure, not a clean attention falsifier.

The copied working precedent is Zoology's two-layer learned causal-softmax MQAR recipe: query-only cross-entropy, AdamW, cosine decay, up to 64 epochs, and early stopping only after open validation exceeds 99%. Its published 100,000-sequence configuration permits up to 51.2 million supervised query presentations. Tinylang independently uses visible key/value/query serialization and query-only loss. This issue copies that core recipe before adding any UOR-specific auxiliary loss.

Primary sources:

- Zoology MQAR configuration: https://github.com/HazyResearch/zoology/blob/1ad20d193b6113cae1e8f3c655c300d7b4b3f4bb/zoology/experiments/paper_configs/iclr24_zoology_figure2/configs.py#L11-L61
- Zoology causal attention: https://github.com/HazyResearch/zoology/blob/1ad20d193b6113cae1e8f3c655c300d7b4b3f4bb/zoology/mixers/attention.py#L6-L53
- Zoology optimizer/early stop: https://github.com/HazyResearch/zoology/blob/1ad20d193b6113cae1e8f3c655c300d7b4b3f4bb/zoology/train.py#L182-L207
- Tinylang associative serialization/loss: https://github.com/aryamanarora/tinylang/blob/50c9434001e1880cb94eb3f914ef9666b67c477a/tinylang/language/ar.py#L31-L101

### Exact mechanism and dataflow

Policy: `R4RoleTaggedAssociativeCurriculumV1`.

The frozen #1043 model remains untouched. A new subclass/wrapper reuses its two layers, four heads, learned Q/K/V/O, RoPE, stable softmax, exact per-position K/V cache, coherent R4 frame execution, and tied output.

Inputs are `token_ids: int64[B,T]` plus `roles: uint8[B,T]`:

| byte | role |
|---:|---|
| `0` | TEXT/filler |
| `1` | KEY |
| `2` | VALUE |
| `3` | QUERY |

The role byte is categorical; residue distance and wheel adjacency carry no semantic claim. A `4 x 48` role embedding is added at the embedding boundary. It adds exactly 192 parameters; all four rows initialize to zero, and row 0 remains fixed zero, so all-TEXT initialization is byte-for-byte behavior-equivalent to the ordinary artifact. Cache dimensions and the 23,040-f32 state budget do not change. H4 frame accumulation remains token-derived only.

Roles are reconstructed from the admitted serialized prefix, never from labels, future tokens, answer values, sealed rows, or hidden assignment metadata. MQAR uses the public fixed record/query layout and token ranges. English roles attach only to physically present key/value/query lexemes and the causally visible answer-prompt position. Matched and unmatched queries receive the same QUERY role; there is no ABSTAIN input signal.

Training computes vocabulary logits only at labelled query/readout positions. This is mathematically the same query-only cross-entropy as scoring the full vocabulary tensor and masking unlabelled positions, but avoids wasting memory and CPU on `B*T` unused logits. No attention-link auxiliary loss, PCGrad, hyperparameter sweep, teacher, provider, CUDA, or MPS is allowed.

Initialization may read only:

- `inputs/ordinary-initialization.safetensors`
- `inputs/r4-group-address-geometry.json`
- `inputs/h4-spin-frame-sidecar.json`
- open `construction/*` inputs

It must never read #1043's failed fitted artifact or `evaluation/sealed/*`.

### Ordered open ladder

The runner stops at the first miss.

**R0 — mechanics/overfit.** Use 32 open matched MQAR sequences with a 256-update cap. Require exact causal role reconstruction, full-prefix/future-suffix invariance, zero future/forbidden/provider/teacher reads, falling loss, and 100% query top-1 on the tiny set. A miss stops `OPEN_MECHANICS_OR_OPTIMIZER_FAILURE`.

**R1 — assignment-disjoint full MQAR.** Deterministically rank the 10,920 open construction rows under a new #1045 BLAKE3 namespace; use 8,192 train, 1,024 development, and reserve the remainder for controls. No pair/assignment may cross partitions. Train the inherited model on 8-record/8-query rows only, batch chosen by the CPU preflight, for at most 64 complete epochs. AdamW uses `lr=4.64e-4`, default betas/epsilon, `weight_decay=0.1`, gradient clip `1.0`, and epoch-level cosine annealing to zero. Evaluate every epoch and stop after two consecutive epochs with train >=99.5% and development >=99.0%. The cap is 4,194,304 supervised query presentations at batch-independent exposure.

After a native pass, score all-TEXT role-off, current-only, value-permuted, binding-permuted, and attention-off controls. R1 establishes ordinary learned associative attention only if native development is >=99%, current-only/value-permuted/binding-permuted each lose >=50 percentage points, and all causal ledgers remain zero. Role-off is attribution evidence: if it does not lose >=25 points, attention may pass but no role-tag benefit is claimed.

A miss stops `OPEN_MQAR_NOT_LEARNED`; the divergent next action is a minimal byte-equivalent port of the stock Zoology cell/data loader to distinguish inherited-model integration from learner failure. A pass advances to English; it does not authorize generation.

**R2 — English binding transfer.** First train only shape-supported English-history rows with an alternating `H,H,H,M` update cycle, keeping MQAR replay active. Development groups are disjoint by key/value assignment and template family. Cap at 2,048 updates. Require English-history >=90%, binding-permuted drop >=35 points, and MQAR >=98%.

Only after that pass, introduce shape-matched unsupported queries containing unrelated visible facts; matched and unmatched inputs receive identical role kinds. Require history and UNKNOWN accuracy each >=90%, unsupported assigned-value guesses `0`, and MQAR >=98%. A miss stops `OPEN_ENGLISH_TRANSFER_MISS`; no empty-history shortcut is accepted.

**R3 — natural-language preservation.** Measure the ordinary initialization and fitted model first on 2,048 open construction windows plus 256 open development windows. If the fitted model is already within `+0.05` nat NLL and `-1pp` top-1, do not train more. Otherwise permit at most 512 low-LR natural replay updates, interleaved as separate population steps rather than a summed mixed loss. Require the language bounds and <=1pp regression on every passed association gate.

Before R2 and before any R3 mixed preservation, compute separate component gradients on 16 fixed open calibration batches and record norms, dots, and cosines globally and by embeddings, Q/K/V/O, gains, and MLP/norm groups. If a protected objective has nonpositive median `g_i dot g_total` or more than 25% nonpositive calibration cases, stop `OPEN_OBJECTIVE_CONFLICT`; this issue does not invent another optimizer.

After all open learning rungs pass, run only a small mechanics fixture for exact top-1 full/incremental replay and plain/coherent-R4 parity. Numerical comparison is ULP/scale-aware; no retroactive change is made to #1043's frozen absolute threshold. Geometry-destroying transport mismatch is reported but H4 superiority is not required because the coherent R4 path is a change of gauge.

### Run contract

    metric to move:       open assignment-disjoint MQAR top-1, from #1043 construction 30/87,360 to >=99%
    reachability ceiling: every R1 label has admitted prior K/V state, so 100% of decisions reach attention; ceiling is 99.966pp from the observed construction rate
    cheap instrument:     R0 exact-role/leakage/overfit plus 1/4/8-thread and eligible-batch CPU timing
    proceed only if:      R0 passes and projected R1 wall <=1,800 seconds with peak RSS <=16 GiB
    if positive:          advance to English transfer, then preservation; still no generation
    if negative:          stop and port the stock Zoology MQAR cell as the integration control
    cost estimate:        R0/preflight <=5 minutes; R1 <=30 minutes hard wall; later rungs run only after prior passes

The timing probe compares Apple Accelerate CPU plans with 1, 4, and 8 intra-op threads and practical query-only batches; it selects the fastest deterministic eligible plan. The prior live probe found 4 threads faster than both 1 and 8, so core count is measured rather than assumed. Independent deterministic repetitions may run as separate workers only when memory permits.

### Claim boundary and files

Expected focused additions:

- `role_tagged_associative.py`
- `role_tagged_associative_data.py`
- `role_tagged_associative_development.py`
- matching focused tests, minimal CLI registration, and `docs/r4_role_tagged_associative_curriculum_1045.md`

No #1043 implementation, terminal artifact, threshold, or result is modified. No sealed population, generation, reasoning, recurrence compression, softmax replacement, quantization, E8/table lowering, browser/WASM, release, or broad permanent gate is in scope.


### comment 5505854242 — 2026-09-02T07:09:09Z — Casey-allard

## Executed R1 result — stop `OPEN_MQAR_NOT_LEARNED`

The frozen open run completed and was independently re-verified from its immutable result file.

### Frozen identities and CPU plan

- implementation tree: `blake3:4f2f6b814772b62ac2d4ab4da464df4cbd48c930cd7871a9fb464ba8efdd8066`
- preparation: `blake3:d26d2d8a607618210eafc439cf6885ad6baa46b01a19363a0be64fe1b749ee7c`
- open split: `8,192` train / `1,024` development / `1,704` controls, `blake3:d36937f974e5e96dc697b219db8a7eb448dff7192abdf88bf6b21000f58b1f48`
- preflight: `blake3:25f1db6d37a449b1b719938d3aaeed064976ad832bb76453af964086fb0b9e4b`
- selected plan: Apple Accelerate CPU, 8 threads, one worker, batch 64; CUDA/MPS forbidden
- R0: deterministic `256/256` top-1 twice, NLL `0.03103681`, zero leakage

### R1

- elapsed: `966.749s`
- exposure: 64 complete epochs, `4,194,304` query presentations
- final construction: `65,500/65,536 = 99.945068%`, NLL `0.2030258` — train gate PASS
- final assignment-disjoint development: `7,137/8,192 = 87.121582%`, NLL `1.1712778` — development gate FAIL
- best development: epoch 53, `7,162/8,192 = 87.426758%`, NLL `1.1540178`
- two consecutive 99% development passes: FAIL
- result: `blake3:d920ad7b7f373c55cb564e27b3ddb1af8949a20c432e0d7cd2b39f1f69999557`
- artifact: `blake3:92bb13caf71c9ef44885a9da39023d080de075118b5902b716d2ca9b0f61f611` (`1,011,656` bytes)
- causal/source audit: zero forbidden, future, provider, teacher, failed source-artifact, and sealed-input reads

Verdict: **`OPEN_MQAR_NOT_LEARNED`**. The near-perfect construction fit shows that the optimizer and plain causal-softmax path are active and can represent seen assignments, but this frozen inherited UOR cell did not meet the assignment-disjoint generalization criterion. This execution used plain attention (`transported_r4_blocks = 0`), so it is not coherent-R4/geometric-attention evidence and does not establish generation or reasoning.

The primary miss stopped the destructive controls as frozen: `native = NOT_RUN_PRIMARY_MISS`; current-only, role-off, value-permuted, and binding-permuted are `NOT_RUN_NATIVE_MISS`. `attention_off = UNAVAILABLE_FROZEN_MECHANICS`; that does not change the negative primary verdict, but it prevents attention-specific attribution.

### Correction and binding next action

#1045 borrowed Zoology's curriculum/optimizer shape; it was **not** a stock Zoology reproduction. It retained UOR's four-head RoPE/SwiGLU 48-wide initialized language cell, four-slot noisy serialization, smaller vocabulary, and smaller repeated population. The released ICLR24 Zoology Figure 2 arm uses its own one-head learned-position cell, identity state mixer, two-token K/V serialization, zero fillers, 100,000 construction examples, and declared width/learning-rate grid.

Per the predeclared negative branch: do not tune or rerun #1045. The next issue will port the released Apache-2.0 Zoology loader and causal-softmax cell as a credited CPU-only integration control, first on its stock data contract and then on the #1045 open serialization. That will distinguish an inherited UOR-cell limitation from a data-contract limitation before any further geometric mechanism is proposed.


### comment 5505927685 — 2026-09-02T07:16:21Z — Casey-allard

## Read-only failure-localization audit

After the frozen verdict, a no-write audit replayed all 8,192 assignment-disjoint development queries from the bound artifact and split. It reproduced the exact `7,137/8,192` score and emitted canonical diagnostic CID:

`blake3:ce11698f62561afb6d8ee5e8f816474df389802559d5e6519bff498c735b7736`

Bound inputs:

- result: `blake3:d920ad7b7f373c55cb564e27b3ddb1af8949a20c432e0d7cd2b39f1f69999557`
- artifact/file: `blake3:92bb13caf71c9ef44885a9da39023d080de075118b5902b716d2ca9b0f61f611`
- open split: `blake3:d36937f974e5e96dc697b219db8a7eb448dff7192abdf88bf6b21000f58b1f48`

Findings:

- errors: `1,055`
- wrong prediction was another physically admitted value from the same row: `893/1,055` (`84.6%`)
- wrong prediction was outside the row: `162/1,055`
- in zero-based second-layer heads 1 and 2, the true value slot was the top source for `7,193/8,192` and `7,209/8,192` queries
- on `6,975/7,137` correct outputs, both heads top-ranked the true value slot
- on `739/893` wrong in-row outputs, both heads top-ranked the slot holding the wrongly predicted value

Interpretation boundary: this is strong localization evidence for unseen-key routing/matching error rather than an absent optimizer, dead attention path, or vocabulary decoder. It is not a causal ablation and does not establish geometric attention. The inherited model divides width 48 across four 12-dimensional heads; the released Zoology control uses one 64-dimensional head. That makes the next control's head geometry a directly motivated variable, not a speculative width sweep.



---
_Comment/review ids in this file: 5505082160, 5505854242, 5505927685_

