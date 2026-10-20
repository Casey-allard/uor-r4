# #953 frozen B0 geometric-intervention contract

- **Date:** 2026-08-28
- **Issue:** #953
- **Initial contract status:** frozen before implementation or held-out scoring
- **Outcome status:** positive after the frozen run and exact external replay
- **Mechanism:** `MultiscaleCountRadiusR4V1`
- **Positive terminal:**
  `PROCEED_TO_A1Q_H_WITH_BOUNDED_SOURCE_FREE_GEOMETRIC_GENERATION`
- **Negative terminal:** `RETAIN_TABLE_BASELINE_GEOMETRY_NO_INCREMENT`

## Frozen reference

The reference is the unchanged #989 `SFTBL001` construction-only table:

- corpus CID
  `blake3:194db0eebf2d49823ece01ee935447a0cc9edeaf018454ceea480ce7590132cf`;
- 3,000 documents under the unchanged D3 split: 2,404 construction and 596
  held out;
- 446,342 held-out known-target positions;
- 99,362/446,342 table top-1 correct;
- packed artifact CID
  `blake3:ccdc399731cb866a329be478467a434cda4e445813421e5d17c21ccc87288297`;
- fixed prompt `The United States`, continuation cap 16; and
- the unchanged lexical codec, first-nonempty trigram -> bigram -> unigram
  support, exact decoder, cycle rules, and lowest-token canonical tie break.

The `SFTBL001` bytes and #989 predictor remain the control. The intervention
may not add or drop a candidate, change backoff order, inspect held-out text
during fitting, or alter decoding.

## Single intervention

For one context, let `C` be exactly the entries in the #989 predictor's first
nonempty row. Scan all of `C` exactly as #989 does, obtain its maximum observed
count, and retain the complete maximum-count tie set. A unique maximum and a
unigram decision are geometry-ineligible and reproduce #989 exactly.

For each tied candidate `c` in an eligible trigram or bigram row, compile the
fixed-point vector

```text
x(c) = [q(n3(c), T3), q(n2(c), T2), q(n1(c), T1), depth]
q(n, T) = floor((n * 2^32) / T)
depth = 3 * 2^30 for trigram, 2 * 2^30 for bigram
```

where `n3`, `n2`, and `n1` are construction-only candidate counts in the
current trigram row, current-last-token bigram row, and unigram table;
`T3`, `T2`, and `T1` are the corresponding complete row totals; and an absent
trigram coordinate is zero. No evaluation target, weight, fitted scalar,
threshold, teacher, provider, source tensor, H4 state, SpiralCore operator,
prime placement, harmonic field, or payload spelling enters this frame.

The geometric radius is the exact unsigned squared Euclidean radius
`R(c) = x0^2 + x1^2 + x2^2 + x3^2`. The geometric arm selects the tied
candidate with greatest `R`, then lowest token id. The disabled arm selects
the lowest token id exactly as #989. The radius is compiled into a deterministic
overlay bound to the unchanged table CID; its choice arithmetic reads stored
radii and uses integer/table comparisons only. The research/evidence API may
allocate and is not an allocation-free or deployed-serving qualification.

The matched evaluator compiles and loads one shared overlay, evaluates one
active row once, derives both final choices, and assigns both choices the same
declared-work ledger. The disabled choice masks only the overlay winner at the
final comparison. Teacher-forced evaluation must report exact token-support
equality and declared-work-ledger equality at every position. This is not
profiler, wall-time, allocation, or machine-runtime equivalence.
Free-running support and declared-work-ledger equality are claimed only through
the first divergent choice; later histories may lawfully differ.

## Cheap instrument

Before the real held-out run, one focused natural fixture uses only these exact
documents:

```text
construction id 14:   The red fox rests.
construction id 657:  The red fox runs.
construction id 4579: The team runs.
construction id 5121: The athlete runs.
held-out id 13:       At dusk the red fox runs.
seed:                 At dusk the red fox
```

It must establish all of the following with the canonical lexical codec:

1. the decisive active trigram support is exactly ` rests` and ` runs`, with
   equal active counts;
2. the #989/disabled arm chooses ` rests` and the geometric arm chooses
   ` runs` from unchanged support;
3. the two choices expose an identical frame and declared-work ledger;
4. the geometric continuation decodes exactly and terminates on punctuation;
5. the overlay recompiles byte-identically and binds the unchanged table CID;
   and
6. structural teacher/provider/source-weight counters are zero.

Failure stops before real held-out scoring. The historical #953 fixture and
selectors do not run.

## Decision-bearing run

Metric to move: #989 held-out top-1, currently 99,362/446,342. Every known
target is evaluated teacher-forced, but geometry is reachable only where the
unchanged active row has more than one maximum-count candidate. The run must
report that exact reachability count before attaching correctness outcomes.

The positive terminal requires all of the following:

- geometric correct is strictly greater than 99,362;
- among changed choices, geometric-correct exceeds baseline-correct;
- support and the declared-work ledger are identical at all 446,342 teacher-forced
  positions;
- the fixed prompt has at least one geometry-caused first divergence within
  16 emitted units, with an exact pre-divergence frame/work witness;
- the geometric output is distinct, valid UTF-8, at least four units, and has
  no period-1 or period-2 cycle;
- artifact, overlay, report, and complete second-run bytes reproduce exactly;
  and
- all source-closure counters remain zero.

If any condition misses, record `RETAIN_TABLE_BASELINE_GEOMETRY_NO_INCREMENT`,
keep #953 open, keep #973/#954 blocked, and do not try a second formula, axis,
weight, threshold, prompt, or fixture in this run. A positive result closes
#953 through protection and permits #973; it does not itself establish
attention, semantics, correctness, reasoning, chat quality, performance
superiority, formal closure, or release readiness.

Estimated cost is one sub-minute focused fixture plus two complete
construction/held-out executions expected to remain below ten minutes each.
Only the focused #953 test, named core/CLI compile, touched-path formatting,
and the two frozen corpus executions are activated. Broader workspace, BDD,
teacher, Gate C, kappa, audit, fuzz, conformance, product, performance, and
release checks remain `NOT_RUN`.

## Frozen outcome

The predeclared comparison is positive. Each complete JSON report retained the
pre-replay decision `PROCEED_TO_A1Q_H_PENDING_BYTE_IDENTICAL_REPLAY`; exact
external comparison of the two table, overlay, and report files promoted that
pending verdict to the frozen positive terminal.

`MultiscaleCountRadiusR4V1` raised
held-out known-target top-1 from 99,362/446,342 (22.261404%) to
103,604/446,342 (23.211797%), an absolute increment of +0.950392 percentage
points and 4,242 additional correct choices. The binding terminal is
`PROCEED_TO_A1Q_H_WITH_BOUNDED_SOURCE_FREE_GEOMETRIC_GENERATION`.

This establishes causal incremental value only for this construction-only,
fixed-point R4 evidence-radius tie intervention over the frozen table support.
It does not establish semantic coordinates, H4 or SpiralCore transport,
attention, factual correctness, reasoning, broad coherence, chat quality,
performance superiority, formal closure, or release readiness.

## Frozen artifacts and reachability

| Item | Result |
| --- | --- |
| Base table bytes | 35,655,288 |
| Base table CID | `blake3:ccdc399731cb866a329be478467a434cda4e445813421e5d17c21ccc87288297` |
| Overlay bytes | 24,250,680 |
| Overlay CID | `blake3:914126a311c3984d1482258a8f0a7fa2e34896540d502d19f1d9076fbd4a9b76` |
| Report payload CID | `blake3:0234eec7aa8087962b74497dd3df591e6bfb9bf197e84595292948254d30c7fe` |
| Eligible bigram rows | 22,343 |
| Eligible trigram rows | 97,345 |
| Rows whose geometric winner differs | 83,098 |
| Held-out reachable tie positions | 76,641/446,342 (17.170914%) |

The base artifact CID and bytes are identical to #989. The overlay binds that
CID and does not alter `SFTBL001`, lexical ids, the first-nonempty active row,
candidate membership, backoff order, decoding, or cycle handling.

## Held-out causal comparison

| Metric | Disabled/table | Geometric |
| --- | ---: | ---: |
| Correct known targets | 99,362 | 103,604 |
| Top-1 | 22.261404% | 23.211797% |
| Correct among changed choices | 2,511 | 6,753 |

The arms changed 56,280 choices. Geometric-correct exceeded baseline-correct
within that changed set by 4,242. All 446,342 teacher-forced comparisons used
the same active support and declared-work ledger: support mismatches = 0 and
declared-work-ledger mismatches = 0. Structural source-closure counters for
teacher calls, provider calls, source-weight reads, and held-out fitting reads
were all zero; these are fail-closed path counters, not external access-log
telemetry.

## Bounded decoded consequence

The disabled arm reproduced the frozen #989 continuation exactly:

```text
. It is the most important thing to do so.
 1985 – The first people
```

The geometric arm emitted:

```text
. It is the most important thing to do so. The first people to live
```

Both emitted 16 valid UTF-8 lexical units and stopped at the cap without a
period-1 or period-2 cycle. The first divergence was the twelfth emitted unit
(zero-based index 11), at one shared trigram frame with 30 support candidates
and a seven-candidate maximum-count tie. Both arms recorded 30 active-row
entry/count reads, 30 maximum comparisons, 30 tie-membership operations, one
overlay-row read, seven overlay-candidate reads, seven radius comparisons, and
one final-choice operation. Equal support and declared-work-ledger values are
claimed through this first divergence only; the later free-running histories
differ by construction.

These exact bytes show that the intervention changes the bounded decoded path.
They are not a correctness or broad-coherence judgment.

## Replay and activated checks

Two complete corpus-to-table-to-overlay-to-held-out-to-decode executions
produced byte-identical base artifacts, overlays, and JSON reports. Their
SHA-256 witnesses were:

- base table: `129d7061c652cc57616863f7a3a456cd00f1ec841780b672964438fa82767d7d`;
- overlay: `4330c8dc021e83b956c1855874b879cc19758260518f272aac514ff13d08ac67`;
- complete JSON report:
  `8975c8271d9ba6ee23e6f8e66a2c4902636bba446a8f89075e2495e9341e01a3`.

The first complete command took 615.00 seconds including a 39-second debug
build; the already-built replay took 577.28 seconds. The original estimate of
less than ten minutes per execution was exceeded by 15 seconds on the first
pass and met by the replay.

Activated checks:

- frozen natural #953 preflight: 3/3;
- unchanged #989 regression: 3/3;
- focused `uor-r4-core` and `r4` CLI compilation: clean;
- touched Rust formatting and diff whitespace: clean; and
- full frozen corpus run plus complete byte-identical replay: positive.

Allocation-free/deployed-serving behavior, runtime performance, workspace-wide
tests/clippy, BDD, teacher/model paths, no-std ladders, Gate C,
kappa reproduction, audit, fuzz, conformance, product, performance, formal,
and release qualification remain `NOT_RUN`, not PASS.

## Forward boundary

#953 may close through protection at the positive terminal. #973 becomes the
next eligible intelligence issue only after the protected merge and live graph
reconciliation. The accepted input to #973 is the frozen #989 table plus this
one R4 tie overlay and its exact support/declared-work/decode boundary;
historical H4, placement, harmonic, and SpiralCore failures do not become
accepted semantic mechanisms.
