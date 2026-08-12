### Implementation Specification: Standardized Prime-Residual Spectral Research Pipeline

#### 1\. Strategic Intent and Research Foundation

The strategic mandate of this specification is to stabilize the analysis of the Chebyshev residual,  $f(x) \= \\psi(x) \- x$ , to isolate latent harmonic structures within prime distributions. While traditional probabilistic models such as the Prime Number Theorem (PNT) describe average densities, they are inherently stationary and fail to capture the scale-dependent dynamics of prime emergence. This implementation transforms exploratory "toroidal story-first" geometric narratives into a rigorous, potential-based arithmetic model. Our "North Star" objective is  **Transport-Corrected Alignment** : acknowledging that naive stationary models fail, we enforce a model stable across scales through transport and renormalization. Log-scale phase coordinates are the required substrate for this work, as they allow for the identification of non-Markovian hidden drivers that remain invisible to Markovian probabilistic approaches. This specification enforces the transition from exploratory observation to a high-fidelity potential field analysis constrained by the physical limits of the data substrate.

#### 2\. Computational Substrate and Grid Architecture

To ensure signal processing stability and mitigate parameter drift across stateless, heterogeneous worker environments, this pipeline operates on a high-resolution, uniform log-grid. Maintaining a "locked" grid architecture is non-negotiable for preventing cumulative rounding errors and ensuring that spectral resolution is bit-perfect across distributed compute nodes.

##### Authoritative Grid Parameters

Parameter,Value,Description  
Grid Length (  $L$  ),"131,072",Optimal power-of-two length for FFT stability; provides spectral resolution required to eliminate aliasing in low-frequency bands.  
Step Size (  $dt$  ),0.00012297217348543395,Uniform step in  $t \= \\log(x)$  space; authoritative for all  $\\gamma \= 2\\pi f / dt$  mappings.  
Random Seed,12345,Mandatory seed for all stochastic operations and initialization.  
Domain Restriction,"$100,000.0, 9,000,000.0$",Physical  $x$ \-space bounds for authoritative signal extraction.  
Train/Test Split,70/30,Strict time-ordered split; no shuffling permitted to preserve causal integrity.

#### 3\. Canonical Signal Processing Pipeline

Consistent  $\\theta\_{mod}$  extraction across nodes requires an immutable processing pipeline. Any deviation in implementation—particularly regarding Hilbert conventions or FFT masking—introduces "SUSPECT" artifacts that contaminate the research history.

##### Exact Implementation Steps

1. **Interpolation:**  Map the raw  $\\psi(x)-x$  residual (from chebyshev\_psi.npz) onto the uniform  $t$ \-grid of length  $L$  over the authoritative domain.  
2. **Causal Detrending:**  Implement a  **past-only moving average**  with window  $W=2000$ . The detrended signal  $u$  is defined by:  $ui \= u\_{raw}i \- \\text{mean}(u\_{raw}i-W+1 : i)$ .  
3. **Standardization:**  Calculate mean ( $\\mu$ ) and standard deviation ( $\\sigma$ ) using  **TRAIN-set data only** . Apply these constants to normalize the full signal.  
4. **Spectral Masking:**  Isolate the  $0.002, 0.01$  cycles/sample band using  **rFFT/irFFT with a non-negative frequency mask** . Standard full-FFT masking is strictly forbidden to prevent phase-velocity distortion.  
5. **Phase Extraction:**  Generate the analytic signal using the  **Scipy implementation**  of the Hilbert transform (scipy.signal.hilbert). Define  $\\theta\_{mod} \= \\text{angle}(z) \\pmod{2\\pi}$ .  
6. **Residual Construction:**  Define the prime-window residual  $R\_i \= (P\_i \- E\_i) / \\sqrt{E\_i}$ , where  $P\_i$  is the actual prime count (from primes.npz) in  $\[x\_i, x\_i \\cdot \\exp(0.01))$  and  $E\_i$  is the expected count.This pipeline generates verifiable cryptographic fingerprints, serving as a concurrency safeguard to validate that all nodes are operating on the same arithmetic manifold.

#### 4\. Cryptographic Verification and Fingerprint Locking

In stateless worker environments, SHA1 hashing is the primary mechanism for maintaining research integrity. Every compute task must bifurcate its workflow to include a mandatory validation step.

##### Canonical Fingerprints (SHA1)

* **$x\_{kept}**$ : c455fedeff29bb2d0b2c81864970c09638720baf  
* **$u\_{bp}**$ : b16b32cb5d4e63be1c61b88fcf4711d0656da14e  
* **$\\theta\_{mod}**$ : 8b33c57df436d37179b13b1899bf86a723f0e98a  
* **$R**$ : 5753ee7d59c7d155c4a1b9b5fba8912d557598f2**Enforcement Rule:**  Workers must load the authoritative cache from /mnt/data/A4\_CANONICAL\_CACHE\_L131072\_x1e5\_9e6\_W2000\_bp002\_001\_D001\_seed12345.npz. If any local SHA1 hash deviates from the canonical reference, the worker must  **Stop and Fail immediately** . No downstream metrics (AUC, Brier, or accuracy) are to be calculated or reported if the  $R$  hash fails, as this prevents metric contamination.

#### 5\. Target Geometry and Experimental Protocols

To overcome "label stickiness"—the persistence of residuals in short horizons—all experiments must use "De-stickied" targets to isolate genuine spectral influence from simple autocorrelation.

##### Target Definitions

* **Primary Classification Target:**   $y\_i \= \\text{sign}(R\_{i+h} \- R\_i)$  
* **Rare-Event Target:**   $y\_i \= 1$  if  $R\_{i+h}$  is in the top-decile of the TRAIN distribution.

##### Mechanism Probe Insights

Architectural analysis reveals that phase features provide the most significant lift ( $\\Delta AUC \\approx 0.055$ ) specifically in  **Low-Regime (**  **$|R\_i|**$  **) states** . In these regimes, the system is not dominated by its own mean-reverting autocorrelation, allowing phase to act as the primary "hidden driver" of prime density. Conversely, in High-Regime states, AR structures dominate, leaving minimal marginal lift for phase predictors.\!WARNING  **MANDATORY SECTION 3 LEAKAGE RULES:**

* Forbid "centered" or forward-looking moving averages; only causal detrending is permitted.  
* All standardization constants and thresholds must be TRAIN-derived.  
* Any tuning of bandpass ranges or detrending windows based on TEST-set performance is a violation of protocol.

#### 6\. Baselines and Performance Benchmarks

Any spectral signal must overcome a rigorous "Burden of Proof." It is insufficient to outperform random chance; spectral models must provide marginal lift over high-order Autoregressive (AR) structures.

##### Benchmarks and Lift

* **AR(40) Baseline:**   $AUC \\approx 0.760$ .  
* **Full Phase Model:**  Incorporating transport  $\\Delta\\theta\_h$  yields a marginal lift of  $\\approx 0.00118$  at  $h=25$ .

##### Falsification Log

The following hypotheses are ruled as  **Pipeline Artifacts**  or non-significant:

* **$\\tau**$  **and 125-sample constants:**  No evidence supports  $2\\pi$  or 125-sample intervals as privileged structural constants; they are classified as artifacts of previous pipeline drifts.  
* **Statistical Significance:**  All p-value calculations must utilize  **autocorrelation-preserving circular-shift nulls**  rather than IID shuffles to account for natural signal persistence.Phase information remains marginal relative to AR structure at the current scale, necessitating the move toward multi-band phase state modeling to determine if interaction between frequency bands can fully recover the forcing state of prime density residuals.

