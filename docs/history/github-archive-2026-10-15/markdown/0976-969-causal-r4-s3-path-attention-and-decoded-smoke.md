# #976 — #969: causal R4/S3 path attention and decoded smoke

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-27T18:53:19Z
- **Updated:** 2026-10-09T14:17:22Z
- **Closed:** 2026-08-27T18:54:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/976
- **Merged:** 2026-08-27T18:54:51Z
- **Merge commit:** bed6a94a1bc4c0597046f8a7020b4e3c78b1dfa4
- **Base / Head:** main / UOR-Foundation:issue-969-local-geometric-attention
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #969

## Outcome

Implements one causal R4/S3 route-attention mechanism and exercises it through one fixed, two-unit decoded smoke. The result is:

`PROCEED_TO_I1_WITH_CAUSAL_R4_PATH_ATTENTION`

This is a load-bearing identity-derived local mechanism result, not a semantic-intelligence or coherent-generation claim.

## Roadmap correction

- Re-centers #969 on the original Prime R4 Router objective: natural sparse route adjacency, an ordered unit-quaternion path on S3, retained causal prefixes, and deterministic least-cost continuation.
- Removes the abandoned pre-mechanism channel-census/qualification programme from living guidance.
- Keeps paired-H4/E8, zeta, n-let, Hopf, radial, and other stored fields structural or future hypotheses.
- Leaves #953 as the next source-free generation stage.
- Leaves #973 after accepted #953 for paragraph/conversation/global qualification; #954 remains behind both #953 and #973.

## Frozen mechanism

For admitted candidate `c`:

- `P(0) = identity`
- `P(k+1) = P(k) composed with route(x_k)`
- retained `P(0)..P(t-1)` are causal memory
- `Q_t(c) = P(t) composed with route(c)`
- rank by the minimum exact `(round-S3 angular shell, causal lease age)` of `K^-1 Q_t(c)`
- exact candidate-cost ties abstain

The existing schema-2 lookup supplies natural support unchanged. The exact binary-icosahedral H4 table is used only as a finite S3 unit-quaternion codebook. No target, future route, stored continuation, digest distance, learned readout, teacher output, Ollama, or hosted provider enters selection.

## Fixed smoke and identities

Matched histories:

- left: `aa bb dd qq`
- right: `bb aa dd qq`

Both have the same length, multiset, last-two suffix, natural `{ll, rr}` union, source counts, row reads, payloads, and candidate/key group-comparison budget.

Bound kappas:

- reused #952 construction artifact: `blake3:2b70588d654c8e8bb2d8ab063f41853d45a21487d742ff7567f93a42cfb9011b`
- attention manifest: `blake3:1c77c4103732964af6776f1dfcabc8b2a9191eea875a8ba205c36ebbf5618a99`
- H4 root table: `blake3:8d33d62a239fb8001fea2bd14a9a5ec7321d0f07d81c74a5715eaeb3df53aa76`
- H4 multiplication table: `blake3:90ee73a27ee2e8ba5bccd1507d7fb37ed1f044b1640772c86752bc0bb2111759`
- smoke fixture: `blake3:cc36b703b95bf1da11f2691ed91bbe94a81a4385f0eff8483cb9402191f46332`
- canonical serialized smoke record: `blake3:60360a9e22a56ea4af363e43f7103bb8104d015d58feb582d921fc17afaf207f`

No separate construction/validation split exists after the issue pivot; the fixed smoke is a mechanism demonstration and makes no transfer claim.

## Exact result and controls

- full retained path: `aa bb dd qq -> rr ll`; `bb aa dd qq -> ll rr`
- both first winners use retained non-identity prefix index 1
- first query: 7 schema-2 rows, 2 entries, 2 candidates, 8 path comparisons
- second query: same support, 10 path comparisons
- admitted payload inversion: 8/8
- selected exact decode: 4/4
- incremental causal-path rebuild equality: 4/4
- deterministic double execution: byte-identical record and pinned kappa
- last-only: ties and abstains on both first queries
- state-disabled: selects `rr` on both first queries and cannot reproduce the incompatible outputs
- hierarchy replay: final six-unit decoded sequences reproduce their current-route identity with at most two changed nodes/event; this is post-selection replay, and only causal path state feeds the second choice

The three arms have identical natural support and identical candidate/key group-comparison budgets. This is not a claim of measured identical CPU cycles.

## Focused verification

- `local_geometric_attention_969`: 1/1
- `recursive_geometric_attention_952`: 2/2
- `recursive_geometric_attention_967`: 2/2
- `recursive_geometric_attention_970`: 1/1
- focused `uor-r4-core` all-target compilation
- focused source-closure check
- WASM library check
- `cargo fmt --check`
- claim-wording validation
- `git diff --check`

NOT RUN by scope: workspace-wide tests/clippy, broad corpus compile, teacher/model run, generation canary, correctness, reasoning, chat/product QA, optimization, formal closure, and release QA. Protected merge CI remains the integration verdict.

## Nonclaims

This does not establish learned or lexical semantics, useful or coherent free-running generation, knowledge, correctness, reasoning, paragraph/conversation/global attention, performance advantage, chat quality, or release readiness. #953 must now determine whether the local mechanism can participate in actual source-free language.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

