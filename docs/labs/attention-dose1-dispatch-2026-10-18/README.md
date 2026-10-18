# PM Dose 1 dispatch record — 18 October 2026

## Question and scope

Record the actual Project Manager dispatch of the seed-1 paired
`LearnV1`/`FrozenZeroV1` attention dose: its receipts, the packet and source it
binds, the runtime allocation, release build, AGY review, finalization,
check-plan and launch sequence, the owner-stopped partial run, the verified
custody, and the separate `rows_v1` successor. Decision:
**DISPATCHED; EXECUTION STARTED AND OWNER-STOPPED AFTER A PARTIAL RUN; CUSTODY
VERIFIED; DOSE NOT_COMPLETED; SUCCESSOR RELEASED SEPARATELY**.

This is not a model result: no completed dose, scientific verdict, capability
claim or progress-count reset follows. The original attempt ran 904 treatment /
1,065 frozen-zero batches, never reached Q1, and was stopped by owner decision;
final device maps, the 224 capture and fresh transfer are **NOT_RUN**. No pod,
SSH, build or model compute was started for this record; `$0` and no GPU work.
The facts below are read from the operator's and public receipts at their exact
hashes, not re-executed here.

## Exact authorities

| Authority | Identity |
|---|---|
| PM dispatch receipt | [#2029 `6244964455`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6244964455), 2026-10-18T02:59:19Z |
| Activation reply | [#2029 `6244978507`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6244978507) `DISPATCH_VALID`, 2026-10-18T03:01:09Z |
| `dispatch.json` | SHA256 `4f7681690f0ded838f8793a64117f3fdd96f2bf5464b2803730eb12f4fa543bb` |
| Static packet manifest | `REVIEW-SHA256SUMS` SHA256 `5d5481cd6c19513c316d1f7144257bcf623b0c660afb0fb9c34f9052c0fb7c44`, 234 members |
| AGY whole-packet verdict bytes | SHA256 `e3209a2ade698c34743a36ad6c1d6d59894b4211bd33ea2b423676b29d2066df` ([#2029 `6244893735`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6244893735)) |
| Protected and compiled source | `23d5b146b46cf6479d27695a555e03e3cc98e972` |
| Scientific sheet | `docs/harness/prep/m1-learned-attention-link-2026-10-17.md`, SHA256 `844814009ecc7368c0a3ecabf62434bae7d72850363e19f67c0126810fee7ffc` |
| Release-build contract | `596a978d9f81e7404a858931596accb5cbbdf7695e196ab79ba27520a418a73f` |
| Admission glue (`admission.sh`) | `f0f5c06421c625651c3d483afab7312192cca659e2d6299a4625bcec6b4ab736` (prepare/finalize/collect) |
| Driver identity | `9538f5c3908dc7f3dacea13fb97ac081b5ab7a72fcc4deabc519bfc1ff8bd04c` — SHA256 over the concatenated `git show 23d5b146:crates/uor-r4-discrete/src/` bytes of `corpus.rs`, `training.rs`, `controller.rs`, `completion.rs`, `completion_baseline.rs`, `redirect_admission.rs`, `attention_learning.rs`, `attention_evidence.rs`, `attention_transfer.rs`, `attention_preflight.rs`, `main.rs`, recomputed read-only |
| Applicable FIT | [#2029 `6230317153`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6230317153) |
| Owner resolutions | [#2029 `6236444507`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6236444507) |
| Attempt label | `uor-cf1-dose1-ac60c495ec8d601b267319b8cf548910` |
| Runtime allocation | `runtime-allocation.json` SHA256 `1f1ed8dbf12004c5cf5adde82a84a9fefcaae7a73d34d66143232a2e80058515`: pod `vast-56488188`, two GPUs `[0,1]` |
| Release build receipt | `runtime-build.json` SHA256 `eae570b4c6c10504e72fbb89fdd7ed0f13c8906a1b5cc15688257251abe053b7`, `RELEASE_BUILT_METADATA_VERIFIED`, `build_exit` 0 |
| Runtime binary binding | `controller/runtime-binary-binding.json` SHA256 `2921dce26c686e7b48cda535b22b25929d7318bb1dfc5664f38fb0538fbafe60` |
| Per-arm plan candidates | treatment `64869df7d5fde9eaf23162e3046e1fc5cbd9aaaff4d7533cd4274c2d7510113f`; frozen-zero `b52933ecf2f0291877d85b568bb1d6961e6d6d405c0c413eadbe9bc53a64ec7a` (`uor-r4.attention-learning-owner-plan/2`) |
| Launch delegation | [#2029 `6245307092`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6245307092), 2026-10-18T03:43:30Z |
| Start receipt | [#2029 `6245367341`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6245367341), 03:51:32Z; GPU0 03:48:36Z, GPU1 03:48:38Z |
| Owner stop decision | [#2029 `6245711750`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6245711750), 04:36:29Z — “Stop it; rerun on the fast path” |
| Independent stop/custody audit | [#2029 `6246074493`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6246074493), 05:24:39Z; top manifest `6b5c4f29587789adb38e63c9517a0246c70f74642d88d0dc21da0a74f9e3a9e5` |
| Local closure record | `authority-ac60c495/STRATEGIC-STOP-CLOSURE.json` SHA256 `c5ca0f6aff2467333e3e40682f054f2de55f82fb0a5850055aed5a05ba98274e` |
| Separate `rows_v1` successor | [#2029 `6246440910`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6246440910), 06:13:00Z; packet `01833fa7…`, source `20d158ae…` |

`dispatch.json` is the operator's private local packet file
(`codex-attention-dose1-operator/dispatch.json`); it is not committed here.
Its `pm_receipt_url` points at the receipt above and its
`test_fitness_reference` at the FIT comment.

Read-only checks run on 18 October for this record:

- `jq -e . dispatch.json` → parses; schema `uor-r4.attention-dose1-dispatch/1`,
  status `PM_DISPATCH_VALID`, seed 1, `max_batches_per_arm` 16404, layout
  `one_pod_two_gpus`, attempt label as above.
- `shasum -a 256 dispatch.json` → `4f768169…543bb`, matching `DISPATCH_VALID`.
- `shasum -a 256 -c REVIEW-SHA256SUMS` → all 234 members `OK`.
- `readiness-verdict.md` → `e3209a2a…066df`, matching the dispatch and the AGY
  verdict bytes.
- `release-binding/release-build-contract.json` → `596a978d…8a73f`, and
  `jq` equality of `dispatch.release_build` with that contract → `true`.
- `admission/admission.sh` → `f0f5c064…ab736`.
- Sheet hash at the base commit → `84481400…fee7ffc`.
- `git cat-file -t 23d5b146` → commit; `git diff --stat
  23d5b146..35c4c4038 -- crates/uor-r4-discrete` → empty, so the discrete
  crate is unchanged from the protected pin to the base.
- Driver provenance: the exact `for f in corpus.rs training.rs controller.rs
  completion.rs completion_baseline.rs redirect_admission.rs
  attention_learning.rs attention_evidence.rs attention_transfer.rs
  attention_preflight.rs main.rs; do git show 23d5b146:crates/uor-r4-discrete/src/$f;
  done | shasum -a 256` → `9538f5c3…bd04c`, matching the build receipt's
  `driver_source_sha256`.
- `runtime-allocation.json` → `1f1ed8db…58515`; its `dispatch_sha256` equals
  the dispatch above, pod `vast-56488188`, count 2, GPUs `[0,1]`.
- `runtime-build.json` → `eae570b4…053b7`, matching the binding's
  `runtime_build.sha256`; the on-host `bin/discrete-stack` → `1f7dd526…dbb3`,
  matching the receipt's ELF.
- `controller/runtime-binary-binding.json` → `2921dce2…afe60`; its
  `construction_manifest.sha256` equals the build directory's `SHA256SUMS`
  (`e2527d24…c5777`).
- `shasum -a 256 -c SHA256SUMS` inside the build directory → every member
  `OK`; cross-receipt equality of `dispatch_sha256` among the allocation,
  build and binding → `true`, and `build.allocation_sha256` equals the raw
  allocation file hash.
- Build interval: `release-build-status/started.utc` `2026-10-18T03:11:34Z`,
  `ended.utc` `2026-10-18T03:12:39Z`, and `build.log` ends `Finished release
  profile ... in 1m 02s`; `stat` on the ELF is 8,099,440 bytes.
- AGY review files: treatment `dfd100aa…0534`, frozen-zero `03c36420…ea13`;
  both carry exactly the six keys and `APPROVE_ATTENTION_LEARNING_ADMISSION_DIFF`
  for scopes `m1-attention-learning-relation-k8-seed1-batches0..16404` and
  `m1-attention-learning-relation-frozen-zero-k8-seed1-batches0..16404`, with
  payloads `7f8f47ec…bc9e` and `6d0749f4…6ae8` and receipt URL `6245205468`.
- `finalize.exit` → `0`; final typed plans treatment `e8d25b21…0297` and
  frozen-zero `a8b76562…8de40`; `FINAL-SHA256SUMS` → all members `OK`.
- Both `*-source-check-plan.json` → `PLAN_METADATA_VALID`,
  `compiled_identity_checked` `true`, `cuda_context_created` `false`,
  `fresh_content_read` `false`, `owner_release` `STILL_REQUIRED`, `training`
  `HELD`; their `plan_sha256` values equal the final typed plans.
- Launch/stop receipts verified with `gh`: delegation `6245307092`, start
  `6245367341` (GPU0 03:48:36Z, GPU1 03:48:38Z), owner stop `6245711750`;
  independent outcome/custody audit `6246074493`.
- `STRATEGIC-STOP-CLOSURE.json` → SHA256 `c5ca0f6a…8274e`, recording 904
  treatment / 1,065 frozen-zero batches, raw exit 143 on both, native ends
  `04:58:38Z`/`04:58:40Z`, `Q1 NOT_REACHED_BOTH`, `scientific_verdict
  NOT_ISSUED`, final maps/capture/fresh `NOT_RUN`, the custody MD5s and down
  `05:16:47Z`. `strategic-custody-source-terminals.json` independently records
  the same counts, classification and `NOT_RUN` final phases.
- Successor identity only: owner release `6246440910`, packet
  `codex-attention-dose2-rows-operator` `01833fa7…`, source `20d158ae…`, pod
  `vast-56510253` UP `06:24:07Z` (`6246536845`); its outcome is not read or
  claimed here.

The PM activation reply records that `lifecycle/dose-common.sh::dose_authority`
and the dependency pins also pass. That function writes
`lifecycle/latest-packet-check.private.log` inside the operator packet, so this
record did not re-invoke it; it re-ran the read-only checks above instead and
did not mutate the packet.

## What the dispatch releases

- The **initial dose only**: seed 1, two arms at 16,404 batches per arm, one new
  Vast pod with two RTX 5090s, one arm per physical GPU, attempt
  `uor-cf1-dose1-ac60c495ec8d601b267319b8cf548910`.
- Required construction: `cargo +1.97.1 build --release --locked -p
  uor-r4-discrete --features cuda`, with actual post-allocation ELF, source,
  kernel, geometry, config and lineage bindings. No debug or pre-allocation
  binary promotion.
- Fresh native fleet/rate/disk/source admission before `CREATE`, within the
  standing caps (`≤4` pods, `≤$8/h`) and the reviewed reserves plus the 30 GiB
  host floor. The receipt states that the local pod status seen at dispatch
  time was not fleet proof.
- These gates still preceded model work, in order: the same-dispatch release
  build; two no-context build identities; actual per-arm `/2` plans with
  genuine non-Codex admission-diff reviews; `attention-check-plan`; then the
  owner release. The build, identities, both-arm AGY approval and both
  check-plan entries are recorded below. The owner then delegated the launch
  (`6245307092`), superseding the personal-typing requirement for this initial
  dose only; execution started and was later owner-stopped (below).

## What is not released

- No operational retry. The prospective `≤2`-retries/24 h ceiling requires a
  separate actual PM grant after external full-context review; there is no
  automatic `CREATE` and no replay.
- No second seed, no optimization rung and no fresh-panel scoring. The
  pre-registered tokens/s rung stays gated on a completed dose outcome.
- No replay of this attempt. The partial run is not a source failure, mechanism
  negative, Q1 diagnosis or KEEP, and it does not reset any count.

## Fresh V2 sealed input metadata

Accepted as **metadata only**, at the exact values bound in the receipt: 400
documents, 44,678 payload tokens, 14,479 eligible response targets, 3,594
copy-successor rows and zero exact full `TRAIN`/`DEV` document overlap. Raw
tokens `0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8`,
response mask `a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e`,
ordered private row binding `c2d4c7e512379650ae3d72703c1affc9a0e85d754da7b73703dc5aad2cee6c06`
and seal report `df5d074ae09c743aa586e3463ffe87faca5703f44f7802bff8ac49ee2990395f`.

Content and labels stay sealed and unused until the typed paired `DEV H_geom`
decision and a separate PM decision. No fresh capture, ablation, selection,
redraw or automatic KEEP is released. The conditional paired fresh remains 227
batches per arm (454 paired) and is **NOT_RUN**.

## Allocation, release build and admission state

At the base commit the packet carried allocation intent and private provider
logs only: `precreate-inventory.json` was `[]` and the operator log was 0
bytes. The sequence after the dispatch is now preserved in three receipts.

### Allocation — confirmed

`runtime-allocation.json` (schema `uor-r4.attention-dose1-allocation/1`, SHA256
`1f1ed8dbf12004c5cf5adde82a84a9fefcaae7a73d34d66143232a2e80058515`)
binds lab `codex`, session `codex-attention-dose1`, pod `vast-56488188`, the
attempt label above, `dispatch_sha256` `4f768169…543bb`, count 2 and GPUs
`[0,1]`. Its `authority_issued` is `false`: the allocation is confirmed, but it
is not an owner release.

### Cost

The current provider quote is USD `1.893688888888889`/h total for the two GPUs
— a **quote, not an invoice**. Storage and transfer remain **UNMEASURED**; no
completed-dose scientific outcome or validated pure first-64 throughput claim
exists. The first-64 observations (treatment 64 receipts/281 s, frozen-zero 64
receipts/276 s) are poll-inclusive and are not a pure GPU/batch duration.

### Release build — metadata verified

`runtime-build.json` (SHA256
`eae570b4c6c10504e72fbb89fdd7ed0f13c8906a1b5cc15688257251abe053b7`, under
`metadata-uor-cf1-dose1-ac60c495ec8d601b267319b8cf548910/construction/release-build/`,
directory manifest `e2527d24…c5777`) records
`RELEASE_BUILT_METADATA_VERIFIED` with `build_exit` 0 and the exact required
argv `cargo +1.97.1 build --release --locked -p uor-r4-discrete --features
cuda` at `UOR_NVCC_ARCH=compute_120`.

| Build identity | Value |
|---|---|
| Shared ELF (both arms) | `1f7dd5268b6e7b69c3a22a2c7a4905d5484cdedb720ee89ceb541b18d8b2dbb3`, 8,099,440 bytes, ELF64_X86_64; the on-host `bin/discrete-stack` matches |
| Protected source commit | `23d5b146b46cf6479d27695a555e03e3cc98e972` |
| Compiled source | `64e484637425f009969c6ec270ccbae6ab7ca50e89fc244240058231d00d9dad` |
| Kernel | `5cd0e73241fd895f2dd0004c83af3d1eb9f89e00ec2e933c1acc44fc7127bfd6` |
| Geometry | `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327` |
| Driver | `9538f5c3…bd04c` (composite above) |
| Treatment config / lineage | `c446aa7f…9f216` / `ffad0e1d…ba2e1` |
| Frozen-zero config / lineage | `1435f016…071c3` / `f859bf22…7a9ae` |

The sealed ELF is **8,099,440 bytes** (host `stat` and the runtime build pin).
AGY's earlier explanatory `14,768,528` byte count is corrected; the raw ELF
SHA and every review/payload/plan hash are unchanged.

`cuda_context_created` is `false` for both arms, `metadata_identity_calls` is
2, `model_commands` and `init_commands` are 0, and `training` is `HELD`. The
receipt's own `validation_scope` is "actual release build and no-context
metadata; no model/runtime CUDA qualification".

`controller/runtime-binary-binding.json` (SHA256
`2921dce26c686e7b48cda535b22b25929d7318bb1dfc5664f38fb0538fbafe60`) repeats
the ELF/source/kernel/geometry/config/lineage identities. At bind time
(03:12Z, before launch) it records `cuda_runtime_resource_admission: NOT_RUN`,
`live_resource_admission: NOT_PERFORMED_BY_HOST_BINDER`,
`model_computation: false` and
`typed_config_owner_admission: EXISTING_PROTECTED_SOURCE_REQUIRED`, with
`authority_issued: false`; the owner delegation and actual launch that followed
are recorded below.

**Distinction:** the RELEASE construction and its metadata identities are
built and verified. That is not runtime CUDA resource admission, not an owner
release, and not training. During the build and check-plan, no CUDA context was
created and no model command ran; execution began only after the owner's launch
delegation.

### Per-arm plans — AGY approved, finalized and checked

Both `/2` plan candidates exist as metadata: treatment raw
`64869df7d5fde9eaf23162e3046e1fc5cbd9aaaff4d7533cd4274c2d7510113f` and
frozen-zero raw
`b52933ecf2f0291877d85b568bb1d6961e6d6d405c0c413eadbe9bc53a64ec7a`, scoped
`batches0..16404` with the exact ten-key `ExecutionIdentity` (base
`ef9d3099…`, reviewed source `23d5b146`, sheet `84481400…`, source/kernel/
geometry/executable/driver/config/lineage as above).

AGY reviewed the genuine payloads and posted
`APPROVE_ATTENTION_LEARNING_ADMISSION_DIFF` for **both arms** in
[#2029 `6245205468`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6245205468)
(2026-10-18T03:30:18Z). The six-key review files are treatment
`dfd100aa5061d139fdd226aba24338499b4178e1d59f70e16bae490129ab0534` with
payload `7f8f47ec620608adc0460e9f1e0c3d0ce5a6a4b900f37a226645b3127e87bc9e`,
and frozen-zero `03c364202de6edeae03d00cafaa249c89acdd75baa609e7f47fa2d427964ea13`
with payload `6d0749f4ce6b9ef1c0097c7a5c24c5cdb3cc7b2ea238147c1813ea4ae16f6ae8`.

Codex's frozen-admission `finalize` then exited 0. The final typed plans are
treatment `e8d25b21b111b0b310eb79233b3864e6679057783771894e23c9ea99b9bb0297`
and frozen-zero
`a8b76562d829beea201d006c2ac506d313d105841eabc5bf8189efab29e8de40`, and
`FINAL-SHA256SUMS` verified. Running the actual RELEASE ELF on both
`attention-check-plan` entries returned `PLAN_METADATA_VALID` for both, with
`compiled_identity_checked` true, `cuda_context_created` false,
`fresh_content_read` false, `owner_release` `STILL_REQUIRED` and `training`
`HELD`. The personal TTY card was ready at `final/PERSONAL-TERMINAL-CARD.txt`; at that
03:34Z gate, model, init and owner-release had not run
([handoff `6245235242`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6245235242),
2026-10-18T03:34:07Z). The owner then delegated the launch and execution began.

### Execution, owner stop and verified custody

The owner delegated the launch for these exact two plans in
[#2029 `6245307092`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6245307092)
(2026-10-18T03:43:30Z): Codex was allowed to run the pod without another
personal kickoff approval, the personal-typing requirement was superseded for
this initial dose only, and operator attribution was recorded. The start receipt
[`6245367341`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6245367341)
(03:51:32Z) records treatment on GPU0 at 03:48:36Z and frozen-zero on GPU1 at
03:48:38Z, both attempt-0001 and `LIVE_RETAIN`.

The owner then chose **“Stop it; rerun on the fast path”**
([`6245711750`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6245711750),
04:36:29Z). After the reviewed driver delivery
([`6245768780`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6245768780),
04:44:10Z) the run was stopped; the stop helper is separately reviewed at
[`6245906700`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6245906700)
(05:02:40Z). Actual partial execution: contiguous **904 treatment / 1,065
frozen-zero batches**, both below the Q1 gate of 4,096. The native log tails end
at `04:58:38Z` / `04:58:40Z` with raw outcome `COMMAND_FAILED_TERMINAL` / exit
`143` on both arms, independently attributed as **OWNER_STOPPED_STRATEGIC** —
not a source failure, mechanism negative, Q1 diagnosis or KEEP.

Consequences, exactly: **Q1 NOT_REACHED_BOTH; dose NOT_COMPLETED; final device
maps, the 224 capture and fresh transfer NOT_RUN; scientific verdict
NOT_ISSUED**. No capability result, completed-dose claim, negative-count reset
or optimization release follows. The original dispatch is **consumed**.

Custody was independently accepted in
[`6246074493`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6246074493)
(05:24:39Z): immutable top manifest
`6b5c4f29587789adb38e63c9517a0246c70f74642d88d0dc21da0a74f9e3a9e5`;
selected same-compressed-reader verification of **198 regular members /
237,872,396 uncompressed bytes**, gzip `24,808,873` / SHA
`05126b3b555ff72d247e895e6d651eb9a69ce41c4db004e1e060e71d20c22014`; cloud data
`25,345,536` / MD5 `ec029a201130ab41b3cd89b3cc6355de` (05:05:53Z) and late
operator `143,634,944` / MD5 `a75a23a70ab438e46f399a96df39ad8e` (05:11:25Z),
both verified before teardown; fresh quiescence at 05:16:41Z with zero
owned/unknown live processes and GPU applications; native **down at 05:16:47Z**,
fresh configured fleet `[]` and own lease absent; separate after-down closure
`4,026,880` / MD5 `fb4a4a264408c44ae61fbad5559a7e05` with manifest
`ff2ddfce4e91f6e1d106a9467b6f17d1be8b19545fbb515612a7d2d20fc9930a`, read back
at 05:23:37Z. The local closure record `STRATEGIC-STOP-CLOSURE.json` (SHA256
`c5ca0f6aff2467333e3e40682f054f2de55f82fb0a5850055aed5a05ba98274e`) matches the
counts, exits, times, custody MD5s and down.

Cost remains a proxy, never an invoice: `8,009 s × $1.894/h = $4.213623889`;
the prior mixed `$4.804471110` gives mixed `$9.018094999` plus separate PM;
storage/transfer **UNMEASURED**.

### Current successor — `rows_v1` (separate identity)

The owner released a **separate** fast-path successor in
[`6246440910`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6246440910)
(06:13:00Z): packet `codex-attention-dose2-rows-operator` with 1,376-member
`REVIEW-SHA256SUMS`
`01833fa7bc540c5e15e02e0156521a1455502a6261cd5747c153519b7dbb7415`, protected
source `20d158ae31052ef4e62896fadcab23ae341deced`, both arms
`counterfactual_path: rows_v1` with configs `52b32dc8…` / `6a035717…`, and a new
pod `vast-56510253` (native UP 06:24:07Z, attempt
`uor-cf1-dose2-rows-322faf70e6b3479087c890897274da24`,
[`6246536845`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6246536845)).
The same sheet, seed, arms and gates carry forward; the source, packet, binary
and pod do not. This record does not conflate that identity, inherit this
attempt's state, or claim its outcome.

### Preserved launch state

The earlier
[#2029 `6245029696`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6245029696)
review (03:07:37Z) remains the record of the launch hazard:
`CONDITIONAL_APPROVE_EXISTING_POD_REVIEWED_POST_UP_TAIL_ONLY`, original
operator `37502` absent, 0-byte outer log with no recorded exit, no setup or
build at that point, and the historical cause **UNVERIFIED**. The later
receipts show the reviewed post-UP continuation completed setup0, five data
transfers with SHA verification, and the native release-build job started at
03:11:34Z and exited 0 at 03:12:39Z (log `1m 02s`) — with no `CREATE`,
rebuild, retry or source patch. The
unverified launch cause is not a mechanism negative, and the create-response
file's provider instance key stays private and is not copied into this record.

## Decision and next action

`DISPATCH_VALID` stands and is now **consumed**. Final state:
**dispatched; allocation confirmed at `vast-56488188`; RELEASE build metadata
verified; both-arm AGY approval and both check-plan entries PASS; execution
started 03:48:36Z/03:48:38Z; owner-stopped at 904/1,065 batches; custody
verified and pod down 05:16:47Z; Q1 NOT_REACHED_BOTH; dose NOT_COMPLETED; final
maps/capture/fresh NOT_RUN; scientific verdict NOT_ISSUED**.

No scientific result, completed dose, mechanism negative or progress-count
reset follows. The current successor is the separate `rows_v1` dispatch released
in `6246440910`; its outcome is not claimed here. Fresh V2 remains sealed pending
typed `DEV H_geom` plus a PM decision.

## Provenance

Base `35c4c40380ad25b2ba4cde0239b93110b26768db`; branch
`kimi/dose1-dispatch-record-2026-10-18`; worktree
`.worktrees/kimi-dose1-dispatch-record`; lab `kimi`; branch claim recorded in
`docs/harness/claims.json`. Record authored and revised 18 October 2026,
host-only. The first revision added the runtime allocation, release-build and
binary-binding receipts and the driver composite provenance; the second
corrected the build interval to start 03:11:34Z / exit 0 at 03:12:39Z, added
AGY's two-arm approval, finalization and both check-plan results, and recorded
the quote; this revision adds the owner launch delegation, start receipt,
strategic stop, partial-run counts, verified custody/closure and the separate
`rows_v1` successor.
