# #650 — chore(research): consolidate AI-Research + Prime-Analysis into research/ (single source of truth)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-12T22:49:22Z
- **Updated:** 2026-10-09T14:16:58Z
- **Closed:** 2026-08-12T23:20:53Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/650
- **Merged:** 2026-08-12T23:20:53Z
- **Merge commit:** a119ff313460ffbf0f794e07440ed3e5ca37dd2d
- **Base / Head:** main / UOR-Foundation:consolidate-research
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Consolidates the two research repositories into `uor-r4` as a single source of truth, under `research/`:

- `research/ai-research/` — snapshot of `Sky-Metrix/AI-Research` (geometric-routing program `ai-router/`, MUDBench, ramsey, preprints, experiment outputs).
- `research/prime-analysis/` — snapshot of `Casey-allard/Prime-Analysis` (spectral/prime research, `theoretical-mathematics/` incl. `formal-math-lean/` and the vendored `UOR-Framework/lean4`, routing-infrastructure prototypes, manuscripts).
- `research/prime-analysis/photos/` — supporting figure/slide images (Git LFS).

One snapshot commit; **11,053 files**, **1,231 stored via Git LFS** (~640 MB media; images, PDFs, `.npz`/`.npy`, office docs). Text, code, CSV, and Lean sources are stored normally.

## Decisions reflected

- **Public repo, sensitive gated.** The unpublished MSA papers, personal/legal PDFs, and packaged artifacts are excluded — and were untracked in the source repos, so they could not enter via the merge. A backstop personal-data scan was run over the snapshot (27 pattern hits, all verified false positives: public LM benchmark corpora — WikiText-2/PTB — containing ordinary words like "affidavit"/"social security", binary byte-matches, and vendored library files; **no personal/legal documents are present**).
- **History preserved on archive refs.** To keep this repo's squash-clean mainline intact, full commit history of each source repo is on `archive/ai-research-history` and `archive/prime-analysis-history` (media blobs >1 MB stripped from those refs to stay lean; full-fidelity media remains in the originals). This PR itself is one clean snapshot commit.
- **Media via LFS**; standalone Photos included, Videos excluded.
- Committed virtualenvs, `__pycache__`, `node_modules`, and OS junk (91 MB) were dropped — not research content.

## CI safety

`research/` is added to the root `Cargo.toml` workspace `exclude`, so it is outside the crate build/lint/test. The claim-wording gate scans only `docs/**`, `crates/*/README.md`, and root `README.md`, so `research/` is out of its scope. No repo-wide file scanners exist in the workflows.

## Review guidance

This is a structural addition — review the folder layout, `research/README.md` provenance, the exclusions, and the LFS/`.gitattributes` setup rather than line-by-line. Squash-merge (the queue does this by default) keeps mainline to one commit. Originals are retained upstream; the change is additive and reversible.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

