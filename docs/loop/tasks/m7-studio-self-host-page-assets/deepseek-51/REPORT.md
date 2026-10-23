# REPORT: m7-studio-self-host-page-assets, deepseek-51 (page-load verification first)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item `m7-studio-self-host-page-assets` (residual of #2868; M7 #2035). Base: `7bbaadcdd`. `References #2868` until the item's issue exists (PM sync).

## What did not work, first

- **The claim could have been asserted again.** The existing `verify_studio_worker.mjs` checks `index.html` by substring (`http://`/`https://` anywhere), which does not model load references and would also flag ordinary navigation links; the item asked for a page-load check, not a token grep. The new scanner parses load attributes specifically and keeps `<a href>` out of the bar.
- **The demo page is not a three-link fix.** It carries 1,647 lines and 263 distinct utility classes, including the custom `cyber-*` Tailwind palette with opacity variants, gradients, animations and Font Awesome icon classes. Self-hosting it faithfully means a bespoke CSS subset plus inline SVG for icons — outside the size this item implies. Per the ORDER's stop rule I stopped and posted a decision card rather than silently deleting or hand-waving the page (#2028 and #2868, 2026-10-23).
- **The item has no issue yet.** It was created by #3072 and awaits the PM roadmap sync, so the design note and this record live on the parent #2868 until then.

## What changed

| File | Change |
|---|---|
| `web/studio/verify_studio_page.mjs` | page-load scanner: load attrs, loading `<link rel>` tokens (incl. `preconnect`/`dns-prefetch`), `<base href>`, inline `<style>` `url()`/`@import` (both forms), fail-closed inline-script external literal scan, one level of on-origin linked stylesheets; fixture self-test with eleven planted refs; `index.html` strict, `--strict-router` for the demo page |
| `web/README.md` | invocation line for the new check |
| `docs/products/web-client.md` | page-load status row: `index.html` machine-checked on-origin; demo page tracked as the residual with its decision card |
| `docs/labs/m7-studio-self-host-page-assets/README.md` | lab record: verified scanner, the demo-page decision card, the stack-bound-tokenizer plan, the GitHub-API position, D36 status |
| this folder | ORDER (committed first) and this REPORT |

## Measurements

- `node web/studio/verify_studio_page.mjs` → `PAGE-LOAD CHECKS PASSED`; `index.html` clean; demo page's three CDNs reported as legacy residual.
- `node web/studio/verify_studio_page.mjs --strict-router` → exit 1 with exactly `cdn.tailwindcss.com`, the cdnjs Font Awesome stylesheet and the Google Fonts stylesheet — the failing-before evidence.
- Scanner self-test: eleven planted external refs detected exactly (multi-token `rel`, `preconnect`, `dns-prefetch`, external `<base>`, `//host` script sources, `@import` strings, inline-script literals); `<a href>` navigation ignored; prose fragments starting `//` are not URL literals.

## Checks run at the head

`node web/studio/verify_studio_page.mjs` (pass), `node … --strict-router` (expected fail), `loop report --check`, `loop pr-check --local`; full gates/CI after push.

## Skills used

- `solution-integrity`: modelled the actual load semantics instead of a substring grep; made the scanner prove itself with planted references; stopped on the demo page rather than pretending 263 utility classes were three links.
- `scope-guard`: web/**, docs/products/** and the item's lab folder only; no Rust change, no archive/deletion, no Pages deploy change, no bar.

## Still open

- **Demo page decision** (PM/owner): retire (needs archive/scope), declare legacy-exempt for this item (recommended), or fund a follow-up rewrite. `--strict-router` is ready for the day the page is fixed.
- **Stack-bound tokenizer artifact + worker load path/label** (next unit in this item; design in the lab record §3).
- Runtime page-load trap owed; tokenizer artifact; demo-page decision; no compute, no bar.

## Plain summary

- Landed the machine-checked page-load claim for the Studio page and the failing-before evidence for the demo page's three CDNs, stopped on the demo page's real cost with a decision card, and recorded the remaining tokenizer plan. `index.html` is verified clean; the demo page is a tracked residual.