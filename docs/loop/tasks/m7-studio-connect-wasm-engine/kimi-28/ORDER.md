# ORDER: m7-studio-connect-wasm-engine, kimi-28 (de-stale docs/products/web-client.md after #2990 and the #3072 flip)

Lab: kimi. Item: [m7-studio-connect-wasm-engine](../../../../roadmap/items/m7-studio-connect-wasm-engine.md) (#2868).
Base: `fb6156b46` (content commit lands after PR #3072 merges, so the item flip it references is on main).

## Goal

`docs/products/web-client.md` still describes the pre-#2990 page: it says the Studio loads third-party code and fonts from CDNs (Monaco, pdf.js, three.js from cdnjs; marked, KaTeX from jsDelivr; Google Fonts) and that self-hosting every CDN asset is wholly missing. On main, #2990 (`9baf8452c`) already replaced the main page's CDN loads with `web/studio/assets/css/uor_offline_fonts.css` and `web/studio/assets/js/uor_offline_vendor.js`, and the Wikipedia/DuckDuckGo prompt search is disabled. What actually remains is: the router demo page (`web/studio/geometric_prime_router_webapp.html`) still loads Tailwind, Font Awesome and Google Fonts (lines 9/34/37); the page-load zero-external-requests claim is asserted by the shims but not yet machine-checked; and user-initiated GitHub features talk to `api.github.com` (refused under Tauri/offline). The residual item `m7-studio-self-host-page-assets` (created by #3072) carries "`docs/products/web-client.md` updated to match" in its done-when — this unit is that update, split out of the umbrella PR because #2904's scope does not cover `docs/products/**` while this item's does.

## Cancel probe (run first)

Class docs (host-only). At `fb6156b46`: `grep -n "cdnjs" docs/products/web-client.md` still finds the privacy bullet's CDN list as present-tense Studio behaviour, while `web/studio/index.html` at the same base loads the offline shim assets instead. Task stands; the edit waits for #3072's merge so the item-status wording it touches is true.

## Files in scope

- `docs/products/web-client.md`
- this task folder

## Done when

- The rewiring-missing-piece row, the self-hosted-assets row and the privacy bullet describe the post-#2990, post-#3072 main: main page loads no CDN assets (#2990 shims), the item is `done` via kimi-27 (PR #3072), the router demo page's three CDN loads and the unverified page-load claim are named as the residual carried by `m7-studio-self-host-page-assets`, and the `api.github.com` calls are recorded as user-initiated and refused under Tauri/offline.
- No other file changes; `loop report --check`, `loop pr-check --local` and `loop gates --full` pass; the PR body carries `References #2868` and `References #2904` (the item's issue closes via the post-#3072 roadmap sync, not via this PR).

## Stop and ask if

- #3072 does not merge, or merges with different item wording — this unit's text must be re-based on whatever main actually says.
- Any file beyond `docs/products/web-client.md` and this task folder needs to change.
