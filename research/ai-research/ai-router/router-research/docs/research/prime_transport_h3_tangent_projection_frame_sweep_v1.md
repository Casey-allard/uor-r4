# H3 Tangent Projection Frame Sweep — v1

## 1. Mechanism Lock

### State
- `tau_hyb in R^{N x 16}`: 12 angular dims (6 harmonic pairs, unit-normalised) + 4 magnitude dims.
- `apply_anchor_two_i` active: `(cos t, sin t) -> (-sin t, cos t)` for each pair.
- H3 pair (dims 10-11). Class k in 2i frame:
  - k=0: [0, 1], k=1: [-1, 0], k=2: [0, -1], k=3: [1, 0]

### Prototype
- `build_class_prototypes_h3(blocks, apply_2i=True)` -> (4, 2) tensor.
- **Both state and prototype in the same 2i-rotated frame. Zero frame mismatch.**

### Projection Operators

| Operator | Formula | Range | Notes |
|----------|---------|-------|-------|
| cos(t) | u.v = u[0]*v[0] + u[1]*v[1] | [-1, +1] | 1=aligned, 0=ortho, -1=anti |
| sin(t) | u[0]*v[1] - u[1]*v[0] | [-1, +1] | signed; 0=aligned/anti |
| tan(t) | sin/cos (NaN if abs(cos)<0.01) | (-inf,+inf) | unstable at +/-90 deg |

### Analytic Predictions (t=0, exact prototypes)

| class_dist | relation | cos | sin (signed) | |sin| | tan | tan_stable |
|------------|----------|-----|--------------|-------|-----|------------|
| 0 | matched | +1.0 | 0.0 | 0.0 | 0.0 | yes |
| 1 | adjacent +1 | 0.0 | +1.0 | 1.0 | NaN | no |
| 2 | opposite | -1.0 | 0.0 | 0.0 | 0.0 | yes |
| 3 | adjacent -1 | 0.0 | -1.0 | 1.0 | NaN | no |

**Key structural property**: cos and sin are complementary.
- cos discriminates matched vs opposite; loses at +-90 deg (adjacent classes).
- signed sin discriminates adj+1 vs adj-1; loses at 0/180 deg (matched/opposite).
- tan = sin/cos: inherits both instabilities; no independent information.

---

## 2. Matched vs Mismatched Tables (Cases A and B)

### D=1, eps_high=1.0, t=0 (initial state)

| sub_case | class_dist | cos | sin (signed) | |sin| | tan | tan_unstable |
|----------|------------|-----|--------------|-------|-----|--------------|
| matched        | 0 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 |
| adj_plus1      | 1 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 |
| opposite       | 2 | -1.0000 | -0.0000 | +0.0000 | +0.0000 | +0.0000 |
| adj_minus1     | 3 | +0.0000 | -1.0000 | +1.0000 | NaN | +1.0000 |

### D=5, eps_high=1.0, t=0 (initial state)

| sub_case | class_dist | cos | sin (signed) | |sin| | tan | tan_unstable |
|----------|------------|-----|--------------|-------|-----|--------------|
| matched        | 0 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 |
| adj_plus1      | 1 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 |
| opposite       | 2 | -1.0000 | -0.0000 | +0.0000 | +0.0000 | +0.0000 |
| adj_minus1     | 3 | +0.0000 | -1.0000 | +1.0000 | NaN | +1.0000 |

### D=20, eps_high=1.0, t=0 (initial state)

| sub_case | class_dist | cos | sin (signed) | |sin| | tan | tan_unstable |
|----------|------------|-----|--------------|-------|-----|--------------|
| matched        | 0 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 |
| adj_plus1      | 1 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 |
| opposite       | 2 | -1.0000 | -0.0000 | +0.0000 | +0.0000 | +0.0000 |
| adj_minus1     | 3 | +0.0000 | -1.0000 | +1.0000 | NaN | +1.0000 |

### D=20, eps_high=1.0, t=D (final state)

| sub_case | class_dist | cos | sin (signed) | |sin| | tan | tan_unstable |
|----------|------------|-----|--------------|-------|-----|--------------|
| matched        | 0 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 |
| adj_plus1      | 1 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 |
| opposite       | 2 | -1.0000 | -0.0000 | +0.0000 | +0.0000 | +0.0000 |
| adj_minus1     | 3 | +0.0000 | -1.0000 | +1.0000 | NaN | +1.0000 |

### D=1, eps_high=0.0, t=0 (initial state)

| sub_case | class_dist | cos | sin (signed) | |sin| | tan | tan_unstable |
|----------|------------|-----|--------------|-------|-----|--------------|
| matched        | 0 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 |
| adj_plus1      | 1 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 |
| opposite       | 2 | -1.0000 | -0.0000 | +0.0000 | +0.0000 | +0.0000 |
| adj_minus1     | 3 | +0.0000 | -1.0000 | +1.0000 | NaN | +1.0000 |

### D=5, eps_high=0.0, t=0 (initial state)

| sub_case | class_dist | cos | sin (signed) | |sin| | tan | tan_unstable |
|----------|------------|-----|--------------|-------|-----|--------------|
| matched        | 0 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 |
| adj_plus1      | 1 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 |
| opposite       | 2 | -1.0000 | -0.0000 | +0.0000 | +0.0000 | +0.0000 |
| adj_minus1     | 3 | +0.0000 | -1.0000 | +1.0000 | NaN | +1.0000 |

### D=20, eps_high=0.0, t=0 (initial state)

| sub_case | class_dist | cos | sin (signed) | |sin| | tan | tan_unstable |
|----------|------------|-----|--------------|-------|-----|--------------|
| matched        | 0 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 |
| adj_plus1      | 1 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 |
| opposite       | 2 | -1.0000 | -0.0000 | +0.0000 | +0.0000 | +0.0000 |
| adj_minus1     | 3 | +0.0000 | -1.0000 | +1.0000 | NaN | +1.0000 |

### D=20, eps_high=0.0, t=D (final state)

| sub_case | class_dist | cos | sin (signed) | |sin| | tan | tan_unstable |
|----------|------------|-----|--------------|-------|-----|--------------|
| matched        | 0 | +0.7695 | +0.2305 | +0.2305 | +0.0000 | +0.2305 |
| adj_plus1      | 1 | -0.2305 | +0.7695 | +0.7695 | +0.0000 | +0.7695 |
| opposite       | 2 | -0.7695 | -0.2305 | +0.2305 | +0.0000 | +0.2305 |
| adj_minus1     | 3 | +0.2305 | -0.7695 | +0.7695 | +0.0000 | +0.7695 |

---

## 3. Operator Separation (matched minus avg mismatched)

| D | eps_high | t | cos_sep | sin_sep | |sin|_sep | tan_sep |
|---|----------|---|---------|---------|-----------|---------|
| 1 | 1.0 | 0 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 1 | 1.0 | 1 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 5 | 1.0 | 0 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 5 | 1.0 | 1 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 5 | 1.0 | 2 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 5 | 1.0 | 3 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 5 | 1.0 | 4 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 5 | 1.0 | 5 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 20 | 1.0 | 0 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 20 | 1.0 | 1 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 20 | 1.0 | 2 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 20 | 1.0 | 3 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 20 | 1.0 | 4 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 20 | 1.0 | 5 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 20 | 1.0 | 6 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 20 | 1.0 | 7 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 20 | 1.0 | 8 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 20 | 1.0 | 9 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 20 | 1.0 | 10 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 20 | 1.0 | 11 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 20 | 1.0 | 12 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 20 | 1.0 | 13 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 20 | 1.0 | 14 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 20 | 1.0 | 15 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 20 | 1.0 | 16 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 20 | 1.0 | 17 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 20 | 1.0 | 18 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 20 | 1.0 | 19 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 20 | 1.0 | 20 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 1 | 0.0 | 0 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 1 | 0.0 | 1 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 5 | 0.0 | 0 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 5 | 0.0 | 1 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 5 | 0.0 | 2 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 5 | 0.0 | 3 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 5 | 0.0 | 4 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 5 | 0.0 | 5 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 20 | 0.0 | 0 | +1.3333 | +0.0000 | -0.6667 | -0.0000 |
| 20 | 0.0 | 1 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 20 | 0.0 | 2 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 20 | 0.0 | 3 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 20 | 0.0 | 4 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 20 | 0.0 | 5 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 20 | 0.0 | 6 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 20 | 0.0 | 7 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 20 | 0.0 | 8 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 20 | 0.0 | 9 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 20 | 0.0 | 10 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 20 | 0.0 | 11 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 20 | 0.0 | 12 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 20 | 0.0 | 13 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 20 | 0.0 | 14 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 20 | 0.0 | 15 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 20 | 0.0 | 16 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 20 | 0.0 | 17 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 20 | 0.0 | 18 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 20 | 0.0 | 19 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |
| 20 | 0.0 | 20 | +1.0260 | +0.3073 | -0.3594 | -0.0000 |

---

## 4. Success vs Failure Tables (Case C)

### D=1, eps_high=1.0

| sub_case | timestep | cos | sin (signed) | |sin| | tan | tan_unstable | n |
|----------|----------|-----|--------------|-------|-----|--------------|---|
| success   | 0 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 1 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |

### D=5, eps_high=1.0

| sub_case | timestep | cos | sin (signed) | |sin| | tan | tan_unstable | n |
|----------|----------|-----|--------------|-------|-----|--------------|---|
| success   | 0 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 1 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 2 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 3 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 4 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 5 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |

### D=20, eps_high=1.0

| sub_case | timestep | cos | sin (signed) | |sin| | tan | tan_unstable | n |
|----------|----------|-----|--------------|-------|-----|--------------|---|
| success   | 0 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 1 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 2 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 3 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 4 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 5 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 6 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 7 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 8 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 9 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 10 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 11 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 12 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 13 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 14 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 15 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 16 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 17 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 18 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 19 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 20 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |

### D=1, eps_high=0.0

| sub_case | timestep | cos | sin (signed) | |sin| | tan | tan_unstable | n |
|----------|----------|-----|--------------|-------|-----|--------------|---|
| success   | 0 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 1 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 1 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |

### D=5, eps_high=0.0

| sub_case | timestep | cos | sin (signed) | |sin| | tan | tan_unstable | n |
|----------|----------|-----|--------------|-------|-----|--------------|---|
| success   | 0 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 1 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 1 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |
| success   | 2 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 2 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |
| success   | 3 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 3 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |
| success   | 4 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 4 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |
| success   | 5 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 5 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |

### D=20, eps_high=0.0

| sub_case | timestep | cos | sin (signed) | |sin| | tan | tan_unstable | n |
|----------|----------|-----|--------------|-------|-----|--------------|---|
| success   | 0 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 1024 |
| success   | 1 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 1 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |
| success   | 2 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 2 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |
| success   | 3 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 3 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |
| success   | 4 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 4 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |
| success   | 5 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 5 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |
| success   | 6 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 6 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |
| success   | 7 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 7 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |
| success   | 8 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 8 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |
| success   | 9 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 9 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |
| success   | 10 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 10 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |
| success   | 11 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 11 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |
| success   | 12 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 12 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |
| success   | 13 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 13 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |
| success   | 14 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 14 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |
| success   | 15 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 15 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |
| success   | 16 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 16 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |
| success   | 17 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 17 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |
| success   | 18 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 18 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |
| success   | 19 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 19 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |
| success   | 20 | +1.0000 | +0.0000 | +0.0000 | +0.0000 | +0.0000 | 788 |
| failure   | 20 | -0.0000 | +1.0000 | +1.0000 | NaN | +1.0000 | 236 |

---

## 5. Stability Analysis (Tangent Operator)

tan becomes NaN when |cos| < 0.01 (threshold TAN_EPS).

| D | eps_high | sub_case | class_dist | t | tan_unstable_frac |
|---|----------|----------|------------|---|-------------------|
| 1 | 1.0 | matched | 0 | 1 | +0.0000 |
| 1 | 1.0 | adj_plus1 | 1 | 1 | +1.0000 |
| 1 | 1.0 | opposite | 2 | 1 | +0.0000 |
| 1 | 1.0 | adj_minus1 | 3 | 1 | +1.0000 |
| 5 | 1.0 | matched | 0 | 5 | +0.0000 |
| 5 | 1.0 | adj_plus1 | 1 | 5 | +1.0000 |
| 5 | 1.0 | opposite | 2 | 5 | +0.0000 |
| 5 | 1.0 | adj_minus1 | 3 | 5 | +1.0000 |
| 20 | 1.0 | matched | 0 | 20 | +0.0000 |
| 20 | 1.0 | adj_plus1 | 1 | 20 | +1.0000 |
| 20 | 1.0 | opposite | 2 | 20 | +0.0000 |
| 20 | 1.0 | adj_minus1 | 3 | 20 | +1.0000 |
| 1 | 0.0 | matched | 0 | 1 | +0.2305 |
| 1 | 0.0 | adj_plus1 | 1 | 1 | +0.7695 |
| 1 | 0.0 | opposite | 2 | 1 | +0.2305 |
| 1 | 0.0 | adj_minus1 | 3 | 1 | +0.7695 |
| 5 | 0.0 | matched | 0 | 5 | +0.2305 |
| 5 | 0.0 | adj_plus1 | 1 | 5 | +0.7695 |
| 5 | 0.0 | opposite | 2 | 5 | +0.2305 |
| 5 | 0.0 | adj_minus1 | 3 | 5 | +0.7695 |
| 20 | 0.0 | matched | 0 | 20 | +0.2305 |
| 20 | 0.0 | adj_plus1 | 1 | 20 | +0.7695 |
| 20 | 0.0 | opposite | 2 | 20 | +0.2305 |
| 20 | 0.0 | adj_minus1 | 3 | 20 | +0.7695 |

---

## 6. Critical Questions

### Q1: Does cosine remain sufficient once fully aligned?
- cos separation at D=20, eps=0.0, t=20: +1.0260
- |sin| separation at D=20, eps=0.0, t=20: -0.3594
- By construction at t=0: cos and sin are complementary (class_dist=1 -> cos=0 but sin=1).
- Whether cos alone is sufficient depends on routing dynamics.

### Q2: Does sine capture signal that cosine suppresses?
- At class_dist=1 and class_dist=3 (adjacent mismatches): cos=0, sin=+-1.
- sin provides positive signal precisely where cos fails.
- signed sin also distinguishes adjacent_+1 from adjacent_-1 (which cos cannot).

### Q3: Does tangent encode higher-order structure?
- tan = sin/cos is a derived quantity; no independent information.
- tan is undefined at the most informative adjacent mismatch cases.
- tan provides no advantage over (cos, sin) pair.

### Q4: Which operator provides the strongest and most stable discrimination?
- cos: best for matched vs opposite discrimination; stable everywhere except cos=0.
- sin: best for adjacent mismatch discrimination; stable where cos fails.
- tan: unstable at key mismatch cases; adds no new information.
- Combined (cos, sin): full discrimination of all 4 discrete classes.

---

## 7. Structural Differentiation Summary

| Operator | Matched | Adjacent | Opposite | Stable | Independent |
|----------|---------|----------|----------|--------|-------------|
| cos | +1.0 | 0.0 | -1.0 | yes | yes |
| signed sin | 0.0 | +-1.0 | 0.0 | yes (at 0/180) | yes |
| tan | 0.0 | NaN | 0.0 | no (at +-90) | no (derived) |

- sin provides signal where cos = 0 (adjacent mismatch cases).
- cos provides signal where sin = 0 (opposite mismatch cases).
- tan is unstable at exactly the cases where sin has maximum signal.
- Operators are NOT identical in behaviour across all regimes.

---

## 8. Final Conclusion

**Evidence**:
- cos and sin are provably complementary: each captures signal the other suppresses.
- sin is NOT redundant: it has maximum signal (+/-1.0) at class_dist=1 and 3,
  exactly where cos has zero signal (0.0). This is structural differentiation.
- tan provides no independent information; it is derived from cos and sin,
  and is undefined at the most informative (adjacent mismatch) regime.
- Structural differentiation between operators is confirmed both analytically
  (from exact 2i-frame geometry) and empirically (from routing trajectory data).

**Interpretation**:
- sin reveals a directional component that cos suppresses entirely at +-90 deg.
- This is not a weak or marginal effect -- it is the maximal possible separation (1.0 vs 0.0).
- The tangent operator introduces instability without new discriminative signal.

H3 PROJECTION FRAME SWEEP STATUS: SUPPORT
