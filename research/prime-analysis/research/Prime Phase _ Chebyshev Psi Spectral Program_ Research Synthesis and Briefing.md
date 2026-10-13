### Prime Phase / Chebyshev Psi Spectral Program: Research Synthesis and Briefing

#### Executive Summary

The Prime Phase / Chebyshev Psi Spectral Program is an empirical investigation into whether arithmetic observables derived from prime numbers—specifically the Chebyshev residual  $f(x) \= \\psi(x) \- x$ —possess a recoverable spectral or phase structure linked to Riemann zeta-zero frequencies. The program aims to determine if these phase variables can predict forward prime-density residuals out-of-sample.Critical takeaways include:

* **Spectral Extraction:**  A stable, low-frequency oscillatory mode exists in the Chebyshev residual when analyzed on a uniform log-scale grid. A dominant peak near 0.008 cycles/sample is consistently observed.  
* **Zeta Alignment Limits:**  While spectral peaks can be robustly extracted, current evidence suggests these peaks are not uniquely aligned with true zeta zeros beyond what is expected from random distribution density (p ≈ 0.62). However, they do outperform phase-randomized surrogates.  
* **Predictive Power:**  Hilbert-phase variables provide a statistically meaningful lift in predicting future prime-density residuals. This effect is most pronounced at short horizons ( $h=25$  to  $h=50$ ) and in "low-residual" regimes where autoregressive (AR) baselines are less dominant.  
* **Methodological Rigor:**  To combat the volatility of stateless compute environments, the project has implemented a "fingerprint-locked" canonical pipeline. This ensures reproducibility by requiring SHA1 hash validation of all intermediate data structures.

#### Theoretical Framework and Goals

The project operates under the core hypothesis that the distribution of primes corresponds to the extrema or minima of a smooth arithmetic potential,  $U(n)$ . This potential is modeled as a phase-modulated Cox process or a latent-state intensity model.

##### Primary Objectives

1. **Empirical Testing:**  Probe whether observables derived from  $\\psi(x)-x$  on a log scale contain recoverable spectral structure related to zeta-zero frequencies.  
2. **Forward Prediction:**  Determine if phase variables can predict forward prime-density residual activity in multiplicative windows out-of-sample.  
3. **Structural Modeling:**  Derive a smooth field or potential on integer-space where primes are enriched near specific boundaries or minima.

#### The Canonical Pipeline and Methodological Standards

To ensure a "leakage-free" workflow, the project follows a strict protocol for data processing and model evaluation. All analyses are performed on a uniform log-space grid ( $t \= \\log x$ ).

##### The Standard Preprocessing Sequence

Step,Action,Specifications  
1,Data Loading,Load Chebyshev  $\\psi$  including prime powers up to  $N=10^7$ .  
2,Grid Interpolation,Interpolate  $\\psi(x)-x$  onto a uniform log grid (typically  $L=131072$ ).  
3,Domain Restriction,"Restrict analysis to  $x$  in range  $100,000, 9,000,000$ ."  
4,Detrending,"Apply causal, past-only moving average ( $W=2000$ )."  
5,Standardization,Normalize using  TRAIN-only  mean and standard deviation.  
6,Bandpass Filter,"Apply rFFT/irFFT mask (typically  $0.002, 0.01$  cycles/sample)."  
7,Phase Extraction,Compute Hilbert analytic signal to define  $\\theta(t) \= \\text{angle}(\\text{analytic}) \\mod 2\\pi$ .

##### Leakage and Validation Rules

* **Train/Test Split:**  Data is split chronologically (70% Train / 30% Test). Shuffling is strictly forbidden.  
* **Fingerprint Locking:**  Cryptographic SHA1 hashes must match for x\_kept, u\_bp, theta\_mod, and R (residual) before results are accepted.  
* **Baselines:**  All phase-based models must be compared against a Persistence predictor, AR(1) on  $R$ , and a Logistic AR model using lagged  $R$ .

#### Key Research Findings

##### 1\. Spectral Stability and Zeta Alignment

The Chebyshev residual in log-space exhibits a coherent oscillatory component. However, the "zeta-uniqueness" of this signal remains unproven:

* Real peak-gammas are  **not better than uniform random gammas**  in the same range ( $p \\approx 0.62$ ).  
* Real peaks  **do beat phase-randomized surrogates**  ( $p \\approx 0.039$ ), suggesting that peak extraction depends on time-domain structure beyond the power spectrum alone.

##### 2\. Predictive Performance of Phase Variables

Hilbert phase  $\\theta$  extracts predictive signal for future prime activity, achieving out-of-sample AUCs between 0.61 and 0.76.

* **Non-Markovian Dynamics:**  While  $\\theta$  is predictive, 1D and 2D phase-space models fail to forward-predict  $\\theta$  itself, implying the existence of a hidden driver or higher-dimensional state.  
* **Regime-Dependent Lift:**  Phase-based models show varying degrees of "lift" over AR baselines depending on the state of the residual ( $R$ ):  
* **Low**  **$|R|**$  **Stratum:**  Phase features provide significant improvement ( $\\Delta AUC \\approx 0.055$ ).  
* **High**  **$|R|**$  **Stratum:**  AR baselines are already highly accurate ( $AUC \\approx 0.908$ ), leaving less room for phase-based lift ( $\\Delta AUC \\approx 0.011$ ).

##### 3\. Transport and Mean Reversion

Research into "phase transport"—the wrapped difference in phase over a horizon  $h$ —reveals that it is a weak secondary signal:

* Transport provides a small lift at short horizons ( $h=25$ ) but becomes harmful to model accuracy at longer horizons ( $h=100$ ).  
* No evidence currently supports the existence of "privileged constants" (such as  $\\tau \= 2\\pi$  or  $125$ ) in the rotation rate.

#### Current Project Frontier and Next Steps

The project has moved away from "constant-hunting" and toward structural modeling of the prime residual.

##### Established Falsifications

* **Wheel/Primorial Analysis:**  Sufficient for explaining sieve logic, but not causal for spectral phase dynamics.  
* **Phase-Only Models:**  Stationary phase coordinates are insufficient; transport and interaction with arithmetic "pressure" are required.  
* **Zero Superiority:**  Earlier claims of "Zero-based" prediction superiority were eliminated under proper Maximum Likelihood Estimation (MLE); Real and Zero data converge under sufficient  $\\Gamma$  (\~300).

##### Ranked Priorities for Future Research

1. **De-sticking Targets:**  Design targets that are not trivially persistent, such as predicting the sign of the residual change ( $\\text{sign}(R\_{i+h} \- R\_i)$ ) or rare event (top decile) exceedance.  
2. **Spectral-Statistics (Hilbert–Pólya):**  Test for a genuine zeta operator signature by comparing spacing statistics to Gaussian Unitary Ensemble (GUE) distributions rather than raw peak matching.  
3. **Multi-band State Models:**  Transition to two-band or multi-band phase state models to recover latent drivers that single-phase dynamics cannot capture.  
4. **Magnitude Prediction:**  Explore predicting the magnitude  $|dR|$  rather than just the binary sign of change.

