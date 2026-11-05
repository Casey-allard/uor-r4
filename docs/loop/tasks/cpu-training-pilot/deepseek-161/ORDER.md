# ORDER: cpu-training-pilot, deepseek-161 (manifest-only evidence repair)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `3e8780e61`.

## Goal

Repair the published attempt-8 evidence without committing generated corpus
bytes. The merged manifest names local `tokens.u16`, which is intentionally
ignored. Public evidence must identify and hash the local panel while omitting
the corpus payload.

This changes no experiment output, source, decision or model run.

## Cancel probe

At the base, published evidence fails only because the ignored token file is
absent while the manifest names it. The sealed local token still hashes to
`62166941a67bb3cb17f8614fe4efc2698c3b3377a7aa6af88bbde8f745981f15`.

## Files in scope

- `docs/loop/tasks/cpu-training-pilot/deepseek-161/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-159/REPORT.md`
- `docs/labs/cpu-training-pilot-copy-generated-stop-2026-11-05/evidence/**`
- `docs/labs/cpu-training-pilot-copy-generated-stop-2026-11-05/resource-ledger-exact-stop-confirm.json`
- `docs/harness/claims.json`

## Done when

1. No corpus payload is committed.
2. Attempt-8 and published manifests exclude the local token file and verify
   from the public tree.
3. `LOCAL-RUN-SEAL.txt` records the token hash, original run root and verification
   instruction.
4. REPORT and resource-ledger hashes match regenerated public manifests.
5. Exact-head review, protected merge and fresh-main verification complete.
