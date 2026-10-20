# #2107 — Pod tooling usable from other Runpod accounts; tokenized TinyStories public on HF (References #2037)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T23:03:54Z
- **Updated:** 2026-10-09T23:32:34Z
- **Closed:** 2026-10-09T23:29:27Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2107
- **Merged:** 2026-10-09T23:29:26Z
- **Merge commit:** 204fd7c84c982bafffb3c805056ba4e6b6c688ac
- **Base / Head:** main / UOR-Foundation:claude-pod-portable
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Why:** the owner asked what stops others from using our GPU setup. The answer is account details, not secrets or drivers.
- The driver, NVRTC and nvcc come from NVIDIA through the image.
- Our kernels are CUDA C source, compiled at startup by NVRTC, and are already public under MIT.

**Change**
- **Optional config file:** `uor-pod` reads `~/.config/uor-pod/config`; the environment wins over it.
- **New overrides:** volume, datacenter, image, repository slug and new-volume size. `UOR_POD_TEMPLATE=none` creates pods from the public image; `UOR_POD_BOARD=none` posts no GitHub comments.
- **Settings reach the pod:** the bootstrap takes `--repo-url` and `--hf-store`. Their characters are restricted and `--check-args` validates them. With `--hf-store`, the bootstrap fetches on any volume.
- **Docs:** `scripts/pod/config.example` and `docs/compute/pods-quickstart.md`, linked from compute.md, cuda-training.md and the README.
- **Records:** `docs/labs/open-pods-2026-10-09/README.md` and a current-state entry.
- **Hugging Face (already published):** `caseyallard/uor-r4-data` now holds `tinystories/tinystories_train.u16` (555,385,505 tokens, md5 87dc182a…, CDLA-Sharing-1.0 inherited from TinyStories) with its own licence note. An anonymous download returns HTTP 200 and the full size.

**Checks at this head**
- `bash scripts/pod/tests/uor-pod-dryrun.sh`: **passed 240, failed 0** (224 before, plus 16 new). With the change set aside, 12 of the new checks fail.
- With no config, the dry-run output of `up --gpu 4090 --count 2` is identical to main c7fce45b1, apart from the throwaway test repo's commit id.
- `bash -n` passes on both scripts, and shellcheck shows no new warnings against base.
- Claim-wording gate passes.

**Not verified:** a real `up` from another account (dry runs only).

References #2037

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 6090977135 — 2026-10-09T23:25:45Z — Casey-allard

**Exact-head self-review at `084d0cbb818e746dc708f62a0e649a592fe0c6f4`**

Fresh-context review of 5610ba01e: **no injection path, no change to default behaviour, no `set -u` hazard.**

**One medium finding, fixed in 084d0cbb:** an empty `UOR_POD_BOARD` or `UOR_POD_TEMPLATE` fell back to the project's board and template, because the defaults used `:-`. They now use `-`, so a set-but-empty value means none. The three new empty-value checks fail with the fix set aside.

**Also addressed:**
- a loaded config file is now reported on stderr;
- `uor-pod` warns when the canonical DC is outside `UOR_POD_VOLUME_DCS`;
- the `--repo-url` comment is corrected.

**Checks at this head:**
- `bash scripts/pod/tests/uor-pod-dryrun.sh`: **passed 246, failed 0** on 4 consecutive runs.
- Earlier runs on this head had 1–2 failures in two wall-clock-timed tests, `reap: deletes idle unleased B` and `lock held, fresh heartbeat without progress` (stale timer 2 s). That happened while the laptop's 8 cores were busy with a training job. Main c7fce45b1 passed 3/3 under the same load. Neither test touches the changed code paths, but I can't fully rule out a timing interaction from the extra tests.
- `bash -n` passes, and shellcheck at warning level shows the same count as base (1).

**Not verified:** a real pod from another account.

### comment 6091003954 — 2026-10-09T23:28:24Z — Casey-allard

Merged origin/main (current-state.md conflict: both entries kept, newest first). At `d862a1629566666f3495dafb2b54f719af140561`: dry-run suite passed 246, failed 0.

### comment 6091042047 — 2026-10-09T23:32:10Z — Casey-allard

Correction to my previous comment: at `d862a1629566666f3495dafb2b54f719af140561` the first run after the merge was **passed 245, failed 1**, not 246. It was the same wall-clock-timed flake (I didn't capture which of the two). The next 3 runs at this head were **passed 246, failed 0**.


---
_Comment/review ids in this file: 6090977135, 6091003954, 6091042047_

