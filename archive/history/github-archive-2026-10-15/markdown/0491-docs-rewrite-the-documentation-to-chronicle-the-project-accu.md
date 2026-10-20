# #491 — docs: rewrite the documentation to chronicle the project accurately

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-08T00:40:19Z
- **Updated:** 2026-08-08T01:08:05Z
- **Closed:** 2026-08-08T00:59:44Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/491
- **Merged:** 2026-08-08T00:59:44Z
- **Merge commit:** 94c0fbf4289140832e9dd53e5a8b809fb89c8513
- **Base / Head:** main / UOR-Foundation:docs-accurate-state
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The README was an **865-line lab notebook with a manual appended**. 178 lines of research negatives sat between "Current status" and "Requirements"; the genuinely zero-setup path was at line 343; there was no install step, no project layout, no license section and no contributing pointer. It also carried claims this week's measurements overturned.

## Structure

README **865 → 490 lines**, reordered for someone arriving cold: what it is → quickstart (three tiers, cheapest first) → requirements → what actually works → layout → architecture → CLI → HTTP → config → testing → troubleshooting → docs map → contributing → license.

Three big blocks moved out whole rather than being deleted:

| New file | Was |
|---|---|
| `docs/RESEARCH.md` | the 178-line research programme, corrected |
| `docs/MODEL_LIFECYCLE.md` | the 262-line 7-step compile tutorial |
| `docs/CONFIGURATION.md` | new — every env knob, defaults and owning module |
| `CONTRIBUTING.md` | new — the short form of AGENTS.md |

## Corrections

- The README asserted the `* 100` weight **"has never been measured against alternatives"** three sections after a table row saying #484 closed it.
- It quoted the **two-change** #486 number (0.8763) as the **one-change** number (0.8542).
- It called **TLA5 "the current artifact format" in seven places**; compiles have emitted TLA7 by default since #318 Phase B, with TLA6 as fallback and `R4_TLESS_TLA6/7=0` to opt out. Neither appeared anywhere.
- CI was described as "every push" and omitted the claim-wording gate, the BDD suite, Gate E, the Gate C trend alarm and the docs-only fast path.
- The **clippy invocation was missing `--all-features`** — a contributor following the README passes locally and fails CI.
- #434's Spectral row still credited the geometry with a score #486 showed was lexical.
- `#456–#459` read "Active track" with three of four landed, one of them a negative worth recording.
- `score_report.json` is schema **26** and the README gave no schema at all, after twenty-six fields moved under `gate_c.right_context_arms`.

## Added

Badges, install, license, project layout, and a real **60-second quickstart** — the router and dashboard need no model, no download and no compile, which was true all along and buried at line 343.

Also: the OpenAI-compatible `/v1` surface (which is what the shipped client actually speaks, and was entirely undocumented), the full engine-selector semantics including that **omitting `engine` runs the cascade rather than `transformerless`**, a troubleshooting table led by the vacuous-green κ trap, and the harness inventory with the cheap gate that must precede any long run.

The **router crate README now opens with what #484/#486 established**, because it is the first thing anyone touching retrieval needs: the deployed ranking is word overlap, the cosine was comparing a routing vector against a content vector, the fix is default-off pending #490 — plus the three gotchas that have cost real measurements (identity-scoped retrieval, `window_index` is not a store index, `slice_norm` is per-bucket).

## Not docs-only

Two dead paths printed by `r4 transformerless --help` and in the transformerless module header (`docs/PROOF.md` → `docs/transformerless/PROOF.md`). That is CLI-visible documentation, so it belongs with this change even though it costs the fast path.

Verified: all internal links resolve, claim-wording gate passes, fmt/clippy `--all-features` clean, wasm target checks.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

