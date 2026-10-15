# #1709 — Step 0b: tokenizer/format key-value tuple audit of the D19 draw (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T02:32:41Z
- **Updated:** 2026-10-09T13:08:49Z
- **Closed:** 2026-10-05T03:25:10Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1709
- **Merged:** 2026-10-05T03:25:10Z
- **Merge commit:** bfc0c1254d8e582ea02df7c65fbdd14bb55d1e01
- **Base / Head:** main / UOR-Foundation:codex/step0b-key-tuple-audit
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Step 0b of the 2026-10-05 barrier assessment (`docs/research/barrier-assessment-2026-10-05/final-assessment.md`, branch `codex/barrier-assessment-20261005`). This is a training-free, model-free instrument. It adds one Rust bin, `crates/uor-r4-training/src/bin/step0b-key-tuple-audit.rs`.

## What it measures
- **Draw.** It regenerates the `m-world session world=v2` draw through the generator's own API: development split, default mix, seed 9101, 300 conversations, 109 MQAR queries. **Identity check:** all 1075 of 1075 user turns equal the recorded `session-chat-8m-a` report (sha256 `64a1336a…`).
- **Tokenizer.** The production #1017 tokenizer (`blake3:3f42bcfc…`, sha256 `d36d3e87…`).
- **Encoding.** Each content is encoded exactly as `DialogueProtocol` encodes it. Protocol v2 is `" " + content` and is what the chat emitters use. Protocol v1 is the content alone. Occurrences are taken by byte span, and the m-tuple is the first m pieces.
- **Positions.** A = the assertion. Q = the question. R = the expected reply. M = the sieve recall line `Memory: {v}.`.
- **Mismatch causes.** Each mismatch is classified as leading space, casing, both, punctuation attachment, or segmentation.

Report root (claimed exclusively, sealed with BLAKE3 `manifest.json` and verified): `~/uor-r4-local/step0/b-tokenizer/`. It contains `report.json` (sha256 `c5926442…`), `rows-v2.jsonl` (sha256 `f33090b1…`) and `sha256-manifest.txt`.

## Results (D19 draw, protocol v2 unless stated)
| Pair | of | mismatch m=1 / 2 / 3 | cause |
|---|---|---|---|
| MQAR key A vs Q | 109 | 0 / 0 / 0 | – |
| MQAR key A vs R (rehearsing replies only) | 41 | **41 / 41 / 41** | casing (`' l'` vs `' L'`) |
| MQAR value A vs R | 109 | 0 / 0 / 0 | – |
| MQAR value A vs recall line M | 109 | 0 / 0 / 0 | – |
| Relation value A vs R (open / closed) | 52 / 17 | 0 / 0 / 0 | – |
| Copy first word, user vs reply | 33 | 0 / 0 / 0 | – |
| Copy first word, **protocol v1** | 33 | 33 / 33 / 33 | leading space |
| MQAR key A vs R, **protocol v1** | 41 | 41 / 41 / 41 | space and casing |

- **Where the key mismatch sits.** Of the 41 rehearsed keys, 26 differ only in their first piece (`' l','u','k',…` vs `' L','u','k',…`). The other 15 are segmented differently after capitalization.
- **Cause.** The generator `capitalize`s the expected reply (`milestone_world_v2.rs:1446`), so a rehearsing reply starts with the key in a casing that never occurs in the assertion or the question.

**Expected reply forms (D19, 109 rows):**

| Form | Rows | Share |
|---|---|---|
| Rehearse `{K} is {v}.` | 41 | 37.6% |
| `It's {v}.` | 30 | 27.5% |
| `That's {v}.` | 38 | 34.9% |

The compact draws pick the shortest reply, which biases the long-distance cells towards bare forms. Bare replies (63%) never restate the key, so the key's tuple cannot be matched at the reply start in those rows.

**Train-split draw** (seed 1, 5600 conversations; these are the corpus defaults, not the exact training corpus): 743 of 2008 MQAR replies rehearse (37%). All 743 mismatch the assertion key by casing. Values and relations match 100%.

**Observed `chat-8m-a` session replies** (sieve on), 109 MQAR rows:

| Form | Rows | Passing |
|---|---|---|
| Rehearse | 21 | 20 |
| `It's` | 28 | 27 |
| `That's` | 39 | 39 |
| Other | 21 | 21 |

## Decision rule (frozen in the assessment)
The denominator is every MQAR value tuple (A vs R) plus every key tuple at the reply start: 150 tuples. Mismatches are 41 / 41 / 41 at m = 1 / 2 / 3, which is **27.3% (> 10%)**. Counted per fact, 41 of 109 facts (37.6%) contain a mismatched tuple.

**Outcome: copy boundary fix (Step 4) must precede chat lineage verdicts.** The defect is narrow:
- It is the casing of the rehearsed key at the reply start, plus the leading space under protocol v1 only.
- Values, recall lines, relations and copy all match under protocol v2.
- A Step 4 fix can be either of two things. One is the generator emitting the key in its asserted casing (or a lower-case rehearsal). The other is a case-folding key match in the copy path.

## Caveats
- This is a tokenization audit of the reference text. It does not measure what the network does.
- The train-split draw is not the exact training corpus.
- Positions are measured per content segment. That is exact under `DialogueProtocol`, which encodes each content separately.
- The run used the binary built before `cargo fmt`. The fmt change is whitespace only.

## Checks at this head
- `cargo fmt --check -p uor-r4-training`: pass.
- `cargo test --release -p uor-r4-training --bin step0b-key-tuple-audit`: 2 passed.
- Release build: ok.

References #820

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5987815032 — 2026-10-05T03:55:31Z — Casey-allard

Post-merge note (Claude lab): this PR entered the merge queue without a recorded exact-head review; it was not queued by the Claude lab. **Checks reported by its author agent at the merged head:**
- `cargo fmt --check` passes;
- focused tests pass, or there is no Rust to test (0a is docs and labels);
- the report root is sealed.

The result is accepted as the Step 0 instrument record (see #820). Any defect found later is fixed forward. References #820


---
_Comment/review ids in this file: 5987815032_

