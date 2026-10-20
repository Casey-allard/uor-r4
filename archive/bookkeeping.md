# Bookkeeping: keeping the README and the geometry page current

**Proposed 13 October 2026 by the Claude lab at the owner's request, for the PM (Kimi) to adopt into the
shared plan.** The rules in [pm-rules.md](harness/pm-rules.md) govern how work moves between labs. This
page covers one narrower question: how the public face of the project (the [README](../README.md), the
[geometry page](geometry.md), the [mechanism index](mechanisms/README.md) and the generated figures) stays
true without becoming a running log.

## Principle

The README is a curated overview, not a record. Every number on it is copied from a record that already
exists and links back to that record. When the two disagree, the record wins, and the README is
corrected.

## What each surface shows, where its truth lives, and when it changes

| Surface | Source of truth | Changes when |
|---|---|---|
| README status line and badges | [current-state.md](integration/current-state.md), the owner holds | a hold is lifted or set, or a milestone completes |
| README milestone table | milestone issues #2029–#2036 and [milestones.json](harness/milestones.json) | a headline acceptance number moves, or a milestone's status changes |
| README "current path" table | the [operator plan](labs/operator-phase-2026-10-12/README.md) and the active rung sheet in [harness/prep/](harness/prep/) | an operator merges, a dependency lands, or a rung reports |
| README rules table | [DECISIONS.md](integration/DECISIONS.md) | a decision is added or superseded |
| README and index mechanism status | the brief in [mechanisms/](mechanisms/README.md) | a brief's status changes (KEEP, built, held, reopened) |
| README results table | the dated record `docs/labs/<topic>-<date>/README.md` | a result that moves a headline, or a KEEP, merges |
| Figures (`docs/figures/*.svg`, `docs/figures/geometry/{compliant-path,operators}.svg`) | `xtask/src/figures.rs` | any of the above changes something a figure draws |
| Geometry page, compliant path and operators | the operator source and its records | an operator on the executed path changes |
| README statistics | `git log`, `gh pr list`, the workspace | about once a month |

## Rules

1. **Copy, scope, link.** A README number carries its scope, meaning its artifact, panel and form, and
   links to its record. Pre-D23 float-form results are marked *historical*.
2. **No journal.** Dated progress goes in records and in current-state, never in the README. The README
   changes only when a capability, a headline result, a milestone, a command or a mechanism changes.
   This rule matches [CONTRIBUTING § Delivery cadence](../CONTRIBUTING.md#delivery-cadence-every-completed-unit-owner-9-october-2026).
3. **Shape.** The always-visible part covers the question, the architecture, where we are, the current
   path, the rules, the mechanisms, the key results, bookkeeping, how the project is run and the quick
   start. Long tables (all results, models and data, the repository map, statistics) go in `<details>`
   sections. A new section needs a reason.
4. **Figures are generated, not drawn by hand.** The status figures are built by
   `cargo run -p xtask -- figures --write` from the data in `xtask/src/figures.rs`, and
   `cargo run -p xtask -- figures` fails when a committed SVG no longer matches. The generator also
   refuses text that overflows its box. Change the data, regenerate, and commit the code and the SVGs
   together.
   The architecture also has a generated narrow layout. Embed it with the
   existing `<picture>` source in the README and geometry/source-map pages;
   keep the desktop SVG at its native landscape proportions. Detailed
   mathematical figures retain their original 1000×700 canvas and links to
   the vector originals for zooming.
5. **Same PR as the change, when it is small.** A lab whose merged unit changes one README cell or one
   mechanism status updates it in its own delivery PR. Anything larger, such as a figure, a new
   section or a restructure, goes to the README keeper.

## Who does it

- **Each lab** keeps its own records, current-state entries and mechanism briefs, and updates the
  single README or index cell its unit changes (rule 5).
- **The PM (Kimi)** decides when the overview needs a larger update and says what changed. It does
  this in one line on #2028, for example: "README: M1 headline moved to 52/232; operator table: OP3a
  landed".
- **The README keeper (Claude, proposed)** turns that into one PR: README text, regenerated figures,
  the geometry page and the mechanism index, with every link checked. The keeper does not choose what
  the project's state is; it reports the state the records and the PM give it.

## Follow-ups

- Port `docs/figures/geometry/generate.py`, the mathematical figures, to the Rust generator (D27). The
  figures it draws are math and change rarely, so this is not urgent.
- Run `cargo run -p xtask -- figures` in the harness `check` or the PR checks, so that a stale figure
  fails the same way a stale document does.
