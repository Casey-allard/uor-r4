# REPORT: deterministic integer R4 image decomposition and Studio input

Lab: kimi. Item: [geometric-image-input](../../../../roadmap/items/geometric-image-input.md) (#3619), dependent on delivered [studio-conversational-composer](../../../../roadmap/items/studio-conversational-composer.md) (#3608). Order: [ORDER.md](ORDER.md). Design: [DESIGN.md](DESIGN.md). Original base `6729e5d507bd39ef400c6515bc29b75f97e682d7`; public base `2789caa9000c28440f881b8ac2b2a4ae7d67edf8`; measurement head `4806f2675c50b799c2c0914f822d7a3e7ad98c81`; earlier review base `35bfa270800e433f4225dbe418d1ecd4be6b28ba`; merged review base `8078271e06138f9e4a185aeb36e3f57f4b068ee0`. The PR is the exact final head/tree and external-delivery identity record.

## What did not work

- After merging `main`, the binary image test intermittently reset on the deliberate 17-byte body against a 16-byte limit and lost the 413 JSON body. The first fixed-memory drain repaired that case but the exact-head reviewer rejected its unbounded pre-authentication byte/time cost. The replacement drains at most `max_body + 1` bytes under a 250 ms total deadline. The small case now asserts the exact JSON message, and a huge declared length with no body returns a bounded JSON 413; the full OpenAI suite is 15/15. No test expectation was weakened.
- Merging current `main` replaced the earlier JSON/prompt contract with canonical `R4IMG001` bytes but left the browser expecting retired fields and the native parser without server issuance. Failing-first fixtures produced three Studio failures and a missing `issue_decomposition` native API. Two test-only configuration-shadowing errors were corrected before accepting the native RED result.
- The first real current-head pixel-A send returned a truthful 413: the merged verbose R4 text required 206 positions of the artifact's 192-position context. A compact-renderer regression failed on the missing `R4IMG1/1x1/1x1/1:` payload before lossless base64 encoding was added.
- The broad OpenAI suite reproduced a timing-dependent busy-test failure when a 40-token stream finished before the second request. The fixture now holds a generation permit explicitly, asserts 429, releases it, and asserts a normal stream. A disconnect can also report the OS's expected `NotConnected`; that is accepted only for client-side shutdown, while permit release must still pass.
- Public-head merging exposed fixture and tooling faults: per-server IDs were required, a direct `sendViaApi` fixture lacked its normal user turn, the API stub did not await content promises, `readTextFiles` was missing from page imports, the merged verifier had only Linux browser paths and raced boot, and synthetic file events did not exercise Chrome's real file-input path. These were localized with CDP evidence and repaired without weakening application assertions.
- The independent exact-head review then found five material defects: the recall guard blocked text-only sends and hid Clear, the browser always used 1×1 despite the 2×2 design, decoded source dimensions were unbounded, the README/lineage was stale, and `uor-r4-vision` appeared twice in workspace members. Failing-first regressions now cover each behavior; the duplicate member also passes `cargo metadata --no-deps`.
- A true 2×2 image on the 64-position synthetic verifier returned a truthful **77/64** context overflow. The successful verifier uses a 1×1 PNG, while larger-grid selection and real 192-token 2×2 behavior are tested separately. No overflow is silently truncated.
- The complete integer run passes its library and binaries, but six unrelated adversarial token-boundary tests fail because external tokenizer/bundle fixtures are absent. Workspace-wide rustfmt also reports many untouched baseline diffs. Neither unrelated set was rewritten or presented as a pass.
- DeepSeek's #3628 merged while this repair was underway. This branch is a follow-up to that public implementation, not a competing claim that Canvas or deterministic decomposition was first built here.
- A real delayed-browser Stop remains NOT_RUN; the automated native-preparation abort regression passes with outcome `stopped`. No semantic image-quality evaluation was run.

## What changed

- DeepSeek's `uor-r4-vision` and native ingestion remain the upstream base: checked integer RGBA patches, two signed Q12 R4 vectors, canonical `R4IMG001` bytes, Blake3 identity, bounded query/octet-stream transport, and strict canonical parsing. No float, matmul, learned captioner, external vision model, or semantic label exists.
- The native server retains at most 64 issued decompositions in a mutex-protected FIFO registry. `/v1/images/decompose` returns `X-UOR-Decomposition-Id` and exposes it through CORS. A flat native `uor_r4_image` part requires exactly eight fields: `type`, `schema`, `canonical_hex`, `source_blake3`, `width`, `height`, `grid`, and `decomposition_id`. The complete content must match the issued record. Unknown, expired, substituted, and non-user parts fail with 400; valid replacement bytes and recomputed hashes do not bypass issuance.
- Accepted images render as compact base64 containing every big-endian signed Q12 value, prefixed by dimensions/grid/patch count. This is lossless and avoids the 206-position verbose-text overflow without truncating features. The source hash remains validated transport identity rather than prompt text.
- The merged async Studio preparation preserves DeepSeek's public structure. `studio_api.js` validates every identity header, including issued ID and exact dimensions/grid, checks canonical magic and bounds, and emits the flat part. Per-image abort checks remain. Recall metadata no longer blocks text-only sends: Clear stays visible while recall is active and resets both in-memory and session metadata. Reattaching is required only to send another image.
- Canvas now selects up to 2 columns and 2 rows independently per axis for larger and odd-dimension images, preserving spatial layout. Decoded sources above 4096 pixels per side are refused before canvas allocation or drawing; this is a post-decode guard, not a decoder-memory bound. The duplicate workspace member was removed.
- Dark mode received a token-only Apple-like polish: black/graphite layers, neutral text, restrained borders, and measured system-blue accents. Separate text-accent tokens fix shipped low-contrast states, including light-theme `#8f3f14`, and attachment previews have one 32px rule. Theme changes do not alter layout, system typography, controls, or engine behavior.
- Existing previews, removal, the exact `R4 patch input — experimental; not visual understanding` label, custom `image_url`, local-WASM refusal, and memory-only image storage are preserved. The merged verifier supports macOS Chrome, waits for actual page/UI readiness, and uses CDP file input.
- `src-tauri/Cargo.lock` records the internal dependency. The typed library remains unmounted. Scoped roadmap sync filters issue operations and milestone checklist/label edits to the selected item. #3500 and #3569 remain open.

## Measurements and checks

- Studio at `81d9ed6ca`: **33 files / 284 Vitest tests PASS**, **41 Node tests PASS**, typecheck and Vite build PASS. Regressions cover text-only reload, Clear, 2×2 grid, huge decoded dimensions, issued content, abort cleanup, and computed contrast in all four reviewed states under both themes. Studio worker/static/runtime-page and desktop checks PASS with zero unexpected external requests; both 1440×900 theme views were inspected.
- Native: **15/15 OpenAI integration tests PASS**, including valid issuance, unknown/expired IDs, valid-content substitution, non-user roles, image count, context overflow, CORS, deterministic busy/disconnect behavior, and byte/deadline-bounded oversized-request JSON errors. Vision: **9/9 PASS**. xtask: **222/222 PASS**.
- Native library and binaries at `81d9ed6ca`: **454 library tests PASS, 1 ignored**, plus **17 binary tests PASS**. The six external-fixture adversarial failures are disclosed above and excluded from this count. `cargo metadata --no-deps` passes with one `uor-r4-vision` member.
- Native audit at `81d9ed6caa6228e18d4e29bd22af323d2a4b821b`: **FULL PASS**, 44 symbol ranges and 138 reachable functions, zero forbidden instructions. `uor-r4-stack` SHA-256 `c698749fb72bf7dcdcec0e3a7656519d4964dfa3b085459d4a1f74f20d3a22c8`.
- WASM audit at the same head: **7/7 PASS**, zero floats or unclassified instructions. Bundle 700,756 bytes, SHA-256 `df368909167f21a3b004bff0d723591f9256118ec91292ec3fd59dd9b7a6a9a0`. The same bundle passes Studio worker checks.
- Scoped rustfmt and `git diff --check` PASS. Final repository gates, local policy, exact-head review, GitHub checks, protected queue, and fresh-main verification remain external delivery steps.

## Real browser and native evidence

Used existing dual-c131 artifact SHA-256 `9e80cc6a894ed50c7dc7d0393e2d3b22f74a7230b52d8c635736202a2afa8694`, schema `uor-r4.lut-stack/5`, context 192, with the established W2 tokenizer. The earlier merged smoke at `4780a06ba` used `uor-chat` SHA-256 `12f43539c53821332833fcb1d703b609247c87e8dfc0c986f715d18a6637191e`.

- The repaired synthetic native browser verifier passes image-only send, exact **31 prompt / 6 completion / 37 total usage**, memory-only reload, and a valid second send with historical image markers. Its latest run measured 279.7 tok/s, TTFT 16.783 ms, elapsed 21.453 ms.
- The earlier merged dual-c131 smoke used Studio's actual file picker and Enter-to-send. Pixel A returned `I.`: **53 prompt / 3 completion / 56 total tokens**, 165.0 tok/s, TTFT 17.318 ms, elapsed 18.178 ms. After reload, with no image restored, a text-only turn returned `I.`: **34 / 3 / 37**, 165.4 tok/s, TTFT 17.560 ms, elapsed 18.143 ms. Clear remained visible and removed the recall notice.
- Earlier scoped measurements remain consistent: changed pixel B returned `I.`, **54 / 3 / 57**, 169.3 tok/s; a true 2×2 image returned `I.`, **118 / 3 / 121**, 93.9 tok/s. These establish the changed-pixel and larger-grid paths, not visual quality.
- The earlier 206/192 and 77/64 failures are retained as negative results. A real delayed-browser Stop remains NOT_RUN. Browser tabs and native servers were stopped.

## Roadmap and delivery notes

- `main` contains DeepSeek's #3628 Studio implementation and #3630 evidence repair. This follow-up adds issuance, schema alignment, substitution/expiry coverage, lifecycle fixes, and current-head verification through `8078271e`. The PR references #3619 and #3628 rather than duplicating their closure; #3500/#3569 also remain open.
- Final roadmap/issue state must be read from fresh `main` before any scoped sync. No unrelated issue edits are allowed.
- No production publish, paid provider, useful-chat, production-readiness, M4/M7 completion, trained multimodal, captioning, recognition, or visual-question-answering claim is made.
- Modern Web Guidance `dark-mode` informed root/meta color-scheme and native accent behavior; `color` informed actual computed contrast checks in both themes; `accessibility` informed decorative preview `alt=""` and dynamic error/live-region semantics. No Canvas-specific guide matched; existing native Canvas/fetch patterns were retained.

## Checks run at the head

Implementation measurements and binary audits use `81d9ed6caa6228e18d4e29bd22af323d2a4b821b`; the final REPORT-only head and tree are recorded on the PR with exact-head review. Repository gates, local policy, GitHub checks, protected queue, and fresh-main verification are external delivery records, not preclaimed passes.

## Skills used

Isolated worktrees, plan execution, test-driven development, systematic debugging, solution integrity, modern web accessibility guidance, integer Rust decomposition, native loopback/OpenAI protocols, plain-JS Canvas composition, and real built-in-browser verification. No GPU, paid compute, production publication, visual-quality evaluation, or model-quality evaluation was used.

## Still open

- Final repository delivery checks and fresh-main verification remain. Browser Stop was not retested. External-fixture adversarial failures and unrelated baseline formatting findings remain outside this unit.
- #3500 and #3569 remain open. A future trained multimodal artifact and registered evaluation are required for visual question answering.

## Plain summary

Native `uor-chat` and Studio now agree on bounded binary R4 image input. The server issues each decomposition and rejects altered or expired references, real 1×1 and 2×2 image turns work with exact usage, text-only chat survives reload, and Clear dismisses recalled metadata. This is deterministic representation and transport, not visual understanding.
