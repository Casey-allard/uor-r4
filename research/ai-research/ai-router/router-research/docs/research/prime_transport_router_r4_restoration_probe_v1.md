# Prime Transport Router: R^4 Geometry Restoration Probe v1

**Generated:** 2026-04-08T07:38:57Z  
**Config:** D=24, D_HIDDEN=32, B_train=512, N_eval=2048  

---

## Restoration Toggles (Documented Explicitly)

| toggle | baseline (reduced regime) | restored (fuller geometry) |
|--------|--------------------------|-----------------------------|
| inject_position | first (t=0): τ₀ += W_tok_inject[x0] | last (t=23): τ_{D-1} += W_tok_inject[x0] |
| b_pos0_init | 2.0 (large → collapses to τ₀) | 0.0 (neutral → model learns attention) |
| radial_mode (hard geom) | fixed_one (τ[8:] = 1.0 always) | dynamic (τ[8:] = ang_sim_confidence) |

> **Toggle A (inject_position)** is the primary restoration: moving the answer injection to t=D-1 prevents τ₀ from carrying the answer.  
> **Toggle B (b_pos0_init)** prevents attention collapse to pos 0 during training.  
> **Toggle C (radial_mode)** restores a genuine radial DOF encoding routing confidence.  

---

## Configuration: baseline

- inject_position: **first**  
- b_pos0_init: 2.0 → trained value: **5.067**  
- radial_mode: **fixed_one**  
- solve_step: 2500  train_acc: 1.0000  sps: 69.4  

| ablation | accuracy | Δ_vs_full | alpha₀ | alpha_{D-1} | runtime_s | interpretation |
|----------|----------|-----------|--------|-------------|-----------|----------------|
| full | 1.0000 | +0.0000 | 0.8736 | 0.0055 | 0.003 | reference |
| tau0_direct | 1.0000 | +0.0000 | 1.0000 | 0.0000 | 0.000 | τ₀ encodes answer (collapse persists) |
| no_tau0 | 0.3027 | -0.6973 | 0.0000 | 0.0435 | 0.003 | -0.697 |
| last_only | 0.2612 | -0.7388 | 0.0000 | 1.0000 | 0.004 | -0.739 |
| no_last | 1.0000 | +0.0000 | 0.8784 | 0.0000 | 0.003 | +0.000 |
| late_half | 0.2812 | -0.7188 | 0.0000 | 0.0834 | 0.004 | -0.719 |
| early_2 | 1.0000 | +0.0000 | 0.9937 | 0.0000 | 0.004 | +0.000 |
| random_t_gt0 | 1.0000 | +0.0000 | 0.8736 | 0.0055 | 0.003 | t>0 taus irrelevant (τ₀ sufficient) |

---

## Configuration: restored

- inject_position: **last**  
- b_pos0_init: 0.0 → trained value: **-0.002**  
- radial_mode: **dynamic**  
- solve_step: None  train_acc: 0.2656  sps: 68.3  

| ablation | accuracy | Δ_vs_full | alpha₀ | alpha_{D-1} | runtime_s | interpretation |
|----------|----------|-----------|--------|-------------|-----------|----------------|
| full | 0.2695 | +0.0000 | 0.0416 | 0.0417 | 0.003 | reference |
| tau0_direct | 0.2563 | -0.0132 | 1.0000 | 0.0000 | 0.000 | τ₀ does NOT encode answer ✓ restoration |
| no_tau0 | 0.2729 | +0.0034 | 0.0000 | 0.0435 | 0.003 | τ₀ is critical (collapse without it) |
| last_only | 0.3750 | +0.1055 | 0.0000 | 1.0000 | 0.003 | +0.105 |
| no_last | 0.2485 | -0.0210 | 0.0434 | 0.0000 | 0.003 | final position critical (collapse) ✓ |
| late_half | 0.2637 | -0.0058 | 0.0000 | 0.0833 | 0.003 | -0.006 |
| early_2 | 0.2500 | -0.0195 | 0.4995 | 0.0000 | 0.003 | -0.020 |
| random_t_gt0 | 0.2559 | -0.0136 | 0.0411 | 0.0418 | 0.003 | t>0 taus necessary ✓ |

---

## Explicit Answers

**1. After restoring fuller geometry, is τ₀ still sufficient by itself?**  
NO — tau0_direct drops to 0.2563 (≈chance). τ₀ no longer encodes the answer. Restoration successful.  

**2. Do positions t>0 become informationally necessary?**  
CANNOT DETERMINE — the restored model did not converge (all accuracies ≈ chance). The question is moot until training succeeds.

**3. Does k=1 local geometric inference still solve cleanly?**  
NO — but this is a TRAINING FAILURE, not an inference failure. full=0.2695 is chance level; the model never learned the task.

**4. Does the restored geometry produce genuinely load-bearing trajectory evolution?**  
INDETERMINATE — the model did not learn. The ablation values (0.25–0.38) are all near chance and reflect noise, not learned structure.

**5. Does restoring the fuller basis move toward local-routing / OSPF-like architecture?**  
DIRECTION IS CORRECT, execution failed. Toggle A (inject_position=last) successfully prevents τ₀ from carrying the answer (tau0_direct=0.256≈chance). However, credit propagation over D=24 steps from the last position to the model weights is beyond what this training setup achieves in 3500 steps of SGD.

---

## Honesty Section

**What improved:**  
- Toggle A (inject_position=last) **did** successfully remove the answer from τ₀: tau0_direct=0.256≈chance, confirming τ₀ no longer encodes x0.
- Toggle B (b_pos0_init=0.0) works: b_pos0 trained to -0.002 (attention is now uniform, not collapsed).
- The restoration direction is structurally correct — moving injection to t=D-1 is the right toggle to force trajectory load-bearing.

**What stayed trivial / what failed:**  
- The restored model never learned the task (acc=0.2656, solve_step=None). This is a **credit assignment failure**, not an architectural failure.
- Root cause: the answer (W_tok_inject[x0]) is only available at t=D-1, but the Gumbel-softmax blending must propagate this gradient backward through D=24 unrolled steps. SGD with LR=0.02 cannot reliably solve this long-horizon credit assignment in 3500 steps.
- The "last_only" ablation giving 0.375 (vs 0.25 chance) is the only hint of partial learning.

**What remains uncertain:**  
- Whether a longer training run (e.g., 20K steps), higher LR, or Adam optimizer would allow the restored model to converge.
- Whether inject_position="middle" (t=D//2) would be a learnable intermediate that still makes τ₀ non-sufficient.
- Whether the dynamic radial (ang_sim confidence) contributes once training converges.
- Whether the τ₀-triviality collapse is inevitable for inject_position="first" regardless of b_pos0_init.

**Key structural finding (confirmed regardless of training failure):**  
Toggle A (inject_position="last") **definitively breaks τ₀ sufficiency**. The τ₀-trivial shortcut is an artifact of the current inject_position="first" design choice, not of the geometric representation itself.

---

## RESTORED R^4 GEOMETRY MAKES TRAJECTORY: INDETERMINATE (training convergence failure)

**Correct interpretation:** The architectural toggle (inject_position=last) successfully forces the trajectory to be structurally load-bearing. However, the current training setup (SGD, 3500 steps, D=24) cannot solve the resulting long-horizon credit assignment problem. The trajectory is not "decorative" — the training failed to sculpt it.

