# Retained-evidence preparation contract — #1094

**2026-09-03 — `PREPARATION_CONTRACT_FROZEN`; execution release `NOT_ADMITTED`.**
This is the separately requested preparation/release decision after
[#1096 readiness](r4_isolated_runtime_readiness_1096.md), protected merge
`6f21fc5f4c40b9620c9fec5e95a39097f812ae73`. It freezes a concrete resumption
protocol and its accounting, not a successful invocation of `prepare` or a
model result. No preparation worker, model, fit, withheld access, comparison or
replay ran in this step. #1094 remains open.

The [machine-readable contract](r4_text_clause_preparation_1094_contract.json)
pins the original source/evidence handoff. The [budget audit](r4_text_clause_preparation_1094_budget_audit.md)
and [source audit](r4_text_clause_preparation_1094_source_audit.md) independently
identify why the current executable cannot yet admit resumption. The
[release review](r4_text_clause_preparation_1094_review.md) records contract
acceptance separately from permission to execute.

## Preserved result and selected decision

The original [#1094 preparation](r4_text_clause_adapter_1094.md) remains
`UNAVAILABLE_REFERENCE_REPLAY`: authoring inputs were 320/320 exact, refusals
16/16 exact, and both schema probes exact; the OS denied Python startup.
#1096 later measured one isolated runtime startup with all four harmless
probes denied and zero model loads/forwards/updates. Neither establishes
raw-text model-output preservation. #1079 `LANGUAGE_R4_PRESERVED_CONTROL_WEAK`,
#1082's descriptive result, #973 open and #954 blocked remain unchanged.

**Definition — chosen resumption.** Preserve and reuse the independently
accepted authoring and runtime evidence. Charge the entire original preparation
allocation of 120 seconds as unavailable for further preparation. Implement a
separate retained-evidence assembly that verifies this handoff and emits an
explicit provenance status, `PREPARATION_ASSEMBLED_FROM_RETAINED_EVIDENCE`.
That status must never claim that a new `campaign.prepare()` succeeded.
The current code does not implement or accept this status.

**Empirical Criterion — unchanged future comparison.** The #1085 comparison
still owns 100% acceptance/input/span fidelity, byte-identical complete soft
outputs/logits/decisions, fourteen consumed role decisions per valid row,
exact refusals with zero forwards, and exact fresh-process replay. Keep the
same reader/core, known vocabulary/query, four facts, 1600 valid rows, 80 refusal
rows and 16 boundary controls. These requirements are not evaluated here.

## Budget that cannot be reset

The original final resource receipt contains two slightly different clock
reads: preparation `0.2602927920015645` seconds and cumulative
`0.2602929580025375` seconds. Their arithmetic remainders are upper bounds
because final receipt serialization/write and exit followed those reads.
No unmeasured tail is silently called zero or proved smaller than a rounding
allowance. The full 120-second allocation is reserved conservatively; it is a
policy debit, **not a measurement that the stopped attempt lasted 120 seconds**.

| Ledger | Frozen treatment |
|---|---|
| Original #1094 preparation | Historical `UNAVAILABLE_REFERENCE_REPLAY`; 120-second allocation fully charged; no new preparation invocation |
| Future execution | At most 120 seconds; includes all fresh identity/runtime/release checks, first withheld hash/read and both sequential arms |
| Future fresh-process replay | At most 120 seconds; includes its fresh identity/runtime checks and both sequential arms |
| Campaign cumulative | 120 + actual execution + actual replay, never over 360 seconds |
| Peak RSS | At most 3 GiB combined coordinator/worker bound; peaks are not added across historical runs |
| Logical forwards | At most 6400 total; historical consumption zero; each valid row once per arm per execution/replay |
| Corpus/results bytes | 134,217,728-byte ceiling; original retained ledger is 3,465,401 bytes; at most 130,752,327 bytes remain before new receipts |
| #1096 readiness | Separate consumed one-attempt/60-second/16-MiB ledger; does not replenish or silently debit #1094 |
| Independent input authoring and code review | Separate authoring ledger as #1085 specifies; not model work |

The exact byte sum is 3,397,265 frozen corpus/selection/policy bytes + 1,565
population-manifest bytes + 48 harmless-probe bytes + 66,523 bytes in seven
original comparison files. It includes the 279-byte final resource receipt.
Do not count the same corpus twice when carrying old output into a new folder.
Count every newly retained campaign receipt and temporary oracle stream while
it exists. A later footprint inventory must also identify delivery copies and
other explicitly retained task material; it cannot use a fresh folder to erase
history. The separate #1096 receipts remain preserved under their own ledger.

## Required retained-evidence assembly

The future coordinator change is bounded to accounting, evidence assembly and
launch/identity plumbing. Adapter, worker, curator, policy, learned reader/core,
frames, vocabulary, arithmetic and comparison criteria stay byte-identical to
the accepted handoff. A change to any of these invalidates reuse and requires
a separately reviewed scientific decision before execution.

Before any execution admission, create one exclusive assembly/output directory
outside the sealed corpus tree, so receipt accounting cannot traverse withheld
inputs, and an envelope with schema
`uor-r4.text-clause-retained-preparation/1` containing:

1. Exact SHA256/byte-length references to the old start, unavailable stop,
   final resources and authoring preflight; the original population/selection
   commitments; the #1085 specification; and #1096's manifest, binding,
   profile, result, start, approval and independent result review.
2. The distinct assembled status above, original terminal unchanged, explicit
   reused observations (not rerun counts), 120-second historical debit, zero
   historical forwards, exact prior-output/corpus byte components, and the
   unchanged numerical comparison/resource criteria.
3. The committed coordinator source closure and the unchanged worker/adapter/
   curator/model source identities. Bind both paths if coordinator and worker
   use different source roots. Verify executable source against committed bytes;
   a Git hash or old graph label alone is insufficient.
4. Absolute source/output/binding/profile/probe paths; the exact generated
   profile bytes and enumerated delta from #1096. New source/output literals
   are a rebind, not the previously measured configuration. Removing #1096's
   manifest/profile read literals is narrowing and must be explicit. No new
   runtime/home tree, corpus/reference/history/result content permission,
   network permission or model write permission is admitted.
5. The same interpreter launcher/resolved path, both symlinks, eighteen runtime
   file identities, fixed Python 3.12.14/Torch 2.7.1 CPU/Accelerate settings,
   hardware and five accepted asset identities. The coordinator verifies the
   runtime files and symlinks before and after each future worker; the worker
   performs its existing source/asset/runtime and harmless-denial checks.
6. An exact clean child environment: fixed system PATH, HOME, candidate
   PYTHONPATH, PYTHONNOUSERSITE, no bytecode, unbuffered output, four OMP/Accelerate
   threads and the harmless isolation-probe path. Do not inherit arbitrary
   `PYTHONHOME`, Python/dynamic-loader overrides or other parent environment.
7. The original authoring/withheld commitments and sealed-directory metadata;
   no withheld payload hash, read, traversal, copy, regeneration or filtering.
   During assembly verify committed metadata and original evidence only.
   Fresh execution identity checks consume execution time before release.

This assembly is code/provenance work, not another empirical preparation phase.
It may not import/execute the adapter, rerun authoring rows, launch the worker,
construct a model or use the closed #1096 attempt directory as a new attempt.
It must not manufacture the legacy `COMPARISON_PREPARED` report.

## Release gate and current executable blockers

At source `6f21fc5f4c40b9620c9fec5e95a39097f812ae73`:

- `prepare()` creates a fresh zero-carry budget, regenerates bindings/profile,
  reruns authoring preflight and launches readiness. It cannot implement the
  selected retained-evidence assembly.
- `Budget.check()` uses only fresh elapsed time for the 120-second phase limit;
  `main()` arms a new 120-second timer. Original output bytes are omitted by a
  fresh output directory. History must be carried explicitly and checked.
- `run()` carries only the new preparation's phase time and recognizes only
  `COMPARISON_PREPARED`. It must validate the new envelope and carry 120 seconds
  through execution, replay, resource snapshots and the final result.
- `arm_process()` inherits the parent environment and does not pass #1096's
  readiness manifest/hash. The existing worker exposes one harmless-probe
  receipt, not the four-probe/18-file `readiness_identity` event. Preserve this
  distinction; parent runtime checks and exact profile linkage must be explicit.
- `run()` performs both withheld execution and fresh-process replay in one call.
  No command invoking it is admitted by this preparation-contract review.

An independent reviewer must verify the implemented assembly/launcher/accounting
and bind the exact assembled-envelope digest, source closure, profile delta,
original selection/population commitments, runtime identities and 120-second
carry before writing a distinct release receipt with schema
`uor-r4.text-clause-retained-release/1` and status
`ACCEPTED_FOR_RETAINED_EVIDENCE_COMPARISON`. The old release status alone cannot
approve this new protocol. No such release receipt exists in this delivery.

The future execution start receipt must be written before the first withheld
hash/read and bind that approval, assembly and carry. Global timers and live
resource checks cover identity checks, launch, both arms and evidence writes;
replay uses fresh worker processes without resetting the cumulative debit.
Preserve partial streams/counts on failure and let any stopped receipt override
completion. No retry, tolerance relaxation, input repair or budget renewal
follows a revealed miss or resource stop. Original #1085 terminal actions remain
unchanged.

## Source basis, review and next action

The original NEMESIS carrying report (Mark / NEMESIS 3D Studio, pp. 1–2), W33
`validate_instruction`/canonical JSON/content store, and UOR `check_axis`/G2
canonicalization were reread from the pinned local originals cited in the
[#1085 source audit](integration/clause-segmentation-1085-sources.md); their
SHA256 values still match. They motivate explicit mapping/admission, immutable
receipts and distinct digest axes. UOR's commutative G2 encoding cannot replace
ordered clause/role framing. None proves parsing, runtime isolation, budget
completeness, learned capacity or this proposed assembly mechanism. No source
was vendored and no external witness or proof tool was executed.

Named checks are independent budget/source/contract review, JSON/source/evidence
hash and reference integrity, claim wording and diff whitespace. Existing
source bytes and original receipts are verified without reading the sealed
population. Broad QA, model work and population preparation remain `NOT_RUN`.
Protected PR acknowledgements are transport only.

Deliver this contract/review and current roadmap/claim/index changes through the
protected path. #1094's comparison DoD remains open. The one next action is to
**implement the retained-evidence assembly and its carry-aware launch gate**
inside #1094, then obtain independent approval of the exact implemented envelope
before any withheld execution. No other issue is activated by this step.
