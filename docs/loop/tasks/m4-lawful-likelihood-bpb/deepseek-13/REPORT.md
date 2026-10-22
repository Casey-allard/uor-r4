# REPORT: m4-lawful-likelihood-bpb, deepseek-13 (correct the review-record basis)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `28a26fab17ae75fe633f72546865c923d99917d3`. References #2861; item stays `ready`.

## What did not work, first

- **The PR is not doc-only to the gate, so no same-lab record can count.** The final fresh-context review at `28a26fab1` showed why: the PR adds five `docs/loop/tasks/m4-lawful-likelihood-bpb/deepseek-{8,9,10,11,12}/receipts.jsonl` files, `loopprcheck` classes `.jsonl` as `other` (`crates/uor-r4-harness/src/loopprcheck.rs:1238-1256`), so `loopreview::doc_only` is false (`loopreview.rs:748-758`) and `refusal` reaches the same-vendor branch (`loopreview.rs:399-406`). Verified without posting: `uor-harness loop review --pr 2940 --verdict APPROVE --host ... --context fresh --dry-run` exits 1 with "not counted, same vendor: lab deepseek ... (D46)".
- **Three earlier task-folder statements were false.** ORDER-10/REPORT-10 and ORDER-12/REPORT-12 called the fresh-context records "counted" and invoked a "doc-only allowance". They are corrected in place to "advisory" with the gate reason; the historical substance of those repairs is unchanged.
- **The PR body repeated the false allowance sentence.** `loop close --session` regenerates it from the active REPORT, so REPORT-13 states the correct basis and the body will be refreshed on the next close.
- **What remains is not a defect in the design note:** the three deepseek-12 fixes were verified by the final reviewer (`README.md:61` self-contained; `deepseek-11/REPORT.md:16,19` 11/10 with the exact command; `:9` says three), the owner quote is exact against #2028 `2026-10-22T04:17:03Z`, and mechanics are green. The only missing element is an **exact-head APPROVE from another vendor**.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `docs/loop/tasks/m4-lawful-likelihood-bpb/deepseek-10/{ORDER,REPORT}.md` | docs | small | "counted same-lab ... doc-only allowance" corrected to advisory with the gate reason |
| `docs/loop/tasks/m4-lawful-likelihood-bpb/deepseek-12/{ORDER,REPORT}.md` | docs | small | same correction |
| `docs/loop/tasks/m4-lawful-likelihood-bpb/deepseek-13/{ORDER,REPORT}.md` | docs | small | this order and report |

**Size:** task-folder text only (measured in the diff); the design note is unchanged at this head; no source, test, generated file or item-status change.

## Measurements

- **No numeric research result is claimed.** The gate evidence is the dry-run exit and its refusal text quoted above, plus `pr-check --local` PASS, `report --check` PASS, `gates --full` GREEN and CI green at `28a26fab1`.
- The design's arithmetic bounds are unchanged: `MAX_ABS_SCORE < 2^39`, V=4096, `sum(phi) < 2^52`, minimum mass 1, bits bound `<= 2^-12 + 2^-48` per token.

## Checks run at the head

- `UOR_LAB=deepseek uor-harness loop gates --full`: GREEN on five gates.
- `UOR_LAB=deepseek uor-harness loop report --check`: PASS.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: PASS on every rule.
- `uor-harness loop review --pr 2940 --verdict APPROVE --context fresh --dry-run`: refused (same vendor), as documented.

## Skills used

- **solution-integrity**: test the claim with the tool that decides it, and correct the record when the tool refuses it.

## Still open

- **Required before merge: an exact-head APPROVE from another vendor (Claude, Kimi, Antigravity or Codex) at this head.** Requested on #2028.
- **Open choices:** held-out stream name/hash, calibration, family choice.
- **Implementation and registration units** follow once this design record lands.
- **The item stays `ready`** after this PR merges.

## Plain summary

- **The final reviewer found that my own record was wrong about who can approve this PR.** Because the PR carries gate receipts, the gate does not treat it as doc-only, so my fresh-context reviews can advise but not approve; an APPROVE must come from another lab. The dry-run proves it.
- **I corrected the false "counted"/"doc-only" wording** in the earlier task-folder text and stated the correct rule. The design note itself is unchanged: its repairs were verified, and the owner's decision is recorded exactly.
- **The only remaining step is an other-vendor APPROVE at this head**, then merge, verify and clean up.