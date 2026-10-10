# Cross-state continuation improves saved complete replies

**KEEP: 8/512 → 22/512 complete replies**, with all original eight retained and
14 new successes. The saved field is loaded by the native generator and judged
under its own emitted prefixes on the unchanged frozen panel. M2 remains in
progress: its acceptance is at least 256/512, followed by at least 40% on a fresh
draw whose criteria are frozen before that draw.

**Line: cross-state continuation · count 0/3 · headline 8/512 → 22/512.**
The inherited no-headline count was 4/3 before this run. The measured gain resets
it; it does not reopen the closed ordinary 24/96 recipe or the protected
constructor/attribution/solver line. No preparation PR was used.

## Question and fixed mechanism

The [pre-registration](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6096164858)
and [exact text](preregister.md) precede source execution. The earlier unary
continuation field reached 511 correct entry tokens but only one complete reply;
the ordinary joint Context–prototype run reached zero complete replies.
Those results motivate testing joint dependence on factual and local-prefix
state. They do not prove that additive separation caused the failures.

For token t and lane l, the new field contributes

`delta(t) = 2^22 * sum_l E_l[inverse(factual_post_l)*prototype_tl, inverse(local_prefix_l)*prototype_tl]`.

The factual state is the actual native post-bridge state. The independent local
state replays query plus prior reply prefix from identity. Both relatives use
directed signed full 120 H4 codes. The same token delta reaches Generate and
every physical Copy alias before the existing common clip and alias pooling.
No row ID, current/future target, selected reference, answer length or response
template enters the scorer.

Only 115,200 Q4 coefficients are trained: 8 lanes ×120 ×120. Their padded
128-stride native payload is 65,536 bytes, with invalid cells zero. Native scoring
uses inverses, product tables, indexed coefficient reads, shifts and additions;
the field scorer adds no heap allocation or floating arithmetic. This is a
bounded additional field in the existing native model, not a full-path energy
or allocation qualification. Existing dense per-token vocabulary scoring remains.

The exact accepted Source48/Generate64 parent, Context, Potential, Generate
prototypes/coefficients, bridge, Cue, Prefix, exponent table and tokenizer are
frozen. Schema3 has explicit ordered-state/policy/binding validation. V1/v2
artifacts omit the optional cross-state payload and retain their previous
serialization and scoring policies.

The [configuration](model-config.json) fixes one shuffled pass through all 512
exposed examples: seed 1001, 64 updates, batch 8, 6,664 answer/EOS positions. Fresh
AdamW uses lr 0.03, betas 0.9/0.999, epsilon 1e-8, no weight decay, global norm clip 1
and quarter-master clamp [-1.75,1.75]. Checkpoint 32 is recovery-only; 64 is the sole
endpoint. Frozen native position preparation is on CPU; coefficient gathers and
backwards use CUDA, with the existing CPU alias reducer and transfers charged.

Zero-field evaluation must reproduce the exact original eight before fitting.
The saved/reloaded endpoint receives an early original-eight check, then all 512
rows regardless of loss. The prospective developmental KEEP bar is net complete
count>8; original-eight retention is reported rather than a veto. Here all eight
are retained anyway. Earlier candidates remain rejected under their original bars.

## Measured result

| Measure | Accepted parent | Saved update 64 |
| --- | ---: | ---: |
| Complete replies | 8/512 | **22/512** |
| Correct entry token | 8/512 | 222/512 |
| EOS-ending outputs | 11/512 | 26/512 |
| Complete source-swap pairs | 4 | 5 |
| Teacher-prefix all-token correctness | 8/512 | 22/512 |
| Teacher-prefix correct tokens | 398/6,664 | 3,317/6,664 |
| Equal-episode teacher-prefix CE | 6.1992635176344875 | 4.031771212264852 |

| Memory stratum | Cases | Before | After |
| --- | ---: | ---: | ---: |
| length2 | 128 | 8 | 14 |
| length4 | 128 | 0 | 5 |
| length8 | 128 | 0 | 1 |
| reassert | 64 | 0 | 1 |
| update | 64 | 0 | 1 |

The [independent saved-row review](reviewer-cross.json) lists all 14 gained IDs,
eight retained IDs and zero lost IDs. The early original-eight outputs match
their full-panel counterparts. Native payload inspection finds 107,611 changed,
nonzero legal coefficients out of 115,200, final code range [-7,2], with padding
valid. The first active gradient norm is 0.3889743616347245; the only active
parameter family is `continuation.cross_state`.

Exact saved-output comparisons authenticate all five earlier candidates:

| Prior candidate | Complete before → now | Prior successes lost | Changed outputs |
| --- | ---: | ---: | ---: |
| #2141 | 2 → 22 | 0 | 510 |
| #2143 | 2 → 22 | 0 | 510 |
| #2148 | 4 → 22 | 0 | 508 |
| #2149 | 0 → 22 | 0 | 512 |
| U64 at c64cce4 | 1 → 22 | 0 | 511 |

## Decision and limits

Retain the saved checkpoint 64 and activate the reviewed schema3 native/training
path. The next cycle starts from this measured gain and must be registered and
scored on the same complete-reply panel. Finish protected delivery and cleanup
before that successor. The standing session goal continues; this result does
not meet M2 or change M3.

**Before opening the PR: if this came out the other way, would my next step
differ? Yes.** A flat/regressed complete count would close and archive this
intervention without another dose/rate/seed series. The observed gain instead
retains the model and supports a registered learning continuation.

All 512 examples are exposed development data, not independent held-out trials.
490 replies still fail. No fresh-draw qualification, general conversation,
unique causal necessity of the factual carrier, geometric advantage or
serving-energy improvement is established. The earlier conditional 9/15 gate
belongs to a separate historical artifact and was not rerun here. This result
does not prove that512/512 is attainable or unattainable.

## Verification, identity and cost

The accepted endpoint field SHA-256 is
f61249ae4032a92d452689a52add6c9b5a67099f55b1a9291ba6f6f66a0d9acf.
[Artifact receipts](artifact-receipts.json) bind it to the unchanged upstream
parent and independently reloaded masters.

[Producer receipt](producer.json) binds source
920e4201770e282975eb0340c83f3fa296ca08e7 and executable SHA-256
c41e9f9597c0bb46c041d5f79744b32638aee298db508b3a0d260ce393ae7de4.
Independent source review passed before model compute. Actual scoped checks:
9 native field tests,17 training tests,4 explicitly executed CUDA parity tests,
3 fitter admission/outcome tests; all passed. Scoped formatting/diff checks
passed. The initial offline core-test invocation lacked a locked dependency;
locked fetch repaired it and all 9 tests then ran. An initial source-pinning
launch flattened a multiline shell command and exited before model execution;
the script-based retry succeeded. Both setup failures are retained.

Stock pod bootstrap took 196 s (132 s build, 47 s parity; 37 passed, 3 ignored) with
its stock 15 build jobs. Changed builds use 4 jobs; model execution uses 2 threads.
Source and artifacts stay in the owned canonical-volume workspace because the
laptop had 21 GiB free, below its 30 GiB floor. No local source/build/archive staging
or new spending class was used.

The run started 2026-10-10T09:57:50Z and exited 0 after 433.25941270194016 s:
47.615412459 s native cache preparation,46.813096302 s fit excluding checkpoints,
0.985048590 s checkpoint work and 333.282966895 s evaluation. Process peak child
RSS is 776,408 KiB; sampled GPU memory peaks at 838 MiB. These are measured process
costs, not complete-cycle cost or serving efficiency. The one 5090 pod costs
$1.19/h; complete preparation/build/review/preservation/delivery and failed
setup time are included in the cumulative cycle ledger, closed after pod
deletion. The registered complete-cycle estimate is 180 minutes/$3.57.

The producer seals and verifies the completed report file set. Independent
[reader](reviewer-cross.py) checks all 1,024 endpoint row hashes/IDs, actual-prefix
feedback/chosen-token chains, EOS, frozen exact membership, schedule, frozen
parent bindings, native Q4 crossings and five comparison artifacts. It does not
regenerate tokenizer decoding, native scores, gradients or the complete BLAKE3
seal. Its sole layout repair follows the external configuration path recorded
in sealed `attempt.json`; that config is hashed and crosschecked against
sealed admission/report. Producer evidence was not edited.

Report SHA-256: 738649541df34641ffe0eb6e45a151d612d485ff8902b2d9d8d1e812844a2cde.  
Manifest SHA-256: e5623d996b60308d8ca1080f748c062388ff74c6b14b16f9956b4b792a9ca8c2.  
[Measurement summary](measurement-summary.json), [independent review](independent-review.md)
and [preservation package](PACKAGE.json) and [verified iCloud receipt](preservation.json)
retain the evidence and restoration
identities. Final exact-head checks and protected merge verification are posted
on the result PR; final accounting/cleanup are posted on #2030.

Restore through `cloud-store fetch codex-m2-cross-state-20261010 <absolute-owned-destination>`.
The canonical package descriptor clarifies the original embedded restore prose;
the verified archive bytes and component identities are unchanged.
