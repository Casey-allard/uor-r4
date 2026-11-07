# ORDER: render safe rich replies in the typed Studio

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500), continuing the phased design after merged composer PR #3817. Base: `d0d7048f9066ae4786d98769121400edd48f98e3`.

## Goal

Add a small typed rich-message renderer to the development library and use it in `AgentComposer`. Present common Markdown as useful Preact structure without an HTML-string injection path. Preserve the original message string for copy/insert/download/retry and leave the live plain-JS page unchanged.

## Cancel probe

The typed transcript currently renders every message as one plain paragraph. The legacy live transcript also uses `textContent`; the offline vendor Markdown helper produces HTML strings and is not a safe typed boundary. If common structures cannot be rendered without raw-HTML execution or unsafe links, stop rather than adding `dangerouslySetInnerHTML` or a dependency.

## Files in scope

- New `web/studio/src/components/RichMessage.tsx` and `RichMessage.test.tsx`.
- `web/studio/src/components/AgentComposer.tsx` and `AgentComposer.test.tsx` for integration only.
- `web/studio/src/main.ts`, `web/studio/src/studio.css`, `web/studio/README.md`.
- This task folder and `docs/superpowers/plans/2026-11-07-studio-safe-rich-replies.md`.
- `docs/products/web-client.md` and `docs/research/notebook/m4-serving.md` for scoped evidence.

No other paths are authorized. In particular, do not change `web/studio/index.html`, legacy JavaScript, the offline vendor shim, engine, worker, tokenizer, `studio_api.js`, model artifacts, publisher, deployment, `.github/**`, `Cargo.lock`, real browser data, other labs' work, or owner-controlled/generated files. Add no dependency.

## Done when

- Red-first tests prove the missing renderer, then cover paragraphs/headings, unordered/ordered lists, blockquotes, fenced code with optional language, inline code, strong text, and HTTP(S) links with `noopener noreferrer`.
- Raw HTML stays literal text. `script`, event-handler, `javascript:`, `data:`, `vbscript:`, protocol-relative, and malformed-link cases create no executable element or navigation target. Rendering performs no fetch or model request. Unmatched/unclosed Markdown remains safe and long/Unicode content is not truncated.
- `AgentComposer` renders assistant rich content while copy/insert/save/retry receive the exact original string and existing source/privacy/readiness behavior remains green.
- Focused and complete Studio tests, typecheck/build, plain-JS tests, legacy static/runtime checks, claim/report/local-policy checks, and full gates pass. The PR uses `References #3500`; #3500 stays open.
- Product notes and notebook distinguish typed-component evidence from live-page parity. No production publish, browser-data migration, cutover, useful-chat, M7/M8, production-readiness, or accessibility-certification claim.
- Fresh exact-head review, protected merge, and fresh-main verification are required before cleanup.

## Stop and report conditions

Stop before changing files outside this scope, adding an unsafe rendering path, or weakening a test/verifier. Stop after this declared component unit and its protected delivery; do not mount the library, migrate real data, invoke the publisher, or start another UI unit in this session.
