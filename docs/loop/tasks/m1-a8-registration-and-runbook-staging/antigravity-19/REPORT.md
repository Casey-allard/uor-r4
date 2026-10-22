# REPORT: m1-a8-registration-and-runbook-staging, antigravity-19

Lab: antigravity. Order: [ORDER.md](ORDER.md). Item: [m1-a8-registration-and-runbook-staging](../../../../roadmap/items/m1-a8-registration-and-runbook-staging.md).

## What did not work, first

- **`NOW.md` stale reference on `#2785`:** `docs/integration/NOW.md` lines 23–25 still refer to `#2785` as tracking sheet restoration and qualification binding; `gh pr view 2785 --json state,mergedAt,mergeCommit` confirmed `#2785` merged on `2026-10-20T16:48:41Z` (`c6eb0526566b976ca8cc281c867a1d0c4c3f0b40`), restoring `docs/harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md` byte-exact at SHA-256 `fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450` and recording the qualification binding in `docs/labs/admission-preserving-qualification-2026-10-19/README.md`.
- **Running `verify_a8_address_cells_dose.py --self-test` on later `main` (`767f653b2`):** The consumer verifier rejects `driver_source_sha256 = b13d4920d37a49dd0b1c2305ceaf406e51d5baadd058572cbf481f534bdf9b24 != 0f8ce4061996acdcce3ad4ae01d9e267f280703a77536eb6ecc24bfdc6e6df1d` on later `main` because post-`#2782` commits modified `crates/uor-r4-discrete/src/`. This confirms the source-identity rule in `NOW.md` lines 32–34 (`fd409fcdf3bc43d607e768bae6f8cf403aa75901`, not later `main`), which Step 1.1 of the runbook enforces via `git checkout --detach fd409fcdf3bc43d607e768bae6f8cf403aa75901`.
- **Two separate D40 release chains:** In `fd409fcdf3bc43d607e768bae6f8cf403aa75901` (`#2782`), finite source qualification (`attention-audit-qualification-owner-release` -> `uor-r4.attention-audit-source-qualification-admission/1`, scope `m1-attention-active-audit-finite-source-qualification-v1`) and full-dose whole-interval rejection (`attention-audit-owner-release` -> `uor-r4.attention-audit-owner-admission/1`, scope `m1-attention-active-whole-interval-rejection-v1-{relation,frozen-zero}-seed1-attempts0..16404`) use distinct plans, schemas, and confirmation phrases. Staging only one release command would leave the two-stage workflow incomplete; the runbook stages both in execution order.
- **Notebook scope boundary:** `docs/research/notebook/m1-discrete-attention.md` is outside `m1-a8-registration-and-runbook-staging.md`'s `scope:` (`[docs/harness/prep/m1-a8-registration-and-runbook-staging*, docs/labs/m1-a8-registration-and-runbook-staging/**]`), so the six-column `m1-discrete-attention` research notebook row is recorded in [README.md](../../../../labs/m1-a8-registration-and-runbook-staging/README.md).

## What changed

- Registered the one-page prospective finite-qualification pre-registration sheet [m1-a8-registration-and-runbook-staging-2026-10-22.md](../../../../harness/prep/m1-a8-registration-and-runbook-staging-2026-10-22.md) with all 8 required `prep.rs::REQUIRED_BLOCKS` (`Bar`, `Base artifact`, `Data split`, `Steps`, `Stop`, `Cost`, `Minimum test power`, `TEST FITNESS: FIT`), pinning source `fd409fcdf3bc43d607e768bae6f8cf403aa75901` (`#2782`), protocol `train61-synthetic1-interval64-two-arm-throughput-v1`, scope `m1-attention-active-audit-finite-source-qualification-v1`, TRAIN anchor `8c1b80e9b15e4e2bcc765037afa7800b785418be3e20b8cc80ce13ee2e32b3c1`, the 5 mandatory qualification stage checks (`canonical_initlock_c2`, `required_negatives`, `g_occ`, `sensitivity`, `audited_restore_epoch_cursor`), and the `R <= 0.15` readiness bar from `fcb770e6…` §4 and `docs/labs/admission-preserving-qualification-2026-10-19/README.md`.
- Staged the two-stage qualification and audited-dose runbook, executor sequence, six-column research notebook row, and exact D40 owner-release commands for Casey in [README.md](../../../../labs/m1-a8-registration-and-runbook-staging/README.md).
- Updated [m1-a8-registration-and-runbook-staging.md](../../../../roadmap/items/m1-a8-registration-and-runbook-staging.md) to `status: done` and re-rendered [README.md](../../../../roadmap/README.md).

## Measurements

- **`docs/harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md` SHA-256:** `fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450` (`17,745` B, byte-identical before and after).
- **`docs/harness/prep/m1-a8-registration-and-runbook-staging-2026-10-22.md` prep check:** `8/8` `REQUIRED_BLOCKS` present, `TEST FITNESS: FIT`.
- **Source-pin refusal check (`verify_a8_address_cells_dose.py` on `767f653b2`):** refuses `driver_source_sha256 = b13d4920… != 0f8ce406…` as required by the `fd409fcdf` source lock (`$0.00` GPU spend).

## Checks run at the head

- `shasum -a 256 docs/harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md`: `fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450`
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo test --locked -p uor-r4-harness --lib prep::tests::the_real_sheets_are_readable`: PASS
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo run --locked -p xtask -- roadmap check`: PASS
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo run --locked -p xtask -- roadmap render --check`: PASS
- `uor-harness check`: `0 error`
- `uor-harness loop gates --full`: `7/7 GREEN`
- `uor-harness loop report --check`: `PASS`
- `uor-harness loop pr-check --local`: `PASS`

## Skills used

- `uor-research-architect` (`.agents/skills/uor-research-architect/SKILL.md`)
- `scope-guard` (`/Users/casey.allard/.gemini/config/skills/scope-guard/SKILL.md`)
- `solution-integrity` (`/Users/casey.allard/.gemini/config/skills/solution-integrity/SKILL.md`)

## Still open

- GPU execution of Stage 1 (`attention-audit-qualify`) and Stage 2 (`attention-audit`) remains HELD pending owner cost/scope approval, PM dispatch, and Casey's personal D40 releases (`attention-audit-qualification-owner-release` and `attention-audit-owner-release`) with `UOR_LAB` unset.

## Plain summary

This task registers the one-page finite-qualification pre-registration sheet ([m1-a8-registration-and-runbook-staging-2026-10-22.md](../../../../harness/prep/m1-a8-registration-and-runbook-staging-2026-10-22.md)) with `TEST FITNESS: FIT` while keeping the admission-preserving dose sheet (`fcb770e6…`) byte-identical, and stages the complete two-stage finite-qualification + audited-dose runbook, executor commands, and exact D40 owner-release commands for Casey in [README.md](../../../../labs/m1-a8-registration-and-runbook-staging/README.md).

closes #2832
