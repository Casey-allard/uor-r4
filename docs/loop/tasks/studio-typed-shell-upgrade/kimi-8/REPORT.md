# REPORT: repair rich-message link and fence parsing, kimi-8

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `d0d7048f9066ae4786d98769121400edd48f98e3`. Prior reviewed head: `181bd625aceccf4b305bfc8e28164fbc321a3aef`, tree `3273eef6eef649ac03c0958ce0dbdb6eb27a14bf`. The new exact delivery-head commit/tree and final logs are recorded on PR #3818 rather than embedding a self-referential hash here.

## What did not work

- Fresh review found no executable-HTML/XSS path but returned CHANGES for two parser correctness defects. The CHANGES record is posted on the PR as comment `6707594873`.
- Link matching stopped at the first `)`, truncating a Wikipedia-style destination and turning an unbalanced destination into an anchor. The new balanced-link test initially produced `https://en.wikipedia.org/wiki/Function_(mathematics` without the final `)`, and the unbalanced case produced an anchor instead of literal text.
- Fence matching accepted only a narrow unspaced identifier. `c++` therefore became a real heading inside intended code and the closing fence could swallow following prose. The new fence test first failed with an actual `<h1>code heading</h1>`.
- Those missing cases made the initial REPORT's broad malformed-link/fence wording too strong. This repair updates the evidence and counts rather than hiding the prior claims. kimi-7's 341-test record remains historical.
- No live-page cutover, production publish, browser-data migration, useful conversation, model capability, M7/M8, production-readiness, or accessibility-certification result is claimed.

## What changed

- Markdown link matching now accepts balanced one-level parenthesized groups inside destinations, including multiple separate groups. The complete Wikipedia URL is preserved; an unbalanced destination does not match and remains literal. Existing unsafe-scheme, relative-link, raw-HTML, Unicode, and no-fetch behavior is unchanged.
- Fence starts accept an optional general info string with whitespace/punctuation. Language classes derive only from the first token after replacing unsupported runs with single dashes and trimming boundary dashes, so `c++` becomes `language-c` and spaced `rust` becomes `language-rust`. Code bodies remain inert and following prose remains outside the block.
- Added exact red-first regressions for the balanced Wikipedia URL, unbalanced destination, and `c++`/spaced fence tags. All seven renderer tests and all 26 AgentComposer tests now pass.

No dependency, legacy page/controller, offline vendor shim, engine, worker, tokenizer, `studio_api.js`, model artifact, publisher, deployment, `.github/**`, `Cargo.lock`, real browser data, owner-controlled path, or generated roadmap/issue file was changed. Only the typed parser/tests and authorized records changed.

## Measurements

- Focused renderer/component checks: **33/33 tests pass** (`7` renderer plus `26` AgentComposer).
- Complete typed suite: **343/343 tests pass across 42 files**.
- Plain-JS suite: **112/112 tests pass** at the prior head and rerun at this repair head before delivery.
- `npm run typecheck`: PASS with no TypeScript errors.
- `npm run build`: PASS; `dist/studio-foundation.js` **141.33 kB** (gzip **39.46 kB**) and `dist/studio-foundation.css` **4.85 kB** (gzip **1.41 kB**), uncommitted.
- Repaired-head plain-JS, strict page, runtime page-load, desktop-client, worker, claim/report/local-policy and full-gate results are recorded in the new exact-head logs before fresh review and queueing.

## Checks run at the head

`red-parser-repairs.log` retains all three intended failures. `green-parser-repairs.log`, `full-vitest-repairs.log`, `typecheck-repairs.log`, and `build-repairs.log` record the repaired implementation. New exact-head suite/gate/diff files supersede the 181bd625 evidence; all earlier unique logs remain preserved locally under `web/studio/.superpowers/sdd/studio-safe-rich-replies/checks/`.

The D76 scope advisory covers the ORDER-authorized notebook and implementation plan; the ORDER-first commit check passes. FILEMAP advisories concern unrelated existing folders. No generated roadmap/issue scope was edited, and no new warning was introduced.

Fresh exact-head review, green GitHub checks, protected merge, and fresh-main verification remain required. PR #3818 stays draft until a new review approves this head. It uses `References #3500`, and the issue remains open.

## Skills used

Receiving code review with source verification, red-first TDD, parser security/correctness analysis, and verification before completion. A fresh independent exact-head delivery reviewer follows the repaired commit. No model compute, paid resource, credential, real user data, or production publish was used.

## Still open

The typed library remains unmounted. Arbitrarily nested/custom Markdown constructs, tables, math, raw HTML, images, syntax highlighting, full real-browser/mobile/offline parity, migration, cutover, production publication, and useful model conversation remain outside this repair. #3500 remains open for its full completion contract.

## Plain summary

Balanced parenthesized links retain their complete destination, malformed links remain inert, and punctuation/spaced fence tags keep code content inert with sanitized classes. The expanded component suite is green; no live application or model capability claim is made.
