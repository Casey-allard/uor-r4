# ORDER: m3-policy-precedence-and-sieve-tests, deepseek-31 (owner-directed close-out)

Lab: deepseek. Item: [m3-policy-precedence-and-sieve-tests](../../../../roadmap/items/m3-policy-precedence-and-sieve-tests.md) (#2857). Base: `8d488457e` (merged forward; the close-out started from `b00d9edad`). Item PR (docs-only close-out).

## Owner direction

Casey, in the DeepSeek lab chat on 2026-10-22, observed between 20:03:36Z and 20:06:30Z UTC, verbatim:

> "please find a way to proceed without waiting on other agents if for some reason they are busy... maybe a subagent advesarial analysis or something?"

That is the owner instructing this lab to proceed on the open PM decision without waiting for Kimi (who is out of tokens). The owner is the last fallback for item and scope decisions, so this ORDER records his direction rather than editing an owner path.

## Cancel probe (run first)

Class **instrument** (docs-only close-out). At `b00d9edad`: the item's code was already merged by antigravity (#2947, `fb9e49e92`); `lib.rs` already gates the two modules under `#[cfg(test)]` (PM card point 1, Option A applied); the store-miss/sieve rule and its test are in place and documented descriptively; the only open points are the PM card's coupling wording and the notebook row. No code change is needed; the item is `ready` only on bookkeeping. The task stands.

## Files in scope

`docs/roadmap/items/m3-policy-precedence-and-sieve-tests.md` (`status:` line), `docs/roadmap/README.md` (rendered), `docs/labs/m3-policy-precedence-and-sieve-tests/README.md`, and this task folder. **No receipts file:** this is a doc-only PR and the D46 doc-only exception lets it carry a same-lab fresh-context record instead of another vendor's.

## Resolution of the PM card (`#2028` 6335479675), recommended Option A on all three points

1. **Module surface:** verified already gated — `pub mod relation_compiler;` and `pub mod stack_grounded_session;` carry `#[cfg(test)]` with the exclusion comment in `crates/uor-r4-training/src/lib.rs`; the non-test graph is unchanged.
2. **Store-miss / log-sieve coupling:** keep the current rule documented descriptively (Absent says `Memory: none.` and does not consult `log_recall`; Unresolved does), not declaring the mechanism settled. No behaviour change.
3. **Notebook row:** record it in the in-scope lab README as Option A allows (already present). The real-page row and the item `notebook:` field are a **PM/infra backfill**: the field needs an ORDER whose item scope covers `docs/roadmap/items/*.md` (only the umbrella/PR-8 scopes do, and #2982 holds them today), and a `done` item cannot be claimed normally.

A fresh-context adversarial subagent attacked all three points; its corrections (helper-level honesty, intended-code-with-measured-limitation coupling framing, backfill naming) are applied in the REPORT and are part of this close-out.

## Done when

- The item flips to `done`, the resolution is recorded in the lab README with the owner direction quoted, and the roadmap is rendered.
- A fresh-context same-lab review record at the head (doc-only, D46 exception) supports it; `report --check`, `pr-check --local`, `loop gates --full` and `--break all` pass.

## Stop and ask if

- The owner withdraws the direction or Kimi records a different reading; then this close-out is reopened as a repair.