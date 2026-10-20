# #2336 — File D29 compiled-module compliance + R3 cadence sync (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T15:02:51Z
- **Updated:** 2026-10-13T15:04:21Z
- **Closed:** 2026-10-13T15:03:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2336
- **Merged:** 2026-10-13T15:03:57Z
- **Merge commit:** 674db38bca472e9da04e08fda9f797957715f411
- **Base / Head:** main / UOR-Foundation:kimi/d29-compiled-module-compliance
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line: no model line — governance · count 0/3 · headline 37/232 → 37/232 (unchanged)

## File D29 compiled-module compliance + R3 cadence sync

Docs-only. Record = the changed documents themselves; the newest-first `docs/integration/current-state.md` entry is the change log.
Decision: **NOT YET PROMOTED** — no model line, no compute, no count reset, headline unchanged (reply 37/232, v5 memory 20/40).

### What changed and why

- **D29 appended to `docs/integration/DECISIONS.md`** (after D28, in the file's decision style): banned mechanisms — softmax, softplus, sigmoid, tanh, exp-family squashing; matmul/dense GEMM including tensor-core `mma`/`wmma` contractions — must not **exist** in any NVRTC-compiled module, linked vendor call path (cuBLAS/cuDNN), or executed host fallback; **the compiled-module surface is the compliance boundary** (presence counts, not only execution). NVRTC remains the CUDA compiler; `CUDA_STACK_SOURCE` (`crates/uor-r4-training/src/cuda_stack_kernels.rs`) is split so the rung-0 module compiles only compliant kernels, with historical kernels excluded from compiled modules and preserved as source — nothing deleted. In scope: the rung-0 trunk, the Line-B native learner (`uor-r4-core/src/native_geometric`), D11's integer-engine exp-table masses, and tooling. Enforcement in progress on DeepSeek's harness line (`check_no_matmul.py` into prlint/CI; the semantic path checker extending to CUDA raw-string kernel sources). Rung-0 execution is gated on the split or an explicit owner waiver, with the rung-0 pre-registration sheet's source-identity pin requiring the executed module to be free of banned kernels. Claude builds the replacements/split; Codex reviews and re-qualifies; the PM routes.
- **Cadence sync** of the remaining live "90 minutes" status texts: `docs/labs/audit.md` and `docs/labs/session-goal.md` now state the five-minute live-tier heartbeat (Rust `uor-harness heartbeat` as the only implementation, #2327). The pm-rules R3 text was already synced in #2331 and R10 already carries the disk cadence from #2334, so nothing further existed there.
- **`docs/integration/current-state.md`** newest-first entry: D29 with the elimination inventory (NVRTC only under `crates/uor-r4-training`; rung-0 executed path already clean; backlog = module split + host arms + Line-B 21 sites + D11 6 sites), gate gaps (`check_no_matmul` not in CI yet; semantic checker misses CUDA raw strings), assignments, rung-0 additionally gated on the split, and rung-0b §4 FIT closed (Codex final 6148141029) with Claude owning the build.
- `docs/harness/claims.json` trimmed of store-merge artifacts (my four stale claims released; claude's and codex's entries kept byte-identical; codex's already-released `rung0-count-reference` not resurrected).

### Verification

- [x] Checks run at this exact head (`d44b21966`) — commands and outcomes below
- [x] `python3 scripts/harness/cli.py check` — **0 errors** with this change set, and 0 errors with it stashed (the earlier pre-existing `receipts.parse` error was fixed upstream by #2332)
- [x] `python3 scripts/harness/cli.py lint check --body-file <this body>` — 0 errors (deliberate `record_path` warn, as in every governance PR)
- [x] `python3 scripts/check_claim_wording.py` — PASS
- [x] Docs only — no `crates/` changes, nothing under `local/` committed

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

