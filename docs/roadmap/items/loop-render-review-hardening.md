---
id: loop-render-review-hardening
title: Harden the render-PR review path
milestone: infra
area: tooling
status: done
open_to: lab
needs_gpu: false
depends_on: [infra-loop-render-review]
scope: [crates/uor-r4-harness/src/loopreview.rs, crates/uor-r4-harness/src/looprender.rs, crates/uor-r4-harness/tests/loop_review*.rs, docs/roadmap/items/*.md]
---

## Why it matters

The render-PR review path (#3035) closes the last ORDER-less merge hole, but two reviews recorded the gaps left open on purpose: the gate has no end-to-end test that fails when its render check is removed, the xtask-rendered `docs/roadmap/README.md` is still outside the no-ORDER path, and the loop page's "The render PR" paragraph does not yet name the sentinel `order: kimi/render`.

## What is missing

1. **No fake-git `context()` fixture pins the gate.** Replacing the `verify_render_views` call with a no-op, or disabling the `stamp != base` check, still leaves every `loop_review` test green (each run was demonstrated on #3035). A fixture should fail in both cases, and also for a cross-repo PR, a removed `ReviewContext::of` fallback, an AGENTS edit outside the block, and a hand-edited top-ten block.
2. **`docs/roadmap/README.md` is excluded from the no-ORDER path** until it has its own tamper comparison (an xtask render equality check at the stamp, like the AGENTS block gate).
3. **The queue mirror still admits a fork named `kimi/render`.** `loopprcheck::ReviewContext::of` takes the sentinel path on `head_ref == "kimi/render"` alone, and `headRefName` carries no owner, while `loop review` rejects `isCrossRepository`. It is reachable only by a hand-typed record today, but the mirror must require `isCrossRepository == false` too, with the field plumbed into the queue/PR metadata. **Note:** `loopprcheck.rs` is one of the nine shared registration files (loop page), so it is edited as an always-allowed path and must NOT appear in this item's scope; the test for the change goes in the PR's own test file.
4. **The documented refresh still refuses itself.** After merging main forward, `loop render` stamps HEAD (the branch's merge commit), while the gate requires the stamp to equal the merge base; Kimi had to re-stamp #3038 at the base. Either default `loop render`'s stamp to `$(git merge-base origin/main HEAD)` on `kimi/render` (why `looprender.rs` is in scope) or make the loop-page wording require `--at <merge-base>`. The item must fix one of the two, not just name the sentinel.
5. **Nit for the REPORT owner:** deepseek-42's REPORT has a doubled prefix (`infra-infra-loop-render-review`); the item may record it so whoever next touches that REPORT fixes it.
5. **The loop page wording** ("The render PR") is an owner-only rule source: it should say the branch is `kimi/render`, the views are the four covered paths, the record uses `order: kimi/render`, and how the stamp is set (`--at <merge-base>` or a default); the PM/owner must make that edit.

## Done when

A `context()` fake-git fixture fails when the render check is removed and when `stamp == base` is disabled, and covers the cross-repo, fallback-removed, AGENTS-outside-block and hand-edited-block cases; **`ReviewContext::of` refuses a cross-repository PR named `kimi/render`**; the documented merge-main-forward refresh passes without hand-special-casing (either `loop render` defaults the stamp to the merge base on `kimi/render`, or the loop-page wording requires `--at <merge-base>`); `docs/roadmap/README.md` is either gated by an xtask render comparison at its stamp or explicitly exempted with a recorded reason; and the loop-page wording change is either made by the owner or carded. The planted faults turn the new fixtures red: **a no-op render check must fail the suite, and a fork PR on `kimi/render` must not pass the queue mirror.**

## How to start

1. Read `crates/uor-r4-harness/src/loopreview.rs` (`context`, `verify_render_views`, `top_ten_markers`) and the fake-gh format (`REVIEW_GH_FAKE_ENV`) in the same file.
2. Build the fixture as a test that creates a temporary git repository with a merge base, a render commit and a PR head, plus a fake gh file; run `context()` through it.
3. For the roadmap README, reuse the xtask render path at the stamp commit or record why it stays out.

## Rules that apply

- Delivery path (CHARTER §5.5): PR, exact-head review from another vendor's lab, merge queue, verify on fresh `main`.
- Rust first (CHARTER §3.11); tests stay offline (no network in the fixture).

## Sources

- [`docs/roadmap/items/infra-loop-render-review.md`](infra-loop-render-review.md) (#3035 `6e9d6db41`, #3038 `885df8f3e`)
- reviews on #3035 (Claude 06:26Z, Kimi 05:24Z) and the follow-ups carded on #2028 comment 6358961404

## Delivered (2026-10-23, #3048)

- **Queue-mirror fork check:** `loopprcheck::Pr` carries `is_cross_repository`, `PrGh::is_cross_repository` reads it, and `ReviewContext::of` requires a known same-repository head via `loopreview::render_sentinel` (fork and unknown both refuse); the pr-check `--pr` path fills the flag. `loopprcheck.rs` is a shared registration file, so the test lives in `tests/loop_review.rs`.
- **Render-at-merge-base:** `looprender::default_at(root, branch)` returns `git merge-base origin/main HEAD` on `kimi/render` and `HEAD` elsewhere; `loop render` uses it, so the documented merge-forward refresh passes.
- **`context()` fixture with negative gates:** a temp-git repository plus fake gh and fake render files drive `render_gate` offline; it fails if the render check is removed (tampered view/block pass) or the stamp rule is disabled (branch-only stamp passes). Cases: clean, hand-edited view, fork, AGENTS edit outside the block, hand-edited top-ten block, branch-only stamp. The canned renderer is injected as a Rust `RenderSource` through `cli_with_renderer`; there is no environment hook in the shipped binary.
- **Roadmap README:** recorded as **exempt with reason** - xtask renders the working tree, the views-only set keeps it out of the no-ORDER path, and the docs job gates it advisably. The loop-page wording is owner-only and carded on #2028.
- **REPORT nit:** the doubled `infra-infra-` prefix in deepseek-42's REPORT is recorded here for the report owner; that file is outside any claimable item scope, so a lab PR cannot edit it (the PM can fix it when next touching the folder).
