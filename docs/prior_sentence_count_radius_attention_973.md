# #973 Gate 0 prior-prefix geometric-attention record

- **Date:** 2026-08-28
- **Issue:** #973
- **Contract:** frozen on the live issue before implementation or query outcome
- **Mechanism:** `PriorSentenceCountRadiusR4V1`
- **Outcome:** positive on the exact two-history Gate 0 contract
- **Terminal:**
  `RETAIN_GATE0_PRIOR_SENTENCE_ATTENTION_CONTINUE_PARAGRAPH_CONVERSATION`
- **Issue state after this result:** #973 remains open; #954 remains blocked

## Question and accepted input

The empirical question was whether one candidate-relative R4 operator could
make state before the active sentence causally load-bearing through decoded
output while the local #953 evidence stayed exactly matched.

The operator consumes only the maximum-count tie already admitted by the
unchanged #953 `MultiscaleCountRadiusR4V1` API. It does not alter the source-free
table, lexical ids, active backoff row, candidate support, maximum count,
decoder, or later #953 continuation policy. The production #989/#953 artifacts
remain unchanged; this Gate 0 record uses a separate tiny synthetic fixture.

## Frozen synthetic contrast

Construction used D3-construction partition ids `14` and `657`:

```text
Nora chose tea before breakfast. Later Nora asked for tea.
Owen chose coffee before breakfast. Later Owen asked for coffee.
```

The target-free census then used D3-held-out partition ids `12` and `13`:

```text
Mara chose tea before lunch. When the server arrived, Mara asked for
Mara chose coffee before lunch. When the server arrived, Mara asked for
```

Only after the target-free operator bytes and census were frozen did the probe
attach the sealed continuations ` tea.` and ` coffee.`.

The prose is synthetic. The D3 hashes enforce partition separation for this
probe, but do not supply corpus provenance, natural-distribution evidence, or
semantic transfer.

## Frozen operator

For each candidate `c` in the #953 maximum-count tie, the operator uses

```text
x_H(c) = [q_3(c), q_2(c), q_1(c), q_P(c)]
q_P(c) = floor(n_P(c) * 2^32 / T_P)
R_H(c) = sum_i x_H(c)^2
```

The first three coordinates are copied from the bound #953 evidence. `n_P(c)`
is the exact count of candidate token `c` in the prefix before the last fitted
period token, excluding BOS, the period, and the active suffix. `T_P` is the
sum across the admitted candidates. A missing boundary, zero `T_P`, or a tied
maximum abstains to the exact #953 choice. More than 64 encoded prefix units or
more than eight tied candidates fails closed.

The artifact compiles every bounded `(n_P,T_P)` fixed-point value and square,
plus the trigram/bigram depth squares. Query selection therefore uses exact
token comparisons, integer counts/add/subtract/compare, and table reads; it
does not calculate a fixed-point division or square. The artifact binds both
the source-free table CID and the #953 overlay CID and rejects binding drift,
noncanonical bytes, trailing bytes, tamper, and scope overflow.

Three arms share the same active row, prefix scan, candidate comparisons,
normalization reads, square reads, radius comparisons, support, and declared
work:

- **real:** use the candidate-relative prefix coordinate;
- **scope-disabled:** perform the census but mask that coordinate and return
  the #953 choice; and
- **candidate-permuted:** cyclically reassign the same candidate-relative
  contributions in canonical tied-token order.

## Label-blind Gate 0 result

Both prompts ended in the exact active frame ` asked` / ` for`. The active
trigram support was exactly ` coffee` and ` tea`, each at count one. Both
candidates had identical #953 coordinates

```text
(2^31, 2^31, 330382099, 3 * 2^30)
```

and identical #953 radius `19708817909656044393`. Both #953 arms therefore
selected the canonical fallback ` coffee` with identical local work.

Each prompt had 13 encoded prefix units and the matched census performed 26
candidate-membership checks. The one earlier exact candidate occurrence gave
the present candidate `q_P = 2^32` and radius `27779268441903973225`; the absent
candidate had `q_P = 0` and radius `9332524368194421609`. The transformed
candidate-relative state classes were distinct, each had one unique winner,
and no retained class required incompatible selections.

| Target-free prompt state | Real | Scope disabled | Candidate permuted |
| --- | --- | --- | --- |
| earlier `tea` | ` tea` | ` coffee` | ` coffee` |
| earlier `coffee` | ` coffee` | ` coffee` | ` tea` |

Support mismatches and declared-work mismatches were both zero. Structural
teacher, provider, source-weight, and future-unit read counters were all zero.

## Decoded causal consequence

After the label-blind census was frozen, the same bound operator ran exactly
two histories by three arms. The first selected unit came from #973; subsequent
units returned to unchanged #953 selection. A three-attempt bound allowed the
two visible units to append and then observe EOS.

| Arm | Earlier `tea` | Earlier `coffee` | Exact sealed continuations |
| --- | --- | --- | ---: |
| Real | ` tea.` | ` coffee.` | 2/2 |
| Scope disabled | ` coffee.` | ` coffee.` | 1/2 |
| Candidate permuted | ` coffee.` | ` tea.` | 0/2 |

All six continuations emitted two lexical units, observed EOS, and avoided a
period-one or period-two cycle. The focal `tea` candidate and decoded output
changed only when the real higher-scope coordinate was active. This is the
load-bearing causal result; a changed trace alone would not have qualified.

Support/work equality is claimed at the shared first decision only. Once arms
choose different first units, their later contexts differ and no cross-arm
support/work equality is inferred.

## Canonical artifacts and replay

| Item | Result |
| --- | --- |
| Tiny synthetic source-free table CID | `blake3:60a9d2cc2f6da53f1a2f47d64531f41b03c4654d347ca9640820256ae962146e` |
| Tiny #953 overlay CID | `blake3:d513a15e4d7f3438d728ca438f1031e935f144f7b13875c2ec147accd5e4ba58` |
| #973 operator bytes | 60,688 |
| #973 operator CID | `blake3:556e0ea2e08958ca0671b531bdc1a743e32a4a362767dac778210c6b2da9a38b` |
| Target-free census CID | `blake3:c70d4f333ded6b247bb97ddccb3b5895078cb293eff9567cc53428db5a1b568d` |
| Decoded-smoke CID | `blake3:804d52e9beffd240046d98783cc7f1c506a2b27d70aecab8b33898e806f7ce4f` |

The operator recompiled and reloaded byte identically. Both matched
continuations and the complete decoded-smoke report replayed byte identically.
The CIDs above identify deterministic in-memory probe artifacts reconstructed
by the focused test; no generated binary fixture is checked into the repository.

## Decision and claim boundary

The frozen positive terminal is
`RETAIN_GATE0_PRIOR_SENTENCE_ATTENTION_CONTINUE_PARAGRAPH_CONVERSATION`.

This establishes one bounded exact-candidate prior-prefix geometric
attention/copy mechanism whose state changes an already-admitted candidate and
the decoded output under matched controls. It is the first #973 mechanism, not
pre-attention scaffolding.

It does not establish semantic placement, paraphrastic or anti-recall transfer,
general sentence/paragraph/conversation/global attention, broad coherence,
correctness, reasoning, performance, allocation-free serving, chat readiness,
formal closure, or release readiness. Exact earlier-candidate recurrence is the
operative signal, so calling this semantic attention would be false.

#973 remains open. The next decision-bearing #973 action is one independently
frozen paragraph-scope contrast that cannot be solved by exact recurrence of
the admitted candidate, followed by conversation and bounded-global scope only
if each preceding mechanism remains load-bearing. Corpus scale remains dormant.
#954 stays blocked until #973 reaches its full native terminal.

## Activated checks

- focused #973 Gate 0, binding/tamper/abstention, and exact-support tests: 3/3 PASS;
- touched `uor-r4-core` compilation: PASS;
- touched core lint after suppressing only pre-existing origin-main lint
  families: PASS;
- operator/census/decoded-smoke exact replay: PASS; and
- touched Rust formatting and diff whitespace: PASS.

Unmodified strict core/dependency clippy was exercised but is `UNAVAILABLE` as
a change-specific verdict because current `origin/main` already violates the
new toolchain's warnings in untouched graph-runtime, model-source,
prime-route, recursive-attention, and SpiralCore code. Workspace-wide tests,
wasm, BDD, teacher/model paths, no-std ladders, Gate C, kappa reproduction,
audit, fuzz, conformance, corpus-scale, product, performance, formal, and
release qualification are `NOT_RUN` unless a later activated check is
explicitly recorded here.

## Forward-action update (2026-08-28)

This bounded result remains unchanged. Its then-next sequencing is complete;
the then-current #973 work was ADR-0005
`PredictiveConnectionRetentionGate0V1`. `ConnectionGaugeCovarianceV4` Phase I
subsequently passed; its target-free held-out freeze is now sealed in PR #1001,
so protected merge/reveal is the current 2026-08-29 action following the negative gated-delta smoke and direct-attention
V3 result (geometric `3/12`, fixed-tangent plain `12/12`).

## Successor direction (2026-08-29)

V4 subsequently completed terminal-negative at `13/24`, without adequate
separation from its destructive controls. V4 will not be rerun or retuned. The
HELM-D-R4 became the full-decoder, gauge-equivalent ordinary-causal-softmax
reference with R4/Spin frame transport. Its parity gate now passes; the verdict
and scope are authoritative only in the
[HELM-D-R4 result](helm_d_r4_softmax_decoder_result_973.json). The active #973
successor is intrinsic R4 distance and normalized-centroid attention, followed
conditionally by multi-resonance replacement and recurrent lowering.

## Attempt 02 successor update — 2026-08-29

This record's bounded evidence and the `HELM-D-R4` gauge-equivalent ordinary-
softmax PASS remain unchanged. The separately trained intrinsic Lorentz-R4
successor stopped before D3 at
`UNAVAILABLE_INTRINSIC_LORENTZ_R4_STOP_BEFORE_HELD_OUT` (result CID
`blake3:da2a63323d6211b8d581e5a4ed75d788eb919ff0f210d2e3beb8a749ee1bc64f`):
normalized-barycenter covariance was `9.1214e-8` against the frozen `1e-8`
limit, and construction-validation NLL was diagnostically worse than the donor
by `1.2531` and the matched flat control by `0.20893` nats/token. No reveal
marker or held-out result exists. No Attempt 03 is authorized under this freeze;
any further intrinsic work must be a newly frozen, source-faithful
learned-manifold successor. Multi-resonance, recurrence, lowering, scale, and
#954 remain blocked. See the
[owning intrinsic record](intrinsic_lorentz_r4_attention_973.md) and the
[compact result summary](intrinsic_lorentz_r4_attention_attempt_02_summary_973.json).

## Learned-manifold V2 outcome and current successor — 2026-08-29

Source-faithful learned-manifold V2 completed a valid non-D3
construction-validation run. Donor/gauge parity and all destructive-control
separations passed, but learned Lorentz failed donor retention and matched
Euclidean parity; the controls establish sensitivity only. The sole current
#973 action is the frozen 8/8
[score-by-readout localization](helm_d_score_centroid_localization_973.md).
D3 remains `NOT_RUN`; resonance, recurrence, lowering, scale, and #954 remain
blocked.
