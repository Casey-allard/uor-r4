# #2057 — README v2: goal, progress, mechanisms with figures, training record, HF models and data, credits; CONTRIBUTING and CITATION

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T16:50:56Z
- **Updated:** 2026-10-09T16:53:13Z
- **Closed:** 2026-10-09T16:52:47Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2057
- **Merged:** 2026-10-09T16:52:47Z
- **Merge commit:** 6e857870ec14c4298b035baf3c44d88e04c7f99a
- **Base / Head:** main / UOR-Foundation:claude/readme-v2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Rebuilds the public front door as an open research project (owner request, 9 October 2026).

- **README.md:**
  - the goal and how close we are (0 of 8 milestones done, 4 in progress), and what works and what doesn't;
  - the architecture, how it works, and training (model table with verified numbers, data sources with licenses checked against their HF cards, compute);
  - serving under D11, results (positives and negatives), the roadmap, quick start, and the public Hugging Face models and data;
  - how the project is run, statistics, the repository map, acknowledgements and citation.
- **docs/figures/*.svg:** five figures (architecture, training pipeline, serving pipeline, geometry stack, milestone progress) with one colour-blind-safe status legend, self-contained, with title and description.
- **CONTRIBUTING.md:** the current owner-approved workflow for labs, agents and humans, and how to propose a change to it.
- **CITATION.cff** for the paper.
- **STATUS.md:** `uor-chat --stack` now serves the stack (#2050).

Acknowledgements: Alex Flom, Ari, Maura, N3mesis, Matthew (SpiralCore), DarkUnicorn (GoldSnnail and goldworm-coder), each with only what the evidence and the owner confirmed.

Checks: claim wording passes; every workspace member is named; all links and images resolve; CITATION.cff parses; the SVGs parse as XML with no external references and were rendered and inspected.

References #2028

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6085325882 — 2026-10-09T16:51:03Z — Casey-allard

Exact-head self-review at `89bf9a1b3af08a3fcc75d0c3d938585fd7eaf823`: read README sections, corrected training facts and license table against the HF cards, rendered and inspected the architecture figure; claim wording passes; figures parse with no external refs; CITATION.cff parses.


---
_Comment/review ids in this file: 6085325882_

