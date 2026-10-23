# ORDER: infra-loop-harness, kimi-27 (PM flip: m7-studio-connect-wasm-engine to done; residual item m7-studio-self-host-page-assets)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (PM maintenance of item files, as with kimi-2's scopes/closes edits and kimi-22's pointer fix).
Base: `259f9b9b2` (re-homed from `m7-studio-connect-wasm-engine/kimi-26` at repair; origin/main `fb6156b46`).

## Goal

DeepSeek flagged on #2028 (2026-10-23T13:12:54Z) that `m7-studio-connect-wasm-engine` is delivered but still `status: ready`. The item's done-when (M7 criterion 3: Studio runs client-side in a Web Worker with truthful capability reporting and zero remote model calls; the `.wasm` passes `audit_zero_matmul_wasm.py --serving --call-graph --tap` 7/7) is met by the merged and verified PR #2978 (`24f9d8ab6`) with its antigravity-21 repair: `remoteRequests=0`, truthful labels, `7/7 PASS` audit on the built module. This PM unit flips the item to `done` with that evidence, updates its stale "still calls the retired engine" text, and records the residual as a new item so the gap survives this item closing: page-load trap verification of the zero-external-requests claim, the router demo page's three CDN loads, the byte-mod stand-in tokenizer, and a recorded position on the user-initiated `api.github.com` calls. PM call: the done-when's terms are all about the model worker path and are met; the remaining concerns are real but are a separate, unregistered bar (the item text itself says so).

Re-homed under #2904 after two counted CHANGES at `bfb5ff38f` (deepseek 6366395951, claude 6366462558): roadmap item files sit in this umbrella's scope (`docs/roadmap/items/*.md`), not the m7 item's globs. The `docs/products/web-client.md` de-stale moves to a separate small m7-scope PR (kimi-28), since #2904's scope does not cover `docs/products/**`.

## Cancel probe (run first)

Class docs/PM (host-only). At origin/main `fb6156b46`: `docs/roadmap/items/m7-studio-connect-wasm-engine.md` reads `status: ready` while its done-when evidence is merged (PR #2978 at `24f9d8ab6`, verified); `grep -rc "uor_r4_wasm_router" web/` finds zero references; the worker imports `pkg/uor_r4_integer.js` and calls `loadStack`; #2990 (`9baf8452c`) already replaced the main page's CDN assets with offline shims. Task stands.

## Files in scope

- `docs/roadmap/items/m7-studio-connect-wasm-engine.md`
- `docs/roadmap/items/m7-studio-self-host-page-assets.md` (new)
- `docs/roadmap/README.md` (re-rendered; shared file)
- this task folder

## Done when

- `m7-studio-connect-wasm-engine` reads `status: done` with the #2978 evidence; stale present-tense claims corrected; the residual named and carried by the new item file, whose content is built on #2990's base (page-load trap verification; router demo page CDN loads at `web/studio/geometric_prime_router_webapp.html` lines 9/34/37; tokenizer stand-in at `web/studio/uor_model_worker.js:140`; recorded position on the `api.github.com` calls at `web/studio/index.html:2866-2872`).
- `cargo xtask roadmap render` produces the matching `docs/roadmap/README.md` and `cargo xtask roadmap check` passes; `loop report --check`, `loop pr-check --local` (scope PASS under this umbrella), and `loop gates --full` pass.
- #2868 closes via the post-merge `roadmap sync --apply` (sync closes the issue of a done item and creates the new item's issue); the REPORT carries `Closes #2868` and `References #2904`.

## Stop and ask if

- The flip would change a milestone bar or M7's status rather than one item's (it does not; M7 has other open items).
- The owner wants the Pages deploy re-enabled (D36 — explicitly out of scope here).
- Any bar or spend registration appears needed — that is the owner's call (CHARTER §5.1, §5.2), never the lab's.
