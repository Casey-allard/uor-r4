---
id: m7-studio-self-host-page-assets
title: Verify the Studio's offline claim, retire the demo CDNs, bundle tokenizer
milestone: M7
area: serving
status: done
open_to: anyone
needs_gpu: false
depends_on: []
scope: [web/**, docs/products/**, docs/labs/m7-studio-self-host-page-assets/**]
---

## Why it matters

The Studio's model worker is fully client-side (m7-studio-connect-wasm-engine, PR #2978), and #2990 (merged `9baf8452c`) already replaced the main page's CDN assets with offline shims (`web/studio/assets/css/uor_offline_fonts.css`, `web/studio/assets/js/uor_offline_vendor.js`). What remains is proving that claim at page load, three leftover CDN loads on the router demo page, and the stand-in prompt tokenizer.

## What is missing

1. **Verification and both pages are done.** The page-load claim is machine-checked statically (`#3073`) and at runtime in headless Chrome (`#3080`), and the router demo page's three CDN loads are self-hosted behind the same traps (`#3110`, `0d5d6791d`). The tokenizer leg remains.
2. **The tokenizer.** The byte-mod stand-in is real (`web/studio/assets/js/uor_model_worker.js:140`, `rawBytes[i] % vocabSize`); bundle the stack-bound tokenizer artifact in its place.
3. **GitHub API calls, recorded.** `web/studio/index.html:2866-2872` calls `api.github.com`; the calls are user-initiated and refused under Tauri/offline. The privacy model in `docs/products/web-client.md` records them as user-initiated, **not** page-load requests, and therefore outside the "on load" bar (`#3074`); no change to the calls is needed.

Website deploys stay paused (D36) regardless; this item changes assets and verification, not the deploy.

## Done when

`web/studio/index.html` and (if kept) the router demo page make zero external requests on load, checked by the extended trap verification; the stack-bound tokenizer artifact replaces the stand-in; the GitHub API calls' position is recorded in the item's record; `docs/products/web-client.md` updated to match.

## How to start

1. Read #2990's offline shims and its record, the "Still open" list on #2868, and claude's review of PR #3068 (which enumerated (a)–(d)).
2. Post a design note on the issue covering: how page-load trapping verifies zero external requests, the demo page's self-host-or-retire decision, how the tokenizer artifact reaches the page, and the position on the user-initiated GitHub API calls.
3. Implement, extend the verification, and open a PR that says `Closes #<this issue>`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): no softmax or squashing, no dense matrix multiplication, including in any asset the page ships.
- **Records** (§4.1, §5.6): one row in `docs/research/notebook/m4-serving.md`.

## Sources

- [m7-studio-connect-wasm-engine](m7-studio-connect-wasm-engine.md) (resolution note)
- PR #2990 (offline shims), PR #2978 (worker rewiring), issue #2868
- claude's review of PR #3068 (residual enumeration)
- [`docs/products/web-client.md`](../../products/web-client.md)
