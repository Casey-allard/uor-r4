# ORDER: preserve normal process exits in the bounded native host, kimi-76

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `35f55d334e654047821b4fe66226e8e4ea3d3069`, tree `40f4e1084d8c09945a2e851789c6ab0c6024a742`.

## Finding and evidence

The exact-head default desktop suite failed `scoped_file_and_git_commands_enforce_granted_folder_and_refuse_dangling_symlinks`: the unborn repository reported `HEAD (timeout)` instead of `HEAD (unborn)`. The complete control binary took 5.55 seconds although the declared process deadline is 30 seconds; the focused test then passed in 0.94 seconds. This is the same diagnostic disclosed by packaged Changes acceptance, not a measured 30-second Git stall.

`run_bounded_process` breaks when both output readers disconnect. If the child has closed its pipes but `try_wait` cannot yet observe its normal exit, the post-loop path sets `timed_out` and terminates that child. A normal process exit therefore has a scheduling-dependent false timeout.

## Selected repair

Add a deterministic regression whose child closes stdout/stderr, remains alive briefly, and exits successfully before the declared deadline. It must fail on the current helper by reporting a timeout and unsuccessful status.

When readers disconnect without a prior kill, wait for the child only until the original deadline before using the existing timeout/termination path. Do not remove or extend the deadline. Preserve output caps, descendant-pipe timeouts, process-group termination, exact host approvals, and all other callers.

## Files in scope

- `src-tauri/src/lib.rs`, only `run_bounded_process` and its unit tests.
- Measured claim updates in the parent `REPORT.md`, `web/studio/README.md`, `docs/products/web-client.md`, `docs/roadmap/items/studio-typed-shell-upgrade.md`, `docs/research/notebook/m4-serving.md`, and `docs/integration/current-state.md` only where they currently disclose this timeout as open.
- This repair folder and task-owned local evidence.

No new dependency, engine/model/worker/tokenizer/`studio_api.js` change, second UI, real credential/browser profile/external test endpoint, production publication, signed-release claim, or Phase B work.

## Done when

- The new regression fails before the helper change and passes afterward; the existing genuine-timeout, descendant-pipe, and output-cap tests still pass.
- Default and GUI desktop suites, the packaged app contract and real packaged Changes workflow, and the affected product/legacy/native/repository checks pass at the exact committed head.
- The previous open-timeout claims are corrected only to the measured scope. A fresh `uor-delivery-reviewer` approves the exact head before protected PR delivery; actual merge and fresh-main verification are confirmed.

## Stop and report

Stop before weakening a timeout/assertion, hiding a remaining failure, adding a retry to disguise the race, changing unrelated process behavior, or routing around protected checks. If the deterministic regression does not distinguish old and repaired behavior, do not deliver the helper change.
