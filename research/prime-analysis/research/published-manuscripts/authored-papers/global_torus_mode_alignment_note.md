# Global torus-mode alignment test at W=30030

Built global torus coordinates on the exact transport orbit:
- b = j mod 35
- f = floor(j/35) mod 143

For H in {13, 21}:
1. build the global spin transition operator,
2. extract the first nontrivial eigenvector,
3. compare it to low-order torus characters:
   exp(2pi i (kb*b/35 + kf*f/143))
for small kb,kf in {0,1,2,3}.

This tests whether the dominant global spin mode aligns with simple complex
phase modes on the torus-like phase/fiber state.
