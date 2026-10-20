# Same-checkpoint emission/selection diagnostic result

**Scope.** Executes the predeclared
[diagnostic plan](emission-selection-diagnostic-plan-2026-09-27.md) after the
[completed full-context continuation](language-continuation-result-2026-09-26.md).
Read-only: the same step-15,672 checkpoints, frozen prompts/scids/seeds/policy
and Read mode; no weight, decoding, gate, panel or candidate change. It cannot
overturn the frozen prose verdict and promotes nothing.

## Instrument and parity gate

A new additive `joint-emission-trace` subcommand in `uor-r4-training`
(`generate_traced`; the retained `generate_inner` is untouched) replays the five
frozen prompts and records the missing per-decision evidence: the exact Q32
sampling table (temperature 0.8, top-40), the draw threshold, the full-row rank
of the selected token, per-token vocabulary/copy component split and the full
source-weight row for story seed 2014. Executed source `9bed4d8b` (plan
`4226f527`, parent `19991e40`); binary sha256 `cc113728…`; built offline
`--release -p uor-r4-training --features metal,cpu-accelerate,reference-accelerate`
with `UOR_BUILD_SOURCE_COMMIT=9bed4d8b`.

The replay must reproduce the retained packet exactly or the instrument is
invalid. It did:

- **`PARITY_EXACT` in both arms**: 0 mismatches, maximum absolute float delta
  `0.0`, across all five stories and every decision (quaternion
  128/128/76/128/128; householder pair 128×5). A separate principal comparison
  of every retained field in Python also returned 0 mismatches.
- The mandatory runtime instrument checks passed at every decision: the
  replicated Q32 interval contains the actual draw, and the replica post-draw
  sampler state equals the recorded state. Maximum mixture reconstruction error
  `7.24e-8` (quaternion) / `6.34e-8` (householder pair).
- A pre-rebase run (`ca66663b`) and the delivered post-rebase run (`9bed4d8b`)
  produced identical selected tokens, probability hashes and generated ids.

Identities, roots and hashes are in the
[evidence record](../evidence/emission-selection-diagnostic-2026-09-27.json).

## Witness result (predeclared classification)

Selected and model-preferred tokens at the witnessed failing decisions (story
seed 2014). `q` is the declared sampling probability; the model-preferred token
is the rank-0 candidate (the greedy token).

| Arm | Decision | Selected | rank / q | Model-preferred | q_best | ratio | Class |
|---|---|---|---|---|---|---|---|
| Quaternion | 37 | ` an` | 2 / 0.1206 | ` a` | 0.2754 | 0.438 | departure |
| Quaternion | 38 | ` idea` | 0 / 0.7945 | ` idea` | 0.7945 | 1.000 | top_choice |
| Quaternion | 39 | ` called` | 6 / 0.000383 | `.` | 0.9709 | 0.000395 | departure |
| Quaternion | 40 | ` out` | 1 / 0.1648 | ` a` | 0.2616 | 0.630 | near_tie |
| Quaternion | 41 | ` in` | 8 / 0.00746 | `.` | 0.6069 | 0.0123 | departure |
| Quaternion | 42 | `.` | 1 / 0.1550 | ` the` | 0.6946 | 0.223 | departure |
| Householder pair | 25 | ` wanted` | 0 / 0.3278 | ` wanted` | 0.3278 | 1.000 | top_choice |
| Householder pair | 26 | ` the` | 1 / 0.01463 | ` to` | 0.9591 | 0.0153 | departure |
| Householder pair | 27 | ` most` | 2 / 0.02984 | ` new` | 0.6571 | 0.0454 | departure |
| Householder pair | 28 | ` so` | 9 / 0.01682 | ` exciting` | 0.4188 | 0.0402 | departure |
| Householder pair | 29 | ` exciting` | 1 / 0.1411 | `c` | 0.1876 | 0.752 | near_tie |
| Householder pair | 30 | `!` | 5 / 0.06701 | `,` | 0.3315 | 0.202 | departure |

Witness counts: **8 departure, 2 near_tie, 2 top_choice, 0 copy_dominated**.
Whole seed-2014 story (128 decisions per arm): quaternion 61 top_choice /
18 near_tie / 49 departure / 15 copy_dominated; householder pair 63 / 19 / 46 /
16\. All five stories are in the evidence record.

**Principal semantic read (distinct from the mechanical class).** At the
departures the model's own preferred token was a short grammatical or terminal
continuation (` a`, `.`, ` to`, ` new`, ` exciting`, `,`), while the sampled
token was a malformed continuation. This is a same-prefix top-token reading; it
does not establish a coherent greedy trajectory. At the 15–16 copy-dominated
decisions in the seed-2014 stories, the copy path matters, but none is a
witnessed decision.

## Decision (predeclared rule applied)

The predeclared rule maps a predominantly `departure`/`near_tie` witness set to
the selection interface, `copy_dominated` to copy/read attribution, and
`top_choice` to emission/ranking. The witnesses are 8 departure + 2 near_tie
out of 12, with no copy-dominated decision.

**Supported interface: selection policy.** A later, separately authorized
bounded same-checkpoint selection-policy diagnostic (no weight change) is the
supported next work card; it must also explain the retained greedy source-panel
regressions. No root cause is claimed resolved; those ranking-side failures
remain independently actionable. No decoding or weight change is executed or
authorized here, and no candidate is promoted.

## Limits

- Twelve witnessed decisions at one exposed seed-2014 story per arm; the classes
  use the predeclared 0.5-ratio cuts and are descriptive, not inferential.
- Sampling probabilities are for the declared Q32/temperature-0.8/top-40 policy
  and do not revise the frozen prose verdict.
- A better-ranked token is not a coherent alternative trajectory; no greedy or
  alternate rollout was executed.
- Copy-dominated decisions exist elsewhere in the same stories (15–16 per arm)
  but not at the witnesses.
- No mechanism outside the emission/selection interface is promoted or demoted.

## Cost and delivery

- Focused release check 54.2 s; first release build 26.8 s; post-rebase release
  rebuild 213.4 s; focused tests 15.9 s (4 pass); witness runs 7 s / 8 s.
- Four report roots total 22.5 MiB; the new SSD target is 1.4 GiB
  (`/Volumes/UOR-Workspace/BuildCaches/uor-r4-emission-20260926-1`); no
  internal-disk target; no fit; no external compute.
- Complete elapsed charged once to the shared ledger; receipt
  `ledger-emission-trace-delivery-1.json` in the continuation investigation
  directory.
- Delivered through a protected PR referencing #973 and #820; the owning issue
  and run handoff are updated with outcome, limits and next action.
