# REPORT: cpu-training-pilot, deepseek-161

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `3e8780e61`.

## Status

Complete. The public attempt-8 evidence now omits the generated corpus payload
while still binding its local hash and original run seal. This repairs the
fresh-main checksum failure from deepseek-159 without changing the experiment,
source or REJECT decision.

## What did not work

- Fresh-main verification of PR #3703 passed 286 library tests with 4 ignored
  and all 82 binary tests, but the published manifest named an ignored local
  `tokens.u16` that was absent from Git.
- A preliminary repair attempted to force-add that corpus file. Exact-head
  review correctly rejected it under CHARTER §3.13 and `.gitignore`; the draft
  branch was closed before merge and replaced by this manifest-only repair.

## What changed

- Public attempt-8 and top-level evidence manifests now cover only public
  evidence files.
- `LOCAL-RUN-SEAL.txt` records the original local run root, original manifest
  hash, token filename and token SHA-256, with the command to verify locally.
- The deepseek-159 REPORT links this post-merge repair.
- No generated corpus, model output, source, serving, CUDA, float-training or
  Claude-owned file is committed.

## Measurements

- Local panel token SHA-256:
  `62166941a67bb3cb17f8614fe4efc2698c3b3377a7aa6af88bbde8f745981f15`;
- original local run manifest SHA-256:
  `e936ec3bd7cbb7c42ee97ee6ce46b4ee933a3f7f558f12c72d1062f81bbdd595`;
- public attempt-8 raw manifest SHA-256:
  `f7bef7aa2c61a82f4f382815b8722ea3edc6acc5f04b09246e2b80e4a7ebf2be`;
- public evidence manifest SHA-256:
  `a6634328d115b945ce6a8e14c9c7daaac256d9cbe7ab6cc7da7ac5aa76600ab0`;
- public attempt-8 evidence size: 4,286,140 bytes.

## Checks run at the head

- Attempt-8 and top-level public manifests verify without corpus payloads.
- The original local run manifest verifies from its own root.
- The local token hash matches the original panel manifest.
- No `.u16` or other new corpus payload is added by this PR; the existing
  16,339-byte response mask remains unchanged from the base.
- Report, scope, policy and `git diff --check` pass.
- Fresh exact-head review approved `b4d7385f1` in
  [#issuecomment-6651077623](https://github.com/UOR-Foundation/uor-r4/pull/3706#issuecomment-6651077623).
  PR [#3706](https://github.com/UOR-Foundation/uor-r4/pull/3706) merged as
  `e29a7f8eb3c11835a90b23f094ac7b7246336d27`.
- Fresh `main` passes the attempt-8 and top-level public evidence manifests and
  tracks no `.u16` corpus file.

## Size

Against base `3e8780e61`, this repair changes eight bookkeeping/evidence paths
(**+136 / -10**). No corpus payload is added.

## Skills used

No system-provided Codex skill applies. Project evidence and delivery
procedures were used.

## Still open

The exact-body stop result remains REJECT; copy-boundary termination and
terminal-token reconstruction remain open.

## Plain summary

The corpus stays local. Public evidence now carries the corpus hash and local
run seal while committing only manifests and reports, so the evidence verifies
without violating the repository's no-corpus rule.
