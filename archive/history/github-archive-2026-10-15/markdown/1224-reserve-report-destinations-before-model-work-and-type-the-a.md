# #1224 — Reserve report destinations before model work and type the answer oracle

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-10T19:11:23Z
- **Updated:** 2026-10-09T13:08:48Z
- **Closed:** 2026-09-10T19:12:28Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1224
- **Merged:** 2026-09-10T19:12:28Z
- **Merge commit:** 85fcdb88b0462061ef24674e2803928b304b151b
- **Base / Head:** main / UOR-Foundation:codex/repair-safeguards
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Safeguard corrections requested by the independent review of PR #1223, which verified the reassertion-versus-truncation repair itself with new actual generations (50/50 original scored prompts, 60/60 new reassertion and follow-up probes, zero correct-to-wrong changes against `e3a906c8` and `e44a9ae4`). No model, artifact pointer or prior evidence changes; retained `90ebd843` is unchanged.

- **Destinations reserved before model work at every caller.** Every driver claims its report directory, and the API check creates its report file, immediately after argument validation and before any model is loaded (version-intent `probe`/`promote`/`compare`/`diagnose`/`fit`/`evaluate`/`preserve`, handoff, word/Rust, lexical, instruction and operation drivers, `native_historical_check`). New opt-in real-driver test `report_output_drivers` (`UOR_DRIVER_DIR`): fifteen driver invocations and the API check against an existing destination with an invalid model path fail on the collision before touching the model, keep prior bytes and write nothing; a fresh destination is claimed before the model read fails. Executed: 15 directory drivers plus the API report file, collisions reported before any model access, prior bytes unchanged.
- **Derived API inputs out of sealed roots.** `native_historical_version api-cases CASES OUT` writes API cases into an exclusively claimed, sealed sibling attempt bound to the source case file and its manifest; `report_output::verify` / `verify DIR` checks a sealed root's complete file set (listed hashes plus no unlisted files), with a unit test for an added and a changed file. The five API case files written beneath four sealed roots of the repair cycle are recorded with hashes in a scope correction beside the preserved manifests. Executed: `verify` rejects the old `fresh-2-3` root for its unlisted `api-cases.json`, accepts `fit3-construction-2`, and accepts the new sibling `api-cases-fresh-2-3` attempt (976 cases bound to the source manifest).
- **Typed answer oracle.** `uor_r4_core::answer_oracle` authors accepted answers once from typed intent (initial/previous/current/abstain) and both evaluators judge membership only; the evaluator no longer accepts `OWNER is in OLDVALUE` for a previous request, and a unit test rejects it. All 154 follow-ups of `90ebd843` in construction and fresh matched exact authored text; re-evaluating the frozen fresh draw under the tightened oracle reproduces 976/976 targets, 154/154 follow-ups, 864/976 exact text and 892/892 preservation cases, so the review's follow-up results stand.
- **Wrapper accounting.** One idempotent finalizer for spawn errors and close, a missing-executable check before spawning, exactly one charge per row (missing-executable probe recorded), and the 22 ms omitted for five failed spawns appended once as a reconciliation row; original receipts unchanged.

Local checks executed: `cargo fmt --check`, claim-wording gate, offline `cargo check --all-targets`, unit tests for the verifier and the oracle, the real-driver destination test, and the fresh-draw re-evaluation (206 s). Full-suite, HTTP/browser/WASM and energy checks are NOT_RUN. Queue CI statuses are compatibility acknowledgements, not tests.

Process: the actual-artifact multi-turn check now precedes the preservation, comparison and final-fresh campaign in candidate qualification. Next model task, before intermediate-version intent: the disclosed live-head same-value reassertion gap (`o in a. o now in b. o in b.` answers `Unknown.` for initial and previous requests on `90ebd843`, `e44a9ae4` and `e3a906c8`), with its own recorded projection and separate selection/fresh evidence.

References #973. The writer-context contribution obligation under #1139 remains separate and open.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

