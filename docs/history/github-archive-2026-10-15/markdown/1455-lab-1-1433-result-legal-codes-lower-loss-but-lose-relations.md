# #1455 — Lab 1: #1433 result — legal codes lower loss but lose relations (References #973, #962)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-28T06:41:27Z
- **Updated:** 2026-10-09T13:09:01Z
- **Closed:** 2026-09-28T06:42:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1455
- **Merged:** 2026-09-28T06:42:33Z
- **Merge commit:** 5ced49b05ca318a8daff67cb5d57d0db36cca707
- **Base / Head:** main / UOR-Foundation:lab/claude/t4-1433-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab:** Lab 1 (Claude). It executed Lab 4's (Codex's) frozen #1433 study unchanged, after the owner transferred it because Codex's allowance is exhausted.

**Result: the candidate FAILS its pre-registered retention rule** ([record](docs/integration/dialogue-code-choice-result-2026-09-28.md); [evidence](docs/evidence/dialogue-code-choice-result-2026-09-28.json)).

| | Continuous child | Nearest | Learned legal codes |
|---|---:|---:|---:|
| 161-response panel NLL (QQ) | — | 2.849148 | **2.796064** |
| Relation answers at the question turn | 9/10 | 4/10 | **2/10** |
| Greedy flips against the parent (3,914 saved positions) | — | 909 | **962** |

- **Lost relations.** Learned recovers neither Momo nor green when asked, and it loses Tokyo and pizza, which nearest kept.
- **Generic replies.** It answers 5 of 10 "What is my …?" questions with "Yes, I can help with that."
- **Gains.** It recovers one greeting and stops at EOS more often.
- **Scale of the change.** 6.4% of codes moved.
- **Disposition.** Nearest-hard stays the same-child baseline, and the learned artifact is preserved. As pre-registered, no dose, seed or decoder follow-up.

**Execution:**
- The fit resumed from 231 to 512 updates: 3,841 s, peak RSS 4.69 GB, `COMPLETE_FIXED_RECIPE`.
- The endpoints ran exactly as fixed in [#973](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5863711869) before the run, in 164 s, and every step exited 0.
- The same-binary nearest re-observation reproduces Codex's original exactly: 909/3/4 flips and identical NLLs.
- The binaries are Codex's frozen `b49a2810` builds, hash-checked before running.

**What changes:**
- Corpus cross-entropy pulled the artifact toward the corpus and away from its parent. D4 (Lab 3) is therefore unblocked with a **fidelity objective**, distillation from the continuous child, and a different weight representation (Hadamard + E8 or grouped codes). The retention check is the ten question-turn relations.
- ROADMAP T4 and §4.4, a dead-path row and the director log are updated.
- current-state.md gains a September 28 entry covering D0, the approved track, the audit and this result.

**Scope and checks:**
- This is open development evidence, not held-out qualification. Dense parameter access is unchanged.
- Documentation and evidence only.
- `python3 scripts/check_claim_wording.py` passes, the relative links resolve, and the table columns are consistent.

References #973, #962, #1433, #820

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

