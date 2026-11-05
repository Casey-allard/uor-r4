# ORDER: cpu-training-pilot, deepseek-162 (release exact-stop claims)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `e29a7f8eb`.

## Goal

Release the completed `deepseek/copy-exact-stop` and
`deepseek/copy-exact-evidence-manifest` claims, release this temporary release
claim, and record final merge and fresh-main verification receipts.

This is bookkeeping only. It changes no model output, source, serving, CUDA,
float-training or Claude-owned file.

## Cancel probe

Verify PR #3703 merged as `3e8780e61`, PR #3706 merged as `e29a7f8eb`, draft
PR #3705 is closed unmerged, and only the two completed DeepSeek claims remain
to release.

## Files in scope

- `docs/loop/tasks/cpu-training-pilot/deepseek-162/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-159/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-161/REPORT.md`
- `docs/harness/claims.json`

## Done when

1. No DeepSeek exact-stop claim remains; no unrelated claim changes.
2. DeepSeek-159 records the merged exact-head review, merge and fresh-main code
   and evidence verification.
3. DeepSeek-161 records its merge and fresh-main manifest verification.
4. Exact-head review, protected merge and fresh-main verification complete.
