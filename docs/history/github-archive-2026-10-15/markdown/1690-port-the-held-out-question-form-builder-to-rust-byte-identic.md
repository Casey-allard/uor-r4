# #1690 — Port the held-out question-form builder to Rust, byte-identical

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-04T16:02:17Z
- **Updated:** 2026-10-09T13:17:48Z
- **Closed:** 2026-10-04T17:10:28Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1690
- **Merged:** 2026-10-04T17:10:28Z
- **Merge commit:** 689e8787099913de063e66a56f77f7d5c69360c8
- **Base / Head:** main / UOR-Foundation:codex/qform-set-rust
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Why

Rust port of `build-qforms.py`, on the #1659 pattern: **the bar is byte identity** with the frozen JSON, so the held-out set stays reproducible from committed source. **This lands independently of whether v26 passes** — a frozen instrument should not depend on the experiment it measures.

```
$ qform-set verify dir=~/uor-r4-local/ladder/qform-heldout
  rows regenerated: 100
  requests byte-identical: true
  expected byte-identical: true
  assertion failures: 0
```

```
$ qform-set build out=/tmp/qf-rust
  23429064f03b7a6f70de3a6ac4ba9449829534a4e38140328d8206dbb0178549  qform-requests.json
  82175ceda2903d0b3da11558f10d0e05b477191479e9f7857580ce763fe1673d  qform-expected.json
```

Both match the frozen set exactly.

## One audited RNG, shared

The port **reuses the verified CPython Mersenne Twister (`init_by_array`) and the `json.dumps(indent=1)` emitter from `disjoint-panel`**, so the two instrument builders share one RNG: a future change to it cannot silently diverge between them. Byte identity requires CPython's seeding, which is unrelated to the crate's `splitmix64`.

## Why a held-out set

The disjoint panel is a frozen instrument for the **store-read path**. This is a second, independent instrument for the compiler's **question-turn parse**: the training question forms authored for v26 must use templates and values **disjoint** from this set, so it measures **generalisation to unseen phrasings** rather than recall of the training forms.

100 rows, 10 relations, seed `20261004` — distinct from the panel builder's `20261003` and the panel run seed `9101`.

## Disjointness is mechanical, not eyeballed

`templates()` and `normalize_template()` (lowercased, whitespace collapsed, surrounding punctuation trimmed) are exposed so the v26 training build can **assert**

```
normalize_template(training) ∩ normalize_template(held-out) = ∅
```

Value disjointness is asserted in the tests against the frozen panel's expected answers.

## Tests — 4 pass

```
build_is_deterministic_and_well_formed ... ok
a1_answers_are_verbatim ... ok
values_are_disjoint_from_the_frozen_panel ... ok
template_normalization_is_stable ... ok
```

`cargo fmt --all --check` clean.

## Scope

One new file, `crates/uor-r4-training/src/bin/qform-set.rs`. The M-world v2 generator extension is **not** in this PR.

References #1552


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5982128051 — 2026-10-04T16:33:18Z — Casey-allard

Exact-head review at 92dff3da10fad163884f460570cd77c0ee69c3b8, checked locally from a git archive of this head.

**Checks**
- `cargo fmt --check`: passes.
- `cargo test --bin qform-set`: 3 passed, 1 ignored by default; 4/4 with `--include-ignored` and `QFORM_PANEL_EXPECTED` set.
- Rebuilds: `build version=1` gives 23429064…/82175ced… and `build version=2` gives b52f192e…/96c4c9e1…, both matching the frozen hashes.
- Full `verify` with panel=, panel_requests=, training_forms= and training_values=:
  - **v1 fails** on the panel-interrogative collision, as intended;
  - **v2 passes** every disjointness check and is byte-identical.
- `verify panel=/nonexistent.json` refuses with UNAVAILABLE.

**Two fixes before I queue it**
1. Compiler warnings: `use std::path::PathBuf` is unused (line 16) and `fn strings_of` is dead code (line 845). Remove both.
2. `verify` runs each disjointness pool only when its argument is given. A verify run without `panel_requests=` prints no panel-interrogative line, so a short command reads as clean. For every pool that was not given, print `NOT CHECKED: <pool> (no <arg>=)`, so a partial verify can never read as a full one.

The code is otherwise approved.

### comment 5982420875 — 2026-10-04T17:09:36Z — Casey-allard

Exact-head review at 7754b2753946631006f41155f943162376e9edf4: **approved**. Checked locally from a git archive of this head:
- `cargo fmt --check`: passes.
- `cargo test --bin qform-set`: 3 passed, 1 ignored. No unused-import or dead-code warnings remain.
- **v1 verify:** fails as required on the panel-interrogative collision.
- **v2 verify:** byte-identical, 0 assertion failures.
- **Bare verify:** prints NOT CHECKED for all four pools.

Queuing.


---
_Comment/review ids in this file: 5982128051, 5982420875_

