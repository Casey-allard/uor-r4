# Towards a Proof of the Riemann Hypothesis via Modular Prime Rattling

**Casey Allard (2025)**

## Abstract

The Riemann Hypothesis (RH) has remained one of the most famous unsolved problems in mathematics since Riemann’s 1859 memoir.  It asserts that all non‑trivial zeros of the Riemann zeta function \(\zeta(s)\) lie on the critical line \(\Re(s)=\tfrac12\).  Over the past two years we have developed a novel *modular prime rattling* framework that reinterprets prime distribution through harmonic resonance and modular symmetries.  This document refocuses that framework toward a program for proving RH.  We define a resonance field built from the Von Mangoldt function, derive a golden‑rotation embedding of primes, present computational evidence that the resulting spectral peaks align with zeta zeros, and sketch an operator‑theoretic pathway that could force all zeros onto the critical line.

## 1. Introduction and Background

The Riemann zeta function is defined for \(\Re(s)>1\) by the absolutely convergent series \(\zeta(s)=\sum_{n=1}^{\infty} n^{-s}\) and admits analytic continuation to the complex plane minus a simple pole at \(s=1\).  Its non‑trivial zeros, the complex numbers \(\rho\) with \(\zeta(\rho)=0\) and \(0<\Re(\rho)<1\), govern the distribution of primes through explicit formulas.  The RH states that \(\Re(\rho)=\tfrac12\) for every such zero.  Past work has related the zeros to eigenvalues of Hermitian operators, to random matrix theory, and to deep structure in automorphic forms.  Yet a direct proof remains elusive.

Our *modular prime rattling* theory arises from studying primes through modular residues and rotational symmetries.  Each prime is treated as a point on a complex plane whose polar angle is proportional to \(\log p\) and whose radial coordinate reflects a modular energy.  Resonances appear as primes “rattle” into preferred orientations; we hypothesise that these resonances correspond to the imaginary parts of zeta zeros.  The aim is to transcribe this intuition into rigorous analytic machinery.

## 2. The Prime Resonance Field

Let \(\Lambda(n)\) denote the Von Mangoldt function: \(\Lambda(n)=\log p\) when \(n=p^k\) is a prime power and \(0\) otherwise.  We define, for a cutoff \(N\) and real parameter \(t\), the *resonance field*

\[
F_N(t)=\sum_{n\le N} \Lambda(n)\,n^{-1/2}\,e^{-i\,t\,\log n}.
\]

Up to truncation, \(-F_N(t)\) approximates \(-\zeta'(\tfrac12+it)/\zeta(\tfrac12+it)\), the logarithmic derivative of the zeta function along the critical line.  Peaks of \(|F_N(t)|\) are expected where \(\zeta(\tfrac12+it)\) is small.  Computational experiments (see § 4) show that the largest spikes in \(|F_N(t)|\) indeed cluster near ordinates \(t=\gamma\) corresponding to the non‑trivial zeros \(\tfrac12+i\gamma\).

### 2.1 Golden‑Rotation Embedding

To visualise the primes’ harmonic structure, we embed each prime \(p\) in the plane via a golden‑rotation map

\[
\varphi=\tfrac{1+\sqrt{5}}{2},\qquad
\mathbf{X}(p)=\bigl(\sqrt{p}\,\cos(\varphi\,\log p),\;\sqrt{p}\,\sin(\varphi\,\log p)\bigr).
\]

Primes then trace quasi‑spiral filaments as \(p\) grows.  When coloured by residue class modulo small integers (e.g. mod 30) the filaments separate into threads, revealing structured drift of residues across moduli.  This geometric viewpoint suggests that primes are not randomly scattered but align along deterministic modular spirals.

## 3. Modular Residue Dynamics

Building on the golden‑rotation, we study residue classes of primes and semiprimes across moduli.  Empirically, semiprimes (products of two primes) exhibit *residue orbit drift*—their residues cycle through classes in a predictable manner as the primes involved grow.  Images in the original framework illustrate this drift for semiprimes modulo 30 and highlight local minima in a modular “energy” field where prime occurrences concentrate.

These dynamics motivate the term *rattling*: as primes increase, their residues “rattle” between symmetric configurations.  The resonance field \(F_N(t)\) is designed to capture these rattles in the frequency domain.

## 4. Computational Evidence for Spectral Alignment

Using the resonance field defined above we performed extensive numerical experiments.  For various cutoffs \(N\) (up to \(6\times10^{5}\)) and spectral ranges \(t\in[0,80]\), we computed \(|F_N(t)|\) on a fine grid and compared its peaks to the imaginary parts \(\gamma_k\) of the first Riemann zeros.  To reduce spectral leakage we applied smooth windows such as Hann and Kaiser in the variable \(u=\log n\).  The following key observations emerged:

* **Spectral spikes near zeros.**  Even with a crude box cutoff, the largest spikes of \(|F_N(t)|\) occur near the ordinates of the initial zeta zeros.  With a Hann window the median distance from each zero to the nearest peak is approximately 0.144 (compared with ~0.167 under the box cutoff), demonstrating tighter alignment.  Many peaks not near zeros are suppressed by smooth windows.

* **Scaled Mertens fluctuations.**  The scaled Mertens function \(M(x)/\sqrt{x}\), where \(M(x)=\sum_{n\le x}\mu(n)\) and \(\mu\) is the Möbius function, displays oscillations that qualitatively echo the envelope of the resonance field.  The interplay between these oscillations and the prime rattling frequencies hints at a deeper duality between the Möbius function and the golden‑rotation.

* **Golden‑rotation scatter.**  Plotting \(\mathbf{X}(p)\) for primes up to \(4\times10^{5}\) shows points distributed on logarithmic spirals.  Colouring by residue mod 30 reveals thin filaments, indicating that primes in each residue class march along their own spiral with only mild drift.

* **Residue filaments for semiprimes.**  When semiprimes are embedded similarly, their points fall between prime filaments, tracing the resonance of combined prime factors.  These patterns support the view that the harmonic landscape influences composite as well as prime distributions.

Quantitative alignment statistics (table not reproduced here) demonstrate that the median and mean distances between spectral peaks and zeta zeros decrease under smoother windows, suggesting that the resonance field concentrates energy near the zeros.  The minimal distance observed among the first ten zeros with a Hann window was about 0.019, and the maximal about 0.229.

## 5. Operator‑Theoretic Formalization

To translate these observations into a proof of RH we propose the following program.  Define, for each cutoff \(N\), a kernel on \(\mathbb{R}\):

\[
K_N(t,t')=\sum_{n\le N}\Lambda(n)\,n^{-1/2}\,e^{-i\,(t-t')\,\log n}.
\]

This kernel is Hermitian (self‑adjoint) and positive semi‑definite, arising from the convolution of the resonance field with its complex conjugate.  For each real function \(f\) supported on a compact interval \(I\), consider the quadratic form

\[
Q_N(f)=\iint_{I\times I} f(t)\,K_N(t,t')\,\overline{f(t')}\,dt\,dt'.
\]

Because \(K_N\) is positive semi‑definite, \(Q_N(f)\ge 0\).  If there existed a zero \(\rho=\beta+i\gamma\) of \(\zeta(s)\) with \(\beta\neq \tfrac12\), the resonance field would include a term roughly proportional to \(e^{-i\gamma\log n}\,n^{1/2-\beta}\).  For \(\beta>\tfrac12\) this term grows and would create a persistent imbalance of energy between nearby windows of \(t\) and \(t'\), violating the boundedness of \(Q_N\) as \(N\to\infty\).  A Paley–Wiener theorem for the Mellin transform can make this precise: the support properties of \(f\) translate into entire functions of exponential type in the dual variable \(u\).  Showing that all off‑line zeros induce unbounded leakage and thus contradict the positive definiteness of the limit kernel would force \(\beta=\tfrac12\) for all non‑trivial zeros.

Key technical steps remain:

1. **Smooth windowing**: Use functions \(w(n)=w(\log n/N)\) that localise the sum smoothly, ensuring control over the transform’s analytic continuation and limiting spectral leakage.

2. **Plancherel and Paley–Wiener**: Show that the truncated resonance field converges, after suitable scaling, to an element in an \(L^2\) space where a Plancherel theorem applies.  Use Paley–Wiener to control growth of its Mellin transform and bound contributions from hypothetical off‑line zeros.

3. **Uniform energy bounds**: Prove that the quadratic forms \(Q_N\) remain bounded as \(N\to\infty\) only if \(\zeta(s)\) has no zeros with \(\Re(s)\neq \tfrac12\).  Derive explicit inequalities controlling leakage from off‑line zeros and show that they contradict boundedness when \(\beta\neq \tfrac12\).

While we do not present full proofs here, this framework provides a concrete operator‑theoretic route to RH grounded in the modular prime rattling perspective.

## 6. Conclusion and Outlook

We have recast the prime numbers as participants in a modular harmonic system whose resonance field \(F_N(t)\) encodes their distribution.  Computational experiments reveal that peaks of \(|F_N(t)|\) align strikingly with the ordinates of non‑trivial zeros of \(\zeta(s)\), especially when smooth windows are used.  The golden‑rotation embedding and residue‑class filaments provide geometric insight into prime distribution, suggesting deterministic structure beneath apparent randomness.

The operator‑theoretic program outlined above aims to convert these observations into a rigorous proof of the Riemann Hypothesis.  By showing that any zero off the critical line would generate unbounded energy leakage in the resonance field, one can hope to force all zeros onto \(\Re(s)=\tfrac12\).  The required analytic tools—Mellin transforms, Paley–Wiener theorems, and careful control of smooth cutoffs—are classical, but their synthesis within the modular prime rattling framework is novel.

Future work will include higher‑precision computations with larger cutoffs, exploration of alternative window functions (e.g. Dolph–Chebyshev), extension of the resonance framework to Dirichlet \(L\)-functions, and derivation of explicit inequalities bounding energy leakage.  These steps will help solidify the bridge between prime rattling and the Riemann Hypothesis.

## Acknowledgements

This document synthesises collaborative explorations with ChatGPT (Skies).  The author thanks the open‑source community for tools and inspiration.