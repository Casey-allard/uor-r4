# #1711 — WIP: open relations wiring — blocked on global OnceLock (do not merge)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T02:45:42Z
- **Updated:** 2026-10-09T14:17:25Z
- **Closed:** 2026-10-05T13:18:12Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1711
- **Merged:** not merged
- **Merge commit:** abab81993e962464bd4e7304e3cf37c49bffef7d
- **Base / Head:** open-relations-extractor / UOR-Foundation:open-relations-wiring
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## WIP: open relations wiring — BLOCKED on global OnceLock (DO NOT MERGE)

Stacked on #1707 (the extractor). **Draft: this is blocked and must not merge.**

### What it does

Wires the deterministic relation identity from #1707 into the compiler and session, **scoped to a world with open relations (`world=v2r`)**, so `world=v2` is untouched by construction.

### Measured — **DEVELOPMENT result**

**137/200** on the relation-split set. **Labelled DEVELOPMENT, not held-out**: rows of that set were diagnosed to find the causes below, so it cannot be reported as held-out. The held-out measurement is item (ii) below.

```
panel memory:  137/200        (baseline 84, first attempt 36)
storage:       asked_fact_stored 200   never_stored 0   wrote_other_than_asked 0
paths:         store_read 137   log_without_store 0
misses:        recalled_wrong_value 0   stored_not_recalled 60   misrendered 3   errors 0
world=v2 gate: MQAR 22/23; open relation 11/12; closed relation 3/3; errors 0
```

**The residual 60 is exactly the ceiling's `identity_pairs_differ` set by row id** — empty symmetric difference, recomputed independently — and exactly the anaphoric `"What is it?"` rows. So the mechanism recovers precisely the recoverable set and fails on precisely the unrecoverable one, with no leakage either way.

### Three causes, in order of effect

**1. The `table_derived` precedence guard was in the WRONG match arm (36 → 69).** It sat in `OpPolicy::TableStatements` while sessions run `op_policy=unless_query`, so **it was unreached** — which is why an earlier attempt to fix this measured byte-identical and was wrongly withdrawn. Moved into `UnlessQuery`.

**2. The log sieve is consulted only for `MemoryEffect::Unresolved`** (`stack_grounded_session.rs`), so a resolved query that missed its slot returned `"Memory: none."` and never reached the log. A miss now falls back to the sieve under open relations. **Neutral alone (69 → 69)** — necessary but not sufficient.

**3. The span head marks a "value" inside a question** — `"What is mousbror's name?"` hands it the word `name` — so questions compiled as **statements** and **wrote instead of reading** (69 → 137).

### The narrower `?` rule, with its two fixed counter-cases

A bare `?` test was **too blunt**, measured:

```
before:  act=query_current  "Can you remember my vet is Ola?"   <- SHOULD STORE
         act=query_current  "Did I mention my bank is Monzo?"   <- SHOULD STORE
after:   act=assert span="Ola"    "Can you remember my vet is Ola?"   FIXED
         act=assert span="Monzo"  "Did I mention my bank is Monzo?"   FIXED
         act=query_current        "Tell me my vet's name."            held
         act=query_current        "Remind me what my landline is"     held
         act=query_current        "What is mousbror's name?"          held
```

**Rule:** a turn is a query iff a cue is present — terminal `?`, or a wh-word / `tell me` / `remind me` / `can you` / `did i` opening — **AND** the span head marks no value **outside the relation phrase**. A span whose words all occur in the relation phrase is the relation's own words, not a value held for it. **2 rows moved on the dev set, both correct; 0 moved the wrong way.**

### BLOCKER — why this must not merge

`open_relations()` is a **process-global `OnceLock`** set once by the CLI and never unset. Two consequences:

- **`cargo test` runs one process**: any test that enables it silently switches **every later `world=v2` test in that process** into the open branch — a flaky leak that could also hide a v2 regression;
- the compiler's behaviour depends on a flag **bound to neither the compiler artifact nor the session report**.

### Remaining work before this can merge

1. **Thread `with_open_relations(bool)`** through `SavedCompiler` and a session config field, record `"open_relations": true|false` in the session report, **delete the global**, and add a test that a v2 compiler and a v2r compiler **coexist in one process with different behaviour**.
2. **relation-split v2** — fresh seed, a **new** held-out English noun list authored before running anything (disjoint from `train-english` **and** the v1 held-out list), fresh nonsense draws; freeze it, then score **once**, reporting its own ceiling and /200.
3. **Score the frozen disjoint panel** (`3e5d481b…` / `4fae646a…`) **once** with open relations on, across **29M / 96M-B / 96M-C**. **That is the real test** — English relations and real phrasings.

### Also owed on this branch

Per review: **report per row which path answered** (store vs log), with an assertion that under open relations any row answered by the log **while a store record exists for that key** is flagged — so the fallback cannot mask a store defect unseen.

### Checks at this head

```
cargo fmt -p uor-r4-training -- --check    clean
cargo test --release -p uor-r4-training --lib
  (UOR_BUILD_SOURCE_COMMIT set)            651 passed, 0 failed, 6 ignored
```

Refs #1552. Stacked on #1707.


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 5987728235 — 2026-10-05T03:44:57Z — Casey-allard

## Blocker FIXED — `open_relations` is now an explicit setting

Pushed `a3338f8d`. The process-global `OnceLock` is **deleted**; the compiler and the session each take an explicit setting, and the report records it.

| | |
|---|---|
| `SavedCompiler::with_open_relations(bool)` + `open_relations()` | default **OFF**; the compiler reads its own field, never process state |
| session `with_open_relations(bool)` | so the write path and the read path agree on whether derived addresses are in play |
| session report | **`"open_relations": true\|false`** — an open-relations result is identifiable from its own artifact |
| CLI | derives it from the world variant: `matches!(world_of(args)?, World::V2r)`, set on both |

**The world's relation-POOL switch is kept, renamed** `enable_open_relation_pool()` / `open_relation_pool()` and documented as the **world's** switch, not the compiler's — the pool is part of the relation table the CLI builds once for the selected variant, which is a different concern from compiler behaviour.

### The coexistence test

`open_and_closed_relations_coexist_in_one_process`: a closed compiler and an open one, **in one process**, must differ on an unseen relation — the open one deriving the address, the closed one not — with the default asserted **OFF**. **While the global existed this test could not have passed**, which is what makes it the regression test rather than a decoration.

### Behaviour is unchanged by the threading

```
world=v2 gate   MQAR 22/23; open relation 11/12; closed relation 3/3; errors 0
relation-split  137/200; store_read 137; never_stored 0; recalled_wrong_value 0;
                stored_not_recalled 60; errors 0
report          open_relations=false (v2)   /   true (v2r)
tests           652 passed, 0 failed, 6 ignored;  fmt --check clean
```

### Blockers remaining

**Items (ii) and (iii) are untouched** — the held-out work:
- **relation-split v2**: fresh seed, a **new** held-out English noun list authored before any run (disjoint from `train-english` *and* the v1 list), fresh nonsense draws; freeze, score **once**, report its own ceiling and /200;
- **the frozen disjoint panel** (`3e5d481b…` / `4fae646a…`) scored **once** with open relations on, across **29M / 96M-B / 96M-C**. **That is the real test.**

**Also still owed** (from review): per-row reporting of **which path answered** (store vs log), with an assertion that under open relations any row answered by the log **while a store record exists for that key** is flagged — so the fallback cannot mask a store defect unseen.

Still a draft. **137/200 remains a DEVELOPMENT result.**


### comment 5988478546 — 2026-10-05T05:08:12Z — Casey-allard

## HELD-OUT RESULT: significant on the frozen real-English panel (+10, p=0.0129)

Item (iii) is done, and it is the first **held-out** measurement on this card.

**The frozen disjoint panel** (`3e5d481b…` / `4fae646a…`), scored **once** with open relations on, across all three models. Conditions **identical except `world`**:

```
29M     final-chat-29m-B-lr5e-4-sieve   world=v2    op_policy=unless_query  log_recall=sieve  seed=9101
        panel-open-chat-29m-B-lr5e-4    world=v2r   op_policy=unless_query  log_recall=sieve  seed=9101
        (same panel file both sides)
```

| model | baseline (closed) | open relations | delta | fixed | broken | **exact McNemar** |
|---|---:|---:|---:|---:|---:|---:|
| 29M | 23 | 29 | **+6** | 9 | 3 | p = 0.1460 *(ns)* |
| **96M-B** | 25 | **35** | **+10** | 12 | 2 | **p = 0.0129** |
| **96M-C** | 29 | **39** | **+10** | 12 | 2 | **p = 0.0129** |

**Both 96M models improve significantly: 12 rows fixed, 2 broken each.** The 29M model moves the same way, not significant at 100 rows.

### Why this is held-out, unlike the 137/200

**The panel was never diagnosed.** The three causes were found by diagnosing rows of the *relation-split* set. The panel is a separate, independently authored set with **real English relations and real phrasings**, frozen and scored before any of this work. **So this result can be reported as held-out; the 137/200 cannot and is still labelled DEVELOPMENT.**

### Why it works: consistency, not correctness

```
rows where the statement's and question's relation IDs AGREE : 15
of those, correct                                            : 15   (100%)
```

**Identical across all three models** — as it must be, since the addressing is deterministic and compiler-side. **The address needs to be CONSISTENT between the two turns, not semantically correct**: a consistent wrong key still retrieves its own value. The score is bounded by *agreement*, which is weaker than *correctness*.

### The binding constraint is now specific: 38 rows

```
distractor_only_written : 38    <- the FACT statement extracted nothing; only the distractor wrote
query_but_no_read       : 24
question_not_a_query    : 20
stored_and_read         : 12
```

**38 rows fail because the fact statement declines**, and the cause is this branch's own out-of-scope guard: a non-possessive determiner is only read on an interrogative, so `"dust brings me out in a rash."` and `"I grew up in Tulsa."` extract nothing. That guard exists to stop `"The weather is nice today."` being stored — **and it also stops real English statements.** The tension is measured, and the prize is 38 rows.

**That is the next decision, and it is not mine to take alone:** either accept the prose false-write risk (measured), or give the extractor per-relation frames — which is the closed label set in another form.

Still a draft. Remaining before merge: item (ii), the relation-split v2 held-out set, and the per-row store-vs-log path reporting.


### comment 5995261404 — 2026-10-05T13:18:11Z — Casey-allard

## Closing this as a retrieval claim — with the two pieces worth keeping

**Decision: do not merge the wiring.** The measurement that would justify it does not exist, and the measurements that do exist point the other way.

### Why

| evidence | result |
|---|---|
| `mqar-bench layout=fact`, 5 configs + a 2.4x capacity probe | `relevant-min` **0.000** every time |
| the same arm on `layout=synthetic` | **0.969** — so the read works, the transport does not |
| per-cell, all 24 × 2 splits | `first_piece` ~0.30, `full` ~0.03 — uniform across key length and gap |
| the Claude lab's audit of our fine-tune data | every run supplies the answer in a `Memory:` line |
| this PR's own held-out gain | **store +3 / log +7** — the log path supplies that line |

**This PR's headline was a supplied-answer measurement on the statement axis, and the statement axis is already solved by form breadth (37.8% → 94.6%, p=3.1e-11).** The wiring addresses a problem that was already fixed, using a mechanism the audit shows the network never learned.

**Keeping it open would leave a draft whose central claim is contradicted by three independent measurements.** That is worse than closing it.

### The two pieces that are defensible and should not be lost

1. **The schedule-precedence guard** (`UnlessQuery`): a derived-address statement must not be overridden by the op model. This was **unreached** in its first placement and its "no effect" measurement was meaningless until it moved — a genuine finding about how that policy dispatches, independent of open relations.
2. **The store-miss log fallback**: a `StoreRead::Absent` under open relations must fall back to the sieve, as `Unresolved` already does. It measured neutral alone but it is the reason `log_without_store` did not go to zero once questions began resolving.

Both are small, self-contained, and testable without any of the open-relation machinery. **I will file them as one issue rather than carry them inside a PR whose premise is gone.**

### What I am not claiming

That open relation identity is a dead end. **The deterministic reader is alive** — it is merged (#1707), and PR #1737 now takes it to **7/7 trained and 3/5 held out** on the corpus's question forms, matching the learned arm's 63.5% with no training. **What is dead is this wiring, which joined that reader to a training curriculum that supplies the answer.**

Refs #1552, #820. Superseded in direction by #1737.



---
_Comment/review ids in this file: 5987728235, 5988478546, 5995261404_

