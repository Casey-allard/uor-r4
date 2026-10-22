# Policy precedence and store-miss vs log-sieve tests (antigravity-7)

Status: **extracted rules unit-tested under `#[cfg(test)]` on host** (`crates/uor-r4-training`).

> **Scope & compilation note:** Both `crates/uor-r4-training/src/relation_compiler.rs` and `crates/uor-r4-training/src/stack_grounded_session.rs` are compiled only under `#[cfg(test)]` in `crates/uor-r4-training/src/lib.rs`, so the non-test native qualification library graph is unchanged. Inside those modules, the historical `Softmax`, `SparseSoftmax`, `Trunk`, `SavedCompiler` (including `SavedCompiler::action` at `:2345`), and `GroundedSession` (including `GroundedSession::emitter_suffix` at `:834`) items remain gated under `#[cfg(any())]` and are neither built nor tested. The unit tests below pin the pure extracted helper rules `resolve_op_policy` and `resolve_recall_line` directly, not a running compiler or session.

## 1. `OpPolicy` precedence and derived-ID guard (`relation_compiler.rs`)

`resolve_op_policy` (extracted from the `#[cfg(any())]`-gated `SavedCompiler::action` in `crates/uor-r4-training/src/relation_compiler.rs`) combines a `table` action closure and an `op` action closure under four policies:

| Table action (`table_action`) | Op action (`op_action`) | `OpPolicy::Op` | `OpPolicy::Table` | `OpPolicy::TableStatements` | `OpPolicy::UnlessQuery` |
|---|---|---|---|---|---|
| `QueryCurrent` / `Query` | Statement (`Assert` / `Correct`) | Op's statement | Table's query | Table's query (`!statement(&table)`) | Table's query (`matches!(table, QueryCurrent \| Query)`) |
| Statement (`Assert` / `Correct`), closed ID (`< DERIVED_RELATION_ID_BASE`) | Another statement (`Assert` / `Correct`) | Op's statement | Table's statement | Op's statement | Op's statement |
| Statement (`Assert` / `Correct`), closed ID (`< DERIVED_RELATION_ID_BASE`) | Non-statement (`QueryCurrent` / `Unresolved`) | Op's action | Table's statement | Table's statement | Table's statement |
| Statement (`Assert` / `Correct`), derived ID (`>= DERIVED_RELATION_ID_BASE`) | Statement (`Assert` / `Correct`, closed ID) | Op's statement | Table's derived statement | Table's derived statement (`!table_derived`) | Table's derived statement (`!table_derived`) |
| `Unresolved` ("table says nothing") | Statement (`Assert` / `Correct`) | Op's statement | `Unresolved` | `Unresolved` (`!statement(&table)`) | Op's statement (table did not read a query) |

Note on where `TableStatements` and `UnlessQuery` differ:
- When the table reads a query (`QueryCurrent` or `Query`) and the op model emits a statement, **both** `TableStatements` and `UnlessQuery` keep the table's query (`TableStatements` because `!statement(&table)` is true; `UnlessQuery` because `matches!(table, QueryCurrent | Query)` is true).
- The two policies differ when the table reads `Unresolved` ("table says nothing") and the op model emits a statement: `TableStatements` keeps `Unresolved` (trusting the table's non-statement classification), whereas `UnlessQuery` takes the op model's statement.
- Under both `TableStatements` and `UnlessQuery`, a table statement whose relation ID is derived (`*relation >= DERIVED_RELATION_ID_BASE`) is protected by `!table_derived` so the op model (which emits closed labels only) cannot overwrite an open-relation derived store address with a closed label.

Pinned on the extracted helper by unit test `op_policy_precedences_and_derived_id_guard_resolve_disagreements` in `crates/uor-r4-training/src/relation_compiler.rs`.

## 2. Store-miss (`StoreRead::Absent`) vs `log_recall` sieve (`stack_grounded_session.rs`)

In `resolve_recall_line` (extracted from the `#[cfg(any())]`-gated `GroundedSession::emitter_suffix` in `crates/uor-r4-training/src/stack_grounded_session.rs`):
- `MemoryEffect::Read { read: StoreRead::Absent }` returns `(Some("Memory: none.".to_owned()), RecallDisposition::Absent)` directly and does not call `log_recall`.
- `MemoryEffect::Unresolved if read` consults `log_recall` and, when the sieve returns a non-sentinel value, emits `(Some(format!("Memory: {value}.")), RecallDisposition::LogValue)`.

This documents the current code's coupling descriptively: a turn mapped to a relation ID whose store slot is empty (`StoreRead::Absent`) emits `"Memory: none."` without consulting `log_recall`, whereas a turn left `MemoryEffect::Unresolved` by the compiler consults `log_recall`. Whether an absent store read should also consult `log_recall` or remain separate is an open mechanism question queued on the PM decision card in [`#2028`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6335479675), not settled by this helper extraction.

Pinned on the extracted helper by unit test `store_miss_returns_absent_without_calling_log_sieve_while_unresolved_calls_it` in `crates/uor-r4-training/src/stack_grounded_session.rs`.

## Notebook row (in-scope record for `m3-durable-memory`)

Because `docs/research/notebook/m3-durable-memory.md` is not listed in `m3-policy-precedence-and-sieve-tests.md`'s frontmatter `notebook:` / `scope:` (queried with the PM in [`#2028`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6335479675)), the notebook row is recorded here in the in-scope lab README:

| Date | Item | Change / Finding | Verification |
|---|---|---|---|
| 2026-10-22 | `m3-policy-precedence-and-sieve-tests` (`#2857`) | Extracted pure helpers `resolve_op_policy` (`relation_compiler.rs`) and `resolve_recall_line` (`stack_grounded_session.rs`) under `#[cfg(test)]` (with `SavedCompiler::action` and `GroundedSession::emitter_suffix` still `#[cfg(any())]`), documented the current `StoreRead::Absent` vs `MemoryEffect::Unresolved` log-sieve coupling descriptively, and pinned both extracted helpers with unit tests and 3 negative gates. | `cargo test -p uor-r4-training --lib` (37 passed: 23 in `relation_compiler::tests`, 1 in `stack_grounded_session::tests`); 3 deliberate breakages failed and recovered. |

## Resolution under the PM card (owner direction, 2026-10-22)

The owner directed the lab to proceed without waiting for the PM (DeepSeek lab chat, 2026-10-22, observed between 20:03:36Z and 20:06:30Z UTC: "please find a way to proceed without waiting on other agents if for some reason they are busy... maybe a subagent advesarial analysis or something?"). A fresh-context adversarial analysis was run on the three PM-card points (`#2028` 6335479675); its corrections are folded in below, taking the card's recommended **Option A on all three points**.

- **Module surface:** `crates/uor-r4-training/src/lib.rs` gates `relation_compiler` and `stack_grounded_session` under `#[cfg(test)]`, so the non-test native qualification graph is unchanged. **Caveat:** both delegating callers are themselves inside `#[cfg(any())]` impls (`relation_compiler.rs:1844`, `stack_grounded_session.rs:446`), so the helpers are compiled only under `cargo test`; the tests are **specification pins**, not live-path regression tests, and no non-test qualification path consumes these rules. Re-admitting the callers is rejected (it drags in the retired float graph and D23/D32 sites); caller-level protection is a separate item if the native session returns.
- **Store-miss / sieve coupling:** record both sides, settle neither. The code path is **deliberate**: `with_log_recall`'s doc says the sieve is for "turns that make no memory action" (`stack_grounded_session.rs:577-581`), and a resolved `Read { Absent }` is a memory action, so it prints `Memory: none.` without consulting the sieve, while `MemoryEffect::Unresolved` does consult it. The **outcome is a measured limitation**: #1738 is labeled a bug and measured `log_without_store 62 → 0` once questions began resolving to relation ids (archive copy, line 32), and `exact-memory.md` says address formation, not storage, caused 77.7% of held-out failures — so `Absent` often means a wrong address, not an absent fact. A fallback is an **open mechanism decision** that changes the trained recall line, collapses `Absent` into `LogValue`, and needs its own tests; this close-out changes no behaviour.
- **Notebook row:** recorded in this in-scope lab README as Option A allows. The real page row (`docs/research/notebook/m3-durable-memory.md`) plus the item `notebook:` field is a **PM/infra backfill**, not a routine m3 follow-up: the field can only be added from a PR whose ORDER item scope covers `docs/roadmap/items/*.md` (after #2997 merged, the mechanism is proven but the m3 item itself cannot bootstrap it), and the only such scopes overlap Kimi's open PR 8 (#2982). Also, once this item is `done` it can no longer be claimed normally, so the backfill must be picked up under the umbrella/PR-8 scope after that PR lands.

With those caveats, the item **done-when** is met: both `OpPolicy` precedences are pinned by unit tests (with the two mutation gates), and the store-miss/sieve behaviour is documented and tested (at the helper level). The flip to `done` is justified by that done-when plus the PM card's Option A, not by a literal reading of step 6's notebook-page line.
## Sources

- Item: [m3-policy-precedence-and-sieve-tests](../../roadmap/items/m3-policy-precedence-and-sieve-tests.md) (#2857).
- PM decision card: [`#2028`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6335479675).
- `crates/uor-r4-training/src/relation_compiler.rs` (`OpPolicy`, `DERIVED_RELATION_ID_BASE`, `resolve_op_policy`).
- `crates/uor-r4-training/src/stack_grounded_session.rs` (`MemoryEffect`, `RecallDisposition`, `resolve_recall_line`).
