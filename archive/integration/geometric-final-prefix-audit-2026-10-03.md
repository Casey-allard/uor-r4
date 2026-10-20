# Final geometric source artifact: Copy-ranking diagnosis, October 3

The project already has Rust autodiff learning, learned geometric context/capture,
and saved native operators. This diagnostic reuses the existing trained64-update
source realizer. It builds no replacement learner and performs zero optimizer
updates. Its purpose is to locate the failure between retained geometric context,
Copy ranking and complete emitted answers, in the grounded-conversation/native
attention track.

## Actual execution and retained identities

The owner authorized CPU work on the existing Claude Runpod. Committed source
`67d3691566bea615b68a26e5d6927ec5f3477ce3` was shipped as a clean git archive.
Host: Linux x86_64, rustc1.97.1, default training features; no Metal/CUDA or GPU.
All writes remain under `/root/codex`. Affinity confines workers to48 available
CPUs; build jobs/Rayon are48. The model, tokenizer, source/native loader and audit
are Rust. Python supervises process/resource limits only; it does not construct
model tensors/state, train, tokenize or generate.

Three focused Rust tests pass: alias mass aggregation, absent target and native
smallest-token-ID ties. Compile/test worker397.863s; sampled process-tree peak
RSS5,603,557,376B. The example build reports9.19s; after a monitor race its exact
exit was unavailable, so the unchanged cached build confirms exit0 in2.187s.
The successful audit worker takes6.235s (report4.386s), max child RSS896,438,272B,
under300s/3GiB caps. These are concurrent-pod CPU costs, not M1 performance or
energy comparisons. Metal-only execution is unavailable and not counted.

The audit loads the actual final source/native checkpoint from the prior fit,
verifies complete input seals before and after, checks parent bytes for identity
without loading/scoring the parent model, and leaves all five packed files
unchanged. The loader reconstructs validation tables offline; this remains a
research loader, not complete product-serving qualification.

Executable SHA256: `636895809b7313373ce89947cc4c2257323dd1e8ab7e7b33df8ff9ca41af0d3f`.
Report SHA256: `9db810a31dbf9fc34189d7d9b0279da5780b17e2aa2c8cd1fce56fcdf29d0f46`.
[Bound evidence](../evidence/geometric-final-prefix-audit-2026-10-03.json).
Local retained root:
`~/uor-r4-local/workspace/research/geometric-final-prefix-audit-20261003/pod-return/audit-2`.
41 returned files, including the executable and both attempt roots, match their
remote SHA256 transfer inventory. This is transfer verification; the Rust seal
checks were executed on the pod.

## Final checkpoint behavior

All20 own-prefix rows, including every integer state/score/mass trace, reproduce
the saved M1 final rows exactly. Complete answers remain4/20, all four repeating
`dancer`; EOS remains20/20. This is component replay across hosts, not a new fit,
heldout result, complete-chat acceptance or bitwise training reproducibility.

The same frozen artifact evaluates all114 canonical next-token positions,
including EOS. These are final-model traces; they replace the incomparable
pre-update64 training snapshots for this diagnosis.

| Stage | Correct native greedy choice | Targets |
|---|---:|---:|
| Initial Copy | 10 | 20 |
| Later Copy | 8 | 54 |
| Period | 18 | 20 |
| Stop/EOS | 10 | 20 |
| Total | 46 | 114 |

Native-mass mean episode NLL is1.3407132581; token-weighted NLL is1.5175203841.
They use different weighting and are exposed diagnostic losses. Even the correct
prefix leaves68 incorrect next-token choices, so generated-history drift alone
does not explain this result.

Of74 Copy targets,54 lose to another Copy action, including10 initial and44
continuation choices. Only two continuation targets have the correct top Copy
but lose to Period/Stop. With these Copy scores fixed, changing stopping alone
cannot repair those54 decisions.

Independent mathematical review recomputes114 ranks/probabilities, verifies4,016
lane features and1,184 within-row occurrence pairs, and checks all packed-file
hashes and entire cross-host generation rows. There are five distinct numerical
inputs and31 distinct complete scorer signatures. No conflicting canonical
labels share a full feature signature, Q24 score vector or Q31 mass vector.
No different-token occurrences have identical active Copy features within a
row, and no token aliases occur in this panel. Four equal-Q24 Copy pairs at
singer's canonical EOS step have different features; they are not structural
collision witnesses.

This rules out those specific observed collision explanations. Distinct features
do not prove that the bounded additive/q4 scorer can realize all rankings, that
its recurrence has learned useful progress, or that its biased adjoint is sound.
The geometric family is not retired by this result.

## Next targeted repair of existing learning

Reuse the saved final checkpoint, current prepared-loss/parameter APIs and q4
potential. Keep context, Period, Stop, source/query/tokenizer and action vocabulary
fixed. Inspect a small declared set of legal one-quantum potential changes and
compare exact native Copy-margin and joint-CE effects with the existing
readout-only adjoint. Retain every row regression. This is a proposed bounded
direction diagnostic, not a new learner or another unrestricted joint fit.

Improving native directions justify focused existing-readout learning; opposite
predicted/actual directions justify examining that credit path. Failure to find a sampled
one-step improvement establishes neither family impossibility nor a need for new
progress state. Diagnostic candidates do not silently become adopted models.
Shared `stack_grounded_session.rs` remains untouched and coordinated on#1552.
The alpha goal remains useful grounded conversation/memory through the same
native geometry path; general prose, NotFound and session integration remain open.

## Execution negatives and complete cost

The missing GNU time tool stopped the first supervisor before Cargo. A transient
Cargo journal made the first example supervisor fail; its log and cached exit
confirmation are retained. Audit-1 fails closed before model load because Mac
tar adds219 AppleDouble sidecars absent from the input seals. Only files with
checked AppleDouble magic/version are removed from Codex's copied inputs;
original local roots and payload/seal bytes remain untouched. Fresh audit-2
passes. These are setup/transport negatives, not model-quality negatives.

Complete new-task preparation/build/errors/reviews/audit/transfer/delivery and
remaining cleanup are charged as a conservative60-minute estimate, separately
from measured workers and previous source-fit charges. The cumulative ledger
moves1,051,635,028→1,055,235,028ms under the unchanged1,130,000,000ms limit.

After reconciling fresh main, committed head
`ebd0a891d19ed23461c980157f6add0dec9140ae` passes the locked training-example
Cargo check on the pod (exit0,46.949s, sampled process-tree peak
RSS2,454,286,336B). The numerical consumer/context/realizer sources and audit
example are unchanged from the executed audit source; this check validates
integration and is not another model evaluation. The Codex job card is released.
Final `/root/codex` allocation is4,287,792KiB (about4.09GiB), below the10GiB
limit. All results and execution receipts have been brought back locally.
