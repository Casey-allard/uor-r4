# K16 qualification attempt — step-0 CUDA build blocker

**Disposition: BLOCKER_TO_FIX / NOT YET PROMOTED.** No model evaluation,
fixture execution, timing, training or `.apply` occurred. This is an instrument
build failure, not a negative result for the admission-rank mechanism (D22/D31).
Line: exact addressed memory, M1 qualification tooling; count 0/3 and suspended
M1 headline 37/232 unchanged.
Codex/Sol executed this attempt. Kimi PM dispatches the record's exact-head review.

## Authority and exact scope

- [Owner qualification release](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6201365250), [executor call](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6205480376), and [PM order ruling](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6205684341).
- [Runbook](../admission-rank-qualification-runbook-2026-10-15/README.md) and [D-Packet](../admission-redirect-packet-2026-10-15/README.md).
- Executed source: `3c651df97293ed4da4aa8f800c407ee351bbffff` (merged #2532). Its timing directory is byte-identical to approved `c6eda17465006c61b36203d66d9c973cd6629fd5`.
- Required order: step-0 CUDA build/clippy on both instruments (plus the discrete driver), then **G-Pre → C-R1..C-R6 → checkpoint-0-only 100-batch timing**. Stop at the first failed command. Q4 remains **NOT_EXECUTABLE**.
- Lease acknowledgment: [#2029 6206126001](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6206126001). Failure notification: [#2029 6206185197](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6206185197). The latter's `compiled_build_identity()` spelling is corrected here to the actual `device::compiled_identity()` function.

## Pod and reproducible build conditions

Vast pod `vast-56182561`, session `codex-k16-qual-20261016`, one RTX 5090
(32,607 MiB; driver 595.84), Czechia. CUDA 12.8 / NVCC V12.8.93;
`UOR_NVCC_ARCH=compute_120`; Rust 1.97.1 (`8bab26f4f`), Cargo 1.97.1
(`c980f4866`). Checkout was detached at the exact source above, with no tracked
source modifications. Only `evals/`, `jobs/`, and `pods/` were untracked.

The lease used `uor-pod up --provider vast --gpu 5090 --count 1 --hours 1`
with explicit lab/session/purpose/source/card and `--no-bootstrap` to avoid an
unrelated historical training-crate build. The Rust toolchain and pinned source
were installed explicitly; the reaper was started. The gate ran through
`uor-pod run ... --gpu 0 -- bash .../step0.sh` under the GPU lock. This was build
work on a GPU-capable pod, **not GPU model execution**.

All builds used `cargo build --release --locked --features cuda --manifest-path`.
The prepared next commands used `cargo clippy --locked --features cuda --all-targets`
for each manifest, with the fixture runner's required `-- -D warnings`.
Each command captured its own exit code and separate stdout/stderr transcript;
the first failure terminated the script, rather than being hidden by a pipe.

| Step | Manifest / scope | Observed result |
| --- | --- | --- |
| Discrete build | `crates/uor-r4-discrete/Cargo.toml` | PASS, exit 0, 44 s command interval |
| Timing build | `docs/labs/k16-timing-2026-10-15/instrument/Cargo.toml` | PASS, exit 0, 35 s command interval |
| Fixture build | `docs/labs/admission-rank-fixture-runner-2026-10-15/instrument/Cargo.toml` | **FAIL, exit 101**, 32 s command interval |
| All three clippy commands | Discrete + timing + fixtures | NOT_RUN; preceding build failed |
| G-Pre | Seeds 1/2, K8/K16 checkpoint 0 | NOT_RUN; admission ceiling not measured |
| C-R1 | Rank/admission correspondence | NOT_RUN |
| C-R2 | Occupied-cell multiplicity | NOT_RUN |
| C-R3 | Virtual candidate parity | NOT_RUN |
| C-R4 | Absent rivals / occupancy below K | NOT_RUN |
| C-R5 | Bounded arithmetic / distributions | NOT_RUN |
| C-R6 | Stored guard / untrained ceiling | NOT_RUN |
| 100-batch timing | Checkpoint 0, policies A/R | NOT_RUN; K16 rate **UNMEASURED** |
| Q4 timing leg | Historical checkpoint rebind dependency | **NOT_EXECUTABLE**; never attempted |

The successful builds emitted warnings; they are not clippy passes or runtime
qualification. The prepared `gates.sh` is archived as **unexecuted intent**, not
an execution receipt. No model checkpoint was initialized or loaded.

## Observed blocker and bounded next action

The CUDA-only fixture runner treats the JSON return of
`uor_r4_discrete::device::compiled_identity()` as a struct:

```text
error[E0609]: no field `kernel_sha256` on type `Value`
  src/cuda_aggregate.rs:93:31 and :96:22
error[E0609]: no field `source_sha256` on type `Value`
  src/main.rs:168:37
error[E0609]: no field `kernel_sha256` on type `Value`
  src/main.rs:169:37
error[E0609]: no field `geometry_sha256` on type `Value`
  src/main.rs:170:39
```

At the executed source, `device.rs:417` declares `compiled_identity() ->
serde_json::Value`. This confirms the compiler diagnosis without executing a
model. Host-only checks cannot compile these CUDA-gated accesses. The underlying
discrete and timing builds did compile on this pod; no inference about the
unreached clippy/runtime gates follows.

**Next:** Kimi's [halt ruling](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6206218569)
and [routing amendment](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6206222931)
assign the repair to **DeepSeek**, with Codex exact-head review. The repair must
carry a pod-side `cargo check --features cuda` transcript before review closes;
the new pre-merge CUDA check is not a waiver of execution-time build/clippy.
Fix these identity accesses with fail-closed identity handling and deliver by
protected merge. Re-enter at the full step-0 gate only after that reviewed
repair; do not skip a gate or silently patch/retry this attempt. Training-start
qualification is still unsatisfied. M2 and Stage B remain HELD.

## Data and artifact integrity

The existing packet tokenizer, train payload/mask and DEV payload/mask were
staged from the previously pinned corpus and SHA-256 verified on the pod.
No rows were evaluated or updated. Fresh-transfer data and packet pins did not
change. Exact input hashes are retained in the archive's
`data-file-sha256.txt`; the corpus itself is not committed here.

Successful build hashes (not qualified executable receipts):

| Binary | SHA-256 |
| --- | --- |
| `discrete-stack` | `c129793e9efec43204eeb846adb46d64cf1a702a72bbadcf9bfe67d8dcfa1885` |
| `k16-timing` | `7d0c644360a5b5d36dd20b34294fc2c6cc8c8e6a852ba2d8d804716fc588c1e7` |

## Sealed archive and cost

Archive sealed at **2026-10-16T01:49:17Z**, copied off the pod and verified against
all 18 entries in `SHA256SUMS`. The manifest SHA-256 is
`7daea68551073c820d63ffed4d2c9fc5dd6bcd38812305044116a9d7adc36c0e`.
It contains full build/failed-build/setup transcripts, command exits/timestamps,
the job log, exact scripts, source/status, data and binary hash manifests.

`cloud-store put codex` uploaded and downloaded the **97,280-byte** archive and
verified MD5 **`b199182f3df546841b7942d8ae986f92`** before release/teardown:

```text
icloud:UOR-R4/results/codex/codex-k16-qualification-20261016.tar
```

Restore using `cloud-store fetch codex-k16-qualification-20261016 DEST`.
The index is `icloud:UOR-R4/index/codex-k16-qualification-20261016.tsv`.
The existing iCloud allocation was used; no new storage plan was purchased.
Transfer/storage invoice charges were not independently measured and are not
represented as zero. Pod storage used the harness default 150 GB ephemeral
allocation, destroyed with the pod; no retained paid pod storage remains.

| Resource / event | Receipt |
| --- | --- |
| Created | 2026-10-16T01:42:46Z |
| SSH ready | 2026-10-16T01:43:50Z |
| Step-0 job started | 2026-10-16T01:45:50Z |
| Failed build ended | 2026-10-16T01:47:42Z |
| Lease released | 2026-10-16T01:50:07Z |
| Pod deleted | 2026-10-16T01:50:22Z; subsequent fleet status: 0 running pods |
| Create-to-delete wall time | 456 s (7 min 36 s) |
| Provider uptime at deletion | 447 s |
| Reported rental rate | $0.815/h (`dph_total`, rounded by uor-pod) |
| Rate × provider uptime estimate | $0.101196; wall-time equivalent $0.103233; not an invoice |
| Build command intervals | 111 s total; no model-compute seconds or corpus updates |

The automatic teardown archive additionally retained 120 KiB of pod result/job
bookkeeping locally. The primary sealed evidence was cloud-verified **before**
teardown. No second lease or retry was made. Resource disposition and machine-
readable phase results are summarized in [receipt.json](receipt.json).
