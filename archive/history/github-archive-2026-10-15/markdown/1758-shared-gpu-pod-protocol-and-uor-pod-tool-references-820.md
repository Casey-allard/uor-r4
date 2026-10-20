# #1758 — Shared GPU pod protocol and uor-pod tool (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T20:31:08Z
- **Updated:** 2026-10-09T13:09:02Z
- **Closed:** 2026-10-05T21:24:44Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1758
- **Merged:** 2026-10-05T21:24:44Z
- **Merge commit:** e445ee94657311ae158edfab292f2c58cd4f5b24
- **Base / Head:** main / UOR-Foundation:claude/compute-protocol
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820. Compute board (owner's log, not a channel): #1750.

One tool and one cadence for every lab session to see, lease, create, share and delete the owner's Runpod GPU pods.

## What is in it
- `scripts/pod/uor-pod` (bash 3.2-compatible, shellcheck-clean): `status`, `up`, `lease`/`renew`/`release`, `down`, `run`, `register`, `keep`/`unkeep`, `prune-stopped`, `reap`, `ssh`, `bootstrap`, `seed`/`pull`/`push`, `log`, `gpus`. No arguments prints the one-screen rules (also at the top of `status`).
  - Identity is **lab + session** (`--lab L --session S` / `UOR_POD_SESSION`); leases `leases/<pod>/<lab>-<session>.json`, atomic writes under one flock (perl `flock(2)` on macOS), auto-expiry, mirrored to `/root/leases/` for the pod-side reaper; every ledger line and board post carries both.
  - An idle leased GPU is **not** free; only release or expiry frees it. An expired lease is taken over only after a probe shows no live job on those GPUs.
  - `status` lists running and stopped pods (stopped = cleanup item), per-GPU utilisation next to its lease holder, NO LIVE LEASE flags, stray temp files, 5090/4090 stock, and a "Free GPUs you can lease now" summary; refusals print it plus "spin up your own pod" guidance.
  - GPU policy: RTX 5090 only default (CAP 120); 4090/L40S/6000 Ada explicit; A100/H100/H200/B200 refused without `--owner-approved`. Placement 2×5090 secure: EUR-NO-1 (canonical volume `lmd1pfah3y`) → EU-RO-1 → EUR-IS-1 (`uor-shared-<DC>` 100 GB created lazily, hot-set seeding from `scripts/pod/hot-set.txt`); no silent fallback (`--wait`, explicit `--gpu 4090`, or `--allow-off-volume`).
  - Caps ≤ 4 running pods and ≤ $8/h for all labs together (`uor-test-` validation pods excluded from the pod count only); `down` refuses while another session's lease is live, KEEP pods, unmirrored `/root/runs`, non-canonical/volume-less pods without `--confirm-archived`.
- Standard template **`uor-r4-cuda128` = `h15vb984sw`**: official `runpod/pytorch:1.0.2-cu1281-torch280-ubuntu2404` (CUDA 12.8.1 devel, nvcc), SSH via account `PUBLIC_KEY`, no Jupyter (only 22/tcp), 60 GB disk, `/workspace`.
- `scripts/pod/uor-pod-bootstrap.sh`: nvcc check, GPU→`CUDA_COMPUTE_CAP`, Rust 1.97.1 from a volume archive unpacked to local disk (compiling from the network FS was IO-bound), cargo jobs = pod CPU quota, binaries cached at `/workspace/bin/<sha>-sm<cap>/` with `BUILD.json` + `parity.log` (only a parity PASS counts), optional Ollama on the volume, `/root/.uor-pod-env`.
- `scripts/pod/uor-reaper.sh`: deletes its own pod via GraphQL `podTerminate` with the pod's `RUNPOD_API_KEY` (read at use, passed on stdin; the pod key gets 403 on REST v1) when no lease is live and GPUs are idle ≥ 20 min.
- `docs/labs/compute.md` (cadence, GPU policy, placement, hot set, layout, caps, takeover, security, recovery, pitfalls, validation record); short inline "GPU pods" rules in `AGENTS.md`; one line in `docs/labs/README.md`.

## Checks at this head
- `shellcheck` on all four scripts: clean.
- `scripts/pod/tests/uor-pod-dryrun.sh` (fake runpodctl/ssh/gh, `UOR_POD_DRY_RUN=1`): 61 passed, 0 failed.
- `python3 scripts/check_claim_wording.py`: passed.
- Real validation (1×5090, EUR-NO-1, commit 6a918a43): create→SSH 30–51 s on four pods; nvcc 12.8 present; cold bootstrap 609 s (fetch 326, build 231, parity 42) with `cuda_stack_ops_parity` PASS 31/0/1; warm bootstrap 2 s; `run` executed on GPU 0; `release`+`down` deleted the pod; the pod-side reaper (idle limit 1 min for the test) deleted its own pod 61 s after release. Test spend ≈ $1.2 of the $3 limit; all test pods deleted.

## Limitations
- First validation pod: 30/31 parity tests reported `CUDA_ERROR_NO_DEVICE` although `nvidia-smi` saw the GPU; not reproduced on the next pod; cause unknown (failed cache kept as `…failed-*`).
- Not exercised for real: 2×5090 cold build, non-canonical volume seeding, `--allow-off-volume`, `--with-ollama` (dry-run only).
- Existing pods were not touched; no reaper was installed on them.

Not to be merged by the author without the owner's exact-head review.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6002392867 — 2026-10-05T20:32:08Z — Casey-allard

Exact-head check at 3198faca (Claude lab): shellcheck clean on uor-pod, uor-pod-bootstrap.sh, uor-reaper.sh and the dry-run test; dry-run suite 61 passed / 0 failed; `scripts/check_claim_wording.py` exit 0. Real validation by the author on 1×RTX 5090 (create→SSH 30–51 s, cold bootstrap 609 s, warm ~1 min, parity 31/0, run/release/down and pod-side self-delete exercised; spend ≈$1.20). My added commit: AGENTS.md iCloud rule for local downloads (owner 10-05) and replacing the stale "No CUDA/external GPU is authorized" sentence with "only through uor-pod within the caps". Approve at this head.


---
_Comment/review ids in this file: 6002392867_

