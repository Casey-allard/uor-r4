# REPORT: m1-a8-registration-and-runbook-staging, claude-70 (Stage 1 erratum, procedure and records)

Lab: claude. Item: [m1-a8-registration-and-runbook-staging](../../../../roadmap/items/m1-a8-registration-and-runbook-staging.md). The item status is unchanged. Nothing is released and no compute is cleared.

## What did not work, first

1. **The plan template is not here.** The `QualificationPlan` at `fd409fcdf` pins four files that exist nowhere on main or in the HF store:
   - `anchor_manifest`, a `TrainAnchorManifest` whose CSV must be `8c1b80e9…`;
   - `accepted_initlock_disposition`;
   - `residual_fit`;
   - `g_init_comparison_or_waiver`.

   `validate_plan` reads each by pin, so each must be a real file, and I did not invent them. They are named for the staging author and the PM on #2842 (6379986044), and the ORDER is amended to match.
2. **The claude-64 source pin cannot be reproduced.** `compiled.source_sha256` hashes ptxas resource reports that contain `Compile time = N ms`, so it changes on every build. Erratum 2 replaces it; this sheet edit needs a targeted, erratum-only, non-author FIT re-confirmation (PM ruling 6373425177).
3. **Pod 1 needed a second attempt.** Attempt 1 stopped on a key-path bug in my operator script, which read `compiled_identity.*` and `scope`. Its outputs are kept at `results/claude/a8-stage1-pod1-20261024/`. The rerun (`…-r2/`) passed 97 checks with 0 failures.
4. **The first records builder handed back without writing anything.** It reached its context limit while reading. A second, pre-briefed builder built the records.
5. **Records not yet validated by the pinned binary.** The records were checked against the validator's per-name rules (lines 1096–1300), but not yet against the parts of the loader that need the full plan (the snapshot against the initial checkpoint, and the request against the panel). That happens at `check-plan` on pod 2.

## Cancel probe

At the base, step 1.1 pinned `compiled_identity.source_sha256 == 177f1b3d…`, and no plan evidence was committed. The task stands.

## What changed

| File | Change |
|---|---|
| `docs/labs/m1-a8-registration-and-runbook-staging/README.md` | **Step 1.1 (erratum 2).** Pins the deterministic `compiled.*` digests (geometry, kernel, decomposition kernel, ordinary and decomposition cubins, baseline-reuse PTX and cubin), the driver digest, and the saved run binary (`executable_sha256 fb4493253d3e…`, `source_sha256 81651081…`, restored from `results/claude/a8-stage1-pod1-20261024-r2/a8s1-pod1.tar`, never rebuilt). It says why and gives the evidence. |
| | **Step 1.3.** Casey's template-approval procedure for the 300 s resource-freshness rule (#2842 6379923206). |
| `docs/harness/prep/m1-a8-registration-and-runbook-staging-2026-10-22.md` | Marks the claude-64 source pin as superseded by erratum 2. The `TEST FITNESS: FIT` block is untouched. |
| `docs/labs/m1-a8-registration-and-runbook-staging/evidence/inherited/` | The 16 `uor-r4.attention-audit-inherited-command/1` records (8 per arm), plus a README. All pins are absolute pod paths under `/workspace/a8s1` or the driver data directory. |
| this folder | ORDER (committed first; done-when 3 amended) and this REPORT |

## Measurements

- **Pod 1** (`vast-57462239`, 2×RTX 5090, $2.171/h, about 17 min, about $0.62).
  - At `fd409fcdf`, every deterministic digest equals the 23 Oct measurement.
  - In both arms: `qualify-relation` exits 0, the negatives exit 2 with the registered reasons, and `top1` is 133 with 256 occupied cells.
  - `attention-dev-b3` gives `P_succ` 1,351/3,847 and full 4,244/14,287.
  - Results are at `results/claude/a8-stage1-pod1-20261024-r2/`, with `SHA256SUMS`; the tar's sha256 is `01ac2067…`.
- **Records.**
  - All 16 parse with `jq`.
  - 158/158 file pins match their files in the tar, or `data-sha256.txt` for driver data.
  - Every record has `executable.sha256 == execution.executable_sha256 == fb4493253d3e…`, `sheet_sha256 == fcb770e6…` (`FIT_SHEET_SHA256`), `reviewed_source_commit == fd409fcdf`, and `base_source == ef9d30999700be11e103f96590e9e3b83b6807c7` (`ATTENTION_BASE_SOURCE` at the pinned commit).
  - The per-name rules hold: argv shapes, exit codes, the exact negative stderr reasons, and the probes' `request_file_sha256` and `snapshot_file_sha256` input pins.

## Checks run at the head

`loop report --check`, `loop pr-check --local`, `git diff --check`.

## Skills used

- **`solution-integrity`:** the four missing plan inputs are named, not invented. The erratum replaces a pin that cannot be reproduced with pins that can.
- **`scope-guard`:** only the item's two scoped paths and this folder.

## Still open

- The four plan inputs (antigravity and the PM).
- After that:
  1. the plan template and its hash;
  2. the targeted FIT re-confirmation of erratum 2;
  3. the template sign-offs (a reviewer lab, Kimi, Casey);
  4. pod 2.

## Plain summary

The A8 qualification's identity check now pins values that can actually be reproduced. The sixteen pre-run observation records are built from the GPU run and committed. The plan itself still needs four documents that nobody has produced yet.
