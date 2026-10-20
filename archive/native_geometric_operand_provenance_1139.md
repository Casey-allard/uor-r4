# Independent computed-result selection — #1139 / #1140

## Decision, 2026-09-06

Retain `af337c28` as the next bounded native artifact. It generates **12/16**
predeclared three-turn name/number/computation-order transfers, versus **3/16**
for `43c54db3`. Its matched exact-code continuation also gets **12/16**: this
experiment establishes no angular-distance advantage over that control.
All twelve trajectories that generate their required intermediates select the
requested result correctly. The four failures occur at the first literal
response and never reach the new selector. Full #1139/#1140 acceptance remains
unmet. General syntax, prose, reasoning, frontier capability and whole-model
laptop efficiency are not established.

For unseen `luma/tavi` and `bela/zori`, the model actually generates 20+3 ->23
and 6+9 ->15. It can then copy either named total or add4 to it, producing
23,15,27,19. Reversing which complete computation comes first preserves all
four outputs. In the second transfer world, putting `kira/fenn` first yields
`Unknown` instead of 7. Expected intermediate text is never inserted.

The [machine evidence](evidence/native_geometric_operand_provenance_1139.json)
binds source revisions, artifacts, data, reports, counters and command receipts.
The local artifact root is
`/Users/casey.allard/uor-r4/.uor-models/native-typed-value-2026-09-05`.
The selected artifact is `independent-warm-angular/model.json`, CID
`blake3:af337c28ade71ae4ef01a888c8f3ddb9b240096ff2155d79d37de0773d6ece1a`.
Its exact structural parent remains `bb79456b`; offline initialization comes
from `43c54db3`. Both identities are bound in the artifact. This replaces the
optional role component rather than stacking another depth-specific head.

## Representation and learned execution

The existing value record retains exact numeric payload, derivation action and
operand IDs, plus four lexical cues captured at literal entry. The new optional
`operand_provenance` role feature compares the current query's sixteen retained
whole words against those literal cues, using ASCII case-folded exact byte
identity. Query/cue position matches become four-bit masks. Exact Copy/Add
ancestry propagates those masks to computed candidates; Copy preserves the
origin's mask and Add unions its parents' masks. Missing/evicted ancestry has
an explicit unavailable bit. Source order and numeric payload do not define
these features. No name whitelist or hardcoded owner-selection rule is added.

This abstraction **retains** which query positions match which retained cue
positions among a result's ancestors. It **compresses** ancestor occurrences
by union and **loses** which parent contributed a shared match, multiplicity,
operand ordering inside that union, words outside the four-cue/sixteen-query
windows, and ancestry beyond retained records. Exact IDs/payloads remain in the
existing value state; the mask is selection metadata, not value reconstruction
or a universal semantic metric.

The existing learned two-lane signed-H4 feature fold and three action landmarks
jointly score Copy, Add and no-operation over the unchanged candidate support.
It still uses exact derivation depth, canonical Copy identity and the explicit
response query boundary. Only the selected exact integer operator executes;
its numeric result is committed by observing the first predicted numeral token
and emitted through the existing causal codec. Prime word addresses, signed H4
products and orientation are active here; fixed-zeta/paired-H4/UOR mechanisms
retain their inherited roles and gain no new semantic qualification from this
experiment. Serving adds no matrix multiplication, floating projection, dense
transformer, provider call or LLM correction.

Offline continuation preserves learned codes by **word identity**, remapping
old prime addresses when the sorted dictionary grows. Landmarks and biases are
reused; new features start at geometric identity. The initialization CID is
artifact-bound. Nothing loads the donor model during serving. The selected
component is 21,758 JSON bytes,205 learned features,27 dictionary words and two
H4 lanes; complete artifact size is in the evidence.

## Preserved failures and scope changes

1. The first diagnostic generated both sums in three worlds. Its initial
   twelve-case report reused original/update final labels; only the first two
   turns are used as evidence from that report. The later full evaluations use
   the correct explicit named-result labels.
2. Reversing individual literal facts failed before selector fitting: `suri`
   and `orin` in reversed order returned4 instead of 17. Reversing complete
   computations with `mira/neri` first also returned9 instead of 14. Both attempts
   stopped before fitting and retain source/error receipts.
3. `independent-reachable-source.json` explicitly separates those exposed
   prefix-failure construction/development rows. Its admitted construction has
   42 prior original/update cases plus 16 reachable independent cases; development
   has 8 reachable cases. The16 name/number/computation-order transfers remain
   unchanged and were evaluated after selecting the continued design.
4. Random initialization with 192 features fits40/58 and generates 8/8 open
   independent cases, but loses18 prior updated-total cases. Retaining all 205
   observed features still fits40/58 with the same18 failures. Dropped query
   words were a real truncation, but not the demonstrated cause of those18
   failures. These are retained negatives, not accepted artifacts.
5. Continuing `43c54db3` by exact word-identity remapping starts at 46/58 and
   finishes 58/58, hinge0; complete construction generation is 58/58 and open
   generation8/8. The exact-code continuation also fits58/58 (hinge6) and gets
   8/8 open and 12/16 transfer. No time cap was hit by these fits.

These are small authored development/after-selection tests, not an independently
sealed general-language benchmark. Prefix failures are included in the16-case
end-to-end denominator. They are not misreported as failures of a selector that
never ran.

## Controls, preservation and cost

Removing the requested intermediate and its Copy aliases gives 0/8 full
operator/provenance successes but **2/8 matching answer texts through
recomputation**. This changes retained ancestry and sometimes selector
eligibility; it is not an isolated angular-feature ablation or universal proof
that the intermediate is necessary.

Preservation checks retain48/48 dependent answers/writes,62/62 earlier cases,
24/24 earlier transfers,28/28 exposed names/writes,28/28 long-context
answers/writes,5/5 persistent turns,6/6 previous numeric transfers and 12/12
previous four-turn transfers. Previously exposed transfer and alias-repair
sets are separately reported. Four unchanged generated Rust functions retain
12 executed semantic assertions; this is preservation, not new generated
arithmetic-code qualification.

Provenance adds256 bytes of fixed mask payload plus bounded temporary arrays;
whole-call stack high-water is not measured. Preprocessing examines at most
16 sources × 16 query words × 4 cues × 32 bytes, plus at most16 ancestry passes
× 16 sources × 16 parent candidates × 2 ID comparisons. Parent-mask union occurs
once per resolved record. Candidate feature extraction remains bounded, with
at most51 features, a 52-entry scratch array and the unchanged ≤256 valid
Copy/Add proposals plus no-operation. It allocates no new growing store.
The actual selected-path allocation and checkpoint checks are listed in the
evidence; allocator results exclude loading, tokenization and session creation.

The 12 successful transferred three-turn trajectories execute36 exact operators
(30 Add), make858 value proposals, and incur24 typed-route predictions,
15,642 source examinations,280,584 comparisons,52,620 code reads,114,414 table
reads and 5,107,614 instrumented logical bytes in the nested typed router.
The evidence also retains the **complete** outer Work sum: for example, the
relation writer still incurs 6,099,912 row comparisons while writing no relation.
These are instrumented logical counters, not a full ISA/physical-memory census
or transformer speed comparison. Detailed Work for the four rejected initial
prefixes is unavailable in the current error-report path; their command wall
and retained storage are fully charged.

## Next implementation

Extend this same learned geometric operand/operator path to **literal-only
first answers**, using exact query/cue binding and existing Copy/Add execution.
The current count-based eligibility leaves those decisions on the older sparse
numeric selector. Start with the exposed reversed-fact and `kira/fenn` failures,
retain correct intermediate generation and all accepted role behavior, then
check unseen names/order on complete trajectories. Reuse the continued role
parameters rather than restarting their learning. Do not add another store,
depth-specific head or broader corpus campaign. Once literal binding passes,
carry the demonstrated selection into generated Rust and execute its semantics.

## Verification and resource receipt

The changed core/probe and final `r4` CLI compile offline with the pinned Rust
1.97.1 toolchain. Six focused typed-state checks pass, including exact word-code
remapping, numeric independence, Copy identity, source permutation and missing
ancestry; four selected-execution checks pass (one additional artifact-dependent
test in that filter is ignored). The kernel source check passes. Actual new
three-turn and prior four-turn allocation tests both measure **zero allocations
and zero allocated bytes**, verify generated output, and pass checkpoint/flag
or boundary rejection checks. The CLI returns Zurich after the location update.
Formatting, architecture policy and claim wording pass. Broad release QA remains
`NOT_RUN` and protected CI status must be read separately from these local checks.

A missing test import was corrected after a failed compile. The first allocation
assertion incorrectly required token IDs from lexical encoding to equal the
valid byte-token numeral sequence. The corrected test checks decoded bytes and
EOS outside the allocation measurement; serving was unchanged. Both failures
remain charged and recorded.

This cycle charges 287.845 seconds of model work and approximately 14 minutes of
monitored engineering. Model/engineering point projections were240/600 seconds;
the configured360/900-second cycle ceilings were respected. The cumulative model
ledger is **2604.356/2730 seconds**, leaving 125.644 seconds. The owner-authorized
extension was360 seconds of cumulative model allowance and 192 MiB of storage.
Peak sampled known storage is 6,484,525,056 bytes under the effective
6,707,802,112-byte stop ceiling; peak sampled model child RSS is 881,786,880 bytes
under 4 GiB. The evidence gives exact final engineering totals and storage samples.
No user/research material was deleted and no external model compute was used.
