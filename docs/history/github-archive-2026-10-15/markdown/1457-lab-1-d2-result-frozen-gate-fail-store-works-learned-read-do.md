# #1457 — Lab 1: D2 result (frozen gate FAIL: store works, learned read does not generalise) + owner decisions + artifact contract

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-28T07:11:27Z
- **Updated:** 2026-10-09T13:08:58Z
- **Closed:** 2026-09-28T15:35:08Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1457
- **Merged:** 2026-09-28T15:35:08Z
- **Merge commit:** e1171feae5ef97edb9e7959d6c47cfbf9329db11
- **Base / Head:** main / UOR-Foundation:lab/claude/d2-aerm
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab:** Lab 1 (Claude).

**D2: architectural exact relational memory in the stack.** Pre-registered at `8b8f4ef5`; the fix at `fb57c4ee` voided attempt 1, and attempt 2 ran 3 seeds × 2 arms with every root sealed.

**Result: the frozen gate FAILS on the margin** ([record](docs/integration/d2-aerm-probe-result-2026-09-28.md), [evidence](docs/evidence/d2-aerm-probe-2026-09-28.json)).

| Seed | Memory, Updated | Control, Updated | Margin (≥ 0.30) | Text Δ (≤ 0.05) |
|---:|---:|---:|---:|---:|
| 1 | 1.000 | 0.854 | 0.146 ✗ | −0.027 ✓ |
| 2 | 1.000 | 0.878 | 0.122 ✗ | −0.039 ✓ |
| 3 | 1.000 | 0.898 | 0.102 ✗ | −0.038 ✓ |

- **In distribution, the memory arm is perfect on every class.** First, Absent, Previous and recency traps all score 1.000, and free-running answers are 32/32. The equal-parameter control answers mostly by latest mention: 0.51 on First, ≤ 0.07 on Absent, 0.25–0.42 on recency traps.
- **Held out, the memory arm collapses to 0.00–0.035.** Its learned **read trigger never fires on the unseen query phrasing**. With the gold register it would score 1.000, and every failure traces to the parser.
- **Reading:** the store and its read work; learned addressing from language does not generalise. The gate stands as frozen.

**Scope and limits:**
- a 1.4M probe with synthetic single-token slots;
- a teacher-forced value-slot gate;
- previous-distinct-value semantics only, with the store fresh per sequence;
- **no weights saved**, because the runner writes reports only.

**Also in this PR:**
- **Owner decisions of 15:27 UTC:** D6 outcomes are evidence, not a veto; the geometric read/address operator **G** is on the main path.
- **ROADMAP:**
  - §2a amended;
  - **§2b adds the artifact contract**, with unknowns marked;
  - **ruling 12 sets delivery discipline**: one target; experiment and product gates kept separate; engineering proceeds now; no fit without saved weights and a consumer; four-fact handoffs;
  - D1 recorded (keep the quaternion transport, +0.071 and +0.077 nats);
  - a log entry.
- **Code:**
  - `stack_aerm.rs` and `aerm-probe.rs`;
  - `logits_cross_entropy` in `geometric_stack.rs`;
  - 7 `stack_aerm` tests, including 30,000 episodes with no eviction and a placement-spread check.

**Checks:**
- fmt passes.
- 7 `stack_aerm` tests and 15 `geometric_stack` tests pass on merged `main`.
- The claim-wording gate passes.
- Relative links and table columns are checked.

References #973, #962, #820

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

