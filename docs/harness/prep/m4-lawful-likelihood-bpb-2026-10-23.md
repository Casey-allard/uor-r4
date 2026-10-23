# M4 — Lawful likelihood for bits-per-byte scoring (`m4-lawful-likelihood-bpb`)

- **Status:** DRAFT pre-registration, `TEST FITNESS: PENDING`. No compute is cleared. Authored by deepseek (`deepseek-49`) from the owner-approved design (`#2028` 2026-10-22T04:17:03Z, Card 1 "Yes, a linear ratio (Recommended)") and the merged instrument (`crates/uor-r4-integer/src/lawful_likelihood.rs`, PR #2945).
- **Milestone / Line:** M4 ([#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032), item [`m4-lawful-likelihood-bpb`](../../roadmap/items/m4-lawful-likelihood-bpb.md), [#2861](https://github.com/UOR-Foundation/uor-r4/issues/2861)); serving path [lut-serving](../../mechanisms/lut-serving.md).
- **Lab:** deepseek. **Class:** instrument (host-only); the registered scoring run is later research compute (GPU) and stays HELD until FIT + owner registration + an exported model.

## 1. Instrument

**PRLL (potential-ratio lawful likelihood)** — the merged module `crates/uor-r4-integer/src/lawful_likelihood.rs`:

1. At a scored position the integer model's Q24 candidate scores `s[0..V)` (V = 4096) are used unchanged; no exponential, table or squashing step is applied.
2. `phi[i] = s[i] - min(s) + 1` in exact `i64` (every `phi >= 1`); with `R = 2^48 - V`, `quota[i] = floor(phi[i] * R / sum(phi))` in `u128`, and the leftover `R - sum(quota)` units go one each to the largest `phi[i]*R mod sum(phi)` remainders, ties by lowest token id.
3. `mass[i] = 1 + quota[i] (+0/1)`; `sum(mass) = 2^48` exactly and every `mass >= 1`.
4. `bits_q12(mass)` = integer fixed-point `-log2(mass / 2^48)` = `48*4096 - (k*4096 + fractional_q12)` with `k = floor(log2 mass)` and 12 squaring rounds; per-token error `<= 2^-12 + 2^-48` bit, deterministic and truncating, never float.
5. `bits_per_byte(total_bits_q12, bytes)` returns an exact integer numerator and denominator; no rounded float is ever published as the result.

**Why lawful (owner Card 1):** a linear ratio of non-negative integer potentials by their sum; no softmax/log-softmax/cross-entropy/sigmoid/tanh/GELU/normalized exponential and no table form of them; no dense matmul; the divide happens in the offline scorer, never in the artifact. `-log2` is used only as a **measurement** of the mass (Card 2 reading).

## 2. Data split

- **Stream:** the chat held-out token stream of the `chat-v0-p2` recipe — `heldout.u16` (u16 LE tokens after a 64-byte header), tokenizer `chat-data-control-20261008/data/tokenizer.json` (4096 vocab + 3 added, sha256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`).
- **File:** **12,389,242 bytes**, sha256 **`e5f400b0e676791fd0458ce810163c5d2542adda80dccc8282f37d9ca8904cef`** (recorded and verified on the pod and after transfer in [`criterion2-protocol-pin-2026-10-09`](../../../archive/labs/criterion2-protocol-pin-2026-10-09/README.md)).
- **Stream byte basis:** 6,194,589 tokens after the header; **17,576,697 stream bytes**; 2.837427 bytes/token; 0 out-of-vocabulary; max id 4095. The scored bytes are the **stream basis** (this is the basis of the archived 0.877/0.933 BPB cells); the tool's target-byte basis is reported as a diagnostic only and never mixed with it.
- **Freeze rule:** the scoring host must re-verify the file sha256 and token count before scoring; any mismatch halts (typed refusal), it is never re-derived or patched.

## 3. Steps

- Deterministic window rule from the archived protocol pin: `stride = (tokens - context - 1) / windows`, starts `k * stride`, `384` targets per window.
- **Context and stride:** `context = 384`; `stride = (6,194,589 - 384 - 1) / 512 = 12,098` (matches the archived protocol pin).
- **Primary arm: 512 windows / 196,608 targets** (exceeds the >= 200 fresh-row floor of CHARTER §4.3 rule 3, D22 §3).
- **Secondary diagnostic: 64 windows / 24,576 targets**, reported separately; the two counts are never compared as one number (the archived 0.045935 BPB movement between window counts is why).
- **Measured noise (CHARTER §4.3):** the archived 512-window re-sampling standard error is **0.014525 BPB** ([criterion2 protocol pin](../../../archive/labs/criterion2-protocol-pin-2026-10-09/README.md)); the archived matched 0.886838 margin (0.013162) is itself inside that SE. A point estimate whose 95% interval straddles 0.90 is therefore **INCONCLUSIVE** and extends the window count (1024, then 2048) instead of deciding (the exact mapping is in the next bullet).
- **What each outcome decides (one interval partitions every reading):** with `se = 0.014525` and `z = 1.96`, let `lo = estimate - z*se` and `hi = estimate + z*se`. **PASS** iff `hi <= 0.90` (the whole interval is at or below the bar) records the M4 bar as met for this artifact/stream and becomes a notebook row; **FAIL** iff `lo > 0.90` (the whole interval is above the bar) closes this artifact/stream configuration (not the mechanism); **INCONCLUSIVE** iff the interval straddles 0.90, in which case the window count extends (1024, then 2048) rather than deciding. None of these outcomes releases training or changes the bar.
- Re-scoring the same stream twice must reproduce `bits_q12` totals bit-for-bit (determinism control).

## 4. Arms

| arm | likelihood | status |
|---|---|---|
| **A0** | PRLL on the served integer model's Q24 scores (this sheet) | primary |
| **B0** | polynomial distance kernel (E8 shell-count weighting), same scorer | separate pre-registration; not scored here |
| **C0** | exact count-table MLE baseline over TRAIN counts; a zero-count target is **unscorable**: `bits_q12(0)` returns `MassZero` in the merged module, so C0 records the count of unscorable targets and that cell is reported as not comparable on them (never as "infinite bits"); add-one variant labelled separately | baseline |
| **U** | uniform over V = 4096: `log2(4096)/2.837427 = 4.2292` BPB on the stream basis | sanity floor |

## 5. Bar (the owner's; not set or changed here)

The M4 bar is **held-out quality at or below 0.90 bits per byte** (`ROADMAP.md:17`, owner's). This sheet makes it scoreable on the frozen stream above; it does not register the bar and releases nothing. Two PM/owner conditions the design note already carries are repeated for the registration card: (i) whether the 0.90 bar may be read through an uncalibrated lawful likelihood (PRLL is lawful but not temperature-calibrated), and (ii) confirmation that the stream above is the chat held-out stream the bar means.

## 6. Base artifact (pending, and why)

Scoring needs a trained discrete model exported through `UORLUT01` — item `m4-export-uorlut01-cli` ([#2859](https://github.com/UOR-Foundation/uor-r4/issues/2859)) — plus the owner's release recorded verbatim in the lab's own chat (CHARTER §5.1; no typed phrase is needed). The archived 19.9M chat stack's original LUT is gone (criterion-2 lab); the retrain (`cbef6906…` checkpoint, `721d4bdb…` LUT) is the only matched artifact that exists and is recorded in the archive, not served. **No model is exported and no run is cleared by this sheet**; the scoring arm lands when the export exists.

## 7. Finite host controls (already green on main)

Nine lib tests in `lawful_likelihood.rs` cover the seven required controls: tiny alphabet with exact masses and sum `2^48`; V=4096 worst case (minimum mass exactly 1, exactly 48 bits); uniform power-of-two vocabulary; non-power-of-two ±1 remainders; token-id ties; monotonicity/argmax over 64 deterministic vectors; permutation equivariance; analytic BPB and MLE baselines; the zero-count-is-unscorable rule. They are host tests (`cargo test -p uor-r4-integer --lib lawful_likelihood`), with no model and no GPU.

## 8. Cost and minimum test power

- **Host (this sheet's controls): $0**, no GPU, seconds.
- **Registered scoring run: research compute (GPU, `scripts/pod/uor-pod`, `uor-pod status` first)**; estimated **<= 0.5 pod-hour** for 512 windows on an existing export. At the archive-observed **$0.89/h** the 512-window run took **16m30s (~$0.245)**, so the estimate is <= $0.45 at that rate; the $8/h cap applies (a 0.5h lease at the cap would be $4.00 and is not planned). Recorded before lease and saved off-pod. It also needs the exported model and the owner's release recorded verbatim in the lab's own chat (CHARTER §5.1).
- **Power:** 512 windows / 196,608 targets exceeds the >= 200 fresh-row floor; the result is deterministic (no seed sampling in the scorer); the model's own training seeds are reported with the artifact.

## 9. Stop conditions

Any stream-hash or token-count mismatch, any mass below 1 or sum != 2^48, any float in the scorer, or any nondeterminism between two identical runs halts scoring. An unscorable zero-count C0 target makes that C0 cell not comparable (`MassZero`) and is reported as such; it does not halt A0 scoring. A negative result closes this configuration (A0 on this stream), never the lawful-likelihood mechanism; instrument vs mechanism is classified before any conclusion.

## 10. Recorded FIT review

TEST FITNESS: PENDING (non-author review to come; the sheet is authored by deepseek, so the FIT must come from Claude, Kimi or antigravity).
