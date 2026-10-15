# #2150 — Softmax-free reads, arm group 1 result: REJECT rank (+0.040 BPB) and Hamming-rank (+0.259 BPB) (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T09:29:26Z
- **Updated:** 2026-10-10T09:41:43Z
- **Closed:** 2026-10-10T09:41:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2150
- **Merged:** 2026-10-10T09:41:15Z
- **Merge commit:** 9b423ad6213feb80e16212edeba0808d815e52ff
- **Base / Head:** main / UOR-Foundation:claude/softmax-free-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: softmax-free served read (M4 item 0) · count 3/3 · headline softmax at runtime yes → yes (served 0.886838 BPB unchanged)

**Result PR of the pre-registered arm group 1** ([pre-registration](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6093484433)), prepared by #2140 (trainer) and #2144 (D11 serving). Decision test: a KEEP would have made the softmax-free read the default served read. This REJECT keeps softmax and forces the pivot at 3/3.

## Result

Held-out `heldout.u16` e5f400b0, 512 windows × 384 = 196,608 targets, stream basis 2.837427 B/token. Served = D11 on the same windows. v4 = frozen-check memory /40.

| arm | float BPB (s1/s2) | served BPB (s1/s2) | v4 (s1/s2) | Δ float vs A | verdict |
|---|---|---|---|---:|---|
| A softmax | 0.877118 / 0.877277 | 0.886817 / 0.886702 | 6 / 3 | — | reference |
| B rank `flock:8:8` | 0.917017 / 0.917037 | 0.925884 / 0.925945 | 3 / 6 | **+0.0398** | **REJECT** (bar 0.01) |
| D hamming_rank | 1.162600 / 1.108824 | 1.173532 / 1.120014 | 0 / 0 | **+0.2585** | **REJECT** (BPB and v4) |

- **Served speed (D11, 8 threads, 32 windows, run one after another):** A 151.8, B 146.0, D 148.2 tok/s, equal within noise. A read touches at most 17 value rows per head against 192.5 on average, but the 19.93M dense weights read per token dominate at context 384.
- **Quantization cost** is unchanged by the read: +0.0097 (A) against +0.0089 (B) BPB.
- **Decision: REJECT B and D.** The served model keeps its softmax read and item 0 stays open. A pivot card follows on #2032: rank weights learned in training, served as a constant table.

## Artifacts and cost

- **Record:** `docs/labs/softmax-free-read-2026-10-10/README.md` § Result, plus a current-state entry with **Next**.
- **Results:** cloud-store `claude/softmax-free-20261010` (566,367,232 bytes, MD5 `d43d071d…`, verified by download), and the EU-RO-1 volume `uor-r4/claude/softmax-free-20261010`.
- **Compute:** about $10 across two pods, both deleted. Laptop CPU for the replies, grading, exports and D11 evaluation.

## Checklist
- [x] Numbers read from the sealed evaluation reports (`evaluation.json`, `report.json`, `grade-replies` reports) in the archive
- [x] Record section and current-state entry with **Next:**
- [x] STATUS/ROADMAP/#2028: headline unchanged; the #2028 M4 row gets the result line after merge
- [x] `check_claim_wording.py` passes
- [ ] After merge: verify on fresh origin/main, delete the branch and worktrees, hygiene, then the status card and pivot card on #2032

References #2032

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6096134153 — 2026-10-10T09:29:37Z — Casey-allard

Exact-head self-review at `540d780f0f67aa3caad7808b15afcd547f59caf9`: docs and record only, based on fresh origin/main 53c8804a. Every number was read from the sealed evaluation, report and grade files in the cloud-store archive, and the claim-wording gate passes. No code change, so no build or test applies.

### comment 6096213078 — 2026-10-10T09:40:14Z — Casey-allard

Exact-head self-review at `bce3117a22ff3ef6c1f60cfba49a75927cb076c6`: merged origin/main (Codex's #2030 entries). current-state.md keeps both top entries. Docs only, and the claim-wording gate passes.


---
_Comment/review ids in this file: 6096134153, 6096213078_

