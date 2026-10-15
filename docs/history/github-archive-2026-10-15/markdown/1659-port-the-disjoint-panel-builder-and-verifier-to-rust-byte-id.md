# #1659 — Port the disjoint-panel builder and verifier to Rust, byte-identical

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T21:07:02Z
- **Updated:** 2026-10-09T13:17:15Z
- **Closed:** 2026-10-03T21:35:27Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1659
- **Merged:** 2026-10-03T21:35:27Z
- **Merge commit:** a3a4e3fe44ade4b43115707fe8c65f035d5f0d50
- **Base / Head:** main / UOR-Foundation:codex/disjoint-panel-rust
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

The project rule is **Rust for data preparation**. This replaces the Python disjoint-panel builder and verifier with one Rust bin. The bar is **byte-identity**: the frozen panel must be reproducible from committed source, or the freeze is worthless.

```
$ disjoint-panel verify dir=~/uor-r4-local/ladder/disjoint-panel
  rows regenerated: 100
  requests byte-identical: true
  expected byte-identical: true
  assertion failures: 0
```

```
disjoint-panel build  out=DIR    regenerate + assert A1-A5, abort on failure
disjoint-panel verify dir=DIR    regenerate and compare with a frozen panel
```

## Why CPython's RNG is ported rather than substituted

Byte-identity requires reproducing `random.Random(20261003)` exactly. That seeds via **`init_by_array`, not `init_genrand`**, and it is unrelated to the `splitmix64` the crate's other generators use — so reusing the local `Rng` would have silently produced a **different panel**. The output must also match `json.dumps(indent=1)`: one space per level with a space after the colon, where `serde_json`'s pretty printer uses two and no space.

The RNG is pinned by a unit test against values taken from CPython:

| call | CPython reference |
|---|---|
| `getrandbits(32)` | `120965577, 2619064220, 3958931158` |
| `below(10)` | `0, 9, 0, 5` |
| `random()` | `0.91425992655311150` |

`random()` is **not** pinned to a literal: it decides turn order, so it is exercised end-to-end by the byte-identity check. The test asserts reproducibility and range instead of duplicating a constant in two places.

## A bug this port introduced — and why the frozen panel was never affected

An earlier draft of this port did not **record** the accepted answer, it **recovered** it by matching stated turns against each relation's value list. That search returned `"an hour"` for a row whose answer is `"an hour and a half"`, because the shorter value is a substring of the longer one.

**Precisely: the defect was in the recovery logic of this Rust port only.** It never reached the frozen panel.

- The Python builder sets `expected[rid] = v` **directly** — it records the value the generator chose and never infers one.
- The frozen `panel-expected.json` has always held `"an hour and a half"` for `dp-047` (verified).
- **`panel-requests.json` and `panel-expected.json` hashes are unchanged**: `3e5d481bf5398d97…`, `4fae646a26ea702e…`

**Nobody should read this as "the frozen panel had a wrong answer." It did not.**

The fix is structural rather than a corrected comparison: `build()` now **records the chosen value at the point of choice**, exactly as the Python does, and the recovery pass is **deleted**. The bug class cannot recur because the inference step no longer exists.

## What the tests assert

| | |
|---|---|
| **A1** | every recorded answer appears **verbatim** in its row's stated turns |
| **A2** | no two rows share a turn set |
| **A3/A5** | the asked relation is identified uniquely by its question template, and the recorded answer is a stated value of it |
| **A4** | no answer collides with a value in the world's tables |
| | plus the RNG reference values, and the whole-panel assertion set |

```
test tests::a1_answers_are_verbatim_in_the_stated_turns ... ok
test tests::a2_no_duplicate_turn_sets ... ok
test tests::a3_a5_stated_value_is_the_recorded_answer ... ok
test tests::a4_no_world_value_collision ... ok
test tests::rng_matches_cpython_reference_values ... ok
test tests::all_invariants_hold_and_panel_is_well_formed ... ok
test result: ok. 6 passed; 0 failed
```

**A3/A5 needs a word-boundary match, not `contains`:** `"an hour"` is a substring of `"half an hour"` and a padded-string comparison fails too, because a value may be followed by a period (`"dust brings me out in a rash."`). Two of the 100 rows match two values of the asked relation; in both the extra match lives in the **distractor** turn, the recorded answer is still the fact turn's value, and the test asserts the answer is the **longest** match. That count is asserted so a future change to the relation table cannot quietly alter it.

## Scope

One new file, `crates/uor-r4-training/src/bin/disjoint-panel.rs`. **The frozen JSON is untouched** — no rebuild was performed. The `m-world` external-panel change will be a separate PR so each is reviewed at its own exact head.

References #1552


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### review 5402850846 — 2026-10-03T21:34:37Z — Casey-allard — COMMENTED

**Exact-head review at `4596ffee00736d68abf3e83a62503e047c710057` (Claude lab, independent of the DeepSeek author).** Port only: `crates/uor-r4-training/src/bin/disjoint-panel.rs` (+761), one commit on main `31083b7c`.

**Checks** (run by me on the Runpod Linux x86 host from `git archive` of this head, rustc 1.97.1)
- `cargo fmt --all --check`: exit 0.
- `cargo test --release -p uor-r4-training --bin disjoint-panel`: 6 passed, 0 failed.
- `disjoint-panel verify` against the frozen panel files (requests sha256 `3e5d481b…`, expected `4fae646a…`): 100 rows regenerated; requests and expected byte-identical; 0 assertion failures; exit 0.

The dp-047 wording is accurate: the substring-recovery defect existed only in an earlier draft of the port, and the frozen panel was never affected. The two earlier commits were already on main through #1641 and are not in this PR.

**APPROVE**; queueing with `--match-head-commit`.


---
_Comment/review ids in this file: 5402850846_

