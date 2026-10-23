# REPORT: m7-studio-self-host-page-assets, deepseek-61 (self-host the router demo page's three CDN loads)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #3075. Base: `e8d6a624b`. `References #3075`; the item stays `ready` for the real-tokenizer leg.

## What did not work, first

- **The earlier record escalated this as a large bespoke CSS/SVG rewrite** (263 utility classes, custom palette, icons). That estimate was wrong for the CSS part: compiling the page's **own inline `tailwind.config`** with the Tailwind v3.4.17 standalone CLI against the page's source reproduces the exact utilities it uses (20,091 B generated file; the page has 263 class tokens, the CSS 249 unique class selectors), so the "three CDN links" can be self-hosted without a rewrite or an owner retirement decision.
- **Fonts and icons could not be self-hosted for free.** Inter/Fira Code woff2 files are not in the tree and the repo's existing offline pattern (`uor_offline_fonts.css`) uses named fonts plus system fallbacks rather than committed binaries, so the demo page follows that pattern; Font Awesome's font files were not vendored and its 21 icon classes became local unicode `::before` rules. Recorded as caveats, not claimed as pixel parity.
- **The real-tokenizer leg remains open by design.** The stack-bound tokenizer artifact from deepseek-52 binds the byte-mod stand-in to the shipped synthetic stack but does not replace it; a real corpus tokenizer needs an accepted M4 export. This unit therefore stays `References`, not `Closes`.

## What changed

| File | Change |
|---|---|
| `web/studio/geometric_prime_router_webapp.html` | CDN tags and the inline CDN `tailwind.config` script removed; local `assets/css/router_demo_offline.css` link added; Font Awesome classes get local unicode `::before` rules; font stacks get system fallbacks |
| `web/studio/assets/css/router_demo_offline.css` (new) | 20,091 B static Tailwind subset compiled from the page's own inline config (v3.4.17 standalone, MIT attribution header, no external URL; 249 unique class selectors) |
| `web/studio/verify_studio_page.mjs` | scanner `note` updated to reference the runtime trap (the scan is static; no behaviour change) |
| `docs/products/web-client.md` | the two claim rows and the privacy paragraph now state both Studio pages are self-hosted and machine-checked, with the user-initiated GitHub API position kept |
| `docs/labs/m7-studio-self-host-page-assets/README.md` | decision section replaced by the self-hosting resolution (method, results, caveats); runtime/static results updated to zero external |
| this folder | ORDER and REPORT |

## Measurements

| Check | Result |
|---|---|
| `node web/studio/verify_studio_page.mjs --strict-router` | both pages clean (zero page-load external references) |
| `node web/studio/verify_studio_page_load.mjs --strict-router` | `index.html`: 6 requests, 0 external; `geometric_prime_router_webapp.html`: **2 requests, 0 external**; planted-external self-test fires (inline, blob-worker, late) |
| generated CSS | 20,091 B; contains the page's `cyber-*` palette utilities; header keeps Tailwind/MIT attribution without an external URL |
| demo page external URL scan | 0 remaining load references |

## Checks run at the head

Both traps above, `git diff --check`, `uor-harness check` (no new failure), `loop report --check`, `loop pr-check --local`, and `loop gates --full` (see closing line). No Rust change, no binary asset, no model/GPU/bar/hold change. No notebook row: this is a web-instrument verification, not a serving measurement (the `m4-serving` notebook's row rule needs a model-measured number through the exported integer artifact).


## Independent fresh-context verification (owner-directed, 2026-10-23)

Casey's standing direction is not to wait on another lab's review; the unit was verified by an independent fresh-context verifier at `98ed057f9` (before these documentation fixes). It **confirmed the technical claim end-to-end** — both `--strict-router` traps pass (index 6 req/0 ext, demo 2 req/0 ext; planted self-test inline/blob-worker/late), the demo HTML has 0 `http(s)`/protocol-relative load references, the CSS is 20,091 B with 0 `url()`/`@import`, the generated palette matches the removed config values, the 21 icon rules and system fallbacks are present, a planted mutation still makes both traps fail, and the scope is 7 files with no Cargo/registry/model/binary change. It also caught three documentation discrepancies, all fixed in this head: a stale lab-README paragraph still reporting the pre-fix 9-request/8-external result, the CSS size (20,033 → 20,091 B), and the "263 utility classes" wording (263 is the page's class-token count; the CSS has 249 unique class selectors).
## Skills used

- `solution-integrity`: the self-hosting was validated by the existing planted-external trap rather than asserted; the CSS is generated from the page's own configuration, not approximated by hand; the font/icon and tokenizer limits are stated as caveats instead of being implied as parity.
- `scope-guard`: only web assets, the product doc, the lab record and this folder; no page retirement, no archive step, no owner decision, no external asset committed.

## Still open

- **Real corpus tokenizer leg:** the byte-mod stand-in is still bound to the synthetic stack; replacing it needs an accepted M4 export, so the item remains `ready`.
- **Optional visual parity:** vendor Inter/Fira Code woff2 (SIL OFL) and real icon artwork if exact visuals are required; the current unicode/system fallbacks are recorded.
- **User-initiated GitHub API calls** remain outside the page-load bar by the recorded position; if the PM reads the bar as covering them, that is a product call.

## Plain summary

- Self-hosted the router demo page: the three CDN loads are gone, its styles come from a Tailwind subset compiled from its own config, icons use local unicode rules and fonts fall back to system families. Both Studio pages now prove zero external page-load requests in the same traps that previously flagged the demo page. The item stays open only because its real-tokenizer leg needs a real M4 model artifact.