# #989 — B0: source-free table-native lexical baseline from corpus to decoded text

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-28T13:11:03Z
- **Updated:** 2026-08-29T01:08:20Z
- **Closed:** 2026-08-28T13:40:17Z
- **Labels:** enhancement
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/989
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Ship the smallest working source-free language baseline before any further geometric attention or algebra qualification:

```text
JSONL text corpus
-> deterministic streaming lexical routes
-> construction-only integer count tables
-> held-out next-route prediction
-> exact route-to-bytes decoding
-> bounded decoded continuation
```

This issue is the sole **Now** item under #820. It deliberately freezes #953, #973, and every new H4/SpiralCore/harmonic/attention qualifier until a plain table model learns something from text and says something.

## Product boundary

Implement one reusable Rust component and one visible CLI command, provisionally:

- `uor_r4_core::source_free_table::SourceFreeTableModel`
- `r4 source-free-table --corpus <articles.jsonl> --prompt "The United States" --continuation-cap 16 --json`

The component must:

1. segment UTF-8 with the existing canonical lexical-run rule, without the 64 KiB/512-unit conversation-artifact ceiling;
2. bind each lexical payload to one deterministic route ID and preserve exact route-to-bytes inversion;
3. partition documents before training with the existing D3 article-ID rule;
4. reject construction/held-out document-ID or complete-text-CID overlap;
5. compile construction-only unigram, bigram, and trigram count tables in deterministic order;
6. predict with integer table reads and comparisons only: trigram, then bigram, then unigram, with one canonical tie rule;
7. score actual next routes from held-out documents without letting them alter vocabulary, counts, candidates, or selection;
8. emit one bounded UTF-8 continuation from the same predictor and exact payload table; and
9. reproduce artifact and report bytes exactly on a second build/run.

Exact n-gram context reuse is allowed and must be reported honestly: this is a statistical lexical baseline, not semantic attention. Complete document duplication across partitions is forbidden.

## Frozen product decision

### Cheap check

One tiny natural, document-disjoint fixture must exercise compile -> held-out score -> decode and satisfy all of the following before the real corpus is opened:

- table top-1 correct count is greater than unigram top-1 correct count;
- at least one bigram or trigram row serves a held-out decision;
- the generated continuation contains at least four lexical units, is valid UTF-8, and has no period-1 or period-2 cycle;
- artifact and report bytes reproduce exactly; and
- teacher/provider/source-weight/geometry counters are all zero.

If this check fails, stop and repair this one product path. Do not create a new harness or activate geometry.

### One bounded SimpleWiki run

Use the already pinned 3,000-document SimpleWiki JSONL corpus and its D3 split. Freeze the prompt as `The United States` and the continuation cap as 16 before measurement.

The baseline works only if:

- the input corpus CID reproduces its manifest;
- all 3,000 unique article IDs are assigned exactly once to construction or held-out;
- at least 100,000 held-out known-target positions are scored;
- context-table coverage is nonzero;
- table top-1 accuracy exceeds unigram top-1 accuracy by at least 5 percentage points;
- the decoded continuation contains at least four lexical units, is valid UTF-8, and has no period-1 or period-2 cycle; and
- two complete compile/evaluate/generate executions reproduce artifact and report bytes.

If positive: keep the table baseline as the reference engine and permit exactly one later geometric intervention to demonstrate incremental value against it. If negative: keep geometry frozen and revise lexical representation/objective within this issue; do not add algebra, controls, or corpus size.

## Run contract

```text
metric to move:       held-out table top-1 minus construction-unigram top-1; no current source-free corpus-to-text baseline
reachability ceiling: 100% of held-out positions whose target payload exists in construction vocabulary; report the actual known-target denominator and context coverage
cheap instrument:     tiny document-disjoint compile/score/decode check; must beat unigram and emit >=4 units before opening SimpleWiki
exit threshold:       >=100,000 scored positions, nonzero context coverage, >=5pp top-1 uplift, decoded >=4 units, deterministic replay
if positive:          freeze this table model as the reference baseline; geometry may later compete only as an intervention against it
if negative:          repair the lexical representation or causal count objective here; geometry remains frozen
cost estimate:        under five minutes and bounded by 3,000 documents; no teacher/model/corpus-observation run
```

## Activated checks

Only these checks are activated for this product decision:

- the focused `source_free_table_baseline` test with its printed tiny product transcript;
- a focused compile of the touched core and `r4` CLI surfaces;
- one real 3,000-document command executed twice; and
- formatting of the touched Rust files.

Workspace, BDD, teacher, graph, Gate C, kappa, audit, fuzz, WASM, conformance, formal, performance, product-chat, and release suites remain `NOT_RUN`.

## Definition of done

- The visible CLI accepts the pinned JSONL corpus and emits a deterministic artifact/report identity, held-out counts, table-versus-unigram result, and decoded continuation.
- The exact tiny and SimpleWiki outcomes are recorded without attention or intelligence inflation.
- Current roadmap/programme prose makes this the only **Now** item and moves new geometry/attention qualification to **Later**, conditional on a working baseline.
- #953 remains an untouched integration regression and is natively blocked by this issue.
- Delivery uses a focused PR and the protected queue as transport only.

## Nonclaims

Success establishes only a source-free corpus-trained lexical table baseline with held-out next-route evidence and bounded decoding. It does not establish semantics, attention, geometry, broad coherence, factual correctness, reasoning, chat quality, performance superiority, multiplication-free serving, or release readiness.


## Final protected closure reconciliation (2026-08-28)

Protected PR #990 closed #989 at `ESTABLISH_TABLE_NATIVE_LEXICAL_BASELINE`.
The source-free construction-only table scored 99,362/446,342 = 22.261404%
held-out known-target top-1 versus 24,163/446,342 = 5.413561% unigram. Two
complete runs reproduced the 35,655,288-byte artifact
`blake3:ccdc399731cb866a329be478467a434cda4e445813421e5d17c21ccc87288297`
and reports byte identically. #989 is closed; earlier active/blocker language
above is historical. Its artifact remains the frozen non-geometric comparator,
not semantics, attention, correctness, reasoning, chat, or release evidence.


## Comments (1)

### comment 5453248222 — 2026-08-28T13:42:13Z — Casey-allard

Protected closure: `ESTABLISH_TABLE_NATIVE_LEXICAL_BASELINE`.

PR #990 merged through the required queue at `a5745a1d2e8b1de420b2dd7a3edaf699f612847a`.

Evidence:
- pinned corpus and manifest CIDs reproduced; 3,000 unique documents split D3 into 2,404 construction and 596 held out;
- 446,342 held-out known-target positions;
- table top-1 99,362/446,342 = 22.261404%; unigram 24,163/446,342 = 5.413561%; uplift +16.847843 percentage points;
- fixed prompt emitted 16 valid UTF-8 units with no period-1/2 cycle;
- two complete runs produced byte-identical reports and 35,655,288-byte artifacts at `blake3:ccdc399731cb866a329be478467a434cda4e445813421e5d17c21ccc87288297`;
- focused natural test 3/3, focused core/CLI compile clean, and named Rust formatting clean.

All broad/workspace/BDD/teacher/graph/Gate C/kappa/audit/fuzz/WASM/conformance/formal/performance/product-chat/release checks were `NOT_RUN`. The five required GitHub names were queue transport acknowledgements, not QA or PASS evidence.

This establishes only a construction-trained statistical lexical table baseline with held-out next-route evidence and bounded exact decoding. It does not establish semantics, attention, geometry, correctness, reasoning, chat quality, multiplication-free serving, performance superiority, or release readiness.

The frozen table reference now permits exactly one later #953 matched geometric intervention. #953 was not started in this issue.


---
_Comment/review ids in this file: 5453248222_

