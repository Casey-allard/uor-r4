# ORDER: m7-studio-self-host-page-assets, deepseek-51 (page-load verification, demo-page CDNs, stack-bound tokenizer)

Lab: deepseek. Item: [m7-studio-self-host-page-assets](../../../../roadmap/items/m7-studio-self-host-page-assets.md) (residual of #2868, M7 #2035). Base: `7bbaadcdd`. `References #2868` until the item's issue exists (PM roadmap sync), then `References` that issue.

## Goal

Close the residual left by the m7 Studio item: (1) make the **page-load** zero-external-request claim checked rather than asserted for `web/studio/index.html`; (2) remove the three leftover CDN loads on `web/studio/geometric_prime_router_webapp.html` (Tailwind, Font Awesome, Google Fonts) by reusing the offline shims or retiring the page; (3) replace the byte-mod prompt tokenizer with a stack-bound tokenizer artifact bundled for the shipped stack; (4) record the position on the user-initiated `api.github.com` calls. Pages deploy stays paused (D36) and no bar is registered or changed.

## Cancel probe (run first)

Class **instrument/build** (host-only, no GPU). At the base: `grep -n 'cdn.tailwindcss.com\|cdnjs\|fonts.googleapis.com' web/studio/geometric_prime_router_webapp.html` finds the three CDN loads; `grep -n 'remoteRequests' web/studio/verify_studio_worker.mjs` shows the worker-only trap set (no index.html page-load check); `grep -n 'rawBytes\[i\] % vocabSize' web/studio/assets/js/uor_model_worker.js` shows the stand-in tokenizer. Task stands.

## Files in scope

- `web/**` (Studio page, worker, verification, tools), `docs/products/**`, `docs/labs/m7-studio-self-host-page-assets/**`
- this task folder

## Plan

1. Design note first (on the parent #2868 until the item issue exists): page-load trap design (static scan of every `src`/`href`/`url()` plus a DOM-level fetch trap if a headless path is available), the demo page decision (offline-shim reuse preferred, retire only if shims cannot cover), the stack-bound tokenizer artifact format and how it is generated from the shipped LUT's header at build time (no new Python; JS/Rust only, and note that editing `mk-synthetic-stack` is outside this item's scope), and the GitHub-API position (user-initiated, not load-time, refused under Tauri/offline — state whether that satisfies the item's bar).
2. Implement the page-load verification first (failing check against the current page/CDNs), then the shims/retire change, then the tokenizer artifact + worker load path + truthful capability label.
3. `docs/products/web-client.md` updated to match; notebook row; `uor-harness check`, `loop report --check`, `loop pr-check --local`, `loop gates --full`; counted non-author review (code PR), then queue/verify/cleanup.

## Done when

- The verification fails today's page/CDNs and passes after the change; `web/studio/index.html` and (if kept) the router demo page make zero external requests on load; the tokenizer is a bundled stack-bound artifact (loading path + truthful label), or the exact artifact-prerequisite is recorded if provisioning is impossible inside this item's scope; GitHub-API position recorded; `docs/products/web-client.md` current.
- No Pages deploy change (D36), no owner path, no bar; `References` the item issue.

## Stop and ask if

- A missing asset cannot be self-hosted from resources already in the repository, the demo page must be deleted as a product call the owner has not made, the stack-bound tokenizer needs a file outside this item's scope, or the GitHub-API calls must change behaviour rather than be documented.