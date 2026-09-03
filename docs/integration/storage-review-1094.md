# Storage review after #1094 preparation

2026-09-03. Review only; no deletion was performed.

The original mixed checkout still has the same two tracked `ppmi_proxy.npz`
deletions and untracked `.serena/`, corpus-producer directory and `tools/` observed
at intake. They were not modified or staged. Existing reader/core/frame assets
were reused; this task created no model download, fitted checkpoint or Rust
build cache. The withheld curator directory remains sealed at mode 000; its
logical size comes from its frozen commitment, not a traversal during review.

| Retained task storage | Observed allocation / identity |
|---|---|
| Isolated issue worktree | 1,164,892 KiB at review; source is bound at `ff925481fcb290e8f91442a28a1b43b51b28dd26` |
| Original stopped preparation | 84 KiB allocated; seven original files total 66,523 logical bytes, copied exactly into the public evidence directory |
| Frozen corpus/selection/policy | 3,397,265 committed logical bytes; metadata and sentinel are accounted separately by the preparation receipt |
| New source-index snapshot and graph | 445,032 KiB allocated |
| Index refresh receipts | 508 KiB allocated at review |
| Available filesystem space | 53,267,152 KiB, approximately 50.80 GiB |

The [index receipt](code-index-1094.md) distinguishes the pre-analyzer setup
failure from the sole analyzer launch and retains both. All previous indexes
and source snapshots remain available. No model output spool exists because
the interpreter never started.

The source worktree and graph are reproducible storage candidates after a later
retention decision, but the stopped preparation currently names that worktree
and runtime paths. Keep them for #1096's diagnosis and evidence replay. Original
and copied receipts, immutable corpus metadata, sealed inputs, unique models,
research sources and user material are retained. The public knowledge import
adds only bounded project records and provenance; private histories are excluded.
