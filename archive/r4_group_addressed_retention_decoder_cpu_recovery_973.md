# #973 R4 retained-decoder CPU recovery

Status: **SCIENTIFIC TERMINAL / RETAINED-DECODER FAIL / STATE ABLATION POSITIVE**

Issue: [#973](https://github.com/UOR-Foundation/uor-r4/issues/973)

Authoritative freeze:
[issue comment 5490260940](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5490260940)

Policy: `R4GroupAddressedRetentionDecoderV1CpuRecovery`

## Decision and predecessor

The create-once MPS attempt is preserved in the
[V1 terminal record](r4_group_addressed_retention_decoder_973.md). It stopped
`UNAVAILABLE_FULLER_DECODER_CONSTRUCTION` before optimization because its
deterministic-MPS timing projection exceeded an arbitrary 600-second process
ceiling. That result did not evaluate the decoder, retained state, or H4.

This resource-only successor asks the already frozen scientific question. It
changes no model, data, initialization, optimizer, dose, intervention,
threshold, or causal control. Its only changed variable is the locally measured
execution plan.

## Unchanged scientific contract

The successor inherits all frozen identities and equations from the V1 record:

- exact predecessor training-view, population, fit-store, fit-index, tokenizer,
  and geometry CIDs;
- train ordinals `8..39`, validation ordinals `40..71`, the first 129 tokens
  per story, 32 stories and 4,096 causal decisions per partition;
- zero model-heldout reads and no heldout path;
- the 3,171,760-parameter, vocabulary-4,096, width-288, two-block decoder;
- four 72-wide heads, 120 group addresses, separate key/value state, exact
  transport, decay, read-before-write stable softmax, residual SwiGLU path,
  and genuinely tied embedding/output storage;
- seed `9737` and byte-identical learned initialization for exact H4 and the
  independently scrambled-H4 control;
- AdamW at `0.003`, betas `0.9/0.95`, epsilon `1e-8`, zero weight decay,
  gradient clip `1`, batch `8`, context `128`, deterministic cyclic order,
  and exactly 256 optimizer steps per arm;
- sequential exact-H4 and scrambled-H4 arms, with C120 mechanical-only; and
- the identical retained-decoder and nested H4 pass criteria, including
  state-off, replay, causality, finite-gradient, and equal-work requirements.

The construction dose remains 262,144 token presentations per arm and 524,288
total. A result can therefore be compared directly with the frozen V1
scientific criteria; there was no V1 optimization result to tune against.

## Frozen execution plan

The execution host must be Darwin with the installed PyTorch build reporting
Apple Accelerate BLAS. Before model work it must establish and record:

```text
torch.use_deterministic_algorithms(True)
torch.set_num_threads(4)
torch.set_num_interop_threads(4)
OMP_NUM_THREADS=4
VECLIB_MAXIMUM_THREADS=4
OPENBLAS_NUM_THREADS=4
```

Execution uses the CPU only, exactly one process, and one arm at a time. MPS,
CUDA, multiprocessing, concurrent arm workers, and backend fallback are
forbidden. Construction tensors remain resident on the selected CPU device.

This choice is empirical rather than ideological. On the same M1 and exact
deterministic step, Apple CPU/Accelerate with four configured PyTorch/Accelerate threads measured
about `0.820 s/step`, eight threads about `0.944 s/step`, and deterministic MPS
about `1.258 s/step`. Two concurrent two-thread workers degraded to about
`2.35 s/step` per worker. The four-thread sequential plan was the fastest
measured strategy.

One warm-up and three measured steps per trained arm are retained as telemetry.
Timing is not an admission gate. The whole process has a binding 900-second
hard wall; crossing it yields `UNAVAILABLE` with completed-step evidence and is
not a scientific negative.

## Terminal interpretation

If the full fit completes within the resource envelope, apply the unchanged V1
scientific criteria. A retained-decoder pass and nested H4 pass have their
original bounded meanings. Under the inherited pre-run action branch, a
retained-decoder scientific miss retires the exact two-layer update/read law
from promotion or scaling. A host, thread, BLAS, memory, or wall mismatch is
`UNAVAILABLE` and cannot retire a mechanism. This paragraph records the frozen
branch; it is not rewritten in light of the result below.

This construction run cannot authorize heldout access, a main campaign,
generation, #954, exact lowering, browser integration, correctness, reasoning,
or a release.

## Official result

The exact [started envelope](r4_group_addressed_retention_decoder_cpu_recovery_started_973_raw.json)
and [result envelope](r4_group_addressed_retention_decoder_cpu_recovery_result_973_raw.json)
are preserved byte-for-byte. The started CID is
`blake3:f6155838d599063698052aa34aea309318bd1d22b50cdda84abe2052eebf9389`;
the result CID is
`blake3:68355ad2f61d02dc73dbf22de4c24834815a23069ed5735630dc365081cf91db`.

The measured execution plan worked as intended:

- all 256 optimizer steps per arm and 524,288 token presentations completed;
- wall time was `438.117083 s < 900 s`;
- timing was `0.7943369310 s/step` for exact H4 and
  `0.7919405416 s/step` for scrambled H4;
- peak process RSS was `2,523,742,208` bytes versus
  `17,179,869,184` physical bytes; and
- Darwin, Apple Accelerate, deterministic algorithms, one process, sequential
  arms, and exact four-thread settings were bound in the started envelope.

All causal and implementation conditions passed. Full-sequence/incremental
maximum deltas were `1.192093e-6` for logits, `4.023314e-7` for final state,
and `6.258488e-7` for gradients. The complete shared-prefix causality delta
was zero; every required gradient was finite and nonzero; all work signatures
were identical; emitted exact-H4 bytes replayed exactly; and heldout reads
remained zero.

## Decision metrics

Both arms memorized the construction training partition:

| arm | initial train CE | final train CE | reduction |
|---|---:|---:|---:|
| exact H4 | `8.362939` | `0.048861` | `99.4157%` |
| scrambled H4 | `8.364136` | `0.044970` | `99.4624%` |

The retained state was strongly load-bearing on the disjoint validation
partition. Exact-H4 state-on scored CE `8.976155` and `662/4096` top-1;
state-off scored CE `9.943382` and `480/4096` top-1. Disabling retained state
therefore cost `0.967227` nats and 182 top-1 decisions, far beyond the frozen
`0.05`-nat and 11-decision thresholds. This is bounded causal evidence that
the retained-memory read changes unseen next-token logits beneficially; it is
not merely connected training code or route telemetry.

The full `RETAINED_DECODER_PASS` nevertheless failed. Exact-H4 validation CE
moved from `8.371911` before training to `8.976155` after training, a
`-0.604243`-nat improvement under the frozen sign convention rather than the
required `+0.10`. Top-1 rose from `2/4096` to `662/4096`, but aggregate
probability calibration worsened enough for cross-entropy to rise. The terminal
verdict is therefore `RETAINED_DECODER_FAIL`.

H4 specificity is formally `NOT_EVALUATED` because it was nested under the
full retained-decoder pass. The diagnostic contrast is not favorable to an H4
claim: scrambled-H4 validation CE was `8.943106`, `0.033049` nats better than
exact H4, while exact H4 led by only four top-1 decisions (`662` versus `658`).

## Interpretation and next direction

### Post-terminal programme interpretation

This interpretation is downstream of, and does not alter, the frozen
`RETAINED_DECODER_FAIL` verdict or its predeclared action branch. The completed
run separately contains a positive causal state-off intervention. That evidence
does not convert the complete decoder to a pass, but it does justify retaining
the bounded attention primitive as a component hypothesis rather than treating
the aggregate decoder miss as evidence that all retained-attention recurrence
is inert. Promotion and scaling of this exact complete-decoder recipe remain
forbidden.

The result separates two questions that earlier campaigns conflated:

1. **Does the bounded retained-attention path work and matter on unseen
   sequences?** Yes, in this construction scope: the causal state-off
   intervention loses `0.967` nats and 182 top-1 decisions.
2. **Does this exact randomly initialized 3.17M-parameter decoder satisfy the
   frozen full-decoder generalization criterion after only 4,096 training
   decisions, or does its H4 action separate from a scramble?** No.

The data/model ratio is about 774 trainable parameters per construction
decision, and the near-zero training CE is consistent with severe
memorization. That diagnosis is an inference from the frozen measurements, not
a post-hoc pass. Under the contract, this exact two-block decoder is not
promoted or scaled as a complete language model. The attention primitive may
remain as a causally qualified component, but the next independently frozen
mechanism must address language-path generalization with an appropriately
sized/data-supported model and an ordinary non-geometric matched decoder. It
must not claim H4 benefit without a geometry-aligned task where destructive
transport loses.

Autonomous generation, coherent text, broad attention transfer, reasoning,
correctness, exact lowering, heldout evaluation, #954, and release readiness
remain `NOT_RUN` or unestablished.

## Language-path and paired-address successors — 2026-09-01

The data-supported successor requested above completed as
[`R4RetainedLanguagePathV1`](r4_retained_language_path_v1_973.md). Its matched
252,160-parameter retained and ordinary arms both generalized on the frozen
nonsealed validation population. Disabling retained attention cost
`0.334987556` nats and 16,660 top-1 decisions, and the retained arm remained
within both frozen competitiveness bounds. Its terminal was
`RETAINED_LANGUAGE_PATH_PASS`; a separately frozen five-prompt smoke then
completed exact autonomous retained decoding. Those results supersede only
this record's then-current sequencing, not its `RETAINED_DECODER_FAIL`.

#973 next tested whether a layer-paired exact-H4 radix address would improve
prompt-conditioned capacity without changing the qualified V1 cell. The
[paired-H4 successor](r4_paired_h4_prompt_capacity_973.md) terminated
`PAIRED_H4_PROMPT_CAPACITY_FAIL`, result CID
`blake3:508a4ff352f1e533d669d9616f65b972b0f13e8efe35867b7b095281ad940274`.
Fresh language fit slightly improved, but prompt gain was `0.0062477543` for
the candidate versus `0.0063672952` for frozen V1, and candidate directional
wins were `282/512` versus the required 308. State-off collapse, replay,
causal, and forbidden-read controls passed.

Qualified V1 is therefore preserved. There is no paired-address retry or new
generation smoke. At this historical checkpoint, the next independent freeze
targeted the prompt-state-to-logit readout seam; that later experiment completed
directional `PARTIAL`, and the next frozen successor was layerwise-normalized.
Attention remains established at the
separately qualified scopes; H4-specific advantage, reasoning, correctness,
and exact lowering remain unestablished. #973 remains open and #954 remains
blocked.

## Readout-ladder completion — 2026-09-01

After the direct readout's directional `PARTIAL`, the one authorized
layerwise-normalized variant completed with the same terminal class. It changed
only `N(a1+a2)` to `(N(a1)+N(a2))/sqrt(2)` over the later qualified V1
substrate, not this record's failed 3.17M-parameter decoder recipe.

The V3 candidate produced prompt gain `0.0286980210` versus V1
`0.0073316237`, delta `0.0213663973`, with `339/512` wins. All fresh-language
and mechanics gates passed; candidate fresh NLL/top-1 were
`3.7126411677` / `31.661826%`, and state removal cost `1.3495375637` nats and
`20,595` correct decisions. The absolute and incremental prompt-gain floors
still missed. Terminal:
`LAYERWISE_NORMALIZED_RETAINED_READOUT_PROMPT_CAPACITY_PARTIAL`, result CID
`blake3:35396bd6e64fc2c0bc7d86a84cc9e212ed913ce28e5353f5f2b8212b4cf2c532`;
independent verification CID
`blake3:3f316541dbab8061ed5ba891bf6a47ef22c55bca21fba01f6f97dbb3cb8497aa`.

This later result does not revise `RETAINED_DECODER_FAIL`. It closes the
parameter-free readout ladder and makes a freshly frozen learned associative
binding/readout the sole #973 successor. The layerwise candidate has no
generation, reasoning, or lowering result; #954 remains blocked.
