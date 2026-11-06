# REPORT: one-model-end-to-end, deepseek-18 (centroid capture delivery)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](ORDER.md). Base: `3a9c2f11d`.

## What did not work, first

- **The first package could not produce the registered inventory.** The scorer always
  cached DEV in addition to TRAIN and four panels, while the strict evaluator accepts
  exactly five no-memory splits. DeepSeek-17 merged the `--query-cache-dev 0` repair.
- **The first job did not name a runnable pod source.** Its default checkout did not
  exist on every fresh bootstrap path, and its source pin did not contain the newly added
  job. The job is now copied separately, uses a clean bootstrap checkout when present,
  and otherwise fetches and checks out its own exact source pin.
- **The first resource checks could not guarantee completion.** They checked 80 GiB
  before and after capture but then made compressed and readback copies. The package now
  starts with 128 GiB, checks 118 GiB immediately before capture, requires 80 GiB after
  capture and gates readback on both compressed archive sizes plus a 31 GiB reserve.
- **The first timeout covered only the scorer.** Downloads, CUDA tests/build, sealing,
  compression and uploads could outlive it, while a self-set wall budget must not cancel
  a healthy run. The self-cancelling timeout is removed; the watcher renews each healthy
  lease until the job writes `EXIT`.
- **A success-path log compared a file while also appending transfer output to it.** The
  transfer record now has a stable source file and is copied to the uploaded `store.log`
  only after the other readbacks finish.
- **A failure could advertise `EXIT` before its diagnostics reached the store.** Failure
  evidence is uploaded and read back first. A complete set is marked by `failure/EXIT`;
  an incomplete set creates `FAILURE_EVIDENCE_INCOMPLETE`. A pod whose cache exists but
  was not transfer-verified is renewed and marked `KEEP` instead of deleting its only copy.
- **The first remote output was not immutable.** A second-resolution id could collide
  with an existing prefix, and `uor-store` updates files rather than replacing a folder.
  The id now has a random suffix and an existing remote prefix is refused.
- **The first readback covered only the archive and its checksum.** Every evidence upload
  needed for an accepted or failed result is now read back and compared before a pod may
  be released.

## What changed

Size: large (four paths, about 1,000 added lines and 50 deleted lines). The roadmap item
has no registered size-line estimate, so a size ratio is undefined.

- The sheet pins source `3a9c2f11d`, captures both pod ids from the reviewed lease
  commands, owns teardown from the first successful `up`, and requires both the job
  `EXIT` and the `uor-pod` wrapper's terminal record. The job uses a clean bootstrap
  checkout or fetches its own exact pinned clone. It leaves either existing busy pod
  untouched.
- A controller-owned run directory records the two immutable output-store identities
  before either pod starts, so CPU restore does not depend on scraping a deleted pod log.
- The capture command uses `--query-cache-memory 0 --query-cache-dev 0`; its cache is
  exactly TRAIN plus `oasst2`, `wiki-00000`, `wiki-00020` and `wiki-00040`. The job
  checks that inventory, its pinned identities and its row totals before sealing.
- The job checks the visible RTX 5090, source and tracked tree, all data/archive/selected
  checkpoint hashes, disk phases, `zstd` and CUDA tests. It has no self-set wall-clock
  cancellation.
- It records the source, run id, manifest, cache archive, normal-output archive and
  capture binary hashes, then uploads and reads back the cache, checksums, logs, GPU
  record and status.
- The CPU section builds the registered evaluator, restores and verifies both the n3S
  checkpoint and captured cache, checks each status against its seed, source pin,
  capture binary, script hash and archive hash, records the evaluator source/binary
  provenance, enforces 170 GiB including a 24 GiB build allowance and 146 GiB after
  build for both complete restores, plus the 31 GiB post-restore reserve, binds each
  report to the captured manifest and checkpoint files, then runs the evaluator and
  aggregate without a self-set wall-clock cancellation. Each CPU attempt gets a fresh
  timestamp-plus-random local root instead of mutating a prior restore.

## Measurements

No model, cache or GPU job ran. The raw no-memory cache is 28.02 GiB by the pinned
27,416,554 rows, width 256 and row records plus the 1,460,697 panel rows. The two
parallel captures are estimated at 2-4 hours each. Each watcher renews its healthy
lease until the job finishes; at a $1.15/h ceiling, a four-hour pair projects $9.20.
The owner checkout's live shared model-time ledger was absent during preparation, so no
historical balance is claimed; actual lease usage will be recorded at execution.

`uor-harness prep-check` reports the sheet structurally complete and `PENDING`. The final
job hash is
`77501c4feebf79835fa0769c54022cc01e0d0595ea33c654726714875aee551a`.

## Checks run at the head

- `bash -n` on the capture job: passed.
- `shellcheck -x` on the capture job: passed.
- `bash -n` and `shellcheck` on the sheet's lease/run block: passed.
- `git diff --check`: passed.
- `uor-harness loop report --check --order` for both deepseek-16 and deepseek-18:
  passed.
- `uor-harness loop pr-check --local`: passed; its only warning was the existing stale
  `docs/start/FILEMAP.md` advisory.
- The two source archives and all eight selected checkpoint files were verified against
  their published and inner SHA-256 pins while preparing the package.

Not run: CUDA tests, pod bootstrap, capture, cache evaluation, panel scoring or any GPU
work.

## Skills used

No system-provided Codex skill applied. The repository's D22, D67, geometry, delivery and
compute rules were used directly.

## Still open

1. Obtain a fresh non-author `TEST FITNESS: FIT` for the sheet.
2. Obtain the D67 pre-run review naming the final script hash and two leases.
3. Run the two captures without touching either existing busy pod.
4. Restore both caches and checkpoints on the D84 host.
5. Run the two evaluators, verify their manifest/checkpoint bindings, and aggregate.
6. Record the registered result in a new active task REPORT and the float-training
   notebook; leave the merged deepseek-16/17 records historical.
7. Keep the roadmap's other sealed cells explicitly NOT_RUN in this n3S slice.

## Plain summary

The centroid experiment now has a complete package that can produce exactly the cache
the CPU evaluator expects. It enforces disk admission without cancelling a healthy run on
a self-set time budget, uses a new output path for every attempt, verifies every input
and uploaded artifact, and restores enough evidence to evaluate both seeds locally. It is
ready for an independent fitness and pre-run review; no GPU work has started.
