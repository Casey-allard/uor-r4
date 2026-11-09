# REPORT: complete browser and packaged Studio acceptance, kimi-71

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Original base: `496ae315224e1faa328ef2c7032d0581408fcf17`; `origin/main` was merged through `973bc075050aef59e753159c9ba30a3086b6c7f1`. Packaged acceptance is committed as `052429eee27d0e319e04c3e30b94f1268846fe57`; final delivery HEAD/tree and current merge base are in the exact-head identity logs rather than embedded self-referentially here.

## What did not work

- The existing legacy `/studio/` remains published at gh-pages `eb832431a`. This unit did not publish typed production, sign a release, finish full Phase A, establish useful chat/M7/M8 or accessibility certification, or begin Phase B. Production publication and final packaged launch are the separate following unit.
- Static-WASM and the `uor-chat` native runner still do not provide native Git/terminal actions. Those unsupported states remain explicit; the separate desktop loopback host supplies full browser host actions, and packaged Tauri uses native IPC. First migration is single-tab only and `ui.v1` is archival only.
- Exact `42` is the R4 geometric arithmetic tool, not useful model generation. Packaged reopen restores conversation/editor state but shows `Artifact: unselected`, so post-reopen model readiness is not claimed. Multiple older untitled fixture tabs remain; the final packaged sequence saved only `untitled-6.md`.
- The first post-repair typecheck found that the new invoke-only adapter test's dynamic JavaScript import lacked the same declaration suppression as its static import. The test-only annotation was added; focused 7/7, full typecheck, Vite build, and 113/113 plain-JS checks then passed.
- One package-contract invocation used the non-executable script directly and failed with permission denied; rerunning through `bash` passed. One combined legacy command had a shell quoting error before any check ran and was rerun correctly. A Vitest run overlapped the Tauri preparation test's `npm ci`, causing missing `node_modules/vitest/suppress-warnings.cjs` worker errors. The locked install was restored and the complete product/legacy/loopback sequence passed serially. These are instrument/setup failures, not hidden product negatives.
- No real browser profile, credential, external test endpoint, new dependency, production publisher, model training, GPU run, or owner-path edit was used. The first exact-head default desktop run hit the known intermittent `remote_auth_timeout_and_size_caps_fail_explicitly` transport error (`small initialize`, loopback URL). Its focused 1/1 rerun and complete default/GUI reruns passed; the initial failure and all three passing logs are preserved in the final-head evidence rather than hidden.

## What changed

- `run_bounded_process` now waits for normal child exit after pipe closure only until the existing deadline instead of immediately labeling it a timeout. Actual deadline expiry, output caps, descendant pipes, and termination retain their boundaries; all six focused process tests pass.
- The existing desktop loopback host accepts `--studio-root <dir>` and serves reviewed Vite dist with existing token, Host, Origin, canonical-path, grant, and dist-only raw-TypeScript checks. `--enable-terminal` and `--approve-command <cwd> <command>` reuse host-owned approval. Page RPC only reports current state and cannot enable or approve itself; exact cwd/command, 60-second, one-shot behavior remains intact.
- `native_choose_project_folder` returns only the new CLI grant, `native_git_diff` dispatches through loopback RPC, and invoke-only loopback file/Git actions no longer require a chat-only `Channel`. The added adapter regression passes; its dynamic plain-JavaScript import has the same TypeScript suppression as the existing static import.
- `verify_studio_loopback.mjs` drives real isolated Chromium against the reviewed dist and host. It verifies workspace selection/import, two-session create/resume, pasted/current-file attachments, actual UOR `42`, non-timeout Git, one approved `pwd`, a separately cancelled second command, page self-enable refusal, exact edit/save bytes, and restored transcript/attachment/editor state. The typed runtime stages the offline vendor shim required by that workflow.
- `docs/loop/tasks/studio-typed-shell-upgrade/kimi-71/PACKAGED.md` and `local/kimi/studio-final-acceptance/packaged-evidence/manifest.json` bind the native packaged observations to executable SHA-256 `c8ddeb4039c1721139c990a06c3d188620596cdbb3cdd654308fa3074f53c8c6`, raw AX, screenshots, and artifact hashes.

## Measurements

- Loopback Chromium: `LOOPBACK STUDIO ACCEPTANCE PASSED`; 15 initial requests and 12 reopened requests, with zero external/raw-TypeScript/runtime errors in each. Exact `Loopback native edit/save acceptance.\n` was saved and restored. Reopen had an empty root grant and `Artifact: unselected`; explicit folder reselection restored access. `local/kimi/studio-final-acceptance/loopback-acceptance.json` and `.png` retain the report and final view.
- Packaged Tauri: the fresh `19+23` session returned exact `42` with its geometric trace. `untitled-6.md` saved exactly `Packaged final native edit/save acceptance.\n` (44 bytes; SHA-256 `24fb7f9e465b859e8f5f4f922d7b14ee2b510f2d5f58f27a7b454cc99391dfb7`) and remained attached after relaunch.
- Packaged Changes returned `HEAD (unborn) · 2 changed files` and the complete intent-to-add `note.txt` diff with no `HEAD (timeout)` or `diff timed out`. Separate native session enablement and exact one-run lowercase `pwd` approval returned the task-owned workspace path. A second run was not treated as approved.
- Quit/reopen restored transcript, attachment, and exact editor payload. It reset source/artifact, folder, and terminal enablement. Reselecting only the task folder preserved the content and did not establish model readiness. All app processes owned by this acceptance were quit.

## Checks run at the head

Evidence under `local/kimi/studio-final-acceptance/052429eee/` records the following passing checks at `052429eee27d0e319e04c3e30b94f1268846fe57`:

- **436/436** Vitest across 49 files with zero unhandled errors; **113/113** plain-JS Node tests; typecheck and Vite build.
- All four legacy checks: strict page, runtime page-load with zero external requests, desktop client, and worker with bit-identical save/restore.
- Complete loopback browser acceptance as above. Typed runtime **4/4**, Tauri runtime **7/7**, and typed dist allowlist verification.
- Default desktop Rust **8/2/20/7/13/8** (library/main/controls/MCP/review/sidecar); GUI **10/2/24/8/13/9**. The packaged-resource contract passed with the measured executable SHA.
- Full typed preparation and unsigned package rebuild passed. The rebuilt executable is SHA-256 `87e16b57f3d1d169f306c1458f6ca0abe8602911a23c10c8270af7a9925ba564`, with its own package-contract pass. Native-runner CDP passed with 13 initial requests, zero external/raw-TypeScript/runtime errors, two independent sessions and exact `42`, restored transcript, and all four migration cases. Harness build passed.

Report/local-PR/full-gate results and the final records-commit matrix are recorded in the exact-head evidence; they are not inferred from this implementation-head list. `loop pr-check --local` does not establish GitHub required checks before a PR exists; no such check is claimed from a local N/A.

## Skills used

Executing the bounded plan, red-first process/host repair, systematic packaged and test-instrument diagnosis, test-driven adapter correction, macOS accessibility verification, and verification before completion. A fresh exact-head `uor-delivery-reviewer` is required before protected delivery.

## Still open

Records commit, final exact-head matrix, fresh `uor-delivery-reviewer`, PR/GitHub checks, protected queue/merge, fresh-main verification, and cleanup are **NOT_RUN** at this edit. Final evidence will be under `local/kimi/studio-final-acceptance/<HEAD>/`. Typed production publication and final packaged launch remain the separate following unit; full Phase A and Phase B remain incomplete.

## Plain summary

One reviewed mounted Studio runtime now has real end-to-end browser and packaged-desktop evidence for the requested agent workflows. Normal Git process exit is no longer mislabeled as a timeout, host approval remains outside page control, and reopen restores content while resetting permissions. The remaining work is protected delivery of this acceptance unit, followed by typed production publication and final launch; useful model conversation and Phase B are not established.
