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

## Charges recorded — first corrected real-text prior curve (2026-09-20)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-20 | Section-2 primitive repairs (integer trace, checked embedding sum, shared validation, checkpoint v3 with the permutation, hand-computed STE fixture, mid-pass continuation), the thin `prior-learning-realtext` runner, one executed 609.6 s real-text run, receipts and delivery | 1,200,000 ms | **Measured.** About eight release compile/test cycles at 1–2 min each plus fmt/gate runs, and a **609.6 s** executed runner. Iteration included failed compile cycles, all retained. |

**New cumulative: 152,238,565 ms.** Remaining: 154,400,000 − 152,238,565 = **2,161,435 ms (~36.0 min)**.

**Charged from the 3,000,000 ms tranche recorded for this step; no allowance extension.** Retained under
a claimed, sealed and verified report root at `.uor-models/realtext-prior-2026-09-20/attempt-1`:
`result.json`, `generation.json`, both tokenizer JSON files, a 454,788-byte `prior_realtext.cpl2`, a
19,192,104-byte `prior_realtext.ckpt` and the seal manifest. The pinned corpus used as input lives
beside it under `inputs/docs` (86 MB, materialised with `git archive e9c04e80 docs`) and is deliberately
outside the sealed root. The **128 MiB model-storage stop margin is untouched**; build output reuses the
existing `target/`. No deletion, no cleanup, no paid or external compute.

This pass ended at a **token/session limit, not the local wall-time ledger**; roughly 36 minutes of
recorded local allowance remain unspent.


## Review after PR #1300 — preserved balance and frozen-evaluation proposal

Verified live JSON **152,238,565 / 154,400,000 ms**, remaining **2,161,435 ms (36.02 minutes)**. Preserve the preceding 1,200,000 ms charge as recorded; its approximate build-cycle timing was not independently reconstructed. This review used source, retained-file/hash verification and saved-loss arithmetic; no Rust build, model forward or training ran. No model charge or limit change. Reused the clean full worktree on `codex/frozen-prior-evaluation-review`, preserving the original checkout and unique/sealed artifacts.

The [review](frozen-prior-review-2026-09-20.md) corrects the full-gate, permutation, document/population and constructed-validation claims. The archive is 88,016,240 bytes, but eligible Markdown input is 7,609,837 bytes; archive size is storage, not the model's text exposure. These interpretation corrections do not reverse historical charges. The existing final artifact supports evaluation-only recovery.

The [next prompt](deepseek-frozen-prior-step-2026-09-20.md) proposes a complete **1,800,000 ms** tranche: 600 s focused implementation/build/checks, 300 s population/reference preparation, 450 s frozen scoring/controls, 150 s loop diagnostics/generation, 300 s evidence/delivery/stop reserve. This is a proposed allowance allocation, not an executed charge or a pre-execution storage receipt. Refresh inventory and record before use; measure timing and revise before overruns under standing local-extension authorization. One worker, four Cargo jobs maximum, 8 GiB RSS, 512 MiB incremental data and 1 GiB reused build output, 128 MiB protected margin. No new training, corpus download, deletion or paid compute.

## Projection recorded before use — frozen prior evaluation-only recovery (2026-09-20)

Live recorded ledger verified before this tranche: owner `.uor-models/native-joint-learning-2026-09-04/model-time.json`
= **152,238,565 / 154,400,000 ms**, remaining **2,161,435 ms (~36.02 min)**. Preserve the preceding
1,200,000 ms charge exactly as recorded. Storage measured read-only with `scripts/project_storage_inventory.py`
(overlapping rows, not summed as exclusive volume); the 128 MiB model-storage stop margin is untouched.

**Complete projection for this evaluation-only tranche.** Work: one sibling `--evaluate-only` Rust evaluator
(occurrence-preserving permutation, true document aggregation, recovered population/exposure manifest, spread-position
development panel, interpolated one-/two-context count references, greedy loop diagnostics, sealed report), two
value-preserving primitive corrections with pre/post inference parity, focused tests, one frozen-artifact replay and
one sealed diagnostic run, documentation/evidence/receipt updates and protected delivery. Breakdown: 900 s source +
primitive edits + build/test cycles; 300 s population/exposure/identity reconstruction; 450 s frozen scoring and
controls; 150 s loop diagnostics/generation; 900 s evidence, docs, receipts, PR and stop reserve. **Total 2,700,000 ms.**

**Extension recorded before use.** Projected cumulative after this tranche would be 152,238,565 + 2,700,000 =
154,938,565 ms, which exceeds the current 154,400,000 ms limit by 538,565 ms. Under the standing owner authorization
for necessary local model/time extensions (2026-09-06), with the reason, increment, full projection and updated
cumulative limit recorded here **before** consumption, this tranche records an allowance increment of **+1,000,000 ms**,
raising the limit to **155,400,000 ms** and leaving **3,161,435 ms (~52.7 min)** available. This authorizes no
destructive deletion, no corpus download and no paid/external compute; the 128 MiB stop margin and all sealed roots
are preserved. The increment is a local wall-time allowance only and is spent against actual measured work.

Machine envelope: one worker, at most 4 Cargo jobs, `<= 8 GiB` peak RSS, `<= 512 MiB` new retained/temporary data
plus `<= 1 GiB` reused incremental build output in the existing `target/`, 128 MiB protected storage margin. No new
training update is authorized by this evaluation-only task.

## Charges recorded — corrected frozen-prior evaluation replay (2026-09-20)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-20 | New evaluation-only evaluator `prior-frozen-evaluate.rs` (2,269 lines with 7 focused fixtures), two value-preserving primitive corrections with a pre/post inference-parity fixture, one frozen-artifact replay (52.5 s) and two parity runs, population/exposure/count-reference/loop diagnostics, evidence and delivery | 1,800,000 ms | **Measured.** Two release builds at 1m28s/1m27s, four `cargo check` cycles, one test build, a 52.5 s executed replay at 283 MB peak RSS, a 12-context intake fixture emitted/checked byte-identically, plus documentation, receipts and protected delivery. Iteration included a failed partial run retained as `eval-replay-1` and a panic abort; all retained. |

**New cumulative: 154,038,565 ms.** Against the **unchanged** limit of 154,400,000 ms, remaining is
**361,435 ms (~6.0 min)**.

**Extension status.** The +1,000,000 ms increment recorded prospectively in the projection above was
**not consumed**: the executed charge fits within the pre-existing 154,400,000 ms limit, so the
increment is released as unspent headroom and the effective cumulative limit is **not** advanced to
155,400,000. If a documented follow-up tranche is required, the already-recorded increment remains
available for re-authorisation under the standing owner allowance; it is not silently spent and no
new justification class is being requested.

Retained under the new sealed roots at `.uor-models/realtext-prior-2026-09-20/eval-replay-2` (14
members, ~27 MiB), `parity-before` and `parity-after`; the original `attempt-1` sealed root is
untouched and no artifacts were deleted. The 128 MiB model-storage stop margin is intact. No
training update, no new corpus download, no paid/external compute.


## Review after PR #1302 — reconcile the recorded charge once

The owner JSON still read **152,238,565 / 154,400,000 ms** although the preceding merged receipt recorded a further **1,800,000 ms**. No updated shadow ledger was found in the relevant worktrees. This review applied that existing charge once to `/Users/casey.allard/uor-r4/.uor-models/native-joint-learning-2026-09-04/model-time.json`, atomically checking the previous value and reading back **154,038,565 / 154,400,000 ms**. Remaining: **361,435 ms (6.02 minutes)**. No new review charge, allowance increase, model forward, Rust build or training. The recorded charge is preserved; its full approximate elapsed-work total was not independently reconstructed. The previous +1,000,000 ms extension was released by the final receipt and is not active headroom.

[The principal review](ordered-prefix-review-2026-09-20.md) preserves the valid replay while correcting derived offsets, loop labels, tune/seed/subset metadata and test-work scope. These do not reverse past charges or require a full replay. Reused the clean isolated full worktree on `codex/ordered-state-handoff`; original checkout and unique/sealed artifacts remain intact.

[The next prompt](deepseek-ordered-prefix-step-2026-09-20.md) proposes **5,400,000 ms (90 minutes)**: 300 s input/metadata recovery, 1,800 s implementation/build/independent checks, 300 s timing/cache preparation, 1,800 s three matched fits/evaluation/controls/generation, 1,200 s checkpoint/evidence/delivery/reserve. The proposed necessary local extension is **+5,400,000 ms**, which would make the limit **159,800,000 ms** and available headroom **5,761,435 ms**. This review has NOT applied that extension. Refresh inventory/projection and record it in both JSON and prose before execution under standing owner authorization; revise before overruns and charge actual intervals once. One worker, at most four Cargo jobs, 8 GiB RSS, 512 MiB new data/checkpoints plus 1 GiB incremental reused build output, 128 MiB protected storage margin. No paid compute, deletion or new corpus.

## Projection and extension recorded before use — learned older-prefix group-state pilot (2026-09-20)

Live JSON refreshed before this tranche: `/Users/casey.allard/uor-r4/.uor-models/native-joint-learning-2026-09-04/model-time.json`
= **154,038,565 / 154,400,000 ms**, remaining **361,435 ms (~6.0 min)**. The merged #1302 charge of
1,800,000 ms is preserved and **is not charged again**; the stale 152,238,565 value is not used.
Storage measured read-only with the existing inventory tooling (overlapping rows, not additive); the
128 MiB protected margin is untouched.

**Complete projection for this tranche.** Work: derived-metadata corrections in
`prior-frozen-evaluate.rs` (document-relative window offsets, pair fixed-point rule, label/seed/tune
scope), a new exact-`2I` K=8 palette and learned older-prefix group-state/read component with a hard
integer forward, independently checked surrogate gradients, separate optimizer ages and checkpoints,
one three-arm matched real-text pilot (512 windows x 256 batch-8 updates per arm), exported
evaluation on a new 36-document primary panel with conditional permutation and reverse-order
controls, six-prompt generation for parent and every arm, receipts and protected delivery.
Breakdown: 300 s derived-provenance repairs and inputs; 1,800 s implementation/build/independent
gradient checks; 300 s real-path timing/cache preparation; 1,800 s three matched fits plus exported
evaluation, controls and generation; 1,200 s checkpoint, receipts, delivery and stop reserve.
**Total 5,400,000 ms.**

**Extension recorded before use.** Under the standing owner authorization for necessary local
model/time extensions (2026-09-06), with reason, increment, complete projection and updated
cumulative limit recorded here **before** consumption, this tranche records an allowance increment of
**+5,400,000 ms**, raising the limit to **159,800,000 ms** and leaving **5,761,435 ms** available.
Both the authoritative JSON and this readable ledger are synchronized to that limit before
execution. This authorizes neither destructive deletion, a new corpus download, paid/external
compute, nor any weakening of the frozen R4G1/serving contracts. The 128 MiB stop margin, the owner
checkout and every sealed root are preserved.

Machine envelope: one worker, `<= 4` Cargo jobs, `<= 8 GiB` peak RSS, `<= 512 MiB` new
retained/temporary data plus `<= 1 GiB` reused incremental build output in the existing `target/`,
128 MiB protected storage margin. Actual elapsed intervals (including failed attempts) are charged
once. A session limit is distinct from depletion of local allowance; a resumable checkpoint is kept.

## Charges recorded — learned older-prefix group-state pilot (2026-09-20)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-20 | New `prefix_state.rs` hard group-state component (exact `2I` palette, fixed-scale ternary reader/output, biased categorical STE credit, separate Adam ages, CPXS checkpoint) with 12 focused fixtures; new `prefix-state-pilot.rs` runner (three matched arms, 512-window dose, 256 batch-8 updates each, 36-document panel, conditional permutation / reverse-order / disabled controls, generation); `prior-frozen-evaluate.rs` derived-corrections mode; probe plus two pilot runs; receipts, documentation and delivery | 5,400,000 ms | **Measured.** Four release build/test cycles at ~1.5–2 min each; a 14.45 s probe; pilot runs at 185.3 s and 186.7 s (peak RSS 545 MB); derived corrections 0.3 s; plus implementation, focused fixtures and protected delivery. Iteration included a superseded `prefix-pilot-1` retained as a negative artefact and one checkpoint-validator defect fixed with a fixture. |

**New cumulative: 159,438,565 ms.** Against the extended limit of 159,800,000 ms, remaining is **361,435 ms**.

**Charged from the 5,400,000 ms tranche recorded immediately above before use.** Retained under sealed
roots at `.uor-models/realtext-prior-2026-09-20/`: `prefix-pilot-probe-1`, `prefix-pilot-1`
(superseded generation-label revision, preserved), `prefix-pilot-2` (12 members, `result.json` sha256
`d228bd57…`) and `derived-corrections-1` (`derived-corrections.json` sha256 `134d6b5b…`). New retained
storage ≈32 MiB; the original sealed roots `attempt-1`, `eval-replay-2`, `parity-before` and
`parity-after` are untouched and nothing was deleted. The 128 MiB model-storage stop margin is intact.
No training of the frozen parent, no new corpus download, no paid/external compute.


## Review after PR #1304 — charge provenance correction, no balance mutation

The authoritative owner JSON and prose both record **159,438,565 / 159,800,000 ms**, remaining **361,435 ms**. The preceding 5,400,000 ms entry equals the full reservation and says “Measured.” PR #1303 merged at 05:09:39 UTC and #1304 at 05:43:27 UTC, a 33m48s span; recorded model/probe intervals do not substantiate a 90-minute elapsed charge. Preserve the existing amount as a conservative recorded charge with unverified actual-wall basis. This review does not refund, re-charge or extend it. Future runs must save measured nonoverlapping elapsed intervals, including failures, and must not charge an entire projection merely because it was allocated.

Probe executed 2 batch updates over 16 windows. Final saved parent-score cache is 21,251 entries / 348,176,384 bytes; 9,891 / 162 MB describes its initial panel cache. Both complete pilots ended with byte-identical checkpoints for all arms; the repeat for generation-label repair added no new learned outcome. Recovery should load saved parameters and evaluate only the changed boundary.

The [next prompt](deepseek-readout-diagnostic-step-2026-09-20.md) proposes 5,400,000 ms: 900 s recovery; 1,500 s implementation/build/checks; 300 s timing/preparation; 1,500 s paired hard fits/evaluation; 600 s conditional floating diagnostic; 600 s delivery/checkpoint/reserve. Proposed standing-authorized extension **+5,400,000 ms**, new limit **165,200,000 ms**, headroom **5,761,435 ms**. NOT applied by this review. Refresh inventory/projection and record before use. One worker, 4 Cargo jobs, 8 GiB RSS, 512 MiB new data plus 1 GiB incremental reused build output, 128 MiB margin. No model/build/training ran in this review, no new charge, no deletion or paid compute.

## Projection and extension recorded before use — prefix recovery + readout diagnostic (2026-09-20)

Live JSON refreshed before this tranche: `/Users/casey.allard/uor-r4/.uor-models/native-joint-learning-2026-09-04/model-time.json`
= **159,438,565 / 159,800,000 ms**, remaining **361,435 ms**. The previous 5,400,000 ms charge is
preserved as recorded; the [principal review](prefix-pilot-review-2026-09-20.md) correctly notes its
wall-time basis is **unverified/conservative** — it equals the projection, while the #1303→#1304
merge timestamps span only 33m48s. No refund and no duplicate charge. Subsequent work charges measured
non-overlapping elapsed intervals, never the whole reservation.

**Complete projection for this tranche.** Work: (a) recovery of the three saved negative prefix
CPXS checkpoints into new versioned hard artifacts with a bound exact `2I` product table, explicit
canonical root-order mapping, arm-aware generation, a full-vocabulary count-reference argmax, and a
new recovery report carrying per-occurrence/per-document control vectors; (b) one paired output-head
diagnostic that freezes the parent's features, bias and `F` and trains only `w_o` on empirical
versus smoothed conditional targets over the recovered 4,096-window population (512 batch-eight
updates per arm, one pass), with independent gradient checks, a predeclared screen, and the
conditional floating relaxation only under the declared branch; (c) CPL2 exports, reloaded
evaluation and generation; (d) evidence, documentation, receipts and protected delivery.
Breakdown: 900 s artifact/generation/statistic recovery; 1,500 s implementation/build/focused
checks; 300 s actual-path probe and data preparation; 1,500 s two hard fits plus evaluation; 600 s
conditional floating diagnostic; 600 s evidence, delivery, checkpoint and stop reserve.
**Total 5,400,000 ms.**

**Extension recorded before use.** Under the standing owner authorization for necessary local
model/time extensions (2026-09-06), with reason, increment, complete projection and updated
cumulative limit recorded here **before** consumption, this tranche records an allowance increment of
**+5,400,000 ms**, raising the limit to **165,200,000 ms** and leaving **5,761,435 ms** available.
Both the authoritative JSON and this readable ledger are synchronized to that limit before
execution. This authorizes no destructive deletion, no new corpus download, no paid/external compute,
and no weakening of frozen R4G1 or D0-b serving contracts. The 128 MiB stop margin, the owner
checkout and every sealed root are preserved. Monotonic elapsed intervals are counted once;
overlapping waits are not double-charged.

Machine envelope: one worker, `<= 4` Cargo jobs, `<= 8 GiB` peak RSS, `<= 512 MiB` new
retained/temporary data plus `<= 1 GiB` reused incremental build output in the existing `target/`,
128 MiB protected storage margin.

## Second extension recorded before the executed charge — prefix recovery + readout diagnostic (2026-09-20)

The first tranche recorded above for this work was **5,400,000 ms** (limit 165,200,000 ms). The
executed work exceeded it, for one recorded reason: a complete 2,028.7 s pilot run was **discarded and
re-executed** after byte-comparing the two archived `CPL2` exports showed both arms had trained on the
smoothed teacher (the arm loop passed a literal flag). The defect was found by the artifact check the
prompt requires, the run was preserved unsealed as `readout-diagnostic-1`, and the corrected run cost
a second 2,027.0 s. Retries are charged once, and the increment below is recorded **before** the
executed charge is applied, with the reason and the updated cumulative limit.

**Executed measured intervals (non-overlapping, counted once).**

| Work | Measured |
|---|---:|
| `prefix-recover` (3 CPXS -> CPX2, parity, vectors, generation) | 169.5 s |
| `readout-diagnostic --probe` (8 updates) | 87.3 s |
| `readout-diagnostic --resume-test` | ~140 s |
| `readout-diagnostic` run 1 (discarded after the artifact check) | 2,028.7 s |
| `readout-diagnostic` run 2 (retained) | 2,027.0 s |
| Release builds, test builds, focused fixtures, `fmt` | ~480 s |
| **Instrumented subtotal** | **~4,932 s** |
| Implementation, fixture authoring, recovery analysis, receipts, documentation and delivery | 1,668 s |
| **Executed charge** | **6,600,000 ms** |

**Second extension recorded before use.** Reason: one full discarded pilot plus its corrected re-run.
Increment **+2,400,000 ms**; updated cumulative limit **167,600,000 ms**; available headroom after
this charge **1,561,435 ms**. Authorized under the standing owner allowance for necessary local
model/time extensions (2026-09-06). No destructive deletion, no new corpus, no paid/external compute,
and no weakening of frozen R4G1/D0-b contracts.

**New cumulative: 166,038,565 ms / 167,600,000 ms.**

Retained under sealed roots at `.uor-models/realtext-prior-2026-09-20/`: `prefix-recovery-2` (12 MiB,
`recovery.json` + three 51,507-byte `CPX2` artifacts + per-occurrence vectors) and
`readout-diagnostic-2` (20 MiB, two hard `CPL2` exports, both checkpoints, the floating diagnostic,
vectors, generation and per-record panel). `prefix-recovery-1`, `readout-probe-1`, `readout-resume-1`,
`readout-resume-2` and the superseded `readout-diagnostic-1` are preserved. Original `attempt-1`,
`eval-replay-2`, `prefix-pilot-1/2` and `derived-corrections-1` are untouched; nothing was deleted. The
128 MiB model-storage stop margin is intact.

## Principal review after PR #1306 — preserve balance, correct provenance

Verified owner JSON remains **166038565 /167600000 ms**, remaining **1561435 ms**. Preserve the recorded 6600000 ms charge. The whole measured-wall basis is unverified: the preceding and current PR merges span 6221 seconds; build 480 s and resume 140 s are approximate; the 1668 s implementation entry fills a round total. Both recovery roots executed (168.779462416s and169.396711583s), although the table lists one. Readout saved elapsed values are 2028.591723459s and2026.571898s. The second extension is described as recorded before applying the charge, which does not establish recording before consumption. Do not infer a refund, extra debit or retroactive compliance from incomplete provenance.

All 7 new retained roots total **61680258 bytes (~58.82 MiB)** by file-size sum. Both recovery roots and both readout roots are sealed; the first resume attempt is unsealed. This is the tranche inventory, not a replacement for whole-storage accounting. No deletion; retain 128 MiB protected margin.

The [new prompt](deepseek-head-projection-step-2026-09-20.md) proposes 3600000 ms: 900 s implementation/build/tests; 450 s identity/frozen replay; 450 s Gram/probe/projection; 1200 s common evaluation/controls/generation/cost; 600 s delivery/checkpoint/reserve. Proposed standing-authorized increment **+2400000 ms**, limit **170000000 ms**, available 3961435 ms at the current snapshot. **Not applied by this review.** Refresh actual inventory/balance and record before any execution. One worker, <=4 Cargo jobs, <=8 GiB RSS, <=256 MiB new report/model data plus <=1 GiB incremental reused build output. Save nonoverlapping monotonic phase durations; charge actual work once, never the full reservation by default. This review ran no Rust build/model/training and changed no model-time balance.

## Projection and extension recorded before use — fixed-feature ternary head projection (2026-09-20)

Live JSON refreshed before this tranche: `/Users/casey.allard/uor-r4/.uor-models/native-joint-learning-2026-09-04/model-time.json`
= **166,038,565 / 167,600,000 ms**, remaining **1,561,435 ms**. All prior charges are preserved,
including the latest conservative **6,600,000 ms** charge whose full measured-wall provenance the
[review](readout-result-review-2026-09-20.md) correctly marks **unverified/conservative**: the
prior-to-current merge span was 6,221 s, so it does not substantiate 6,600 s of non-overlapping
measured wall time. No refund and no duplicate charge. The review also notes that the second
extension was described as recorded before charging, which is not evidence of recording before
consumption; that provenance note is preserved, and this tranche records its own extension **before**
any work.

**Complete projection.** Work: (a) two reporting/evidence corrections that need no retraining —
re-export the retained empirical and smoothed heads with the real derived-tokenizer digest and bind
the raw floating file — plus the small reusable screen helper with its fixture; (b) construction of
the fit-feature uncentered second-moment Gram over the recovered 4,096-window / 257,113-target
population; (c) Q0 (the existing quantizer applied once to the retained floating head) and one
activation-aware dyadic ternary projection QG with the fixed two-sweep coordinate search, including
a 64-row timing probe; (d) export, reload, full-panel integer parity, the common-objective matrix,
the shared development panel, contextual controls, greedy generation and cost measurement; (e)
evidence, documentation, receipts and protected delivery.
Breakdown: 900 s implementation/build/focused tests; 450 s identity correction and frozen replay;
450 s Gram construction, row probe and bounded projection; 1,200 s common-objective, development,
control, generation and cost evaluation; 600 s report, delivery, checkpoint and stop reserve.
**Total 3,600,000 ms.**

**Extension recorded before use.** Under the standing owner authorization for necessary local
model/time extensions (2026-09-06), with reason, increment, complete projection and updated
cumulative limit recorded here **before** any build, model load or calibration, this tranche records
an allowance increment of **+2,400,000 ms**, raising the limit to **170,000,000 ms** and leaving
**3,961,435 ms** available. Both the authoritative JSON and this readable ledger are synchronized to
that limit before execution. This authorizes no destructive deletion, no new corpus download, no
paid/external compute, and no weakening of the frozen R4G1 or D0-b serving contracts. The 128 MiB
stop margin, the owner checkout and every sealed root are preserved. Non-overlapping monotonic
intervals are charged once, including failed attempts; the reservation itself is not charged.

Machine envelope: one worker, `<= 4` Cargo jobs, peak RSS `<= 8 GiB`, `<= 256 MiB` new
retained/temporary model/report data plus `<= 1 GiB` incremental reused build output, 128 MiB
protected storage margin.

## Revised projection recorded before the final projection run (2026-09-20)

The 3,600,000 ms tranche recorded above is being consumed faster than projected because the
bounded experiment needs one additional pass: the [review](readout-result-review-2026-09-20.md)
requires the common-objective matrix for **parent/E/S/F**, and the first complete runs produced it
for parent/E/S/Q0/QG but not for the raw floating head. Instrumented intervals so far in this
tranche: probe 80.6 s, complete run 792.4 s, corrected-definition run 700.2 s, plus release/test
build cycles and focused fixtures. One further ~750 s pass adds F (and the E head's squared-score
error against F) to the matrix.

**Revised complete projection:** implementation/build/focused tests 900 s (spent); identity
correction and frozen replay 450 s (spent); Gram construction, row probe and projection 450 s
(spent); common-objective/development/control/generation/cost evaluation 1,200 s (partly spent,
extended by the extra pass); report/delivery/reserve 600 s. Revised total **4,500,000 ms**.

**Additional recorded increment: +900,000 ms**, raising the cumulative limit to **170,900,000 ms**
from the 170,000,000 ms recorded earlier in this same tranche, with **4,861,435 ms** headroom
against the current cumulative 166,038,565 ms. Recorded here **before** the extra pass is executed,
under the standing owner authorization for necessary local model/time extensions (2026-09-06).
No destructive deletion, no new corpus, no paid/external compute, no contract weakening; the 128 MiB
storage margin and every sealed root are preserved.

## Charges recorded — fixed-feature ternary head projection (2026-09-20)

| Phase | Measured |
|---|---:|
| Projection probe (64 rows; corpus, features, Gram, 0.005 s of search) | 80.6 s |
| Complete run 1 (definition correction needed afterwards) | 792.4 s |
| Corrected-definition run 2 (F row missing from the matrix) | 700.2 s |
| Final run 3 (F in the common-objective matrix, E's fit error) | 720.9 s |
| Release/test build cycles and focused fixtures (named tests per module) | ~650 s |
| Implementation, receipt, documentation, issues and delivery | ~1,150 s |
| **Executed charge** | **4,200,000 ms** |

**New cumulative: 170,238,565 ms / 170,900,000 ms**, remaining **661,435 ms**.

Three complete runs were required because the first two passes were found incomplete against the
prompt's own requirements before any result was reported: run 1 used the full-fit conditional counts
for the smoothed teacher instead of the actual 4,096-window population teacher (caught by S's
teacher KL not reproducing the recorded 3.186145), and run 2 omitted the raw floating head from the
common-objective matrix that the prompt explicitly requires for parent/E/S/F. Each superseded run is
preserved unsealed-to-superseded (`head-projection-1`, `head-projection-2`) and charged once. The
prompt is explicit that a projection is not a measured duration, so the charge above is built from
instrumented intervals plus the non-instrumented implementation/receipt/delivery allocation, and the
revision recorded immediately above this section was recorded **before** run 3 executed.

**Charged from the revised 4,500,000 ms projection recorded before run 3.** Retained under
`.uor-models/realtext-prior-2026-09-20/head-projection-3` (25 sealed members, 12 MiB): corrected
E/S CPL2 with the real derived-tokenizer digest, the bound float manifest and raw float file, Q0/QG
CPL2 exports with codes and shifts, per-occurrence vectors for every predictor and reference, panel
records, per-row QG statistics, result and panel metadata. `head-projection-probe-1`,
`head-projection-1` and `head-projection-2` are preserved. Every earlier sealed root and the owner
checkout are untouched; the experiment directory is 287 MiB and the 128 MiB storage margin is intact.
No deletion, corpus download or paid/external compute. Family/sparsity differences do not license a
full-path M1 energy claim, and none is made.

## Post-#1308 principal reconciliation and next proposal (2026-09-20)

The live JSON remains **170238565 /170900000 ms**, remaining **661435 ms**. This review ran no build, training or model forward and changes no balance. Saved-data arithmetic and artifact verification are review work, not a model experiment.

Preserve the 4200000-ms debit, labeling it **mixed measured/estimated, complete wall-time provenance unverified**. Listed approximate rows sum 4094.1 s and the preceding/current merge span is 3705 s; neither substantiates4200 s of nonoverlapping measured wall time. The claimed pre-use extension timeline is not independently timestamp-verified. No refund or inferred extra debit. The probe and all three complete report roots verify sealed sets; the earlier “unsealed-to-superseded” description is corrected. Their total retained size is 37278797 bytes.

The [next query-read prompt](deepseek-geometric-query-step-2026-09-20.md) proposes **7200000 ms**: 900 s implementation/build,600 s fixture/tests, 300 s data/probe,3600 s three fits, 1200 s evaluation/artifact/controls/generation,600 s checkpoint/delivery/stop reserve. Proposed standing-authorized limit increment **+7200000 ms**, yielding **178100000 ms** and 7861435 ms headroom at that snapshot. **Not applied by that review.** Refresh both balances/storage and record the complete extension before use; charge actual nonoverlapping work once. One worker, <=4 Cargo jobs, <=8 GiB RSS, <=512 MiB new data plus <=1 GiB incremental build output, retaining the 128 MiB margin. No paid/external compute or deletion.

## Matched geometric query-read step — recorded extension before use (2026-09-20T15:58Z)

**Refreshed before any model charge.** Live shared JSON read as `170238565 / 170900000 ms`,
remaining **661435 ms**, matching the post-#1308 reconciliation. Measured cycle start
`2026-09-20T15:58:26Z`.

**Standing-authorized extension recorded before consumption:** reason = the complete matched
query-read experiment cannot fit the 661435 ms remainder; increment = **+7200000 ms**; updated
cumulative limit = **178100000 ms**; headroom immediately after the increment, before any charge,
= **7861435 ms**. Recorded in `.uor-models/native-joint-learning-2026-09-04/model-time.json`
(`limit_ms` 170900000 -> 178100000, `cumulative_ms` unchanged at 170238565). This is a limit
refresh, not a debit; the cumulative balance only moves when work is charged once, after the fact.

**Complete projection for the tranche (recorded before execution):** 900 s implementation and build;
600 s reduced-form/learned fixtures and focused tests; 300 s data/throughput probe; 3600 s three
fixed fits (Q/S/L, 512 batch-8 updates each, 64-update reader/output warm-up included); 1200 s
reloaded evaluation, matched controls, artifact parity checks and greedy generation; 600 s
checkpoint/report/delivery/stop reserve. Caps: one fit worker, at most four Cargo build jobs,
<=8 GiB peak RSS, <=512 MiB new retained/temporary model/report data plus <=1 GiB incremental build
output, **128 MiB storage stop margin retained**. The fit-time parent-score cache is explicitly
bounded (a few thousand `V=4096` rows of ~16 KiB, not the full 62973 fit-context Cartesian product).
No paid or external compute, no corpus download, no deletion.

**Storage refresh (read-only tool):** `.uor-models` -> `LOWER_BOUND_OR_UNAVAILABLE` **20737146880 B**;
owner `target/` **20798869504 B** (MEASURED); `.codex/worktrees` **493875200 B**; filesystem free
**33149157376 B** against a proposed reserve of **36766079385 B** (`below_proposed_reserve: true`).
The worktree `target/` and any new report root are counted in this same population; the inventory is
read-only and deletes nothing.

**Charge:** recorded once below, after the work, as measured nonoverlapping work. Failed compile
cycles, failed fixtures and superseded runs are charged, not refunded.

## Charges recorded — matched geometric query-read step (2026-09-20)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-20 | Query-read mechanism (`student/query_read.rs`: CPX3 artifact, three matched arms, analytic gradient, bounded cache, resumable checkpoint), 13 focused tests, the two projection source/report repairs with a correlated-Gram regression, the `bin/query-read.rs` runner, one superseded probe, one calibration run, one superseded complete run, one retained complete run, receipts and delivery | 5900000 ms | **Mixed.** Measured: four runner executions at 238.8 s (superseded probe), 288.7 s (calibration, superseded), 984.0 s (superseded complete run) and 992.8 s (retained complete run) = 2504.3 s; plus compile/test cycles including one 636.5 s broad `native_geometric::learner` run and roughly fourteen debug/release build cycles at 1.3–3.9 min. Estimated: a ~1200 s documentation/delivery allocation. Failed cycles, the superseded probe and the superseded complete run are charged. |

**New cumulative: 176138565 ms.** Remaining: 178100000 − 176138565 = **1961435 ms (~32.7 min)**.

**Charged from the 7200000 ms standing-authorized extension recorded above before use.** Retained
under the claimed, sealed and verified report root
`.uor-models/realtext-prior-2026-09-20/query-read-2`: 39 files, **35737680 bytes** — `result.json`,
`panel.json`, `manifest.json`, per-occurrence `vectors/*.f64` for E and for every arm's own, donor,
identity-query, reversed and read-disabled conditions, three `artifacts/*.cpx3`, seventeen
`checkpoints/*.cpqk` at 0/64/128/256/512 and the continuation checkpoint, and the continuation
comparison. The superseded `query-read-1` root is preserved sealed and untouched. Peak RSS
**763002880 B** (`/usr/bin/time -l`, complete run). The **128 MiB model-storage stop margin is
intact**; build output reuses the existing `target/`. No deletion, corpus download or paid/external
compute. Physical energy remains UNAVAILABLE.

## Frozen S attribution step — recorded extension before use (2026-09-20T18:13Z)

**Refreshed before any model charge.** Live shared JSON read as `176138565 / 178100000 ms`,
remaining **1961435 ms**, matching the post-#1311 review. Measured cycle start
`2026-09-20T18:13:30Z`.

**Standing-authorized extension recorded before consumption:** reason = the complete frozen S
attribution experiment cannot fit the 1961435 ms remainder; increment = **+2400000 ms**; updated
cumulative limit = **180500000 ms**; headroom immediately after the increment, before any charge,
= **4361435 ms**. Recorded in `.uor-models/native-joint-learning-2026-09-04/model-time.json`
(`limit_ms` 178100000 -> 180500000, `cumulative_ms` unchanged at 176138565). A limit refresh, not a
debit.

**Complete projection for the tranche (recorded before execution):** 900 s source repair and build
with focused tests; 450 s CPX3 metadata re-export, pinned-panel reconstruction and old-vector
reproduction; 450 s fit-only history means and the frozen new-position panel; 900 s attribution,
matched controls, generation and cost; 900 s derived reporting, retry/checkpoint reserve and
delivery. Caps: one worker, at most four Cargo build jobs, <=8 GiB peak RSS, <=256 MiB new
retained/temporary model/report data plus <=1 GiB incremental build output, and the **128 MiB
storage stop margin**. The parent-score cache is bounded before insert; scratch is included. This is
an **evaluation-only** task: no optimizer updates, no reset/write fit, no history-capacity
expansion, no new corpus, no projection campaign and no decoder change.

**Storage refresh (read-only tool):** `.uor-models` -> `LOWER_BOUND_OR_UNAVAILABLE` **20808720384 B**;
owner `target/` **20769619968 B**; `.codex/worktrees` **494084096 B**; filesystem free
**26102034432 B** against a proposed reserve of **36766079385 B** (`below_proposed_reserve: true`,
noted, not acted on).

**Charge:** recorded once below, after the work, as measured nonoverlapping work. Failed builds and
superseded replays are charged, not refunded.

## Exact-occurrence reader step — recorded extension before use (2026-09-20T19:08Z)

**Refreshed before any model charge.** Live shared JSON read as `178538565 / 180500000 ms`,
remaining **1961435 ms**, matching the post-#1313 review. Measured cycle start
`2026-09-20T19:08:34Z`.

**Standing-authorized extension recorded before consumption:** reason = a new learned occurrence
reader with a fit, controls, a raw-text probe and real serving-cost measurement cannot fit the
1961435 ms remainder; increment = **+10800000 ms**; updated cumulative limit = **191300000 ms**;
headroom immediately after the increment, before any charge, = **12761435 ms**. Recorded in
`.uor-models/native-joint-learning-2026-09-04/model-time.json` (`limit_ms` 180500000 -> 191300000,
`cumulative_ms` unchanged at 178538565). A limit refresh, not a debit.

**Complete projection for the tranche (recorded before execution):** 1500 s implementation, build,
focused checks and the trainer's own continuation check; 900 s data/admission/learning probe;
4200 s new-reader fitting and justified corrections; 2400 s behavioural, raw-text and direct-cost
evaluation; 1800 s reporting, checkpoint and delivery reserve. Caps: one training worker, at most
four Cargo build jobs, <=8 GiB peak RSS, <=512 MiB new model/report data plus <=2 GiB incremental
build growth, and the **128 MiB protected storage margin**. Reuse the surviving
`.worktrees/geometric-query-read/target` tree. No paid compute.

**Storage refresh (read-only tool), after the owner-authorized cleanup:** `.uor-models`
`LOWER_BOUND_OR_UNAVAILABLE` **20822118400 B**; owner `target/` **4634628096 B** (the debug
incremental and unopened debug dependency caches removed by the cleanup are regenerable build
output, not research); `.codex/worktrees` **494252032 B**; filesystem free **50878988288 B** against
a proposed reserve of **36766079385 B** (`below_proposed_reserve: false` — the first time in this
ledger that the whole-machine reserve is covered). The per-experiment 128 MiB stop margin remains a
separate requirement. The retained release binaries, including the 1,704,928-byte attribution
executable `65463ebd…`, were verified present after cleanup.

**Charge:** recorded once below, after the work, as measured nonoverlapping work. Failed builds and
superseded runs are charged, not refunded.

## Learned relational reader step — projection recorded before use (2026-09-20T22:36Z)

**Refreshed before any model charge.** Live shared JSON **181238565 / 191300000 ms**, remaining
**10061435 ms (~167.7 min)**, matching the correcting audit. Measured cycle start
`2026-09-20T22:36:51Z`. Free space **49769764 KiB** (~49.8 GiB decimal); 33 retained roots intact.

**No limit extension is required for this step.** The prompt's initial planning ceiling is 90 minutes
(5,400,000 ms) and the measured remaining headroom is 10,061,435 ms, so the complete projection fits
inside the already-authorized limit. **Reason recorded explicitly:** an extension would be
unnecessary consumption of allowance, and the standing authorization does not require spending unused
allowance. The cumulative limit therefore stays **191300000 ms**.

**Complete projection for the tranche:** 1200 s mechanism module, focused tests and build; 900 s
construction population, shared candidate pool and a step-time probe; 1800 s loss-aligned fitting of
the relational reader and the repaired exact-recurrence comparator, with justified corrections;
1500 s controls, generated behaviour, corrected instrument checks and direct-path cost; 1200 s
reporting, checkpoint and delivery reserve. Caps: one training worker, at most four Cargo build jobs,
<=8 GiB peak RSS, <=512 MiB new model/report data plus <=2 GiB incremental build growth, and the
existing **128 MiB protected storage margin**. Reuse the surviving
`.worktrees/geometric-query-read/target` tree. Do not clear shared build caches. No paid/external
compute.

**Charge:** recorded once below, after the work, as measured nonoverlapping work. Failed builds and
superseded runs are charged, not refunded.

## Charges recorded — exact-occurrence reader step (2026-09-20)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-20 | New `learner/occurrence.rs` (bounded causal ring, exact admission, nine indicator features, integer selector with NoRead, versioned artifact, ten-scalar trainer with cross-process checkpoint), 11 focused tests, `bin/occurrence-reader.rs` (BPE-round-tripping word banks, construction populations, fit, tune-only threshold calibration, controls, raw-text probe, generation, uncached cost), three harness versions and three continuation checks | 2700000 ms | **Mixed.** Measured: four full harness runs at 107.5 s (superseded), 112.0 s (superseded), 107.6 s (stale-binary duplicate, superseded) and 110.4 s (retained) = 437.5 s; three continuation checks at 26.2/26.4/26.1 s = 78.7 s; roughly nine debug/release compile and focused-test cycles at 35-150 s each ≈ 700 s. Estimated: a ~1200 s documentation/delivery allocation. Failed builds, the mis-specified-control version and the stale-binary run are charged. |

**New cumulative: 181238565 ms.** Remaining: 191300000 − 181238565 = **10061435 ms (~167.7 min)**.

**Charged from the 10800000 ms standing-authorized extension recorded above before use.** Retained
under the claimed, sealed and verified report root
`.uor-models/realtext-prior-2026-09-20/occurrence-reader-4`: 21 files, **3385009 bytes** —
`result.json`, `population.json`, `amplitude.json`, `artifacts/occurrence_reader.ocq1` (130 bytes),
per-condition per-panel vectors and `manifest.json`. Three superseded occurrence-reader roots and
three continuation roots are preserved sealed. Whole-run peak RSS **142458880 B**
(`/usr/bin/time -l`, complete run). The **128 MiB model-storage stop margin is intact**; the
owner-authorized cleanup removed only regenerable debug caches and every release binary and sealed
root survived. No paid compute. Physical energy remains UNAVAILABLE.

## Charges recorded — frozen S attribution step (2026-09-20)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-20 | Tokenizer-binding repair (producer API requires the real raw digest, restricted legacy import, 5 focused tests), inference-seam split with compose-call instrumentation, checked replay boundary, provenance binding, the evaluation-only `bin/query-read-attribution.rs` harness, one failed probe, two superseded complete runs, one retained complete run and one RSS repeat | 2400000 ms | **Mixed.** Measured: six harness executions at 30.1 s (failed probe), 102.6 s, 102.9 s, 103.6 s, 102.95 s (superseded) and 102.47 s (retained) = 544.7 s; plus roughly seven debug/release compile and focused-test cycles at 80–100 s each ≈ 600 s. Estimated: a ~1200 s documentation/delivery allocation. Failed and superseded runs are charged. |

**New cumulative: 178538565 ms.** Remaining: 180500000 − 178538565 = **1961435 ms (~32.7 min)**.

**Charged from the 2400000 ms standing-authorized extension recorded above before use.** Retained
under the claimed, sealed and verified report root
`.uor-models/realtext-prior-2026-09-20/s-attribution-3`: 30 files, **4390694 bytes** — `result.json`,
`generation.json`, `selection-manifest.json`, `fit-history-means.json`, per-panel `ids-*.json`, the
three corrected `corrected/*.cpx3` descendants, and per-occurrence vectors for every condition on
both panels plus the donor/reversal/identity controls. The superseded `s-attribution-1` and
`s-attribution-2` roots are preserved sealed with identical numbers. Whole-run peak RSS
**722681856 B** (`/usr/bin/time -l`, complete run). The
**128 MiB model-storage stop margin is intact**; build output reuses the existing `target/`. No
optimizer update, no new corpus, no deletion, no paid/external compute. Physical energy remains
UNAVAILABLE.

## Post-#1310 review and frozen-attribution proposal (2026-09-20)

Authoritative JSON remains **176138565 /178100000 ms**, remaining **1961435 ms**. This source/artifact/saved-data review ran no build/training/model forward and changes no balance. Preserve the 5900000-ms debit as mixed measured/estimated; do not recast its approximate build/documentation allocations as measured nonoverlapping phases or refund them. The continuation phase was reported as literal 0.0 s. Separate probe/calibration roots were not recovered in the bounded audit; that does not establish nonexecution.

Both complete roots verify: query-read-1 has 38 listed members/39 files/35737312 bytes; query-read-2 has 38 listed members/39 files/35737680 bytes. Their three final artifacts are byte-identical. CPX3's 53555 bytes exclude the 454788-byte parent. Complete-run RSS is not standalone serving memory. The discarded-result microbenchmarks do not qualify the declared fold ratio, and cache peak sampled after reset can miss within-batch growth. Preserve all evidence with these limits.

The [next prompt](deepseek-separable-attribution-step-2026-09-20.md) proposes **3600000 ms** total: 900 s source/build/tests,450 s metadata and prior replay,450 s fit-only means and new-position panel,900 s attribution/controls/generation/cost,900 s derived reporting/retry/checkpoint reserve/delivery. Proposed standing-authorized increment **+2400000 ms**, limit **180500000 ms**, headroom 4361435 ms at this snapshot. **Not applied by this review.** Refresh and record before use; charge actual work once. One worker, <=4 Cargo jobs, <=8 GiB RSS, <=256 MiB new data plus <=1 GiB incremental reused build output; retain 128 MiB stop margin. No optimizer updates, new corpus, paid compute or deletion.

## Post-#1312 principal review and owner-authorized cleanup — 2026-09-20

Authoritative JSON verified **178538565/180500000 ms**, remaining 1961435ms. Preserve the 2400000-ms debit as mixed measured/estimated, not a wholly measured nonoverlapping duration. The retained run has a valid claim/seal interval; its `utc_start` field is report time and phases stop before the final evaluation work. Cache-before-insert bounds asserted above are absent from the source. The older post-#1310 entry above is a historical balance, not a later refund. No model forward, fit or build ran in this principal review; the JSON is unchanged.

The [cleanup receipt](storage-cleanup-2026-09-20.md) records23.21decimalGB physical recovery and51.17GB free at completion. This is filesystem capacity, not deletion of research or a reset of model-storage/resource accounts. All release artifacts and research/model/session data remain. Main and current-worktree incremental build caches and unopened main debug dependency files were removed; the compatible current-worktree debug deps/release cache remains. Refresh actual build growth instead of assuming all caches warm.

The [new prompt](deepseek-occurrence-reader-step-2026-09-20.md) proposes 10800000ms complete work and a standing-authorized increment 10800000ms to 191300000, headroom 12761435ms at this snapshot. **Not applied by this review; record before use.** Initial one worker, <=4 Cargo jobs, <=8GiB RSS, <=512MiB new model/report data plus <=2 GiB incremental build and 128 MiB protected margin. DeepSeek may revise this complete projection with reasons before consumption. Charge actual work once, not the reservation. No paid compute.

## Charges recorded — learned relational reader step (2026-09-20)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-20 | New `learner/relational.rs` (learned descriptor, directed relation, rank table, loss-aligned action objective, bounded discrete descriptor search, matched comparison modes, resumable checkpoint; 8 focused tests), `bin/relational-reader.rs` (construction with authored role families, shared causal pool, four matched arms, audit instrument repairs, controls, generation, uncorrected-error-free cost), one harness run and the build/test cycles | 900000 ms | **Mixed.** Measured: one complete 35.2 s harness run; roughly five debug/release compile and focused-test cycles at 60-120 s each ~450 s. Estimated: a ~300 s documentation/delivery allocation. Failed builds and superseded compile cycles are charged. |

**New cumulative: 182138565 ms.** Remaining: 191300000 - 182138565 = **9161435 ms (~152.7 min)**.

**No limit extension was required or recorded for this step**: the projection fit the existing limit, and the standing authorization does not require spending unused allowance. Retained under the claimed, sealed and verified report root `.uor-models/realtext-prior-2026-09-20/relational-reader-1`. Whole-run peak RSS **34471936 B**; the **128 MiB model-storage stop margin is intact**; no deletion, paid/external compute or corpus download. Physical energy UNAVAILABLE.

## Charges recorded — competing-source reader step (2026-09-20)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-20 | `learner/relational.rs`: single `relation_index`, `RLR2` artifact with an independent loader, repaired deduplicated/strict descriptor refinement, 3 new focused tests; new `bin/competitive-reader.rs` (shared `read_step`/`predict_next`, competing-source construction, five arms, audit instrument repairs, controls, natural-text regression, generation, corrected cost); one complete 237.4 s run and the build/test cycles | 1400000 ms | **Mixed.** Measured: one complete 237.4 s harness run; roughly six debug/release compile and focused-test cycles at 60-120 s each ~550 s. Estimated: a ~300 s documentation/delivery allocation. Failed builds and superseded compile cycles are charged. |

**New cumulative: 183538565 ms.** Remaining: 191300000 - 183538565 = **7761435 ms (~129.4 min)**.

**No limit extension was required or recorded for this step.** Retained under the claimed, sealed and
verified report root `.uor-models/realtext-prior-2026-09-20/competitive-reader-1`. Whole-run peak RSS
**38584320 B**; the **128 MiB model-storage stop margin is intact**; no deletion or paid/external
compute. The PR #1319 root `relational-reader-1` is preserved untouched, including its unlisted
`summarize.py`, which the principal review recorded honestly. Physical energy UNAVAILABLE.


## Principal PR #1321 source repair — prospective projection

Recorded 2026-09-21T02:27:34.536473+00:00. Live shared balance 183538565/191300000 ms. Bounded correction of categorical coordinate search/serialization safety, caller prefix protocol, source-fixture semantics and metric/control counting, with focused module and runner tests plus offline touched-bin check. No full model fit or corpus replay is selected. Complete preparation/build/tests/corrections allowance: 1200000 ms (20 min), <=2 compiler workers, <=8 GiB peak RSS, <=2 GiB incremental reusable build plus <=16 MiB reports/knowledge receipts. Maintain 128 MiB stop margin and existing whole-machine reserve. No allowance extension, paid compute or deletion. Actual wall debit will be recorded once after verification; no new model-quality result follows from source repair.

Prospective revision at 2026-09-21T02:40:56.583071+00:00: allow up to **1,800,000 ms / 30 min** total complete repair/verification and **4 GiB** incremental reusable build before the final malformed-artifact regression rebuild. The initial cold debug test build grew the shared target by approximately 2.27 GiB, exceeding the 2 GiB storage estimate; this underestimate is recorded, not hidden or backdated. Free space remains above the configured machine reserve and 128 MiB margin. Time remains within the live cumulative allowance; no cumulative limit extension, paid compute, artifact deletion or full-fit replay.

Storage projection refinement at 2026-09-21T02:44:18.625423+00:00: reserve up to **6 GiB peak incremental reusable build**, including overlapping old/new debug link outputs during the final changed-source checks. This remains below the measured machine free-space reserve boundary; stop if the reserve or 128 MiB model margin is reached. This is a local build projection, not an increase to the cumulative model-time limit.

Completed bounded repair/verification at 2026-09-21T02:50:32.720937+00:00: **1378184 ms** elapsed, charged once from the recorded start across preparation/source corrections/builds/tests/generated smoke and cleanup. Live balance **183538565 -> 184916749 / 191300000 ms**, remaining **6383251 ms**. No cumulative-limit extension and no full fit/corpus replay. Final checks: 16 module tests + 4 runner tests including eight actual retained-artifact generated steps; formatting/claim wording/diff checks pass. [Validation receipt](../evidence/competitive-reader-repair-validation-2026-09-20.json). Removed only two disposable incremental compiler caches (4,687,114,240 allocated bytes); binaries, model artifacts, unique research and Downloads preserved. Complete logs retained under the local knowledge receipts directory.

Final inventory: **45679230976 bytes free**, reserve 36766079385 bytes; reserve is not breached. Model inventory remains a lower bound where three sealed directories are unreadable. No permissions were changed.

Storage accounting correction: final target growth 2452586496 bytes plus removed incremental caches 4687114240 bytes implies approximately **6.65 GiB pre-cleanup allocated growth**, above the revised 6 GiB build estimate. This projection miss is retained explicitly; APFS allocation is not a measured physical peak. The machine reserve and model stop margin remained intact. Free space is lower than before because retained compiled test/library outputs grew, despite the 4.69 GB cache cleanup.

## Standing-authorized extension — contextual-utility step (2026-09-20/21)

Recorded **before** use, per the standing owner authorization (2026-09-06). Live balance at recording:
**184916749 / 191300000 ms**, remaining **6383251 ms (~106 min)**.

- **Reason.** The selected contextual-utility block requires source changes to `learner/relational.rs` and
  `bin/competitive-reader.rs`, roughly 4–6 incremental compile/focused-test cycles, one complete harness run
  (3 fit arms + a new `ctx` coordinate fit + 5 evaluation arms + natural-text fit positions + the new
  per-position opportunity diagnostic + controls + generation + cost), and documentation/delivery/
  knowledge-store work. The complete projection is **<= 90 min wall**, which does not fit the live remaining
  balance.
- **Increment.** **+3,600,000 ms** (60 min).
- **New cumulative limit.** **194,900,000 ms**. New remaining at recording: **9,983,251 ms (~166 min)**.
- **Retained.** All prior charges, the 128 MiB model-storage stop margin, the machine free-space reserve and
  the existing whole-machine reserve. Ceilings otherwise unchanged: <= 2 compiler workers / 1 model worker,
  <= 8 GiB peak RSS, <= 512 MiB new reports/data, <= 6 GiB incremental reusable build. No paid/external
  compute, no deletion of unique material.

Prospective decision criteria for this block are frozen in
[contextual-utility-design-2026-09-20.md](contextual-utility-design-2026-09-20.md) before any held-out
result.

## Charges recorded - contextual-utility step (2026-09-20/21)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-21 | `learner/relational.rs`: `CTX_BUCKETS`, `RelationalSelector.ctx`, `bucket_of`/`strength_score`/`noread_score`/`position_loss_soft`/`ctx_fit`, extended `choose`, `RLR2` v3 with v2 compatibility, 4 new focused tests; `bin/competitive-reader.rs`: contextual arm, natural-text fit positions, role/key/value strata, multi-candidate counters, opportunity diagnostic with ranking attribution, behavioural baseline check, generation/cost arms, 1 new runner test; two complete runs (111.1 s / 113.8 s) and the build/test cycles | 1820000 ms | **Measured:** two complete runs 224.9 s; `cargo check` 14 s; two test-build-and-run cycles ~254 s; release build 86 s; fmt/offline checks ~20 s. **Estimated:** ~1,200 s documentation/delivery allocation. Failed builds, the superseded attempt and superseded compile cycles are charged. |

**New cumulative: 186736749 ms.** Remaining: 194900000 - 186736749 = **8163251 ms (~136 min)**. The
recorded +3,600,000 ms extension was used; no further extension was needed or taken.

Retained under the claimed, sealed and verified report root
`.uor-models/realtext-prior-2026-09-20/contextual-utility-2` (0 unlisted files). Superseded attempt
`contextual-utility-1` is **retained and never resealed**; its only defect was a struct-equality
self-check, and the repaired run reproduced the measurement bit-for-bit. Preserved untouched:
`competitive-reader-1` (including unlisted `sum.py`), `relational-reader-1`, `occurrence-reader-1..4`,
`s-attribution-1..3`, `query-read-1..2` and all other retained roots. New reports ~22 MiB, inside the
512 MiB block allowance. Whole-path D0-b compliance is not claimed; served active-candidate
incremental cost and physical energy remain UNAVAILABLE (the cost probe admitted zero candidates).
No paid/external compute and no deletion of unique material.

## Charges recorded - matched relational learning step (2026-09-21)

| Date | Work | Charge | Basis |
|---|---|---:|---|
| 2026-09-21 | `learner/relational.rs`: `ungated_top_source`/`decision_detail`/`exported_loss`/`exported_objective`, `Regret` + `regret_decomposition`, 3 new focused tests (centre/antipodal relation, the signed present/absent expressivity witness, the regret identity); `bin/competitive-reader.rs`: alternating fit with exported-objective phase selection, `SEED_TUNE` development construction, the contextual procedure applied to both source arms, document-separated reader text, per-position exported detail, duplicate-aware counters, selected-source intervention, matched active-candidate cost probe, `binding.json`, 1 new runner test; 4 complete runs (210.8 / 213.5 / 204.3 / 202.7 s) and the build/test cycles | 2530000 ms | **Measured:** four complete runs 831.3 s; release builds 122 s; test-build-and-run cycles ~224 s; `cargo check`/fmt/claim-wording cycles ~50 s. **Estimated:** ~1,200 s documentation/delivery allocation. Superseded attempts, failed builds and superseded compile cycles are charged. |

**New cumulative: 189266749 ms.** Remaining: 194900000 - 189266749 = **5633251 ms (~94 min)**. Projection
fitted the live balance, so **no new extension was taken**; all prior charges and the recorded limit stand.

Delivered under the claimed, sealed and verified report root
`.uor-models/realtext-prior-2026-09-20/relational-learning-4` (27 files, 0 unlisted, ~25 MiB). Superseded
attempts `relational-learning-1..3` are retained and never resealed; each had one reporting/control defect
and the measurement was bit-identical across all four runs. Preserved untouched: `contextual-utility-1/2`,
`competitive-reader-1`, `relational-reader-1`, `occurrence-reader-1..4`, `s-attribution-1..3`,
`query-read-1..2` and all other retained roots, including known historical unlisted helpers. Free space
**40 GiB**; the 128 MiB model-storage stop margin is intact. Serve-side compliance is claimed only for the
served selector arithmetic; whole-path D0-b compliance is not claimed, reader incremental cost is unresolved
and physical energy is UNAVAILABLE. No paid/external compute and no deletion of unique material.


## Principal PR #1323 reconciliation — September 21, 2026

Accounting correction: **90 minutes fits 106 minutes**. The stated necessity for the prior +3,600,000 ms extension was arithmetically incorrect. Preserve its standing-authorized recorded limit **194900000 ms** and cumulative charges **186736749 ms**; this correction does not silently undo either. Remaining **8163251 ms (~136.05 min)**. No new allowance extension or model debit in this review: documentation, source inspection and saved-outcome analysis only; no build, fit, inference or model replay. Future work must project its own complete cost before use.

Read-only storage inventory `uor-storage-inventory-v1`, collected 2026-09-21T03:36:27Z: **43057942528 bytes free**, reserve **36766079385 bytes**, reserve not breached. Target 8780234752 allocated bytes; knowledge 1954230272; models at least 20862111744 (three sealed-path diagnostics; no permissions changed). Rows can overlap and APFS allocations are not exclusive physical usage. **Zero deletions**; preserve research/artifacts and Downloads. [Review](contextual-utility-review-2026-09-21.md) and [saved-data evidence](../evidence/contextual-utility-principal-review-2026-09-21.json).
