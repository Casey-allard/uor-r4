---
id: infra-loop-render-review
title: Let loop review count the render PR (or exempt it at the queue)
milestone: infra
area: tooling
status: done
open_to: lab
needs_gpu: false
depends_on: [infra-loop-harness]
scope: [crates/uor-r4-harness/src/loopreview.rs, crates/uor-r4-harness/tests/loop_review*.rs, docs/roadmap/items/*.md]
---

## Why it matters

The steady-state render PR (branch `kimi/render`, loop page step 14) carries no ORDER, so `loop review --check` refuses it with "adds no ORDER" and no review record can be counted for it. Today it merges on green CI plus a prose verification comment (PM decision, #2028 comment 6357306850), which leaves the one recurring PR outside the counted-review record the rest of the loop keeps.

## What is missing

`loop review` (and the queue's review-record check) has no path for a PR whose whole content is the deterministic render output: no ORDER exists by design, and the real gates are `loop render --check` plus the views-only diff rule. Claude's review of the first render PR (#3031) named exactly this gap.

## Done when

A `kimi/render` PR can earn a counted review record without an ORDER — either `loop review` accepts the render branch with `render --check` and the views-only rule as its gates, or the queue treats it as review-exempt behind those same checks. The planted fault turns the check red: **a hand-edited STATE.md on `kimi/render` fails review.**

## How to start

1. Read the loop page's "The render PR" paragraph and `crates/uor-r4-harness/src/loopreview.rs`'s use of `added_order` (#3031's review thread has the full context).
2. Keep the change small: one branch-name special case, tests both ways (render PR without ORDER counted; a non-render PR without ORDER still refused; a tampered view on `kimi/render` rejected).
3. ORDER and REPORT in `docs/loop/tasks/infra-loop-render-review/<lab>-<n>/`; the exact-head review comes from another vendor's lab.

## Rules that apply

- **Delivery path** (CHARTER §5.5): PR, exact-head review from another vendor's lab, merge queue, verify on fresh `main`.
- **Rust first** (§3.11): the harness is Rust; no new scripts.
- **The rendered views stay locked** (loop page, "The render PR"): lab PRs change inputs only; `kimi/render` changes the views.

## Sources

- [`docs/loop/README.md`](../../loop/README.md)
- [`crates/uor-r4-harness/src/loopreview.rs`](../../../crates/uor-r4-harness/src/loopreview.rs)
- issue#2904

## Delivered (2026-10-23)

- **#3035 merged as `6e9d6db41`**: the render branch can earn a counted `uor-review v1` record with the sentinel `order: kimi/render` and no ORDER; the covered views are STATE, HANDOFF and LESSONS, with the AGENTS top-ten block marker-gated (outside bytes equal the merge base, the block stamp equals the merge base, the block equals `render_top_ten` at that stamp); every stamp must equal the PR merge base with `origin/main`; forks are refused; the gate reads the explicit head sha; and `ReviewContext::of` mirrors the sentinel so the queue's review-record and after-approval rules work. Hand-edited views are refused by name (the planted fault).
- **#3038 merged as `885df8f3e`**: the first real render PR reviewed through the path (stale-base refusal, refresh, counted record).
- Remaining hardening (context fixture, roadmap-README gating, loop-page wording) is carded as [`loop-render-review-hardening`](loop-render-review-hardening.md).
