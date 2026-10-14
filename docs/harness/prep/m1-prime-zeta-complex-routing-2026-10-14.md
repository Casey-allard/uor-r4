# Pre-registration - M1 (prime-zeta-complex-routing)

- date: 2026-10-14
- milestone: M1 #2029 - Language base: a geometric stack language model good enough to carry chat
- mechanism: prime-zeta-complex-routing
- mechanism brief: docs/mechanisms/prime-zeta-complex-routing.md
- run: Finite qualification of prime-rank indexing and complex-plane zeta-phase quantization boundary
- line: exact addressed memory - count 0/3 (D21/D22 count is owned by the milestone, not this sheet)

## Bar (pre-registered)
- bar (finite construction qualification):
  1. Bit-exact integer codebook (`Snapshot.table`) reproduction for `V4096` (`B >= 2` blocks) across named deterministic seeds (`seed = 0, 1, 2`) matching analytic witnesses;
  2. Zero floating-point instructions in the AOT-compiled CUDA PTX module verified via `admit_ptx` (`crates/uor-r4-discrete/build.rs:25-83`);
  3. Strict non-aliasing across `V4096` multi-block allocation (`0` full-vector collisions across all 4,096 vocabulary tokens);
  4. `17/17 PASS` (100%) on the 17 discrete CUDA construction controls (`11` Route `qualify` + `6` Flock `qualify-flock`);
  5. **D22 §3 matched attribution nulls**:
     - **Construction-time structural separation (scored in this qualification)**: The prime-zeta codebook ($r(v) = p_{\text{id}(v)+1}$, $\theta_k(p) = \gamma_k \ln p \pmod{2\pi}$) must exhibit non-zero multi-block Hamming separation and distinct cross-token phase-interference structure relative to both the **tangent null** (critical-line phase shift $\gamma_k \to \gamma_k + \delta$ with fixed offset $\delta = 1.0$) and the **normal null** (seeded prime-rank permutation $\pi(p)$ and off-critical-line amplitude perturbation).
     - **Downstream learned/corpus attribution (explicitly deferred to the first downstream training sheet)**: Evaluating whether prime-zeta routing outperforms the normal null and separates from the tangent null on learned discrete move selection / corpus loss requires corpus updates and is explicitly deferred to the first downstream learned training sheet once the global training hold is lifted. This ordering is safe because the present sheet is a zero-training finite construction qualification (`0` corpus updates) that strictly gates codebook non-aliasing and zero-float PTX compliance before any learned run.
- panel (milestone headline context only - NOT scored in this finite construction qualification; this qualification is scored strictly against the finite construction bar above): `v5-reply` (grader: BLOCKED (D28) - chat-grade by qwen2.5:7b is a NEURAL grader and D28 prohibits neural graders until the owner declares the native model works end to end. D28 names this grader specifically: "This includes the v5 reply panel's grade-replies qwen2.5:7b grader." The panel is scored with DETERMINISTIC checks only (membership checks, exact-answer oracles); a bar that can only be scored by a neural grader is recorded BLOCKED and is NOT RUN., frozen: yes)
- panel noise (milestone headline context): unstated for the reply panel; on the 40-row memory panel +/-3 rows is the noise seen (docs/labs/handoff-deepseek-2026-10-12.md: 'Bars are never narrower than the panel's noise (+/-3 rows on the 40-row memory panel)'). A bar must not be narrower than its panel's noise.
  - reply-acceptable (milestone headline context, not this qualification's bar): acceptable replies on the open reply panel 37/232 -> 116/232 acceptable replies [open]
  - v5-memory-check-pass (milestone headline context, not this qualification's bar): v5 multi-turn memory panel check_pass 20/40 -> 34/40 check_pass rows [open]
- no bar in this sheet may be narrower than the panel noise above (D22 section 3).

## Base artifact
- base: Refreshes from historical discrete shared-writer baseline `a2a5b89c` (and discrete-admission source `35a48f5863ed07711f70602ffa30d8d86209bf27`) to execution-time `origin/main` (`c109b2109d1fdfd80a8cb5bd94d382033f2f8f38` at registration; `crates/uor-r4-discrete/` and `crates/uor-r4-core/` have an empty diff against `origin/main`, and the only `crates/` diff in PR #2433 is the `F9` deterministic grader boundary fix in `crates/uor-r4-training/src/bin/chat-grade.rs` and its three `reply-panel-*-v2.tsv` fixtures).
- float scope (D32 October 14 addendum #2398): ALL pre-quantization floating point (`ln`, `sin`, `cos` on `f64` `ZETA_ZEROS` ordinates along $\mathbb{C}^4 \to \mathbb{R}^4$ and $\mathbb{C}^8 \to \mathbb{R}^8$) is host-side, one-time, offline codebook generation — never in the compiled CUDA module.

## Data split
- split: fixture-tokens V4096 B2 context64 (disposable literal/analytic construction fixtures only; 0 corpus training or evaluation rows)
- training data never includes the frozen panel rows; scoring uses the frozen panel and, for a KEEP, a fresh held-out draw (D22 section 4)

## Steps
- steps:
  1. Map token IDs $\text{id}(v) \in \{0, \dots, V-1\}$ bijectively to prime atoms $r(v) = p_{\text{id}(v)+1}$;
  2. Perform one-time, offline, host-side computation of complex log-prime phasors $\exp(i\gamma_k \ln p)$ for $k=1 \dots 16$ and realify via $\mathbb{C}^4 \to \mathbb{R}^4$ and $\mathbb{C}^8 \to \mathbb{R}^8$;
  3. Quantize immediately at the host integer boundary into `PhaseQ29`, `UnitS3Q30`, $E_8 = H_4 \oplus \varphi H_4$ anchor roots (`ROOTS: [[i8; 8]; 16]`), and the integer codebook `Snapshot.table` across named seeds (`seed = 0, 1, 2`);
  4. Construct the matched **tangent null** ($\gamma_k \to \gamma_k + \delta$, $\delta = 1.0$) and **normal null** (seeded prime-rank permutation $\pi(p)$ / off-critical-line phases) codebooks on host, and verify strict `V4096` multi-block non-aliasing (`0` full-vector collisions) plus non-zero structural Hamming separation against both nulls (deferring learned move-selection / corpus attribution against both nulls to the first downstream training sheet after hold release);
  5. Verify zero floating-point instructions in the AOT-compiled CUDA PTX module via `admit_ptx`;
  6. Execute the 17 discrete CUDA construction controls (`11` Route `qualify` + `6` Flock `qualify-flock`) and require `17/17 PASS`.

## Minimum test power (D22 section 3)
- qualification power (finite construction qualification — no training, no corpus dose):
  1. **Bit-exact codebook across named seeds**: 100% bit-exact integer codebook (`Snapshot.table`) reproduction across named deterministic seeds (`seed = 0, 1, 2`) matching analytic witnesses;
  2. **Zero-float PTX via `admit_ptx`**: 0 floating-point instructions in the compiled CUDA PTX module verified by `crates/uor-r4-discrete/build.rs` (`admit_ptx`);
  3. **Strict non-aliasing across V4096 multi-block allocation**: All `V = 4096` vocabulary tokens map to distinct multi-block (`B >= 2`) root vectors (`0` full-vector collisions) and separate structurally from the tangent null ($\gamma_k \to \gamma_k + \delta$) and normal null (permuted primes / off-critical-line);
  4. **17/17 discrete construction controls**: `17/17 PASS` (`11` Route `qualify` + `6` Flock `qualify-flock`) on the discrete shared writer.
- seeds (D22 training power): `NOT_APPLICABLE` — finite construction qualification with zero training steps; prime-zeta indexing is deterministic from prime rank $p_{\text{id}(v)+1}$ and fixed Odlyzko ordinates $\gamma_k$, and named seeds (`0, 1, 2`) are used only to verify bit-exact reproducibility of seeded control/null fixtures.
- fresh rows (D22 training power): `NOT_APPLICABLE` — finite construction qualification on literal/analytic construction fixtures only; no corpus evaluation or open/held-out reply panel draw is executed in this run.
- trained in rather than bolted on (D22 training power): `NOT_APPLICABLE` — zero training or corpus updates are executed (both the global training hold and M2 hold remain HELD); training the prime-zeta codebook in from initialization across $\ge 2$ seeds / $\ge 200$ fresh rows is deferred to the first downstream training sheet after hold release.
- fresh-transfer plan for a KEEP (D22 section 4): `NOT_APPLICABLE` — finite construction qualification only; does not score or move a learned panel headline.

## Cost (charged cumulatively, before execution)
- preparation: 0.1s CPU (one-time host-side offline codebook quantization and tangent/normal null separation check)
- run: <15s finite CUDA qualification (gated behind TEST FITNESS: FIT and owner hold release; 0 GPU hours executed in this PR)
- controls: 17 discrete CUDA construction controls PASS + host-side tangent/normal null structural separation check (downstream learned tangent/normal null comparison deferred to first training sheet)
- evaluation: bit-exact integer checks, `V4096` non-aliasing verification, and `admit_ptx` zero-float verification
- retries: 0 (diagnose, correct and retry within the remaining budget; a self-set budget is not a stop rule)
- storage: 0 MiB (temporary, retained, and the 128 MiB storage stop margin)
- placement (D24): no model compute is planned; CPU-bound preparation, grading and checks run on the owner's laptop, and no GPU is requested for work that cannot change a decision
- pod/lease plan: NONE in this PR (training and M2 rental holds remain HELD; after TEST FITNESS: FIT and owner authorization, <= 1h single admitted CUDA 12.8+ pod via scripts/pod/uor-pod with release-then-down verified)
- owner caps: <= 4 running pods and <= $8.00/h across all labs, all sessions; no new spending class without an owner decision (AGENTS.md, docs/labs/compute.md section Caps)

## Stop rule
- stop: ptx admission failure, quantization overflow, V4096 multi-block aliasing collision, or CUDA context fault
- self-set step/wall budgets are estimates, not stop rules: extend and renew rather than cancel a healthy run; the only hard stops are the owner caps, the disk floor and a failed or diverged run

## Owner rules that apply
- D21: Three consecutive merged PRs on one line that do not move the milestone headline force a pivot card.
- D22: A negative closes a configuration, never a mechanism. Decisive runs use >=2 seeds or >=200 fresh rows, train the mechanism in, and carry a TEST FITNESS: FIT review on the milestone before compute.
- D23: No softmax anywhere in the active project: training, serving, sampling, objectives, auxiliary losses, teachers, comparators. No table/renamed equivalent. Interlock: `uor-harness ratchet d23|d25` (Rust, #2346) - `scripts/check_no_softmax.py` is retired and kept only as dated evidence; a pre-registration sheet that names the Python script sends a lab to a file that no longer enforces anything.
- D24: No CPU model compute; no GPU on work that cannot change a decision. A host-side fallback is a defect, not a cost to report.
- D25: Current geometric operator phase: OP2 first -> OP1 G2 only -> OP3a -> lawful rung0 after hold release -> OP4/rung0b; OP3b later research. Training and independent M2 rental hold remain. FIT lifts no hold; preserve peer work and frozen constraint set.
- D28: No transformer or neural-network comparisons of ANY kind - comparators, controls, baselines, teachers, neural graders - until the owner declares the native model works end to end; that declaration is the owner's, not a lab's. Panels are scored with DETERMINISTIC checks only (membership checks, exact-answer oracles); a bar scorable only by a neural grader is recorded BLOCKED and is NOT RUN. This includes the v5 reply panel's grade-replies qwen2.5:7b grader.
- D29: No banned mechanism may exist in any compiled or loaded module - presence counts, not only execution. NVRTC (runtime compilation) is banned project-wide; all CUDA kernels compile ahead of time via nvcc into named PTX/cubin modules. Interlock: `uor-harness ratchet d25` and the d29_compiled_modules test.
- D31: No result is ruled out on an instrument technicality. A check must answer the four authority questions in pm-rules.md R12 before it has veto power; when a check fails anyway, the instrument artefact is diagnosed and repaired rather than reported as a model negative. Supersedes older text that allows a first-failure stop or treats an instrument artefact as a code blocker.
- D32: Discrete geometric substrate: model computation AND its instruments, including learning, are discrete/exact and GPU-only - no floating-point forward, gradients, optimizer or numerical model instrument, and no matrix multiplication. Use native SpiralCore/E8/FBS geometry with one integer input/carry/output writer. Supersedes D25's offline scalar floating-point allowance, which is now inactive. Both training holds remain HELD; a FIT review releases neither.
- D22: a negative closes this configuration only, never the mechanism; stopping a mechanism needs the owner.

## Panel target (milestone registry)
- headline: M1: reply acceptable 37/232 -> target 116/232 (milestone headline context only; not scored by this finite construction qualification)
- panel: v5-reply
- source: docs/harness/milestones.json (issue #2029)

## D32 Addendum Item 3 Declaration
- **Complex Exception Map**: $\mathbb{C}^4 \to \mathbb{R}^4$ and $\mathbb{C}^8 \to \mathbb{R}^8$.
- **Host-Only Offline Scope**: ALL pre-quantization floating point is host-side, one-time, offline codebook generation — never in the compiled CUDA module.
- **Inputs**: Token ID $\text{id}(v) \in [0, V-1]$, prime atom $p = r(v) = p_{\text{id}(v)+1}$, Riemann zeta-zero ordinates $\gamma_k$ (`ZETA_ZEROS[0..15]`).
- **Complex Phase Factors**: $\theta_k(p) = \gamma_k \ln p \pmod{2\pi}$, $z_k(p) = e^{i\theta_k(p)} \in \mathbb{C}$.
- **Integer Boundary**:
  - `PhaseQ29`: $\text{round}(\theta_k \cdot 2^{29}) \in \mathbb{Z}$.
  - `UnitS3Q30`: $\text{round}(x_j \cdot 2^{30}) \in \mathbb{Z}^4$.
  - $E_8 = H_4 \oplus \varphi H_4$: lattice root snap into `ROOTS: [[i8; 8]; 16]`.
  - `Snapshot.table`: Integer codebook table.
- **Downstream Compute**: 100% discrete integer / exact CUDA module, 0 floating point forward/gradient/optimizer.

## Verdict
TEST FITNESS: FIT
- review: Conditional FIT by Kimi PM (`kimi/pm1`) on PR #2433 (`PRR_kwDOS1s4Zc8AAAABSLYz3g` / #2029 comment `6163842973`) at `47bf1d86a44e006d66e867961dbad16c0eea2d77`, with N1–N3 registration amendments incorporated above for one-pass confirmation. Training and M2 holds remain HELD; FIT releases neither.
