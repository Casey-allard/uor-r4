# #1102 independent retained-result review

**Empirical Criterion — disposition:**
`ACCEPTED_BOUNDED_NATIVE_REFERENCE_QUALIFICATION` for the one completed
`NATIVE_REFERENCE_PRESERVED` attempt under the unchanged #1086 contract.
This review accepts the measured result for the exact artifact, binary,
runtime and original authoring stratum below. It is not a merge approval or
authority for another run.

The independent reviewer inspected retained evidence after the coordinator
finished. The standalone audit imports only Python standard-library modules.
It does not import project/model code, instantiate a model, execute a forward,
load a native engine, export an artifact, fit parameters, rebuild, rerun the
comparison, or access the withheld population. The two original authoring files
were read solely as evaluator annotations. Retained parameter bytes were hashed
as opaque container components; the reviewer decoded only container metadata,
codec/frame tables and the already retained output tensors.

## Exact binding and retained audit

| Binding | SHA-256 |
|---|---|
| Frozen comparison release | `2c3c2f73eb6cf804eb69b2afb0f979ae623a512ca0492e47df2af70d6cbaca8b` |
| Independent pre-run admission | `88c8f8b4223ab83cca072b263c6a4b2febe542173c040fbf9f73bbc6143f4647` |
| Native binary | `d423d8d3c3acd2d1c6215c21206e1bec7583e4dd37e84f30f70f79e77c40d53f` |
| Runtime receipt | `daba1ad4bf60d28def983378a6a856e0990d7eab20d2ec2365552ad07f3d83d2` |
| Both exported artifact copies | `2c209590a64cae16a4140fd43adc1cb1f87b357c02e3d4959f1e37f4ab8cd5ab` |
| Recomputed native state | `4f453da12a9346356e64b6c16abfbaad1ca99e3966173cd79e9ddbc8c2d9341b` |
| Coordinator result | `ae9c6fce9f50cab67c94ae9695c28ebfd735069b6d42833a2ad73666ab7e8263` |
| Independent machine audit | `5e75c5a3407d91bf14a6ec8d57981a72cfe110cdaf1ddfd0c71f6ea0a07974e8` |
| Standalone review script | `1de3b1ca4bd7eb54b66aa334e3c9503bce442d41d562c97fbe515624804ffc27` |

Source revision is `1c6be403a9de732dcbb1d9aad8fda66c2b18579c`;
contract SHA-256 remains
`e3565728dfa872d9c260dd3d391ae59e0b0d454ebead20cd15cb075b24796115`.
The 221,346-byte machine audit and 23,448-byte script are retained separately at
`/Users/casey.allard/.codex/uor/issue-1102-result-review/audit.json` and
`/Users/casey.allard/.codex/uor/issue-1102-result-review/audit_retained_result.py`.
The audit contains every independently recomputed per-row tensor maximum and
location, gate records, arm summaries and identities for all 92 sealed run files.

## Independently reconciled result

All 236 launch-file identities still match their admission records. The
reference binding object remains exactly equal to the original accepted
binding, including all 169 historical source records. Actual authoring file
lengths and digests match the frozen release; all 336 row IDs are unique, remain
in their original order and belong to the authoring partition. The retained
operator command and environment equal the independently admitted launch.

For each of the four arm/phase streams—initial reference, initial native,
fresh reference replay and fresh native replay—the audit independently found:

- **320/320 correct answers**, with lowest-index full-4096 argmax recomputed
  from retained logits and token spelling checked against the retained codec.
- **4,480/4,480 correct consumed role pointers**; all fifteen reported role
  argmaxes were independently recomputed from the complete role-attention tensor.
- **16/16 correct refusals**, with zero forwards, no parsed input or model
  diagnostics, and the required refusal fields/offsets. All refusal objects
  also match exactly between arms and across replay.
- Exact annotation agreement for inputs, lengths, token/clause spans and raw
  and derived input identities. Derived-input hashes were recomputed from the
  normative integer framing. Token and clause frame indices were independently
  recomputed from the retained multiplication and token-leaf tables.
- All tensor entries finite, with **3,840 padding probabilities per stream**
  represented by exact positive-zero bytes.

The audit read and reconciled all **26,910,720 retained output tensor bytes**
(6,727,680 f32 values). Every row-level maximum and first maximizing index
matches the coordinator's saved comparison record. Both phases give the same
aggregate peaks:

| Tensor | Maximum absolute difference | First row | Tensor coordinate |
|---|---:|---:|---|
| `role_attention[1,5,3,13]` | `3.5762786865234375e-7` | 240 | `[0,4,1,3]` |
| `role_vectors[1,5,3,64]` | `8.940696716308594e-8` | 12 | `[0,0,2,21]` |
| `binding_attention[1,5]` | `2.980232238769531e-7` | 132 | `[0,4]` |
| `logits[1,4096]` | `4.76837158203125e-6` | 140 | `[0,49]` |

Every entry satisfies the frozen absolute ceiling `1e-5`, with relative
tolerance zero. Cross-runtime tensor bytes differ. Within each arm, the entire
initial and fresh-replay JSONL files and tensor files are byte-identical;
completion records also match after excluding phase names and resource timing.
Reference parameter-state identities remain unchanged, and native state
identity and FPCR fields remain unchanged through both phases.

The two 2,172,252-byte exports are byte-identical. The audit independently
checked container framing, canonical manifest, all 21 component descriptors and
hashes, accepted asset/state/source provenance and the native-state hash recipe.
The tied-head declaration is the exact frozen embedding alias; the prior
independent source review established the implementation alias. This review
does not claim a new engine-level alias experiment.

All **eleven independently frozen mutation errors** match the recorded loader
error object and reached-stage sequence. The twelfth, valid loader call succeeds
and refuses ordinary inference without a qualification receipt. Loader gates
perform zero forwards; rejected attempts reach **seven partial reader/core
state validations**, below the frozen 22-state allowance. The full campaign has
**five successful engine loads, ten successful model-state loads, 1,280 logical
forwards, zero refusal forwards and zero parameter updates**. No reference,
discrete, refusal, tensor or replay errors were found.

## Resource evidence and limits

The successful external supervisor receipt and final wall receipt bind the
same release and coordinator result. No stop receipt exists. Recorded phase
times are 0.817787792 seconds for export/integrity, 4.209746583 seconds for
execution and 3.765631000 seconds for replay; the final wall receipt extends
the active replay tail to 3.782249917 seconds. Total recorded wall time through
completion is **8.809783917 seconds**, peak combined sampled RSS is
**602,390,528 bytes**, and the complete ledger is **75,039,076 bytes**.
The reviewer independently summed the 92 retained run files plus the two
original authoring files and obtained that exact ledger total. All are within
the unchanged 120-second phase, 360-second cumulative, 3-GiB RSS and 128-MiB
ledger limits. These values describe this one campaign, not a throughput or
comparative performance benchmark.

The result is measured preservation of the known four-fact, known-vocabulary,
one-query authoring stratum at B=1 under the exact scalar f32/f64 native profile
and pinned Python/Torch reference. It is not an independent semantic holdout,
universal mathematical or floating-point equivalence proof, portability result,
general parsing/context/generation/reasoning/coding result, geometry-superiority
result, or qualification of the final integer kernel. The original unavailable
preparation observations, Python qualification, descriptive findings and prior
negative results remain part of the evidence history.

All sealed run evidence remains unchanged. No user material, model asset,
original authoring record or unique output was deleted. This review authorizes
delivery of the bounded result and construction of the exact trusted
qualification metadata; it authorizes no additional model work.
