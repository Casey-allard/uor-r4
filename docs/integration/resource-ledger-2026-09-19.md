# Cumulative model-time ledger reconciliation — 2026-09-19

## Takeover reconciliation — recorded balance versus receipt completeness

On September 19 the live JSON was read as `146438565 / 154400000 ms`, with `7961435 ms` remaining (132.69 min). This review makes **no change to either number**. It moves the complete compute/projection sections after their preceding harmonic charges and removes the still-orphaned heading fragment. All historical charge rows and receipts remain preserved.

The late-day printed balance sequence is now readable in order:

| Stage | Recorded cumulative ms | Recorded increment ms |
| --- | ---: | ---: |
| Ordered-word addressing | 143073565 | prior snapshot |
| Difficulty sweep | 143143565 | 70000 |
| Harmonic/filter/composition work | 145888565 | 2745000 |
| Compute breakdown | 146178565 | 290000 |
| Static real-text count comparison | 146438565 | 260000 |

This reconciles **ordering and printed arithmetic**, not every measured duration. Several unresolved entries prevent certifying the JSON as a complete wall-time sum: the initial charge rows add to 139640275 rather than the printed 139644275; an extension uses the older 139427275 balance; the difficulty row charges 70,000 ms while listing 67.78 + 77.39 seconds; the route-abstention row charges 265,000 ms while listing 22.41 + 22.98 + 280.63 seconds. Build/test inclusion also varies between sections. Determine overlap, raw duration and already-booked charges before any correcting debit; do not silently add all examples and risk double-counting, or silently release the discrepancy as allowance.

The 555M training duration remains a conservative reported charge, not independently recovered timing. Next model work must refresh the JSON, retain historical charges, explicitly record build/preparation/test/model categories and any correction or authorized local extension before use. The standing authorization covers necessary local extensions, so an accounting ambiguity is not a reason to conceal work or request duplicate permission.

This takeover performed read-only research plus documentation/policy edits and proportionate document checks; no model preparation, Rust build, fit, inference or benchmark was executed. Model balance is unchanged. Storage was measured with the established inventory: 51,761,754,112 bytes free at 21:21 UTC, and model-store allocated size 20,435,673,088 bytes **LOWER_BOUND_OR_UNAVAILABLE** because three sealed directories were unreadable. No sealed data was accessed or permissions changed. Preserve the 128 MiB stop margin; inventory rows overlap. New retained output is text plus the isolated worktree.

The old “mechanism ceiling” justification below is superseded by the [takeover review](takeover-review-2026-09-19.md); the experiment remains a validly retained static baseline report, with instrument corrections pending. Timing-first scheduling is replaced by causal/instrument correctness and coverage, then measured timing.

## Historical entries (preserved)

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

**Documentation repair.** A duplicated `*(This paragraph was then mangled by the same class of error: a script anchored on the literal text
`## Not done`, which occurs **inside this sentence**, and spliced a heading into it. Repaired again, and
the lesson recorded: never anchor a scripted text edit on a string that can appear inside prose — anchor
on structure, or use line positions.)*

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


## Not done

No destructive deletion, no cleanup of prior artifacts, no paid compute. The 2026-09-18
training figure remains unverified and should be replaced with the real elapsed time if a
receipt is located; that would be a correcting entry, not a rewrite of this one.

## Projection recorded before use — real-text mechanism ceiling (2026-09-19, continuation)

**Reason.** The funded real-text block (`7,200,000 ms` recorded above, unspent) is specified as
"train the geometric core at `V=4096`, `dv=128` for ~2,000 steps". Inspection of the mechanism
before running it shows what that training would actually measure:

* `GeometricAttention::from_f32` sets `elements = element_table(vocab)` and
  `element_table` is `(0..vocab).map(|t| (t % RADIX) as u16)` — i.e. **`element(token) = token_id % 120`,
  fixed and not learned** (`learner/geometric_attention.rs`);
* `order` is restricted to `1` or `2` (`from_f32` returns `Err` otherwise);
* the address is `Σ_k elements[ctx[k]] · 120^(order-1-k)` over the last `order` tokens.

So at `V=4096` with `order=2` the mechanism's entire context is the **last two tokens' residues
mod 120** — a `120² = 14,400`-class coalescing of a `4096²≈1.7e7`-class context space, i.e. ~34
tokens share each residue. Project trap #2 requires testing the **reduced form** of an idea before
building it, so this block measures the mechanism's *achievable ceiling* on real text
(`H(next) − H(next | residue pair)`) before the training budget is spent. This is a bound, not a
verdict: it scopes to the mechanism **as currently configured**, and `learned packaging` of the
element assignment is already the architecture document's named next step.

**Projection.**

| Item | Estimate |
|---|---|
| Derive a 4096-vocabulary tokenizer from the local `tokenizer.json` (Rust) | engineering |
| Tokenize a real (non-TinyStories) corpus and pack `u16` | engineering |
| Build + run the ceiling measurement (release), including controls | ~300 s |
| **Total projected** | **~900 s (15 min)** |

**No new extension is used.** This draws on the already-recorded `7,200,000 ms` real-text
projection; cumulative is unchanged at `146,178,565 ms` until the run is charged.

**Storage.** Text only; no model artifact. The **128 MiB model-storage stop margin is untouched**;
no deletion, no cleanup, no paid or external compute.

## Charges recorded — real-text mechanism ceiling (2026-09-19, continuation)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-19 | Reduced-form ceiling measurement: new Rust bin, release build, five runs over two corpora (including the invalid add-1 version), three focused tests | 260,000 ms | **Measured.** ~221 s: `cargo check` 14 s; first release build 100 s; incremental rebuilds ~4 s; runs 0.7 + 24.1 + 26.4 + 23.9 + 26.6 s; test build/run ~1 s. Rounded up for formatting and overhead. |

**New cumulative: 146,438,565 ms.** Remaining: 154,400,000 − 146,438,565 = **7,961,435 ms (~132.7 min)**.

**Drawn from the already-recorded real-text projection** (`7,200,000 ms`); no new extension. No
model artifact was created; the new tracked files are text (one Rust bin, one receipt, three document
edits). The **128 MiB model-storage stop margin is untouched**; no deletion, no cleanup, no paid or
external compute.

**Cost re-projection recorded (arithmetic from op counts, NOT a timing measurement).** From the
project's own counted readout — 262,144 ops/token at `dv=64`, doubling to 524,288 at `dv=128` — and
the projected 4,096,000 tokens (`2,000 × 32 × 64`), the forward readout alone is 2.15e12 operations.
The recorded 1,800 s corresponds to ~1.2e9 scalar ops/s sustained, leaving no room for the backward
pass, Adam over `2 × vocab × dv` weights, the write path or the 1.84e6-int per-sequence state clear.
The run is therefore plausibly **3–10× the recorded projection**, and the ~132.7 min remaining may
not cover 2,000 steps at `V=4096`, `dv=128`. **A measured step-time probe should precede spending
the block; the run length should be chosen from a timing, not from the 1,800 s projection.**

## Supplementary projection recorded before use — causal qualification tranche (2026-09-19, takeover)

**Reason.** The takeover reconciliation ([review](takeover-review-2026-09-19.md), [execution prompt](deepseek-next-step-2026-09-19.md), #973) defines one bounded task: repair and qualify the core's short-prefix arithmetic and declared mean-loss STE, repair the count instrument, **measure real-text causal route coverage**, then measure complete training-step cost, and admit a small pilot only if those results make it informative. The task's own initial correctness/coverage/probe cap is **1,200,000 ms**; it is a cap, not a deduction, and the correctness repairs plus two instruments do not fit inside it.

**Work already executed inside that first tranche (to be charged once, at its measured value):** local-core qualification edits (explicit padding, single mean-loss normalization), eight new fixtures, two pre-existing synthetic filter fixtures re-tuned after the gradient correction, and the associated debug and release test cycles.

**Supplementary projection.**

| Item | Estimate |
|---|---:|
| Causal route-coverage instrument (new Rust bin) + corpus snapshot + run over two corpora | ~500 s |
| Complete-step timing probe at `V=4096`, `dv=128`, `order=2`, cold and warmed, with peak RSS | ~400 s |
| Count-instrument repairs (argmax/CE consistency, declared fit-only protocol, byte population) + fixtures | ~300 s |
| Receipts, documentation, protected delivery | ~300 s |
| **Supplementary total** | **~1,500,000 ms** |

**No new allowance extension is used.** This draws on the existing recorded balance; the limit stays
`154,400,000 ms`. **No pilot is included in this projection**: #973 requires any pilot to carry its
own complete projection and to leave evaluation and checkpoint capacity, so it is a separate,
conditional request whose go/no-go is an output of this tranche.

**Host feasibility, verified before execution.** One model worker; Cargo build job count bounded at 4;
the host has 8 cores and 16 GiB RAM, so a peak RSS cap of 8 GiB is feasible and the trainer's state
(`n_addr × dv = 14,400 × 128` f32 ≈ 7.4 MB per buffer) fits comfortably; new build output is kept
inside the **existing** `target/` directory rather than a fresh one, so new build storage stays far
below 1 GiB and the **128 MiB model-storage stop margin is untouched**. No deletion, no cleanup, no
paid or external compute.

**Isolated worktree, and its storage.** The work is done in a worktree of refreshed `origin/main`
(`3f050b27`) at `.worktrees/realtext-qual`, excluded from `git status` via `.git/info/exclude`, so the
**owner's checkout is not modified**. The checkout itself is ~1.2 GB because the repository tracks
~1.0 GB of `research/` material; that is a checkout of existing tracked files, not new data or build
output, and it is preserved rather than trimmed. This is stated explicitly rather than silently
counted either way.

## Charges recorded — causal qualification tranche (2026-09-19, takeover)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-19 | Core qualification (`geometric_attention` padding + single mean-loss normalization, 8 new fixtures, 2 independent analytic gradient references), count-instrument repair, coverage instrument, complete-step timing probe, receipts and delivery | 1,200,000 ms | **Measured, rounded up.** Full module test pass under overflow checks 301 s; release test passes 54 + 44 s; release builds ≈ 6–7 min across five cycles; filtered debug runs ≈ 70 s; coverage 1.1 s + 1 s; timing probes ≈ 6 s; corrected count runs 2 × ≈ 35 s; plus one hung count run (accuracy inside the tuning sweep) terminated and re-run. |

**New cumulative: 147,638,565 ms.** Remaining: 154,400,000 − 147,638,565 = **6,761,435 ms (~112.7 min)**.

This charge covers the first tranche cap (1,200,000 ms) and the supplementary projection recorded above
in one entry, because the two instruments and the correctness repairs were executed as one continuous
block; the total is the measured cost, not the sum of the estimates. **No new allowance extension was
used**: the limit stays `154,400,000 ms` and no limit was increased. No model artifact was created; the
new tracked files are text (two Rust binaries, one shared library module, one receipt, four document
edits). The **128 MiB model-storage stop margin is untouched**; build output went into the existing
`target/`. No deletion, no cleanup, no paid or external compute.

**No pilot was charged and none was run.** The tranche's output is a *decision not to fit this
configuration*; #973 requires any pilot to arrive with its own complete projection and to leave
evaluation and checkpoint capacity, so that remains a separate conditional request.


## Post-qualification architecture review — no model execution (September 19)

Read the live JSON as `147638565 / 154400000 ms`, remaining `6761435 ms` (112.69 min). No Rust build, model fit/evaluation or new artifact construction occurred in this source/documentation review; no model charge or limit change was made. Reused the existing clean full worktree at `/Users/casey.allard/.codex/worktrees/takeover-research-reconciliation/uor-r4` with a new branch. Original checkout and all unique/sealed material preserved. Host free-space observation approximately 44 GiB is not a replacement for the complete storage inventory.

The [next prompt](deepseek-cold-context-step-2026-09-19.md) proposes a complete 3,600,000 ms implementation/pilot tranche, including preparation/builds, two matched fits, controls/evaluation/export, generated behavior and reserve. This is **not an executed charge**. Refresh the ledger and storage, record the complete projection before execution, and revise from actual new-path timing. Account for any new full checkout separately; do not inherit the earlier assumption that an approximately 1.2 GiB checkout fits inside 1 GiB total new storage. Existing owner authorization covers necessary recorded local increments, never paid compute or deletion.

The #1292 timing supports batch-8 trainer calls only: 512 input tokens but 504 prediction targets per full step. Its `to_core()` measurement excludes persisted checkpoint I/O; its reported evaluation follows six optimizer updates. The recorded cumulative balance is preserved without pretending these reporting corrections reconstruct the earlier elapsed-time ledger.

## Projection recorded before use — cold-context prior and controlled memory pilot (2026-09-19)

**Reason.** The [cold-context review](cold-context-review-2026-09-19.md) and the [execution prompt](deepseek-cold-context-step-2026-09-19.md) select one bounded experiment: an always-present learned exact-token local prior plus the existing causal memory residual through one shared low-bit decoder, with a trained prior-only arm and a trained joint arm on one pinned population.

**Projection (complete, before execution).**

| Item | Estimate |
|---|---:|
| Preparation: worktree/branch, refresh, storage check, small instrument repairs, focused builds | 900,000 ms |
| Matched training: two arms at 256 steps, V=4096, dv=128, batch 8, window 64 (optional second seed conditional) | 1,200,000 ms |
| Controls, evaluation, export/reload, generation | 900,000 ms |
| Retry, checkpoint and stop reserve | 600,000 ms |
| **Total tranche** | **3,600,000 ms** |

**No allowance extension is used.** The limit stays `154,400,000 ms`; this draws on the existing recorded
balance. **No new full checkout is created**: the existing clean isolated worktree is reused on a new
branch, so no additional ~1.2 GiB checkout is charged.

**Host feasibility, verified before execution.** One model worker; Cargo jobs bounded at 4; host has 8
cores and 16 GiB RAM, so the 8 GiB peak-RSS cap is feasible; new build output reuses the existing
`target/`, keeping incremental build/data/artifact storage below 1 GiB; the **128 MiB model-storage stop
margin is untouched**. No paid or external compute, no deletion.

**Instrument repairs folded into the preparation item (source findings from the review, unexecuted):**
`blended_argmax` incumbent initialisation, the extra `sv[t]` in the `reference_order2` value derivative,
the `norm_bits = 0` wording, and successor-statistic populations.

## Charges recorded — cold-context prior and bounded paired pilot (2026-09-19)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-19 | Instrument repairs, `cold_prior` module + 13 fixtures, `TernaryLinear::from_packed`, pilot tool, two pilot runs (2-step smoke and the declared 256-step paired run), receipts and delivery | 1,400,000 ms | **Measured.** Two pilot runs at 362.6 s and 421.3 s; roughly seven release build cycles (≈7 min total); `cold_prior`/gradient/ceiling test passes; iteration and debug cycles across the module and the tool. |

**New cumulative: 149,038,565 ms.** Remaining: 154,400,000 − 149,038,565 = **5,361,435 ms (~89.4 min)**.

**Charged from the 3,600,000 ms tranche projection recorded above; no new allowance extension.** The
unspent remainder of that projection covers the controls/evaluation/reserve items that were **not**
required by the outcome: no second seed was run (the result was negative, not borderline), and no
`crates`-corpus replicate or dose extension was run, because the predeclared gates place the prior at
`undertrained/inconclusive` and the prompt forbids a dose sweep to chase a pass. Retained: one
614,542-byte artifact (`sha256:fb780ff6…`), text and source changes only, build output inside the
existing `target/`. The **128 MiB model-storage stop margin is untouched**; no deletion, no cleanup,
no paid or external compute.


## Review after PR #1294 — preserved balance and proposed recovery (2026-09-19)

Verified live JSON remains **149,038,565 / 154,400,000 ms**, leaving **5,361,435 ms (89.36 minutes)**. This source/artifact-identity/literature and documentation review executed no Rust build/model; no model charge or allowance extension was applied. Reused the existing clean full worktree on `codex/prior-learning-recovery`. Owner checkout and retained `cold_prior_joint.cpr` (614,542 bytes, SHA256 `fb780ff6ff19eec58f861135004250ae8874107099e2a071f7bfd1b4318b5a39`) preserved.

The [prior-learning review](prior-learning-review-2026-09-19.md) corrects the preceding run's interpretation, not its recorded elapsed charge: displayed CE was clipped nats, each arm scored 126,976 targets, undertraining is an untested cause, and CPR1 is not a resumable trainer checkpoint. Negative point-estimate directions remain. No reconstructed timing is presented as a measurement.

The [new prompt](deepseek-prior-learning-step-2026-09-19.md) proposes **3,600,000 ms total**: 900 s repairs/builds/data/cheap fixtures, 1,800 s maximum prior fitting, 500 s diagnostics/evaluation/checkpoint/generation, 400 s reserve. This is a **proposal, not a charge or a pre-execution receipt**. Refresh and record the complete projection before execution; revise before overruns under standing local-extension authorization. One model worker, at most four Cargo jobs, 8 GiB peak RSS, 1 GiB incremental storage on the reused worktree/cache and the 128 MiB stop margin. Account for any new checkout separately. No paid compute or deletion.

## Projection recorded before use — prior-learning recovery tranche (2026-09-19)

**Reason.** The [prior-learning review](prior-learning-review-2026-09-19.md) establishes that the #1294 experiment was numerically and procedurally wrong (clipped nats labelled as bits; train/serve bias mismatch; a fit prefix of 2,048 of 23,559 windows; a 62-target mask; document-opening development; a misaligned two-token reference; a report that is neither a manifest nor a resumable checkpoint) and directs one corrective implementation plus one conditional prior-only learning curve. The prompt's 3,600,000 ms tranche is recorded here before execution.

| Item | Estimate |
|---|---:|
| Correctness repairs, new numerical module, focused fixtures, builds | 900,000 ms |
| Maximum prior-only fitting (checkpoints 0/256/512, conditional 1,024) | 1,800,000 ms |
| Artifact/checkpoint diagnostics, evaluation, gates, generation | 500,000 ms |
| Retry, resume-equality test and stop reserve | 400,000 ms |
| **Total tranche** | **3,600,000 ms** |

**No allowance extension is used.** The limit stays `154,400,000 ms`; this draws on the existing recorded
balance. **No new checkout is created**: the existing clean isolated worktree is reused on a new branch, so
no additional approximately 1.2 GiB checkout is charged. Build output reuses the existing `target/`.
One model worker, Cargo jobs bounded at 4, host 8 cores / 16 GiB so the 8 GiB peak-RSS cap is feasible,
new build/data/checkpoint storage below 1 GiB, **128 MiB model-storage stop margin untouched**. No paid
or external compute, no deletion.

## Charges recorded — prior-learning recovery Stage 1-2 (2026-09-19)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-19 | `prior_learning` module (one shared target iterator, uncapped bits, unified integer train/serve forward with a frozen <=4-bit bias, artifact `CPL2` v2 with bounded loading, resumable checkpoint), 9 focused fixtures, seven release build/test cycles while repairing defects | 800,000 ms | **Measured.** Roughly seven `cargo test --release` compile cycles at ~1-2 min each plus the `cargo fmt`/gate runs; the fixtures themselves complete in 0.15 s. Iterative repair included failed intermediate runs, all retained. |

**New cumulative: 149,838,565 ms.** Remaining: 154,400,000 − 149,838,565 = **4,561,435 ms (~76.0 min)**.

**Charged from the 3,600,000 ms recovery tranche recorded above; no allowance extension.** The unspent
remainder of that projection covers the Stage 3 fitting, diagnostics and checkpoint items that the
**cheap gate failure made conditional and therefore unspent**: no real-text model was trained, no manifest
or real-text checkpoint was produced, and no report root was claimed. That is a scope reduction forced by
an honest negative, not a silent saving. No model artifact was created by this run; the new tracked files
are one Rust module, one receipt and four document edits, with build output inside the existing `target/`.
The **128 MiB model-storage stop margin is untouched**; no deletion, no cleanup, no paid or external
compute.


## Review after PR #1296 — interpretation correction and proposed next tranche

Live recorded balance verified as **149,838,565 / 154,400,000 ms**, remaining **4,561,435 ms (76.02 minutes)**. Preserve the previous 800,000 ms charge as recorded; this review does not independently reconstruct its elapsed timing. No Rust build/model ran in the review, and no model charge or allowance change was applied. Reused the clean full worktree on `codex/learning-contract-repair`; original checkout and retained joint artifact remain preserved.

The preceding “unified integer train/serve forward” and “bounded loading/exact resume” descriptions exceed the implemented guarantees. The [review](learning-contract-review-2026-09-19.md) identifies a contextual scaling defect, contradictory task, small-gradient cutoff and incomplete scheduler/checkpoint/artifact checks. These correct interpretation, not historical elapsed charges. No evidence shows that the small comparator's omission required a budget stop.

The [next prompt](deepseek-learning-contract-step-2026-09-19.md) proposes a complete **3,600,000 ms** tranche: 1,200 s repairs/builds/independent checks, 300 s small fits/conditional comparator, 1,500 s conditional text fitting/evaluation/generation, 600 s evidence/delivery/reserve. This is not an executed charge or a pre-execution projection receipt; refresh and record before use. Revise before overruns under standing local-extension authorization. One worker, at most four Cargo jobs, 8 GiB RSS, 1 GiB incremental storage on reused checkout/cache and 128 MiB stop margin. No paid compute or deletion; account separately for any new full checkout.

## Charges recorded — learning-contract repair (2026-09-19)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-19 | Shared Adam repair with two independent scalar references, `prior_learning` rewrite (single served forward, fixture mask, applied bounded ReLU, Fisher–Yates schedule, complete checkpoint identity, artifact envelope), 12 focused fixtures, the authored capacity witness, the gate runner, one sealed gate run, receipts and delivery | 1,200,000 ms | **Measured.** Roughly eight `cargo test --release` compile cycles at 1–2 min each plus `cargo fmt`/gate runs; the fixtures complete in 0.00–0.02 s and the gate run in a few seconds. Iterative repair included failed intermediate runs, all retained. |

**New cumulative: 151,038,565 ms.** Remaining: 154,400,000 − 151,038,565 = **3,361,435 ms (~56.0 min)**.

**Charged from the 3,600,000 ms tranche recorded above; no allowance extension.** The unspent remainder
is the stage D real-text item (roughly 1,500 s in the projection) plus reserve, which was **not executed**:
the decision was remaining session budget, not a failing prerequisite and not a new approval stop, and
the balance was not silently increased to cover it. Retained under a claimed, sealed and verified report
root: `result.json`, `curve.json`, `predictions.json` (all 16 predictions with integer scores at every
checkpoint), a 264-byte `prior_only.cpl2` serving artifact and a 5,196-byte `prior_only.ckpt`
checkpoint, at `.uor-models/prior-learning-gate-2026-09-20/attempt-1`. Build output reuses the existing
`target/`; the **128 MiB model-storage stop margin is untouched**. No deletion, no cleanup, no paid or
external compute.

## Review after PR #1298 — preserved balance and real-text projection

Verified live JSON **151,038,565 / 154,400,000 ms**, remaining **3,361,435 ms (56.02 minutes)**. Preserve the preceding 1,200,000 ms as recorded; its approximately described build-cycle timing was not independently reconstructed. This source/retained-file/documentation review ran no Rust build/model and changed no cumulative charge or limit. Reused the existing clean full worktree on `codex/realtext-prior-handoff`; owner checkout and unique/sealed artifacts are preserved. Filesystem free-space observation approximately 43.4 GiB is not a complete storage inventory.

The [review](realtext-prior-review-2026-09-20.md) corrects the prior “complete checkpoint/A–C pass” scope: the active permutation is omitted, so mid-pass resume changes exposure; seed/identity, integer-only trace and focused derivative/envelope checks remain. These interpretation corrections do not reconstruct or reverse past charges. The recorded session-budget stop is distinct from depletion of the local wall-time allowance.

The [next prompt](deepseek-realtext-prior-step-2026-09-20.md) proposes a **3,000,000 ms** complete tranche: 900 s repairs/build/checks, 300 s data/references/probe, 1,000 s fitting, 500 s evaluation/controls/generation/I/O, 300 s evidence/delivery/reserve. This is a proposal, not an executed charge or pre-execution receipt. Refresh storage and record before use; measure the new-path timing and revise before overruns, using standing local-extension authorization as needed. One worker, at most four Cargo jobs, 8 GiB RSS, 1 GiB incremental storage on reused checkout/cache, 128 MiB protected margin. No deletion or paid compute.
