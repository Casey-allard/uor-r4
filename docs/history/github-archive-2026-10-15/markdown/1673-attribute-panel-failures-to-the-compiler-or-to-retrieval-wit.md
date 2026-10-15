# #1673 — Attribute panel failures to the compiler or to retrieval, with a per-turn mechanism trace

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-04T05:30:07Z
- **Updated:** 2026-10-09T13:17:46Z
- **Closed:** 2026-10-04T06:15:17Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1673
- **Merged:** 2026-10-04T06:15:17Z
- **Merge commit:** c87817ee000e4786cf356a6af843e17d2e453b6f
- **Base / Head:** main / UOR-Foundation:codex/panel-diagnosis
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Why

The panel told us *that* rows fail (asked fact stored in 50/100, 7–8 read back), not **where**. This records a per-turn mechanism trace and a per-row diagnosis so a failure can be attributed to the **compiler** or to **retrieval**.

**`stack_grounded_session.rs` untouched.**

## Mechanism (corrected)

The truncations are **not** a span-arithmetic defect. In `crates/uor-r4-training/src/relation_compiler.rs` (~1190–1208) the **op-model generates the value text**, and `source.find(value)` then locates it. When generation stops early, `find` matches the **shorter prefix**, and the span is `start..start + value.len()` over whatever was generated — so `"penic"` is stored for `penicillin`, `"seven zer"` for `seven zero six`.

## Field sources

| trace field | source |
|---|---|
| `action`, `relation`, `span_text` | `outcome.action` (`CompiledAction`) + `SourceSpan`, value cut from the user text |
| stored text | `outcome.memory` → `MemoryEffect::Write { value_tokens }`, decoded |
| `read_found` / read value | `outcome.memory` → `MemoryEffect::Read { read: StoreRead::Found(v) }`, decoded |
| `log_value` | `outcome.recall` → `RecallDisposition::LogValue` (this is the **only** use of `RecallDisposition`) |

## Diagnosis labels

| label | meaning |
|---|---|
| `fact_unresolved` | no pre-question turn produced a write at all |
| `distractor_only_written` | a write landed, but never for an accepted answer |
| `question_not_a_query` | the final turn's action is not `Query`/`QueryCurrent` |
| `query_but_no_read` | the question *was* a query, but no store read returned |
| `read_wrong_value` | a store read returned, but **not** an accepted answer |
| `stored_and_read` | the asked fact was stored **and** the read returned an accepted answer |

`stored_and_read` requires the read to be **verified** (`read_verified`), not merely to have occurred; a read that returned something else is `read_wrong_value`.

## Result — disjoint panel, 100 rows, seed 9101

Compiler-driven categories are **fixed by the shared compiler**, so they cannot vary with emitter scale:

| model | `lr` | fact_unresolved | distractor_only | question_not_a_query | query_but_no_read | read_wrong_value | stored_and_read |
|---|---|---:|---:|---:|---:|---:|---:|
| 8M | off | 8 | 42 | 32 | 9 | 1 | 8 |
| 8M | sieve | 8 | 42 | 32 | 9 | 1 | 8 |
| 20M | off | 8 | 42 | 32 | 9 | — | — |
| 20M | sieve | 8 | 42 | 32 | 9 | — | — |
| 29M | off | 8 | 42 | 32 | 9 | — | — |
| 29M | sieve | 8 | 42 | 32 | 9 | — | — |

The 20M/29M rows were measured **before** `read_verified` existed, so their last two columns are left blank rather than backfilled; their compiler-side counts stand because those are compiler-fixed.

Adding `read_verified` **changed a number**: `stored_and_read` was 9 and is now **8**, with **1** row reclassified as `read_wrong_value`. The earlier 9 counted a read that returned a different value as a success.

## What the trace shows

**1. The compiler's relation inventory is ELEVEN labels** — `user_name`, `pet_name`, `friend_name`, `hometown`, `lucky_number`, `code_word`, `job`, `home`, `favorite_food`, `favorite_color`, and the sentinel `none` (from `compiler.json`, the `compiler-save-op-v25-rawtable` identity).

**Correction:** an earlier revision of this PR said "five-relation inventory". That was wrong — I had read the **world's** relation table in `milestone_world.rs` (5 relations) instead of the **compiler's** trained identity (11). The panel's `hometown` **is** a compiler relation; most panel relations simply have no equivalent in the compiler's set (`allergy`, `degree`, `flatmate`, `commute`, `holiday`, `bank`, `shoe_size`, `vet`, `landline`).

**2. Assignment is by surface pattern, not identity**, which is why rel ids vary per row:

| panel relation | assigned compiler relation (`id`) |
|---|---|
| `hometown` | `pet_name`(2) ×6, `home`(8) ×4 |
| `bank` | `job`(7) ×4, `favorite_color`(10) ×3, `favorite_food`(9) ×2 |

So `rel7` = `job` and `rel8` = `home`.

**3. Values are sometimes taken from the wrong noun phrase.** `"I studied history at university."` → span `"university"`; `"shellfish brings me out in a rash."` → span `"rash"`.

**4. Seven writes are truncated prefixes** of the answer (`"penic"`, `"late"`, `"HSB"`, `"Ross"`, `"six three zer"`, `"seven zer"`, `"three eight"`), by the generation-stop mechanism above. The character drop is 1 in four cases and 5 in three, which is consistent with generation stopping, not with a token boundary.

**5. The 32 `question_not_a_query` rows** split `unresolved` 17, `assert` 9, `correct` 6 — questions are frequently parsed as **new statements**, so no read is attempted.

**6. The read path is the smallest loss**: 8 rows stored and read correctly; 9 failed a query that was actually attempted.

## Scope

`crates/uor-r4-training/examples/m-world.rs` (+205/−9). `cargo fmt --all --check` clean; 20 example tests pass.

References #1552


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### review 5404591953 — 2026-10-04T06:14:26Z — Casey-allard — COMMENTED

**Exact-head review at `18c30ed634bebeae7f451e9948d582b9663d70bd` (Claude lab, independent of the DeepSeek author).**

**Verified**
- Char-safe span slicing.
- Real-function tests for all labels, including the (2,3) multibyte split asserting empty text.
- `read_verified` and `read_wrong_value`; the doc comments are in place.
- The PR text now describes truncation as early stopping in op-model value generation, gives the compiler's 11-label inventory, and states that compiler-side counts are identical across arms by construction.
- `stack_grounded_session.rs` is untouched.

**Tests at this head** (pod, Linux x86, `git archive`): fmt 0; `cargo test --release -p uor-r4-training --example m-world` 20 passed.

**Note:** the three `joint_campaign` lib-test failures mentioned in the PR are environmental. They need `UOR_BUILD_SOURCE_COMMIT` set at build time, or `option_env!` falls back to UNBOUND and the binding check refuses by design. They are not caused by this PR.

**APPROVE**; queueing with `--match-head-commit`.


---
_Comment/review ids in this file: 5404591953_

