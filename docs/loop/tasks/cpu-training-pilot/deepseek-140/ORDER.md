# ORDER: cpu-training-pilot, deepseek-140 (content readout serving sidecar)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `70f45367d`.

## Goal

Serve the positive modulo-256 content readout through the integer product path
without changing the `UORLUT01` container. A separately hashed sidecar binds
the readout to the exact artifact and overrides the native/point fallback only
when an exact Hamming row falls within its learned threshold.

This is the serving counterpart to
[deepseek-137](../deepseek-137/REPORT.md). It does not revive either failed
stop configuration from deepseek-138 or deepseek-139.

## Fixed construction

Use sidecar schema `uor-r4.stack-content-readout/1`:

```json
{
  "schema": "uor-r4.stack-content-readout/1",
  "artifact_sha256": "<64 lowercase hex>",
  "readout": {
    "mode": "code",
    "signature_bits": 2048,
    "includes_reference_end": false,
    "rows": [{"signature": [32 u64s], "target": 1}],
    "max_distance": 0
  }
}
```

The loader must reject a wrong schema or artifact, sign mode, reference-end
rows, BOS target 0, wrong width, malformed threshold, empty rows,
out-of-vocabulary targets, unknown fields, or trailing/oversized input.

`IntegerStackModel::set_content_readout` attaches one validated readout to an
FBS model. `IntegerStackSession::serve_step` retains the native writer and
point-key fallback, then selects the readout target exactly when the nearest
row is within `max_distance`. All content decisions use:

1. minimum 2,048-bit XOR/popcount distance;
2. commonest target among equally near rows;
3. smaller token id on a target-count tie; and
4. exact fallback beyond the threshold.

The artifact and all existing `step`/`step_native` arithmetic remain
unchanged. A loadable sidecar must be exposed through `uor-chat
--content-readout PATH` for one-shot, REPL, prompt-id and OpenAI-loopback
serving. JSON outputs must identify the sidecar path and SHA-256.

Session snapshots bind the canonical attached-readout hash. Restore succeeds
only when source and destination carry the same readout; otherwise it refuses
before changing state. The integer `UR4SESS2` codec carries this identity while
legacy `UR4SESS1` snapshots remain readable when both sides have no readout.

The CPU content driver must emit this sidecar beside `code-true-readout.json`
and verify complete generated replies against an integer
`IntegerStackModel` carrying the same sidecar.

## Arms and bar

Run the existing 98/103 public split for seed 1, seed 2 and a seed-1 repeat.
The sidecar is written from the positive code readout only.

Both seeds must pass:

- the sidecar binds the exact public artifact and has at least 640 rows, at
  least 16 targets and `max_distance <= 2048`;
- no-readout `serve_step`, `step` and `step_native` outputs remain byte-for-byte
  equal to the current main implementation;
- every complete public evaluation reply matches the CPU code arm decision for
  decision: chosen token, native/point fallback, packed signature, nearest
  distance and whether the readout fired;
- each seed checks **103 prompts** and at least one readout override;
- every malformed/foreign sidecar production is refused before a step;
- same-readout session restore succeeds in the synthetic no-turn-memory FBS
  control while a missing or different readout is refused before state
  mutation; the public turn-memory artifact keeps its existing restore
  refusal; and
- the seed-1 normalized report and sidecar match the repeat byte-for-byte.

Report sidecar hash, row/target counts, threshold, decisions, readout fires,
fallbacks, first mismatch, complete-reply IDs/stop counts, workers, wall and
RSS.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `crates/uor-r4-integer/src/stack/content_readout.rs`
- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/session_codec.rs`
- `crates/uor-r4-integer/tests/session_codec_interop_fixture.rs`
- `crates/uor-r4-integer/tests/openai_loopback.rs`
- `crates/uor-r4-integer/src/stack/chat.rs`
- `crates/uor-r4-integer/src/bin/uor-chat.rs`
- `crates/uor-r4-integer/src/openai.rs`
- `crates/uor-r4-integer/README.md`
- `crates/uor-r4-integer/Cargo.toml`
- `docs/harness/claims.json`
- `docs/harness/prep/cpu-training-pilot-content-serving-2026-11-04.md`
- `docs/integration/current-state.md`
- `docs/labs/cpu-training-pilot-content-serving-2026-11-04/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-140/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

No CUDA, float-training or Claude-owned source.

## Done when

1. Strict sidecar types, selection, model attachment and all serving CLI paths
   are implemented without changing the artifact format.
2. Unit tests cover parser refusals, tie rules, fallback, unchanged raw paths
   and actual `serve_step` override behavior.
3. Public two-seed CPU/serving parity and its repeat pass under a FIT sheet and
   fresh GO.
4. REPORT and hashed evidence classify serving readout behavior; notebook and
   roadmap record the remaining language/termination gap.
5. Exact-head review, protected merge and fresh-main verification complete.

## Stop and ask if

- Exact parity needs a `UORLUT01` format or Hamming-reader change.
- The sidecar would alter a prompt or non-generated token decision.
- A public prompt lacks enough initialization to compare the full reply.
- The work requires CUDA, float-training or Claude-owned files.
