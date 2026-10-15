# #2110 — reply_exact: discrimination becomes expressible on a single-turn row, and it changes no verdict (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T23:32:48Z
- **Updated:** 2026-10-09T23:49:22Z
- **Closed:** 2026-10-09T23:34:41Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2110
- **Merged:** 2026-10-09T23:34:41Z
- **Merge commit:** a2c9acbbf19f1cd810b22fae8d23910534b4da7a
- **Base / Head:** main / UOR-Foundation:reply-exact-grammar
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## The gap, closed

`forbid` and `keys` parsed only for `exact` (which `validate_checks` requires to be a **multi-turn recall row**) and for `abstain_exact`. 224 of the panel's 232 rows are single-turn, so the three-component shape — **required content, forbidden distractors, distractor keys** — could not be written for them. With anchors alone a reply passes by containing the right content word; nothing checked whether it **also** contains the wrong one.

**`CheckKind::ReplyExact`** is now `exact`'s matching rule **without the recall precondition**, permitted with `history=none`. Eight sites in `crates/uor-r4-training/src/bin/chat-grade.rs`: the enum arm, `name()`, `passes()` (sharing the exact arm), the parser (same `terms != "-"` guard), `forbid` and `keys` acceptance, the expected/forbidden/key overlap refusals, and a `validate_checks` arm requiring the terms absent from the last user turn and **the distractors and keys absent from every user turn** — a distractor the user wrote is not a distractor. **No existing kind's behaviour changed.**

### Focused tests on the changed path — 17 passed, 0 failed

`cargo test -p uor-r4-training --bin chat-grade`, new test `reply_exact_is_exact_without_the_recall_requirement`:

| case | result |
|---|---|
| accepts a reply containing the required content | pass |
| rejects a reply missing the required content | pass |
| rejects a reply containing a forbidden distractor (*"They make honey, not vinegar"*) | pass |
| rejects a reply naming the distractor's key (*"The wasp makes honey"*) | pass |
| **validates on a single-turn row, where the same row as `exact` is refused** | pass |
| refuses a term / a distractor / a key present in the request | pass |
| refuses a missing distractor, a term also forbidden, a term also a key | pass |

`chat-grade.rs` is `cargo fmt --check` clean. The four files `cargo fmt` also wanted to reformat are **other labs' pre-existing unformatted sources** — I reverted them rather than sweeping them into this change.

## Distractors: five rows, not eighty-eight

Authored only where a wrong-but-plausible alternative genuinely exists — a value a confused or hedging reply would actually contain, that is not the required content, is not in the request, and is not itself a correct answer.

| row | required content | forbidden distractor | why plausible |
|---|---|---|---|
| `ask-06` | a farm animal | `lion\|tiger\|elephant\|penguin\|shark\|dolphin` | a wild animal named as a farm animal |
| `heldout-022` | `heapq\|heapify\|heappush\|heappop\|push` | `max heap\|maxheap` | request asks for a **Min** heap |
| `heldout-043` | `96` | `48` | the half-count error |
| `heldout-103` | `28` | `17\|55` | 17 forgets 11; 55 sums 1..10 |
| `heldout-126` | `printf\|stdio\|include\|main(` | `cout\|System.out\|console.log\|print` (keys `java\|javascript\|python\|cpp`) | a C request answered in C++/Java/JS |

**Two candidate rows were rejected, and the rejection is the interesting part.** `ask-07` (*"Why is the sun hot?"*, misconception *"a ball of fire"*) — because **negating** the misconception is normal correct phrasing (*"it is not a ball of fire, it is nuclear fusion"*), so the check would fail correct replies. `ask-02` (*"What do bees make?"*) — because bees really do make wax and royal jelly, so the near alternatives are correct answers. The other 14 content rows have no specific wrong word: their wrong answers are **missing content**, which the anchored `any` check already catches.

## Re-sealed and re-validated

| | before | after |
|---|---|---|
| checks sha256 | `ad4e9a14…` | **`d24ae404be11d25b…`** |
| kinds | `any` 21, `abstain_exact` 67 | **`any` 16, `reply_exact` 5, `abstain_exact` 67** |
| checked rows | 88 (37.9%) | **88 (37.9%), unchanged** |
| `check_panel` | pass | **pass** |
| worst-case context position | 168 / 384 | **168 / 384** |
| `checks_without_loaded_request` | `[]` | **`[]`** |
| manifest from `data/panels/` | 29 OK | **29 files, 29 OK** |
| v4 + v5 novelty | pass, pass | **pass, pass** |
| **canned-reply control (13 strings)** | **0 passes** | **0 passes** |

**The canned control still reports zero**, run by the grader's own `RowCheck::passes` through `chat-grade check constants=`. The new component did not become a path by which a canned reply passes, and the 67 ill-posed rows keep `abstain_exact`, so the safety property measured in the first piece is untouched.

## The honest result: **no verdict changed**

First reading with discrimination: **2 of 88**, exactly as before — 1 of 16 `any` rows, 1 of 5 `reply_exact` rows, 0 of 67 ill-posed. `ask-06` passes (names a horse, no wild animal); `heldout-043` still never states 96, `heldout-103` still answers *"1, 2, 3, 4, 5, 6, 7, 8, 9, 10."* for the primes, `heldout-126` still does not write the program.

**So on this artifact the model's wrong answers are wrong by *omission*, not by naming a competing value** — Result 6's "short wrong answers" confirmed from a new direction. The component is worth having (a reply saying *"honey, not vinegar"* now fails a check that passed it), but **it explains none of the failures and must not be reported as if it did**.

## Delivery

Record section in [`docs/labs/reply-panel-deterministic-checks-2026-10-09/README.md`](docs/labs/reply-panel-deterministic-checks-2026-10-09/README.md) with `Next:` rewritten to the **clarify** kind; newest-first [`current-state.md`](docs/integration/current-state.md) entry ending in **Next:**; stale docs fixed (`data/panels/README.md` inventory, ladder runbook §6). `check_claim_wording.py` passes; `scripts/__pycache__/` not committed. **STATUS/ROADMAP/#2028 unchanged — checked, not assumed. Criterion 1 remains NOT MET and 43/232 is unchanged:** this makes the instrument express discrimination rather than only presence, and it does not move the criterion.

**References #2029.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

