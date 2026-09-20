# Owner-authorized storage cleanup — September 20, 2026

Completed 2026-09-20T18:49:52.609823+00:00 to 2026-09-20T18:51:44.730203+00:00. The owner explicitly requested removing unneeded project and AI-folder material while protecting Downloads except optional large installers. This pass removed identified regenerable files; no unique model/research/conversation data was deleted.

## Measured result

| Filesystem free space | Decimal GB | GiB |
| --- | ---: | ---: |
| Immediately before deletion | 27.95 | 26.03 |
| After completed deletion | 51.17 | 47.65 |
| **Measured recovery** | **23.21** | **21.62** |

The initial discovery scan showed about 25 GiB available; background OS activity changed it before the exact pre-deletion receipt. The table uses the paired deletion-window `df -k /System/Volumes/Data` readings (27296748 and 49966148 KiB). It is the actual free-space change, not the sum of directory allocations. APFS sharing/hardlinks and live activity mean nominal removed bytes do not equal physical recovery. No claim that all remaining disk use is project bloat, or that performance has been benchmarked after cleanup.

## Removed material

| Exact scope | Nominal allocated decimal GB before removal | Reason |
| --- | ---: | --- |
| `~/uor-r4/target/debug/incremental` | 9.451 | Regenerable Cargo incremental compilation cache |
| `~/uor-r4/.worktrees/geometric-query-read/target/debug/incremental` | 3.256 | Regenerable Cargo incremental compilation cache |
| `~/.cache/codex-runtimes/codex-runtime-install-8PuRWB` | 2.031 | Abandoned runtime installer staging; matching installed runtime manifest; live runtime retained |
| `~/uor-r4/target/debug/deps/*.{rlib,rmeta,d,o} (unopened files only)` | 8.639 | Rebuildable main-checkout debug dependencies; retained all dylibs, executables and active/release worktree build outputs |
| `~/Library/Application Support/Claude/Cache` | 0.139 | Recreatable HTTP cache; Claude not running |
| `~/Library/Application Support/Claude/Code Cache` | 0.096 | Recreatable compiled JavaScript cache; Claude not running |
| `~/Library/Application Support/Claude/GPUCache` | 0.004 | Recreatable GPU cache; Claude not running |
| `~/Library/Caches/com.spotify.client` | 1.820 | Recreatable media/browser cache; Spotify not running |
| `~/Library/Caches/ollama/updates` | 0.197 | Downloaded updater cache; model store retained |
| `~/Library/Caches/com.google.antigravity/pending` | 0.168 | Pending downloaded updater cache; Antigravity not running |

Before deleting, checked running processes/open files and confirmed no Cargo/rustc execution or open references in the named directories. Main debug cleanup removed only unopened `.rlib`, `.rmeta`, `.d` and `.o` files; all `.dylib` files and executables were retained because the editor has some compiler libraries mapped. Its size counts unique inodes within the selected set but does not imply unique APFS extents or released hardlinks. Current geometric-query-read debug dependency files remain available. The abandoned runtime staging manifest matched the installed runtime manifest; the active installed runtime was preserved. Claude, Spotify and Antigravity were not running when their named caches were cleared.

This is permanent cache deletion, not a move to Trash. Regeneration/redownload may be required. Main debug rebuilding is partly cold; future jobs must project it rather than assume a warm cache. No release build tree, model artifact or historical result was removed.

## Preserved and remaining large areas

- All `.uor-models`, corpora, source weights, checkpoints, sealed attempts, negatives and research/handoff directories. The model store is about 20.8 decimal GB by the established lower-bound scan; permission-sealed subtrees remain sealed.
- Every source worktree, intentional change, Git ref/object/reflog and research archive. No worktree pruning, Git garbage collection or source reset.
- All release binaries, including the 1,704,928-byte attribution executable SHA256 `65463ebdb87980b05df1622f5e7ff197840278a32bcf2440a0876c01e935c4a1`, checked after cleanup.
- AI conversations, knowledge databases, installed plugins, active Codex runtime and browser/user profiles. Antigravity conversations and brain research (~2.4GiB combined directory allocation) are not disposable caches.
- Claude's VM bundle (~9.77GiB allocated) was preserved: `rootfs.img` and session data may contain user work, so the whole VM is not classified as bloat. No VM was mounted, altered or reset.
- **Downloads untouched.** A 151 MB OpenCode installer was found, but an installed matching app was not established; it was retained. No need to remove uncertain material to recover a negligible fraction of this pass.

Local detailed receipt: `/Users/casey.allard/Documents/Codex/storage-cleanup-2026-09-20.json`; exact selected main debug filenames: `storage-cleanup-2026-09-20-debug-files.txt` beside it. These are cleanup receipts, not model evidence. The public repository records only the relevant category/path summary; no browser content, credentials or conversation contents are included.

## Resource implications

The shared model-time JSON remains 178538565/180500000 ms. Disk recovery does not refund model time or reset the model-storage ledger. The whole-machine proposed reserve 36,766,079,385 bytes is now below the measured free capacity 51,165,335,552 bytes; the per-experiment 128 MiB stop margin remains a separate requirement. Refresh actual space and accounting before another fit, including builds, temporary data and all retained outputs. The [next prompt](deepseek-occurrence-reader-step-2026-09-20.md) includes that recovery and records any necessary standing-authorized limit extension before use.
