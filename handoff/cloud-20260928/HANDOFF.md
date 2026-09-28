# Cloud lab track handoff, 2026-09-28

> **Integration note (Lab 1, director).**
> - This file moved here from the repository root when the handoff merged into `main`.
> - The 131 files under `sandbox/exp/math/pylib/` are vendored third-party Python packages (`autograd`, `mpmath`) and were left out of `main`. They remain on the `claude/cloud-handoff-20260928` branch.
> - The D1 models are on the SSD at `uor-r4-models/investigations/d1-transport-attribution-20260928/`: 24/24 SHA-256 verified, with a receipt. Their transfer branch is deleted.
> - S1 moved to Lab 1, locally (owner direction).

**From:** the cloud lab track (Claude). **To:** Lab 1 (the main session), which takes this work over locally.

**Stand-down.** The owner stopped this track. No run was started after D1, and **S1 was not started**.

The owner assigned S1 at 16:08 UTC. ROADMAP marks S1 "in progress (cloud track)", but no S1 code or run exists anywhere. Please correct that row when you take it.

## Branches

| Branch | Commit | What |
|---|---|---|
| `transfer/d1-models-20260928` | `93cd1f13` | Orphan branch, no PR, 108 MB, 24 files plus `SHA256SUMS`. It holds the four D1 models (`rot_s1`, `id_s1`, `rot_s2`, `id_s2`), each with `model/`, `report.json`, `attempt.json` and `manifest.json`, plus `eval/`. There are no checkpoints: each run finished in one launch and deleted its own. Delete the branch after the copy is verified. |
| `claude/cloud-handoff-20260928` | this branch | `main` plus this file, plus `handoff/cloud-20260928/sandbox/`. That directory holds 421 text files (4.95 MB) copied from the sandbox; see its section below. |

## Work items and their state

| Item | State | Where |
|---|---|---|
| Cycle-4 main comparison (D0) | **Done.** Merged in #1437. The director decided D0: the `rrarra` stack is the main-line core. | Cycle-4 note §6 and packet `main/`. The weights are on the SSD, copied from `transfer/cycle4-main-20260928`, which has since been deleted. |
| D1, transport attribution | **Done.** Merged in #1460. The quaternion transport is kept: +0.0711 and +0.0774 nats at MLP 749, over 2 seeds. | `docs/integration/transport-attribution-d1-2026-09-28.md` and its packet. The models are on the transfer branch. |
| `stack_mlp=` option | **Merged** (#1437, `4eef03e6`) | The `geometric-stack` example |
| Cycle 5, product-key memory with H4/E8 codebooks | Implemented, tested and merged (#1437). **NOT_RUN and held.** Its base must be `rrarra`, and §4 is amended before any arm runs. | `docs/integration/memory-layers-cycle5-2026-09-27.md` and `stack_memory.rs` |
| S1, the stack's D11 serving port | **Not started.** | ROADMAP §2 and §4.3 |
| Local git state | **Nothing unpushed.** `cycle5-followup` (`97c8b6ea`), `cycle5-memory-wip` (`2a5decad`), `read-kernel-wip` (`520cc957`) and `claude/blissful-wozniak-girwwq` (`4eef03e6`, squash-merged) all exist on GitHub refs. | none |
| Rebuilding the D0 executable from `b87acd63` | Cargo adds `"uor-r4-tokenizer"` to `uor-r4-training`'s dependencies in `Cargo.lock`, because `b87acd63`'s lock predates it. The D0 executable (`0b3b94dc…`, built with `-C target-cpu=x86-64-v3`) was built with that one-line lock change. | none |

## What I would do next (S1)

1. **Measure the snap loss first.** Evaluate the D1 `rot_s1` and `rot_s2` models with `u_t` snapped to the 120 unit icosians, and with a table-based unit normalisation. This is evaluation only: no training, minutes of CPU. It shows whether the transport can be served as a Cayley-table read, before any kernel work.
2. **Write the I2 `stack` profile writer and loader**, following `stack_export.rs` and the grid reference. Group-scaled ≤4-bit maps and integer biases, plus:
   - the gate map's rotation rows;
   - the per-lane decay gates;
   - one quaternion product per lane, through product tables or the icosian Cayley table.

   Then check reload equality and fidelity (≤ 0.02 nats against float), and pass the ARM64 audit on the M1.
3. Treat these as optional:
   - a parameter-matched real-gate control, which would separate the rotation from the extra gate rows (needed only if a cheaper serving design is on the table);
   - cycle 5 on the `rrarra` base, when released.

**Reproduction notes.**
- **Numerics are host-specific.** The matrix library's blocking depends on the CPU caches, so a rerun on another CPU follows a different floating-point path. Paired arms must share a host.
- **Sandbox builds used `-C target-cpu=x86-64-v3`.** A host-tuned binary trapped after the sandbox moved to another CPU.
- **The runners are idempotent.** They resume from the newest root that holds a checkpoint. `checkpoint_every` is not part of the lineage.

## Sandbox-only items: not pushed, and lost when the sandbox is reclaimed

Paths are relative to `/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad`.

**Worth keeping**, most important first:

| Path | Size | What | Rebuildable elsewhere? |
|---|---:|---|---|
| `c4/data/registry.u16` | 56 MB | D0's second training stream: lab-BPE tokens of the sandbox's Cargo registry, SHA-256 `af93ca73…` | **No.** It was built from this sandbox's Cargo registry. |
| `c4/data/registry.txt` | 70 MB | Its source text, SHA-256 `cc96831b…` | **No**, for the same reason |
| `c3/data/code/` | 15 MB | The repository code split used by the cycle-4 ablations and D1: `train.u16` `b12707b0…`, plus `valid`, `lens`, `merges`, `stats.json` and `valid_depth.npy`. `valid`, `lens` and `merges` are also on both transfer branches. | Probably, from the repository with the lab BPE, but not verified bit-identical |
| `c3/data/wiki/` | 6.5 MB | Cycle-3 wiki split | Probably |
| `c4/runs/pilot` | 165 MB | Six learning-rate pilot models: stack and control at 1e-3, 2e-3 and 4e-3, 1,000 updates, on the earlier host | Approximately (other host) |
| `c4/runs/ablation` | 83 MB | The `dot`, `norot` and `readsonly` models at 1,000 updates, on the earlier host | Approximately |
| `c4/runs/main` | 219 MB | D0's final roots and step-250 checkpoints. The final models are already on the SSD. | Not needed |
| `c4/ckpt-keep` | 164 MB | D0's step-7,300 checkpoints, already on the SSD | Not needed |
| `c4/resume-v3` | 164 MB | The measured resume-equivalence check (model `7e94e455…`) | Yes, on this host |
| `c4/transfer/cycle4-20260927` | 180 MB | The 09-27 staging of the pilot models and executables | Not needed |
| `c4/runs/speed`, `c4/runs/resume-check` | 55 MB, 11 MB | Throughput runs and the first resume check | Yes |
| `c4/bin` | 24 MB | Executables: `b87acd63-v3` (D0, `0b3b94dc…`), `b87acd63` host-tuned (`05889874…`), `dca1b790` (pilot, `fcf710ad…`) | Rebuild from source |
| `d1/bin` | 9.9 MB | The D1 executable `4eef03e6-v3` (`c90c7346…`) | Rebuild from source |
| `d1/runs` | 107 MB | D1 roots and training logs. The models are on the transfer branch, and the records are on `main`. | Not needed |
| `c5/bin`, `c5/smoke-mem` | 9.8 MB, 89 MB | The cycle-5 executable (`97c8b6ea-v3`, `5796e15e…`) and memory smoke runs | Yes |
| `c3/runs`, `c3/bin_*` (13 files), `c3/analysis` | 140 MB, 87 MB, 7.1 MB | Cycle-3 run roots (2.8 MB models), executables and analysis | Partly |
| `lab/exp` | 597 MB | Lab phase-2 experiment directories. Their code and notes are copied to this branch. | Partly |
| `exp` | 122 MB | Wave-1 review experiment directories, including `lead` (65 MB), `audit` (23 MB) and `arch` (16 MB). Their code and notes are copied to this branch. | Partly |

The `lab/exp` directories are:
- `small_llama`: the teacher stand-in, with its caches and checkpoints;
- `lead_ctx256`;
- `lead_bench`, including `perf.data` and GEMV kernels;
- `c3b` fine-tunes;
- `exp2`, `math2`, `resume_check` and `lead_conv`.

**Not worth keeping** (regenerable or superseded):
- `target`, `target-aarch64` and `target-v3`: about 19.7 GB of build caches;
- `leadlib` and `pylib`: 812 MB of Python packages;
- `wt-b87`, `wt-read` and `wt-cycle5`: 1.5 GB of worktrees at commits already on GitHub;
- `base-9df1afab`: a 52 MB source snapshot;
- `replay5`: 135 MB, the recorded-edit replay that rebuilt #1437's branch after the rollback. The merge superseded it;
- `d1/smoke` (54 MB) and `c4/smoke-v3-*` (56 MB): smoke runs;
- the top-level build and test logs.

## `handoff/cloud-20260928/sandbox/`

This directory holds 421 text files (4.95 MB), under the same relative paths as the scratchpad. 46 other candidates were byte-identical to files already on `main` and are omitted. No credential-like strings were found.

| Path | What |
|---|---|
| `reports/`, `lab/reports/` | Wave-1 specialist reports and lab phase-2 reports |
| `CONTEXT.md`, `REDTEAM_BRIEF.md`, `lab/LAB_BRIEF.md`, `lab/REDTEAM2_BRIEF.md` | Team briefings |
| `draft/`, `lab/draft/` | Document drafts. The delivered versions are on `main`. |
| `exp/`, `lab/exp/` | Experiment code and notes |
| `c4/*.sh`, `c4/*.py`, `d1/`, `build_packet.py` | Run launchers, chains, resume logic and packet builders |
| `ablate/`, `c3abl/`, `rg-work/`, `cache_*.rs`, `patch_cache_export.py` | Ablation crates, the read-geometry probe and cache patches |
| `pr1437-body.md` | The final description of #1437 |
