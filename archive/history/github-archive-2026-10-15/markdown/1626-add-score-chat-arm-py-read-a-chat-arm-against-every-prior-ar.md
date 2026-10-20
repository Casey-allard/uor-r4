# #1626 — Add score-chat-arm.py: read a chat arm against every prior arm

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T17:19:50Z
- **Updated:** 2026-10-02T17:21:20Z
- **Closed:** 2026-10-02T17:21:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1626
- **Merged:** 2026-10-02T17:21:20Z
- **Merge commit:** c40b5469c56b3255ddae31fe49a09b258400abff
- **Base / Head:** main / UOR-Foundation:codex/score-chat-arm-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What and why

One command to read a chat arm against every prior arm on this lane.

**This lane has published a number and retracted it three times**, and two of those were instrument failures rather than model failures:

1. a reply rubric that scored the arm **2/38** because it did not know the panel states each fact in the row's *first* turn — the model had answered correctly and the rubric missed it;
2. a "held-out" panel that changed the question wording but **reused the training relations and values**, producing a reported **2/10 → 6/10 gain** that vanished on a disjoint panel;
3. a comparison that was not like for like (an unmatched reference read as a matched control).

This tool exists so the same mistakes are harder to make. It **pins every arm this lane has measured with its sealed numbers**, so a new arm is read against all of them rather than against whichever is convenient. It scores the replies by the value each memory row actually states, and it prints the **geometry-on-principle** labelling note, so a parity result cannot be reported as an advantage.

## Exact-head evidence

**Reviewed head:** `5a4779d1` (one commit on `origin/main`).

Validated against **two arms with known sealed results**, which is the only meaningful test for a scorer:

| arm | sealed pooled | sealed first-four | tool reports | memory | distinct |
|---|---:|---:|---|---:|---:|
| `chat-prose-init-phase-a-1` | 2.097073 | 1.417128 | **exact match** | 4/10 | 38/38 |
| `chat-mixedphaseB-seed1-1` | 2.207584 | 1.544123 | **exact match** | **10/10** | 37/38 |

The second row is the point: that arm's **10/10 memory is the template leak** the disjoint panel was built to expose, and the tool prints it next to the winner's 4/10 rather than presenting it as an improvement.

Failure paths exercised: a root with no sealed report (`no sealed report yet`) and a root that does not exist (`MISSING`) both exit cleanly with a clear message rather than a traceback.

## Scope

A measurement tool. It makes **no capability claim** and changes no model behaviour. The reference numbers are this lane's development results on a derived panel, labelled as such in the source, and the printed note records that the matched geometric-vs-transformer measurement is **1.9981 vs 2.0111 (Δ −0.0130, inside the ±0.021 seed spread) = parity**, so no result read through this tool can imply a geometric advantage that has not been measured.

References #1512, #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

