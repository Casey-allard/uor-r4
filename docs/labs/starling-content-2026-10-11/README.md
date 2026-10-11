# Starling read, rung 3: a content gate on the kept 7 does not lift recall; the value path is suspect (11 October 2026)

**Milestone:** M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032).
**Line:** starling read · count **3/3** (pivot card follows) · headline "softmax at runtime: yes" → yes.
**Pre-registration:** [6112273931](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6112273931).
**Test fitness:** [FIT](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6112290954), an adversarial review covering completion and decision value.
**Blocker fix:** [6112460879](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6112460879).

## Question

Rung 2 ([record](../starling-anneal-2026-10-11/README.md)) found the following:
- The addresses learn: the true source is among the kept 7 neighbours 43–56% of the time.
- Recall still stays at chance.
- The gate weighted the kept sources by the **4-d address** dot, the same quantity that selected them, so it looked unable to tell them apart.

Does a **content** gate fix the readout?

## Change

`read_route=starling:M:K:content` gates the kept sources by σ(β⟨q_t,k_j⟩/√d + γ), using the read's existing query and key maps.
- **No new parameters.**
- Routing is unchanged: a 4-d address cell lookup, K positions scored, no softmax.
- The training helper keeps scoring by the address dot, so it still teaches the addresses.
- **Tests:**
  - content changes the logits;
  - the query and key gradients match finite differences;
  - the address maps keep a gradient with the helper on;
  - the route saves and loads;
  - gradients stay finite at β = 1e4.
- **Negative gates:** removing the content scores fails the test, and so does the old sigmoid form.

**Blocker, fixed before scoring.** The first attempt diverged in 5 of 6 runs near step 500, with a non-finite gradient norm. `1/(1+exp(−x))` overflows once the unbounded content logit passes about 88. Content routes now use ½(1 + tanh(x/2)); the address route is unchanged. All 6 runs were then re-run on one RTX 5090: 3 min, about $0.10.

## Result

The bench and seeds are the same as in rungs 1–2: context 512, 1,800 steps, `pattern=rarr key_shift=true`, width 128, 4 heads, `starling_anneal=0.5`.

| Arm | Recall per seed | Mean | Hit rate: step 0 → peak → final |
|---|---|---:|---|
| L, L2 softmax read (rung 1) | 0.669, 0.373, 0.809 | 0.617 | — |
| SA7, address gate (rung 2) | 0.0127, 0.0127, 0.0083 | 0.011 | 0.100 → 0.725 → 0.432 |
| **SC7**, `starling:4:7:content` | 0.0117, 0.0098, 0.0073 | **0.010** | 0.100 → 0.609 → **0.561** |
| **SC7w**, `starling:8:7:content` | 0.0107, 0.0073, 0.0059 | **0.008** | 0.099 → 0.469 → **0.465** |

Every run reached step 1,800, and held-out-class recall is 0.000 throughout.

**Decision: row 3, for both arms.**
- Recall is below row 2's bar (about 0.061).
- The final hit rate is well above H (about 0.20).

**What this shows:**
- **The content gate does not help.** The source is kept in about half of the queries, more often than in rung 2's final.
- **The gate was not the bottleneck.** Neither an address gate nor a content gate turns a kept source into a recalled value.
- **What remains is the value path:** the read of a kept source's value into the answer, the hit-rate metric's own leniency, or the per-head availability of the source.
  - The hit rate counts a source as kept if any head in any layer kept it (FIT nit 3), so the per-head availability may be much lower.
  - On a content route the address maps also stop learning after step 900 (FIT nit 1). That is visible: the peak-to-final drop is small here.

## Next

The line is at 3/3, so a pivot card on #2032 decides it: one decisive model-changing run, or stop (stopping needs the owner).

The pre-registered row-3 next step is an **oracle readout**: give the read the true source, to separate the value path from selection. It is a diagnostic, so under D21 it cannot be the decisive run by itself.

## Files

- [`score.py`](score.py) and [`result.json`](result.json). Logs are in cloud-store `claude/starling-content-20261011`; rung 2 is in `starling-anneal-20261011`.
