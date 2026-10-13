# Paper 1 (v6a): All-heights recovery plan (nearfield + farfield + bridge)

This document tracks the work needed to upgrade the **proved nearfield theorem** (low heights, certified rectangles) to the **target all-heights theorem** in `paper1_farfield_v6a.tex`.

## Current state (what is already closed)

- **Manuscript**: `paper1_farfield_v6a.tex`
  - **Closed (unconditional)**: Theorem `thm:nearfield` (zero-free on \(R_{\rm cert}=[0.6,0.999]\times[-20,20]\)).
  - **Target (not closed)**: Theorem `thm:farfield` (zero-free on \(\Re s\ge 0.6\)).
  - **GAP markers**: all remaining obligations are explicitly marked as `\textcolor{blue}{GAP: ...}` and summarized in Section `sec:all-heights`.

- **Verifier**: `verify_attachment_arb.py`
  - `zeta_certify` produces **direct, gauge-free nonvanishing** artifacts:
    - `zeta_certify_sigma06_07_t0_20.json` for \([0.6,0.7]\times[0,20]\)
    - `zeta_certify_sigma07_0999_t0_20.json` for \([0.7,0.999]\times[0,20]\)
  - Pick artifacts exist but are **non-load-bearing** for the nearfield theorem.

**Foundational note (nearfield cleanup)**:
- We attempted a consistent `theta_certify` run with `--arith-det2-mode enclosure` and a rigorous `outer_zeta_proj` build.
  The dominant failure mode was `O_contains0_min_width`: the box enclosure for the projected normalizer \(O_{\rm proj}\) swallowed 0 on many cover boxes, so \(\Theta\) could not be certified.
- To keep the nearfield theorem fully unconditional and audit-friendly, we therefore prefer the **direct `zeta_certify` route** for both rectangles in \(R_{\rm cert}\), which avoids det\(_2\)/outer normalizers entirely.

## Goal (definition of “done”)

Upgrade `paper1_farfield_v6a.tex` so that Theorem `thm:farfield` is **proved unconditionally**, with:

- an explicit **farfield lemma** (no big-O, explicit constants, uniform in \(\sigma\in[0.6,0.999]\)) covering \(|t|\ge T\), and
- if \(T>20\), a finite **bridge certification** covering \(20\le |t|\le T\) via new JSON artifacts and verifier commands,
- and a clean proof that the union of regions equals the target half-plane \(\Re s\ge 0.6\) (plus the trivial \(\Re s>1\) region).

## Strategy overview

We treat the all-heights theorem as a 3-piece union:

- **Nearfield** (already done): \(0.6\le \sigma\le 0.999,\ |t|\le 20\).
- **Farfield** (to prove): \(0.6\le \sigma\le 0.999,\ |t|\ge T\) by a fully explicit analytic lemma.
- **Bridge** (if needed): \(0.6\le \sigma\le 0.999,\ 20\le |t|\le T\) by certified rectangles (finite computation).

## GAP-by-GAP workplan (with deliverables)

### GAP 1 — Choose the right analytic object \(F(s)\)

**Problem**: The farfield lemma template requires an object \(F(\sigma+it)=F_\infty(\sigma)+E(\sigma,t)\) where \(E(\sigma,t)\) has genuine, uniform decay in \(|t|\). Not every natural expression for \(\zeta\), \(\mathcal J\), or \(\Theta\) has that property.

**Work**:
- Extract the precise “pinch object” used in v6a:
  - definitions of \(\mathcal J\), \(\Theta\), and gauges \(\mathcal O\) in `paper1_farfield_v6a.tex` (`eq:J-def` and surrounding).
- List candidate \(F\)’s:
  - direct: \(F=\zeta\) or \(F=(s-1)\zeta\);
  - Schur route: \(F=2\mathcal J\) (Herglotz) or \(\Theta\) (Schur);
  - “smoothed” variants if needed (e.g., Mellin transforms of smooth compactly supported kernels).
- For each candidate, decide whether a farfield decomposition with **decaying** \(E(\sigma,t)\) is even plausible (structural obstruction check: almost periodicity vs smoothing).

**Deliverable**:
- A short “GAP 1 memo” inserted into the plan (and optionally into the paper) that names the selected \(F(s)\), and states exactly what implication it provides (“if \(F\neq 0\) then \(\zeta\neq 0\)” or “if \(|\Theta|\le 1\) then no poles of \(\mathcal J\)”).

**Success criteria**:
- We have a specific \(F(s)\) tied to the proof mechanism, and a credible path to represent its tail as an oscillatory integral with derivative \(L^1\) control.

#### GAP 1 memo (draft; what we learned so far)

**What the manuscript currently uses (v6a)**:
- In `paper1_farfield_v6a.tex` (raw \(\zeta\)-gauge unless stated),
  \[
    \mathcal J(s)=\frac{\det_2(I-A(s))}{\zeta(s)}\cdot\frac{s}{s-1}\cdot\frac{1}{\mathcal O(s)},
    \qquad
    \Theta(s)=\frac{2\mathcal J(s)-1}{2\mathcal J(s)+1}.
  \]
- In `verify_attachment_arb.py`, the compensator is explicitly `B(s)=s/(s-1)` (see `compensator_B`), and the raw arithmetic proxy is `J_arith_raw_zeta(s)=det2_P(s)/zeta(s)*B(s)`.

**Naive “\(t\)-decay to a \(t\)-independent main term” does not look viable**:
- Candidate \(F(s)=(s-1)\zeta(s)/s\) (or similar “main term \(=1\)” normalizations) is *not* empirically close to 1 on \(\sigma\in\{0.6,0.7,0.9\}\) even at \(t\) in the thousands, so a Rouché-style farfield lemma of the form \(|F-1|\le 1/2\) is not currently a plausible closure route without a fundamentally new decomposition.

**Raw-\(\zeta\) Schur route seems incompatible with pointwise diagnostics**:
- Using `det2_full_enclosure` (so **not** just prime truncation) we evaluated \(\Theta_{\rm raw}(s)\) at sample points.
  At \(s=0.6+500i\) the midpoint gave \(|\Theta_{\rm raw}(s)|\approx 1.028\) (stable under larger `P_cut` and higher precision in the diagnostic), which is inconsistent with \(\Theta_{\rm raw}\) being Schur on the full half-plane.
  This does not prove anything by itself, but it strongly suggests: “prove global Schur for \(\Theta_{\rm raw}\)” is the wrong target.

**Proxy outers can still leave \(|\Theta|>1\) spikes at moderate/large heights**:
- A midpoint diagnostic with an `outer_zeta_proj`-style projection outer (built to \(T=1000\)) still exhibited \(|\Theta|>1\) at some sampled \(t\) values (e.g. around \(t\approx 200,320,390,870\) for \(\sigma=0.6\) at step \(10\)).
  This indicates that any “there exists a cutoff \(T\) after which \(|\Theta|<1\) automatically” lemma must be very explicit about the gauge and must address such spikes.

**Decision (provisional)**:
- For an all-heights Schur route, the only structurally grounded candidate is the **canonical outer normalizer** gauge (see `Riemann-final/FF_CANONICAL_ARTIFACT_PLAN.md`), i.e. \(F(s)=\Theta_{\rm can}(s)\) where \(\mathcal O=\mathcal O_{\rm can}\) is the canonical outer.
- This immediately turns GAP 2 into: “build a certified evaluator/encloser for \(\mathcal O_{\rm can}(s)\)” (currently a known missing component; the code has `OuterNormalizerEngineCanonicalStub` which raises `NotImplementedError`).

### GAP 2 — Canonical outer evaluator (make \(\mathcal O_{\rm can}(s)\) computable/certifiable)

**Problem**: The provisional GAP 1 decision (“use \(\Theta_{\rm can}\)”) requires a certified evaluator/encloser for the **canonical outer normalizer** \(\mathcal O_{\rm can}(s)\) on the region of interest. This is currently missing in code and in the Paper‑1 TeX chain.

**Reference spec already in repo**:
- `Riemann-final/FF_CANONICAL_ARTIFACT_PLAN.md` (this is effectively the detailed GAP 2 design doc).

**Work (breakdown into concrete sub-gaps)**:
- **GAP 2.1 — Fix the canonical definition for Paper 1**
  - Specify precisely which analytic function’s boundary modulus defines \(\mathcal O_{\rm can}\) in Paper‑1 notation.
  - Ensure the stable \((X\pm Y)\) evaluation geometry matches the manuscript (see `_theta_arith_point_rig` which already supports the \(\zeta\)-gauge \(X=2\det_2(s)\,s\), \(Y=(s-1)\mathcal O(s)\zeta(s)\)).
- **GAP 2.2 — Effective Poisson/Cauchy truncation**
  - Prove an explicit tail bound for truncating the boundary integral defining \(\log \mathcal O_{\rm can}(s)\) to \(|t|\le T\).
- **GAP 2.3 — Effective boundary approximation**
  - Replace the a.e./BMO boundary trace \(u(t)=\log|F(1/2+it)|\) by a computable surrogate (smoothing / bandlimit), and bound the error explicitly (in a norm strong enough for the Poisson integral).
- **GAP 2.4 — Singularities at boundary zeros**
  - Control the contribution of sets where \(|\zeta(1/2+it)|\) is tiny (or singular), using explicit Carleson/energy bounds already claimed elsewhere in the project, in a way that yields a usable numeric enclosure.
- **GAP 2.5 — Code: implement `OuterNormalizerEngineCanonical`**
  - Add a new engine to `verify_attachment_arb.py` that provides `O_box(s_box)` rigorously and emits a JSON artifact describing parameters, truncation bounds, and certified nonvanishing.
  - Wire `--arith-gauge canonical_zeta` through `--pick-certify` (and optionally `--theta-certify`) so we can reuse the existing Pick machinery for canonical \(\Theta\).

**Deliverables**:
- A TeX subsection in `paper1_farfield_v6a.tex` (or a companion note) stating the exact analytic lemmas used to make \(\mathcal O_{\rm can}\) effective.
- A new code path in `verify_attachment_arb.py` implementing `OuterNormalizerEngineCanonical` with a machine-checkable JSON artifact format.

**Success criteria**:
- We can run (in principle, and eventually in practice) something like:
  - `python verify_attachment_arb.py --pick-certify --arith-gauge canonical_zeta ...`
  and obtain a certified Pick gap + a certified tail bound in canonical gauge.

**Fallback plan B (kept alive)**:
- If canonical outer cannot be made effective, revisit the original “explicit farfield \(t\)-decay lemma” route, but this will require a different \(F(s)\) than the raw objects in Paper 1 (see GAP 7 notes).

### GAP 3 — Uniform nonvanishing margin \(m>0\) for the main term

**Problem**: Need \(m=\inf_{\sigma\in[0.6,0.999]}|F_\infty(\sigma)|>0\).

**Work**:
- Prove analytically (or certify numerically in 1D with interval arithmetic) a uniform lower bound on \(|F_\infty(\sigma)|\).

**Deliverable**:
- Explicit numeric lower bound \(m\) and its proof/certificate.

### GAP 4 — Explicit tail constant \(C_k^{\max}\) and cutoff \(T\)

**Problem**: Need \(C_k^{\max}\) explicit and uniform in \(\sigma\), then choose \(T\) from \(C_k^{\max}/T^k\le m/2\).

**Work**:
- Bound \(C_k(\sigma)\) and \(\sup_{\sigma\in[0.6,0.999]} C_k(\sigma)\).
- Prefer a **certified 1D** computation (interval arithmetic) if the integral is explicit.

**Deliverable**:
- A computed/certified \(C_k^{\max}\), chosen \(k\), and an explicit \(T\).

### GAP 5 — Bridge artifacts if \(T>20\)

**Problem**: If farfield only starts at \(T>20\), we must certify a finite number of rectangles covering \(20\le |t|\le T\).

**Work**:
- Extend `verify_attachment_arb.py` invocation list (no new math required) to cover additional \(t\)-ranges:
  - `zeta_certify` for right rectangle(s) \([0.7,0.999]\times[t_0,t_1]\),
  - and/or `theta_certify` for left rectangle(s) \([0.6,0.7]\times[t_0,t_1]\),
  - with symmetry to cover negative \(t\).
- Update Appendix audit manifest and Table 1 accordingly.

**Deliverable**:
- New JSON artifacts + updated paper table + updated audit checklist.

### GAP 6 — Gauge consistency

**Problem**: Farfield lemma might be proved for a different normalization (gauge) than the nearfield artifacts.

**Work**:
- Prove the gauge factor is holomorphic and nonvanishing on the relevant regions so pole-exclusion transfers.
- If needed, add explicit certified lower bounds for gauge factors on bridge rectangles.

**Deliverable**:
- A short lemma in the paper: “Gauge changes preserve pole sets on regions where the gauge is nonvanishing.”

### GAP 7 — Structural obstruction check (decay vs almost periodicity)

**Problem**: Naive Dirichlet series / Euler product expressions are typically almost periodic in \(t\) and do **not** decay, so the IBP farfield lemma cannot be applied unless the representation genuinely introduces smoothing / absolute continuity.

**Work**:
- For the chosen \(F\), explicitly verify that the tail term is an oscillatory integral with \(g^{(k)}\in L^1\) and vanishing boundary terms.

**Deliverable**:
- A paper-ready remark/lemma explicitly confirming the hypotheses of Lemma `lem:ibp-farfield` / `lem:mellin-farfield` and bounding boundary terms.

## Work log

- 2026-01-29: Created `paper1_farfield_v6a.tex`, inserted GAP markers, added Section `sec:all-heights` with IBP/Mellin templates and a concrete checklist, and confirmed TeX compiles.
- 2026-01-29: Added this plan file and wrote the GAP 1 memo (candidate \(F(s)\) triage + diagnostics).
- 2026-01-29: Started a nearfield det\(_2\) enclosure sanity-check run:
  - output: `theta_certify_sigma06_07_t0_20_outer_zeta_proj_det2_enclosure_draft.json`
  - result: `ok=false` in 60s (failures with `abs_theta_upper > 1` near \(\sigma\approx 0.69, t\approx 18.4\)).
  - interpretation: this run used `outer_mode=midpoint`, so \(O_{\rm proj}\) was built from truncated boundary samples while \(\det_2\) was evaluated with an enclosure; next step is to rerun with `outer_mode=rigorous` (so the outer build uses the same det\(_2\) mode/enclosure consistently).
- 2026-01-29: Reran the consistent nearfield test with `outer_mode=rigorous`; it still failed, now because `O_contains0_min_width` dominated, and we wrote a draft artifact:
  - `theta_certify_sigma06_07_t0_20_outer_zeta_proj_det2_enclosure_rigOuter_draft.json`
- 2026-01-29: Generated a direct left-rectangle nearfield certificate:
  - `zeta_certify_sigma06_07_t0_20.json` with `results.ok=true` and `zeta.min_abs_lower=9.432519465010592e-06`.
- 2026-01-29: Began GAP 2 code scaffolding for canonical outer evaluation:
  - enhanced `OuterNormalizerEngineCanonicalStub` in `verify_attachment_arb.py` with `load_cache()` + rigorous `logO/O/O_box` evaluation on interior points, once a canonical boundary cache exists.
  - wired `--pick-certify --arith-gauge canonical_zeta` to require `--outer-cache-load` (so canonical-gauge Pick artifacts become a concrete next deliverable once the cache-builder exists).

