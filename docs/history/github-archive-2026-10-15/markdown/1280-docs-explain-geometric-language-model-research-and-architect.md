# #1280 — docs: explain geometric language model research and architecture

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-14T20:06:59Z
- **Updated:** 2026-10-09T14:17:02Z
- **Closed:** 2026-09-14T20:08:17Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1280
- **Merged:** 2026-09-14T20:08:17Z
- **Merge commit:** a5655e935c3c3b82d2b9c0bb7986c04eb2cb8d1d
- **Base / Head:** main / UOR-Foundation:docs/geometric-language-model-research-map-2026-09-14
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Rewrite the root README as a coherent research and architecture entry point, following the owner's request to explain both the active implementation and the mechanisms available for reuse.

- Put the objective first: a learned geometric alternative to transformer-based LLMs, targeting frontier-level capability on local commercial hardware.
- Preserve the strict inference contract: no transformer backbone, mathematical matrix products, disguised lookup/add contractions or serving responses supplied by another model. Offline Rust training may use matrix multiplication.
- Separate the target end-to-end architecture, retained native artifact, active experimental language core, historical engines and research candidates.
- Explain prime/ordered-n-let identities, fixed zeta phases, signed R4/S3/H4 state, exact Z[phi], paired icosian/E8 construction, Hopf/fiber information, Hamming comparison and directed relative transport.
- Map active source modules, reusable infrastructure, imported work, research archives, evidence, interfaces and reproducibility prerequisites.
- Preserve pre-alpha status, the September 8 qualification withdrawal, exact September 14 artifact/result boundaries and negative findings. The canonical plan and current-state file remain the execution authorities.

## Scope

README.md only. No runtime, training, artifacts, schemas, resource ledgers or historical experiment records are changed. No model is promoted and no new capability result is claimed.

Source review is anchored at edc59b5fd95f459d3c4a528dad910f8391c08e12. References #973, #964 and #820; this documentation change does not close their broader acceptance.

## Validation

- Ran the repository's unchanged `scripts/check_claim_wording.py` against the edited README in an isolated README-only staging directory: PASS. The checker was verified against upstream Git blob `f98c53a7dad8bfb4bc24cd279022281713ae587c`. This is not a whole-repository wording check.
- Parsed the Markdown and checked its internal navigation: 18 headings, nine navigation targets, seven tables and four fenced blocks; PASS.
- Checked whitespace, placeholders and the required architecture/status boundaries: PASS.
- Reviewed 54 unique repository link targets against retrieved source, the repository tree and existing source maps. This is not a live HTTP check of every target.
- Committed README Git blob `9e52ad5ab6bf61fb65da0bea6b36b57011f4223d` matches the validated text; content SHA-256 `f6bf6ecf388b8ecf50fa0ccf8abcc82067776acf8d3ad6e6ab37f92a952f3806`.
- Rust build/tests, training, model evaluations and benchmark replays: NOT_RUN, documentation-only change.

Shell GitHub access failed DNS resolution, so repository reads and the branch/file mutation used the connected GitHub tools. No original local checkout or retained research artifacts were touched. Protected PR/queue acknowledgements are not described as tests.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

