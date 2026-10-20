# D22 numerical source snapshots, 10 October 2026

This inactive source archive preserves all 20 immutable snapshots from the bound inventory, including compile/setup failures and superseded variants. It is preservation evidence, not a model result, a numerical admission certificate, or activation of any source. Replay15 was still running and unqualified when these historical labels were recorded; consult the integrated result for its terminal state.

`patches/00-synthetic-attempt1.patch` creates the first complete source tree from empty. Apply subsequent numbered binary Git patches in order. Each delta preserves the entire archived file set, including harnesses, documentation and source manifests, not only solver.rs. `manifest.json` binds each original compressed archive SHA256 and size, original inventory tree identities, per-file size/SHA256/mode, explicit directory entries, and each patch SHA256. Gzip/tar headers and timestamps are not reproduced; restored file bytes and modes are. All 20 restorations were independently compared directly against the original tar members. None of these 20 snapshots contains an empty regular file; the full file-set comparison includes zero-length files if present.

Use Python 3 and Git, with a destination that does not exist and whose parent already exists:

```sh
python3 restore.py replay-attempt15-actual-rhs-error-type /absolute/owned/local/restored-numerical-source
```

A snapshot name or zero-based index selects the endpoint. The script applies every predecessor and checks every intermediate file set and SHA256, fails on an existing output directory, and clears inherited `GIT_*` variables and isolates Git discovery from any surrounding checkout. Restoration does not compile or activate code. The source fixtures and replay need separate resource and scientific admission before execution.

`verification.json` records direct byte comparisons; `original-inventory.json` preserves the supplied inventory. `status-source-readme.md` captures the numerical owner's existing report at preservation time. `preparation-setup-failure.json` preserves an initial archive-verifier path-context failure; it changed no tracked source and was repaired before successful reconstruction. Statuses distinguish numerical no-assignment, compile/setup failure, test failure, scoped qualification and a pending replay; none is reclassified as a model negative.

This package is staged for review and promotion within the single integrated D22 result PR. No separate preparation PR, current-source replacement or INDEX mutation is performed here.

The revised restoration boundary was checked with `GIT_DIR`, `GIT_WORK_TREE`, index/common/object directories and Git configuration variables aimed at a disposable decoy repository. All 20 snapshots restored in the requested destination; every decoy file and mode remained unchanged. See `restore-environment-check.json`. The earlier package is explicitly superseded for this restoration-tool repair; source patches and historical identity manifests are unchanged.
