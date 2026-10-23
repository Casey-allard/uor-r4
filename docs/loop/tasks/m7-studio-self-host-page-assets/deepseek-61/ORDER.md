# ORDER: m7-studio-self-host-page-assets, deepseek-61 (self-host the router demo page's three CDN loads)

Lab: deepseek. Item: [m7-studio-self-host-page-assets](../../../../roadmap/items/m7-studio-self-host-page-assets.md) (#3075). Base: `e8d6a624b`. Partial unit: `References #3075`; the item stays `ready` for the real-corpus-tokenizer leg, which needs an accepted M4 export.

## Goal

Remove the router demo page's three external CDN loads (`cdn.tailwindcss.com`, cdnjs Font Awesome, Google Fonts) by self-hosting: compile the page's own inline `tailwind.config` with the Tailwind v3.4.17 standalone CLI against the page's classes into `web/studio/assets/css/router_demo_offline.css`, replace Font Awesome with local unicode icon rules and Google Fonts with the repo's existing system-font fallback pattern, and prove zero external page-load requests with the static scan and the headless-Chrome runtime trap (`--strict-router`). Update the product claim and the lab record. No owner/product decision is needed: the item's text allows "self-host them", and this is that option rather than a page retirement (which would need archive scope).

## Cancel probe (run first)

At base `e8d6a624b`: `web/studio/geometric_prime_router_webapp.html` loads `https://cdn.tailwindcss.com`, the cdnjs Font Awesome stylesheet and the Google Fonts stylesheet; `node web/studio/verify_studio_page_load.mjs --strict-router` fails on the page with 8 external requests (and `verify_studio_page.mjs --strict-router` reports its 3 load references). The task stands.

## Files in scope

- `web/studio/geometric_prime_router_webapp.html`
- `web/studio/assets/css/router_demo_offline.css` (new)
- `web/studio/verify_studio_page.mjs` (record-only note about the existing runtime trap)
- `docs/products/web-client.md`
- `docs/labs/m7-studio-self-host-page-assets/**`
- this task folder

## Done when

- The demo page has no external load references; its styling comes from the locally generated Tailwind subset (built from its own config), its icons from local unicode rules and its fonts from system fallbacks.
- `node web/studio/verify_studio_page.mjs --strict-router` is clean for both Studio pages; `node web/studio/verify_studio_page_load.mjs --strict-router` reports 0 external requests for both pages and its planted-external self-test still fires.
- `docs/products/web-client.md` states the self-hosted result and keeps the user-initiated GitHub API position; the lab README records the method, results and caveats; the item stays `ready` (real tokenizer leg open).
- `loop report --check`, `loop pr-check --local` and `loop gates --full` pass; a counted non-deepseek review records at the exact head.

## Stop and ask if

- A page retirement/archive step, owner decision, GPU run or file outside the listed scope is needed.