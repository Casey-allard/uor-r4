# Readback receipt: d11-exporter-float-trunk, kimi-46

Hash readback was recorded on 2026-10-27 before pod deletion; the pod section was updated after `down`.

- **Attempt 1:** `results/kimi/d11-fbs-longer-dual-2026-10-26/`. The initial local readback contained 40 total files: 38 payload files listed in `SHA256SUMS`, plus `SHA256SUMS` and the live `job.log`, which are deliberately not self-covered. `shasum -a 256 -c SHA256SUMS` passed all 38 entries. A later readback of attempt 2 nests 85 more files below this local path; that is not attempt-1 inflation.
- **Attempt 2:** `results/kimi/d11-fbs-longer-dual-2026-10-26/attempt-2/`. Local `find . -type f | wc -l`: 85. Local `wc -l < SHA256SUMS`: 83. `shasum -a 256 -c SHA256SUMS` passed all 83 entries; `SHA256SUMS` and `job.log` are excluded by the job's manifest rule. `scripts/pod/uor-store ls results/kimi/d11-fbs-longer-dual-2026-10-26/attempt-2/ | wc -l`: 85.
- **Train receipt:** attempt-2 `train.json` SHA-256 `b4ecb87da64916683033cd1098a7c19f1af679549fc5c74dfbd55f438836f342`, matching the local file and the REPORT.
- **Storage:** after readback, `df -k ~` reported `32731228` KiB available, above the required `31457280` KiB.
- **Pod:** compute-ledger up `2026-10-27T00:21:49Z`, release/down `2026-10-27T01:00:48Z`; `vast-57891354` is deleted. `scripts/pod/uor-pod status` reports 0 pods and $0/h. The 2,336-second provider-lifetime proxy at $0.975/h is $0.633, not a final invoice.

The local commands were run from `/Users/casey.allard/uor-r4/.worktrees/kimi-d11-float-trunk`; the hash checks ran in each archive's readback directory. The HF store retains both attempt paths.
