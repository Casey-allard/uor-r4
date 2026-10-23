# REPORT: infra-loop-harness, kimi-27 (PM flip: m7-studio-connect-wasm-engine to done; residual item)

Lab: kimi. Order: [ORDER.md](ORDER.md). Item #2904 (Setup row: 2). Base: `259f9b9b2`. `Closes #2868` (via the post-merge `roadmap sync --apply`, which closes the issue of a done item; the residual is carried by `m7-studio-self-host-page-assets`). `References #2904`.

## What did not work, first

- **Repair 2 (counted CHANGES at `bfb5ff38f`: deepseek 6366395951, claude 6366462558) — scope only.** Both reviewers verified the content and blocked on one finding: `loop pr-check --local` fails `scope` because the two roadmap item files sit outside the m7 item's globs. Verified at head before fixing: `pr-check --local` FAILs `scope` on exactly those two paths; #2904's scope covers `docs/roadmap/items/*.md` but not `docs/products/**`; `infra-loop-harness` is a setup item (`loopreview.rs:306`) whose ORDER needs a valid `Setup row:`. Fix, per both reviewers' prescriptions: ORDER/REPORT re-homed to this folder with Item #2904 and `Setup row: 2` (ORDER committed first and alone, same pattern as kimi-22/#3054), and the `docs/products/web-client.md` de-stale moved out of this PR into a separate small m7-scope PR (kimi-28) — the new item's done-when already carries "`docs/products/web-client.md` updated to match", so no work is lost.
- **Repair 1 (claude's counted CHANGES at `1ea096a8e`).** Two blocking findings, both verified at base before fixing: (1) the new item's premise was stale — #2990 (merged `9baf8452c`, after #2978) had already replaced the main page's CDN assets with offline shims, so `index.html` has zero CDN URLs at base; the real residuals are page-load *verification* of that claim, the router demo page's three CDN loads, the tokenizer stand-in, and a recorded position on the user-initiated `api.github.com` calls. (2) The resolution note said "#2868 stays open" while the REPORT/body carry `Closes #2868`. Both fixed: the resolution note now says the residual moved to the new item and #2868 closes via sync; the new item is rewritten around the four real residuals citing #2990.
- **The item's done-when was met on main but its status and text said otherwise.** PR #2978 (merged `24f9d8ab6`, verified) delivered the M7-criterion-3 evidence — client-side worker, truthful capability reporting, `remoteRequests=0`, `7/7 PASS` WASM audit — while the item still read `status: ready` with stale present-tense claims about the retired router engine. DeepSeek flagged the mismatch on #2028 (2026-10-23T13:12:54Z) and declined to edit the PM-owned file.
- **First render/check failed on section names.** The roadmap format requires literal `## What is missing` and `## Done when` sections; the initial edit had renamed/replaced them. Fixed by restoring both headers with the resolution recorded inside them.
- **The residual was real and would have been orphaned.** The author of #2978 deliberately kept the item `ready` over page-level CDN UI assets and the byte-mod stand-in tokenizer. The item's own text always treated those as an unregistered bar, and the registered done-when's terms are all about the model worker path — so the PM call is: done-when met, item flips to `done`, and the residual becomes a new item (`m7-studio-self-host-page-assets`) rather than being lost or silently re-scoped into this one.

## What changed

| File | Change |
|---|---|
| `docs/roadmap/items/m7-studio-connect-wasm-engine.md` | `status: done`; stale "still loads the retired router engine" text replaced with a resolution note carrying the #2978 evidence and the residual pointer |
| `docs/roadmap/items/m7-studio-self-host-page-assets.md` | new item: page-load trap verification, router demo page CDN loads, stack-bound tokenizer artifact, recorded `api.github.com` position, `web-client.md` update |
| `docs/roadmap/README.md` | re-rendered (`roadmap render`; 91 items, done 28) |
| this folder | ORDER (committed first and alone) and this REPORT, re-homed from `m7-studio-connect-wasm-engine/kimi-26` |

## Measurements

- None new: this is a PM record change resting on merged, verified evidence (PR #2978: `remoteRequests=0`, `trapSelfTestHits=6`, WASM audit `7/7 PASS` mul=0/div=0/float=0; `node web/studio/verify_studio_worker.mjs` ALL PASSED). Spot-checked on fresh main: zero `uor_r4_wasm_router` references under `web/`; the worker imports `pkg/uor_r4_integer.js` and calls `loadStack`; `index.html` loads `web/studio/assets/css/uor_offline_fonts.css` and `uor_offline_vendor.js` (#2990).

## Checks run at the head

`cargo xtask roadmap render` + `roadmap check` (91 items OK), `uor-harness loop report --check`, `loop pr-check --local`, `loop gates --full`.

## Skills used

- `scope-guard`: roadmap item files, the rendered README and this task folder only; no web code touched, no deploy change (D36 stays); the `docs/products/**` edit moved to its own m7-scope PR rather than widening this umbrella.
- `solution-integrity`: the flip rests on the done-when's own terms and merged evidence, not on the item's looser prose.

## Still open

- #2868 will be closed by the next `roadmap sync --apply` after this merges (sync closes the issue of a done item and creates the new item's issue — dry-run confirmed: create `m7-studio-self-host-page-assets`, update + close #2868, milestone #2035 checklist change); the residual work is carried by the new item.
- The `docs/products/web-client.md` de-stale lands separately as kimi-28 (m7 scope), referenced from the new item's done-when.
- GitHub Pages deploy stays disabled (D36).

## Plain summary

- The Studio-to-WASM-engine item is done on its registered terms with merged evidence; the remaining CDN/tokenizer work is now its own roadmap item instead of an implicit blocker. The unit runs under the loop-harness umbrella because roadmap item files are that item's scope.
