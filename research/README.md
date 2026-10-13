# research/

Research material collected during the project. Most of it is small and tracked.

Three directories were **archived to the iCloud store and untracked on 12 October 2026**
(owner direction: labs work in one checkout and duplicate no data). Together they were
about 1 GB of mostly binary material — a 53 MB `.npz` (twice), 20 MB CSVs and 14–23 MB
PDFs — that every worktree and clone used to carry:

| archive | path | size | files |
|---|---|---|---|
| `prime-analysis` | `research/prime-analysis` | 566 MB | 7,713 |
| `ai-research` | `research/ai-research` | 378 MB | 3,337 |
| `archives` | `research/archives` | 103 MB | 129 |

They are not lost. Each is a tar in the MD5-verified store, indexed at
`icloud:UOR-R4/index/<name>.tsv`, and restorable in about a minute:

```sh
scripts/storage/research-data status                  # what is archived, what is present
scripts/storage/research-data restore prime-analysis --run
```

`restore` downloads the tar, verifies it against the index MD5, and extracts it in place.
The manifest with sizes and checksums is [docs/harness/research-data.json](../docs/harness/research-data.json).

One detail, stated because it is a real difference: the tar round trip normalises the
encoding of non-ASCII filenames. Verified before the local copies were removed, the
restored `prime-analysis` tree had the same 7,713 files and a byte-identical content
digest; two filenames in `research/theoretical-mathematics/prime-number-theory/` come back
with normalised Unicode. Contents are unaffected.

The bytes still exist in this repository's git history; they are simply no longer checked
out. This is a working-copy change, not a history rewrite, so it is reversible.
