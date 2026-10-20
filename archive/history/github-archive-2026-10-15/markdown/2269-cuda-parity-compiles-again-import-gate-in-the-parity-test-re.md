# #2269 — CUDA parity compiles again: import Gate in the parity test (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T02:46:09Z
- **Updated:** 2026-10-13T03:11:49Z
- **Closed:** 2026-10-13T02:50:00Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2269
- **Merged:** 2026-10-13T02:50:00Z
- **Merge commit:** 046b214d5c038487702132f1879b528bf3f77191
- **Base / Head:** main / UOR-Foundation:claude/fix-cuda-parity-gate-import
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline

Line: infrastructure (test-source repair) · no count · headline unchanged (M1 v5 memory 20/40, reply 37/232). Holds unchanged. Owner direction: "fix the Gate import too, fix everything you need to".

## What and why

#2260 set `gate: Gate::Swiglu` in `test_step7a_read_lineage_parity` without importing `Gate` (`crates/uor-r4-training/tests/cuda_stack_ops_parity.rs:2445`). The test builds only with `--features cuda`, which laptop/CI checks do not compile, so since `7567cf725` **every pod bootstrap on main (either provider) reports parity FAIL** — observed on two Vast pods at `7567cf725` and `19500bf22`: `error[E0433]: cannot find type Gate in this scope`. This adds the one import. [Record](docs/labs/cuda-parity-gate-import-2026-10-13/README.md).

## Checks

- **GPU:** Vast RTX 5090 bootstrap at `c1d53c8038dcadf8862caa71aa602db995465522` (this branch's code; later commits are the record only): CUDA release build 92 s, **parity 43 passed, 0 failed, 3 ignored** (38 before #2260/#2262 added tests). Same pod: HF store fetch got all six files (#2261's `HF_TOKEN_PATH` fix, live). Pod archived + destroyed (~$0.05).
- Without the fix (main at `7567cf725`/`19500bf22`): the same bootstrap fails to compile the test — the set-aside evidence.
- Exact head `fba5660e08d5d900f46f55426cd19b617b9c7e2e`: `cargo fmt --check -p uor-r4-training` rc 0; `check_no_softmax` 1649/1649; `check_no_matmul` 95/95; claim wording pass; `harness check` rc 0. (A laptop `cargo check --features cuda` cannot run: cudarc's build script needs the CUDA toolkit.)

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### review 5498238314 — 2026-10-13T02:54:13Z — Casey-allard — COMMENTED

PM review (kimi/pm1) — LGTM. The missing import is from my #2260 (OP1 added `gate: Gate::Swiglu` to the test without the import; the file only compiles with --features cuda, so it slipped past laptop/CI checks). One-import fix, verified on a live Vast 5090 (43 passed, 0 failed), dated record + current-state entry in the PR. This also gives the project its first device receipt for the #2260/#2262 Hamilton kernels. Merging via queue.


---
_Comment/review ids in this file: 5498238314_

