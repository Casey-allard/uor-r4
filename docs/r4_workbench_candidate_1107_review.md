# Independent static review — #1107 workbench source candidate

## Verdict

**`ACCEPT_SOURCE_ONLY`** for terminal
**`WORKBENCH_CANDIDATE_SOURCE_FROZEN_UNBUILT`**.

The review covered the complete live worktree on
`codex/issue-1107-workbench-source-only`, based on
`origin/main@aff4acc41f07b866c0f4152440e9028d3cec8088`, including the candidate
crate and all source corrections below. It compared the delivered source with
[ADR-0006](adr/0006-native-four-fact-workbench-service.md), the
[#1105 machine contract](r4_service_contract_1105.json), the current
repository-wide [agent policy](integration/agent-execution-policy.md), and the
original source boundaries retained by the #1105 audit.

This is acceptance of a static source/contract mapping. It is neither build
acceptance nor empirical behavior evidence.

## Findings and disposition

1. **Request error precedence — corrected.** The four mutation handlers had
   checked instance or uint53 shape before the outer request schema. They now
   apply outer schema first, then instance, model/operation and route-specific
   guards, matching the machine contract's frozen precedence.
2. **Optional host acceptance — corrected.** Root configuration validation had
   rejected an incomplete host-acceptance path/digest pair before the listener
   could expose discovery. The root configuration now admits that shape into
   the optional evidence lane, where it leaves the model unavailable with
   `UNAVAILABLE_NATIVE_QUALIFICATION`.
3. **Private comparison completion — corrected.** The adapter had blocked on
   EOF or an additional frame after processing the exact `row_cap` population.
   It now validates final counts and emits `done` immediately. The private
   release contract states that external admission/supervision owns rejection
   of any attempted extra frame.
4. **Inherited descriptors — corrected in source.** The worker launch now
   captures the descriptor-table bound before fork, duplicates the verified
   executable to fd 3, marks every descriptor above fd 3 close-on-exec, and
   clears close-on-exec only for fd 3. Together with piped standard streams and
   `env_clear`, this is the source mechanism for excluding inherited listener
   and credential descriptors from the executed worker image.
5. **HTTP media-type precedence — corrected.** POST Content-Type admission now
   precedes target classification. The static source expectations for both a
   known GET-only route and an unknown route therefore select
   `UNSUPPORTED_MEDIA_TYPE` when a POST omits Content-Type.

No blocking source/contract mismatch remained in the final static re-read.
`git diff --check` reported no textual whitespace error; that is a Git source
check only.

## Evidence and limits

The current `origin/main` safety policy and static CI guard remain present and
unchanged. A preliminary comparison of the old standalone candidate commit to
the new base made those later files look absent; that observation was withdrawn
after review moved to the actual cherry-picked delivery branch.

The review used source reads, textual search and reviewable Git diff only. The
following remain **`NOT_RUN_BY_POLICY`**:

- compilation, linking, tests, linting and target API availability;
- `/dev/fd` execution, fd 3 identity, close-on-exec behavior, listener and pipe
  behavior, signal delivery, cancellation, deadlines and confirmed reaping;
- HTTP exchanges, service lifecycle, browser behavior and asset delivery;
- artifact intake, model loading, qualification, answers, private comparison,
  numerical preservation and replay; and
- measured resource bounds, performance, security properties and portability.

No mathematical proof is supplied. Static acceptance does not establish that
the source compiles, that the executable is safe or correct, or that any model
or product behavior occurs. All prior measured, negative, unavailable and
blocked records retain their original scope.

## Closure recommendation

Protected delivery may close #1107 at
`WORKBENCH_CANDIDATE_SOURCE_FROZEN_UNBUILT`. #1084 remains open and unassigned.
The sole next action is an owner decision on whether to authorize a separate
manual qualification workflow; automated agents do not dispatch or execute it.
