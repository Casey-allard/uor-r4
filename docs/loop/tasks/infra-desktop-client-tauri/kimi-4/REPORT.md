# REPORT: repair the unified harness exact-head review findings

Lab: kimi. Item: `infra-desktop-client-tauri` (#2826). Delivery scope: development verification and candidate delivery only; no useful-chat or production qualification.

## What did not work

- Fresh reviewers returned CHANGES at `32578efaf` (Pages/Git/static/MCP/origin/file/history findings), at `7e6d523e8` (lifecycle, provenance, matrices and record drift), and at `fe67d1e50` (exited-parent/descendant hangs, terminal execution overrides, bundled-pkg/root-symlink verification and unit attribution). The final repairs address each finding rather than only the initial review.
- The bundled engine remains synthetic. A fresh GUI reply is a token sequence, not useful chat. Replaying the prior assistant's literal `<|bos|>` output in the same session returned HTTP 400 for literal special-token injection. No output sanitization or useful-chat relabeling was used to conceal this.
- Quick Look cropped the architecture thumbnail, so it was not accepted as the visual check. The actual SVG rendered completely in the built-in browser. One default Rust invocation named `src-tauri/Cargo.toml` from inside `src-tauri` and ran zero tests; that was an instrument error, discarded rather than counted.
- Native GUI automation encountered placeholder values, an off-screen window and focus-routing failures. Direct selection plus ordinary key events established the exact fixtures. The application source did not periodically rebuild MCP, so no speculative DOM repair was made.
- A 20-minute fixture expired before the corrected input was established; it was restarted with a longer window and then stopped. The 30 GiB free-space goal is not met: targeted removal of this task's regenerable outputs leaves 25 GiB free.

## What changed

- Runtime verification binds every generated local model/tokenizer/audit path and SHA-256 in `models/` and bundled `pkg/` to `models.json`; extra files even with allowed suffixes, modified declared artifacts and root symlinks fail. A trained-manifest-format staging regression using synthetic bytes and provenance negatives pass.
- Process-group termination checks `try_wait` and does nothing after a child has exited/reaped, preventing repeated cleanup from signaling a reused PID. Bounded terminal processes and MCP stdio requests abandon stuck reader joins after direct-child exit and return bounded timeouts rather than hanging on descendant-held pipes. Sidecar cancellation registers through one `entry` operation; a deterministic two-phase probe proves an immediate cancel is not overwritten. Terminal `rg --pre` and Git config/exec-path overrides are refused before execution.
- Git checks cover local and worktree scopes with includes. One process-wide guard serializes repository writes with status/diff and terminal Git execution. Direct, included, worktree and concurrent-mutation filter fixtures pass without executing their marker scripts.
- Static candidates are canonicalized and must remain beneath the canonical Studio root. Native failed MCP listing removes the session, releases its slot and kills/reaps a stdio child. macOS navigation accepts only `tauri://localhost`; HTTP(S) aliases are confined to platforms that use them.
- MCP replacement uses one in-flight token, disables both controls, retains the working session on failure/cancel, disconnects the old session only after replacement success, and releases stale results. Recent files persist `{path, rootPath}` with legacy migration and root labels; concurrent opens re-check identity after awaiting.
- Changes/Terminal deep links are initial history entries rather than Chat-plus-target. A non-ASCII MCP argument test proves the existing UTF-8 byte cap. Documentation now names the qualified-bundle limitation, actual native engine/sidecar/MCP paths and platform-specific asset origins. The corrected product diagram was inspected in a real browser.

## Measurements

### Failing-first and complete checks

Every behavioral repair was observed failing for its intended reason before implementation. The UTF-8 test passed immediately because it covers already-correct behavior. After repair:

- Studio TypeScript typecheck and **314/314** Vitest tests pass.
- JavaScript/Node **66/66** tests pass, including non-ASCII arguments, transactional MCP and stale-result cleanup.
- Python runtime preparation **7/7**, including trained-manifest staging and unexpected-file refusal.
- Default desktop: **6** library, **12** controls, **7** MCP, **12** review, **8** sidecar.
- GUI-feature desktop: **8** library, **16** controls, **8** MCP, **12** review, **9** sidecar.
- Packaged app verifier **2/2**. Bundle: 20.85 MiB; executable SHA-256 `bce98712b4e19d12d4a0ab60f45acaa1fa02855989936da5b8e7edec2295fc40`. Sidecar, stack, tokenizer and catalog retain their previous published pins.

### Real native GUI acceptance

- A fresh `Hi` returned 16 tokens with usage `19/16/35`, `5614.6 tok/s`, TTFT `1.050 ms`, total `2.850 ms`, finish `length`. This is synthetic protocol evidence only.
- After the lifecycle repairs and formatting, the rebuilt app returned another fresh `Hi`: 16 tokens with usage `19/16/35`, `18549.8 tok/s`, TTFT `0.458 ms`, total `0.863 ms`, finish `length`. Normal Quit left app PID 2471 and sidecar PID 2474 absent. After bounded-process/execution/catalog repairs, the final app returned 16 tokens with usage `19/16/35`, `11903.3 tok/s`, TTFT `0.873 ms`, total `1.344 ms`, finish `length`; normal Quit left app PID 54712 and sidecar PID 54744 absent. These are affected-path protocol checks, not model-quality claims.
- Project A saved exactly `Project A edited through repaired native host\n\n`; project B remained exactly `Project B original\n`. The UI said `Saved to disk`; recent entries displayed both roots.
- Remote MCP showed one tool and returned `native remote result`. Stdio showed the exact executable/cwd and returned `stdio result`.
- Cancelling a stdio replacement displayed `Connection approval was cancelled; nothing started. Previous connection is still active.` A second call returned `stdio result` through the preserved session.
- Normal Quit removed app PID 34426, sidecar PID 34429 and stdio PID 39630, verified by `ps` and the running-app list. The remote HTTP fixture was stopped. Forced-kill cleanup is not claimed.
- Targeted cleanup removed the regenerable app target, generated runtime resources, temporary diagram and GUI fixtures. Free space is 25 GiB. Unique evidence and other labs' files were not removed.

## Checks run at the head

- From `web/studio`: `node --test assets/js/*.test.mjs`: 66/66.
- From `web/studio`: `npm run typecheck && npx vitest run`: 314/314.
- From the repository root: `python3 scripts/tests/test_tauri_runtime.py`: 7/7.
- From `src-tauri`: `cargo test --manifest-path Cargo.toml --locked`: default matrix above.
- From `src-tauri`: `cargo test --manifest-path Cargo.toml --locked --features gui`: GUI matrix above.
- From `src-tauri`: `../scripts/studio/prepare-tauri-runtime.sh` and `cargo tauri build --bundles app -- --features gui`: one `.app`.
- `UOR_TAURI_APP=... UOR_DESKTOP_SHA256=bce98712… python3 scripts/tests/test_tauri_app.py`: 2/2.
- `cargo fmt`, `git diff --check`: clean.
- `UOR_LAB=kimi uor-harness loop gates --full`: all **11** currently registered gates GREEN; `uor-harness loop report --check`: PASS.
- `origin/main` at `cf0c16d58db1e7b60c9f576879f84359cfa44337` is merged forward with no conflicts. Lifecycle/evidence repair ORDER is `ac49815f9`; the fully tested implementation head is `b220a02320924ffcbd92c62c707fd1661377ef3b`, and this final evidence is recorded at the exact review head.
- Diff size: 71 files, 9,517 insertions and 391 deletions, including this REPORT, DESIGN and repair ORDERs.

## Skills used

`uor-research-architect`, working-discipline, scope-guard, solution-integrity, systematic debugging and test-driven red/green development; computer-use and kimi-cu for real native dialogs/workflows; the built-in browser for the actual diagram render. Verification-before-completion and fresh exact-head review are required before queueing.

## Still open

Independent exact-head re-review, GitHub checks at the final head, protected merge queue, fresh-main verification, Pages publication and final cleanup remain. #2826 stays open with `References #2826`, not `Closes`. Useful chat, visual understanding, M3/M7/M8, exact sidecar hidden-state resume, signed distribution, forced-kill cleanup, GitHub authentication and 30 GiB free are not claimed.

## Plain summary

The web and desktop harness repairs work at development scope: trained-manifest-format Pages fixtures stage, Git/static paths fail closed, native MCP cleans up, replacements preserve working sessions, recent files retain their projects, and deep links start correctly. The actual app was rebuilt and exercised through file saves, both MCP transports, cancellation and normal exit. Its engine still produces synthetic text and can reject that text on replay, so this delivery does not claim a working conversational model.
