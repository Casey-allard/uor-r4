# #2347 — D29 correction: quarantine the native-context and selected-potential CUDA modules (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T16:39:46Z
- **Updated:** 2026-10-13T16:42:05Z
- **Closed:** 2026-10-13T16:41:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2347
- **Merged:** 2026-10-13T16:41:37Z
- **Merge commit:** 26fb9b92ff5686005179ed6b7b0d32edcabb3027
- **Base / Head:** main / UOR-Foundation:claude/d29-quarantine-2026-10-13
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #2028 · D29 / D30 · correction to #2344

**Line:** compliance · no model line · headlines unchanged. No model compute.

Codex's independent source review of #2344 (#2028 [6149477404](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6149477404), [6149949044](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6149949044)) found learned dense contractions in two modules #2344 called compliant:
- **`selected_potential.cu`:** `selected_utilities` applies 16 freely learned coefficients as a 4×4 contraction.
- **`native_context.cu`:** `score()` plus `readout_credit`/`transition_credit` form a freely learned 4 → 120/33 class-bank projection.

The D29 interlocks are lexical and instruction-level, so they cannot see a contraction written as scalar loops. Both findings are accepted.

## Change
- **Out of the build:** both modules leave `build.rs`, whose `MODULES` list goes 5 → 3: the stack F32/BF16 builds and starling K1.
- **Evidence:** both sources move verbatim to `docs/history/d29-excluded-kernels-2026-10-13/{native_context,selected_potential}_quarantined.cu`.
- **Refusal:** `native_geometric_cuda_kernels::ptx()` and `geometric_selected_potential_cuda::ptx()` return a D29 refusal, so every launch fails loudly.
- **New interlock:** `quarantined_dense_contraction_modules_stay_out_of_the_build`.
- **Records:** a correction section in the D29 record and a current-state entry. Line B is unfinished, not negative (D22). Rung 0 uses neither module.

## Checks at this head
- **D29 interlocks** (host): 4/4 pass.
- `cargo fmt --check` is clean.
- **Ratchets:** `uor-harness ratchet d23` OK at 1647/1647; `ratchet d25` OK at 93/93.
- **CUDA-feature build: NOT RUN.** It needs nvcc, because candle-kernels' build script requires it. The Rust change is two `ptx()` bodies that use error forms already present in the same files. The combined-tree pod qualification (Codex, with #2345) builds it.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6150438538 — 2026-10-13T16:39:53Z — Casey-allard

**Exact-head APPROVE at `7bf2820aa`** (self-review, owner rule 09-30). At this head: D29 interlocks 4/4 (host), fmt clean, `uor-harness ratchet d23` OK 1647/1647, `d25` OK 93/93. CUDA-feature build NOT RUN locally (needs nvcc); stated in the body. @Codex: this is the preserve-and-refuse correction for your two findings; please include it in the combined-tree qualification pin.

### comment 6150442944 — 2026-10-13T16:40:07Z — chatgpt-codex-connector[bot]

To use Codex here, [create an environment for this repo](https://chatgpt.com/codex/cloud/settings/environments).


---
_Comment/review ids in this file: 6150438538, 6150442944_

