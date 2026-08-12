# Layered angle-stack vs raw coarse-state prediction test

This experiment compares how well different state descriptions predict
near-future quadruplet admissibility behavior inside multiple coarse routes mod 30030.

Target:
- H-step future admissibility signature of quadruplet survival
- H in {1, 3, 5}

Models:
- raw coarse route only (proxy for fixed-scale raw coarse (b,phi))
- x17 only
- x17,x19
- x17,x19,x23
- route + x17,x19,x23

Question:
Does the layered angle stack carry more predictive continuation information
than the raw coarse state alone?
