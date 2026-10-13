# Prime Transport: State Construction Encoding Probe v1

**Branch**: state_construction_encoding_probe_v1

## State Construction Lock

### 1. Functions that construct tau_init

**`convert_onehot_to_angular_multi(tau0_oh, blocks)`**:
For block 3 (s=9, e=21, m=12, n_h=3), `k_idx = argmax(onehot[9:21])` in [0..11]:

| harmonic | indices | formula | period |
|----------|---------|---------|--------|
| h=1 | 6,7 | cos(2πk/12), sin(2πk/12) | 12 |
| h=2 | 8,9 | cos(πk/3), sin(πk/3) | 6 |
| h=3 | 10,11 | cos(πk/2), sin(πk/2) | **4** |

**`apply_anchor_two_i`**: rotates (cos,sin) → (−sin,cos). After rotation:
- H3 at [10,11]: `(−sin(πk/2), cos(πk/2))` — encodes k%4
- H2 at [8,9]:  `(−sin(πk/3), cos(πk/3))` — encodes k%6
- H1 at [6,7]:  `(−sin(πk/6), cos(πk/6))` — **replaced** each transport step

**`tau0_hyb = cat([tau0_ang, ones(n_blocks)])`**

### 2. H2/H3 carrier components

Both frozen by `eps_high=1.0` (h≥2 harmonics: blend = 1.0×prev):
- **H2** (indices 8,9): encodes k%6, period 6
- **H3** (indices 10,11): encodes k%4, period 4

### 3. Previously solved trivially

| Target | Readout | Accuracy |
|--------|---------|----------|
| original_s42 (k%4+0) | H3 phase | 1.0000 |
| shift1_s42 (k%4+1) | H3 phase | 1.0000 |
| k%6 | H2 phase | 1.0000 |
| k%12 | CRT(H2,H3) | 1.0000 |

### 4. Hypothesis definitions

**Explicit encoding**: `h=3` in a period-12 block directly produces a period-4 component.
Chain: `k → πk/2 → H3=(−sin(πk/2),cos(πk/2)) → atan2 → k%4`.
Target is precomputed at initialization. No transport needed.

**Emergent mapping**: label not computable from any single frozen harmonic;
requires dynamics or non-trivial multi-component interaction.

---

## 4-Case Results

| Case | Description | acc | H3 survives | direct_readable | max_formula_err | runtime_s |
|------|-------------|-----|-------------|-----------------|-----------------|-----------|
| A | Formula audit (D=0) | 1.0000 | True | True | 1.32e-06 | 0.032s |
| B | H3 ablated in construction | 0.2520 | False | False | 0.00e+00 | 0.204s |
| C (H3 only) | Permuted target, H3 readout | 0.2578 | True | — | — | — |
| C (CRT) | Permuted target, CRT readout | 1.0000 | True | True | — | 0.169s |
| D | Soft-onehot perturbation | N/A | True | True | 5.74e-07 | 0.453s |

### Case A — Dependency chain

```
k=argmax(onehot[9:21]) -> angle_h3=2*pi*3*k/12=pi*k/2 [written by convert_onehot_to_angular_multi: block3 h=3 m=12] -> apply_anchor_two_i: (cos,sin)->(-sin,cos) -> H3=(-sin(pi*k/2), cos(pi*k/2)) -> atan2(H3)=-pi*k/2 -> k%4=round(-atan2/(pi/2))%4
```

**Formula match error (max over all k in [0..11])**: `1.32e-06`
(machine precision — H3 coordinates match formula exactly)

**H3 readout at D=0** (no trajectory at all): acc = `1.0000`
The answer is present before the first transport step.

### Case B — Ablation

CASE B: h=3 harmonic zeroed in convert_onehot_to_angular_multi. H3 max abs in ablated tau_init=0.00e+00. acc=0.2520 after ablation (expected ~0.25 baseline). Transport and readout unchanged. Carrier disappears when construction term is removed.

Accuracy without H3 carrier: `0.2520` ≈ random baseline (1/VOCAB=0.25)

### Case C — Permuted target

CASE C: permuted per-k target (not aligned to k%4). perm_labels=[0, 3, 2, 1, 3, 2, 1, 0, 1, 2, 0, 3]. k%4 agreement in perm: 3/12. H3-only acc on perm target=0.2578 (expected ~0.25). CRT+perm_lut acc=1.0000 (expected ~1.0). Construction encodes full k; any per-k target trivially readable.

- H3 readout on permuted target: `0.2578` (fails — wrong period)
- CRT readout (k%12 → perm_lut): `1.0000` (succeeds — full k encoded)

Shows the construction encodes **full k** (12-state identity), not just k%4.
Any per-k label function is trivially readable from (H2,H3).

### Case D — Input perturbation

CASE D: soft one-hot interpolation. Mean H3-angle error vs formula: 4.32e-07 rad. Max error across alpha steps: 5.74e-07 rad. Per-alpha: [alpha=0.0:err=3.62e-07; alpha=0.1:err=4.19e-07; alpha=0.2:err=5.10e-07; alpha=0.3:err=3.98e-07; alpha=0.4:err=5.60e-07; alpha=0.5:err=3.56e-07; alpha=0.6:err=5.74e-07; alpha=0.7:err=3.33e-07; alpha=0.8:err=5.26e-07; alpha=0.9:err=3.53e-07; alpha=1.0:err=3.62e-07]. H3 shifts linearly with alpha per explicit formula. Confirms: construction writes label directly.

Max H3-angle error vs formula across all alpha: `5.74e-07` rad
(consistent with floating-point precision — H3 shifts exactly per formula)

---

## Conclusion

**STATE CONSTRUCTION STATUS: EXPLICIT ENCODING**

---

## Honesty Section

### What is explicitly written at initialization

- **k%4** is written directly into H3 (indices 10,11) by the h=3 harmonic of the
  period-12 block. The formula `cos(2π·3·k/12) = cos(πk/2)` has period 4 by
  construction. This is not accidental: choosing h=3 in a 12-cycle produces
  a period-4 component, which matches VOCAB=4.
- **k%6** is written into H2 by the h=2 harmonic (period 6).
- **k%12 = k** is recoverable from H2+H3 via CRT, meaning the full cycle
  identity is encoded at initialization.

### What is NOT written

- H1 (indices 6,7) is NOT preserved — replaced every transport step.
- Nothing about the transport dynamics contributes to correctness for k%4.
- No learned weights are used.

### Is this clever encoding or genuinely revealing mapping?

**This is clever encoding, not a genuinely revealing mapping.**

The mechanism is: the state construction formula includes a harmonic whose
period matches the task period. The target (k%4) is a direct mathematical
consequence of how the h=3 harmonic of a 12-cycle was chosen.

The readout does not discover anything about the input — it extracts a
pre-written label. The 'geometry' is a convenient coordinate system for
label storage, not a representation that makes the label emergent.

There is no evidence for emergent mapping: all four cases confirm the label
is written at construction time and extracted trivially.

CASE A: formula error = 1.32e-06 (machine precision match).
CASE B: acc collapses from 1.0 to 0.2520 when construction term removed.
CASE C: H3-only acc = 0.2578 on non-period-4 target (fails); CRT acc = 1.0000 (succeeds via full-k encoding).
CASE D: H3-angle error vs formula = 5.74e-07 rad (explicit linear shift).