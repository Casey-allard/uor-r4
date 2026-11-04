# REPORT: conversation-first Studio and working native connection

Lab: kimi. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440). Order: [ORDER.md](ORDER.md). Design: [DESIGN.md](DESIGN.md). Fresh main base: `2833c451`. Source PR [#3651](https://github.com/UOR-Foundation/uor-r4/pull/3651) merged as `a877ef9162d86908873c5edca0f1d0711bb6321a`; its tree `3c4c6470cd3426a56e02fc7299a39e3d7e4c6398` is identical to approved head `f1b752714`. Live `gh-pages`: `eb832431a0db59661c701d5b37168e9cfa3d3ae4`, tree `2d5fe037cffa8ad676e22a2bb1e6dac1149c32fa`.

## What did not work

- The public model remains a synthetic protocol fixture. Its responses are not useful conversation, and no visual understanding, production readiness, M7/M8 completion or trained-model quality is claimed.
- GitHub Pages cannot host an API. HTTPS Pages must not call insecure HTTP loopback, and CORS alone is not a security or connectivity fix. No public tunnel, paid provider, plugin requirement, token persistence or admission/context weakening was added.
- A native browser prompt needing 69 positions against the synthetic stack's 64-position context returned HTTP 413 without truncation. A fresh short session was used for the successful stream.
- Browser inspection caught legacy responsive and ID-level rules hiding Connections/New chat and a dead mobile Menu. Focused regressions were added and the cascade corrected before packaging; no green claim rests only on source inspection.
- Independent exact-head review initially returned CHANGES for hidden native guidance, persisted-WASM boot overwrite, raw connection errors, publish hygiene and retained evidence. These were fixed RED→GREEN, the publish was explicitly allowlisted, and final review returned APPROVE.

## What changed

- `web/studio/index.html`, `index.css`, `assets/js/model_sources.js`, `studio_ui.js` and `studio_api.js`: one system-type dark conversation column, default-only bundled boot with stale-load protection, real offline/model/native actions, visible native guidance, rounded messages and floating composer, compact mobile controls, and actionable missing-runner, insecure-loopback, authentication and unsupported-endpoint errors. Existing keyboard, history, long-paste, attachment, image, session and API paths remain intact.
- New focused Studio/Node regressions cover the empty state, persisted-WASM boot gate, visible guide DOM, connection error call paths, panel-free layout, visible controls and mobile typography. Existing model cards and `models.json` are preserved.
- `scripts/studio/native-runner.sh`: side-effect-free `--check`, pinned HTTPS downloads and SHA-256 verification, safe runtime extraction, Apple silicon validation, and loopback-only `uor-chat --studio-root … --token-stdin`. The token stays in memory and is not printed. The verified archive supplies both synthetic artifacts, so the runner has no mutable Pages dependency.
- [Native runner prerelease](https://github.com/UOR-Foundation/uor-r4/releases/tag/studio-native-runner-v0.1.0): `native-runner.sh`, `uor-chat-arm64-apple-darwin`, runtime-only `studio-local.tar.gz`, and `SHA256SUMS`. No source library, tests, package/build configuration, declarations, nested verifier or README is included.
- The Pages publish used an explicit runtime allowlist, not the older copy-all `scripts/stage-studio-site.sh`. Nine runtime files changed; development-only package configuration and declarations were removed, and the existing catalog stayed byte-identical.

## Measurements

- Full Studio suite: **289/289**, plus typecheck and Vite build. Node API suites: **45/45**. Native loopback: **15/15**. Vision: **9/9**. xtask: **222/222**. All **11/11** harness gates and local PR checks pass, with retained transcripts at final head `f1b752714`.
- Worker: `ALL STUDIO WORKER CHECKS PASSED`, save/restore identity and `remoteRequests: 0`. Static and headless runtime page traps pass with `external: []`; planted external detection fires. WASM audit: **7/7**, zero float/unclassified instructions in the declared scope.
- Offline browser at 1440×900: automatic demo ready, Enter sent a real prompt, 32 completion tokens with live tok/s and finish `length`, and ArrowUp restored the exact prompt. At 390×852 the compact controls and composer remain visible with system typography. Native setup visibly displays the Pages/API and local-runner guidance.
- Native browser: authenticated runner-served same-origin stream returned **16 completion tokens**, exact prompt/completion/total usage **19/16/35**, measured **21,697.4 tok/s**, TTFT **0.366 ms**, elapsed **0.737 ms**, finish `length`. These are one local synthetic run's measurements, not a model benchmark.
- Release assets: binary SHA-256 `f4f36c72baea6b0fb76c72ff511cf8e9615fd328022d3b08937660b28cc05f7e`; Studio archive SHA-256 `bf78084e09d03736d3de5cfbe98040e632cc102946f6396434f12314b9ede04d`; synthetic stack `d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df`; tokenizer `6758318dbcad5925a550103a5c2ee41e35898c30a7daf066da09b0159a41c57b`.
- Fresh public `gh release download` verifies all three `SHA256SUMS` entries. A separate empty-cache launch of the downloaded runner reaches `/healthz` with `ready: true` and the expected stack hash. No token appears in agent logs.

## Checks run at the head

Independent final review returned **APPROVE** with no Critical or Important source blockers. PR #3651 passed required GitHub checks and entered the protected merge queue. Fresh detached `main` at `a877ef916` has the same tree and passes fresh Studio, Node, runner and worker checks. `gh-pages` advanced by fast-forward `233b97fb9 → eb832431a`; the Pages API reports **built** at that commit with `updated_at` `2026-11-04T07:26:52Z` (no build ID invented). Hosted `index.html` matches the candidate, `models.json` is byte-identical at SHA-256 `96e296b74530e8488a6c525c59b4dfbd9f980f519ce39c1c338ed7a5cd12554a`, and hosted WASM is 700,756 bytes at SHA-256 `df368909167f21a3b004bff0d723591f9256118ec91292ec3fd59dd9b7a6a9a0`. Development source, package configuration, test, declaration and README paths return 404. The hosted page generated 32 tokens at 1440×900, displayed all five model cards and working Native setup, and showed the clean conversation layout at 390×852.

## Skills used

Project research/delivery workflow, isolated worktrees, test-driven development, systematic browser debugging, modern web forms/IME/dark-mode/color/accessibility guidance, real built-in-browser desktop/mobile verification, independent exact-head review, protected merge queue, runtime-only publication, public checksum verification, and hosted desktop/mobile checks. No GPU, paid compute, new dependency, new trained model or publisher-code change was used.

## Still open

Only merged-worktree cleanup and the independent review's non-blocking residuals remain: repeated Connections renders may duplicate the guide; loopback errors could distinguish Ollama from `uor-chat`; ready states still offer the offline demo prominently; legacy direct card loads need operation metadata; older historical base references remain; and retained browser evidence is local rather than a GitHub artifact. These do not alter the verified current publish. Useful conversation, trained visual understanding, M7/M8 and production readiness remain unestablished.

## Plain summary

The redesigned Studio is live at https://uor-foundation.github.io/uor-r4/studio/ with a simple dark chat interface and a working synthetic offline demo. The public prerelease gives Apple silicon users a verified local runner for the authenticated native API on the same loopback origin. GitHub Pages remains a static client, not an API host; no useful-chat or visual-understanding claim is made.
