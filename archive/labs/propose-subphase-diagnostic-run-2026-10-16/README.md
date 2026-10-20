# Checkpoint-0 propose sub-phase diagnostic — sealed execution

**COUNTERFACTUAL_DOMINATED in all four operation/policy cells.** This is the
bounded diagnostic authorized by [Kimi PM](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6216273515)
under the [FIT sheet](../../harness/prep/m1-propose-subphase-diagnostic-2026-10-16.md).
It is **diagnostic, not qualification / not a p95 estimate / not a training rate**.
Training/M2/Stage B remain HELD; Q4 NOT_EXECUTABLE. No training, corpus rows,
apply, fixtures, extra batches or retry. M1 qualification instrumentation
subline count 0/3 and suspended headline 37/232 remain unchanged; no mechanism
acceptance, negative or registry count reset follows.

## Observed result and decision

Each A/R arm executes one excluded warmup and exactly three timed batches.
Each batch proposes `Route(0)` and then `VectorBlock(0)`, sixteen candidate
ordinals each, without apply. The table describes **three samples per cell**;
means and ranges are descriptive, not estimates of tails or trained throughput.
All timings are from the same-stream event pairs and enclosing caller wall
timers, not inferred from CPU wait alone.

| Policy / proposal | Propose wall mean (s) | Counterfactual event mean (s) | Counterfactual / wall range (%) | Chooser event mean (µs) | Decision |
| --- | ---: | ---: | ---: | ---: | --- |
| A / Route | 7.364528 | 7.364325 | 99.996313–99.997961 | 27.392 | Counterfactual dominated |
| A / VectorBlock | 7.320291 | 7.320091 | 99.997191–99.997328 | 34.986 | Counterfactual dominated |
| R / Route | 7.366710 | 7.366554 | 99.997768–99.998097 | 19.776 | Counterfactual dominated |
| R / VectorBlock | 7.323035 | 7.322821 | 99.995770–99.997945 | 27.285 | Counterfactual dominated |

A is `nearest_address_margin_v1`; R is `occupied_cell_rank_hinge_v1`.
Every timed sample independently exceeds the prospective 80% counterfactual
threshold. No cell is averaged into another to reach that decision. The raw
[record](k16-propose-subphase-record.json) retains all twelve timed profiles,
four excluded-warmup profiles, candidate observations and both bridge receipts.
[analysis.json](analysis.json) retains per-cell min/mean/max, all nine host-span
means and integrity checks; it is derived metadata arithmetic, not model execution.

The disjoint `status_read_check` host span averages 7.364376/7.320142 seconds
for A Route/VectorBlock and 7.366601/7.322877 seconds for R. It includes the
wait for the counterfactual kernel. This is **not** evidence of seven-second
host transfer or allocation work: the GPU event interval overlaps that wait.
All other individual host spans average below 63 µs per cell. The wall minus
the two disjoint GPU events is 126.865–267.292 µs across the twelve samples;
that residual is not an exclusive attribution of overhead or profiling perturbation.
Chooser events range 13.344–42.495 µs. Event readback averages 4.248–5.851 µs
per cell and remains outside the original propose/full-unit timers.

**Never add GPU-event durations to CPU wall/wait durations.** The original
full-unit sums (carry + both proposes + both observations) are 44.056677681 s
for A and 44.071248951 s for R across three batches each. Diagnostic batch-wall
sums are respectively 44.056850697 s and 44.071408759 s: additional readback,
epoch reads and gaps contribute 173.016/159.808 µs, of which event readback
accounts for 31.319/28.794 µs. The separately labeled caller context-sync
intervals average 4.682–8.302 µs per proposal cell. Serialization after the
batch-wall interval is excluded. Excluded warmup wall durations are
14.694918160 s (A) and 14.700249498 s (R), never included in the timed table.

**Next decision:** separately preregister and review a decomposition of the
counterfactual kernel's candidate/token/context work before any further
instrument/optimization execution. These observations localize the dominant
interval; they do not establish algorithmic necessity, identify the instruction
or memory bottleneck, or show a semantics-preserving optimization. No matched
profiling-off control was run, so perturbation is not independently bounded.
Do not pool allocations, remove synchronization, run extra samples or resume
qualification on this receipt's authority. The D-Packet 100-batch rate slot
remains unfilled for the optimized stack.

## Exact execution and identities

The run used merged main `8199af0076d861bade607b7aa2fc29bfaecb81ae` (#2542),
whose entire tree was verified identical to approved
`38c08160365e299d166cfc258bf256a27b8a5f1d`, not merely an inferred ancestry match.
#2541 supplied the instrument; #2542 supplied the independently reviewed
[evaluation-only bridge](../checkpoint0-identity-bridge-2026-10-16/README.md).
The [executed pin](diagnostic-pin.json) is byte-identical to the merged template.
Synthetic windows/targets only: rows64, V4096/B4/C192/K16, Route read, seed1,
epoch0. There is no language-quality measurement or fresh-transfer substitute.

| Identity | SHA-256 |
| --- | --- |
| Raw diagnostic record | `3b53d6c49df76b627e2db407455b75493afa8396ea9f7afcf9e1651e3c8ff961` |
| Executed pin | `8d753b52877198747a313b66908e53cbf5fdd6958896d702a483bad749e38f3d` |
| CUDA release executable | `ca4ad89bbb9e7fd6402cb48986c89d6bf7e43748102e8b2f6d7a22ec4a9e940c` |
| Compiled production source | `4c0bf3ab97a82c3fed5187eb448b5d007a7dbe7271908b4f491e66d27697ea45` |
| PTX (unchanged) | `1a21d6c3e2773a427c14ebff9499e03386147ab41b3ce413b7b5c9123925fd28` |
| Geometry | `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327` |
| Original checkpoint file | `9a76743e401759b7d3aabc708ab0a26a9470ea474dc140f328c7b1aab59e49e4` |
| Original checkpoint source | `f6c4cc9f7a842ecb7a58fb569dcb7af61f84eb533cb4a059ee614be9c0f18fa7` |
| Initialization config file | `eff7a77e1c3c8454324c9ddf31a6c08bdbe1b213d3324fe7b7889c0fe7665ae4` |
| Typed initialization config (inherited sealed pin) | `58b99b595aa6110c8250e8183b3875e93d19d887a941d295c20937dea73b370d` |

Original checkpoint bytes remain unchanged before/after execution. Both mapping
receipts name the pre/post source hashes, `bridge_used:true`, unchanged payload
hash `2c2490a00545fc58c35fb5c3756382b7eb2fc14586e43ae85103019ca6aeaab9`, and
in-memory serialized snapshot hash
`d70ec39b97613de568050ffac4eba2293c9900753fdbb8146229578dd40d0ddd`.
Only the source identity is adapted, before the existing A/R policy selection;
no rebound checkpoint is written and training resume is false. The unchanged
production loader performs device admission. Mapping alone is not admission:
the completed cells and invariants below supply the separate execution evidence.

Pod `vast-56253645`: one NVIDIA GeForce RTX5090, UUID
`GPU-1daf8397-88e7-c3e5-e50c-0dbc1459969c`, driver595.71.05, CUDA12.8.93,
Rust/Cargo1.97.1, `UOR_NVCC_ARCH=compute_120`, release profile. Before/after
`nvidia-smi -q` captures are archived: graphics/SM clocks 2895/2910 MHz,
memory14001 MHz, power limit575 W. These endpoint readings are not continuous
clock or energy measurements. The runtime reports counterfactual local bytes
21,536; that is resource metadata, not a measured spill/occupancy diagnosis.

The [full command/build transcript](transcript.log) records:

```sh
export PATH=/root/.cargo/bin:/usr/local/cuda/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
export UOR_NVCC_ARCH=compute_120 CARGO_TARGET_DIR=/workspace/cuda-check-target
cargo +1.97.1 build --locked --release --features cuda \
  --manifest-path docs/labs/k16-timing-2026-10-15/instrument/Cargo.toml
/workspace/uor-r4/evals/propose-subphase-diagnostic/k16-timing diagnose \
  --pin docs/labs/propose-subphase-diagnostic-2026-10-16/diagnostic-pin.json
```

Build, diagnose, enclosing remote script and job-footer exits are all **0**.
Release build took 27.97 s. Diagnose ran 13:59:09–14:01:10 UTC, 121 seconds
including warmups/admission/record emission. Existing dependency warnings are
retained, not suppressed. Pre-merge check/strict-clippy evidence remains in
[#2542](https://github.com/UOR-Foundation/uor-r4/pull/2542#issuecomment-6215964367);
this diagnostic lease did not rerun fixtures, clippy, qualification or training.

## Integrity checks

- Exactly two checkpoint0 bridge receipts, two excluded warmups, six timed
  batch records (indices1–3 in each arm), twelve timed proposals and two cell
  summaries in the expected order. Every proposal is epoch0, sixteen candidates,
  with `applied:false`; throughput/rate fields are null.
- A before/after snapshot digest:
  `0327ceaa81a580182757ea699f356f6584d9308d5e5f186ee9b3da209919a599`.
  R before/after snapshot digest:
  `a6e2674ec9c2ce495e5c6e8199c58f80176f5b68be6c7c74ae5d4476f095dd74`.
  Both epochs remain0. The different A/R digests include the declared policy
  selection before each cell and are not evidence of an update.
- All sixteen warmup/timed profiles have nine completed disjoint host spans
  summing exactly to their API-call wall duration. Every stored vendor clock
  bit pattern is finite/nonnegative and independently integer-decodes to the
  stored nanoseconds. Event pairs lie within enclosing wall plus the registered
  `max(1 ms, 1% wall)` tolerance; no contradictions or faults.
- All six batch-wall durations bound their full-unit sums plus readback; both
  cell full-unit sums independently reconstruct from raw carry/proposal/
  observation records. Original file/pin hashes and tracked source are unchanged.
- Local post-run checks are stored-receipt parsing/hash/arithmetic only, never
  CPU model/reference execution. All26 archive manifest entries verify; copied
  raw record, pin, transcript and manifest match the sealed bytes.

## Archive and complete cost

Session `codex-propose-subphase-20261016` used one fresh bounded lease with an
EXIT trap installed before creation. It archived before release and deletion:

- `icloud:UOR-R4/results/codex/codex-propose-subphase-20261016.tar`
- **5,202,432 bytes**, cloud upload/download MD5 **`a088932f8b921630a127198b0dac8984`**.
- [SHA256SUMS](SHA256SUMS):26 entries, checked after extraction. Paths in this
  manifest are relative to the archive root; the repository embeds only the
  review subset. Full archive includes original checkpoint, executable/PTX,
  config/pin, raw record, build/diagnose/job/setup transcripts and exit files,
  setup/execution recipes, GPU context, source-head and empty tracked diff.
- Provider uptime **468 s at $0.908/h = $0.118040**, estimated not invoiced.
  Ledger up13:54:03Z, release14:01:49Z, down14:01:53Z; ledger wall470 s differs
  from provider uptime and is not substituted into the charge estimate.
  Creation-to-SSH was250 s, so provisioning alone exceeded the PM's approximate
  $0.05 expectation for the whole run; no additional dose or lease was added.
- Cumulative **18-lease episode estimate $2.230435278**, carrying forward the
  [17-lease $2.112395278 total](https://github.com/UOR-Foundation/uor-r4/pull/2542#issuecomment-6215981349).
  Storage/transfer dollar charges **UNMEASURED**, not assumed free. Local and
  cloud archive bytes are disclosed; this is not an energy or provider invoice.
- Release/delete completed and fleet **0 pods, $0/h** was verified.
  [cost.json](cost.json) carries the ledger events and arithmetic basis.

The result was posted on [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6217213855).
This receipt PR requests independent exact-head review and protected delivery;
it does not authorize its separately preregistered next diagnostic.

## Resource-process deviation and remaining hold

The pre-lease `df` check showed approximately28 GiB free. Codex failed to enforce
the repository's30 GiB disk-floor preflight stop; the later harness check
explicitly surfaced the warning after execution and teardown. This is a
[reported process deviation](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6217327847),
not a waived rule or a retrospective authorization. It is separate from the
passing measurement/zero-update invariants and must remain visible in review.
No further compute runs. Only this lab's idle, regenerable
`local/propose-subphase-target` cache was cleaned (Cargo reported6547 files,
919.6 MiB); free space rose to approximately29 GiB, still below the floor.
Unique recovery material and peer branches/worktrees were not touched. PM must
coordinate remaining disk-floor restoration before any subsequent lease;
global hygiene was not applied indiscriminately to other labs' workspaces.
