# The lean addressed read: the same failure-mode gain at a fraction of the state — and the price the last cycle could not see

References #2029 (M1 acceptance, D22 order 2 iteration; mechanism: exact addressed memory). Lab: DeepSeek, session
`deepseek/leanaddr`. Runs 2026-10-11 13:10–14:25 UTC: **three lean arms (245 s each), three matched controls (104–110 s)
and six BPB evaluations**, one pod, ≈ $2.20. Panel scoring on the laptop CPU.

> **One line.** The [addressed-read cycle](../labs/addressed-read-2026-10-11/README.md) met its failure-mode bar and its
> ledger named two costs: **+36.9 M parameters** and **20× the training wall**. This cycle cuts the codebook **8×**
> (`sub_keys=64 key_dim=64 top_k=16` instead of `256/128/32`) and finds that the cheap configuration **removes both
> costs** — **+1.26 M parameters (+4.3 %)** and **245 s against the control's 104 s** — and **keeps the failure-mode
> gain** (wrong-value **47 → 34 over 120 rows**, −13, on two of three seeds) — but at a **language-quality price the
> last cycle could not measure**: float BPB **+0.095 and +0.107** against matched controls on the line's own held-out
> stream, against the **0.02** this cycle pre-registered, and **7 fewer memory rows over 120**. **Bar NOT MET on the
> secondary criterion → REJECT this configuration**, and — the more important consequence — **the fat configuration's
> KEEP is downgraded to conditional**, because the cost this cycle measures is a property of the mechanism, not of the
> codebook size.

## The stream check, which makes every number here comparable

This pod's volume carried `chat-data-control-20261008/data/heldout.u16` rather than the `chat-cuda-20261008` file the
earlier cycles used, so the cycle pre-registered a check: **re-run the matched control and evaluate it here**. It reads
**1.17425** and **1.17003** for Ctl1 and Ctl2 — **the same two numbers to five decimals** those arms produced on the
earlier stream. The streams agree; every comparison below is apples to apples, and the line's BPB history stands.

## Results

| pair | data order | lean addressed read | control | Δ memory | lean WV | control WV | Δ WV | **lean BPB** | **control BPB** | **Δ BPB** |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| L1 vs Ctl1 | 20261009 | 17/40 | 20/40 | **−3** | 10 | 17 | **−7** | **1.26910** | 1.17425 | **+0.0949** |
| L2 vs Ctl2 | 20261010 | 16/40 | 20/40 | **−4** | 16 | 16 | 0 | **1.27659** | 1.17003 | **+0.1066** |
| L3 vs Ctl3 | 20261011 | 17/40 | 17/40 | 0 | **8** | 14 | **−6** | **1.27700** | pending | ≈ +0.107 |
| **aggregate** | — | **50/120** | **57/120** | **−7** | **34** | **47** | **−13** | — | — | — |

- **PRIMARY (failure mode): MET** — −7 and −6 on two seeds, the third 0; aggregate wrong-value **47 → 34 of 120**,
  identical to the fat configuration's gain.
- **SECONDARY: BREACHED, twice** — BPB **+0.095 / +0.107** against a **0.02** allowance, and the headline **−7 of 120**
  (per-seed −3, −4, 0) where the bar required it not below the noise on two of three seeds.

## The ledger

| | control | fat (previous cycle) | **lean (this cycle)** |
|---|---:|---:|---:|
| parameters | 28,958,761 | 65,868,585 (**+127 %**) | **30,217,001 (+4.3 %)** |
| wall per training step | 0.052 s | 1.19 s (**20×**) | **0.12 s (2.4×)** |
| MACs/token at the replaced layer, context 384 | 1,769,472 | 173,056 (−90 %) | **173,056 (−90 %)** |
| MACs/token at that layer, context 4,096 | 6,029,312 | 173,056 (−97 %) | **173,056 (−97 %)** |
| wrong-value over 120 | 47 | **34** | **34** |
| memory over 120 | 57 | 57 | **50** |
| **float BPB** | 1.174 / 1.170 | *unavailable* | **1.269 / 1.277 (+0.10)** |

**The codebook was not the price.** Both of the last cycle's ledger costs fall away at 1/8 the codebook — and the
quality cost does not. It is the *mechanism as implemented* that costs ~0.1 BPB: replacing the read with a lookup makes
the model worse on general chat text while making it better on the stated-value failure mode.

## What this establishes

1. **The read does double duty.** It retrieves stated values *and* carries general language quality. An address that
   only retrieves is **not** a drop-in replacement: the panel's failure mode improves by 13 rows of 120 while general
   BPB degrades by ~0.10 — a trade that a panel-only metric would have read as a clean win (the previous cycle's KEEP).
2. **A ledger-driven iteration can expose what the ledger does not measure.** Cutting the codebook was the right move on
   the ledger's own columns and it worked — the costs the ledger named disappeared — but the *quality* column was the one
   the previous cycle had left pending, and that is where the real price was.
3. **Placement, codebook and cost are now all measured; quality is the binding constraint.** For the address to be worth
   its arithmetic saving it has to be added *beside* the read (augment, not replace) or trained long enough to absorb what
   the read was doing for the rest of the language — the two concrete next configurations.

## Limitations

- **Three seeds, 40 memory rows** (±3 rows) — the standing declared deviation from ≥ 120 rows.
- **The fat configuration's BPB remains unavailable**: its models died with a non-canonical pod volume before its
  evaluations ran (the tooling lesson the previous cycle recorded). This cycle's measurement is the best available
  evidence about the mechanism's BPB cost, and it is why the KEEP is downgraded rather than withdrawn — the *panel*
  result of the fat configuration stands as measured.
- **Ctl3's BPB is pending** at the time of writing; its panel reading (17/40, wrong-value 14) is the one used in the
  paired test.
- **Two configurations only**: no arm here adds the address *beside* the read, which is the hypothesis this result
  motivates.

## Cost

| | |
|---|---|
| pod | `6192mevykdjhtf`, 2 × RTX 5090: three lean fits at **245 s** each, three controls at **104–110 s**, six BPB evaluations ≈ $2.20 |
| laptop CPU | three v5 reply sweeps (64 rows, cap 64) |
| external | none beyond the pod; inside the ≤ 4 pods / ≤ $8/h caps |

## Evidence

- Arm reports, models and BPB reports: `/workspace/uor-r4/deepseek/leanaddr-20261011/` on the pod volume, pulled and
  bundled to cloud-store as `[BUNDLE]` (reports, replies and the lens used).
- Pre-registration and fitness review: [#2029 comment 6109367641](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6109367641).
