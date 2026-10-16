# Compile-only gate receipts

## Attempt 1 — 004793d1a (failed build-script configuration)

- Exact head `004793d1a037de7f6a2fadfa5fcb827bd811e226`, PR #2550.
- Session `codex-cf1-gate2550`, pod `vast-56278204`, RTX5090, Rust1.97.1,
  CUDA12.8.93, virtual architecture compute_120. Enforced disk preflight
  37,674,312 KiB available (>30 GiB). No CPU model/reference or GPU model call.
- Admitted instrument recipe `cargo +1.97.1 check --locked --features cuda`
  exits101. It emits/admit-checks ordinary and diagnostic PTX, then stops on
  resource cubin generation: `nvcc fatal: Option '--cubin' is not allowed when
  compiling for a virtual compute architecture`. Strict clippy **NOT_RUN**;
  full CUDA-feature Rust type checking is not claimed complete.
- Ordinary PTX SHA256 remains exactly
  `1a21d6c3e2773a427c14ebff9499e03386147ab41b3ce413b7b5c9123925fd28`.
  Diagnostic PTX SHA256
  `85448f9e35b21660b7b69ce57a76ff98e0c120942afeffc34fc9cbd1c8a86b11`.
  These are compiler artifacts, not runtime parity/resource or timing results.
- Corrective source change: preserve `-arch=compute_120` and explicitly pass
  `--gpu-code=sm_120` for the **resource-only cubin compilation**, consistent
  with [NVCC's architecture/code distinction](https://docs.nvidia.com/cuda/archive/12.8.0/cuda-compiler-driver-nvcc/index.html#gpu-code-code).
  Ordinary/D PTX invocations and all model operations are unchanged. No source
  was patched or gate retried on the failed lease.
- Archive: `icloud:UOR-R4/results/codex/gate2550-archive.tar`, **2,103,296 bytes**,
  cloud MD5 round-trip **a273f478cabb14824b60f1f278f17027**, SHA256 manifest
  checked before release17:44:35Z / down17:44:40Z. Includes full gate/setup
  logs, explicit exits, recipe, source head/diff, generated CUDA/PTX and
  available compiler-resource report. Own pod deleted; real fleet0/$0h.
- Provider uptime132s × $0.908/h = **$0.033293333 estimate**, not invoice;
  creation-to-SSH35s is included. Inherited Codex episode estimate now
  **$2.263728611** ($2.230435278 + this lease); PM's separately recorded
  ~$0.086 recipe lease is additional. Storage/transfer charges UNMEASURED.

This is an instrument/build blocker to repair (D22/D31), never a mechanism
negative. Diagnostic/parity/qualification/training NOT_RUN. Training/M2/
Stage B HELD; Q4 NOT_EXECUTABLE. Subsequent exact-head gate transcripts,
archives and full provider-uptime costs are attached to [source PR #2550](https://github.com/UOR-Foundation/uor-r4/pull/2550);
no source delivery or compile transcript is execution authorization.

## Attempt 2 — 4dd7e7c38 (compile passes; register confound)

- Exact head `4dd7e7c38f2d34dd1db6278895dfbe1bad67078f`, session
  `codex-cf1-gate2550-r2`, pod `vast-56278739`, same declared CUDA/Rust profile.
  **check0 / strict-clippy0 / run0**. Full transcript attaches to PR #2550.
  No model or fixture calls. Both PTX hashes equal attempt1.
- Static ptxas report: U88 vs D194 registers, both21,536 bytes stack, both
  zero spill loads/stores. **D exceeds the prospective U+16 allowance.**
  Compilation is not a valid causal-instrument result or diagnostic release;
  runtime/JIT resources, parity and timing remain NOT_RUN.
- Bounded source repair: make only probe-counter/timer storage volatile local
  storage, preserving model operations, launch and the unchanged allowance.
  No source patch/retry on the lease; repair follows archival and deletion.
- Archive `icloud:UOR-R4/results/codex/gate2550-r2-archive.tar`, **3,104,256 bytes**,
  cloud MD5 **0010ddee8ae7d7dd0007a49668e374e2**, SHA manifest checked before
  release17:52:15Z / down17:52:21Z, real fleet0/$0h.
- Provider193s × $0.977/h = **$0.052378056 estimate**; both source-gate leases
  **$0.085671389**; Codex episode **$2.316106667** plus PM's separate ~$0.086.
  Not invoices; storage/transfer UNMEASURED. No mechanism negative/count reset.

## Attempt 3 — d15e39fc7 (compile passes; reduced register confound)

- Exact head `d15e39fc716db0cbeacc1423dd6bef0507f7951f`, session
  `codex-cf1-gate2550-r3`, pod `vast-56279268`: **check0 / strict-clippy0 /
  run0**, no model/fixture/diagnostic execution. Ordinary PTX retains its pin;
  D PTX is `29615646becefc333c697f685157a4c124847b5022e070ab00d7b9c749bc8506`.
- ptxas U88/D126 registers, U21,536/D22,096 stack bytes; zero spills both.
  Volatile local observation storage reduces the194-register confound but
  still exceeds the unchanged104-register ceiling. Attribution remains
  UNRESOLVED; do not run a known-inadmissible diagnostic.
- Next bounded compilation repair caps **D only** at104 registers, derived
  from measured U88 + the already declared16 allowance. Both PTX generation
  and the D resource assembly use the cap; any new spill still disqualifies
  the instrument. Ordinary compilation/model operations/launch are unchanged.
- Archive `icloud:UOR-R4/results/codex/gate2550-r3-archive.tar`, **3,039,232 bytes**,
  cloud MD5 **8169faec3981813500a0700694e8ad6b**, SHA manifest checked before
  release18:00:30Z/down18:00:37Z; real fleet0/$0h. Provider274s × $0.977/h =
  **$0.074360556 estimate**, all three gates **$0.160031944**, Codex episode
  **$2.390467222** plus PM's separate ~$0.086. Storage/transfer UNMEASURED.

## Attempt 4 — 0d0ca6c11 (compile passes; cap introduces spills)

- Exact head `0d0ca6c1111a434269a7eb423a800042c45447d5`, session
  `codex-cf1-gate2550-r4`, pod `vast-56279979`: check0 / strict-clippy0 /
  run0; no model, parity or diagnostic execution. U remains88 registers /
  21,536 stack / zero spills, with the sealed ordinary PTX hash unchanged.
- Capped D uses103 registers /22,160 stack bytes / **56-byte spill stores
  and56-byte spill loads**. It fails both the original and amended guards.
  D PTX `c3fd5e16aa0ed40123469173bde8b3fe3ddb5e01f0d05049c3882d1529e04802`
  is **superseded**, never an admitted execution identity.
- Archive `icloud:UOR-R4/results/codex/gate2550-r4-archive.tar`,3,172,352 bytes,
  MD5 `a723983218bfd25ba9f75caff634caf6`, SHA manifest checked before
  release18:07:36Z/down18:07:42Z; real fleet0/$0h. Provider180s ×$0.977/h =
  **$0.048850000 estimate**; four source gates **$0.208881944**, Codex episode
  **$2.439317222**, PM's separate ~$0.086 additional. Not invoices;
  storage/transfer UNMEASURED. Full transcript: [PR comment6221844082](https://github.com/UOR-Foundation/uor-r4/pull/2550#issuecomment-6221844082).

## Prospective disposition — PM ruling6222308938, no dose run

The original **U+16 register ceiling is explicitly superseded** by the
[conditional PM FIT amendment](https://github.com/UOR-Foundation/uor-r4/pull/2550#issuecomment-6222308938).
Attempts2/3 retain their original inadmissibility labels above as historical
calibration, not current rules. The next source removes all cap sites, returning
to attempt3's uncapped local-observation variant (126 registers /22,096 stack /
zero spills in that earlier compile). The fresh gate must regenerate identities
and compiler resources; prior values are expectations, not new observations.

Runtime registers/AOT registers and runtime local/AOT stack must each match
exactly (zero deltas) for both U/D. Require zero AOT spills, identical shared,
D local and stack≤U+1024, disclose max_threads_per_block≥16 as launchability,
retain the fixed host block and all E/S/parity/phase/retained-sample rules.
The zero deltas conservatively refuse any observed JIT resource-layout drift;
they are not a runtime spill counter or an occupancy claim. Runtime checks,
GPU parity and attribution remain NOT_RUN. Further exact-head compile/identity,
archive and cost receipts attach to PR #2550 without moving the gated head.
