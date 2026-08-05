# Issue #290 §5.2 — novel-context accuracy protocol

Status: protocol and incumbent baseline recorded; the compiler-folded fixed-point
translation table and allocation-free runtime kernel are implemented in #361.
The novel-context accuracy and cost decision for #290 remains open.

*Addendum 2026-08-05: the decision is no longer open — issue #290 recorded a
negative (`research/290-fmm/RESULT-52.md`): the far-field operator cannot reach
usable precision (Eckart–Young bound; rank 20 carries ~6% operator error), and
the interaction kernel is exactly rank ≤ 288 by construction, so there is no
O(n²) for an FMM to remove. The uncalled FMM section emission and the runtime
packed-kernel evaluation path were removed under #425; the format-crate parser
(`fmm.rs`, `SectionId::FMM`) is retained so old artifacts still validate, and
the certifier-side `FmmCandidateScorer` remains as the exploratory BDD S7
measurement harness. Disposition details:
`docs/deferral_record_2026_08_05.md`.*

## Purpose

Issue #290 proposes a far-field/FMM approximation for long-range semantic
interactions. The existing Gate C score is not a suitable acceptance test: its
positions are drawn from the corpus used to compile the graph and therefore
measure exact-context retrieval and in-distribution memorization. §5.2 must
measure genuinely novel context and compare the incumbent R4G1 path with the
candidate under identical teacher-forced histories.

## Fixed evaluation set

Use the eight deterministic prompts already pinned by the teacher-parity BDD
suite:

```text
why is the sky blue?
what is the capital of France?
explain gravity to a child
how do computers work?
what is photosynthesis?
tell me about the moon
how does a bicycle work?
what is the internet?
```

At each prompt position, advance the teacher with the true preceding token and
ask the evaluated runtime to predict from that same true history. Do not feed a
runtime's prediction back into the teacher or into the next runtime window;
that would measure compounded free-running divergence rather than the local
decision being tested. The default budget is 256 positions, with the current
fixture run below using 96 positions so it is directly comparable with the
existing pinned BDD thresholds.

The evaluation must record, for each implementation:

- evaluated positions and abstentions;
- top-1 agreement with the teacher argmax;
- top-8 recall;
- teacher negative log-probability of the selected token, in bits/token;
- an implementation label, artifact κ, teacher κ, prompt-set κ, and the exact
  candidate configuration (rank, tolerance, admissibility rule, and quantile).

The prompt-set κ is the BLAKE3 digest of the UTF-8 prompt bytes in listed
order (concatenated without separators); for this set it is
`blake3:3735fa62ec052aac7266a421702b281598ea298ab13b9892325d02ba40b00a2b`.
A candidate result without all three input κs is not a reproducible
§5.2 result.

## Incumbent baseline

Command used on 2026-08-02:

```bash
R4_PARITY_POSITIONS=96 \
  cargo test --test bdd --offline -- --name 'S[23]' --concurrency 1 -v
```

Pinned fixture run:

| implementation | positions | abstains | top-1 | top-8 | mean teacher-argmax gap (bits) |
|---|---:|---:|---:|---:|---:|
| legacy TLS store | 96 | 0 | 0.0104 | 0.1771 | 9.2121 |
| R4G1 graph | 96 | 3 | 0.0104 | 0.0521 | 11.4626 |

The same replay also records teacher cross-entropy for the selected token:

| implementation | teacher bits/token |
|---|---:|
| legacy TLS store | 11.7423 |
| R4G1 graph | 13.9549 |

The last column is the existing parity harness's regret-style diagnostic: the
teacher-logit gap between its argmax and the selected token, clipped at zero.
It is not the requested bits/token metric and must not be presented as one.
The BDD scenario remains the source of truth for the incumbent pass/fail
floors; this table is the §5.2 baseline snapshot.

## Candidate comparison and decision rule

The first certifier-side candidate is implemented in
`uor-r4-graph-certify::fmm`. It forms the prototype/emission interaction map
from the validated graph, diagonalizes the deterministic symmetric Gram matrix
`PᵀP`, retains a bounded basis, and scores the same artifact-derived query
signature through that basis. It uses floating point and is explicitly not a
serving implementation by itself. The compiler folds its fixed-point factors
into the optional FMM1 section on the normal scored-artifact path; the deployed
reader uses only sign-selected table reads and saturating integer add/sub.

Run it through the S7 parity scenario with:

```bash
R4_FMM_POSITIONS=256 \
  cargo test --test bdd --offline -- --name 'S7' --concurrency 1 -v
```

The eight prompts contain 96 positions in total, so the 256-position budget
does not increase this fixed snapshot. With rank 20 and relative singular
tolerance `1e-2`, the pinned run produced:

| candidate | positions | abstains | rank | retained energy | top-1 | top-8 | teacher bits/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| certifier FMM | 96 | 0 | 20 | 0.6967 | 0.0104 | 0.2604 | 10.5879 |

The fixed-point translation-table form uses Q1.15 basis entries and a common
power-of-two factor scale. On the same replay (`factor_fraction_bits = 29`),
it selected the same token at every position and produced identical metrics:

| representation | storage estimate | top-1 | top-8 | teacher bits/token |
|---|---:|---:|---:|---:|
| float factors | 337,888 B | 0.0104 | 0.2604 | 10.5879 |
| fixed-point factors | 164,056 B | 0.0104 | 0.2604 | 10.5879 |

The fixed-point scorer also exposes a caller-buffered selection method with no
per-call allocation. The packed runtime adapter is covered by an end-to-end
R4G1 fixture, including deterministic tie-breaking, recent-token penalties,
and a steady-state zero-allocation census. Compiler-side factor products remain
outside the deployed kernel.

Against the incumbent R4G1 graph, top-1 is unchanged, top-8 improves by
20.83 percentage points, and teacher cross-entropy improves by 3.3670
bits/token. The packed table is now an implementation result, not an accuracy
claim: artifact reports expose its byte footprint, rank, and candidate count,
while the runtime fixture measures the fixed work shape (`dimension ×
candidate_count` sign/add updates per query). A pinned §5.4 comparison on the
full teacher fixture is still required before choosing it as the default serving
route.

The candidate must expose the same teacher-forced prediction contract as the
incumbent. First compare it at the same 96-position snapshot, then rerun at
the default 256-position budget. Keep the prompt set, teacher math mode,
artifact, graph, and tokenizer fixed. Report both absolute metrics and the
candidate-minus-incumbent deltas.

The pre-registered issue rule is:

- kill the FMM route if novel-context top-1 falls by more than 0.5 percentage
  points versus the incumbent, or
- kill it if its teacher cross-entropy (bits/token) is worse than the pinned
  teacher-floor budget for the evaluation.

An implementation that improves the score but does not demonstrate a bounded
far-field approximation is not an FMM result. Conversely, a candidate that
passes accuracy must still clear the §5.4 constant-factor check: measured
operator work, stored translation data, and routing overhead must beat the
incumbent on the same fixture.

## Remaining work

The certifier candidate and bounded fixed-point adapter now exist, but issue
#290 is not yet closed. The remaining decision is whether the measured accuracy
gain justifies enabling the FMM route in serving. That requires the full
teacher-forced replay plus the §5.4 cost comparison against R4G1, with the
artifact and runtime evidence linked to the result.
