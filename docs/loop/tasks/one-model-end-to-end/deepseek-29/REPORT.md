# REPORT: one-model-end-to-end, deepseek-29

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Preparation only so far; no model, pod
or spend.

## What did not work, first

- PR #3845 merged after Claude's docs-only claude-315 change, so the pre-merge
  runtime identity no longer matched fresh `main`.
- A post-merge D67 review found that CPU grading did not seal the host CPU
  identity required by D84.
- The 1,024-step run reached row 6: free EOS stops moved, but complete accepted
  replies remained zero.

## What changed

- The job now records `cpu.json` and `cpu.txt` before model work and requires
  each grade to carry the D84 CPU-development substrate and GPU-required
  qualification labels.
- Every base and checkpoint grade gets a sidecar binding its exact bytes to the
  recorded CPU model, architecture and logical processor count.
- Final verification inventories every grade, requires its sidecar, and checks
  the sidecar again before sealing; a missing CPU file fails the manifest.
- OOM retry compacts bulky partial-attempt checkpoints, exports, panels and
  scores into a sealed hash manifest while retaining logs and progress.
- Gate and mutation coverage require both CPU labels, both grade bindings,
  CPU-file seal inclusion and retry compaction.
- `docs/harness/claims.json` carries this branch's claim and is the sole
  out-of-item-scope path; D76 makes that scope finding advisory.
- The 2,048-step sheet and its merged runtime have recorded FIT and D67 GO.

## Measurements

No model or GPU step has run. The completed 1,024-step run projects the
2,048-step dose at about 24 minutes of training, 45-50 minutes of pod wall time
and roughly $0.45-$0.55 at the observed $0.563/hour offer.

## Checks run at the head

- `gate.sh`: 235 checks passed.
- `test-mutants.sh`: 99 of 99 semantic mutations caught.
- `bash -n`, ShellCheck at warning severity and `git diff --check`: passed.
- The target sheet has no prep-shape error; FIT and D67 GO are recorded.
- Fresh FIT: **FIT GO**, no blockers, at
  `536d95242b8301541b207fe171fbe5a96b2fa244`, runtime
  `8928ce7cae729894820ae1b7f3106d071f18dda77bdbf198a7dd4d8556d1eec5`,
  reviewer session `01a1b933-75a7-7df2-9d05-7997d387beaa`.
- Main-integrated FIT/D67: **GO**, no blockers, at
  `c020b4d2e5946703e43ac158cfee7d48622dd79e`, runtime
  `b5aa959857f387bdc2329d0e68e243cb6473e88fa46f56553f95a47c884eaa80`,
  reviewer session `01a1b944-30a2-7ce0-aac4-8b01fc2cc6d2`.
- The job records the CPU model, architecture and logical processor count before
  grading and rejects a base grade whose D84 substrate labels are absent.
- `test-hardware-evidence.sh` exercised the isolated helper library: it captured
  this machine's CPU, rejected unknown and zero-processor identities, rejected
  unlabelled and tampered grades, bound the exact grade SHA-256 to the CPU,
  compacted six partial-attempt files, measured all sealed bytes, and verified
  that `cpu.json` and `cpu.txt` are present in `SHA256SUMS`.

Not run: pod creation, remote build, training, export, panel scoring, store
write or seal readback.

## Skills used

No system-provided Codex skill applied. The repository's D22, D67, D84, geometry
and delivery rules were used directly.

## Still open

- Merge the reviewed CPU-identity and merged-runtime repair.
- Execute and evaluate the 2,048-step dose; no automatic second extension.

## Plain summary

This unit repairs the post-merge launch evidence and executes the one registered
2,048-step extension from the row-6 result.
