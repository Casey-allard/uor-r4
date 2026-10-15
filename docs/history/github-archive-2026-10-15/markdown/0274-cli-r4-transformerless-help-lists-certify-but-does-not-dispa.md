# #274 — CLI: 'r4 transformerless --help' lists 'certify' but does not dispatch it

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T23:27:46Z
- **Updated:** 2026-07-29T16:46:28Z
- **Closed:** 2026-07-29T16:46:28Z
- **Labels:** bug
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/274
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Symptom

`r4 transformerless --help` lists `certify` among its subcommands, but `r4 transformerless certify` does not dispatch — certification is only reachable as the ROOT subcommand `r4 certify`.

This is not cosmetic: during the issue #244 measurement setup, an automated pipeline invoked `r4 transformerless certify` based on the help text and stalled. Anyone scripting against the help output will hit the same wall.

## Expected

One of:

- (a) remove `certify` from the `transformerless` help listing (minimal, honest help text), or
- (b) make `r4 transformerless certify` forward to the root `certify` dispatch.

Recommendation: (a). Certification spans the graph path too (`r4 certify` drives `uor-r4-graph-certify`), so it is not conceptually a `transformerless`-scoped command; forwarding would entrench a misleading location.

## Scope

Help-text/dispatch only in `uor-r4-graph-cli`. No behavior change to certification itself. Standard 4-gate local checks apply.

Observed on main @ current HEAD during the #244 measurement runs (2026-07-28).

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

