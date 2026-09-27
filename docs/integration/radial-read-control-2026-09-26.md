# Offline radial read comparison

Fourth lab, September 26 local / September 27 UTC, 2026.

## Decision and source

The next attention comparison keeps the radial information in Claude's native
Lorentz reader and changes its distance-to-score law. This implements a research
question; it is not a new language result or a serving-policy change. The owning
[work card](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5851517971)
records scope, ownership, distinct outcomes and cost before execution.

The native reader is derived from the continuous implementation in
[PR #1401 at dcf979df](https://github.com/UOR-Foundation/uor-r4/blob/dcf979df687ac867277ad8f3fc7dbc53393f67a8/crates/uor-r4-training/src/joint_model.rs).
The [independent review](fourth-lab-native-read-review-2026-09-26.md) preserves
the scope of that lab's reported results. Its converted transformer engine,
integer Lorentz kernel, wide learned serving scale and proposed D10 are not
adopted by this offline comparison. During development PR #1401 merged into
main, so the final increment extends that existing reader with LorentzAffine
and preserves its upstream behavior. It does not overwrite the concurrently
delivered Lorentz implementation with a second implementation.

## Mechanism and control

For query and key vectors, define

```text
q0 = sqrt(1 + ||q||²), k0 = sqrt(1 + ||k||²)
z  = max(q0*k0 - q·k, F32(1 + 1e-6))
Lorentz distance: d(z) = acosh(z)
score = exp(learned_log_beta) * (learned_offset - d(z))
```

The lifted cross-dot between `[q, -q0]` and `[k, k0]` is exactly `-z` before
clamping. Thus a lifted dot product followed by the same acosh kernel is the
same function, not a separate explanatory control. At read width 64, the lift
uses 65 deterministic features without additional learned feature weights.
Radius is query-dependent compatibility information; an angle-only code would
discard it. None of this algebra establishes predictive benefit.

The affine control uses a fixed reference derived from the existing initializer:

```text
d_star = lorentz_initial_offset(config)
z_star = cosh(d_star)
d_affine(z) = d_star + (z - z_star) / sinh(d_star)
score_affine = exp(learned_log_beta) * (learned_offset - d_affine(z))
```

The reference stays fixed when the learned offset changes. Both arms share the
query/key maps, radial lift, clamp, two learned scalars, admission, age terms,
NoRead, recurrent transport, values and copy/output composition. The affine arm
matches distance and derivative at the reference point. It does **not** match
every initial logit, attention entropy, NoRead mass or gradient. These residual
differences must be recorded in any experiment rather than hidden under a
claim of identical initialization. Shared learned matrices can be identical.

Lorentz and affine lifted scores rank raw candidates before age identically;
nonlinear spacing changes their weights and gradients. Adding learned age
after the score can also change final ordering. The nonlinear Lorentz law
produces approximately power-law weights in large `z`, whereas the affine
control gives an exponential penalty in `z`. Whether this helps distant
context and generated language is a hypothesis for the paired study.

## Research and serving boundary

`Dot` retains its old parameter inventory and omitted default configuration
field, preserving the declared old serialization contract. Lorentz preserves
Claude's exact continuous numerical contract so the reused operator does not
create an unnecessary checkpoint-format fork. LorentzAffine has its own
serialized operator identity and versioned continuous contract. Checkpoint
loading retains exact contract validation; a geometry label alone is insufficient.
Source-derived compatibility checks are separate from replay of Claude's
continuous trained checkpoints, which remain external to the newly shared
[native packet](../evidence/native-lorentz-packet-2026-09-26/README.md). Its four
packed QAT models are available, but are not a continuous parent for this control.

The new LorentzAffine control supports continuous offline training and
evaluation only. Its quantization, packed export and integer loading are
explicitly refused until the corresponding operator is implemented and adopted.
Existing Dot and upstream Lorentz paths retain their behavior. This prevents
an affine-control artifact from silently receiving another computation. The
current chatbot's persistent and dialogue readers are separate integration
paths; supporting a new geometry requires both, along with compatible value
width. Upstream now rejects unsupported Lorentz/width-128 conversational use.
The two cross-branch risks are recorded in the
[integration review comment](https://github.com/UOR-Foundation/uor-r4/pull/1401#issuecomment-5851495403).

The existing `joint-fit CAMPAIGN_JSON NEW_REPORT_ROOT {cpu|metal}` command reads
`Campaign.model` as `JointConfig`. Set `model.read_geometry` to `lorentz` or
`lorentz_affine` in a prospectively declared continuous research campaign, with
no quantization transition for this comparison. `joint-evaluate` reads the saved checkpoint identity.
This command wiring was inspected in source; no new paired campaign has been
prepared or executed for this implementation increment.

## Validation and next decision

The final source `48fd43c16cf2596e6379a5869500b19d43571731`, including main
`ab90b9e5`, was compiled with its actual `UOR_BUILD_SOURCE_COMMIT`. All four
`radial_read_` checks and both retained `calibrated_shadow_` checks pass on the
same CPU test executable. They cover forward/backward credit to query/key/value,
NoRead and scalar parameters; both reader paths and causality; checkpoint
identity; retained Dot/Lorentz behavior; and unsupported affine serving.
The [execution receipt](../evidence/offline-radial-read-validation-2026-09-27.json)
binds the five source files, executable, prior attempts and cost. An earlier
retained check failed because the executable had an unbound source identity;
the source-bound build resolves it without changing the model or the check.
These are functional checks, not fitted-model or language evidence.

The final build/check commands took 79.98 and 11.10 seconds; the tests themselves
took 3.32 and 11.07 seconds. Peak command RSS was 2,237,857,792 bytes. Including
the superseded builds and failed provenance attempt, this increment used
462.02 seconds of compile/check command time. The cumulative elapsed receipt
also charges preparation, review and shared build-slot waiting; no paired
training study, capability evaluation, accelerator work or paid compute ran.
All six declared implementation risks are resolved; further broad testing is
not a prerequisite to selecting the next research comparison.

Before any new fit, select one paired development comparison with fixed data,
draws, shared initial matrices, optimizer, full 256-token access and complete
resource projection. Keep training length, evaluation length, memory access
and vector width separate. Compare actual generated behavior along with loss
and read/copy diagnostics; retain the existing Dot result as an anchor.
Keep the selection policy identical between the reader arms. The concurrently
delivered [emission diagnostic](emission-selection-diagnostic-result-2026-09-27.md)
localizes several exposed failures to sampling departures, without executing
an alternative trajectory. Its separate same-checkpoint policy investigation
can proceed under its existing owner; it is neither a new reader gate nor
evidence that the radial score has fixed the prose failures.

- If the affine arm retains the useful benefit, prefer its simpler score law
  provisionally while retaining the radial geometric mechanism.
- If Lorentz improves useful generated behavior and loss, its nonlinear score
  family has evidence at that scope. Unique curvature, hierarchy and energy
  claims still require their own evidence.
- If only read/copy calibration changes, or lower loss does not improve emitted
  behavior, use the existing emission-localization track to select the next
  integration change.
- If both fail the language objective, preserve them and advance the independent
  language-interface or recurrence work. Do not automatically expand a sweep.

The optional signed-2I design remains available for a demonstrated unmet need;
this comparison does not demote the broader prime/zeta/R4 programme.

## Initialization decision before a learning run

After implementation, source/artifact review found that the canonical evaluator's
retained TinyStories train/dev stores, tokenizer and prompts are locally available.
A fresh matched pair is feasible without importing Claude's scratch tokenizer.
The retained Dot checkpoint cannot simply resume as a different reader; its
strict configuration identity correctly rejects that substitution.

Do not launch a long default-scale fit merely because it is already executable.
The canonical constructor starts at beta about 12.92 for state256/read64, whereas
Claude's context256 result uses beta 1. Its context128 four-seed result supports
the latter as a more consistent start, not a universally better one: flat-start
NLLs are 3.236, 3.246, 3.222 and 3.222, versus 3.450, 3.234, 3.218 and 3.215 for
the matched start. These are that packet's reported development results, not a
new fourth-lab replay or comparable TinyStories scores.
[Retained study](hyperbolic-cycle3-2026-09-26.md).

The campaign now accepts the typed optional `read_initialization: "unit_scale"`
field. `Campaign::fresh_model` sets only learned log-beta to zero for either fresh
radial arm, before optimizer construction and initial evaluation. The absent
default preserves the existing constructor and old serialization. Resume retains
the evolved scalar and binds the initializer in campaign/checkpoint provenance;
missing or changed declarations are rejected, including quantization/projection
transitions. Dot rejects this override. The pinned Lorentz numerical contract
remains byte-compatible and describes the constructor default; campaign/report
metadata separately records the override and the actual starting scalars.

The no-update `radial-startup` example uses that same initialization path, four
prospectively fixed natural development blocks (0, 21, 42, 63) at full context256,
state256/read64, shared seed240924 and all causal history. It records parameter
identity, actual scalars, language gradients, read/NoRead mass, conditional read
entropy and eight-token untrained continuations through the existing generation
API. The text-generation API decodes/re-encodes its 32-token source prefix; actual
prompt IDs are retained, with no token-boundary equivalence assumption. It
constructs no optimizer and performs no training updates or calibration search.
The [prospective work card](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5851873486)
records the decision and full resource envelope.

### Executed unit-scale startup result

All four initializer checks and both retained calibrated-checkpoint checks pass
on source `17f0cde0`; the only later source change (`0aa932dc`) clarifies that
measured zeros are descriptive observations. The implementation preserves
default serialization, changes only the shared log-scale, rejects changed
lineage and reproduces the next update after loading an evolved scalar and
optimizer. Compile/check commands took 488.98 seconds, including the example
build; their maximum command RSS was 1,608,859,648 bytes. No broader test suite
or additional calibration was needed.

Antigravity independently launched the exact example from this worktree at
`0aa932dc`. The fourth lab verified its executable hash, every sealed file,
evaluator/data/tokenizer identity and all four actual input windows, and retained
a byte-identical import plus the executable. The existing native report verifier
accepts both original and import. This completed 202.23-second execution is
reused; the planned duplicate fourth-lab run was not launched. Its launcher
thread limit and true peak RSS were not verified, so it supplies no performance
or memory qualification. The [complete receipt](../evidence/radial-unit-start-validation-2026-09-27.json)
separates own checks/builds, external model execution and complete elapsed cost.

| Fixed startup observation | Lorentz | LorentzAffine |
| --- | ---: | ---: |
| Actual beta / offset | 1 / 5.33172894 | 1 / 5.33172894 |
| Mean next-token NLL, nats | 6.819337 | 6.829711 |
| Mean causal read mass | 0.97011269 | 0.97011410 |
| Mean NoRead mass | 0.02988731 | 0.02988590 |
| Mean conditional read entropy, nats | 4.352088 | 4.365421 |
| Exactly zero causal read positions / 1,020 | 0 | 0 |
| Finite nonzero required gradient arrays | 6 / 6 | 6 / 6 |

The shared initial arrays match and remain unchanged through backward and
generation. At the first common read, Lorentz-minus-affine read log-odds are
0.6583, 0.3285, 0.6821 and 0.2815 across the four windows. Their later states
diverge, so the tiny reversed ordering of aggregate NoRead means does not
contradict the common-input inequality. These are untrained startup observations,
not an advantage result. Both paths have active language credit on these inputs;
there is no numerical basis for a calibration correction or tuning sweep.

Supplementary untrained outputs are `little<|bos|> wood<|bos|> play a favorite Tim`
and `quiet a of a to<|bos|> his favorite`. Both reach the eight-token cap. Their
actual prompt begins `[0, 0, ...]`: the decoded development prefix contains BOS
and the unchanged text-generation API prepends another. This input condition is
retained explicitly; it does not affect the separately recorded gradient
windows and supplies no prose-quality evidence. Changing this supplementary
prompt alone does not justify repeating the startup observation.

The affine tangent is above the concave acosh distance, so at identical current
query/key tensors and scalar values its read scores are no greater than the
Lorentz scores. Its NoRead odds are consequently no smaller when null and age
logits are fixed. Beta 1 reduces the raw score gap relative to the default; it
does not make the read distributions or subsequent trajectories identical.
Record one no-update startup comparison on fixed canonical prefixes: actual
scalars, read/NoRead mass, conditional read entropy and query/key/scalar gradients.
Only numerical loss of an active learnable read would justify a specific
calibration correction; ordinary differences do not justify a tuning sweep.

A proposed 2,048-update-per-arm study would be 8,388,608 target visits per arm at
B16/T256. Prior Dot M1 runs took 3.295 and 4.992 seconds per update and roughly
3.4–3.5 GB peak sampled process RSS; extrapolating gives about 3.8–5.8 hours for
that pair, before preparation/delivery and unmeasured radial overhead. Neither
that dose nor its compute has been selected. Choose the complete exposure and
resource projection after the initializer and startup evidence, preserving
open development and actual generated behavior as the research objective.

The next learning design should prefer an explicit transfer of shared learned
Dot parameters into both radial arms over using that short fresh dose to judge
prose. This is a new adaptation lineage, not a permitted geometry-changing
resume. Bind the actual parent, evaluator, tokenizer and copied parameter
hashes; retain identical shared arrays and initialize only the two new radial
scalars. Reset Adam identically and declare a new common sampler seed/local
step zero, recording prior exposure separately. Actual startup with the learned
query/key scale must be observed before selecting its dose. The step15,672
continuous candidate remains a retained negative, not an accepted replacement.
This design is not implemented or launched by the initializer increment.

The [mainline termination-objective work card](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5851930160)
already owns its loss change and paired dose. Do not duplicate it or confound
the first score-law comparison with a new emission channel. A direct learned
read-to-vocabulary residual is a distinct later hypothesis for wrong-source
emission; no causal diagnosis or implementation is established for it yet.
