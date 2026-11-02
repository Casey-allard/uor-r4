# REPORT: m7-studio-live-model-sources, deepseek-1 (fix the Studio; live publish is the post-merge step)

Lab: deepseek. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440). Order: [ORDER.md](ORDER.md).

References #3440. The item stays ready in this PR; the live publish below is the post-merge step, and a one-line records PR then sets the item done with the live URL/hashes and closes #3440.

## What did not work, first

- **The live page is still the 13 October deploy.** It loads external CDNs and the retired router engine; docs-site.yml builds only the book and preserves an old studio/ directory, so nothing ever built the integer bundle. This unit ships the build + publish scripts; the live replacement is the deliberate post-merge step (owner: publish live directly).
- **The first records review caught that the item had been marked done before the live publish existed, and that the docs said "published" in the past tense.** The item is now ready, the PR says References, and the docs say the live page is replaced by the post-merge publish. A one-line records PR follows that publish.
- **The first attempt at this PR was assembled with the ORDER commit last and without Lab/Order trailers on every commit.** The history was rebuilt: the ORDER is the first commit, the four content commits re-create the specialist branch trees with identical trees and full trailers (they are new commits, not ancestor merges), and the ORDER/REPORT say so. The FILEMAP row for scripts/studio and the item scope for scripts/stage-studio-site.sh were added.
- **"No network request at page load" was overstated.** The page fetches only the same-origin ./models.json catalog and its bundled assets at boot; the enforced bar is no external/off-origin request (static scan and headless-Chrome runtime trap both report external []). The item, code comment and this REPORT now say that.
- **The worker advertised lut-stack/7 as supported while the engine refuses it.** The worker now reports served schemas /1-6 and the /7 turn-memory refusal (engine_supported false with the engine's reason), and the worker verifier asserts it.
- **The caps disagreed:** the page accepted 256 MiB while the worker refuses above 64 MiB. The page cap is now 64 MiB, and the URL loader checks a declared Content-Length before reading.
- **No public trained artifact exists.** The bundled model is the synthetic seed-5 demo, clearly labelled "synthetic stand-in"; the private HF store is not published. The model-source panel offers the demo, a local file, a user URL, and OpenAI-compatible API providers (add/edit/remove, keys only in localStorage) - the owner's chosen option.
- **The publisher cannot be run from a detached checkout**, and an early guard test accidentally pushed one commit to gh-pages; it was reverted within 18 s (gh-pages back at bac74f13, Pages built from it). The committed script now requires --yes for any push and an attached main at origin/main for live gh-pages.

## What changed

- web/studio: index.html 7,062 -> ~1,200 lines (inline CSS gone), rewritten index.css, new assets/js/model_sources.js and assets/js/studio_api.js, worker schema/model-bytes support with honest /7 reporting, make_tokenizer.mjs schemas, new build_serve_wasm.sh, rewritten verify_studio_worker.mjs and verify_desktop_client.mjs.
- scripts/stage-studio-site.sh and scripts/studio/build-and-publish.sh (dry-run default; --push needs --yes; live needs attached main; Pages 100 MB/file and 1 GB site checks; models.json schema uor-r4.studio-models/1).
- web/studio/README.md, docs/products/web-client.md dated update (future tense for the live replacement), web/README.md, docs/research/notebook/m4-serving.md row, docs/start/FILEMAP.md row.
- Not changed: no owner path, no bar/status change (the item stays ready), no pinned receipt, no private artifact, no committed binary, no new dependency.

## Measurements

- wasm-pack 0.15.0 / Rust 1.97.1 locally (the proposed CI pins 0.13.1): uor_r4_integer_bg.wasm 495,334 B sha256 d971fe7d…7882d7; synthetic-stack.lut 469,252 B sha256 d60f88d6…92df; tokenizer.json 1,206 B.
- Worker verify: 32-token trajectory expected, save/restore bit-identical, allocDelta 0, trap self-test 6, remoteRequests 0, weights/token 15,968.
- Parity: verify_m7_parity worker==wasm on all 8 prompts, remote 0.
- Page load: static clean and headless-Chrome runtime trap external [] (index 10 same-origin requests, router 0 external).
- Wasm audit: 7/7 serving functions, float 0, unclassified 0, integer mul/div allowed (D80/D81).
- Browser throughput on the synthetic demo: about 3.4-3.9k tok/s in two local headless runs (3,368 and 3,855 tok/s; demo only, no trained model, no committed benchmark).
- Publisher dry run: 21 studio files / 1,325,721 B staged, site 240 files / 29,494,962 B; pushed nothing; the previous gh-pages head is recorded for rollback.
- Non-vacuity: the wasm engineer planted five page regressions and the reviewer planted two (external script, altered status chip); each turned the relevant verifier red.

## Checks run at the head

- web/studio/build_serve_wasm.sh (build + worker verify) -> all worker checks passed.
- node web/studio/tools/make_tokenizer.mjs ... --check -> tokenizer matches the stack.
- node web/studio/verify_studio_page.mjs -> PAGE-LOAD CHECKS PASSED.
- node web/studio/verify_studio_page_load.mjs -> RUNTIME PAGE-LOAD TRAP PASSED, external [].
- node web/studio/verify_m7_parity.mjs -> matches on 8 prompts.
- python3 scripts/audit_zero_matmul_wasm.py ... --serving -> 7/7 ok, float 0.
- node web/studio/verify_desktop_client.mjs -> all offline page & desktop client checks passed (pageWired:false).
- headless-Chrome page E2E (local helper, not committed): boot, load bundled demo (469,252 B, lut-stack/1 synthetic stand-in, 15,968 weights/token), generate 32 tokens (3,855 tok/s), add/use an API source with the privacy badge, editor autosave and Use-as-prompt.
- scripts/studio/build-and-publish.sh dry run -> staged tree, DRY RUN nothing pushed.
- `cargo run -q --locked -p uor-r4-claim-wording -- .` and `git diff --check` passed; the local PR check passed.
- The exact-head review record is posted at the head before this PR goes ready; the five required checks carry policy: pass (run <id>).

## Skills used

working-discipline, scope-guard, solution-integrity, uor-research-architect; the three specialist reports in local/lead/studio-audit/.

## Still open

- **Post-merge step:** run scripts/studio/build-and-publish.sh --target gh-pages --push --yes from an attached main at origin/main, verify /studio/ serves the new page and bundle, then a one-line records PR sets item #3440 done, records the live URL/hashes and closes it.
- **Owner-only workflow patch:** the docs-site.yml diff that builds the Studio automatically (path filters + build steps) is in the deploy report for the owner to land; until then the manual publisher is the path.
- **A public trained model:** the page ships a labelled synthetic demo; the M1/FBS trained artifacts stay private until the owner publishes one, and the model manifest has the public-URL slot ready. Browser tok/s on a real trunk is not measured (M4/M8 open).
- **Residuals:** /7 turn-memory stacks are refused by the serving engine; the page's desktop wiring is retained but not wired to the page; browser coverage is macOS Chrome only.

## Plain summary

The Studio is rebuilt clean, wired to the real integer wasm engine, and given a model-source panel with the bundled demo, local files, user URLs and OpenAI-compatible APIs. The build and publish path is scripted and verified end to end; publishing it live is this unit's post-merge step, and the item stays ready until that is verified. No private model artifact was published.
