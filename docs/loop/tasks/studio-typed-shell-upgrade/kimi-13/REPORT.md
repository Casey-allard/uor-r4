# REPORT: repair MCP call, attachment, and cleanup contracts, kimi-13

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `ddb3b25138defbb98cb39dc3cc1b94a62036df16`. Prior reviewed head: `327da602187fa3d8297d059085ac53bfdd05c3b7`, tree `a57ba2444b83b99dd8474afb523fe826110243e1`. The new exact delivery head/tree and final logs will be recorded on PR #3820 after protected delivery; before push they exist only in local evidence.

## What did not work

- Fresh review comment `6709441123` returned **CHANGES**: Connect could overlap a tool call and latch its busy state; **Attach result** reported success without evidence that the existing API retained the entry; failed previous-session cleanup still installed the replacement; and the notebook overstated use of the real factory.
- The resumed MCP suite reproduced three product failures and one factory-instrument failure with **8/12 passing**: Connect stayed enabled during Call, attachment refusal produced a success toast, failed previous-session cleanup displayed the replacement's tools, and the factory fixture could not create a session.
- The first factory fixture used a `Response` mock incompatible with the actual remote client. This was an instrument defect, not a product negative. A loopback HTTP JSON-RPC fixture replaced it and observed `initialize`, `notifications/initialized`, `tools/list`, and `tools/call` through the real factory.
- The loopback fixture's static `node:http` import then failed typecheck because this project does not enable Node's type declarations. A structural `vi.importActual` type keeps the same runtime module and checks without a dependency or configuration change.
- No real MCP endpoint, credential, browser-data migration, live-page cutover, production publish, useful conversation, model capability, M7/M8, production-readiness, or accessibility-certification result is claimed.

## What changed

- Connect now has both an immediate ref guard and a disabled rendered state while a tool call is active, and recovers when the call settles.
- **Attach result** re-reads `ui.getAttachments()` after `ui.addAttachment` and requires the exact name/kind/content/source entry before showing **Attached** or a success toast. Refusal and thrown errors leave the action available and produce an error toast.
- Failed previous-session cleanup attempts to release the candidate and restores the prior session without replacing its tools. Cleanup uses `disconnect` only when supplied; the direct-browser client's missing hook remains an explicit no-op.
- Regression coverage includes nested/malformed arguments, attachment refusal, failed old-session cleanup, the actual factory through loopback HTTP JSON-RPC, hostile tool names, unmount credential handling, stale candidates, and composed `@` attachment mentions.

No dependency, legacy JavaScript/HTML, `mcp_client.js`, native MCP host, engine, worker, tokenizer, `studio_api.js`, model artifact, publisher, deployment, `.github/**`, `Cargo.lock`, real endpoint/credential/browser data, owner-controlled path, or generated roadmap/issue file was changed. Cargo output stayed under `target/kimi-lexlean`.

## Measurements

- Focused MCP/composition checks: **44/44 tests pass** (`12` MCP panel, `32` StudioApp).
- Complete typed suite: **371/371 tests pass across 44 files**.
- Plain-JS suite: **112/112 tests pass**.
- `npm run typecheck`: PASS with no TypeScript errors.
- `npm run build`: PASS; `dist/studio-foundation.js` **171.46 kB** (gzip **47.77 kB**) and `dist/studio-foundation.css` **5.20 kB** (gzip **1.47 kB**), uncommitted.
- `verify_studio_page.mjs --strict-router`: PASS; both strict pages clean.
- `verify_studio_page_load.mjs --strict-router`: PASS; planted traps detected, actual external request lists empty.
- `verify_desktop_client.mjs`: PASS; binary contract, 32 streamed tokens, 38 invoked commands, zero remote requests.
- `verify_studio_worker.mjs`: PASS; 32 expected tokens, bit-identical save/restore, guarded close/reset, six trap hits, zero remote requests.

## Checks run at the head

Red-first evidence is preserved under `local/kimi/studio-mcp-connections/superpowers/checks/`, including the original `red-mcp-repairs.log` and the resumed focused failure. At the final committed repair state, focused/full Vitest, typecheck, build, plain-JS, all four legacy checks, harness build, `loop report --check`, `loop pr-check --local`, and all 11 `loop gates --full` checks pass. The desktop check streamed 32 tokens with 38 invoked commands and zero remote requests; the worker check passed 32 expected tokens with bit-identical save/restore and zero remote requests. Fresh exact-head review, PR push/update, GitHub checks, ready/queue/merge, and fresh-main verification are **NOT_RUN**. The exact delivery HEAD/tree and final logs are preserved in local evidence and will be recorded on PR #3820 after push. The PR uses `References #3500`; the issue remains open.

## Skills used

Working discipline, scope guard, solution integrity, red-first TDD, systematic async/state diagnosis, modern web form guidance, and verification before completion. A fresh independent exact-head delivery review follows the committed head. No model compute, paid resource, credential, real user data, MCP endpoint, or production publish was used.

## Still open

The typed library remains unmounted. Full real endpoint/native GUI acceptance, browser/Tauri mounting, real session/mobile/accessibility/offline parity, browser-data migration, production publication, and useful model conversation remain outside this unit. #3500 remains open for its full completion contract.

## Plain summary

The typed MCP panel now prevents overlapping Connect/Call actions, reports attachment success only after the exact result is retained, and keeps a working old session if its cleanup fails. The real factory is covered by a local loopback fixture. All declared product checks pass, but no real endpoint or packaged application acceptance is claimed.
