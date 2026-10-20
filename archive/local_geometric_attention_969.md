# #969 causal R4/S3 path-attention prototype

- **Issue:** [#969](https://github.com/UOR-Foundation/uor-r4/issues/969)
- **Date:** 2026-08-27
- **Verdict:** `PROCEED_TO_I1_WITH_CAUSAL_R4_PATH_ATTENTION`
- **Scope:** one local mechanism and one two-unit decoded smoke

## Why this is the pivot

The original Prime R4 Router paper describes the useful mechanism spine:
natural sparse route adjacency, bounded R4 state normalized on S3, causal
transport, and deterministic least-energy continuation. The original
`Casey-allard/prime-router` implementation did not implement a least-geodesic
candidate search: its window router chose maximum projection norm, its lexical
choice used trigram/cosine/repetition terms, Hopf values were recorded after
selection, and Ollama could author language.

#969 therefore implements the missing source-free mechanism rather than another
pre-mechanism qualification framework. H4 is not presented as a result from the
paper. Its 120 binary-icosahedral roots are used here only as an exact finite
unit-quaternion codebook on S3. The golden-coupled `H4 + phi H4` / E8 state
remains structural storage and control, not the attention score.

## Frozen mechanism

For observed routes `x_1 ... x_t`:

```text
A(i,j)  = existing natural schema-2 candidate adjacency
P(0)    = identity
P(k+1)  = P(k) composed with route(x_k)
M(t)    = P(0) ... P(t-1)
Q_t(c)  = P(t) composed with route(c)
cost(c) = minimum (exact round-S3 angular shell, causal lease age)
          over K in M(t), using K^-1 composed with Q_t(c)
```

The exact signed real coordinate of `K^-1 Q` orders the nine H4 angular shells
from coincident through antipodal. Lease age breaks equal-shell comparisons for
one candidate. Equal minimum costs across candidates abstain; canonical address
order only stabilizes trace order. Candidate admission, payload identity, and
the existing schema-2 rows are unchanged.

The full arm uses every retained prefix before the current state. Last-only
repeats `P(t-1)` and algebraically reduces the relative query to the last route
composed with the candidate. State-disabled repeats identity. All three arms
perform the same candidate/key comparison count and H4 table-operation shape;
this is an equal group-comparison budget, not a measured claim of identical CPU
cycles.

## Bound identities

| Object | Kappa |
|---|---|
| Reused #952 construction artifact | `blake3:2b70588d654c8e8bb2d8ab063f41853d45a21487d742ff7567f93a42cfb9011b` |
| Embedded attention manifest | `blake3:1c77c4103732964af6776f1dfcabc8b2a9191eea875a8ba205c36ebbf5618a99` |
| H4 root table | `blake3:8d33d62a239fb8001fea2bd14a9a5ec7321d0f07d81c74a5715eaeb3df53aa76` |
| H4 multiplication table | `blake3:90ee73a27ee2e8ba5bccd1507d7fb37ed1f044b1640772c86752bc0bb2111759` |
| Smoke fixture | `blake3:cc36b703b95bf1da11f2691ed91bbe94a81a4385f0eff8483cb9402191f46332` |
| Canonical smoke record | `blake3:60360a9e22a56ea4af363e43f7103bb8104d015d58feb582d921fc17afaf207f` |

The smoke fixture contains no target labels or continuation rows. It binds only
the two histories, natural candidate names, and two-unit bound.

## Decoded smoke

The matched histories are:

```text
left:  aa bb dd qq
right: bb aa dd qq
```

They have the same length, multiset, last-two suffix, natural candidate union,
support counts, and group-comparison budget. Neither complete history occurs in
the construction sentences. On both steps the natural union is exactly
`{ll, rr}`; seven schema-2 rows read two entries, and each candidate has only
one adjacent-spin support count.

| Step | Left full path | Right full path | Comparisons/query |
|---|---|---|---:|
| 1 | `rr` — 36 degrees, age 4, retained prefix 1 | `ll` — 36 degrees, age 4, retained prefix 1 | 8 |
| 2 | `ll` — 36 degrees, age 6, identity prefix | `rr` — 36 degrees, age 5, retained prefix 1 | 10 |

The exact decoded continuations are therefore:

```text
aa bb dd qq -> rr ll
bb aa dd qq -> ll rr
```

Both first choices depend on non-identity retained prefixes. Last-only ties and
abstains on both first-step queries. State-disabled selects `rr` on both, so it
cannot reproduce the incompatible full-path outputs. All 8/8 admitted candidate
values invert to exact payload bytes and all 4/4 selected values decode. Each
selected route updates the causal path state; all 4/4 incremental path updates
equal a clean rebuild, and no exact prefix state repeats in either bounded
trajectory.

The canonical hierarchy cursor was also replayed over each final decoded
six-unit sequence and reproduced its final current-route identity with at most
two changed hierarchy nodes per event. This is a post-selection hierarchy
replay; only the incremental causal path state feeds the second selection. A
fully appendable hierarchy cursor belongs to the later decoded engine, not this
prototype.

Two complete executions produced byte-identical canonical smoke-record bytes
and the record kappa above.

## Decision and nonclaims

The matched intervention demonstrates that causal route geometry is
load-bearing: changing only earlier route order changes candidate choice and a
two-unit exact decoded continuation, while natural support and the bounded
group-comparison budget remain matched and the two reduced-state controls are
weaker. This is the first mechanism result authorized by the #969 pivot.

It does **not** establish learned or lexical semantics, coherent free-running
generation, knowledge, correctness, reasoning, paragraph/conversation/global
attention, performance advantage, chat quality, or release readiness. The
current lexical-to-H4 placement remains identity-derived. #953 must now test
whether this mechanism can participate in actual source-free language; #973
alone may later qualify higher-scope state through that decoded loop.
