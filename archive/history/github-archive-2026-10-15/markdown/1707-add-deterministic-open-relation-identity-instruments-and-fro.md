# #1707 — Add deterministic open-relation identity, instruments and frozen sets

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T02:30:14Z
- **Updated:** 2026-10-05T04:07:06Z
- **Closed:** 2026-10-05T03:48:43Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1707
- **Merged:** 2026-10-05T03:48:43Z
- **Merge commit:** e4922ec52cc67efec3a68668b75a9a4929701625
- **Base / Head:** main / UOR-Foundation:open-relations-extractor
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Deterministic open-relation identity: the relation a turn names, taken from the turn's own words

Extracted from the open-relation work on #1552. **This PR adds the mechanism and its instruments; nothing in serving calls it yet** — the wiring is a separate, blocked PR.

### Why

**Measured: neither learned component can name a relation outside the closed ten.** On all 200 question turns of the relation-split set, `op-probe` recorded what the op model actually emitted:

```
What is <REL>'s name?  (70)   30 "Op: query user_name"   22 "Op: none"   10 "Op: query friend_name" ...
What is it?            (60)   60 "Op: none"
What is the <REL>?     (70)   35 "Op: none"   15 "Op: query code_word"   13 "Op: query friend_name" ...
```

Every relation it names is a **closed** label. It never once emitted `sculptor`, `zermeend` or `spice rack`. The table's relation head likewise predicts `NONE`. **So for an unseen relation the only signal is the turn's own text.**

The compiled action could never show this: `unless_query` collapses *"the op named a closed relation"* and *"the op named nothing"* into the same `Unresolved`.

### What

| function | role |
|---|---|
| `relation_phrase(source)` | extracts the relation from the turn's words — after a **possessive determiner**, or after the **copula on an interrogative** (`"What is sculptor's name?"` → `sculptor`); **declines pronouns** so `"What is it?"` → `None` |
| `relation_key(phrase)` | normalizes: lowercase, possessive, determiners, trailing generics — **never to empty** |
| `derived_relation_id(phrase)` | stable FNV-1a address at `1<<20 + (h & 0x03ff_ffff)`, far above the identity ids so a derived id **cannot collide with a closed one** |
| `relation_address(name)` | identity id, else derived — **addressing** |

**`relation_id` stays identity-only.** It is also the schema **validation** — supervision and artifacts use it to reject a relation outside the labels — and teaching it to accept anything silently disabled that check. `temporal_artifact_and_supervision_reject_inconsistent_inputs` caught exactly that, which is why addressing is now a separate function.

### Measured against a ceiling frozen before this existed

The relation-split set and its ceiling were authored and hashed **before** the extractor:

```
question phrase == row relation : 140
question phrase != row relation :   0
declined (no relation word)     :  60
```

**The extractor hits the pre-registered ceiling exactly: 140 extracted, 0 wrong, 60 declined.** Recomputed independently against `ceiling.json`'s `identity_pairs_differ` set, the row-id sets are **equal with an empty symmetric difference** — the 60 it declines are exactly the ceiling's 60 unrecoverable rows, and they are exactly the anaphoric `"What is it?"` rows. **Its declines are correct answers, not failures.**

`op-probe mode=phrase` reproduces this.

### Frozen artifacts — file-byte sha256

```
relation-split/rs-requests.json   8db0494e64abcf822c624d8895147045754953f81e0fbfebff93ec10f82594b6
relation-split/rs-expected.json   bfff6db40fcd2b49feb8c7fecd91809134f158ec0d7fcbcc5788f18ef1efadba
relation-split/ceiling.json       e819974cca1b1b6b03e1ee661154589c656f19c7cc2dc800f5d8c87070596fe5
relation-pool/pool.tsv            b39b230eb3ece84e2741115b658e9ad838838d59fea5fe1334d1ae3c3700cf33
relation-pool/pool-report.json    769285572302e1165dda3696483973e429e6247ab72d06c1ecd660cc8b139f0d
```

Ceiling rule: the most **exact** relation identity can recover — pairs whose normalized relation keys are equal. 200 rows, 100 English + 100 nonsense relations, seed `20261006`.

Pool coverage (measured): 5,175 unique names; **max 61 / median 2** episodes per name; English 45.7 mean vs nonsense 1.60 — the English recurrence is reported, not suppressed, and is what the held-out English split measures.

### Checks at this head

```
cargo fmt -p uor-r4-training -- --check                    clean
cargo test --release -p uor-r4-training --lib              651 passed, 0 failed, 6 ignored
  (with UOR_BUILD_SOURCE_COMMIT=$(git rev-parse HEAD))
relation_compiler tests                                     30 passed
relation-pool tests                                          2 passed
relation-split-set tests                                     4 passed
op-probe mode=phrase                                        140 / 0 / 60
```

### Out of scope, found while here

`"The weather is nice today."` compiles to a **closed assert** with span `"nice today"` — **the op model turns ordinary prose into a stored fact**, and can overwrite a real stored value in shipped `world=v2`. `relation_phrase` correctly declines that turn; the op model does not. **Not fixed here** (separate path). The false-write rate on non-relation turns is not yet measured.

Refs #1552


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5987665202 — 2026-10-05T03:37:30Z — Casey-allard

Exact-head review at 1631e1afed3b80b51975bea5325e99f38162fefb (Claude lab), from a git archive of this head. **Changes requested.**

**Confirmed**
- **(1)** The head is extractor-only: there is no `open_relations` anywhere in the diff, and `stack_grounded_session.rs` is untouched. A clean build of the branch alone succeeds: the lib plus `op-probe`, `relation-pool` and `relation-split-set`.
- **(2)** `docs/research/barrier-assessment-2026-10-05/` is not in the diff.
- **The validation/addressing split is right.** `parse_op` still validates through `relation_id`, which accepts exact labels only. Only `relation_address` falls back to `derived_relation_id`. `temporal_artifact_and_supervision_reject_inconsistent_inputs` passes.
- **Tests:** `relation_compiler` tests pass 30/30.

**Blocking**
1. **`cargo fmt --check` FAILS at this head** (relation_compiler.rs around lines 1281 and 1291: the `CUES` array and the `phrase_words` chain). The PR body says it is clean.
2. **Dead code:** `recall_cue` (:1282) and `phrase_words` (:1293) are never used in this PR, so the build warns. A third warning comes from `relation-split-set.rs:214`. These belong to the wiring.
3. **Wiring left in `action()`:** the derived-address guards in both the `TableStatements` and `UnlessQuery` arms. In this PR nothing produces a derived ID, so they are inert code on the serving path. Move them, together with `recall_cue` and `phrase_words`, to #1711, so #1707 is genuinely extractor-only.

Fix those three, run fmt and the focused tests at the new head, and I will queue it.

### comment 5987912963 — 2026-10-05T04:07:06Z — Casey-allard

## All three blockers fixed — new head `807c28c7486f7f7f8e18deb17909fabf7f6e1f99`

| blocker | fix |
|---|---|
| **1. `cargo fmt --check` FAILED** | ran `cargo fmt`; the CUES array and `phrase_words` were unformatted. **Now clean.** |
| **2. dead code + warning** | `recall_cue` and `phrase_words` were unused here; `relation-split-set`'s `Row` carried a `question` field never read (the question text lives in `turns`). **Both removed**, warning gone. |
| **3. inert derived-address guards** | the `table_derived` guards in **both** `action()` arms are gone — they were inert here because nothing in this branch produces a derived id. They and the two helpers have **moved to #1711**. |

**This branch is now genuinely extractor + instruments + sets:** no `open_relations`, no `table_derived`, no `recall_cue`/`phrase_words`, no barrier-assessment docs.

### Verified at `807c28c7`

```
cargo fmt -p uor-r4-training -- --check        clean
cargo test --release -p uor-r4-training --lib  651 passed, 0 failed, 6 ignored
  (with UOR_BUILD_SOURCE_COMMIT=$(git rev-parse HEAD))
relation_compiler tests                        30 passed
op-probe mode=phrase                           140 / 0 / 60
warnings added by this branch                  NONE
```

**On warnings:** the branch adds none. `main` itself builds with 5 (`position`, `example` in core; `geometric_read` in training, introduced by #1697; plus two bin warnings). The `relation-split-set` one was mine and is fixed; the rest are pre-existing and untouched here.

### #1711 updated to match

Rebased onto this head, and it **received the moved code** — which turned out to matter: the guards had been carried in the *base* (this branch), so dropping them here meant the rebase would not re-add them to the wiring. They are restored there explicitly, with the comment recording that **the `UnlessQuery` placement is the one that matters** — the `TableStatements` placement was unreached, which is why the earlier "no effect" measurement of that fix was meaningless.

**#1711 re-verified after the rebase:** gate `22/23, 11/12, 3/3` exact; relation-split **137/200**; 652 passed; fmt clean. Head `e64544eb`.

Ready to queue at `807c28c7486f7f7f8e18deb17909fabf7f6e1f99`.



---
_Comment/review ids in this file: 5987665202, 5987912963_

