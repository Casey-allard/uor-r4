# Native rung-0 complete learning/output path repair

References #2029, #2028 and #2030. Final source `05c84bece50789163bd0f049a2ae76b8a314ef45` implements the observed dispatch,
learning-credit, configuration, row-cap and evaluation repairs and passes the
named finite CUDA controls at the actual rung-0 shape. This unit delivers through
[protected PR #2318](https://github.com/UOR-Foundation/uor-r4/pull/2318), with prospective scientific rung-0 FIT recorded against the corrected sheet; this is implementation/device
evidence, not a trained language result. Source checkpoint `6f6eebe68` and the
intervening failed/corrected attempts are preserved as preparation history.
The [indexed restoration patch](../../history/branch-archive/rung0-pre-repair-inactive-20261013.patch),
SHA256 `ac72df2ae916cc783f2a0b649355fdec5e30a0a464dc57d6a5a9e86e5400ffa5`,
restores the training crate to pre-repair base `d4dd3caa9`; an isolated Git-index
application reproduces that base tree. Retention does not permit historical
prohibited execution.
**Training remains held, including the separate M2 training/rental hold.**
Line: unified geometric route/action/learning integration, count9/3→10/3 in
PR2318, then11/3 for postmerge harness-cache/ownership reconciliation; no
headline reset. M1 memory20/40, reply37/232; M2
saved437/512, opened0/128 and genuinely fresh NOT_RUN are unchanged.

## Current decision and mechanism

Kimi's [ratification6139449267](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6139449267)
approves the repairs and direct interior initialization, and supersedes
Amendment 2's dev-window truncation: every masked target with a within-document
predecessor is scored, state resets at BOS, and context may overlap for a tail
that is scored once. Kimi remains the project manager. The scoped Amendment4
corrects the active sheet under the owner's complete-path instruction, and its
prospective FIT review preserves every acceptance bar and owner hold.
The [prospective finite FIT6139464540](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6139464540)
is for fixed lawful CUDA implementation controls only, not a training release.

The [head/shared writer](../geometric-head-shared-writer-2026-10-13/README.md)
landed through #2301 at `d41dc422f66ff9bc69d0b5c39c6773ca5eb4fb07`. Its
head-only controls did not qualify the ordinary complete learning path. The
source audit found five concrete defects:

| Observed defect | Implemented repair |
| --- | --- |
| The train CLI omitted `rec_gates`, leaving a fresh configuration historical | Accept and bind `rec_gates=geometric`; native train requires the declared CUDA/F32 recurrence-only Hamilton/FBS/head/margin configuration |
| Ordinary recurrence dispatch entered the historical fused sigmoid/exp/GELU core despite the geometric setting | Route the actual ordinary forward through composed geometric recurrence; reject unimplemented transport-snap/host fallbacks |
| The writer's 4096-row cap rejected batch 16×context 384=6144 | Admit up to 8192 rows with a bounded candidate-vector-element guard; source fixtures cover the actual 6144-row shape |
| Dev/final evaluation still requested NLL and the run defaulted to host sampling | Use native masked-response token/address margins and raw ranks, require `sample_tokens=0`, preserve historical evaluator source outside the admitted path |
| The lower rate-table edge used an unsigned displacement in its secant credit | Use signed neighbor displacement in the CUDA adjoint and read the exact declared keep table |

Additional source repairs use direct round-robin rate-address bin centers for
geometric `rec.decay`: `((lane %16) +0.5)/16`, covering 0.03125 through 0.96875.
Historical initialization remains unchanged. The previous historical transformed
logits saturated the geometric clamp. The rate credit remains an explicit secant surrogate, not the true
derivative of hard address selection or a measured descent guarantee. CUDA
norm/clipping stays on device; the host receives the final diagnostic norm and
does not compute a clipping scale. Native source guards reject unsupported
read/memory/served/snap configurations and host tensor diagnostics
(`UOR_NAN_TRACE=1`). A healthy native run extends a `max_seconds` estimate with
one notice rather than stopping; step timing synchronizes GPU boundaries and
includes the entire step, including host window preparation. Final compilation and fixed-control timing is reported below. No training
throughput measurement was made.

The admitted source call path is: pinned tokens → shared vector lookup →
normalization → six composed geometric recurrences and Hamilton/FBS feed-forward
blocks → final normalization → typed product-cell candidates → explicit omitted-
target token margin plus auxiliary address margin → backward → device clipping
and CUDA AdamW implementation. No optimizer step is executed by this
qualification. The CLI records a same-seed zero-step constant, actual dev curves
and final native metrics; checkpoint lineage binds the native head contract,
mask contents and dev policy, and model artifacts retain source/geometry binding.

## Dev inventory and metric contract

Rust metadata inventory (`local/dev-inventory.json`) observes 85776 physical
bytes: a 64-byte UORT header followed by 42856 u16 payload tokens. The response
mask has 42856 bytes, 14287 positive entries and14287 eligible within-document
response targets across 400 documents; verified BOS identity is 0 from the pinned
tokenizer. The payload gives 111 global 384-token blocks plus 232 tail tokens.
The earlier 42888 count divided the entire file size by two and accidentally
counted 32 header words as tokens; no corpus is regenerated, padded or changed.
The evaluator's document windows are reported separately from these global
inventory blocks. BOS is resolved from the actual tokenizer special-token map,
never assumed globally. Kimi's
[header correction6139637354](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6139637354)
is applied in the [scoped active sheet correction](../../harness/prep/m1-rank-margin-rung0-2026-10-13.md).
Codex's [coordination6140439165](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6140439165)
records its limited factual/source scope; Kimi's stale claim is preserved.
Seeds1/2 are paired across M1/M8, tokenizer restoration is byte/SHA verified at
private HF commit `435d5f781cd7b8518185cf6e7fda2a7addb0532a`, and exact
source/CLI/init pins are in the sheet. Scientific verdict is FIT for this configuration;
acceptance bars and all holds are unchanged. All eligible masked targets, including the tail, are
scored once. The report shows actual/expected denominators and any mismatch.

Training uses token_margin + address_margin, coefficient 1 each. Dev bars and
unviolated-target diagnostics read token_margin alone under Kimi's amended
contract; address/total margins and total-unviolated rate remain separate.
Top-1 and recall@8 use the actual bounded kept list with smallest-token-ID ties
and no target injection. Table-scale diagnostics report the mean per-token L1
norm. The count reference uses raw occurrence counts over the entire decoded
dev stream, including the tail, on CUDA; its hinge has count units and is never
compared as a loss in the model's mean-L1-displacement units. Normalization,
rank/count arithmetic and metric reductions remain on CUDA; only final receipt
scalars cross to the host. No likelihood, BPB or normalized-exponential scorer
runs on this native path.

The implemented CLI form (preparation only; the owner hold forbids execution)
is:

```text
geometric-stack train arch=geometric pattern=rrrrrr width=256 heads=4 stack_mlp=384 context=384 rec_gates=geometric gate=hamilton projection=spiralcore_fbs head=spiralcore_product objective=hinge device=cuda precision=f32 batch=16 train=TRAIN_TOKENS valid=DEV_TOKENS valid_mask=DEV_MASK tokenizer=PINNED_TOKENIZER expected_dev_targets=14287 eval_every=128 eval_windows=111 final_windows=111 sample_tokens=0 steps=7324 lr=0.002 warmup=200 min_lr=0.1 weight_decay=0.1 clip=1 checkpoint_every=128 max_seconds=inf seed=SEED out=EXCLUSIVE_REPORT_ROOT
```

The M8 arm uses `objective=hinge8`. Dataset/tokenizer paths, seeds, input hashes,
retained bars and any tail-policy amendment belong to the exact reviewed sheet;
this example command is not approval to run it.

## Qualification and outcome

**Finite CUDA implementation qualification: PASS, 18 controls on final source
`05c84bece50789163bd0f049a2ae76b8a314ef45`.** Eleven library controls pass (ten CUDA and
one metadata/CPU-refusal control); the CLI passes six metadata/argument controls
and one CUDA evaluator/artifact control. Final CUDA library compilation takes
6.02s and CLI test compilation 4.79s. The individually recorded test times sum to 9.68s;
the full batch 16×context 384 core control takes 2.04s. These are scoped control
wall times, not training speed or a summed campaign-duration measurement.

The actual batch 16×context 384 six-recurrence graph, width 256, MLP 384, V=4096,
passes forward/backward with every collected gradient on CUDA and finite, and
nonzero opening/decay credit in all six layers. Stored zero-state composed
answers, signed rate edges/exact table bits, CUDA clipping known answers and
subnormal-configuration refusal pass. The full 6144-row writer control passes.
The native evaluator tests exact token/address/total units, IDs/ties/recall8,
document-mask/raw-count known answers and artifact reload/emission; missing and
stale head bindings refuse. Its fixture uses V512 so both M1/M8 have enough
admitted non-target rivals, and exclusively claims its `/workspace` report root.
CPU tests perform metadata/argument work only; no CPU model reference executes.

Exact qualification identities:

| File/executable | SHA256 |
| --- | --- |
| `src/geometric_stack.rs` | `a6d26a13864708053262041a44a4de886cc822b8b3a110dc21ee021d55c2879c` |
| `src/cuda_stack_kernels.rs` | `fe1052569be56a585ef351621d96bc8ba776034a9a34407500134e513c156b96` |
| `src/native_vector_writer.rs` | `6a444fe2158c9fa990f638ca7758613c446c9ef65437595da2dfb18528dcaae3` |
| `examples/geometric-stack.rs` | `7dfae090af3bd919e012c21022fff1c7ccaca13dd94c2e05bdd34963c37f5467` |
| `examples/geometric-stack/native_rung0.rs` | `69dc381e82eb5a4c7c16768efcfa9d1051a4daaa32963557f2cd2b4df0b58069` |
| Library test binary,739052032 bytes | `c47941dfbda7b2c41b86dd97389ab97434d908ef8b9e57191492162048bb0bee` |
| CLI test binary,630206392 bytes | `e5ecba08467118ce2b9fe9f379113e16676d0de7db0bcc6e30ec01365a092614` |

Source paths above are relative to `crates/uor-r4-training/`. Sampled GPU peak
is 9274MiB; the maximum recorded host test RSS is 486044KiB. The sample is an
observation at these fixed controls, not an allocator bound or whole-run cost.

The review found and corrected a diagnostic entry-point bypass: direct
`composed_logits`/helper calls now apply the native path guards too. Earlier
setup/compiler/control failures are preserved, not counted as mechanism
negatives. An intermediate parent passed four runtime controls before a stale
control namespace stopped the attempt; the later small-fixture context repair
is included in the final source above. Those partial attempts do not qualify
the final revision; all 18 final controls were rerun on the frozen source.

**No optimizer update executes.** The CUDA AdamW implementation is compiled and
source-reviewed; only its clipping helper receives a fixed device control.
There is no training, corpus evaluation, generation campaign, quality comparison
or BPB result. The finite controls qualify the declared forward/backward/output
boundaries, not useful language, fresh transfer, full M2 intent/compiler use,
energy savings or integer/table D11 serving. The prospective scientific rung-0
FIT review binds Amendment4's exact source/data/operator/metric identity; its
reviewed pre-stamp sheet blob is `bf0a0ededd2027794ff5ddcdab23f53782683df3`.
Protected source delivery and explicit owner hold release precede execution.
A FIT verdict never releases an owner hold.

## Resources and delivery

Owned worktree `codex-rung0-complete-path`; Kimi's sheet and other labs' source,
artifacts, leases and jobs are preserved. Local source/metadata preparation is
non-model work. The finite qualification card projects ≤$1.50 for setup/build/
controls through uor-pod, one owned CUDA12.8+ Ubuntu24 GPU, within shared≤4 pods/
≤$8/h caps; target≤30GiB, host RAM≤8GiB, device graph≤24GiB, retained receipts≤1GiB.
Local admission observed 44GiB free with 30GiB floor and 128MiB absolute margin.
Executable identities and final compilation/control results are recorded
above. Owned pod `vast-55690699`: RTX 5090, Ubuntu 24.04.3, CUDA 12.8.93,
Rust/cargo 1.97.1, driver 580.105.08, image tag
`runpod/pytorch:1.0.2-cu1281-torch280-ubuntu2404` (image digest UNVERIFIED).
The initially absent Rust toolchain and all corrected build-input/controller/
fixture failures are preserved; installation succeeded and the final compiler
identity is retained. The owning session released and deleted the pod after
preserving its logs. Ledger uptime is 3116 seconds at $0.536/h, an estimated
$0.463938 before provider billing. Verified preservation: private HF dataset
`caseyallard/uor-r4-store`, commit `8c3c1fe0f0b37b92fe6e2a592ecc7d57b1155815`,
prefix `pod-setup/rung0-complete-path/20261013-05c84bece`; all five files were
downloaded at that commit and byte-compared, with SHA256 manifest verified.
iCloud object `codex-rung0-complete-path-20261013.tar`, 1629696 bytes, MD5
`c55748b417084bad009115f5b6ab55c5`, passed round-trip and exact index readback.
The final canonical merge is appended separately after protected delivery.
Missing historical
model-time ledger is not a zero balance. Only source-bound finite controls are
admitted; no model fit or unrelated rental campaign is released.

Protected PR #2318 merged at `5b22739a31eafb131d25e098bb7f02ab803d4fe7`.
Fresh origin/main contains it. Full merge tree `fdfc1405bcbb7d73f4b04ea0620c87265176f937`
equals the reviewed head and independent merge combination; the training crate
is byte-identical to qualified source05c84bece. Peer #2317 added only the existing
tempfile dependency membership to the harness; full Cargo.lock equality is not
claimed. Private HF DELIVERY.json is pinned/download/byte/SHA verified at commit
`91cdedd6a307ee96c829472ce7d414bb8b0778b0`, SHA256
`5510e7ab9959a55f4360df5f530e0a3851d7dc9e0cd51bfae2f13e3d20e339ee`.
Actual merge/source/tree proof and owned cleanup receipts are preserved separately. Historical
source and failures retain their exact scope; a tooling defect is a blocker to
repair, not a mechanism negative. Head/writer #2301 delivery is complete and is
not reopened by this successor's qualification work.
