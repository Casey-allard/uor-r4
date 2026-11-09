# REPORT: close workspace-origin and unmount PTY cleanup failures, kimi-7

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Order: [ORDER.md](ORDER.md). Base: `ed8fdbcaecdde3938b3fb445c90b87ec6740e723`, tree `f1c98b9c866398bde7bd32b926342b1eadadc2cb`; initial merged main `f75725d3071abb9488e058ac158a04d8d093014d`, latest merged main `fbf15e56f3ab1961b30610893392e7a97fd0ab18`. Fresh reviewer agent-68 returned CHANGES at the base head. PR #3890 remains draft and unqueued.

## What did not work

- A workspace change increments the revision before starting the close. Capturing that new revision on rejection made the old workspace appear current and restored its session; a stale retry could restore it again. The earlier test started close before the workspace change and did not cover this ordering.
- A rejected close after unmount had no retry UI and only one close attempt, so a backend-reinserted session could remain without cleanup. The existing unmount regression used only successful closes.
- Pending read/input/resize rejections from an old session could overwrite a new session's status. The shared reading flag also suppressed the new session's first read until the old request settled.
- Active current-state, typed-shell and desktop-client records still pointed to intermediate da6/ec49 evidence, and the integer result was written as 499/499 despite one ignored test. The final-head evidence directory did not include the actual red logs.
- The first merged acceptance rebuild failed when the entire `target/` directory disappeared during Cargo compilation, producing missing fingerprint/dependency-info errors. The directory was confirmed absent; free space had increased to 32 GiB. This is an external build-output cleanup race, not a terminal or Rust test failure. `gui-tests-target-removed-red.log` and the first integer/default passes are preserved; standard target directories were recreated without deleting anything else or editing tests.

## What changed

- The runtime stores the shell's origin workspace revision. Normal close failure restores visible identity only when that origin is still current; workspace-change and stale-retry failures expose cleanup retry without mounting. Restoration no longer depends on the later close-start revision.
- Unmount makes one bounded automatic `closeTerminal` retry. A second failure is logged explicitly and does not create an unlimited retry loop or detached-process-containment guarantee.
- Read ownership is per session. Close clears it so a new shell can read immediately, while an old request's finally cannot clear the new shell's ownership. Read, input, resize and mount error handlers also check session identity before displaying an error.
- Current records name this repair and 21 focused/453 typed tests, and point to the actual c663 acceptance instead of intermediate native sources. Earlier native evidence remains intermediate rather than being relabeled as evidence for this implementation. Integer results say 499 passed, 1 ignored, and actual red logs are preserved beside green evidence.
- Latest main `fbf15e56f3ab1961b30610893392e7a97fd0ab18` changes only documentation and a reply-panel tool, not desktop/frontend/integer/tokenizer/vision inputs. The package was rebuilt after the external target removal and all acceptance was rerun.

## Measurements

- `never restores an old workspace when close and retry both reject`: **RED old `session-a` restored → GREEN no active session, Retry close only, third close succeeds**.
- `retries a rejected close once after unmount`: **RED one close call → GREEN two close calls for the same session**.
- `drops stale read input and resize completions after a new shell opens`: **RED second session's first read not called → GREEN second read called and no stale error displayed**.
- All terminal tests **21/21**, complete typed suite **453/453 across 52 files**, plain-JS **113/113**, Playwright **45/45**, launcher **6/6**, worker/page/static/runtime checks and **7/7** WASM pass. TypeScript and Vite build pass.
- Integer library **499 passed, 1 ignored**. Complete serial default **11/3/21/4/7/13/8** and GUI **13/3/25/4/8/13/9** pass. Repository gates **11/11** and local policy PASS with disclosed historical full-change-surface scope/commit-order and FILEMAP advisories; the narrow manifest exception is not a general waiver, and protected queue acceptance remains outstanding.
- Lifecycle implementation is `dde958250ce8c98a28ac99a7698538eda0d3d872`; report/pointer and merge commits add no application-code changes. Acceptance source is `c66377b8fa77bbd225e52acba6539885dedc0c4a`, tree `16a6f5e6392310b6dae71f213e826226e61b9919`. Final record-only head/tree belongs in the PR and durable identity.
- Interactive loopback at c663 returned `ANSWER:fixture-answer`, exact fixture cwd, `21 133`, exit and closed-input refusal, with 15 initial requests and zero external/raw-TypeScript/runtime errors. Host SHA `bc9a4d1463079881d0c34b092e3cf3355fa682f791f30cadaf2f1f1a70b67b08`, dist `aa571a0c49a94bdf22e3c5d7caeb21bd6378a5167ec78fe18f9b8e9474a1abe7`.
- Combined loopback restored exact transcript/session identity and native edit/save, with 15 initial/12 reopened requests and zero external/raw-TypeScript/runtime errors. Host SHA `d95ed5507d99e0b489b0d088c01b6acfbb99ae74103f13b5652261dad619a88b`, dist `c81c44ea0506da8b6c7f15c4d4a8569347e1bba7478175f7fbe59dbc19440d78`.
- Rebuilt WASM artifact SHA-256 `4a111daf7a1376b37445766c5d30bc297f509dfc355793b67c6234b5d29d9aba`, 806,759 bytes, **7/7** serving audit. Native package is 21.40 MiB with executable SHA-256 `8a545748249b867b7f55d71d49f881780ec8ab3b56db18b45506a7f2b064d83a`. Native first output included `NATIVE_FINAL`; `read` returned `ANSWER:fixture-answer`, exact fixture cwd and `24 80`, then exit 0. A distinct second-shell approval was cancelled with no new PTY. App and sidecar were absent after normal quit.
- Previously missing `58d14a29…` negatives, `interactive-terminal-clean.log`, agent-67 CHANGES and agent-68 CHANGES are preserved in the main checkout and project store. File-based storage cannot retain empty fixture directories; a complete archive was uploaded, downloaded, extracted and compared byte-for-byte. Individual logs also match.

## Checks run at the head

- `cargo test -p uor-r4-integer --lib --features serving-wasm`: **499 passed, 1 ignored**. The unsupported-default and optional all-target attempts are retained separately as instrument/environment negatives, not represented as passing or as terminal failures.
- `CARGO_TARGET_DIR="$PWD/target/cowork-tests" cargo test --manifest-path src-tauri/Cargo.toml` and the same command with `--features gui`: complete default **11/3/21/4/7/13/8**, GUI **13/3/25/4/8/13/9**, PASS serially after the target-directory rebuild.
- `npm test --prefix web/studio`: **453/453 across 52 files PASS**, including terminal **21/21**. TypeScript and Vite build, **113/113** plain-JS, **45/45** Playwright, **6/6** launcher, worker/page/static/runtime, desktop and **7/7** WASM checks PASS.
- `node web/studio/verify_interactive_terminal.mjs` and `node web/studio/verify_studio_loopback.mjs`: PASS at c663 with exact identities above.
- `UOR_COWORK_NO_OPEN=1 CARGO_TARGET_DIR="$PWD/local/kimi/cowork-parity-target" bash scripts/studio/open-cowork.sh`: PASS, 21.40 MiB unsigned package, followed by the actual native workflow above.
- `uor-harness loop gates --full`: **11/11 GREEN**; `uor-harness loop pr-check --local`: PASS with disclosed advisories. Final record-only head gates are separately recorded.
- Evidence is `local/kimi/harness-parity/c66377b8fa77bbd225e52acba6539885dedc0c4a/`, including `native-acceptance.md`, three native screenshots and the actual red logs; combined identity is in the main checkout's `local/kimi/studio-final-acceptance/c66377b8fa77bbd225e52acba6539885dedc0c4a/identity.json`. Earlier `ed8fdbcaec…`, `351172842…`, `c67eaa2a5…`, `da6cbaf4…`, `3e201b987…` and `58d14a29…` evidence is intermediate.

## Skills used

Adversarial review reception, systematic root-cause investigation, red-first test-driven repair, origin-revision/state-ownership analysis, artifact preservation, documentation synchronization, native accessibility acceptance, and verification before completion.

## Still open

Fresh exact-head review, required GitHub checks, protected queue acceptance of the head scope, actual merge, and fresh-main verification. Full persistent runtime, parallel tasks, memory, Dispatch, browser/computer workflows, live R4 API, production publication, signed release and useful-model capability remain unestablished.

## Plain summary

Close recovery now depends on the shell's origin workspace rather than the later close time. Unmount gets one bounded cleanup retry, and stale async completions cannot affect a new session. The repaired lifecycle has complete browser, loopback and packaged native evidence, with negatives preserved. Protected delivery remains pending; this is not full Cowork parity.
