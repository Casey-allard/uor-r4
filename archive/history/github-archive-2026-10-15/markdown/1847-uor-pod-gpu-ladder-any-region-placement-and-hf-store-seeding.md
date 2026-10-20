# #1847 — uor-pod: GPU ladder, any-region placement and HF-store seeding (owner-approved 10-07; References #1750)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T16:36:12Z
- **Updated:** 2026-10-09T13:09:04Z
- **Closed:** 2026-10-07T16:48:05Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1847
- **Merged:** 2026-10-07T16:48:05Z
- **Merge commit:** ea850cff55e40400f291741de48bbbd09ca69876
- **Base / Head:** main / UOR-Foundation:claude/pod-any-region
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Owner-approved 7 October 2026. References #1750.

## What changes
- **GPU ladder.** Without `--gpu`, `up` tries 2 × 5090, then 2 × 4090, then 2 × PRO 6000, then 1 × each in the same order. `--gpu KEY` asks for that type only.
- **Any-region placement.** The volume datacenters come first: EUR-NO-1, EU-RO-1, EUR-IS-1. When `UOR_POD_VOLUME_DCS` is unset, the allowlisted `EXTRA_VOLUME_DCS` follow, ordered by stock. A pod outside EUR-NO-1 is non-canonical.
- **Token copy.** `up` copies `~/.cache/huggingface/token` to `/root/.cache/huggingface/token` over stdin with `umask 077`. This happens only on a real (non-dry) run and only when the file exists. Only "hf token copied" is logged.
- **HF-store seeding.** The bootstrap has a new `--non-canonical` flag. `up` passes it when the pod's cached `canonical` is false, and the preflight checks both argument vectors. If any of `tokenizer.json.tar`, `ft-balp.tar`, `ft-dev.tar`, `step5-inputs.tar`, `MD5SUMS` or `hot/sieve-panel.tar` is missing from `/workspace/uor-r4/data`, the bootstrap installs `huggingface_hub[cli]`, runs `hf download caseyallard/uor-r4-store --repo-type dataset --include 'data/*'` (`UOR_HF_STORE` overrides the store) and checks the files with `md5sum -c`. It only downloads. Every failure is a warning, and the bootstrap continues.
- **Docs.** New section "Any region" in `docs/labs/compute.md`, covering the ladder, placement, the HF store and how to fetch a base, the token, and `runpodctl send`/`receive`. One new bullet under "GPU pods" in `AGENTS.md`. The `uor-pod` usage header now describes the ladder.

## Checks (local, at this head)
- `bash scripts/pod/tests/uor-pod-dryrun.sh` gives `passed 219, failed 0`.
- `bash -n` passes for `scripts/pod/uor-pod` and `scripts/pod/uor-pod-bootstrap.sh`.
- New tests: a non-dry fake `up` with a fake token checks that the ssh copy happened and that `hf_FAKETOKEN123` appears in neither the output nor the recorded calls. A second test checks that `--check-args --non-canonical` is accepted.
- The token test fails without the change. With the token-copy hunk set aside the suite reports `passed 217, failed 2` (the copy assertions), and it passes again after restoring.
- The bootstrap's HF fetch has not been run on a real pod. Only argument acceptance is tested.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6042555477 — 2026-10-07T16:47:09Z — Casey-allard

Exact-head self-review at `0067f53983de56751c72e66170a77bf208c5578d` (owner rule: self-review allowed). Fresh-context review found no token leak / no infinite loop; its blocking finding (rate-cap miss on one rung ended the ladder) and two medium findings (orphan 100 GB volumes on failed create; HF fetch could overwrite existing data) plus F4 messages are fixed in 0067f539. At this head: `bash -n` both scripts clean; `scripts/pod/tests/uor-pod-dryrun.sh` **passed 224, failed 0**; negative gates: ladder rate-cap test fails with the F1 hunk set aside (222/2), token-copy tests fail with the copy hunk set aside (217/2). Not exercised: the HF fetch block on a real non-canonical pod (no harness test) — first real run will be watched. Owner-approved ladder/any-region/HF store (Casey, 10-07). APPROVE.


---
_Comment/review ids in this file: 6042555477_

