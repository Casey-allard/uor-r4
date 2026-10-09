# Verified Codex consolidation cleanup

PR #2016 merged at `20a62312f7211a73a8a2e4c78fd5e400fa0b2529`;
all 17 delivered files matched reviewed head on fresh main. PR #2018 merged at
`7c76c7437c4d8ed55614ccb2eca5e89390187e45`; all 14 modified/untracked
files were verified by SHA-256 and exact bytes against main before removal.

Cleanup removed eight Codex worktrees: five clean and three containing the
preserved tails. Foreign active DeepSeek work was untouched. The original
checkout and its untracked token-optimizer material were preserved. A detached
phase-binding checkout was removed by the parallel participant, not counted
among these eight Codex removals.

486 surviving historical GitHub branches were retired with expected-head
compare-and-delete protection; 90 were already absent from parallel cleanup.
526 local historical refs were retired atomically with expected-head checks;
94 were already absent. Changed heads were not deleted. The live lab-state
coordination record is retained. Temporary consolidation delivery refs are
removed after this receipt's actual merge; they hold no unique work.

726 stale regenerable marketplace staging entries were removed after checking
age, unchanged modification times, active plugin symlinks and open handles.
The manifest recorded 4,542,287,872 allocated bytes. This is not a claim of
physical reclamation attributable to this lab. Free space rose from about
19 GiB at the inventory to about 37 GiB during the combined cleanup; `df`
receipts report actual observed space. Parallel participants also cleared space.

Three result directories were uploaded through cloud-store, MD5 round-trip
verified and indexed, then compared file-by-file against their iCloud archives
before moving the local copies to the Trash. Complete source file sets and
SHA-256 identities match: Track B conversion (41 files), fourth-lab selected
code-choice fit (29 files), and active-attention records (472 files). Apple's
tar metadata entries remain preserved in the archives. No global Trash emptying
was performed. The full receipts identify every source and retained location.

These receipts establish preservation and cleanup. They do not compile,
activate or qualify the 59 missing historical Rust files. Pending native CUDA
alias integration and the coupled export defect remain in the source inventory;
source archived on main is recoverable, and its validation status remains honest.
No learning or new model experiment was run during consolidation.
