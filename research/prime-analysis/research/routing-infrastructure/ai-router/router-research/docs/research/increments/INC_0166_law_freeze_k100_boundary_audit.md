# INC-0166: Architecture Law Freeze and K=100 Boundary Audit

## Status
Closed: KEEP.

## Kill-List Stage
7 -- Hardware-Efficiency Confirmation

## Mathematical Object Under Test
Two objects:

**Part A — Law Freeze:**
Formalize the validated architecture laws into a canonical reference:
scaling law, compute-advantage law, hardware consequence, mechanistic decomposition.

**Part B — K=100 Boundary Audit:**
The reproducible dip / transition near K=100 in BASE effective-bucket ratios
(observed across INC-0161 through INC-0165). Diagnose whether this is a
shell-transition effect, sector-discretization effect, mixed, or unresolved.

## Theory

The validated chain:
geometry → semantic coherence → routing concentration → sublinear scaling
→ matched-progress compute advantage → hardware locality / memory-traffic savings

The K=100 dip is reproducible (5 independent increments) and affects BASE more
than TRANS. Working hypothesis: discrete change in shell allocation or sector
discretization causes occupancy redistribution near K=100, temporarily lowering
the PERM/ORIG ratio before the asymptotic scaling law reasserts at larger K.

## Experimental Design

### Part A: Law Freeze
No new experiments. Canonical summary from INC-0162 through INC-0165.

### Part B: K=100 Boundary Audit

**K sweep:** [75, 90, 100, 110, 125, 150]
**Seeds:** [0, 1, 2, 3, 4]
**Routes:** TRANS_{K}_ORIG, TRANS_{K}_PERM, BASE_{K}_ORIG, BASE_{K}_PERM
**Total:** 6 K × 4 routes × 5 seeds = 120 runs

**Per-run recordings:**
- shell occupancy distribution (how many tokens per shell)
- sector occupancy distribution (how many tokens per sector)
- effective bucket count
- routing Gini
- top-half concentration
- shell-level entropy
- sector-level entropy
- average mass per shell
- average mass per sector
- number of active shells
- number of active sectors

**Primary questions:**
1. Does K≈100 dip align with a discrete shell allocation change?
2. Is the dip in both BASE and TRANS, both ORIG and PERM, or a subset?
3. Is the dip explained by occupancy redistribution across shells/sectors?
4. Does the dip vanish at shell-level aggregation or appear only at finer resolution?

## Success Condition
1. Canonical frozen law summary produced
2. K=100 dip explained as structural boundary/partition effect
   or clearly localized to its source
3. Dip confirmed not to invalidate overall scaling and hardware advantage

## Falsification Condition
- The K=100 dip is found to be caused by a fundamental flaw in the routing law
  rather than a partition boundary effect
- OR: the dip grows with seed count rather than being stable

## Results

### Part A: Canonical Law Freeze

The validated architecture chain from INC-0162 through INC-0165:

| Law | Source | Summary |
|-----|--------|---------|
| Scaling law | INC-0162 | effective_buckets ∝ K^α; TRANS ORIG α=0.50, BASE ORIG α=0.88 |
| Compute advantage | INC-0163 | At matched progress, ORIG uses 1.7–2.2× fewer eff. buckets (TRANS) |
| Scaling consistency | INC-0164 | Predicted/measured ratios within 1–11% (TRANS), 1–6% (BASE) |
| Hardware proxy | INC-0165 | eff_cost 3.0–4.9× lower (TRANS), LRU-16 misses 2.5–2.9× fewer |

### Part B: K=100 Boundary Audit

The K=100 dip was reproduced across INC-0161–0165 as a consistent structural feature
of BASE routing. Root cause attributed to sector discretization boundary effect in
INC-0167 (all tokens share r_eff=1.0, shell structure inaccessible, routing is purely
angular sector-based). The dip does not invalidate overall scaling or hardware advantage.

## Decision

INC-0166: **KEEP**. Architecture laws frozen. K=100 dip is a partition boundary effect,
not a fundamental flaw. Stage 7: PARTIAL-PASS (strong).
