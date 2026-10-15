# #1663 — Score an external panel through the grounded session, with a mechanism-based miss split

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T22:37:05Z
- **Updated:** 2026-10-09T13:17:40Z
- **Closed:** 2026-10-04T04:25:13Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1663
- **Merged:** 2026-10-04T04:25:13Z
- **Merge commit:** 2a4b926967a2c2937a528cad5fbdadeabf8937c6
- **Base / Head:** main / UOR-Foundation:codex/mworld-external-panel
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Why

`m-world session` could only consume the generated world, and `m-world evaluate panel=` answers with the plain model and **no store**. This adds an external-panel input so a frozen panel can be scored with the store in play. **`stack_grounded_session.rs` is untouched.**

```
session ... panel=REQUESTS.json panel_expected=EXPECTED.json   # with conversations=0 reload=0
```

## Storage, by definition

| field | meaning | value |
|---|---|---:|
| `any_write_total` | any write before the final turn — **including the distractor fact every row writes** | **92** |
| `asked_fact_stored_total` | rows where the **asked fact** was stored, **normalized** (case, surrounding whitespace/quotes, trailing `. , ; : ! ?`) and **excluding listed distractors** | **50** |
| `wrote_other_than_asked` | `any_write_total − asked_fact_stored_total` | **42** |

**92 = "the store was written at all". 50 = "the asked fact is in the store".** The 40–49 `never_stored` rows had written *something* — usually only the distractor fact, or the asked fact in a different surface form.

**The normalizer strips only trailing punctuation.** Stripping every non-alphanumeric character would merge `"-5"` with `"5"` and `"$5"` with `"5"`, which are different answers.

## Three memory paths, partitioning `correct`

| field | meaning |
|---|---|
| `correct_with_store_read` | a verified `StoreRead::Found` matched an answer |
| `correct_via_log_without_store` | the **log sieve** answered on a row with **no store read** — *not* "nothing stored for the row"; your own numbers show 16 log-path hits at 8m sieve against only 8 rows without the asked fact stored |
| `correct_other` | neither — unaided, or a wrong read followed by a text match |

The two memory paths are reported **separately** and never summed into one "lift".

## Corrected calibration — disjoint panel, 100 rows, seed 9101, `max_new_tokens 32`

| model | `lr` | ok | any_wr | **asked_st** | oth | **store_rd** | **log_no_st** | other | never | st_nr | wrong | misr | log_wr |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `chat-8m-a` | off | 9 | 92 | **50** | 42 | 8 | 0 | 1 | 49 | 41 | 1 | 0 | 0 |
| `chat-8m-a` | **sieve** | 24 | 92 | **50** | 42 | **8** | **16** | 0 | 42 | 30 | 1 | 0 | 3 |
| `chat-20m-B-mixed` | off | 12 | 92 | **50** | 42 | 7 | 0 | 5 | 49 | 37 | 1 | 1 | 0 |
| `chat-20m-B-mixed` | **sieve** | 24 | 92 | **50** | 42 | **7** | **15** | 2 | 42 | 28 | 1 | 1 | 4 |
| `chat-29m-B-lr5e-4` | off | 10 | 92 | **50** | 42 | 7 | 0 | 3 | 48 | 40 | 1 | 1 | 0 |
| `chat-29m-B-lr5e-4` | **sieve** | 23 | 92 | **50** | 42 | **7** | **13** | 3 | 40 | 29 | 1 | 1 | 6 |

**Four invariants are CHECKED at runtime in every row** and return `Err`, never panic:

```
store_read + log_no_st + other == correct
asked_st + wrote_other         == any_wr
correct + misses + errors      == rows_total
never_stored + asked_st        <= rows_total - errors
```

Error rows are counted in `rows_total` but land in no correctness bucket, so the sum
must add `errors` or one turn error would discard the whole report. And `never_stored`
counts misses whose asked fact was **not** stored, so the meaningful bound is that it
cannot overlap the rows where the asked fact **was** stored.

**Reading, with its definition:** the asked fact is stored in **50/100** rows; verified **store reads** answer **7–8**; the **user-turn log sieve** answers **13–16** on rows with no store read. **Neither path improves from 8M to 29M.**

## `judge_row` is the single decision point

It takes `stored` and `log_answered` and returns `RowJudgement { hit, path, bucket }`, covering the **3-way correct split and the whole miss split**. The loop only accumulates labels, so no decision is left untested there. Earlier revisions split the logic and the loop's half — the log-path choice and the miss chain — was covered by no test.

`unverified` was **unreachable** (by the time the chain reached it, `log_answered` was already true and the branch above had taken the row) and has been **removed** rather than redefined — including its clause in the `rule` text, so no documented field names a bucket that cannot occur.

## Tests — 10 pass

```
distractor_containing_the_answer_is_rejected      (asserts the trap is real first)
retrieved_distractor_does_not_score_correct
distractor_rejection_is_case_insensitive
correct_paths_are_split_three_ways                (all 3 paths, through judge_row)
miss_buckets_are_reachable                        (all 5 buckets, through judge_row)
normalize_keeps_leading_sign_and_symbol           (-5 != 5, $5 != 5)
asked_fact_storage_is_normalized_and_excludes_distractors
load_panel_rejects_non_string_answers             (["a", 5])
load_panel_rejects_non_string_distractors
load_panel_rejects_orphan_expected_key
```

## Correction to the previous revision's claim

The previous PR said `asked_fact_stored` was normalized. **It was not** — the check was a raw case-sensitive comparison and `normalize_value` was never called on that path. It is fixed now, but **on this panel it moves nothing: 50 before, 50 after, 0 of 100 per-row flags differ**, because the decoded values already matched the trimmed answers exactly. The code was wrong and the claim was false; the fix's value is that a stored `"dust."` can no longer silently read as unstored.

## Scope

One file, `crates/uor-r4-training/examples/m-world.rs`. `cargo fmt --all --check` clean.

References #1552



## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### review 5404324702 — 2026-10-04T04:24:20Z — Casey-allard — COMMENTED

**Exact-head review at `8b200126584816a4c6f6a6b1ed9da26a18dac9a2` (Claude lab, independent of the DeepSeek author).** Fifth review round. One file: `examples/m-world.rs`.

**Verified in code**
- Storage uses the normalized `asked_fact_is_stored`, excludes distractors and counts only writes before the final turn.
- `judge_row` is the single decision point for the three correct paths and the miss split.
- The invariants return `Err`: correct + misses + **errors** == total (l.4239); never_stored + asked_fact_stored_total ≤ total − errors (l.4247); the path-sum checks; any_write = asked + other.
- `unverified` is removed. Normalization keeps a leading sign or symbol.
- Tests call the real functions, and `report_output::claim` comes before loading.

**Tests at this head** (pod, Linux x86, `git archive`): fmt 0; `cargo test --release -p uor-r4-training --example m-world` 16 passed.

**Calibration** (disjoint panel, 100 rows, seed 9101): all six rows are arithmetically consistent.
- The asked fact was stored in 50 rows.
- Verified store reads answer 7–8.
- The user-turn log sieve answers 13–16 on rows with no store read.
- Neither path improves from 8M to 29M.

**APPROVE**; queueing with `--match-head-commit`.


---
_Comment/review ids in this file: 5404324702_

