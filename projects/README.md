# Side projects

This folder holds research projects that are related to UOR-R4 but are not part of the language model. Some feed ideas into the model (the model's prime and zeta-zero geometry draws on one of them). Others, like a network transport or a Lean proof of a matrix kernel, run alongside it. None of them is on the model's training or serving path, and none of them is evidence that the model works. The model itself is pre-alpha: 0 of 8 milestones are complete and no model holds a useful conversation yet (`STATUS.md`).

![Seven side projects shown as boxes around the central model. Four are marked as live source kept in git (spiralcore, riemann-lean, prime-analysis, wasm-gemm-gnaf), three as mostly archive (ai-research, research-archives, msa). Dashed arrows show where a project was consulted or drawn on by the model, and a dashed line to a private Hugging Face store shows where the bulky originals live.](../docs/figures/projects/projects-map.svg)

*The seven projects and where each connects to the model. Solid boxes keep their source in git. Dashed arrows into the model mean consulted or drawn on; the dashed arrow to the cylinder points to the large originals kept outside git.*

## Why these projects live in this repository

The owner decided (19 October 2026) that side projects stay in this repository under `projects/`, with one README each, rather than moving to separate repositories. Two reasons follow from that decision. First, the history of the model cannot be read without them: the model's prime-routing and zeta-phase ideas draw on the Prime Analysis program, and the SpiralCore references were consulted when the SpiralCore/FBS mechanism was designed (the mechanism itself follows a separate v63 reference and a hosted v69 file; see [the mechanism page](../docs/mechanisms/spiralcore-fbs.md)). Second, most of this material existed only on one laptop, with no version control and no remote. Putting the source in git and the bulky files in a checksummed store fixes that.

Three rules keep the side projects from being mistaken for model results:

1. **Nothing here changes the model's status.** A proof scaffold, a transport benchmark or a Python probe is not a trained capability. See the claim rules in [`docs/formal_vocabulary.md`](../docs/formal_vocabulary.md).
2. **Old claims keep their original scope.** Files imported from the owner's earlier workspaces carry labels such as "canonical" that applied to their own old repository, not to this one.
3. **Source goes to git, bulk goes to the store.** Code, Lean files, LaTeX and short notes come into the project folder. Images, PDFs, corpora and run outputs stay in the private store and are checked by hash.

Until 20 October 2026 the owner's laptop kept working copies of these projects in a gitignored `research/` folder at the top of the repository. That folder is retired. The source is here, and the full tree is in the private store under `research-archive/2026-10-19/`, which was checked file by file against the laptop copy on 20 October. The folder's old README is kept in [`archive/research-root-README.md`](../archive/research-root-README.md).

## The projects

| Folder | What it is | Where it stands | Model link |
|---|---|---|---|
| [`spiralcore/`](spiralcore/README.md) | Wire-transport exploration (plain zlib framing, no SpiralCore routing yet) plus two owner-supplied SpiralCore references | Wire transport works on loopback; not wired into the model | Related reference material, consulted but not the build source of the [SpiralCore/FBS mechanism](../docs/mechanisms/spiralcore-fbs.md) |
| [`riemann-lean/`](riemann-lean/README.md) | Lean 4 scaffold for a prime/Riemann theorem chain, plus the Python probes behind it | Scaffold. The Riemann hypothesis is not proved | None on the model path |
| [`prime-analysis/`](prime-analysis/README.md) | The Prime Phase / Chebyshev psi spectral program: notes, slides and simulations | Research notes. Not a proof of anything | A late, concluding collection of the prime-router studies that the [prime/zeta codebook](../docs/mechanisms/prime-zeta-codebook.md) draws on |
| [`ai-research/`](ai-research/README.md) | Snapshot of the original root research workspace (router research, MUDBench, Ramsey experiments) | Historical, Python era | Earlier workspace in the lineage of the Rust model |
| [`msa/`](msa/README.md) | Two papers on Modular Structural Arithmetic by Alex | Papers only, third-party authorship | Cited in earlier route-attention work |
| [`research-archives/`](research-archives/README.md) | The Gemini-era r3/r4 sandbox | Archive, not a live project | History only |
| [`wasm-gemm-gnaf/`](wasm-gemm-gnaf/README.md) | Lean 4 mechanization aimed at an optimality theorem for a WebAssembly matrix kernel | Theorem **not** established | None. Separate project |

## Where the originals are

Six of the seven projects came from the owner's untracked `research/` folder (the seventh, `wasm-gemm-gnaf`, is already tracked in git). On 19 October 2026 the whole folder was backed up to a private Hugging Face dataset before any file was moved.

| Item | Value | Source |
|---|---|---|
| Dataset | `caseyallard/uor-r4-store` (private) | `docs/labs/repo-renormalization-2026-10-19/backup/README.md` |
| Path | `research-archive/2026-10-19/` | same |
| Layout | eight tars under `tars/`: one per project (seven pages here, with `archives.tar` holding research-archives and `spiralcore-v68.tar` holding spiralcore), plus `causal-continuation-2026-09-24.tar` (a September 2026 causal-continuation experiment folder: scripts, plans and freeze files) and `root-files.tar` (the old `research/` README and two value-faith panel JSON files), which have no project page; `tars.sha256` and `files.sha256` sit one level above `tars/` | verified by listing the dataset on 19 October 2026 |
| Tar integrity | SHA-256 of each tar in `tars.sha256` | same |
| File integrity | SHA-256 of every file inside each tar in `files.sha256` (11,456 lines for all tars, including the loose files) | `research-archive/2026-10-19/files.sha256` in the store |

To restore one project, fetch its tar, check it against `tars.sha256`, unpack it, and check individual files against `files.sha256`. Each project README lists its own tar, size and checksum (`wasm-gemm-gnaf` has none: it is already in git). The folder sizes quoted on the project pages come from the audit inventory `docs/labs/repo-renormalization-2026-10-19/audit/folders.json`, which is an audit working file; `files.sha256` holds hashes only.

## What happens to the code

The plan for each project is the same: the source files named in that project's README are copied into its folder here, byte for byte, and the originals stay in the tar. Where the same file appears in two projects, one copy is kept and the other is recorded as a duplicate by hash (see [`prime-analysis/`](prime-analysis/README.md) and [`ai-research/`](ai-research/README.md)). Third-party and personal documents are not committed until the owner has reviewed their licensing or privacy. The folder inventory behind this plan is `docs/labs/repo-renormalization-2026-10-19/audit/folders.json` (research_projects).

## Compliance scope

Code under `projects/` is preserved research. The repository never runs it, and it is outside the model's compliance scope: the no-softmax, no-matmul and GPU-only rules ([D23](../docs/integration/DECISIONS.md), D24, D25) govern the model, not these side projects.


## Go deeper

- [Charter](../CHARTER.md) for what counts as the model and what does not.
- [Glossary](../docs/start/glossary.md) for terms such as SpiralCore, E8 and zeta zero.
- [Geometry overview](../docs/geometry/README.md), including [SpiralCore](../docs/geometry/spiralcore.md) and [primes and zeta](../docs/geometry/primes-and-zeta.md).
- Older records that mention these projects are in `archive/` (see the catalog under `docs/`).
