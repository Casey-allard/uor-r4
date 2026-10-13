# Prime Transport Router: Fuller-Geometry Fair Retest Under Trainable Delayed Regime v1

**Generated:** 2026-04-08T08:25:03Z  
**Config:** D=24, D_HIDDEN=32, B_train=512, N_eval=2048, steps=3500, LR=0.02, b_posLast_init=2.0  

## Definition of Restored Fuller Geometry

Both regimes use the same training setup (inject@last, b_posLast=2.0, dynamic radial).  
Only the angular/radial representation changes.  

| Component | Reduced Baseline | Restored Fuller |
|-----------|------------------|-----------------|
| PHASE_BLOCKS | [(0,2,2,1),(2,7,5,1),(7,9,2,1),(9,21,12,1)] | [(0,2,2,1),(2,7,5,**2**),(7,9,2,1),(9,21,12,**3**)] |
| Block 0 (mod=2) | 1 harmonic → 2 dims | 1 harmonic → 2 dims |
| Block 1 (mod=5) | 1 harmonic → 2 dims | **2 harmonics → 4 dims** (k=1,2; gcd(2,5)=1: fully separating) |
| Block 2 (mod=2) | 1 harmonic → 2 dims | 1 harmonic → 2 dims |
| Block 3 (mod=12) | 1 harmonic → 2 dims | **3 harmonics → 6 dims** (k=1,2,3: subgroup structure) |
| d_tau_ang | **8** | **14** |
| d_tau_hyb | **12** | **18** |
| d_in_hyb | **16** | **22** |

**Rationale:** The mod-5 and mod-12 groups have non-trivial subgroup structure that a single Fourier harmonic cannot capture. Adding k=2 for mod-5 fully distinguishes all 5 states; k=1,2,3 for mod-12 captures its cyclic substructures (Z₆, Z₄, Z₃, Z₂).  

---

## Training Results

| regime | geometry | d_tau_hyb | accuracy | solve_step | α₀ | α_{D-1} | b_posLast | runtime_s |
|--------|----------|-----------|----------|------------|----|---------|-----------|-----------|
| delayed_baseline | reduced | 12 | 1.0000 | 2500 | 0.0049 | 0.8739 | 5.0626 | 58.5 |
| delayed_fuller | fuller | 18 | 0.9946 | — | 0.0054 | 0.8603 | 4.9476 | 73.5 |

---

## Ablation Results

| regime | ablation | accuracy | Δ_vs_full | α₀ | α_{D-1} | interpretation |
|--------|----------|----------|-----------|----|---------|----------------|
| delayed_baseline | full | 1.0000 | +0.0000 | 0.0049 | 0.8738 | reference |
| delayed_baseline | tau0_direct | 0.2588 | -0.7412 | 1.0000 | 0.0000 | τ₀ does NOT encode answer ✓ |
| delayed_baseline | no_tau0 | 1.0000 | +0.0000 | 0.0000 | 0.8781 | trajectory sufficient without τ₀ ✓ |
| delayed_baseline | last_only | 1.0000 | +0.0000 | 0.0000 | 1.0000 | final position carries full answer ✓ |
| delayed_baseline | no_last | 0.2578 | -0.7422 | 0.0386 | 0.0000 | t=D-1 critical (collapse without it) ✓ |
| delayed_fuller | full | 0.9990 | +0.0000 | 0.0054 | 0.8604 | reference |
| delayed_fuller | tau0_direct | 0.2520 | -0.7470 | 1.0000 | 0.0000 | τ₀ does NOT encode answer ✓ |
| delayed_fuller | no_tau0 | 0.9990 | +0.0000 | 0.0000 | 0.8651 | trajectory sufficient without τ₀ ✓ |
| delayed_fuller | last_only | 1.0000 | +0.0010 | 0.0000 | 1.0000 | final position carries full answer ✓ |
| delayed_fuller | no_last | 0.2568 | -0.7422 | 0.0388 | 0.0000 | t=D-1 critical (collapse without it) ✓ |

---

## Explicit Answers

**1. Is trajectory load-bearing in delayed_baseline?**  
YES — no_tau0=1.0000 (sufficient without τ₀); no_last=0.2578 (collapse without t=D-1).  

**2. Is trajectory load-bearing in delayed_fuller?**  
YES — no_tau0=0.9990 (sufficient without τ₀); no_last=0.2568 (collapse without t=D-1).  

**3. Is fuller geometry beneficial under the fair delayed test?**  
Baseline solves (acc=1.0000); fuller geometry only reaches 0.9946. Richer representation does not help under this budget.  

**4. Is τ₀ still non-sufficient in restored geometry?**  
YES — tau0_direct=0.2520≈chance.  

**5. Is previous dismissal of fuller geometry overturned?**  
CONFIRMED — even under fair trainable conditions, fuller geometry does not improve results.  

**honest_tested**  
Both regimes use identical inject@last + b_posLast=2.0 setup. The sole variable is the angular/radial representation (single vs multi-harmonic). This is the first fair test of richer geometry under trainable delayed injection.  

**honest_unresolved**  
Whether even richer geometry (e.g., more harmonics, higher D_HIDDEN) would change the result at larger D remains untested.  

**honest_premature**  
NO — the previous probe was indeterminate for the right reason (bootstrapping failure), and this fair retest confirms fuller geometry is not beneficial here.  

---

## Honesty Section

**What is now fairly tested:**  
Both regimes use identical inject@last + b_posLast=2.0 setup. The sole variable is the angular/radial representation (single vs multi-harmonic). This is the first fair test of richer geometry under trainable delayed injection.  

**What remains unresolved:**  
Whether even richer geometry (e.g., more harmonics, higher D_HIDDEN) would change the result at larger D remains untested.  

**Whether previous dismissal was premature:**  
NO — the previous probe was indeterminate for the right reason (bootstrapping failure), and this fair retest confirms fuller geometry is not beneficial here.  

---

## FULLER GEOMETRY UNDER FAIR DELAYED TEST IS: IRRELEVANT

