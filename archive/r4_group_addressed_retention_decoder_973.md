# #973 R4 group-addressed retention decoder

Status: **TERMINAL UNAVAILABLE ON MPS / OPTIMIZATION NOT RUN**

Issue: [#973](https://github.com/UOR-Foundation/uor-r4/issues/973)

Authoritative pre-run freeze:
[issue comment 5489101447](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5489101447)

Mechanism: `R4GroupAddressedRetentionDecoderV1`

## Decision this contract owns

The predecessor `R4GroupAddressedRetentionLMV1` reached a valid construction
terminal after its geometry, population, reachability, gradient, memory, and
equal-work checks passed but its timing and disposable learning checks failed.
Its result is preserved in the
[#973 predecessor record](r4_group_addressed_retention_973.md). That exact
embedding-plus-one-cell mechanism is retired without tuning.

This independent freeze asks one narrower successor question: does adding a
complete residual language-model path around group-addressed fixed-state
attention make the retained state learnably and causally useful on disjoint
construction data, and does exact H4 transport add value beyond an equal-work
destructive transport control?

The pre-run contract below is preserved verbatim as the authority for the
create-once MPS attempt. Its terminal result is recorded after the frozen
decision section.

## Frozen predecessor identities

The immutable predecessor root is `issue-973-group-retention`. The successor
must reproduce all of these identities before it may write a run-start marker:

- training-view CID:
  `blake3:ce26777ed9fa8d25410b3f27acf30a0b33d9d725d9d3fb1e614137bf91581f31`;
- population CID:
  `blake3:35af5002bfbe92d68403e2cf8742fae4a22b7d6b11109a3f861fab9e15d2b52e`;
- fit-store CID:
  `blake3:3ce77ac0b15dd3173add6382dd070016a880e8258821951a9ba9bbffa03ea43c`;
- fit-index CID:
  `blake3:73ba637e007c404ab19084ddf627b4082c4d5ab93fe468dbe42b087e29d9c12b`;
- tokenizer CID:
  `blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc`;
  and
- geometry-artifact CID:
  `blake3:55447c00c1eb86a1d05324d6c83d044407bdc89f653f46957bf6f0bccb6c000b`.

The loader must verify the predecessor training view and physical seal without
naming or opening a held-out token path.

## Frozen architecture

**Definition.** Let `G` be the supplied 120-address group with identity `e`,
`P_a` the supplied action for the observed token leaf `a`, `D = 288`, four
heads of width `d = 72`, and two decoder blocks. The vocabulary is 4,096,
context is 128, and each block has a 768-wide SwiGLU channel mixer.

The model contains:

- one genuinely shared `4096 x 288` token-embedding/output-head weight;
- exactly two pre-norm decoder blocks;
- bias-free `Q`, `K`, `V`, and `O` projections in each block;
- one RMSNorm before retained attention and one before SwiGLU in each block;
- residual additions after retained attention and after SwiGLU;
- a final RMSNorm before the tied output head;
- no dropout and no RoPE; and
- per block, separate key and value fields of shape
  `[batch, 4, 120, 72]` plus one shared 120-bit logical occupancy mask.

The model imports no donor weights, teacher trace, Transformer implementation,
token-position attention matrix, or KV cache.

### Retained-attention recurrence

**Definition.** For block `l`, head `h`, token time `t`, and address `g`, let
`x_t^l` be the residual-stream input and define

```text
u_t^l = RMSNorm_attn^l(x_t^l)
q_t^l, k_t^l, v_t^l = split_heads(W_q^l u_t^l,
                                  W_k^l u_t^l,
                                  W_v^l u_t^l)

K_bar[t,l,h](g) = rho[l,h] * K[t-1,l,h](P_a_t(g))
V_bar[t,l,h](g) = rho[l,h] * V[t-1,l,h](P_a_t(g))
O_bar[t,l](g)   = O[t-1,l](P_a_t(g))
```

The read occurs before the current write. Every head scores all 120 fixed
addresses, masking unoccupied addresses:

```text
s[t,l,h](g) = dot(q_t^l[h], K_bar[t,l,h](g)) / sqrt(72)
w[t,l,h]    = stable_softmax_g(s[t,l,h], mask=O_bar[t,l])
r_t^l[h]    = sum_g w[t,l,h](g) * V_bar[t,l,h](g)
a_t^l       = W_o^l concat_h(r_t^l[h])
```

An empty occupancy mask returns a zero retained read. It does not create an
implicit occupied address. The state-off intervention executes the same
transport, decay, projections, 120-address scoring, stable softmax, aggregation,
output projection, and write. It changes only the scalar immediately before
the retained residual addition:

```text
y_t^l       = x_t^l + enabled * a_t^l       # enabled is 1 or 0
m_t^l       = RMSNorm_mlp^l(y_t^l)
x_t^(l+1)   = y_t^l
             + W_down^l(silu(W_gate^l m_t^l) * W_up^l m_t^l)
```

After the read, the current key and value delta-write at the exact identity
address:

```text
K[t,l,h](g) = K_bar[t,l,h](g)
              + 1[g=e] * eta[l,h]
                * (k_t^l[h] - K_bar[t,l,h](e))
V[t,l,h](g) = V_bar[t,l,h](g)
              + 1[g=e] * eta[l,h]
                * (v_t^l[h] - V_bar[t,l,h](e))
O[t,l](g)   = O_bar[t,l](g) or 1[g=e]
```

The four decay gates initialize to half-lives `[4, 16, 64, 256]` tokens, so
their initial retained factors are `rho[h] = 2^(-1 / half_life[h])`. Write
logits initialize to zero, giving `eta[h] = 0.5`. Both gate families remain
learned. Final logits use the same storage as the input embedding:

```text
logits_t = E * RMSNorm_final(x_t^2), where E is the token embedding
```

Compiler-side training may use ordinary floating-point Apple MPS operations and
stable softmax. This is fixed-state geometric softmax attention over addresses;
it is not a claim of softmax replacement or exact-runtime lowering.

## Exact parameter, state, and work boundary

**Definition.** With vocabulary `V = 4096`, residual width `D = 288`, two
layers `L = 2`, mixer width `F = 768`, and four heads `H = 4`, the tied-head
trainable count is

```text
V*D + L*(4*D*D + 3*D*F + 2*D + 2*H) + D
= 3,171,760 parameters.
```

The terms are the shared embedding/head; per-layer Q/K/V/O projections,
SwiGLU projections, two RMSNorm weights, and eight learned decay/write gates;
and the final RMSNorm. A separately parameterized output matrix is forbidden.

**Definition.** Learned recurrent state per sequence is

```text
2 layers * 2 fields * 4 heads * 120 addresses * 72 values
= 138,240 f32 values
= 552,960 bytes,
```

plus exactly two 120-bit logical occupancy masks. Parameter count and recurrent
state are independent of prefix length.

The two optimizer arms are exact H4 and the already frozen identity-fixing,
non-homomorphic scrambled-H4 action. They must have byte-identical learned
initialization and identical parameters, recurrent-state shape, optimizer,
batches, token presentations, route reads, attention-slot reads, and output
work. State-on and state-off execute the same work ledger. C120 remains a
mechanical equal-ledger check and is not a third optimizer run.

## Construction-only population

Prior disposable-smoke ordinals `0..7` are excluded. From the immutable fit
partition:

- construction training uses story ordinals `8..39`;
- disjoint construction validation uses ordinals `40..71`; and
- each story contributes its first 129 tokens, yielding 128 causal decisions,
  32 stories, and 4,096 decisions per partition.

Exactly 4,064 of the 4,096 validation decisions have at least one prior write
available, a 99.21875% reachability ceiling. The first decision of each story is
necessarily memory-empty. The reused geometry already generates `120/120`
states in exact H4, C120, and scrambled-H4 arms. No other fit story and no
held-out story participates in this construction selection.

## Cheap instrument and admission

Before optimization, require all of:

- the exact parameter and state counts above;
- genuine tied storage rather than merely equal embedding/head values;
- byte-identical learned initialization between trained arms;
- full-sequence versus incremental parity for logits, final key/value fields,
  occupancy, and gradients;
- strict prefix causality;
- finite nonzero gradients through the embedding, both retained-attention
  blocks, every Q/K/V/O projection, both gate families, norms, SwiGLU, residual
  stream, and tied head;
- equal work ledgers across exact H4, scrambled H4, C120, and state-on/off; and
- one warm-up plus three measured MPS steps, with peak memory below the device
  recommendation and
  `1.25 * mean_step_seconds * 512 <= 600 seconds`.

A mechanical, memory, or wall-clock miss stops
`UNAVAILABLE_FULLER_DECODER_CONSTRUCTION`. It is not a scientific model
verdict and does not authorize a longer run.

## Frozen optimization

If and only if the cheap instrument passes, train exact H4 and scrambled H4
sequentially on Apple MPS:

- seed `9737`;
- AdamW with constant learning rate `0.003`, betas `0.9/0.95`, epsilon `1e-8`,
  weight decay `0`, and gradient clip `1`;
- batch `8`, context `128`, and deterministic cyclic four-batch order;
- exactly 256 steps per arm;
- 262,144 token presentations per arm and 524,288 total; and
- a 600-second hard wall.

There is no sweep, retry, alternate seed, checkpoint continuation, CPU fallback,
CUDA execution, or held-out read. The estimated cost is 3–7 minutes on the
current M1 after the cheap timing gate, with a strict ten-minute process ceiling.

## Frozen decisions

**Empirical Criterion — `RETAINED_DECODER_PASS`.** Require all of:

- both trained arms reduce construction-training cross-entropy by at least 50%;
- exact H4 improves its disjoint construction-validation cross-entropy by at
  least `0.10` nats;
- disabling the trained exact-H4 retained state worsens construction-validation
  cross-entropy by at least `0.05` nats;
- exact-H4 state-on gains at least 11 of 4,096 top-1 decisions over state-off;
  and
- all causal, finite, replay, and equal-work conditions pass.

**Empirical Criterion — nested `H4_SPECIFIC_PASS`.** After
`RETAINED_DECODER_PASS`, exact H4 must additionally beat the independently
trained scrambled-H4 arm by at least `0.02` construction-validation nats and 11
of 4,096 net top-1 decisions.

If retained-decoder and H4-specific criteria pass, preserve the fitted
source-free decoder baseline and independently freeze one full-fit/held-out
natural-language promotion campaign. If retained decoder passes but H4
specificity misses, preserve the working recurrent-attention decoder without an
H4-advantage claim and repair only the geometry seam. Neither branch opens
held-out data under this preflight.

If the scientific retained-decoder criterion fails, retire this exact two-layer
update/read law. Do not scale it, lower it, or add route families; select a
different decoder mechanism under a new independent freeze.

## Nonclaims and current evidence

Construction output, order sensitivity, and state use can support only this
bounded decoder-selection decision. Even a complete construction pass would not
establish coherent autonomous generation, general attention transfer, inference
quality, reasoning, correctness, exact integer or shift-add runtime, H4/E8
superiority in general, #954 completion, browser/WASM integration, release
readiness, or frontier capability.

At the terminal, optimization, scientific construction evaluation, checkpoint,
fitted-artifact publication, autonomous generation, held-out evaluation, and
every downstream stage remained `NOT_RUN`.

## Official create-once result

The sole MPS attempt stopped
`UNAVAILABLE_FULLER_DECODER_CONSTRUCTION` before optimization. Its exact
[started envelope](r4_group_addressed_retention_decoder_preflight_started_973_raw.json)
and [result envelope](r4_group_addressed_retention_decoder_preflight_result_973_raw.json)
are preserved byte-for-byte. The started CID is
`blake3:7dcc8d1f1f39352edaa16635094f240288d5f87d541974e02423e4e6d8399069`;
the result CID is
`blake3:aef070691138c7a333d84c0b25437abf3e7d8dc87b3244ab7b6acfff89e73a5b`.

All non-timing mechanical conditions passed:

- full-sequence/incremental maximum deltas were `1.549721e-6` for logits,
  `5.513430e-7` for final state, and `7.152557e-7` for gradients;
- the complete shared-prefix causality delta was exactly zero;
- every required learned gradient was finite and nonzero;
- exact H4, scrambled H4, C120, and state-off work signatures were identical;
  and
- peak MPS allocation was `4,732,616,704` bytes, below the device recommendation
  of `12,713,115,648` bytes.

The exact-H4 and scrambled-H4 timing means were `1.2602548470` and
`1.2431917084` seconds per step. Their equal-weight mean was
`1.2517232777`; the frozen formula therefore projected
`1.25 * 1.2517232777 * 512 = 801.1028977` seconds, above the `600`-second
ceiling. The harness stopped after `19.904245` seconds with optimization
`NOT_RUN`, zero held-out reads, no fitted artifacts, no main/reveal command,
and H4 specificity `NOT_EVALUATED`.

This is an execution-budget result, not a retained-decoder negative. It does
not activate the scientific retirement branch above and supplies no evidence
against geometric attention. A post-terminal backend audit measured the same
deterministic step at about `0.820` seconds on Apple Accelerate with four
configured PyTorch/Accelerate threads, versus about `1.258` seconds on
deterministic MPS;
eight CPU threads and two concurrent workers were both slower. Those
diagnostics are not part of the scientific result. The next authorized action
is a separately named resource-only successor: identical population, model,
initialization, optimizer dose, controls, and scientific thresholds, but the
measured-fast deterministic four-thread CPU/Accelerate execution plan.

## CPU-recovery follow-up — 2026-09-01

The separately frozen
[`R4GroupAddressedRetentionDecoderV1CpuRecovery`](r4_group_addressed_retention_decoder_cpu_recovery_973.md)
used the identical population, model, initialization, optimizer dose, controls,
and scientific thresholds with the measured-fast deterministic four-thread
CPU/Accelerate execution plan. It completed all 512 steps in `438.117083`
seconds. On the disjoint construction-validation partition, state-off lost
`0.967227` nats and `182/4096` state-on top-1 decisions. The whole-model
validation CE worsened by `0.604243` nats, so the exact complete-decoder recipe
did not satisfy its frozen generalization criterion. Formal H4 specificity
remained `NOT_EVALUATED`; diagnostically, scrambled transport CE was `0.033049`
nats better while exact H4 led top-1 by only four decisions, below threshold.
The recovery record is authoritative for that scientific result.
