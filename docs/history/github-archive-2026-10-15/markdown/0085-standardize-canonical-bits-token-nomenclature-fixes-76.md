# #85 — Standardize canonical bits/token nomenclature (Fixes #76)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-23T03:22:58Z
- **Updated:** 2026-07-23T03:46:03Z
- **Closed:** 2026-07-23T03:34:18Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/85
- **Merged:** not merged
- **Base / Head:** main / UOR-Foundation:issue-76-canonical-bits-per-token
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

This PR resolves #76 by standardizing the bits/token metrics in Gate C evaluation and certification output.

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5053947670 — 2026-07-23T03:23:03Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### comment 5054009092 — 2026-07-23T03:34:18Z — auser

Thanks for picking this up — but the canonical definition has already landed and this PR conflicts with it on every file. Current main: bits_per_token is the single canonical field (f64) across the Gate C harness and report schema, with the definition in docs/transformerless/GLOSSARY.md and the historical families annotated in BASELINE.md (issue #76, commit 1615127). The two-field approach here (wb_bits_per_token + an Optional bits_per_token with a fallback) reintroduces exactly the ambiguity #76 resolved by naming the scorer+distribution alongside values instead of splitting the metric. Also note the baseline_kappa.json change — the pin moves only by explicit maintainer re-pin (AGENTS.md documents the procedure). If you'd like to take one of the open chain issues (#69, #71–#75, #77–#80 under tracker #65), several are ready and unassigned; happy to point you at a good first one (#77 CI trend tracking is small and self-contained).


---
_Comment/review ids in this file: 5053947670, 5054009092_

