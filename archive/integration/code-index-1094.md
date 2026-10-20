# Scoped source-navigation refresh — #1094

Status: `COMPLETE_SCOPED_SOURCE_NAVIGATION`, 2026-09-03. This is a local
GitNexus navigation receipt, not model QA, a proof or complete call-graph
verification. No model or comparison payload was accessed by this refresh.

The snapshot binds committed source
[`ff925481fcb290e8f91442a28a1b43b51b28dd26`](https://github.com/UOR-Foundation/uor-r4/tree/ff925481fcb290e8f91442a28a1b43b51b28dd26).
Use repository alias **`uor-r4-ff92548-1094-active`**. The snapshot is named
`uor-r4-ff925481fcb2-1094`; its exact source manifest has SHA256
`2eb8cb7a2e5bff0b94e3eca9c3ccf9168dd40af1b8328c82eb4e022e7879db76`.
The [public receipt](code-index-1094.json) includes the manifest's 397 path,
size and SHA256 records, source tree, original receipt identities, measured
resource use and observed query/context results. It includes no bulk source,
comparison rows or private logs. `--skip-git` leaves GitNexus's commit field
empty; the source manifest supplies revision and exact-byte provenance.

## Selection and actual coverage

All 389 selected source paths carried from #1082 retain their prior hashes.
Eight added files are the text adapter package's `__init__.py`, `__main__.py`,
`adapter.py`, `campaign.py`, `contract.py`, `curate.py`, `worker.py` and
`policy.json`. Every new file is represented. All 397 snapshot hashes were
checked after indexing.

| Scope | Observed count |
|---|---:|
| Manifest source files | 397 |
| Represented source File nodes | 392 |
| Additional manifest File node | 1 |
| Total File nodes | 393 |
| Graph nodes / edges | 32,686 / 86,732 |
| Communities / reported processes | 977 / 876 |
| Embeddings | 0 |

Five source paths are omitted. `crates/uor-r4-graph-cli/src/lib.rs` and
`src/server.rs` exceed the unchanged 512 KiB limit. The three
`crates/uor-r4-core/src/bin/` files `r4-group-geometry-export.rs`,
`r4-h4-spin-frame-export.rs` and `r4-zoology-frame-export.rs` are absent for
an undetermined analyzer reason. No limit expansion or analyzer retry ran.

The analyzer also reported 413 callable candidate sets exceeding cap 32,
57 unlinked cross-language property sites, 5,929 of 6,129 candidate entry
points not ranked in, eight depth-capped traces, 3,893 branching-capped
callees and 88 budget-capped walks. Graph absence does not establish source
absence or an unused path.

## Observed navigation, limits and retained setup history

One stdio File inventory, one adapter keyword query and one context lookup
were completed. The query found `segment_request` and related package
symbols. Its context reported incoming `prepare` and `_adapter_record`, and
outgoing `_boundaries`, `_lex`, `_refuse`, `_syntax_refusal` and
`derived_input_sha256`. No process groups were returned. These are observed
navigation results, not execution or correctness evidence.

GitNexus 1.6.10 launched exactly once with two workers, `--index-only`, no
embeddings and a 512 MiB Ladybug buffer pool. Recorded wrapper, analyzer and
MCP work totaled **60.961895 seconds** against the unchanged 120-second
budget. Allocated snapshot/receipt storage measured **456,216,576 bytes**
before the consolidated receipt, below 2 GiB. The database pool setting is
not a total-RSS bound.

The initial Python 3.9 wrapper failed before analyzer launch because its
`Path.stat` lacked the requested keyword. The authorized Python 3.12.14
continuation reused the exact verified snapshot for the sole analyzer launch.
The first MCP response parser rejected guidance appended after JSON; the
retained response was parsed correctly without repeating the File inventory,
then the query/context completed. Both setup failures and original local
receipts remain recorded.

Prior snapshots and registry entries were preserved; only the new alias was
added. Shared configuration, hooks, scientific inputs and user material were
unchanged. The comparison's withheld directory remains sealed. This refresh
does not revise any scientific result or authorize model execution.
