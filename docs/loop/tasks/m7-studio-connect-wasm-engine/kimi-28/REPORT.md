# REPORT: m7-studio-connect-wasm-engine, kimi-28 (de-stale docs/products/web-client.md)

Lab: kimi. Order: [ORDER.md](ORDER.md). Item #2868. Base: `fb6156b46`, content on main `7bbaadcdd` (post-#3072). `References #2868`, `References #2904` (the item's issue closes via the post-#3072 roadmap sync, not via this PR).

## What did not work, first

- **`web-client.md` described the pre-#2990 page.** It listed self-hosting every CDN asset as wholly missing and its privacy bullet said the Studio loads third-party code and fonts from CDNs in the present tense. On main, #2990 (`9baf8452c`) already replaced the main page's CDN loads with `web/studio/assets/css/uor_offline_fonts.css` and `web/studio/assets/js/uor_offline_vendor.js`, and prompt search is disabled (`index.html:5849`, `isWebSearchActive = false`). The doc overstated the gap.
- **The first draft of this de-stale rode the wrong PR.** It was part of kimi-26 (#3068) until two counted CHANGES (deepseek 6366395951, claude 6366462558) found `docs/products/**` outside the umbrella item's scope; split out per deepseek's two-PR proposal, and its wording was held until #3072 merged so no sentence asserts unmerged state (the original said "item done (kimi-26, PR #3068)"; #3068 closed unmerged).

## What changed

| File | Change |
|---|---|
| `docs/products/web-client.md` | Three de-stales: the rewiring row now records the item as `done` (kimi-27, PR #3072); the self-hosted-assets row credits #2990's offline shims and names the router demo page's three CDN loads plus the unverified page-load claim as the residual; the privacy bullet states the post-#2990 main page, the disabled search, the residual, and the user-initiated `api.github.com` position (refused under Tauri/offline) |

## Measurements

- None: docs-only, every claim checked against the tree at the head — `web/studio/index.html:11,14` load the shim assets, `:5849,5852` hold `isWebSearchActive = false`, `web/studio/geometric_prime_router_webapp.html:9,34,37` load Tailwind/Font Awesome/Google Fonts, `web/studio/uor_model_worker.js:140` holds the byte-mod stand-in, `index.html:2866-2872` holds the `api.github.com` calls.

## Checks run at the head

`uor-harness loop report --check`, `loop pr-check --local`, `loop gates --full`.

## Skills used

- `scope-guard`: one docs file plus this task folder, inside the item's `docs/products/**` scope.
- `solution-integrity`: each sentence was re-verified against the post-#3072 tree rather than carried over from the kimi-26 draft.

## Still open

- The residual itself (router demo page CDN loads, page-load trap verification, tokenizer artifact, `api.github.com` position) is the work of `m7-studio-self-host-page-assets`, not this unit.

## Plain summary

- The web-client product page now says what main actually does after #2990 and #3072, and points at the new residual item for what remains.
