# Rung-0 pilot — owner release plan (D4 gate)

Date: 2026-10-14. Milestone: M1 #2029. PM: Kimi (kimi/pm1).

This directory is the non-authorizing release packet for the A2 pilot. It
contains no owner decision. Under **R15** (`docs/harness/pm-rules.md`), no lab
writes, edits, copies or generates `owner.json` or any stand-in; Casey alone
creates the release record, from his own terminal, when and only when he
chooses to release the pilot.

## Contents

- `PILOT_PLAN.json` — the `OwnerReleasePlan` (schema
  `uor-r4.pilot-owner-release-plan/1`, status `HELD_OWNER_ACTION_REQUIRED`).
  It binds the complete execution identity, the pilot `DriverConfig`, and the
  SHA-pinned `sheet.md` and `review.json` below.
- `pilot-config.json` — the same `DriverConfig` as a standalone file for the
  pod-side `pilot` command. Seed 1, batch 64, spec V4096/B4/C192/K8, schedule
  `canonical_vector_then_read_v1`; content pins match
  `validate_pilot_config` exactly (only the registered A2 seed-1 ascending
  VectorBlock pilot is admitted).
- `sheet.md` — byte copy of the rung-0 training sheet
  (`docs/harness/prep/m1-discrete-rung0-training-2026-10-13.md`) at merge
  `c1f920ca7` (FIT (design), Amendment 2), sha256
  `1029c3b92771e0938d2ec393ce419db81cdc17125dad4814243c235cfb7463d5`.
- `review.json` — the `ReviewRecord` binding Claude's exact-head closeout
  APPROVE on PR #2421
  (`cbeeee084d5563dd87cd0e8e4850b65d2ea540c0`, comment 6160178151), sha256
  `1e8563c4324ab0865e576b40ea51b377c80fbb1c533bb504d63f4d08000cd97b`.

## Execution identity (pinned)

| Field | Value |
| --- | --- |
| base_source | `e7d926fbe38537f4faa90b817705c61a780778d6` |
| reviewed_source_commit | `cbeeee084d5563dd87cd0e8e4850b65d2ea540c0` (Claude APPROVE; discrete crate byte-identical to qualified `35a48f5863`) |
| source_sha256 (CUDA) | `e18361495276e491525674fb3fdce308b0ad787c7fd6860a811adb60dacd9e1a` |
| kernel_sha256 (PTX) | `931a511bece20641ae00c6b0e943f4f2a380c0f35a824d85fcc0447f1e1ccf5b` |
| geometry_sha256 | `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327` |
| executable_sha256 | `d12877b6d8adc6a60445a014be682ab82459a781f760ac99a226b07335cbfcac` |
| driver_source_sha256 | `236e7fb6e5269e3bd4fed21d391d1ce82f7d9bd31275f1a0d97dd0c4ce9d9806` |
| config_sha256 | `6252b613c6bf84efeb0dbdcfde69877eb75da1ec6d868eaac3461d93d75a3d04` |
| lineage_sha256 | `6db5ecf5d6e1b334b6f788a28b741b7ff2e012e7bf451e5f89ed29f89ef82a12` |
| sheet_sha256 | `1029c3b92771e0938d2ec393ce419db81cdc17125dad4814243c235cfb7463d5` |

Source/executable/PTX/geometry/driver hashes are the pod-built pins from the
qualified admission run (#2029 comment 6160049539, sheet
`docs/harness/prep/m1-discrete-admission-controls-2026-10-14.md`; 25/25
controls PASS at `35a48f5863`, comment 6160106734). `config_sha256` and
`lineage_sha256` were reconstructed with serde declaration-order encoding and
the method was verified by reproducing the published seed-7 lineage hash
`2cebef05e321cb52654b9866900b874c079d73a81b4da4f27d28d72156d90f9c` exactly;
Codex is asked to cross-confirm both seed-1 values with one
`build-identity pilot-config.json` run on the CUDA binary before any pod
launch. Admission is fail-closed: a wrong pin makes the pilot refuse, never
misrun.

## What the pilot is (and is not)

Seed 1, exactly 1,640 batches, canonical ascending VectorBlock coordinates
0..1639 only (token IDs 0..409); Route sensitivity NOT_RUN; diagnostic only —
no bar or guard is decided, fresh transfer UNAVAILABLE/NOT_RUN, and the full
two-seed run is a separate later owner decision. Known-component cost subtotal
1.784167939 h ≈ $1.62 at $0.908/h, UNVERIFIED, excluding unmeasured overhead
(#2029 comment 6159198070). P1/P2 reports are produced (slice TRAIN-count share
and init collision metadata).

## How D4 is released (Casey only)

From his own terminal, with the owner checkout on a main that contains this
directory (`git -C ~/uor-r4 pull --ff-only` first), and a fresh empty output
path, e.g. `~/uor-r4-r4-release-20261014` — or any path that does not exist:

```sh
cd ~/uor-r4
env -u UOR_LAB ./<path-to>/discrete-stack owner-release \
  docs/labs/rung0-pilot-release-2026-10-14/PILOT_PLAN.json \
  NEW_RELEASE_DIR
```

(The binary path is the one Codex's pod build produced; Codex confirms the
exact on-pod path when the pilot pod is staged. Casey runs this on his own
machine; it starts no model computation.)

The command prints the pinned plan and requires typing exactly:
`I am Casey; release D4 for the 1640-batch diagnostic pilot only`

It writes `owner.json` and the admission manifest into `NEW_RELEASE_DIR` and
nothing else. Training and M2 remain HELD in every other respect; this releases
D4 for the A2 pilot only.
