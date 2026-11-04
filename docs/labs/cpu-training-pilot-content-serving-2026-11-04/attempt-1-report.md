# REPORT: content-readout serving sidecar, deepseek-140

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `70f45367d`.

## What did not work

- **Serving parity did not improve language output.** The content arm remained
  at **73/64** and **58/39** top-1/depth, produced **0** exact references and
  capped all 103 replies on both seeds.
- **The readout remains sparse.** It fired on 103/1,648 generated decisions on
  seed 1 and 113/1,648 on seed 2. The native or point-key fallback handled the
  other 1,545 and 1,535 decisions.
- **The serving artifact is now two files.** `UORLUT01` bytes remain unchanged
  by the readout, but a deployer must carry both `serving.uorlut` and its
  artifact-bound sidecar.
- **This is CPU/serving integration evidence, not GPU or useful-chat
  evidence.** It does not establish termination, fresh phrasing or semantic
  correctness.

## What worked

The positive modulo-256 content readout now has an exact integer serving path.
`IntegerStackModel::serve_step` obtains the normal native writer or point-key
fallback and then applies the sidecar's nearest-example target only inside its
training-fitted Hamming radius. Raw `step` and `step_native` do not consult the
readout.

The sidecar uses schema `uor-r4.stack-content-readout/1`, binds the complete
serving artifact SHA-256, and rejects wrong artifacts, sign-mode or reference-end
payloads, BOS targets, malformed widths and thresholds, unknown fields,
out-of-vocabulary targets and oversized input.

`uor-chat --content-readout PATH` attaches the table for one-shot, REPL,
prompt-ID and OpenAI-loopback serving. The model is configured before all four
paths consume it.

The CPU content driver now writes the training-derived turn-memory artifact
and sidecar, then checks every generated evaluation decision against an
independent `IntegerStackModel` carrying that sidecar.

## Measurements

Both seeds used the registered 98-training/103-evaluation split with 1,120 and
1,085 content decisions respectively.

| Public seed | Serving artifact SHA-256 | Rows | Targets | Radius | Chosen / fires / fallbacks |
|---:|---|---:|---:|---:|---:|
| 1 | `ed06a88b…d47daffc` | 894 | 207 | 537 | 1,648 / 103 / 1,545 |
| 2 | `2c071845…4eaf7e842` | 894 | 207 | 533 | 1,648 / 113 / 1,535 |

Decision-for-decision parity:

| Public seed | Prompts | Decisions | Token mismatches | Decision mismatches | First mismatch | CPU = serving digest |
|---:|---:|---:|---:|---:|---|---|
| 1 | 103 | 1,648 | **0** | **0** | none | `54a91121…f89eed93` |
| 2 | 103 | 1,648 | **0** | **0** | none | `f40ea8b7…ad56a813` |

The compared fields were chosen token, native/point fallback, complete
2,048-bit signature, nearest distance, readout-fired state, complete reply
tokens and stop class.

Sidecars:

- seed 1: `e60a909e14a8fa2ab310a9a6255fa995920f51202471936661077c8f7ea290e7`;
- seed 2: `20534cdd3c4710b77e3df26d18da33edc55810269bbda3358f10cc1426d4fdd6`.

The one-shot CLI smoke loaded the seed-1 `serving.uorlut` and sidecar, recorded
both hashes, and generated `[2564, 1, 1, 1]` over four capped tokens.

Build and runs:

- driver build: **19 s**, peak RSS **1,196,384 KiB**, target
  **266,899,497 bytes**;
- `uor-chat` build: **12 s**, peak RSS **1,214,656 KiB**, target
  **104,633,028 bytes**;
- walls: **3.22/3.18/3.21 s** at 32 Rayon workers for seed 1, seed 2 and the
  seed-1 repeat;
- run RSS maxima: **228,092/227,624/228,676 KiB**;
- driver and chat binary SHA-256:
  `804f5a6f0a42ec8b386d0a41c0b6527318ba8273c3f422346804052160ba4c5b`
  and
  `043c169ab2285011fac121969e54a0717042be5ae7d924b919f6ea529394a686`;
- committed evidence: **49,493,960 bytes**;
- evidence `SHA256SUMS`:
  `409283d3eadb555666fe9eb0e20b593de215b67cdda0adbccdbd38ae13118895`.

Compact evidence:
[comparison.json](../../../../labs/cpu-training-pilot-content-serving-2026-11-04/evidence/comparison.json).
The source pipeline occupied **49,487,494 bytes**.

## Checks run at the head

- Full `uor-r4-cpu-train` release suite: **246 passed, 4 expected ignores**,
  with all binary tests green.
- Full `uor-r4-integer` serving library suite: **459 passed, 1 expected
  ignore**.
- Focused tests cover sidecar strictness, BOS refusal, voting and fallback,
  raw-step isolation and complete CPU/serving parity.
- Strict `uor-r4-cpu-train` Clippy, rustfmt and `git diff --check` passed.
- The integer crate's blanket all-target Clippy still reports pre-existing
  warnings-as-errors outside this unit; all-target compilation and the full
  serving library tests pass. Filtered diagnostics found no new warning in the
  new content-readout module.
- Both evidence manifests, the normalized repeat, generated JSON and
  `serving.uorlut`/sidecar files matched.
- Claim wording, docs catalog, roadmap schema/rendering, prep-check, D23 and
  D25 passed. The full harness check exited 0 with no errors and 159
  repository-wide warnings, with no warning attached to this unit.

## Skills used

No system-provided Codex skill applies. Independent FIT and GO reviews used
separate subagents.

## Still open

- Complete replies and a geometric termination rule.
- Fresh-phrasing and semantic-correctness panels.
- GPU parity and throughput for sidecar serving.
- Claude's P3 decode-cache unit and later end-to-end model evaluation.

## Plain summary

The geometric content table can now travel with an exact local model and drive
the real integer serving selector with byte-for-byte agreement to the CPU
trajectory. This removes the serving-integration gap for the positive content
readout. It does not solve completion or produce useful chat: the same sparse
content decisions and capped, inexact replies remain.
