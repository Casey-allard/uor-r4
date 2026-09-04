# Build-first pre-alpha execution

The project is in `build_first_pre_alpha` mode. The former
`deterministic_source_only` policy is superseded. It prevented the project from
being compiled or run and turned routine development into evidence production.

The pre-alpha objective is concrete: load a source-free model artifact and
produce prompt-dependent text through the repository CLI or local service on
the target local hardware. Work that does not materially help reach that
behavior is deferred.

## Routine work

Use one active task in an isolated full worktree from refreshed `origin/main`.
Implement the feature, compile it, and run the smallest command that directly
exercises the changed behavior. Focused tests are useful when they protect a
specific failure mode; they are not required merely because code changed.
Deliver through a protected pull request. Preserve user material, unique
artifacts, and prior negative results.

Agents may run builds, linters, focused tests, models, training, evaluation,
CLIs, services, and browser flows when the active implementation requires
them. A run estimated to exceed 15 minutes, create more than 10 GiB, or incur
external cost requires explicit owner or active-issue authorization, a hard
resource limit, and a stop condition.

Automatic retry campaigns are prohibited. After a concrete failure, inspect
the existing output, make one direct source or input correction, and rerun once
when that change plausibly addresses the failure. Otherwise park the command
and report the blocker. Do not construct supervisor programs, watchdogs,
receipt harnesses, workspace capsules, or environment-diagnostic campaigns.

## Deferred until a working alpha

Formal proof work, claim-ledger maintenance, knowledge-index maintenance,
independent review, frozen experiment contracts, fresh-process replay,
receipt packages, duplicate roadmap synchronization, publication work,
NEMESIS/W33 mapping, and broad release certification are not routine pre-alpha
requirements. Only an explicit owner instruction can activate one before the
working-alpha condition is met.

Historical records remain available and retain their original meaning. They do
not impose their old process on new work. A routine PR needs the primary code
or product deliverable, the direct behavior result when execution matters, and
a concise statement of the remaining limitation and next action.

## Claim boundary

Do not call unrun code working. Keep mathematical proof, measured behavior, and
unverified hypotheses distinct. This honesty rule does not require a proof
artifact, claim ledger, evidence dossier, or separate review.

After the alpha exit condition is met, the owner can select the small amount of
testing, formalization, reproducibility, and release work that the implemented
product actually needs.
