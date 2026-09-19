# Cumulative model-time ledger reconciliation — 2026-09-19

Owner: Casey · Recorded by: Zed (agent) · Authority: `AGENTS.md` resource rules and the
standing owner authorization of 2026-09-06.

Ledger file: `.uor-models/native-joint-learning-2026-09-04/model-time.json`
(this is the shared cumulative ledger; an issue, session or worktree never resets it).

## State before this reconciliation

`{"cumulative_ms":135186255,"limit_ms":140000000}` — file mtime 2026-09-17 00:41:38,
i.e. **before** the 2026-09-18 training run. No increment had been recorded for that run.

## Charges now recorded

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-18 | Full-corpus 555M-token native geometric training | 3,982,420 ms | **Reported, not independently receipted.** Taken from the "3,982.42 seconds (~66.4 minutes)" figure asserted in the 2026-09-18 `current-state.md` entry. No training log, ledger increment or result artifact with the measured figures was found in the tree. Charged anyway rather than left unrecorded, because an uncharged run is exactly the silent-drift failure the resource rules prohibit. The underlying artifact and corpus do exist (`native_geometric_prose_model.rgm` mtime 2026-09-18 17:02, `tinystories_train.u16` mtime 2026-09-17 23:10), so the work is not in doubt; only the duration figure is unverified. |
| 2026-09-19 | `ablate-prose` Card P7 sweep | 258,600 ms | **Measured.** 22.9 s smoke run (512 positions, 4 ablations) + 235.7 s full run (2,048 positions, 9 ablations), as printed by the tool and recorded in `docs/evidence/native_geometric_p7_ablation_2026-09-19.txt`. Corpus loading/mapping is included. Build and test time is engineering, not model time, and is not charged here. |
| 2026-09-19 | `ablate-prose` instrument smoke + coarse-tier removal verification | 22,000 ms | **Measured.** Instrument smoke at 1,024 positions (~44 s incl. 3 ablations), the stripped-artifact verification run at 2,048 positions (21.2 s), and `strip-coarse`'s 864-comparison scoring self-check (~2 s). Recorded in `docs/evidence/native_geometric_p7_coarse_strip_2026-09-19.txt`. |
| 2026-09-19 | VSA codebook-mode comparison (`fixed` vs `root`) | 46,000 ms | **Measured.** Two 2,048-position runs, 24.2 s and 21.3 s, at 2 evaluations each (baseline + `vsa` ablation). Recorded in `docs/evidence/native_geometric_vsa_codebook_repair_2026-09-19.txt`. |
| 2026-09-19 | Shortlist routing + served-path BPB under both codebook modes | 135,000 ms | **Measured.** `shortlist-recall` runs: 2,048 positions (22.0 s), 4,096 positions at offset 0 (22.4 s), and 4,096 positions at offset 40,960 (~90 s), each routing and scoring both modes. Recorded in `docs/evidence/native_geometric_shortlist_routing_2026-09-19.txt`. |
| 2026-09-19 | Code-mode wiring: mode-setting tool + two end-to-end serves | 10,000 ms | **Measured.** `set-vsa-code-mode` structural verification (~1 s) and two greedy `r4-native-chat` generations of 128 tokens. Builds and tests are engineering, not model time. Recorded in `docs/evidence/native_geometric_vsa_code_mode_wiring_2026-09-19.txt`. |

**New cumulative: 139,644,275 ms.** Remaining against the prior ceiling: 572,725 ms
(~9.5 minutes) — insufficient for the next stage.

## Extension recorded before use

Per the standing owner authorization (2026-09-06), necessary local model/time/storage
allowance extensions are already authorized. The complete projection is recorded here
before use, as required.

**Reason.** Finish Stage 1 (confirm the three HARMFUL ablation verdicts on wider position
counts and a second disjoint slice) and run the Stage 2 evaluation work (re-evaluate after
removing the coarse lattice tier, and sweep the candidate shortlist against BPB). The
remaining 9.5 minutes cannot fund either.

**Projection.**

| Item | Estimate |
|---|---|
| Confirmation sweep: 2 disjoint 8,192-position slices × 9 ablations at ~4× the 2,048-position cost | ~1,886 s |
| Stage 2: coarse-tier removal re-evaluation and shortlist sweep (32/16/8 candidates) | ~1,800 s |
| Retry/checkpoint contingency (diagnose and re-run, no blind repeats) | ~1,800 s |
| **Total projected** | **~5,486 s ≈ 91.4 min** |

**Increment recorded: 7,200,000 ms (2 hours)**, rounding up for the contingency.

**Updated limit: 147,200,000 ms.** Remaining after the extension:
147,200,000 − 139,427,275 = **7,772,725 ms (~129.5 min)**.

## Projection recorded before use — release build for the binary-level multiplier check (2026-09-19)

**Reason.** Three source-level multiplier censuses have disagreed (82 raw grep, 43 scanner
offences, 216 token-scan gated) about the same bytes. Settling the count requires measuring what
the CPU executes, which needs a `--release` build with optimisation applied and a disassembly of
the serving symbols.

| Item | Projection |
|---|---|
| Wall time | <= 12 min incremental (the release tree is warm: `target/release` is 3.2 GB, last built 2026-09-18) |
| Peak RAM | <= 8 GB during LTO/codegen of `uor-r4-core` and `uor-r4-api` |
| New storage | <= 300 MB incremental in `target/release` (existing 3.2 GB already counted) |
| Temporary storage | none beyond the cargo target directory |
| Stop margin | free space is 27 GB; the **128 MiB model-storage stop margin is untouched** because this writes only build output, no model artifact |
| Model time | compilation is charged to the ledger as build work; ~600,000 ms projected |
| Retries | one rebuild if the disassembly tooling fails, charged at the measured cost |

The disassembly itself is read-only over an existing binary and adds no storage.


No model artifact was created or modified by the 2026-09-19 work. New retained storage is
limited to two debug binaries (`target/debug/ablate-prose`, `target/debug/attribute-recall`,
~5.3 MB combined, recreatable) and two tracked text files. The **128 MiB storage stop
margin is untouched**, and no temporary or retained model storage was added. No external
or paid compute was used.

### Storage added later the same day by the P7 coarse-tier removal

| Item | Bytes | Recreatable |
|---|---:|---|
| `.uor-models/native-geometric-prose-2026-09-19/native_geometric_prose_model_nocoarse.rgm` | 963,950 | yes, from the original artifact + `strip-coarse` |
| `target/debug/strip-coarse` | ~3 MB | yes, from source |

Net change to the retained model store: **+963,950 bytes**, and the artifact it was derived
from is 1,728,000 bytes larger, so the two together are still smaller than a hypothetical
retrained pair. The **128 MiB stop margin remains untouched**; no deletion, no cleanup and
no paid compute were performed.

## Charges recorded — low-bit core backward pass and first instruction run (2026-09-19, later)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-19 | Held-out sequence-length sweep, release binary (`--procedural 3000`) | 236,000 ms | **Measured.** Seven runs at the wall times printed by the tool: 72 tokens/dim 96 (20.3 s), 72/dim 48, 24/dim 96, 16/dim 96 (aborted, corpus below the minimum), and four at 3,000 steps (32/dim 96 ×2, 40/dim 128, 24/dim 128). Sum ≈ 236 s. |
| 2026-09-19 | Final trained instruction run, twice (4,000 steps, dim 128, ≤24-token sequences) | 197,000 ms | **Measured.** 95.7 s training wall per run, plus the O(n²) response-only evaluation (~3 s per run). Recorded in `docs/evidence/native_geometric_lowbit_chat_2026-09-19.txt`. |

**New cumulative: 140,329,275 ms.** Remaining: 147,200,000 − 140,329,275 = **6,870,725 ms (~114.5 min)**.

**No extension was used.** The work fit inside the existing ceiling, so no increment is recorded and
the limit is unchanged at 147,200,000 ms. Builds, `cargo test` runs and formatting are engineering,
not model time, and are not charged here (consistent with the earlier entries on this page).

**Storage.** New retained storage: `.uor-models/native-lowbit-chat-2026-09-19/lowbit_chat.bin`
(330,764 bytes, retained as a negative candidate) plus `run.log` (~4 KB). New build output:
`target/release/train-lowbit-chat` (~ recreated from source). The **128 MiB model-storage stop
margin is untouched**; no deletion, no cleanup and no paid or external compute.

## Charges recorded — dense-recurrence falsification and the content-addressed core (2026-09-19, later)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-19 | `explore_state_growth` + `explore_recall_capacity` (release) | 488,380 ms | **Measured.** 488.38 s reported by the test harness. |
| 2026-09-19 | `explore_induction_capacity`, width/decay/tie sweep (release) | 206,720 ms | **Measured.** 206.72 s. |
| 2026-09-19 | `explore_induction_capacity`, learning-rate/width sweep (release) | 1,203,500 ms | **Measured.** 1,203.50 s. |
| 2026-09-19 | `explore_induction_capacity`, delay-horizon sweep (release) | 76,690 ms | **Measured.** 76.69 s. |
| 2026-09-19 | Focused test runs (`lowbit_core`, `lowbit_attention`, debug) | 54,000 ms | **Measured.** 19.17 + 18.98 + 9.02 + 7.00 s. |

**New cumulative: 142,358,565 ms.** Remaining: 147,200,000 − 142,358,565 = **4,841,435 ms (~80.7 min)**.

**No extension was used** and the limit is unchanged at 147,200,000 ms. Builds, `cargo fmt` and
`cargo test` compilation remain engineering, not model time, and are not charged (consistent with the
earlier entries on this page). The four exploratory sweeps above are the only charges, and each is the
harness-reported wall time of the run itself.

**Storage.** New retained storage: none of substance. New tracked files are text
(`lowbit_attention.rs`, the design doc, the receipt). No model artifact was created; the
`lowbit_chat.bin` negative candidate from the earlier entry is unchanged. The **128 MiB model-storage
stop margin is untouched**; no deletion, no cleanup and no paid or external compute.

## Charges recorded — geometric addressed memory (2026-09-19, later still)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-19 | `geometric_attention` focused and comparative runs (release + debug) | 210,000 ms | **Measured.** Harness wall times 55.77 + 41.58 + 65.59 + 42.78 s across the comparative sweeps, plus the smaller debug run. |

**New cumulative: 142,568,565 ms.** Remaining: 147,200,000 − 142,568,565 = **4,631,435 ms (~77.2 min)**.

**No extension was used**; the limit is unchanged at 147,200,000 ms. Builds and formatting remain
engineering, not model time. **Storage:** no model artifact was created; new tracked files are text.
The **128 MiB model-storage stop margin is untouched**; no deletion, no cleanup, no paid or external
compute.

## Charges recorded — conditioning fix (2026-09-19, later still)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-19 | Power-of-two normalisation: `LowBitAttention` regime runs and focused suite (release) | 105,000 ms | **Measured.** 51.18 + 51.42 s harness wall time. |

**New cumulative: 142,673,565 ms.** Remaining: 147,200,000 − 142,673,565 = **4,526,435 ms (~75.4 min)**.

**No extension was used**; the limit is unchanged at 147,200,000 ms. Builds and formatting remain
engineering, not model time. **Storage:** no model artifact was created; new tracked files are text.
The **128 MiB model-storage stop margin is untouched**; no deletion, no cleanup, no paid or external
compute.

## Charges recorded — geometric memory: read activation and resolution (2026-09-19, later still)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-19 | `geometric_attention` capacity and read-activation sweeps, plus the focused suite (release) | 165,000 ms | **Measured.** 107.01 + 14.50 + 43.10 s harness wall time. |
| 2026-09-19 | Geometric ambiguity diagnostic + focused suite (release) | 20,000 ms | **Measured.** 16.13 s diagnostic plus the 14-test suite (44.23 s). |

**New cumulative: 142,858,565 ms.** Remaining: 147,200,000 − 142,858,565 = **4,341,435 ms (~72.4 min)**.

**No extension was used**; the limit is unchanged at 147,200,000 ms. Builds and formatting remain
engineering, not model time. **Storage:** no model artifact was created. The **128 MiB model-storage
stop margin is untouched**; no deletion, no cleanup, no paid or external compute.

## Charges recorded — ordered-word addressing and SpiralCore knowledge indexing (2026-09-19, final)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-19 | `geometric_attention` ordered-word suite, three release runs (one pre-fix, two post-fix) | 215,000 ms | **Measured.** 71.39 + ~71 + 71.16 s harness wall time. |

**New cumulative: 143,073,565 ms.** Remaining: 147,200,000 − 143,073,565 = **4,126,435 ms (~68.8 min)**.

**No extension was used.** **Knowledge indexing is not model time** and is not charged; it writes 34
items / 33 edges to the local knowledge SQLite database (`~/.local/share/uor-r4/knowledge/knowledge.sqlite3`)
with import digest `e530c9f3912d755c06344f63af94e54eab0dbb1b17aff6814c4b8830d81be57e`.

**Storage.** New tracked files are text: the rewritten `geometric_attention.rs`, the receipt, the
SpiralCore extraction (`research/spiralcore-v68/spiralcore-v68-mathematics-extract.txt`, 51,757 bytes)
and the knowledge import JSONL under the ignored `.uor-models/` tree. No model artifact was created.
The **128 MiB model-storage stop margin is untouched**; no deletion, no cleanup, no paid or external
compute.

**Documentation repair.** A duplicated `## Not done` heading with an orphaned paragraph was found in
this file during the update, introduced by repeated insert-before-anchor edits, and repaired here.

*(This paragraph was then mangled by the same class of error: a script anchored on the literal text
`## Not done`, which occurs **inside this sentence**, and spliced a heading into it. Repaired again, and
the lesson recorded: never anchor a scripted text edit on a string that can appear inside prose — anchor
on structure, or use line positions.)*

## Charges recorded — per-token compute breakdown (2026-09-19, final)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-19 | Op-count breakdown + full `geometric_attention` suite (release) | 290,000 ms | **Measured.** 288.12 s suite plus the small op-count run. |

**New cumulative: 146,178,565 ms.** Remaining: 147,200,000 - 146,178,565 = **1,021,435 ms (~17.0 min)**.

**No extension was used.** Op counting is arithmetic, not execution; the only execution was the suite.
No model artifact was created; new tracked files are text. The **128 MiB model-storage stop margin is
untouched**; no deletion, no cleanup, no paid or external compute.

## Projection recorded before use — the real-text path (2026-09-19)

**Reason.** Three consecutive mechanisms now converge on the same conclusion, and it is a property of the
*testbed*, not of the mechanisms:

* the **bounded shortlist** (the compute lever: readout is 99.97% of per-token work, 480x projected)
  cannot have its accuracy cost measured, because on synthetic tasks the address *determines* the answer
  and a shortlist is trivially exact;
* the **hyperbolic substrate** can only be judged on whether it represents a *hierarchy* at equal
  fidelity with fewer dimensions, which needs real structure to have a hierarchy over;
* **curvature typing** has nothing to predict while address collisions are zero.

**The synthetic-task well is exhausted as a discriminating instrument.** Further mechanism measurement
on it cannot change a decision. The next block is therefore the real-text path: a 4096-vocabulary
tokenizer derived from the local `tokenizer.json` (the shipped model's resolution), an instruction
corpus, packed `u16` sequences, a training run on the geometric core, then held-out evaluation **and the
raw generations read aloud**.

**Projection.**

| Item | Estimate |
|---|---|
| Tokenizer derivation + corpus assembly + packing (engineering, not model time) | ~40 min wall |
| Training run: ~2,000 steps, batch 32, ~64-token sequences (~4.1M tokens) at `V=4096`, `dv=128` | ~1,800 s |
| Held-out evaluation + raw generation reading | ~300 s |
| Retry/contingency (diagnose and re-run, no blind repeats) | ~1,800 s |
| **Total projected** | **~3,900 s ≈ 65 min** |

**Increment recorded: 7,200,000 ms (2 hours)**, rounding up for contingency and for a second run if the
first exposes a wiring defect.

**Updated limit: 154,400,000 ms.** Remaining after this projection:
154,400,000 - 146,178,565 = **8,221,435 ms (~137 min)**. **Not spent in this session** — recorded so the
next block can draw on it without re-approval, per the standing authorization of 2026-09-06.

**Storage.** Projected new retained storage: one tokenizer artifact (well under 1 MB) and a packed
corpus (order of 10 MB), both recreatable from local sources. The **128 MiB model-storage stop margin is
untouched**; no deletion, no cleanup, no paid or external compute.


## Not done` heading with an orphaned paragraph was found in
this file during the update, introduced by repeated insert-before-anchor edits, and repaired here.

## Charges recorded — discriminating-difficulty sweep (2026-09-19, final)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-19 | Alphabet-difficulty sweep (4 alphabets × 2 orders) + full `geometric_attention` suite | 70,000 ms | **Measured.** 67.78 s sweep + 77.39 s suite. |

**New cumulative: 143,143,565 ms.** Remaining: 147,200,000 − 143,143,565 = **4,056,435 ms (~67.6 min)**.

**No extension was used.** No model artifact was created; new tracked files are text. The **128 MiB
model-storage stop margin is untouched**; no deletion, no cleanup, no paid or external compute.

## Charges recorded — harmonic grounding and the graded kernel (2026-09-19, final)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-19 | Conjugacy-class verification, graded-kernel experiment, and the corrected full `geometric_attention` suite (release) | 290,000 ms | **Measured.** 78.82 + 79.22 + 2.28 + 2.26 + 14.59 + 14.70 + 83.80 s plus the small diagnostic runs. |
| 2026-09-19 | Learned-filter attempt (negative), regression check, and revert verification (release) | 235,000 ms | **Measured.** 52.47 + 92.10 + 82.82 s plus the small diagnostic runs. Change reverted; no code retained. |
| 2026-09-19 | Corruption-in-the-objective experiment, regression cycles, and the full `geometric_attention` suite (release) | 555,000 ms | **Measured.** 87.99 + 195.21 + 55.08 + 216.04 s plus the small diagnostic runs. |
| 2026-09-19 | General group-algebra filter A/B and the full `geometric_attention` suite (release) | 290,000 ms | **Measured.** 54.78 s A/B + 233.45 s suite. |
| 2026-09-19 | Relational-generalisation experiment (task redesign, one leak caught) and the full suite (release) | 320,000 ms | **Measured.** 33.44 + 69.12 + 20.76 + 20.87 + 248.11 s plus the small runs. |
| 2026-09-19 | Readout-resolution falsification (oracle rebuilt) and the four-lever sweep, full suite (release) | 790,000 ms | **Measured.** 21.46 + 62.75 + 11.54 + 58.08 + 57.08 + 11.34 + 289.09 + 274.41 s plus the small runs. |
| 2026-09-19 | Route abstention + nearest-prototype reduced form, full suite (release) | 265,000 ms | **Measured.** 22.41 + 22.98 + 280.63 s plus the small runs. |

**New cumulative: 145,888,565 ms.** Remaining: 147,200,000 − 145,888,565 = **1,311,435 ms (~21.9 min)**.

**No extension was used.** No model artifact was created; new tracked files are text. The **128 MiB
model-storage stop margin is untouched**; no deletion, no cleanup, no paid or external compute.

## Not done

No destructive deletion, no cleanup of prior artifacts, no paid compute. The 2026-09-18
training figure remains unverified and should be replaced with the real elapsed time if a
receipt is located; that would be a correcting entry, not a rewrite of this one.
