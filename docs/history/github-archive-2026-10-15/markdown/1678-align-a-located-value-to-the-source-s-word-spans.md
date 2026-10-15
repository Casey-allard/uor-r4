# #1678 — Align a located value to the source's word spans

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-04T06:22:50Z
- **Updated:** 2026-10-09T13:17:09Z
- **Closed:** 2026-10-04T07:38:13Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1678
- **Merged:** 2026-10-04T07:38:13Z
- **Merge commit:** 9a88bd0a7dc6a874b0810b77e05cf2d804dbdbe9
- **Base / Head:** main / UOR-Foundation:codex/compiler-word-spans
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Why

`parse_op` **generates** the value text and then locates it with `source.find(value)` (`relation_compiler.rs` ~1190–1208). When generation stops early, `find` matches the **shorter prefix**, and the span is `start..start + value.len()` over whatever was generated — so a truncated value is **stored**: `"penic"` for `penicillin`, `"HSB"` for `HSBC`, `"late"` for `latex`.

Seven of the panel's stored values were such prefixes. The character drop was **1 in four cases and 5 in three** — consistent with generation stopping, and **not** with a token boundary, which a character-level tokenizer would make a 1-character effect every time. (That hypothesis was tested and refuted before this fix.)

## Fix

`align_to_words` widens the located range to the union of the source's **word spans** it overlaps. This restores the whole word because the compiler's contract already says a value is the world's own words cut from the unchanged source (`word_spans`, and the existing test `word_spans_are_the_worlds_words_cut_from_the_source`).

Alignment only ever widens **within words the match already touched**, so it cannot swallow a following word — asserted by `alignment_does_not_swallow_the_following_word`. Inverted and out-of-range offsets are clamped and normalised, so the returned pair always satisfies `lo <= hi`.

## Measured — disjoint panel, 8M, 100 rows, seed 9101 (off and sieve identical)

| | truncations | asked_stored | distractor_only | never_stored |
|---|---:|---:|---:|---:|
| baseline (main) | **7** | **50** | **42** | **49** |
| class 1 | **2** | **55** | **37** | **44** |

## Measured — world regression guard, 60 conversations, seed 9101

```
MQAR 22/23; open relation 11/12; closed relation 3/3; errors 0
```

**Identical to the baseline on main**, so the known-relation cells do not regress.

## Decision rule (pre-registered in the class-1 card)

| rule | result |
|---|---|
| truncations → 0 | ❌ **FAILS — 2 remain** |
| `asked_fact_stored` above 50 | ✅ 50 → **55** |
| `distractor_only_written` falls | ✅ 42 → **37** |
| world cells do not regress | ✅ identical |

## Why it is not a full pass, and what that shows

Word alignment **cannot** reach the two remaining cases, because the generated text is missing words **in the middle**, not at a boundary:

```
"seven zero six"  generated as "seven zer"   -> alignment yields "seven zero"
"three eight five" generated as "three eight" -> alignment yields "three eight"
```

Pinned by `alignment_floor_cannot_restore_an_untouched_word`, which asserts the result
is `"seven zero"` **and not** `"seven zero six"`. If a later change makes that test
fail, the floor has moved.

`"six"` was never touched by the match, so no word-level rule can recover it. **The first fix has a hard floor, and the finding is that generation loses mid-phrase content, not merely stops one word early.** Reaching the last two needs the **generation** side, not span arithmetic.

**Also recorded, not explained away:** `question_not_a_query` **rose 32 → 36** under this change. My decision rule did not anticipate that direction. The world regression guard is clean and `asked_fact_stored` still rose, so the change is a net gain — but the rise is real and unexplained here.

## Scope

**Rebuilt on current main after #1673 merged (`c87817ee`), so the diff is now this
PR's own change only.**

| file | change |
|---|---|
| `crates/uor-r4-training/src/relation_compiler.rs` | `match_at_word_start`, `align_to_words`, parse-op tests |
| `crates/uor-r4-training/examples/m-world.rs` | **+5 lines**: the `value_location: "word-aligned/1"` marker |

```
crates/uor-r4-training/examples/m-world.rs      |   5 +
crates/uor-r4-training/src/relation_compiler.rs | 196 ++++++++++++++++++++++--
```

`cargo fmt --all --check` clean; relation_compiler suite 20/20.

## Word-start matching, and why it is numerically neutral

A plain substring search could match **inside** a longer word, and widening made that
worse: a generated `"pen"` matching inside `"spend"` would be stored as the whole word
`"spend"` rather than as a truncation of the intended one. `match_at_word_start` now
requires the match to **begin where a word begins**.

**On this panel the guard changes nothing** — no row matches inside a longer word:

| build | truncations | asked_stored | distractor_only | question_n_a_q |
|---|---:|---:|---:|---:|
| baseline (main) | 7 | 50 | 42 | 32 |
| class-1 initial | 2 | 55 | 37 | 36 |
| class-1 + word-start | 2 | 55 | 37 | 36 |

It is recorded as **neutral**, not as a gain — it closes a defect the panel does not
exercise.

## Versioning

The parse rule changes which spans a turn stores **without changing the compiler
identity**, so re-running an older sealed report would silently produce different
values. The report now records **`value_location: "word-aligned/1"`**.

## The `question_not_a_query` rise is explained, not unexplained

My earlier revision recorded the 32 → 36 rise as unexplained. **It is mechanical, and
the moved rows confirm it.** `diagnoses_row` reaches `question_not_a_query` only when
`stored` is true, so rows whose value was previously truncated (not stored) sat in
`distractor_only_written` and could not reach it. Alignment stored them, and they
re-classified:

```
rows whose diagnosis changed: 5
  distractor_only_written -> question_not_a_query   4   (dp-011, dp-014, dp-062, dp-091)
  distractor_only_written -> query_but_no_read      1
all 4 were stored=false before and stored=true after.
```

**Those rows were always failing their question turn**; the change moved them from one
failure label to the correct one. The diagnosis is more accurate, not worse.

References #1552




## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5977605356 — 2026-10-04T07:13:01Z — Casey-allard

Exact-head review at 46ba5a42dc7b0bb66f6ebaee5b0f4ade18113712. The code is approved. The parse_op doc is one block and describes the word-start match and the word-widened span. The diff touches only m-world.rs (+5) and relation_compiler.rs. Checked locally from a git archive of this head: cargo fmt --check passes, and `cargo test -p uor-r4-training --lib relation_compiler` passes 20/20. **Merge held as pre-registered:** this queues only after the ~96M fine-tunes are scored with the pre-#1678 build.


---
_Comment/review ids in this file: 5977605356_

