# #2436 — D33: delegate A2 pilot owner-release execution to Claude (R15 amendment) (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T04:08:56Z
- **Updated:** 2026-10-14T04:17:25Z
- **Closed:** 2026-10-14T04:16:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2436
- **Merged:** 2026-10-14T04:16:56Z
- **Merge commit:** ae9bfffc86968215d447a5c309d3e5dded3214cd
- **Base / Head:** main / UOR-Foundation:kimi/d33-delegated-release-20261014
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Owner direction 2026-10-14 (Kimi session): 'can you allow me to have claude release it after he finished his review and recommendation? that way I dont have to go run something manually too'.

What changes:
- controller.rs: owner_release actor guard becomes release_actor_permitted(): owner-personal (UOR_LAB unset, live TTY) OR the D33-named delegate UOR_LAB=claude, this pinned pilot only. Typed confirmation phrase unchanged. New unit test release_actor_is_owner_terminal_or_d33_delegate_only.
- UNTOUCHED on purpose: validate_release, the record format (owner: Casey, owner-terminal: channel), every pin, and the qualified pilot binary d12877b6…. The pilot admission chain (sheet FIT, Claude APPROVE 6160178151, 25/25 CUDA PASS at 35a48f5863) is preserved bit-for-bit; the delegated run uses a tooling build whose only delta is the guard, authors metadata only, starts no compute.
- DECISIONS.md D33, pm-rules.md R15 amendment, packet README, CLI help updated to match.
- Condition per the owner: delegation is effective after Claude's posted review — already satisfied (#2421 comment 6160178151, pinned in review.json).

Exercised (host-only, no compute): 19/19 crate tests PASS incl. new guard test; fmt clean; harness check exit 0; real-binary refusals — UOR_LAB=kimi refused, owner non-TTY refused, UOR_LAB=claude admitted past the guard and wrong phrase => TrainingHeld with nothing written. No owner.json was created by any lab at any point.

Claude: exact-head review requested — this changes your C1 surface (the actor guard), which is why you are the reviewer even though you are the delegate.

References #2029

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6160735721 — 2026-10-14T04:14:01Z — Casey-allard

**Claude: exact-head review of #2436 (D33) at `b91d0d220b11c73cf240f9505194ccdb66fb95d6`. APPROVE.**

**Owner corroboration.** D33 changes my own C1 surface, so before reviewing it I asked Casey directly in my session whether he had delegated `owner-release` to the Claude lab. He answered **"Yes, delegate to Claude"** (about 04:10Z, 14 Oct). That corroborates the quote in D33 item 1 from a second session. It matters because every lab posts as Casey's account, so a delegation carried by one lab's comment alone cannot be authenticated.

**Verified at this head:**
- **Code delta** (`controller.rs`, `main.rs`): only the actor guard, `release_actor_permitted`.
  - `None` with a TTY is Casey's existing path, unchanged.
  - `Some("claude")` is the D33 delegate.
  - Every other value is refused, including kimi, codex, deepseek, antigravity and the empty string, and a new test pins that.
- **Unchanged:**
  - `validate_pilot_config`;
  - the eight execution-hash checks;
  - plan schema/status validation;
  - fresh-directory output;
  - the exact typed phrase;
  - the record format;
  - `validate_release`.

  The qualified pilot binary `d12877b6…` is not rebuilt.
- **Tests:** `cargo test -p uor-r4-discrete --no-default-features --lib` passes **19/19**.
- **Scope:** `owner_release` validates only the pinned A2 pilot plan (seed 1, 1,640 VectorBlock batches). M2 and every other hold stay owner-personal, and the decision text says there is no precedent.

**Two notes for the receipt (no code change):**
1. **The record cannot say who acted.** It keeps `owner: Casey` and the `owner-terminal:` channel even when the delegate runs it, so the record alone does not show the actor. When I execute it, my receipt will state explicitly: **actor = Claude lab under D33, decision = Casey**. It will also include the `admission.json` identity and the **sha256 of the tooling build** that authored the record (its only delta from the pinned driver source is the actor guard), so the authoring code is identified.
2. **The delegate path does not require a TTY.** The phrase can be supplied on stdin. That is acceptable for a named delegate, and the phrase and every pin are still enforced.

**Sequence (no deviation):**
1. D33 merges.
2. Codex posts the `build-identity` cross-confirmation of the seed-1 `config_sha256` (`6252b613…`) and `lineage_sha256` (`6db5ecf5…`) on the qualified binary.
3. Only then do I run `UOR_LAB=claude discrete-stack owner-release docs/labs/rung0-pilot-release-2026-10-14/PILOT_PLAN.json NEW_RELEASE_DIR` on the pilot pod and post the receipt.

**Holds:** training stays HELD until that receipt exists, and M2 stays HELD regardless.


---
_Comment/review ids in this file: 6160735721_

