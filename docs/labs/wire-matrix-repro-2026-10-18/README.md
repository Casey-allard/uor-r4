# Wire matrix reproducibility and claim correction  2026-10-18

The merged wire side exploration (`spiralcore-base/wire/`) had three evidence-integrity defects at
its edges, not in its transport code:

- three adapter leg scripts hardcoded `$HOME/uor-r4/target/release/objpeer`, so the legs only
  ran on the owner checkout;
- the matrix ran 18 legs, but the usability doc claimed `15/15` in one place and `13/13` in
  another (README and optimization closeout said 18/18);
- the doc cited `tools/verify_all.sh` and `tools/check_evidence.py` and reported
  `VERIFY_ALL_PASS` / `EVIDENCE_OK`, but neither tool exists in the folder.

## Change

- `tools/wire_adapter_smoke.sh`, `wire_adapter_hub_smoke.sh` and
  `wire_adapter_protocol_smoke.sh` resolve objpeer repo-relatively
  (`<repo>/target/release/objpeer`), still overridable with `OBJPEER`, and fail with the exact
  build command if it is missing.
- `tools/wire_interop_matrix.sh` now preflights objpeer (fails loudly instead of silently
  skipping adapter legs) and supports `--list` (18 leg names, `LEGS_COUNT=18`).
- `docs/WIRE-USABILITY.md`: matrix claim corrected to 18/18 with the measured date/head; the
  `VERIFY_ALL_PASS` / `EVIDENCE_OK` claims withdrawn (tools absent); reproduce section gives the
  objpeer build command and `--list`; no aggregate evidence tool is claimed.
- `docs/labs/spiralcore-wire-side-2026-10-18/README.md` and the current-state entry carry the same
  correction, so the historical record stays honest rather than silently rewritten.
- `README.md` and `docs/OPTIMIZATION-CLOSEOUT.md` already said 18/18 and are correct.

## Verification (host-only, no pods, no model compute)

    bash tools/wire_interop_matrix.sh --list      -> 18 names, LEGS_COUNT=18
    OBJPEER=<built objpeer> bash tools/wire_interop_matrix.sh
      -> WIRE_MATRIX_RESULT pass=18 fail=0 ; WIRE_INTEROP_MATRIX_PASS tests=18
         (2026-10-18, worktree of main 03d82fb7b; all 18 leg markers PASS:
          socket-layer, socket-routing, socket-tls, mux, mux-routing, interposer-macos,
          interposer-linux-mvm, hub-cli, rust-interop, rust-hub, rust-adapter, rust-mux,
          adapter-pair, adapter-hub, adapter-policy, adapter-metrics, adapter-idle, adapter-protocol)
    OBJPEER=/nonexistent bash tools/wire_interop_matrix.sh
      -> exit 1 with MATRIX PRECHECK FAIL and the cargo build command

## Limits

- The matrix is a host-side manual regression, not a CI job; it needs the object-store `objpeer` binary
  built (`cargo build --release -p uor-r4-object-store --bin objpeer`), Python 3, and (for the Linux
  interposer leg) an mvm microVM. A host without mvm fails that leg rather than skipping it.
- Per-leg logs live under `local/wire-matrix/` (gitignored); this record cites the markers and the
  measured aggregate, not a committed evidence bundle.
- No transport or model-path behaviour changed; wire remains a dormant candidate and any promotion is a
  PM/owner decision.
