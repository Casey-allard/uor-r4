# #1738 — Two unreached scheduling paths: an unless_query precedence guard, and a store miss that skips the log sieve

- **Type:** Issue
- **State:** open
- **Author:** @Casey-allard
- **Created:** 2026-10-05T13:18:21Z
- **Updated:** 2026-10-09T14:43:19Z
- **Labels:** bug
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1738
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Two scheduling defects found while closing #1711, each independently testable

Both were found in the open-relations work but **neither depends on it**, and #1711 is closed so they would otherwise be lost. Each has a reproduction below.

### 1. `unless_query` dispatched the derived-address guard to an UNREACHED branch

`OpPolicy::UnlessQuery` is what sessions run, but the guard that stops the op model overriding a derived-address statement was first written into `OpPolicy::TableStatements`. **It was therefore never executed, and its "no effect" measurement was meaningless** — a null from unreached code is evidence of nothing.

Moving it into `UnlessQuery` flipped `"My mousbror is bokstaik."` from a closed id 3 to a derived id, and took that arm from 36/200 to 69/200.

**Why it matters beyond open relations:** the two arms of that policy differ in whether a **table statement can be overridden by the op model**, and nothing in the test suite asserts the precedence. `relation_compiler.rs` documents the policy as *"the op model's statement is used unless the table reads a query"* — and that is exactly what the `TableStatements` arm does **not** do. A test asserting the precedence per arm would have caught the misplacement.

**Suggested:** a focused test that both arms dispatch as documented, with a statement whose table and op actions disagree.

### 2. A store MISS does not fall back to the log sieve

`stack_grounded_session.rs` consults the sieve **only for `MemoryEffect::Unresolved`**. A query that resolves to a relation and then misses the store returns `StoreRead::Absent` and the reply is `"Memory: none."` — **the sieve is never consulted**, even though it can answer from the conversation.

Measured consequence: once questions began resolving to relation ids, `log_without_store` went **62 → 0** and nothing answered, taking the arm's score down with it.

**Why it matters beyond open relations:** any change that makes more questions resolve to a relation id will silently disable the sieve for them. **The sieve is a second memory path and it is currently coupled to the *failure* of the first.** Whether a miss *should* fall back is a design question, but that it currently cannot is not documented anywhere.

**Suggested:** decide and document the intended coupling, then test it. If a miss should fall back, the fix is small; if it should not, the `"Memory: none."` reply for a miss deserves a comment saying so, because the behaviour is currently indistinguishable from "the sieve is broken".

### How these were found

While wiring deterministic relation identity (merged as #1707). Both surfaced as measurements that "did nothing", and both turned out to be code that was never reached. **Neither is a blocker for anything; both are cheap and neither is currently tested.**

Refs #1552, #820.


## Comments (1)

### comment 6083193343 — 2026-10-09T14:43:19Z — Casey-allard

Kept open as a bug in the 9 October tracker rebuild (#2028); related milestone: #2031.


---
_Comment/review ids in this file: 6083193343_

