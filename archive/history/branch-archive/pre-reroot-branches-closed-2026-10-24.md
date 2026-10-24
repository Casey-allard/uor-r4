# Branches from before the 13 October re-root, closed out (24 October 2026)

These five local branches sat on the history from before `main` was re-rooted on 13 October 2026 (first commit `9556c1bd3`, #2320), so they share no commit with `main`. Each was checked file by file against `main` before deletion: nothing on them is lost.

| Branch | Tip | Delivered as | Where its content is on `main` | Disposition |
|---|---|---|---|---|
| `inference-fix` | `80272c4f6` (9 Oct) | PR #2128 | `archive/labs/pointer-selected-ids-decode-2026-10-09/README.md`, byte-identical (blob `8d0fb260b7ee`). The `current-state` entry is in `archive/history/current-state-2026-10-03-to-2026-10-13-archive.md`. | Deleted 24 Oct |
| `pcopy-accessor` | `10021457d` (9 Oct) | PR #2131 | `archive/labs/pcopy-accessor-handover-2026-10-09/README.md`: main's copy is a strict superset (the same text plus an 11-line later note). The `current-state` entry is archived as above. | Deleted 24 Oct |
| `repro-fix` | `6cc7f16cd` (9 Oct) | PRs #2122, #2123 | Both READMEs are byte-identical: `archive/labs/numeric-trace-reproduction-2026-10-09/README.md` (`a01681bb164a`) and `archive/labs/numeric-pointer-split-2026-10-09/README.md` (`2c15000733b4`). `crates/uor-r4-training/src/bin/numeric-pointer-trace.rs` is live and newer, and keeps all 48 lines the branch added. | Deleted 24 Oct |
| `tmp-census` (was `kimi/softmax-census-2026-10-12`) | `6a901d4c1` (12 Oct) | PR #2253 (`3f6d97bc4`) | `archive/labs/softmax-census-2026-10-12/README.md`, byte-identical (`8f1b41955`). `scripts/softmax_path_check.py` is live and byte-identical (`ee8ea65cc`). See also the `kimi/d23-learning-law-2026-10-12` entry below. | Deleted 24 Oct |
| `preserve/deepseek-router-prep-2026-10-12` | `72edb444b` (12 Oct) | The router gate itself: PR #2243 (`49174351a`); `read_router` is live | Its only unique file was `.agents/knowledge/audited.md`, antigravity's peer-audit ledger for PRs #2233–#2242. It is kept byte-identically as [`peer-audit-ledger-2026-10-12.md`](peer-audit-ledger-2026-10-12.md) (blob `9378a7a86`). | Deleted 24 Oct, after this record merged |

