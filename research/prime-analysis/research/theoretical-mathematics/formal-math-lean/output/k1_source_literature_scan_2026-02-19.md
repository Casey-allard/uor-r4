# K1 Source Literature Scan (2026-02-19)

## Target
Close the remaining K1-source theorem term:

- `PrimeRiemannBridgeOscillatoryReduction.ZeroToCosSinPhaseTransfer`

This is the proposition that a right-half zeta zero (under the Von-Koch endpoint criterion) yields a cos/sin explicit-formula decomposition with vanishing normalized remainder.

## Primary Sources Checked

1. Pintz (2017): *Distribution of zeta zeros and the oscillation of the error term of the prime number theorem*  
   DOI: [10.1134/S0081543817010163](https://doi.org/10.1134/S0081543817010163)  
   Index page: [journals.rcsi.science/0081-5438/article/view/174251](https://journals.rcsi.science/0081-5438/article/view/174251)

2. Pintz (1980): *On the remainder term of the prime number formula II; On a theorem of Ingham*  
   EuDML page: [eudml.org/doc/205693](https://eudml.org/doc/205693)

3. Révész (2023): *Oscillation of the Remainder Term in the Prime Number Theorem of Beurling, “Caused by a Given zeta-Zero”*  
   DOI: [10.1093/imrn/rnac274](https://doi.org/10.1093/imrn/rnac274)  
   Open repository page: [real.mtak.hu/191438](https://real.mtak.hu/191438/)

4. Loeffler–Stoll (2025): *Formalizing zeta and L-functions in Lean* (AFM)  
   PDF: [afm.episciences.org/15954/pdf](https://afm.episciences.org/15954/pdf)

5. Clay Mathematics Institute RH page (status check, unsolved frontier)  
   [claymath.org/millennium/riemann-hypothesis](https://www.claymath.org/millennium/riemann-hypothesis/)

## What these sources give us

- The analytic-number-theory literature supports the zero-driven oscillation program (Ingham/Pintz/Revesz line).
- The Lean AFM paper confirms that current Lean mathlib formalizes zeta/L-functions and an RH statement, but not an unconditional RH proof.
- No source provides a drop-in Lean theorem term for our exact `ZeroToCosSinPhaseTransfer` proposition.

## Inference to K1 (explicitly marked as inference)

From the explicit-formula and given-zero oscillation framework in the cited literature, one infers a decomposition route of the same analytic shape as K1 (oscillatory main term plus controlled remainder).  
**Inference note:** this is a mathematically standard extraction step, but not currently available as an already packaged Lean theorem term matching our exact proposition signature.

## Practical conclusion

- Unconditional RH closure is still not available from existing published formal proof artifacts.
- The remaining work is to either:
  1. formalize the K1-source theorem term directly in Lean from explicit-formula machinery, or
  2. import a trusted theorem term for the exact proposition and close conditionally on that imported term.
