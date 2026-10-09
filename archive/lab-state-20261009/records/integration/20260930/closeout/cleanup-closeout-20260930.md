**Internal storage cleanup closeout — 2026-09-30**

At **03:26:52 UTC**, the internal data volume had **69,735,235,584 bytes available (69.735 GB; 64.946 GiB)**. The recorded starting availability was **41,731,727,360 bytes (41.732 GB; 38.866 GiB)**, for a host-observed physical increase of **28,003,508,224 bytes (28.004 GB; 26.080 GiB)**. GB means 1,000,000,000 bytes; GiB means 1,073,741,824 bytes. These figures use `df -k`, not directory-size estimates. Concurrent host activity can change free space, so the total is a shared host observation. The reading was **264,764,416 bytes (0.265 GB) short of 70 GB**; the target was approached but not exactly reached.

The owner explicitly authorized: “Remove verified candidates now; waive the wait for this cleanup only.” That waiver applied only to this verified batch. **The standing two-hour notice requirement resumes for subsequent eligible cleanup.** This receipt grants no further deletion authority.

Removed **14 inactive project worktree checkouts**, **3 compiler incremental directories** and **75,642 authorized cache files**. The cache inventory accounted for **8,017,567,744 allocated bytes (8.018 GB; 7.467 GiB)** and **7,875,245,755 logical bytes**; these figures must not be added to the physical `df` increase. The cache deletion journal exactly equals the authorized candidate set: zero excluded or unlisted files removed. Whole-cache/component scopes passed fresh full-file-set and identity checks; active-client scopes removed only listed unchanged, unopened payload entries. No application was terminated.

Project checkout disposition is recorded below. Removing a checkout did not delete its branch or Git history and does not establish a new capability result or resolve remaining source-review findings.

| Removed checkout | Retained HEAD | Preservation evidence |
|---|---|---|
| `g2-keys-20260929` | `3fc3d5922e0e` | Patch-equivalent merged change; [PR #1505](https://github.com/UOR-Foundation/uor-r4/pull/1505); branch retained |
| `natural-20260929` | `f7f19990035d` | Patch-equivalent merged change; [PR #1504](https://github.com/UOR-Foundation/uor-r4/pull/1504); branch retained |
| `s4-serving` | `f7a5c4f07413` | Patch-equivalent merged change; [PR #1493](https://github.com/UOR-Foundation/uor-r4/pull/1493); branch retained |
| `arm-c-result-20260929` | `db31305af3ac` | Patch-equivalent merged change; [PR #1499](https://github.com/UOR-Foundation/uor-r4/pull/1499); branch retained |
| `gate-20260929` | `1ea58d93b206` | Patch-equivalent merged change; [PR #1502](https://github.com/UOR-Foundation/uor-r4/pull/1502); branch retained |
| `arm-c-20260929` | `9667f49e2084` | Patch-equivalent merged change; [PR #1498](https://github.com/UOR-Foundation/uor-r4/pull/1498); branch retained |
| `from-stack-20260929` | `f98561466062` | Patch-equivalent merged change; [PR #1497](https://github.com/UOR-Foundation/uor-r4/pull/1497); branch retained |
| `g-binding-result-20260929` | `c5a3f854fb2a` | Patch-equivalent merged change; [PR #1496](https://github.com/UOR-Foundation/uor-r4/pull/1496); branch retained |
| `s4-adoption-doc` | `7924d07520cd` | Patch-equivalent merged change; [PR #1494](https://github.com/UOR-Foundation/uor-r4/pull/1494); branch retained |
| `kimi-1479-base` | `a10d71ec5d6f` | Exact HEAD is in audited main; fresh checks resolved initial detached-HEAD metadata mismatch |
| `aerm-decompose-20260929` | `3623ad7d31d4` | Patch-equivalent merged change; [PR #1489](https://github.com/UOR-Foundation/uor-r4/pull/1489); branch retained |
| `g-v1-20260928` | `9af92e899cd8` | Patch-equivalent merged change; [PR #1482](https://github.com/UOR-Foundation/uor-r4/pull/1482); branch retained |
| `s1-4-snap` | `19ec19a30c68` | Exact merge-tree equality; [PR #1506](https://github.com/UOR-Foundation/uor-r4/pull/1506); branch retained |
| `lab-runner` | `4d0defa0b619` | Identical 19-file import; merged [PR #1521](https://github.com/UOR-Foundation/uor-r4/pull/1521); remote archive tag below |

The original runner source remains remotely recoverable at [`archive/lab-runner-4d0defa0`](https://github.com/UOR-Foundation/uor-r4/tree/archive/lab-runner-4d0defa0), an annotated tag with object `b17d6c46928ef1097b7b57fe54e7f85e13697147` that peels to `4d0defa0b619ef775df294b636b776e582d769c3`. All 19 original changed-file blobs were identical at import `7ed92aa404f303129dc9f42a433c05b8777e58a0`; the imported patch ID also matches. The import was incorporated into merged PR #1521. The installed internal runner binary remains available; only its obsolete inactive checkout was removed.

Preserved the owner’s dirty checkout; all dirty or unmerged worktrees; the managed H4 checkout; branches and Git history; models and sealed reports; external recovery originals and rollback image; conversations, profiles and credentials; Claude VM images; dirty plugin marketplace sources; installed runtimes and rebuilt proc macros. Only the three verified compiler incremental subdirectories were removed from build caches. No external volume was cleaned.

The detailed cache file lists, absolute user paths and process inventories remain local. The accompanying sanitized JSON binds the local source receipts by SHA-256 and includes full retained project commit identities, without publishing browser entry names or private per-file manifests. Prepared for GitHub publication; this preparation step published nothing and performed no deletion.
