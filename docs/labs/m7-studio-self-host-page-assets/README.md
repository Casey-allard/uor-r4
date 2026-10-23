# Studio page-load assets and stack-bound tokenizer (`m7-studio-self-host-page-assets`)

Residual item of `m7-studio-connect-wasm-engine` (#2868); see the closure note in #3072. Lab: deepseek (`deepseek-51`, draft PR #3073).

## 1. What is verified now (this unit)

`web/studio/verify_studio_page.mjs` is a **static** scan of every load-time reference in the Studio HTML pages — `script[src]`, `img[src/srcset]`, `link[href]` for loading `rel`s, `iframe/source/video/audio/object` load attributes, and CSS `url(...)` — and asserts each resolves on-origin. Navigation links (`<a href>`) are not load requests and are deliberately excluded (the GitHub links on the page are navigation). The scanner has a fixture self-test that plants four external references (including a protocol-relative script and a `url()` in inline CSS) and proves it reports exactly those while ignoring an `<a href>` navigation link.

At the current head:
```text
node web/studio/verify_studio_page.mjs
  index.html: clean (zero page-load external references)
  geometric_prime_router_webapp.html: 3 load references reported as the known legacy residual
node web/studio/verify_studio_page.mjs --strict-router
  FAIL (by design today): cdn.tailwindcss.com, cdnjs Font Awesome, fonts.googleapis.com
```
The strict failure is the **evidence the claim was asserted-only before**: the demo page still loads three CDNs.

**The runtime page-load trap is still owed.** A static scan cannot see requests a script computes at load time (for example `index.html:2872` assembles `api.github.com` from string pieces). The item's done-when asks for "checked by the extended trap verification", so this unit records the static check as landed and the runtime trap as a remaining part (extend `verify_studio_worker.mjs`'s traps to a page-load harness, or add a headless page harness without a new dependency).

## 2. Open decision (PM/owner) — demo page

The demo page is 1,647 lines with **263 distinct utility classes**, including a custom `cyber-*` Tailwind palette with opacity variants, gradients, animations and Font Awesome icon classes; faithful self-hosting is a large bespoke CSS/SVG rewrite, not the "three CDN links" the item text implies. Decision card on #2028 and #2868 asks for one of: (a) owner retires the page (archive step needed, outside this item's scope), (b) PM rules the page-load bar covers `web/studio/index.html` only and the demo page is a documented legacy residual, or (c) a follow-up item funds the demo page rewrite. Recommended: (b).

## 3. Open decision (design) — stack-bound tokenizer

The worker's `UTF-8 byte-mod stand-in` mapping (`uor_model_worker.js:140`) is unbound: it derives token ids from the prompt bytes only. Plan (next unit): a JS tool under `web/**` reads the shipped `pkg/synthetic-stack.lut` header (magic/schema/vocab size), emits `pkg/tokenizer.json` `{stack_sha256, vocab_size, mode: "byte-mod", schema}` plus the derived 256-entry table, and the worker loads it, labelling the capability truthfully ("stack-bound byte-mod tokenizer, vocab N, stack sha256 …") with the old unbound path only as an explicit fallback. A real corpus tokenizer requires a real model (M4 export) and is out of this item. No Rust change (outside scope), no new Python.

## 4. GitHub API calls

`web/studio/index.html:2866-2872` calls are user-initiated button flows, not page-load requests, and are already refused under Tauri/offline. Position recorded here: they fall outside the page-load bar; the static scan checks load references only. If the PM reads the bar as covering user-initiated calls, that is a product call (opt-in/removal), not changed silently.

## 5. Pages deploy

Stays paused (D36); this unit touches assets and verification only.