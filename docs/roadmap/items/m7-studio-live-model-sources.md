---
id: m7-studio-live-model-sources
title: Publish the fixed Studio with working WASM and selectable model sources
milestone: M7
area: serving
status: done
open_to: lab
needs_gpu: false
notebook: docs/research/notebook/m4-serving.md
depends_on: [m7-studio-connect-wasm-engine, m7-studio-self-host-page-assets]
scope: [web/**, crates/uor-r4-integer/src/wasm_serve.rs, crates/uor-r4-integer/src/wasm_bindgen_exports.rs, docs/products/**, scripts/studio/**, scripts/stage-studio-site.sh, docs/research/notebook/m4-serving.md, docs/loop/tasks/m7-studio-live-model-sources/**, docs/roadmap/items/m7-studio-live-model-sources.md]
done_class: host
---

## Why it matters

*At item start, before the Resolution below.*

The Studio is the project's web client: the M7 milestone requires the model reachable through an API, in the browser (WebAssembly) and in the Studio, with bit-identical tokens. At item start, its source (web/studio) was wired to the integer engine and its page-load harness was clean, but the published page at https://uor-foundation.github.io/uor-r4/studio/ was still the 13 October deploy: it loaded external CDNs and the retired router engine, because the docs-site workflow preserved the old studio/ directory and never rebuilt the wasm bundle. The owner asked on 30 October 2026 to fix that page, make it cleaner, and make the wasm actually run a current model, or offer choosing a model by API or adding other APIs. The owner chose: ship the bundled synthetic demo plus a local-model loader (file or URL) and OpenAI-compatible API providers with an add-API form; do not publish the private HF artifacts; publish live directly once the work is reviewed.

This item is the source-repo change and the built-artifact publish. The Pages workflow itself stays owner-only (.github/** is an owner path); the build/publish helper lives under scripts/studio/ and pushes the staged site/studio/ directory to gh-pages or gh-pages-preview, which the existing workflow preserves across future book deploys. The owner is given the workflow patch as a proposal.

## What is missing

*At item start; the current `/7` state is noted inline and in the Resolution.*

- The published /studio/ was the old deploy: CDN loads, old router engine, stale engine-status banner, and no working wasm bundle at studio/pkg/uor_r4_integer*.
- The wasm bundle was not built by any workflow and web/studio/pkg is gitignored; the build recipe existed only in web/README.md and the verification scripts.
- At item start, the engine parsed uor-r4.lut-stack/1-7 and served /1-/6, refusing a /7 turn-memory stack; the Studio worker accepted only /1. The source page already had a local UORLUT01 file loader and Model Manager, gated to /1, with no model URL or remote-API source. **Current:** `/7` turn memory is served on synthetic fixtures; trained-artifact parity and memory-session restore/export/import remain open until its turn state is saved.
- The page was visually cluttered: ~7k lines with inline CSS, repeated status/provenance blocks, stale "frozen per D36" and "0/8 milestones" claims, and no model-source panel.
- No documented, repeatable publish step existed for agents or the owner; the workflow path filter excluded web/** and crates/uor-r4-integer/**.

## Done when

*Item-start acceptance; the current `/7` state is noted inline and in the Resolution.*

1. **Clean Studio page.** web/studio/index.html and index.css are restructured into a coherent, responsive interface with an honest current-status header (project state, engine state, model source), a clean chat pane and code pane, and no stale D36/0-of-8/provenance clutter. It remains offline-first against other origins: no external or off-origin request at page load (only same-origin bundled assets, including ./models.json, are requested), enforced by web/studio/verify_studio_page.mjs and verify_studio_page_load.mjs, which report external [].
2. **Model sources.** A Model Source panel offers (a) the bundled synthetic demo stack, (b) extend the existing /1-only local sidecar loader to a local .lut/.bin file loader, (c) load a model from a user-provided URL, and (d) an OpenAI-compatible API provider list with add/edit/remove and presets (local uor-r4 API, Ollama, custom), persisted in localStorage, with no keys committed and no automatic external requests. The worker is the only WASM backend and its capability labels are truthful (synthetic vs loaded artifact, unqualified stack, API mode).
3. **Working wasm.** The serving-wasm bundle builds reproducibly with the documented script and the worker accepts the engine's served stack schemas (uor-r4.lut-stack/1-6; /7 parses but the serving engine refuses a turn-memory stack, recorded honestly) plus a user-selected model; verify_studio_worker.mjs and the M7 parity harness still pass on the synthetic fixture and the bundle loads in a browser/headless check. **Current follow-up, not a rewritten criterion:** `/7` turn memory is served on synthetic fixtures; trained-artifact parity and memory-session restore/export/import remain open until its turn state is saved.
4. **Live publish.** The built site/studio (page, JS, pkg, synthetic-stack.lut, tokenizer.json, model manifest) is pushed to gh-pages by scripts/studio/build-and-publish.sh, the live https://uor-foundation.github.io/uor-r4/studio/ serves it, and the page passes the page-load traps and loads the demo model. The owner is given the docs-site.yml patch proposal (path filters + build steps) as a report, not applied.
5. **Docs and records.** docs/products/web-client.md, web/README.md and the studio README state what is now built and what is still missing (no public trained model artifact; M4/M8 remain); one row is added to docs/research/notebook/m4-serving.md; and the PR carries ORDER/REPORT with the build/publish commands and evidence. No binary is committed to the source repo, and no owner path, bar, status, pinned receipt, or private HF artifact is touched.

**Item-start record preserved:** no public trained model artifact existed at item start; M4/M8 remain open. The follow-up release now publicly serves M1 as top-8 next-token prediction, not a new acceptance condition written in this PR.

## Cancel if

The owner withdraws the Pages publish direction, the integer wasm engine stops building for wasm32, or the Studio is superseded by a different M7 web client; then this item is re-scoped or closed with the decision recorded.

## How to start

Read web/README.md, web/studio/, docs/products/web-client.md, docs/labs/m7-studio-connect-wasm-engine/README.md and the owner direction on #2028. Build the bundle with RUSTFLAGS='-C target-feature=-simd128' wasm-pack build crates/uor-r4-integer --target web --no-opt --out-dir ../../web/studio/pkg -- --features serving-wasm, generate the seed=5 synthetic stack and tokenizer, run the three node verification scripts, then work through the done-when. The build/publish helper is scripts/studio/build-and-publish.sh (dry-run by default; --publish-gh-pages requires the owner's live-deploy decision).

## Resolution (30 October 2026, live publish)

Published live at https://uor-foundation.github.io/uor-r4/studio/ by scripts/studio/build-and-publish.sh --target gh-pages --push --yes from an attached main at origin/main (cdaed4330). gh-pages moved bac74f13 -> d8e67c66976f19af86beb0dd1060f05143c56e30 and the Pages build for it reports built. The live page shows the "Pre-alpha · M1 complete · 1 of 8 milestones" status and makes no external/off-origin page-load requests; pkg/uor_r4_integer_bg.wasm is HTTP 200 sha256 d971fe7d… (495,334 B), pkg/synthetic-stack.lut 200 sha256 d60f88d6… (469,252 B), pkg/tokenizer.json 200 sha256 6758318d…, models.json 200, and the old pkg/uor_r4_wasm_router_bg.wasm is 404 (the retired engine is gone). The build/publish scripts, verification harnesses and limits are recorded in PR #3442; the owner-only docs-site.yml patch that would automate the publish remains in DeepSeek's non-repository deploy report, not applied.

**Live follow-up publish (Kimi, owner-requested):** gh-pages advanced by fast-forward `454bb30fc → 233b97fb9780699db3409fe3ce1fa67aa0408e6d` from source base `15c974d13`, with publish tree `eceaab6fd6fa1e5bbadfe90e0197d10dbba2abdf`. The live Studio serves the 700,628-byte wasm bundle with sha256 `b4ab83f124907b4e4e9120999ef7b70356c5e7ab710629aa7136bd1b36b70d42`, while the previous `models.json` remains byte-identical and lists both M1 seeds and both FBS research seeds. Pages build `1339751024` and hosted file/browser checks passed. Development-only paths were removed before the approved publish; no new trained model or native-loopback-only capability is claimed.

## Follow-up release (30 October 2026, owner request)

The owner then asked to release a trained model and serve it, so the model-source layer now serves the **trained M1 model** and the engine's **/7 turn memory**:
- **Released:** the M1-closing 19-table discrete trunk pair (`uor-r4.lut-discrete-trunk/1`, 86,536,132 B / 84,943,328 B, sha256 `04652a3d…`/`1b49e8bd…`) at [`caseyallard/uor-r4-m1-model`](https://huggingface.co/caseyallard/uor-r4-m1-model) with both stack-bound tokenizers; the 4-table Stage-T base exports at [`caseyallard/uor-r4-m1-discrete`](https://huggingface.co/caseyallard/uor-r4-m1-discrete); the research FBS trunk pair at [`caseyallard/uor-r4-fbs-dual-c131`](https://huggingface.co/caseyallard/uor-r4-fbs-dual-c131). Public MIT, anonymous 200, hashes and CORS verified.
- **Served:** the Studio's primary Model Source card loads the M1 seed-1 trunk from the Hugging Face resolve URL only after an explicit click (no external request at page load), verifies its sha256 and tokenizer binding, loads it into the wasm discrete runtime and switches to a **top-8 next-token predictor**; greedy continuation repeats tokens, so the page labels it a language base, not a chat assistant, and refuses generation on the discrete engine. The 86.5 MB artifact is not bundled in the site (remote-only card); page/worker cap is 128 MB.
- **/7 turn memory:** the integer engine serves `uor-r4.lut-stack/7` (units 2-3 of turn-memory-integer): memory tables, caller-marked reply start, the opening/reply keys and moves at the readout; the Studio worker reports `memoryInfo`, marks the reply start and refuses session restore/export/import for a memory session until its turn state is saved. Units 4-5 (DEV parity under a sheet, cost) remain open.
- The owner-only `.github/workflows/docs-site.yml` automation patch is still a proposal in DeepSeek's non-repository deploy report, not applied; the manual publisher now takes `--trained-manifest`.
- **Agent-console redesign (#3447):** the owner asked for a look closer to Claude Code / Codex desktop. The first console shell had a command bar, icon rail, sessions sidebar, transcript with terminal-frame composer, Model providers page, compact context inspector and mono status bar, with a warm graphite + ember palette and a designed light theme, a keyboard-accessible Cmd/Ctrl+K command palette, persisted theme/chrome toggles and a 390px layout with zero horizontal overflow. ORDER/REPORT: `docs/loop/tasks/m7-studio-live-model-sources/deepseek-4/`.
- **Workable AI studio v2 (#3448):** owner feedback then drove a second redesign: one labeled navigation sidebar with exactly four destinations (Agent, Workspace, Models, Guide) and a permanent Sessions section (title/preview/time rows, rename/duplicate/export/delete with in-app confirm); Workspace merges a File System Access folder tree with editor tabs, active-tab content, dirty state and save/save-as; the Agent gains context attachments with chips and token hints, six Build task templates, message actions (copy/insert into editor/save as file/retry) and explicit local-vs-API privacy; Models is four labeled sections (Local engine, Published models, API providers, Research collapsed); Settings, Guide, an eight-group scrollable command palette and a single coherent labeled status bar complete the shell, with a labeled mobile drawer and zero horizontal overflow. The full shortcut map is implemented and advertised only where wired; old element ids remain as hidden compatibility controls, the four verifiers pass unchanged, and the engine/API layer is preserved. ORDER/REPORT: `docs/loop/tasks/m7-studio-live-model-sources/deepseek-5/`.

- **Conversation-first source and native runner (4 November 2026):** Kimi's follow-up replaces the default IDE panel layout with one dark conversation column and automatic bundled synthetic demo while retaining secondary model/workspace/session views and existing composer/API behavior. The `studio-native-runner-v0.1.0` prerelease supplies hash-pinned macOS arm64 native and runtime-only Studio assets; fresh public download/checksum/empty-cache launch checks pass, and a real native browser stream returned exact `19/16/35` usage. GitHub Pages remains static and is not presented as an API host; no public tunnel or insecure HTTPS-to-loopback path is added. This is synthetic protocol evidence, not useful conversation, visual understanding, M7/M8 completion or a new live-Pages claim. ORDER/REPORT: `docs/loop/tasks/m7-studio-live-model-sources/kimi-5/`.

## Rules that apply

- **Delivery (§5.5):** short branch, exact-head fresh-context review, checks at that head, merge queue, verify on fresh main, then delete the branch and worktree. The built site is published from scripts/studio/build-and-publish.sh, dry-run by default; a live publish needs the owner's live-deploy decision.
- **Owner-only boundaries (§5.1):** .github/** is an owner path; the docs-site.yml patch is proposed to the owner, never landed by a lab. No private HF artifact, key, secret or committed binary/corpus enters the repository (§3.13).
- **Serving (§3.6):** the integer wasm engine on client devices is the product and is allowed; research compute stays GPU-only (§3.5). The page must remain offline-first: zero external requests at load, and any remote API call is a user-chosen model source.
- **Rust first (§3.11):** the engine is Rust compiled to wasm; page helpers are plain JS, no new dependency or framework.
- **Evidence (§4):** report the page-load harness results, the worker/parity tests, the built bundle hashes and the live URL; a green CI is not proof of the live page.

## Sources

- Owner direction and answers, [#2028 owner log comment 6534841207](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6534841207) (30 Oct 2026).
- [CHARTER.md](../../../CHARTER.md) §2, §3.6, §3.13, §5.5; [PLAN.md](../../../PLAN.md).
- [docs/products/web-client.md](../../products/web-client.md), [the M7 Studio WASM record](../../labs/m7-studio-connect-wasm-engine/README.md), [the self-hosted assets record](../../labs/m7-studio-self-host-page-assets/README.md).
- [web/README.md](../../../web/README.md), [web/studio/README.md](../../../web/studio/README.md), [.github/workflows/docs-site.yml](../../../.github/workflows/docs-site.yml).
