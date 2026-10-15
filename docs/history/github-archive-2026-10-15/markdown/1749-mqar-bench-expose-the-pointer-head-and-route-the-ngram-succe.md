# #1749 — mqar-bench: expose the pointer head and route (the ngram successor route was unreachable)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T18:49:50Z
- **Updated:** 2026-10-09T14:17:23Z
- **Closed:** 2026-10-05T18:52:27Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1749
- **Merged:** 2026-10-05T18:52:27Z
- **Merge commit:** a27c1b82d28afd05c44be135e5aaa32c14e69688
- **Base / Head:** main / UOR-Foundation:mqar-pointer-route
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Expose the pointer head and route on `mqar-bench` — the ngram successor route was unreachable

**`mqar-bench` hardcoded `pointer: None` at both construction sites** (`:583`, `:1725`), so the geometric `ngram` pointer route — `RouteAdmission::Ngram`, *"admitting by the longest ordered n-let match"*, **fully implemented in `geometric_stack.rs`** — had **never been exercised on this instrument.**

**This is the third instance tonight of the pattern [D20](docs/integration/DECISIONS.md) §2 condition 3 now requires be checked: code that is implemented, reachable in principle, and never reached in a real run.** (The others: a precedence guard written into an unreached policy arm, and a fallback that was reachable, gated, gate-exact and fired **0 times in 60 conversations**.)

### What this adds

A `pointer=<spec>` argument, wired to **both** halves the head needs — the first attempt failed with `"the model has no pointer head to route"` because `StackConfig.pointer` is a `PointerConfig` and the route alone is not enough:

```rust
model.add_pointer(PointerConfig::new(config.width), common.seed)?;   // the head
model.set_pointer_route(Some(route))?;                              // the route
```

Unset — the default, and what every prior run used — leaves both off, so **existing arms are unchanged**. Accepted: `none`, `prime:<w>`, `prime-ranked:<w>`, `ngram:<w>`, `ngram-ranked:<w>`, via the existing `parse_pointer_route`.

### First result, pre-registered before the run

1,200 steps, `final_sequences=256`, seed 7, `layout=fact`, **1,176 cells per arm**. `full|first` = `P(continuation | first piece correct, rest teacher-forced)`:

| arm | first_piece | full | **full \| first** | rule |
|---|---|---|---|---|
| `pointer=none` | 0.2941 | 0.0331 | **0.1125** | 0.623 |
| `pointer=ngram:2` | 0.1484 | 0.0160 | **0.1082** | 0.623 |
| `pointer=ngram:4` | 0.1471 | 0.0131 | **0.0890** | 0.623 |

**1. The ngram route does not fix transport** — `full|first` is flat to worse.
**2. It damages addressing** — `first_piece` halves.
**3. The baseline reproduces the sealed measurement** — `0.1125` here against `0.117` in sealed `p0-fact-wide`, so the null is interpretable rather than noise.

### The result is the LOCALISATION, per D20 §4

`full|first` sits at **0.089–0.113** across arms that differ **only in how the key is formed**, while the non-learned rule holds **0.623** throughout.

> **Changing the key does not move continuation. The missing component is not on the key side — it is downstream, in the READ-OUT.**

**This rules out the family the project's own design suggested.** A successor path built as a *key transformation* (n-gram keys, lineage keys) is addressing the wrong side of the read. The next candidate is a **read-out** mechanism: a persistent offset that advances during decode, where the emitted token steers the next read.

### This does NOT retire `ngram`

Per [D20](docs/integration/DECISIONS.md) §4 — merged today — a comparison gap is a **localiser, not a verdict**, and *"retirement is not an available outcome."* `ngram` admission stays a live candidate for **addressing** work (where it already measured 0.27/0.44/0.33 against a 0.81/0.84/0.68 ceiling on one draw). What this run shows is that it is **not the transport fix**.

### Checks

```
cargo fmt -p uor-r4-training -- --check    clean (0 diffs)
git diff --name-only origin/main..HEAD     crates/uor-r4-training/examples/mqar-bench.rs   (1 file)
```

**Instrument only.** No model, no serving path, no existing arm behaviour changed. The `cargo fmt` reformatting of two pre-existing dead helpers in `relation_compiler.rs` (a local rustfmt version difference, unrelated to this work) was **deliberately excluded** to keep the diff to one file.

Refs #820, #1552, #1716.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6000970567 — 2026-10-05T18:51:32Z — Casey-allard

## Exact-head self-review — `7f9b25aa64c81f1641c51c7037ff9ba6b36e2068`

`local == remote` verified. Self-review permitted under the owner's current exact-head rule.

### Checks

```
1. git diff --name-only origin/main..HEAD     crates/uor-r4-training/examples/mqar-bench.rs   (1 file)
2. cargo fmt -p uor-r4-training -- --check     clean, 0 diffs
3. cargo build --release --example mqar-bench  OK
4. required acknowledgements / delivery evidence   PASS      mergeStateStatus CLEAN
5. local head == remote head                   7f9b25aa…   OK
```

### Verified by reading the diff, not the description

- **The default is genuinely inert.** `common.pointer` is `None` unless `pointer=` is passed, and the new block is `if let Some(route) = …`, so **every existing arm takes exactly the old path**. No arm behaviour changes.
- **Both halves are wired.** The first attempt failed at runtime with `"the model has no pointer head to route"` — that error is quoted in the commit message because it is the reason `add_pointer` is there alongside `set_pointer_route`. A route without a head is not a working configuration, and the code now supplies both.
- **Parsing reuses the existing route parser**, so `none`/`prime:`/`prime-ranked:`/`ngram:`/`ngram-ranked:` all behave consistently with every other binary; no new grammar.
- **Instrument only.** No model code, no serving path, no existing arm touched.

### Scope defect found and fixed during review

**The first commit for this work swept in an unrelated `cargo fmt` reformatting of two pre-existing dead helpers in `relation_compiler.rs`** — a local rustfmt version difference, nothing to do with the bench. That is the second time on this workstream that `git add -A` has widened a diff beyond its stated purpose. **Corrected: this PR is one file.** (`cargo fmt --check` passes here either way, so CI is unaffected; the unrelated hunks simply do not belong in this PR.)

### On the result being a null

**The PR's value is the instrument, not the number.** The ngram route did not fix transport — `full|first` went 0.113 → 0.108 → 0.089 while the non-learned rule held 0.623 — but the *usable* output is the localisation: **the arms differ only in key formation and continuation does not move, so the missing component is downstream of the key.** That is what tells the next attempt where to look, and it is exactly what D20 §4 asks a comparison to produce.

**The baseline reproducing the sealed figure (`0.1125` vs `0.117`) is what makes the null interpretable rather than noise**, and it is why the sealed measurement was cited in the PR rather than a fresh claim.

**Reviewing at `7f9b25aa`. Approving and merging.**



---
_Comment/review ids in this file: 6000970567_

