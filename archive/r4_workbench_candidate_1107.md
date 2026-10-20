# Native Four-fact workbench source candidate — #1107

**Decision:** freeze the exact workbench candidate as reviewable source and
stop at **`WORKBENCH_CANDIDATE_SOURCE_FROZEN_UNBUILT`**.

This decision supersedes #1107's earlier automated build expectation under the
repository-wide [`deterministic_source_only`](integration/agent-execution-policy.md)
policy delivered by #1108/#1109 and the [issue amendment](https://github.com/UOR-Foundation/uor-r4/issues/1107#issuecomment-5535521748).
Automated agents may inspect and correct source, update evidence records and use
the protected Git path. They may not run or dispatch compilation, tests, model
work, QA, service/browser checks, operating-system diagnostics, supervisors or
retries.

## Delivered source boundary

The candidate adds the dedicated `uor-r4-workbench` workspace crate, one
`r4-workbench` executable, its private same-executable worker and comparison
entries, and the first Four-fact shell specified by
[ADR-0006](adr/0006-native-four-fact-workbench-service.md) and its
[machine contract](r4_service_contract_1105.json). The source leaves the
existing root server and its routes unchanged.

The executable source contains four mutually exclusive entries:

- `--config ABSOLUTE_PATH CONFIG_SHA256` for the parent service;
- `--internal-worker` for the private worker;
- `--private-compare-host RELEASE_PATH RELEASE_SHA256 ADMISSION_PATH
  ADMISSION_SHA256` for the separately admitted empirical adapter; and
- `--private-metadata` for a future zero-model binary/runtime metadata record.

The parent source declares the seven #1105 routes, strict wire objects,
single-slot lifecycle, bounded static assets, same-origin boundary, framed
worker protocol, cancellation/deadline/reap states and the sole
`answer_four_fact_raw_text/v1` operation. The private comparison source remains
outside the public route surface. Its [release contract](r4_workbench_private_release_1107.json)
requires fresh executable permission separately from the immutable provenance
of the previously exported artifact. It now defines an exact `row_cap`
population and immediate `done` reply after the last result, so completion does
not depend on EOF or an extra blocking frame read.

## Independent static review and corrections

Independent review inspected the delivered branch against ADR-0006, the #1105
machine contract and current `origin/main`. It found five source-level
contract mismatches before delivery:

1. Mutation handlers checked instance/uint53 shape before the outer schema,
   contrary to the frozen error precedence. The handlers now admit the outer
   schema first, followed by instance, model/operation and route-specific
   guards.
2. Root configuration validation rejected an incomplete optional host
   acceptance pair before listening. The pair now reaches the optional evidence
   adoption lane and leaves discovery available with
   `UNAVAILABLE_NATIVE_QUALIFICATION`.
3. The private comparison adapter waited for EOF or an extra frame after the
   exact final row. It now validates frozen counts and emits `done` immediately;
   external admission owns rejection of any attempted frame beyond the exact
   population.
4. The worker launch path did not explicitly prevent arbitrary inherited file
   descriptors from surviving `exec`. It now marks every descriptor above the
   retained executable fd 3 close-on-exec, leaving only parent-created standard
   pipes and fd 3 in the executed worker image.
5. HTTP admission classified the target before enforcing POST media type,
   contrary to the frozen precedence. It now rejects a missing/invalid POST
   content type before known-route classification, and its static expectations
   reflect that order.

The final [review record](r4_workbench_candidate_1107_review.md) covers static
source/contract consistency only. It does not establish compiler acceptance,
executable correctness, memory/process safety, service behavior, numerical
behavior or target portability.

## Source basis and claim boundaries

The accepted [#1105 source audit](r4_service_contract_1105_sources.md) remains
the research basis. NEMESIS motivates explicit state, transition and primitive
declarations, but its arbitrary-network, zero-error and energy statements are
neither proof nor behavior evidence for this crate. W33 supplies useful
immutable-content, receipt and path-copy patterns without proving storage
authenticity, tamper resistance or operating-system isolation.

Pinned UOR sources require separately typed content, realization, derivation
and topology identities. `uor-addr` JCS/NFC identity and commutative composition
do not define ordered neural wiring; kappa registry bytes are not silently
recast as dCBOR; and a hologram derivation key is not the result bytes. Digest
equality here is a source-integrity relation, not an injectivity proof or a
behavioral result. Donor UI material informs only static visual/state choices;
its worker, cache, fallback, cancellation and model claims do not transfer.

The local project knowledge index was queried read-only. Its retained #1105,
NEMESIS, W33 and UOR records led back to the pinned originals; it did not yet
contain #1107. This delivery updates the tracked roadmap and claim/source
ledger. Automated CLI ingestion is **`NOT_RUN_BY_POLICY`**.

## Evidence classification and limits

- **Mathematical proof:** none supplied. No universal algorithm, protocol,
  security or floating-point guarantee follows.
- **Measured behavior:** none added. Compilation, tests, model loads, forwards,
  comparisons, qualification calls, HTTP/service execution, browser acceptance,
  resource use, performance and target behavior are all
  **`NOT_RUN_BY_POLICY`**.
- **Static source evidence:** the full worktree contains the declared crate,
  entries, routes, schemas, assets and source-level corrections described above.
  Static review can detect textual/control-flow inconsistencies; it cannot show
  that the program compiles or behaves as intended.
- **Unverified hypothesis:** the source may implement the intended workbench
  when manually built and qualified on an accepted target. This remains
  unverified.

The previous empirical result remains `NATIVE_REFERENCE_PRESERVED` only for the
old #1102 artifact, binary/runtime and accepted authoring population.
`LANGUAGE_R4_PRESERVED_CONTROL_WEAK`, `TOKEN_EXPOSURE_DESCRIPTIVE_COMPLETE`,
`CLAUSE_ADAPTER_PRESERVED`, every retained unavailable preparation result,
#973 open and #954 blocked remain unchanged. The accepted reader/core, known
vocabulary/query forms and Four-fact context remain fixed.

## Closure boundary

Protected delivery closes #1107 only. #1084 remains open and unassigned. No
local model/evidence store, sealed input, research artifact, user material or
original mixed-checkout change is deleted.

**One next action after protected closure:** the owner decides whether to
authorize a separate manual qualification workflow. Automated agents do not
dispatch or execute it.
