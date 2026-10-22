# REPORT: m1-a8-erosion-diagnosis, claude-39 (the census tool and its five controls)

Lab: claude. Base: `ba7222b1b`. Order: [ORDER.md](ORDER.md), committed first (`44a3f2a4a`). The cancel probe at the base found only `DESIGN.md` under `docs/labs/m1-a8-erosion-diagnosis/`: no census crate and no result. So the task stands.

## What did not work, first

- **The attribution correction's interval count is wrong.** It says "all 66 64-batch intervals (0→4,096)". Both arms hold 65 contiguous checkpoints, 0 to 4,096, which is 64 intervals. Its join claim still holds on all 64 (control 4).
- **The placement data cannot explain Relation's extra loss at the whole-vocabulary level.** FrozenZero moved more tokens' cells (91 against 80) and made more cell-changing writes (103 against 91), yet lost less reach. No cell was emptied or filled in either arm, so the occupied-set route is absent. This narrows the question; it does not answer it. The target and predecessor sets are still uncounted.
- **The first download fetched only one arm.** Given several `--include` patterns, `hf download` treated the later ones as file names and ignored `--include`. I fetched the Relation folder again on its own, and the binary re-verifies every file against its run's `FILES.sha256` anyway.
- **`geometry::ROOTS` is private to `uor-r4-discrete`.** The builder took the roots from the public `normalization_mechanism::orbit_root_coords`. The comparison code is the census's own, and its planted fault is caught (below).
- **The builder handed back at its context limit** after the crate commit. I ran the real run and wrote the results README myself.

## What changed

| Commit | What |
|---|---|
| `44a3f2a4a` | This ORDER, alone. |
| `b3bcbb650` | `docs/labs/m1-a8-erosion-diagnosis/census/`: a standalone crate detached from the workspace, with a library (cell rule with ABSENT, cells, changed slots, words digest) and the `census` binary. 8 unit tests on synthetic tables. |
| `c0cf7d539` | `docs/labs/m1-a8-erosion-diagnosis/README.md`: the run command, the output sha256, the five controls, Q1 and Q3 tables, and what is and is not supported. |
| this REPORT's commit | This REPORT. |

## Measurements

From `census.json`, sha256 `8f8555e28df3cf42047a3e075e4bdb73899d7fb6ee21b444e42092582dfacc4a`, run on the laptop host-only in about 56 s.

**Controls:**
1. Checkpoint-0 tables: 0 of 131,072 words differ across the arms.
2. The digest is `4122cbab…` in both arms.
3. 256 occupied cells, max occupancy 16, and 0 of 4,096 host cells differ from the GPU receipt.
4. 0 join mismatches over 64 intervals per arm.
5. 64 intervals, not 66.

**Q3, to Q1:**

| Arm | Block 0/3 writes | Tokens whose cell changed | Applied gate writes |
|---|---|---|---|
| Relation | 91 | 80 | 204 (87 of them relation) |
| FrozenZero | 103 | 91 | 120 (0 of them relation) |

**Q1:** all 256 cells stay occupied at every checkpoint in both arms, and max occupancy is 20 at Q1.

## Checks run at the head

- **Crate:** `cargo test --locked`: 8 passed. `cargo clippy --locked --all-targets -D warnings`: exit 0 (the only warnings printed are pre-existing ones in path dependencies).
- **Planted fault** (the builder's run): with the cell rule swapped to `last·16 + first`, 2 of 8 tests fail. Restored, 8 pass.
- **Real run:** controls 1–4 PASS. Control 5 records 64 intervals and `claim_66_holds: false`.
- **Claim-wording** on the README: passed.
- `loop report --check` and `loop pr-check --local`: see the PR body.

## Skills used

scope-guard: the census crate and the results README under `docs/labs/m1-a8-erosion-diagnosis/`. The 886 MB of inputs stay in gitignored `local/`.

## Still open

- **Q2, Q4 (i) and (ii), and Q5** need the P_succ target tokens and their TRAIN predecessors. That is the next unit, and it ends in the notebook row.
- **The downloaded inputs** (`local/claude/a8-census-data/`, 886 MB) are kept for that unit and deleted after it.

## Plain summary

The saved snapshots show that the training arm which lost less reach actually moved slightly more words to new places. Across the whole vocabulary, then, moving words around does not explain why the learning arm lost more. The next step checks whether the specific words the test depends on were the ones that moved.
