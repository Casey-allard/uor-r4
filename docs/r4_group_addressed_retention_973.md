# #973 R4 group-addressed retention language model

Status: **CONSTRUCTION TERMINAL `UNAVAILABLE_FRAME_POPULATION_OR_LOCAL_BUDGET` / HELD-OUT NOT RUN**

Issue: [#973](https://github.com/UOR-Foundation/uor-r4/issues/973)

Live independent freeze: [issue comment 5488383326](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5488383326)

Pre-training leaf correction: [issue comment 5488480068](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5488480068)
Mechanism: `R4GroupAddressedRetentionLMV1`

## Decision this work owns

The working #1014/#1017 baseline already establishes ordinary learned causal
Q/K/V stable-softmax attention inside coherent R4/Spin frames. It is
source-backed at execution. The source-free suffix student looped, and its
recurrent trace-state successor changed no actual-next or teacher top-1
decision. C1-SB5 also closed negative and does not authorize a C1-SB6.

#973 now owns one different question: can a fixed-size, source-free geometric
state beneficially change natural-language next-token logits, and does the
exact H4 group law contribute beyond equal-budget non-H4 retained state?

This record is append-only. Until a phase below has an observed result and
bound artifact identity, it remains `NOT_RUN`.

## Why the old matrix recurrence is not resumed

A coherently encoded and decoded `P S P^T` memory is algebraically a gauge
reparameterization of a fixed-frame delta cell. It cannot by itself identify a
geometric advantage. The historical #973 token atlas also draws from four
Hurwitz-unit leaves, so its reachable subgroup must not be compared with a
nominal 120-state control as though both arms had equal reachable capacity.

The active mechanism instead uses a new, explicitly versioned token-leaf
artifact derived from the already normative S0 route rule:

```text
BOS          -> exact H4 identity
token t > 0  -> H4[first_primes[t - 1] mod 120]
```

This enumerates primes over lexical tokens rather than consuming `p_0 = 2`
for BOS and then replacing it with identity. The exact Rust exporter must prove
that those leaves generate all 120 H4 states before any optimizer is admitted.

## Frozen cell

Let `G` be the exact 120-element binary-icosahedral/H4 multiplication table,
`e` its actual identity offset, and `D = 288 = 72 * 4`. Four banks store one R4
field over `G`:

```text
R[t,b](h) = M[t-1,b](leaf(x_t) * h)
D[t,b](h) = rho[b] * R[t,b](h)
M[t,b](h) = D[t,b](h)
              + 1[h=e] * eta[b] * (V[x_t] - D[t,b](e))
A[t](h) = sum_b softmax(alpha)[b] * M[t,b](h)
z_t(c) = (dot(Q[c], V[x_t]) + dot(Q[c], A[t](leaf(c)))) / sqrt(288)
cost_t(c) = -z_t(c)
```

The exact trainable count is `2,359,308`: separate `4096 x 288` query and
value tables plus twelve bank scalars. Per-sequence state is always
`4 * 120 * 288 = 138,240` f32 values, or `552,960` bytes. State is independent
of prefix length. The current-token dot is the matched bigram path; the second
dot is the only retained-state path.

Training and measurement may use floating-point full-vocabulary
cross-entropy. Runtime selection needs only minimum cost over all 4,096 tokens;
there is no softmax over prior positions. Runtime input is the current observed
token, prior bounded state, the exact geometry artifact, and the learned cell
artifact. It has no donor weights, teacher traces, QKV cache, stored full
prefix, future labels, Transformer layer, MoE, or provider call.

## Matched arms

All arms have byte-identical CPU initialization, parameters, state, examples,
optimizer schedule, stationary-frame execution, and operation/read ledger.
They train sequentially on Apple MPS.

The compiler-side training implementation may compose the supplied address
permutations into one stationary frame. In that frame, decay and gated writes
act independently at the exact transported addresses, so it avoids physically
recentring all 120 slots at every token. Focused tests must match the literal
step-by-step recurrence in logits, final state, and gradients. The main path
uses this exact closed form without activation checkpoint recomputation when
its measured memory is below the MPS recommendation; the literal recurrence
remains the semantic reference and deployed incremental form.

1. `exact_h4`: the canonical H4 left-regular address permutation.
2. `cyclic_120`: the same raw prime-residue leaves and identity offset under
   `a (+) b = e + ((a-e) + (b-e) mod 120)`.
3. `scrambled_h4`: candidate reads retain their true H4 leaf, but transport
   uses one frozen identity-fixing, non-homomorphic leaf-action bijection.

The scrambled artifact must bind a concrete multiplication-law witness and
must still generate all 120 transport actions. Relabeling transport, slots,
and candidates consistently is forbidden because that would be an isomorphic
coordinate rename rather than a destructive control.

Evaluation-only interventions use the trained exact-H4 artifact with retained
state disabled and with earlier-prefix order shuffled while keeping the current
token and target fixed.

## Frozen population

Only the existing #1017 TinyStories tokenizer and training store are reused.
No #1017 weights or traces enter this model.

- train store CID:
  `blake3:b18679a2d8efc005ff96c5dc3f7652693fea461489f46afc19b29a87a74ad6c6`;
- train index CID:
  `blake3:f422386cff6425e9b44336559942a16e4b286ddc41c64db798d77f488ba6d46a`;
- tokenizer CID:
  `blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc`.

Select the 320 lowest eligible story CIDs, where an eligible index record is
untruncated and has at least 257 tokens. Take the first 257 tokens per story.
The first 256 stories form fit and the next 64 form the held-out partition:
65,536 fit and 16,384 held-out actual-next rows, with no cross-story window.
The held-out paths remain unreadable to optimization until all three fitted
artifacts are frozen.

## Construction-only hard gate

Before any held-out model score, require all of:

- exact and byte-reproducible source, tokenizer, H4 table, leaf, population,
  and initialization identities;
- `120/120` generated-state coverage for exact H4 and the cyclic control;
- disjoint, untruncated 256-story fit and 64-story held-out populations with
  every target below 4,096;
- a label-free structural opportunity census `R_action >= 41` and at least one
  H4-versus-both-controls difference in every held-out story;
- equal parameter, state, logical-operation, read, and presentation ledgers,
  plus direct-recurrence output/final-state/gradient parity for the
  stationary-frame implementation;
- finite nonzero gradients through recurrence, overwrite, retained read, and
  full-vocabulary scoring;
- two warm-up and eight measured full forward/backward steps per arm, with
  `1.25 * mean_step_seconds * 768 <= 720 seconds` and peak MPS memory below the
  device recommendation; and
- one disposable 8-story, 64-step smoke per arm: every arm reduces its initial
  fit CE by at least 80%, while disabling H4 state worsens CE by at least 0.05
  nats.

A miss is
`UNAVAILABLE_FRAME_POPULATION_OR_LOCAL_BUDGET`. It forbids held-out scoring and
is not a model-quality or geometry result. CPU fallback and CUDA are forbidden.
No hour-scale run is authorized.

## One-shot main budget and terminal decisions

If and only if the cheap gate passes, train each arm for 256 AdamW steps at
batch 8 and context 256: eight exact fit passes, 524,288 token presentations
per arm and 1,572,864 total. Seed is 9736; learning rate is `3e-4` with a
16-step warm-up and cosine decay to `3e-5`; betas are `.9/.95`, epsilon
`1e-8`, weight decay `.1`, and gradient clip `1`. The three arms run
sequentially, and combined optimization has a 15-minute hard ceiling. There is
no retry, sweep, or post-reveal change.

The single run returns two nested verdicts:

- `ATTENTION_PASS`: exact-H4 state-on beats its own state-off intervention by
  at least 0.05 held-out nats and 0.25 top-1 percentage points (41 net rows),
  with a positive paired 64-story bootstrap bound; order shuffle is at least
  0.02 nats worse; replay, audits, and a prebound non-looping 32-token
  continuation pass.
- `H4_ADVANTAGE_PASS`: exact H4 additionally beats each independently trained
  cyclic and scrambled control by at least 0.02 held-out nats and 0.25 top-1
  percentage points, with both paired story-block bootstrap lower bounds above
  zero.

If attention passes but H4 advantage fails, retain the cell as a source-free
recurrent baseline while retiring the H4-advantage claim for this exact law.
#954 remains blocked. If attention fails, retire the whole cell. Only both
passes authorize #954 to freeze an independent final promotion probe.

## Observed construction result

The sole create-once construction preflight ended
`UNAVAILABLE_FRAME_POPULATION_OR_LOCAL_BUDGET`; result CID
`blake3:72ed93ec7e1356091cb40bdc6f35d89cf6520f4262778149908817a514832018`.
It emitted no main authorization, did not open held-out bytes, and did not run
the 256-step three-arm fit.

The geometry, structural opportunity, equal-ledger, memory, and gradient paths
passed. Stationary/direct recurrence parity was separately verified by focused
logit, final-state, and gradient tests; the official envelope's
`direct_recurrence_parity = REQUIRED` field is a contract marker, not an in-run
parity measurement. The official 24-step timing mean was `0.8296007534 s`,
giving a frozen projection of `796.4167 s`; that misses the conservative `720 s`
admission ceiling, although it remains below the separate `900 s` hard wall.
Peak synchronized allocation was about `3.56 GB` versus `12.71 GB` recommended.

The decision-bearing miss was learning, not geometry reachability. After each
arm's sole 64-step, eight-story disposable fit:

- exact H4 CE moved `8.3177948 -> 8.3070354` (`0.12935%` reduction);
- C120 moved `8.3177929 -> 8.3067646` (`0.13259%` reduction);
- scrambled H4 moved `8.3177967 -> 8.3070116` (`0.12966%` reduction); and
- exact-H4 state-off CE was `8.3073578`, only `0.0003223` nats worse than
  state-on, versus the required `0.05`.

All learned gradient paths were finite and nonzero, so the result is not a
disconnected implementation. The exact cell and frozen optimizer simply did
not learn enough for a natural-language main run to have decision value. Under
the predeclared contract there is no retry, sweep, held-out score, attention
verdict, or H4-advantage verdict. The main-run harness was therefore not
retained as inactive scaffolding.

## Post-terminal source and reveal disposition

The exact create-once
[started envelope](r4_group_addressed_retention_preflight_started_973_raw.json)
and [result envelope](r4_group_addressed_retention_preflight_result_973_raw.json)
are preserved byte-for-byte. The started envelope binds trainer tree
`blake3:126df4d82a020ada3898dff0d61d17f71beca4dd614cbc97a017b80227877ab9`.
The terminal shipping tree is
`blake3:066ac85303e791dcfe024a434e1dd259aeac60fcfac9c022f3bc547b1cfeeab2`;
it intentionally differs after the result.

At execution, `group_retention.py`, `group_retention_campaign.py`, and
`group_retention_data.py` matched their signed CIDs. The signed package also
contained an unimported and unexecuted 55,494-byte draft
`group_retention_main.py`
(`blake3:5b6cbf746d3062e07bb4da2949d3764aa9c06b0aa950e23378d699abacbe76c5`).
It was deleted only after the negative terminal because no main run was
authorized. Post-terminal documentation/help edits changed `pyproject.toml`,
`__init__.py`, and `cli.py`. The shipping `group_retention_data.py` additionally
removes its unused reveal helper: that helper accepted self-authored, merely
well-formed artifact CIDs rather than verifying fitted artifacts plus a positive
authorization. The terminal package therefore exposes no held-out-open API;
the official held-out directory remains mode `000`, with no reveal marker and
zero reads.

No experiment was rerun after these source-disposition changes. The raw signed
tree remains the provenance of the observed terminal; the shipping tree is the
safer retained implementation and is not presented as byte-identical to it.

## Claim boundary

Even both passes establish only bounded context-256 source-free
geometry-native retained attention. They do not establish broad coherent
generation, reasoning, correctness, H4/E8 superiority in general, efficiency,
exact integer/shift-add lowering, Rust/WASM or browser integration, or release
readiness.

## Evidence log

- 2026-09-01 — live #973 independently re-scoped and assigned. Implementation
  began from `main` at `ae7dbd6fd5aebdafc5493b7438bf5fdebe6e38fe`.
- 2026-09-01 — the first construction attempt exposed a pre-training indexing
  contradiction: assigning token `t > 0` to `p_t` discarded the only even
  prime, so the odd-offset C120 control generated only `60/120` states. Before
  any optimization or held-out access, the policy was corrected to enumerate
  non-BOS tokens from `p_0`: token `t > 0 -> p_(t-1) mod 120`.
- 2026-09-01 — an implementation-only MPS probe showed that literal per-token
  full-field recentering was dispatch-bound. An exact stationary-frame closed
  form reduced a full batch-8/context-256 forward/backward step from about
  `4.65 s` to an eight-step mean of `0.67549 s`; the frozen ETA formula projects
  `648.47 s`. Observed MPS memory was `1,597,398,528` current and
  `3,521,118,208` driver bytes versus `12,713,115,648` recommended. This is
  implementation qualification, not the official construction result.
- Geometry export: `PASS` — canonical artifact
  `blake3:55447c00c1eb86a1d05324d6c83d044407bdc89f653f46957bf6f0bccb6c000b`;
  leaf map
  `blake3:0ede705abd034327e8d3cf622e1daf04b0817a81f26cb345e7f84939101fb627`;
  exact H4, C120, and scrambled-action generated coverage are each `120/120`;
  direct support is 35, identity offset is 119, and the scramble moves 118
  actions with a concrete non-homomorphism witness.
- Population freeze: `PASS` — population manifest
  `blake3:35af5002bfbe92d68403e2cf8742fae4a22b7d6b11109a3f861fab9e15d2b52e`;
  fit-only training-view manifest
  `blake3:ce26777ed9fa8d25410b3f27acf30a0b33d9d725d9d3fb1e614137bf91581f31`;
  preparation manifest
  `blake3:8be923654394d66019e54a1424a7222319bb196090cc163b60f32ea9ff3d6dfe`.
  The held-out directory is mode `000` with zero training reads. The label-free
  census finds `R_action=16,234/16,384`, all 64 held-out stories have an
  opportunity, and the minimum is 251 rows per story.
- Construction gradient/timing gate:
  `UNAVAILABLE_FRAME_POPULATION_OR_LOCAL_BUDGET`; gradients/memory/equal work
  passed, frozen timing projection `796.4167 s > 720 s`.
- Disposable overfit smoke: `FAIL` — approximately `0.13%` CE reduction in
  every arm; exact-H4 state-on delta `0.0003223` nats.
- Main optimization: `NOT_RUN` / unauthorized.
- Held-out evaluation: `NOT_RUN`.

## Independently frozen fuller-decoder successor (MPS terminal)

The predecessor result above remains terminal and is not reopened or tuned.
The sole successor is now independently frozen as
[`R4GroupAddressedRetentionDecoderV1`](r4_group_addressed_retention_decoder_973.md),
under the authoritative
[live #973 contract](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5489101447).

The successor adds a tied token/residual path, final and per-block RMSNorm,
exactly two group-addressed retained-attention blocks, SwiGLU channel mixing,
residual connections, and a tied output head. It reuses the exact predecessor
geometry and fit-only training view while excluding the predecessor's disposable
smoke stories. Its construction training and validation partitions are disjoint.

The sole MPS attempt stopped
`UNAVAILABLE_FULLER_DECODER_CONSTRUCTION` before optimization: deterministic
MPS measured `1.2517232777 s/step`, so the frozen safety formula projected
`801.1028977 s > 600 s`. Causality, direct/incremental parity, gradients,
equal work, and memory passed. Optimization and held-out evaluation remained
`NOT_RUN`; H4 was `NOT_EVALUATED`. This is a backend/time-budget result, not a
retained-decoder failure and not evidence against geometric attention.

The exact terminal is recorded in the successor document. A resource-only
successor may retain every scientific choice while using the independently
measured-fast deterministic Apple CPU/Accelerate four-thread execution plan.
The predecessor's raw envelopes and source-disposition record remain
unchanged.

## Fuller-decoder CPU-recovery follow-up — 2026-09-01

The separately frozen
[`R4GroupAddressedRetentionDecoderV1CpuRecovery`](r4_group_addressed_retention_decoder_cpu_recovery_973.md)
subsequently completed all 512 construction steps in `438.117083` seconds. On
the disjoint construction-validation partition, state-off lost `0.967227` nats
and 182 of 4,096 top-1 decisions. The exact complete-decoder recipe did not
satisfy its frozen generalization criterion because trained exact-H4 validation
CE was `0.604243` nats worse than initialization. Formal H4 specificity remained
`NOT_EVALUATED`; diagnostically, scrambled transport CE was `0.033049` nats
better while exact H4 led top-1 by only four decisions, below threshold. This
is bounded retained-attention component evidence, not a complete language-model
or H4-advantage result.
