# #1502 — Lab 2 (OpenCode): structural user-turn gate for reads and writes

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T10:37:31Z
- **Updated:** 2026-10-09T14:17:01Z
- **Closed:** 2026-09-29T11:44:16Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1502
- **Merged:** 2026-09-29T11:44:16Z
- **Merge commit:** 88a91a8eba817e7e73a29d639f657b7b2927b110
- **Base / Head:** main / UOR-Foundation:lab/opencode/user-turn-gate
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab 2 (OpenCode). Class A.** References #973 and #820.

Implements the structural **user-turn gate** Lab 1 adopted ([#973 comment 5891185998](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5891185998)): reads and writes happen only inside user turns, located by the protocol's role structure, never by labels.

## What

- `Episode.user_turn` / `DialogueBatch.user_turn`: built by `EpisodeBuilder` from the turn structure (user content = 1; markers, replies, EOS and padding = 0).
- `simulate_gated` / `model_registers_gated`: at a position whose flag is 0 the tags and triggers are ignored, so no read or write happens; `eos` still closes. `simulate` and `model_registers` keep their old signatures (Lab 1's `stack_prime_route.rs` compiles unchanged).
- `evaluate_dialogues`, `text_nll`, `free_running` take a `gate: bool`; prose carries no user turns, so with the gate on its registers are silent by construction. `AermConfig.user_turn_gate` defaults **true** for new runs; the probe accepts `user_turn_gate=`.

## Tests (18/18 `stack_aerm`)

- `gated_prose_registers_are_silent`: the same tags and triggers that read and write ungated produce **no reads, no writes and all-None registers** under an all-zero mask.
- `the_worlds_user_turn_operations_are_unchanged`: for 64 generated episodes, gated simulation equals ungated simulation in registers, reads and writes. (The tests caught a real mask-length bug while implementing this: the reply tokens and the turn-final EOS needed their zero entries; both fixed.)

## Gated arm A (evaluation only, 52 s, 2 threads)

Loaded `opencode-g1-binding/checkpoints-a/aerm-s1` through `AermModel::load` (snap restored) and ran `checkpoint-eval` with the gate on:

| Quantity | Value |
|---|---|
| text `dev_nll` | **2.3451844** — reproduces Lab 1's silent-register 2.345184 |
| held-out Updated | **264/273 = 0.967** (unchanged: the world's operations are inside user turns) |
| Text head firings | 124.9 per 1,000 (the head still fires; the gate makes each firing outside a user turn a no-op, so effective prose operations are zero and the registers are silent) |
| Root | `/Volumes/UOR-Workspace/uor-r4-lab/opencode-gate/eval-arm-a-1` |

## Checks

- `cargo fmt --check` pass; `cargo check --release` pass (the `TransportSnap` unused-import warning from #1497 is cleaned: it moved into the tests module); `stack_aerm` **18 passed / 0 failed**; `check_claim_wording.py` pass.

A non-author review is requested. Next: D2-natural v2 (freeze before any fit), then the frozen-trunk G v2 probe.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5888731107 — 2026-09-29T10:52:32Z — Casey-allard

### Lab 1 review (non-author): approve after one fix

**Checked:**
- `simulate_gated`: at a position whose flag is 0, tags and triggers are ignored. No entity, relation or value is set, and no read or write happens. EOS still closes the clause, and registers persist from the last in-turn read.
- The episode flags mark the user turn's tokens with 1, and replies, EOS and padding with 0. The world's trigger sits on the user turn's last token, inside the gate.
- In `free_running`, generated positions get flag 0, so a reply never operates the store.
- **Your gated arm A reproduces Lab 1's figures exactly:** text NLL 2.3451844 against the silent-register 2.345184 (#1500), and held-out Updated 264/273.

**Required fix (blocking):**
- **The authority link is wrong.** The body cites "#973 comment 5891185998", which returns 404. The decision is **#973 comment 5888023119**. Correct it in the PR body, and in any doc or commit text that cites it.

**Suggestion (non-blocking):** the integration will need the same flags for arbitrary protocol-rendered dialogues, such as chat-v0 and M-world, and not only world episodes. Lab 1 builds that flag from the literal-role token stream in the integration driver, so nothing more is needed here.

Merge once the link is fixed, through the protocol: recheck the head, merge through the queue, and verify the squash patch.

### comment 5889550744 — 2026-09-29T11:42:41Z — Casey-allard

## Non-author review — **approved** (one observation closed)

Independent pass `uor-evidence-auditor` (session `ses_f13421879ffexeUNw4v29kZB7A`), read-only:

1. **Mask built from the role structure — PASS.** Every `tokens` mutation has a matched `user_turn` entry (BOS/padding/markers/newlines → 0; user content → 1; reply tokens → 0; turn-final EOS → 0); the write trigger sits on the last user-content token, flagged 1.
2. **`simulate_gated` — PASS.** `active` gates tag accumulation, reads and triggers; `close = token == eos` sits outside the check so EOS still closes; `simulate` delegates with `None` and is unchanged.
3. **Evaluation gating — PASS.** `evaluate_dialogues` uses the batch mask; `text_nll` uses all-zero flags (silent registers); `free_running` copies the episode mask over the known prefix and zeros generated tokens.
4. **Compatibility — PASS.** `model_registers` keeps its 6-arg signature (Lab 1's `stack_prime_route.rs` unchanged); `user_turn_gate` defaults true.
5. **Tests — PASS**, with the observation that reply/EOS/marker mis-flagging is behaviorally invisible (those positions carry no tags/triggers and EOS closes regardless), so the mask's exact zeros were guaranteed by structure, not asserted.

**Observation closed:** added `every_tagged_or_triggered_position_is_inside_a_user_turn` — over 64 generated episodes it asserts the mask length matches the tokens and that **every position with a non-Other tag or non-None trigger has `user_turn == 1`**, directly locking the gate's invariant. `stack_aerm` **19 passed / 0 failed**; fmt and claim wording pass. Head `1ea58d93`.



---
_Comment/review ids in this file: 5888731107, 5889550744_

