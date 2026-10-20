# #1554 — Add the count-only §8 panel length check for the owner (D18 §9)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-30T21:30:10Z
- **Updated:** 2026-10-09T13:08:48Z
- **Closed:** 2026-09-30T21:57:17Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1554
- **Merged:** 2026-09-30T21:57:17Z
- **Merge commit:** 8c1778ca4dbffe773ebb8d33919f7b652f9b3685
- **Base / Head:** main / UOR-Foundation:codex/s8-panel-count
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this adds

`s8-panel-count` is the count-only length check that D18 section 9 asks for. The owner runs it on the sealed §8 panel; no lab runs it there.

It reads `panel.json` and the #1017 tokenizer and prints one JSON object:
- `max_conversation_tokens`: the longest conversation in protocol tokens, with every scored reply empty;
- `max_with_reply_budget`: the most positions any scored reply can reach when each scored reply runs to 32 ids;
- `fits_context`: whether that second number is at most 256;
- the panel's SHA-256 (to compare with `SEAL.txt`), the tokenizer CID and the dialogue-protocol identity.

It prints no panel text:
- Errors name a category, conversation index and turn number only.
- A JSON error reports only its line and column, because serde's messages can quote the offending value.
- The tool refuses any tokenizer other than #1017's (`blake3:3f42bcfc…`).

## Accounting

The counts use the same accounting as `check_panel` and `reply_panel`:
- BOS;
- each user prefix;
- each reply;
- the EOS that closes a generated reply before a later turn.

Unscored turns carry `reference_reply`. Each one counts as the protocol encodes assistant content, plus its EOS.

## Checks run (local, fixtures only)

- `cargo fmt --check -p uor-r4-training`: passed.
- `cargo test -p uor-r4-training --lib s8_panel_count`: 5 passed. The tests check that:
  - the counts match `check_panel`'s admission boundary exactly (admits at N, refuses at N−1);
  - reference-reply histories equal the canonical `encode_assistant_prefix` / `encode_open_history` serialization;
  - a planted string never appears in any of 11 malformed-panel errors;
  - the report holds a fixed key set.
- A release binary run on a non-panel fixture with the real #1017 tokenizer:
  - the CID matched;
  - the output contained counts and hashes only;
  - a bad argument and invalid JSON both exited 1 with a message that quotes no text.
- The exact-head GitHub run is on `codex/ci/v2/s8-count`.

This is a measurement tool. It is not a language result.

References #1511, #820

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5920419003 — 2026-09-30T21:56:24Z — Casey-allard

**Claude lab, exact-head self-review at `b24edf46`: APPROVE** (owner rule, #820). Read the whole diff:
- `s8_panel_count.rs` parses the panel as a `Value`, and every error names only category, index and turn. A JSON error gives only line and column, so no serde message can quote the panel.
- The accounting is check_panel's, and reference replies are counted through the canonical encoder.
- The binary refuses any tokenizer other than #1017's and prints only counts, hashes and identities.

Exact-head run [36779789620](https://github.com/UOR-Foundation/uor-r4/actions/runs/36779789620):
- fmt: success.
- all-targets check: success.
- `uor-r4-training` lib: **400 passed, 3 failed, 4 ignored**. The 3 failures are main's known `joint_campaign` tests ("checkpoint/evaluator binding differs"), identical on main. There are no new failures, and the 5 new tests pass.

### comment 5920451276 — 2026-09-30T21:58:56Z — Casey-allard

Merged as 8c1778ca4dbffe773ebb8d33919f7b652f9b3685. Verified: `git merge-tree --write-tree dd457291 b24edf46` equals the squash tree (31806fa5). The owner's command is unchanged: the pinned binary `~/.local/share/uor-r4/bin/s8-panel-count-b24edf46` was built from this exact head.


---
_Comment/review ids in this file: 5920419003, 5920451276_

