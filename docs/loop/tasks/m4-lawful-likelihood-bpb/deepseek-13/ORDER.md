# ORDER: m4-lawful-likelihood-bpb, deepseek-13 (correct the review-record basis)

Lab: deepseek. Item: [m4-lawful-likelihood-bpb](../../../../roadmap/items/m4-lawful-likelihood-bpb.md) (#2861). Base: `28a26fab17ae75fe633f72546865c923d99917d3`.

## Goal

Correct the review-record basis after the final fresh-context review at `28a26fab1` found that a same-lab record cannot count on this PR: it adds five `receipts.jsonl` files, `crates/uor-r4-harness/src/loopprcheck.rs:1238-1256` classes `.jsonl` as `other`, `loopreview.rs` `doc_only` is therefore false, and `refusal` falls through to the same-vendor branch (D46). The earlier ORDER/REPORT text calling those fresh-context records "counted" and invoking a "doc-only allowance" is false and must be corrected; this PR needs an exact-head APPROVE from another vendor.

## History

- deepseek-8 `0a4549689` added the design note; Claude CHANGES at `e9748f1`; deepseek-9 repaired; a fresh-context review at `effe1e9c1` produced deepseek-10; the owner answered Card 1 and deepseek-11 recorded it; deepseek-12 fixed three residual record defects. None of the fresh-context records above is a counted gate record on this PR.

## Cancel probe (run first)

Class **instrument** (a design record). `UOR_LAB=deepseek uor-harness loop review --pr 2940 --verdict APPROVE --host ... --context fresh --dry-run` exits 1 with "not counted, same vendor: lab deepseek ... (D46)", proving the doc-only allowance does not apply. The task stands: correct the claims, then obtain an other-vendor APPROVE at the head.

## Files in scope

The item's seven globs and this task folder; this repair edits `deepseek-10` and `deepseek-12`'s task-folder text and adds `deepseek-13`.

## Done when

- The false "counted same-lab"/"doc-only allowance" claims in ORDER-10/REPORT-10 and ORDER-12/REPORT-12 are corrected in place to advisory and to why.
- REPORT-13 records the dry-run evidence and states that an other-vendor APPROVE at the exact head is required before merge.
- No design content, code or measurement changes; the item stays `ready`.
- `loop gates --full` GREEN; `report --check` and `pr-check --local` pass.

## Stop and ask if

- A design-content change is needed (none is); a bar, status, constraint or spend change is needed.