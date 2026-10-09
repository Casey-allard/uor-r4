# Owner-directed main consolidation, 9 October 2026

Main is the shared source of truth. This inventory records 1,442 remote branches,
including 577 Codex branches, and 1,282 local branches, including 620 Codex
branches. Of the remote Codex heads, 510 have an exact merged PR receipt; 67
require reconciliation (one already ancestral, one patch-equivalent, and 65
with unmatched patches). A squash merge can preserve delivered behavior while
leaving its original commits outside main ancestry.

The protected merge queue requires SQUASH and this account has no administrator
permission to change it. Consequently a restorable Git source bundle is retained
in main, preserving the exact original histories without replacing the current
implementation with historical branch snapshots.
This is preservation, not validation or promotion of unfinished code. Use
the source-bundle restoration instructions below, then
`git show <recorded-head>:<path>` to recover exact original source or records.
The JSON inventory binds names, exact heads, PR status and unmatched patch commits.

The live `codex/lab-state` coordination branch is excluded from branch retirement:
it is a changing operational claim store, not an alternate model source.
Non-Codex branches and foreign active workspaces are preserved.

Pending source integration must remain explicit. PR #1841's opt-in CUDA alias
reducer has source review but its declared compile/CUDA/gradient validation was
pending in its published PR; retaining it here does not turn that into PASS.
The current coupled-episode source on the pod at
80eb132f4f554af5a775d1fbbfb8d3c7dcdbf0d7 has 115 focused tests and an optimized
CUDA-feature build, but its export attempt failed with file-exists I/O;
independent native reload is NOT_RUN. Neither is a qualified learned result.
No new learning task starts until consolidation and pending delivery are resolved.

The October 9 owner direction makes development worktrees optional. Temporary
branches may transport protected delivery, and must be removed after actual main
merge verification and preservation. An open PR, pushed branch or merge queue
entry is not completion. Accepted and negative artifacts, dirty source and other
labs' active material must survive cleanup. Storage receipts are updated after
physical free-space measurement; apparent directory sizes are not reclaimed bytes.

## Pending implementation inventory

The file triage found 143 distinct absent paths, including 59 Rust files, across
32 historical heads. Absence is not proof that a mechanism should be activated.
The attached pending summary distinguishes missing diagnostics, historical
comparators, superseding source and unfinished implementation. Those records are
retained source, not fresh compile or capability results. Resolve an applicable
implementation or record its reasoned retirement before proceeding past that
dependency. The active native geometric research path remains the current plan.

## Restore retained source

The tracked `preserved-source.bundle` is SHA-256 bound by
`source-bundle-receipt.json`. Its prerequisites are commits already in main.
In a checkout of main, run:

```sh
git bundle verify docs/labs/consolidation-2026-10-09/preserved-source.bundle
git fetch docs/labs/consolidation-2026-10-09/preserved-source.bundle HEAD
```

Read any exact head from `preserve-heads.json` with `git show <sha>:<path>`.
The thin bundle pack passed strict object validation against main prerequisites.
It retains the recorded source histories relative to the main base, including
the pod-only coupled source. Recovering a commit does not adopt its code.
