# #1559 — Design memo: retrieval by an exact conversation log and a prime sieve (D18 outcome D)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T01:59:51Z
- **Updated:** 2026-10-09T13:08:44Z
- **Closed:** 2026-10-01T02:01:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1559
- **Merged:** 2026-10-01T02:01:06Z
- **Merge commit:** 6f21e584fa29dbf441f0d3d894a54531d7351e26
- **Base / Head:** main / UOR-Foundation:codex/log-sieve-memo
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Docs only: `docs/integration/log-sieve-retrieval-design-2026-10-01.md`, the design memo D18 outcome D requires. Nothing was run for it; every number is cited to its source.

**Design:**
- **Exact log.** Word atoms, not BPE tokens, which removes the copy-boundary problem. It is lossless under a declared cap, so a sliding window loses no facts.
- **Identity channel (exact).** Primes per word, gcd/merge admission, ranking separate from admission, latest version wins.
- **Relation channel (learned).** The language-to-atom compiler #958 never had. G v2 decoded held-out relations at 0.45–0.49, so this is the crux.
- **Emission.** The found value is inserted as one recall line before the reply, where the A1 pointer's measured recency copying becomes the correct behaviour.

**Experiments:**
- **E1:** an untrained `R-sieve` rule.
- **E2:** an oracle recall line on the existing A1 weights.
- E1 and E2 need no training, so they fit outcome D's rules. **E3** (the relation compiler) and **E4** (end to end) need the owner's approval.

**Checks:** `git diff --check`; `check_claim_wording.py` (passed); every cited file exists, and the two linked records resolve.

References #1552

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5923211121 — 2026-10-01T01:59:59Z — Casey-allard

**Claude lab, exact-head self-review at `63d05f1d`: APPROVE** (owner rule, #820). Docs only: one new Markdown file. Every measured number cites its source. I re-checked the A1 table and the R-recency row against the sealed reports, and the G v2, D2 and #958/#1487 figures against the cited records. Every 'design' or 'derived' statement is labelled as such. No experiment was run.

### comment 5923366666 — 2026-10-01T02:14:37Z — Casey-allard

Merged as 6f21e584fa29dbf441f0d3d894a54531d7351e26. Verified: `git merge-tree --write-tree f0502629 63d05f1d` equals the squash tree (5f936e64).


---
_Comment/review ids in this file: 5923211121, 5923366666_

