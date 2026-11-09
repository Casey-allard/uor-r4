# ORDER: close exact-head acceptance review findings, kimi-78

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `af86d7ca07d4ab97936f6254b35f210f10da560d`. Prior fresh reviewer agent-61 returned CHANGES; PR #3879 stays unqueued.

## Findings and decision review

The static handler's case-sensitive `.ts`/`.tsx`/`.map` guard is bypassable on the default case-insensitive macOS filesystem by uppercase or mixed-case requests that canonicalize to source files. Add the red-first mixed-case regressions against renamed and default roots, reject those extensions ASCII-case-insensitively, and preserve valid assets. Add the missing regular-file `--studio-root` regression.

The MCP fixture must prove that the intended 15 ms tool call actually arrived with its complete authorized body. Fixture writes must tolerate deliberate client disconnects, and thread join must expose unexpected panics. Do not change production MCP behavior or the original 15 ms/25 ms hanging-call assertions.

Replace the false broad “later src/lib.rs differences are test-only” statement: the static-source guard and canonical-root validation are production changes after the historical `42bc46f81` package. Historical GUI acceptance does not cover them. Keep `759da8585` dirty evidence, `7e3afedb7` clean host/dist evidence, and `af86d7ca0` documentation-only intermediate state distinct; current final head and full-gate logs must be recorded explicitly. Do not conflate `pr-check --local` with `loop gates --full`.

## Files in scope

- `src-tauri/src/lib.rs` for the case-insensitive static extension guard only; `src-tauri/src/main.rs` and `src-tauri/tests/desktop_client_controls.rs` for root regressions; `src-tauri/tests/mcp_protocol.rs` for fixture proof/disconnect handling.
- `web/studio/README.md` and `web/studio/verify_studio_loopback.mjs` only if acceptance identity needs clarification.
- `docs/products/{desktop-client,web-client}.md`, `docs/integration/current-state.md`, `docs/roadmap/items/studio-typed-shell-upgrade.md`, `docs/research/notebook/m4-serving.md`, and `docs/loop/tasks/studio-typed-shell-upgrade/kimi-{74,75,77,78}/**` for exact claim correction and evidence.
- Ignored evidence under `local/kimi/studio-final-acceptance/`.

No model/engine/worker/tokenizer/R4 API/neutral-workbench implementation, real endpoint/profile/credential, production publish, signed release, or useful-chat claim. Do not include predecessor ORDER files in the separate interactive-terminal branch.

## Done when

- Uppercase/mixed-case `.TS`, `.Ts`, `.TSX`, `.Tsx`, `.MAP` and lowercase equivalents fail on every root; valid assets still load, and regular-file explicit roots fail before serving.
- The MCP test asserts the exact complete authorized hanging `tools/call`; expected disconnects do not panic the fixture, unexpected fixture panics fail the test, and all original limits/refusals remain.
- Complete default/GUI suites, typed/plain-JS checks, exact-head loopback, full `loop gates --full`, GitHub checks, fresh exact-head review, protected merge and fresh-main verification pass.
- Records identify production versus historical package scope, all retained evidence identities, and remaining full agent/runtime/API work honestly.

## Stop and report

Stop before weakening a security assertion, accepting a timeout without proving the intended request arrived, hiding a fixture panic, or claiming that a previous dirty or docs-only head is final exact-head evidence.