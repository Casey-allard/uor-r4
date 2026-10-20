# #973 finite-indexed R4 nonlinear block

**Date:** 2026-09-04
**Status:** `FINITE_INDEXED_R4_NONLINEAR_EXECUTED_LANGUAGE_UTILITY_UNESTABLISHED`

## Implemented decision

`R4H4FrameQuaternionCubeResidualV1` replaces the dense SwiGLU call in the
accepted sparse recurrent path with one bounded, parameter-free R4
nonlinearity. The sparse candidate selector, transported learned Q/K/V/O
attention, recurrent state, tokenizer, vocabulary head, and sampler stay
unchanged. `R4SparseGeometricCandidateSoftmaxKVBindingV1` remains the dense
SwiGLU comparator over the same learned artifact.

After each attention residual, the existing post-attention RMSNorm produces
`n[B,48]`. The new state map splits it into twelve ordered R4 blocks
`n_j in R4`. For the current token's cumulative H4 frame `F_h`, using the
existing convention `F_h: local R4 -> model R4`, each block executes

```text
q_j       = transpose(F_h) n_j
C(0)      = 0
C(q_j)    = q_j^3 / ||q_j||^2                 when q_j != 0
G_h(n_j)  = F_h C(q_j)
delta_j   = G_h(n_j) - n_j
values'   = values + flatten(delta_0, ..., delta_11)
```

Writing `q=(a,v)` with `v in R3`, the f32 implementation uses the closed form

```text
C(q) = (
  a * (a^2 - 3 ||v||^2),
  (3 a^2 - ||v||^2) * v
) / (a^2 + ||v||^2).
```

The zero branch uses exact zero detection and returns zero. It introduces no
epsilon or fitted threshold. All twelve blocks are evaluated from the same
normalized input and the same current frame. The operator adds no learned
parameters or persistent state. Dense MLP tensors remain in the accepted
artifact so the two runtime arms stay byte-compatible, but this candidate
does not invoke or compute with them in its predictive forward path. Artifact
validation and export still read and serialize every retained tensor.

## Why this operator

Over real quaternions, norm multiplicativity gives `||C(q)||=||q||`. The map
is odd and homogeneous over real scalars, and its residual obeys
`||C(q)-q|| <= 2||q||`. These are algebraic consequences of the declared
operator. They are not claims about f32 equality, trained behavior, language
quality, or the surrounding learned decoder.

The current H4 element selects from 120 frame-index entries on continuous R4
blocks. Because the cube is odd, antipodal frames `F` and `-F` produce the same
conjugated map, so this bank has at most 60 distinct operators. “Finite” here
means a finite indexed operator bank with fixed work and fixed state; it does
not mean that hidden values are quantized.
The implementation is transitional f32 compiler-side arithmetic. Integer/table
lowering, zero-multiply serving, and allocation-free execution remain later
work. The direct quotient is also not total over every possible finite
subnormal input: a nonzero block's squared norm can underflow before the
reciprocal, at which point the implementation rejects the nonfinite result.
Neither observed prompt approached that case.

At batch one, each layer and token evaluates twelve R4 blocks. Its declared
logical work is 384 frame-coordinate products, 168 closed-form scalar products
for the quaternion cube, twelve reciprocals, and 48 residual subtractions. The
dense comparator executes 18,432 learned-weight products per layer and token
across its `48 -> 128`, `48 -> 128`, and `128 -> 48` projections. These are analytical
operation counts, not latency or hardware-energy measurements.

## Donor boundary

The project knowledge index was queried for the active nonlinear-block seam.
Original sources were then inspected for that decision.

- SpiralCore v66, SHA-256
  `38449bddefd359d69a497ca4965e6d14a39f7ab2d33940e41fd9747598b3a4ea`,
  supplies finite operator-index and refusal discipline, but its implemented
  Cl(0,6) bivectors are linear signed 8x8 maps over a separate 240-root E8
  system. It supplies no R4/R8 carrier or nonlinear language block, so no E8
  operator is imported here.
- The pinned W33 chamber operators act on `Q^160`. Their rank-48 projector is a
  particular subspace, not a canonical identification with this decoder's
  `R48`, so they are not imported.
- The pinned NEMESIS material supplies an attributed Cayley-Dickson/quaternion
  product description and a useful encode/operate/decode discipline. Its wider
  ring, inverse, complexity, and physical claims are not transferred, and no
  source code is copied.
- The existing UOR/H4 sidecar supplies the validated 120-frame carrier used by
  this implementation. Exact H4 table identities and f32 model behavior remain
  different evidence layers.
- The zeta channels and signed `Z[phi]` heatmap remain separate trace/control
  hypotheses. The #970 0/6 strict-transfer result forbids treating those
  classes as an established semantic activation.

This mechanism does not revise the #967 shortest-Cayley 6/6 tie, the #970
heatmap 0/6 transfer result, or the uneven 12-token/3-token sparse trajectory
preservation recorded by the preceding checkpoint.

## Direct comparison fixed before execution

Run exactly the existing two development prompts through two no-fit arms:

1. accepted sparse recurrent attention plus dense SwiGLU; and
2. the identical sparse recurrent attention plus
   `R4H4FrameQuaternionCubeResidualV1`.

Both arms use the accepted artifact, tokenizer, exact-H4 geometry/frame
sidecar, seed 9738, top-k 40, temperature 0.8, and at most sixteen generated
tokens. Record the generated token IDs and continuations, common prefixes,
finite output/state checks, sparse source ceiling, forbidden-read counters,
dense-MLP calls, operator blocks, maximum f32 block-norm error, and maximum
residual-bound ratio. Do not fit, tune, inspect held-out data, add a third arm,
or launch a scale campaign in this task.

## Measured result

The candidate completed both prompts. The accepted dense-SwiGLU sparse outputs
were read from the preserved #1122 artifacts and were not rerun. Asset CIDs and
prompt token IDs matched, and sparse candidate-selection traces matched exactly
for every forced prompt token. After generation began, both candidate runs
diverged on the first sampled token.

| Prompt | Dense-SwiGLU sparse continuation | Quaternion-cube continuation | Common generated prefix |
|---|---|---|---:|
| `A purple turtle found a clock in the garden` | `, there was a time, there was a little girl named I saw a little` | ` Sue face heistol and�,\u0003 was� to<\|unk\|> She� up “` | 0 |
| `Albert Einstein was born in` | ` his friend, and Lily were very sad. He said, "So and` | `` jack hisleton thenaug you` always'ter very�\n all bor.`` | 0 |

The visibly degraded no-fit continuations are an honest observation, not a
language-quality score. The accepted artifact was fitted jointly with its
dense SwiGLU weights; this intervention disables 36,864 such weights without
refitting the surrounding representation.

Across 53 processed tokens, the candidate executed 1,272 R4 block evaluations,
40,704 H4 frame-coordinate products, 17,808 quaternion-cube scalar products,
1,272 reciprocals, and 5,088 residual subtractions. It executed zero dense-MLP
calls and zero dense-MLP weight products; the corresponding dense comparator
would analytically execute 1,953,792 MLP weight products across those
layer-token calls.
The largest observed f32 block-norm error was
`7.152557373046875e-07`; the largest observed ratio
`||delta|| / (2 ||normalized block||)` was `1.0`.

Both runs kept the recurrent state at 2,304 f32 values / 9,216 bytes, stayed
within nine attention sources, and reported zero complete-prefix scans,
unselected K/V reads, provider calls, teacher calls, future reads, and
forbidden reads. The focused causal/mechanism check reported:

```text
Ran 4 tests in 0.190s
OK
```

Preserved raw artifacts:

- turtle candidate SHA-256:
  `2fd4e8a9945b0913c417badc97fb7cb9c7bec682c48bc533a1f0baea0ed418da`
- Einstein candidate SHA-256:
  `dd7de4ffd4da84b181e7559af327d7a6b1757a73938a3fabb6c8c6d49c690f1b`
- comparison summary SHA-256:
  `502bccf9482ba06f6582507243e884e4a97a5eddc6ee7eae43a64fb45ea5a5c3`

They remain outside Git under
`.uor-models/research/issue-973-quaternion-cube-r4-v1/comparison/`.
Independent review corrected the descriptive bank cardinality from 120 maps to
120 frame-index entries with at most 60 distinct maps. The canonical files were
regenerated once after that metadata-only source correction; their numerical
fields and continuations remained exact. The pre-correction bytes remain in
the `pre-review-overstated-frame-count/` child directory.
Review also narrowed the retained-weight claim to the predictive forward path:
artifact validation and export read and serialize every retained tensor, while
candidate prediction performs zero dense-MLP calls or weight products. The raw
records were refreshed after that field-name correction; their only changes
were the old and new model-field keys. The immediately preceding bytes remain
in `pre-review-forward-path-claim/`.

## Evidence boundary and next action

- **Mathematical result:** over real quaternions the declared cube map is odd,
  real-scale equivariant, block-norm preserving, and has the stated residual
  bound. No formal proof artifact was produced.
- **Measured behavior:** the focused test and two direct executions establish
  the reported f32 bounds, deterministic audit counts, dense-MLP bypass,
  unchanged prompt-stage sparse selection, bounded state, and causal access
  counters for this implementation and these inputs.
- **Unverified hypotheses:** that this finite-indexed operator can be fitted to
  useful language, that H4 frame selection helps, or that it supports reasoning,
  coding, table-native serving, or architectural-alpha behavior. Trainability
  at the exact zero branch and numerical hardening for extreme subnormal inputs
  also remain unmeasured.

This completes the nonlinear geometric mechanical checkpoint. It does not
erase any prior negative result and does not qualify the observed text as
useful. #973 remains open because the bounded architecture still needs learned
language behavior.

The one next action is to specify a bounded development-data fit of this exact
sparse-plus-quaternion-cube architecture against the retained dense-SwiGLU
comparator before increasing model or data scale.
