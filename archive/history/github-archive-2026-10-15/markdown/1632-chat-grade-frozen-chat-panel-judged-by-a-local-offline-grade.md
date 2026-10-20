# #1632 — chat-grade: frozen chat panel judged by a local offline grader for the scale ladder

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T17:51:18Z
- **Updated:** 2026-10-09T13:08:42Z
- **Closed:** 2026-10-02T21:00:49Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1632
- **Merged:** 2026-10-02T21:00:49Z
- **Merge commit:** 03b080d249fc8c651c89300f3cb9b295cb8f299e
- **Base / Head:** main / UOR-Foundation:codex/chat-grade
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820

**What.** The scale ladder's chat-quality instrument, a new bin `chat-grade` with two modes:
- **`extract`** picks held-out chat-v0 first user turns by a seeded shuffle.
- **`grade`** answers a frozen request panel greedily with any saved stack model (`stack_dialogue::reply_panel`). It then asks a local Ollama judge, an offline grader that never serves, two yes/no questions per reply: is it clear English, and does it respond sensibly to the user's last message.
  - It also grades a derangement control, pairing each reply with the next request, so that relevance must fall if the judge is measuring relevance.
  - The report root is claimed, sealed and verified. The report records the judge's digest and raw answers.

**Calibration.** emit-6r (2.1M) on the frozen panel: 32 everyday requests plus 32 stretch requests from held-out chat-v0. The panel is at `~/uor-r4-local/ladder/panel`, with sha256 values in `SHA256SUMS.txt`.

| Judge | Fluent | Relevant | Acceptable | Control relevant | Unparsed answers |
|---|---|---|---|---|---|
| qwen2.5:1.5b, 1–5 scale (first draft) | 3.0 mean | 3.44 mean | — | 2.94 mean | 28/64 |
| qwen2.5:1.5b, yes/no | 26/64 | 41/64 | 26/64 | 30/64 | 0 |
| **qwen2.5:7b, yes/no** | 11/64 | 3/64 | **3/64** | 2/64 | 0 |

- The 1.5B judge is too lenient on relevance: it passed "Hey! Nice to hear from you." as an answer to "What do bees make?".
- With the owner's approval, the 7B judge was pulled for the ladder; it is strict.
- emit-6r's floor of 3/64 is the ladder's baseline.

**Checks.**
- A unit test covers yes/no parsing and first-turn extraction.
- Both modes were run with the release binary as above.
- GitHub evidence: `codex/ci/v3/chat-grade` (fmt, check of all targets, lib).

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### review 5396796270 — 2026-10-02T20:59:58Z — Casey-allard — COMMENTED

Exact-head review at 741b71e48552b411029c34a19ba1981466a68a45 (Claude lab; self-review per owner rule on #820). New bin chat-grade only. The unit test covers yes/no parsing and first-turn extraction. Both modes were run with the release binary: the extract panel and emit-6r grading with qwen2.5:1.5b and 7b. GitHub evidence https://github.com/UOR-Foundation/uor-r4/actions/runs/37043529761 (parent = this head): fmt, check of all targets, lib 576 passed / 0 failed.

### comment 5961411274 — 2026-10-02T21:01:18Z — Casey-allard

Merged as 03b080d2; chat-grade.rs on main equals the reviewed head 741b71e4 (blob comparison).


---
_Comment/review ids in this file: 5961411274, 5396796270_

