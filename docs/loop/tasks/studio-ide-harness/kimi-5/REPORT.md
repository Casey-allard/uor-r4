# REPORT: close PTY lifecycle and evidence findings, kimi-5

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Order: [ORDER.md](ORDER.md). Base: `ec49e32b4d5af2f12cd6b84328584336c6b182ba`, tree `fa2e0af9a80c5994616423ce7b02400c6331ca27`; merge base `ad2f12edd8e3c2396b2fe678dbf16d02d128bb9f`. Fresh reviewer agent-66 returned CHANGES; PR #3890 remains draft and unqueued.

## What did not work

- A late `openTerminal` result could mount after unmount and was not closed. A rejected close suppressed its error and discarded the session id. The initial unmount regression observed no close call; the close-failure regression had no visible `reap failed` message and lost the session.
- Closing also needed to block input immediately and prevent a second open while the first close was pending. The existing input/workspace tests exposed those races during repair. A disposed restored session must not remount or restart polling, but must retain retryable close state and the visible error.
- An empty initial cleanup could mark the panel disposed while its first enable was pending, so the ordinary real-browser open stopped before `openTerminal`. The mounted trace showed `disposed=true` after enable. Empty cleanup now marks disposal only after actual unmount; live-shell close still blocks input immediately. A disposed restored session with the same id cannot remount, but a genuinely new session id can mount.
- The approval-consumption RED run is observed in the session, but was not retained as a separate raw log at that time. A session-observation excerpt is now preserved; it is not described as a complete original raw log.
- `kimi-4` retained earlier source/count roles and a stale final-evidence pointer. The notebook said merged before protected merge. Product records had earlier typed counts/package scope, and the parity matrix called 975eb the final native head. A second local body draft was historical but not labeled. Those roles are corrected rather than treated as new product claims.

## What changed

- Unmount/workspace disposal blocks new input and closes any subsequently returned stale shell. A close in progress also prevents another open in both the guard and button state.
- Close captures the previous session and id, disposes the renderer and pollers immediately, and only clears identity after success. Failure restores the session identity and visible error without restarting the terminal; a later retry can close it. An actual unmount does not restore UI state.
- The two regressions pass alongside all prior terminal cases. The unmount assertion checks the close mock's actual argument object. Readiness waits make the existing workspace-change tests exercise the next shell after close completion rather than race it. Agent-67 later demonstrated the separate pre-effect unmount, rejected-stale-close and cross-workspace restoration races; kimi-6 owns that broader lifecycle repair and supersedes the broader cleanup claim here.
- Report/source evidence now distinguishes ec49 and its final checks from the new repair candidate. The final candidate's HEAD/tree belongs in the PR and durable identity, not an invented self-hash in this report. The notebook says pending protected merge, README uses the actual 445 typed count, desktop/package scope includes the measured native acceptance, and that unit's parity matrix binds da6. `pr-body.md` is explicitly historical; `pr-cowork-body.md` and the live PR are authoritative.
- Existing earlier screenshots/JSON, including `canonical-launch.jpg` (SHA-256 `25e570b3ca2b44c886135527cf98e95cc389502e1e731789ccbcdc929d4ef009`), are copied to the main checkout without overwriting. No unsupported missing-file claim is introduced.

## Measurements

- `closes a shell whose opening completes after the panel unmounts`: **RED no close call → GREEN one close call** for the returned shell.
- `retains the session and visible error when close fails, then permits retry`: **RED missing error/dropped session → GREEN retained error/session and second close succeeds**.
- All terminal component tests **13/13**; all typed tests **445/445 across 52 files**; TypeScript and Vite build PASS.
- Merged `origin/main` at `1e5e84a1d497222099a97e22da024b07759750a8`; that merge changed only Claude's model-training source and its records. Harness source was unchanged, and the packaged native executable was `fff9ec28eec68375dca72c56881eb878586f48abe747280c1a0d3503075e874d` at both candidates. Debug host binaries differ and are identified separately in each `interactive-terminal.json`; distribution identities are recorded separately in the loopback identity. Acceptance was rerun at merged source `da6cbaf4d52d47892584e9d661d9b50d8d6560da`, tree `47b20e8d84a56a8d07038561def2682e550385a9`; the final record-only head/tree belongs in the PR and durable identity rather than an invented self-hash here.
- Complete serial default **11/3/21/4/7/13/8**, GUI **13/3/25/4/8/13/9**, plain-JS **113/113**, Playwright **45/45**, launcher **6/6**, worker/page/static/runtime checks, **7/7** WASM, **11/11** repository gates, Tauri formatting and local policy all passed at the merged source. Policy retains the disclosed scope, filemap and historical commit-order advisories.
- Interactive loopback PTY returned `ANSWER:fixture-answer`, exact fixture cwd, size `23 133`, exit and closed-input refusal, with 15 initial requests and zero external/raw-TypeScript/runtime errors. Combined loopback restored the exact transcript/session identity and native edit/save, returned size `24 80`, and refused a second shell without approval. Native executable SHA-256 `fff9ec28eec68375dca72c56881eb878586f48abe747280c1a0d3503075e874d`; packaged native acceptance returned `NATIVE_FINAL`, exact fixture cwd, `ANSWER:fixture-answer`, `24 80`, exit 0 and a distinct second-shell approval cancelled with no new PTY. App and sidecar were absent after normal quit.
- The first repaired-candidate native launch timed out before its answer was entered. That incomplete instrument was not a product negative; a no-timeout rerun completed. The merged-source native rerun also completed normally. Earlier concurrent GUI, obsolete loopback selector, approval-consumption observation, and first-open mount failures remain preserved.
- This repair changes `TerminalSession.tsx`, its tests, and evidence records only. No model/API/engine/worker/tokenizer, new dependency, or public service is added.

## Checks run at the head

- `CARGO_TARGET_DIR="$PWD/target/cowork-tests" cargo test --manifest-path src-tauri/Cargo.toml` and the same command with `--features gui`: complete default **11/3/21/4/7/13/8**, GUI **13/3/25/4/8/13/9**, PASS serially.
- `npm test --prefix web/studio`: **445/445 across 52 files PASS**; terminal component **13/13** included. TypeScript and Vite build, **113/113** plain-JS, **45/45** Playwright, **6/6** launcher, worker/page/static/runtime, desktop, **7/7** WASM and **11/11** full repository gates PASS.
- `node web/studio/verify_interactive_terminal.mjs` and `node web/studio/verify_studio_loopback.mjs`: PASS at the merged source with exact host/dist identities.
- `UOR_COWORK_NO_OPEN=1 CARGO_TARGET_DIR="$PWD/local/kimi/cowork-parity-target" bash scripts/studio/open-cowork.sh`: PASS, 21.37 MiB unsigned package, followed by the actual native workflow above.
- Evidence is `local/kimi/harness-parity/da6cbaf4d52d47892584e9d661d9b50d8d6560da/`, including `native-acceptance.md` and three native screenshots; combined identity is under the main checkout's `local/kimi/studio-final-acceptance/da6cbaf4d52d47892584e9d661d9b50d8d6560da/`. The active PR body binds the final record-only head/tree. Earlier `ec49e32b…`, `58d14a29…`, and `3e201b987…` evidence is intermediate.

## Skills used

Systematic lifecycle root-cause investigation, red-first unmount/close-failure tests, adversarial review reception, race/readiness analysis, artifact preservation, documentation synchronization, native accessibility acceptance, solution-integrity and scope checks, protected branch completion, and verification before completion.

## Still open

Fresh exact-head review, required GitHub checks, protected queue/merge, and fresh-main verification. Full persistent agent runtime, parallel tasks, project memory, scheduling/Dispatch, browser/computer workflows, live R4 API integration, production publication, signed release, and useful-model capability remain unestablished.

## Plain summary

The terminal now closes late opens, blocks input while closing, and exposes failed-close state with a retry instead of silently losing the session. The merged workbench and terminal have complete browser, loopback and packaged native evidence, with unique artifacts preserved outside the worktree. Protected delivery is still pending; this is not full Cowork parity.
