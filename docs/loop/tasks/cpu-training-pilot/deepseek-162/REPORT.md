# REPORT: cpu-training-pilot, deepseek-162

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `e29a7f8eb`.

## Status

Complete. Released the two completed DeepSeek claims and recorded final merge
and fresh-main verification.

## What did not work

- The first evidence repair, draft PR #3705, tried to commit the ignored panel
  corpus. Exact-head review blocked it; the draft was closed unmerged and
  replaced by manifest-only PR #3706.
- Fresh `main` from the initial exact-stop merge exposed that packaging issue;
  the model result itself was unchanged and authoritative.

## What changed

- Removed `deepseek/copy-exact-stop` and
  `deepseek/copy-exact-evidence-manifest` claims.
- Released the temporary `deepseek/copy-exact-release` claim after recording
  the close, leaving no exact-stop claim behind.
- Recorded PR #3703 and #3706 reviews, merges and fresh-main checks in their
  task reports.
- No model, source, serving, CUDA, float-training or Claude-owned file changed.

## Measurements

- PR #3703 reviewed head:
  `8baf7c98d3f17db9fd1760593dca97bba85b1dec`;
- PR #3703 merge:
  `3e8780e6153524f2321e49c01836059545aa1284`;
- PR #3706 reviewed head:
  `b4d7385f1fb054b91b435d0a710e0f122e818079`;
- PR #3706 merge:
  `e29a7f8eb3c11835a90b23f094ac7b7246336d27`;
- draft PR #3705: CLOSED_UNMERGED.

## Checks run at the head

- Fresh `main` contains both merges and no tracked `.u16` corpus.
- Fresh `main` passes the attempt-8 and top-level evidence manifests.
- Fresh `main` passed 286 CPU library tests with 4 expected ignores and all 82
  binary tests.
- Claims JSON parses and no unrelated claim is removed.
- Report, scope, policy and `git diff --check` pass.

## Size

Against base `e29a7f8eb`, this release changes five bookkeeping paths
(**+116 / -14**).

## Skills used

No system-provided Codex skill applies. Project review and delivery procedures
were used.

## Still open

The exact-body stop result remains REJECT; copy-boundary termination and
terminal-token reconstruction remain open.

## Plain summary

The result and repaired evidence are merged. This PR closes their coordination
records without changing the experiment.
