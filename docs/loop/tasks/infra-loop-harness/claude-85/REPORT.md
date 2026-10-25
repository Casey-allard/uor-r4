# REPORT: infra-loop-harness, claude-85

Lab: claude. Order: [ORDER.md](ORDER.md).
References #3165. References #2904.

## What did not work, first

- **The 20 Oct archive README says nothing new goes into iCloud.** It cannot be edited, because archive files are add-only. So the owner's 24 Oct direction to use iCloud again is recorded in a new note beside it, not as an edit.
- **The first pass was to tar every local folder whole.** It excluded build output instead: `target`, `*-target` and `harness-target`, about 3.5 GB of rebuildable caches. It also skipped the extracted legal case, which is a byte-verified copy of a tar already in this archive.
- **`mvm-review` and `mvm-issue-draft.md` were left in place.** They are a clean clone of an external repository and a draft that may belong to the separate VM work the owner asked us to leave alone.

## What changed

- **`archive/storage/icloud-index-2026-10-25.tsv`:** six rows, in the columns of the 20 Oct index, sorted by remote path.
- **`archive/storage/icloud-additions-2026-10-25.md`:** the owner's words, what each tar holds, and how to fetch one.
- **`docs/reference/storage.md`:** a "where is it" row pointing to both files.

## Measurements

- Six tars, 1,029,389,312 bytes in all.
- Each md5 matched its iCloud copy, and each copy was then evicted with `brctl evict`, which succeeds only after upload.
- Free disk went from 41 GiB to 52 GiB. Most of that came from deleting build folders: the tars themselves are small, and part of the gain is still held by iCloud's local cache until eviction finishes.

## Checks run at the head

- `loop report --check` and `loop pr-check --local`: PASS.

## Skills used

- None.

## Still open

- None for the archive.

## Plain summary

The laptop's leftover lab data and old pod results are now in the iCloud archive. Each tar has a recorded size and checksum, so it can be fetched again when needed, and the local copies are gone.
