# #2339 — Amend D29: NVRTC banned project-wide, prebuilt modules are the compliant path (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T15:33:37Z
- **Updated:** 2026-10-13T15:54:06Z
- **Closed:** 2026-10-13T15:52:56Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2339
- **Merged:** 2026-10-13T15:52:56Z
- **Merge commit:** 2bc8513d920867391e6476a8cde23e94ddd2ded5
- **Base / Head:** main / UOR-Foundation:kimi/d29-nvrtc-amendment
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line: no model line — governance · count 0/3 · headline 37/232 → 37/232 (unchanged)

## Amend D29: NVRTC banned project-wide, prebuilt modules are the compliant path

Docs-only amendment of D29 (4 hours old, not a historical record) per the owner's 13 Oct ruling. Record = `docs/integration/DECISIONS.md` itself; the newest-first `docs/integration/current-state.md` entry is the change log.
Decision: **NOT YET PROMOTED** — no model line, no compute, no count reset, headline unchanged (reply 37/232, v5 memory 20/40).

### What changed and why

- **D29 edited in place**, with an "**Amended 13 Oct (owner): NVRTC itself banned**" marker: **NVRTC (runtime compilation) is banned project-wide** — no runtime compiler anywhere in the chain. All CUDA kernels compile ahead of time (`nvcc` via `build.rs`) into named PTX/cubin modules, hash-bound into artifacts, and load through the audited prebuilt-module launcher (`launch_prebuilt` pattern: driver-loaded module, no kernel shared with the historical `CUDA_STACK_SOURCE`). The compliance boundary is now the **prebuilt module surface** — what `nvcc` compiles at build time is exactly what the driver may load. The old "NVRTC remains the CUDA compiler" clause is superseded; the D29 core ruling (no banned mechanism — softmax/softplus/sigmoid/tanh/exp-family; matmul/GEMM/mma — in any compiled or loaded module) stands unchanged.
- **Consequence updated:** rung-0 execution waits for Claude's prebuilt-module split (the NVRTC path is retired, not merely trimmed); Codex re-qualifies on the prebuilt module; cudarc's `nvrtc` feature comes out of `uor-r4-training`'s dependency set when the split lands. The rung-0 pre-registration sheet's source-identity pin keeps requiring an executed module free of banned kernels.
- **Current-state entry**, with facts verified on fresh `origin/main` (one correction to the draft received): Claude's redesign has **landed as #2338** (`0497528e2`; the launcher commit `22eaa349a`; rung-0b §4 16/16 finite CUDA controls on RTX 5090 at branch HEAD `235c8a6c2`), so the entry cites the merged state rather than the branch; the remaining piece is the `CUDA_STACK_SOURCE` split — main still compiles it through `cudarc::nvrtc` (`cuda_stack_kernels.rs:3420`). Also recorded: #2337 made `uor-harness` the entry point (the PM scan switched to the Rust binary), and the `~/uor-r4-local` iCloud archival is in progress under R10.
- `docs/harness/claims.json` trimmed of store-merge artifacts (my stale claims released; claude's and codex's entries kept byte-identical; codex's already-released `rung0-count-reference` not resurrected).

### Verification

- [x] Checks run at this exact head (`8bc6c8a9b`) — commands and outcomes below
- [x] `uor-harness check` (the binary, via the entry point) — **0 errors**; warns are the standing ones
- [x] `uor-harness heartbeat` — checked in on the live tier (R10-clean, no checkout write)
- [x] `python3 scripts/check_claim_wording.py` — PASS
- [x] Guard compliance (`harness.entry_point`): no `python3 scripts/harness/cli.py` invocation introduced in any touched document (grep: DECISIONS.md 0; the one current-state occurrence is inside the #2337 record's own text, exempt as log/evidence)
- [x] Docs only — no `crates/` changes, nothing under `local/` committed

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

