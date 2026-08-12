# Riemann Hypothesis in Lean (with or without mathlib)

By Brandon Gomes and Alex Kontorovich

Department of Mathematics, Rutgers University

This repository contains the formalization of the Riemann Hypothesis (in terms of the Dirichlet eta function) in the Lean Interactive Theorem Prover.

The documents `basic`, `cauchy`, `complex`, `dirichlet_eta`, `exponential`, `riemann_hypothesis`, and `riemann_zeta` are standalone files not using mathlib. They assume certain properties of real and complex numbers, and certain (not entirely trivial) facts about the real/complex logarithm and exponential functions.

The (equivalent) formulation of the Riemann hypothesis stated in `riemann_hypothesis` is: if 0<Re(s)<1 and DirichletEta(s)=0, then Re(s)=1/2.

We also give a file, `impl`, which is a witness that these properties can be proved within mathlib (including lemmata on real/complex log/exp functions which may be of independent interest in mathlib).

We would like to take this opportunity to thank Alex Best, Kevin Buzzard, Colin Fan, Sebastien Gouezel, Heather Macbeth, and the Zulip Lean community for many suggestions that made this file possible. The work of the first-named author was supported by the Rutgers Math Department through the DIMACS REU, and both are supported in part by the second-named author's NSF grant DMS-1802119.
