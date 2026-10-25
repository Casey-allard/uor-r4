# ORDER: m1-position-mechanism, antigravity-62

Lab: antigravity. Item: [m1-position-mechanism](../../../../roadmap/items/m1-position-mechanism.md). Base: `dc5f409b0`.

## Goal

Execute the required next-unit steps on `m1-position-mechanism` (`#2843` comment `6391161531`, `PLAN.md` line 68):
1. Add GPU device-vs-host controls for each of position modes 1–3 (`PrefixHolonomyV1`, `EndpointControlV1`, `IdentityControlV1`) on a fixed small batch in `Flock` and `Relation`, plus a GPU `Device::from_snapshot` refusal control for a malformed step candidate `16` reaching `discrete_validate` (`status 10`).
2. Compact the per-thread position stack arrays in `discrete.cu` (`prefix[243]`, `transport_root[364]`, `transport[364]`) from 32-bit `U` (`3,884` bytes) to 8-bit `unsigned char` (`971` bytes), and measure the compiled `ptxas` local-stack footprint (`forward_local_bytes`, `counterfactual_local_bytes`, `evaluation_local_bytes`) and GPU forward/evaluate throughput (`tok/s`) on an RTX 5090 against pre-`#3111` (`5844d911c`) and `#3111` (`dc5f409b0`).

## Cancel probe (run first)

Ran on `dc5f409b0`:
- `command grep -rn "new_with_position\|PrefixHolonomyV1\|EndpointControlV1\|IdentityControlV1" crates/uor-r4-discrete/src/device.rs` shows `Device::new_with_position` is defined (`device.rs:862`) and called by `new_with_normalization` (`None`) and `from_snapshot`, with **0** GPU tests in `device.rs` exercising `PositionMode` `1..=3`.
- Inspecting `Device::from_snapshot` (`device.rs:1196-1233`) shows it passes `snapshot.position.clone()` into `Self::new_with_position`, which calls `config.validate(spec, read_spec)?` on the host before `model.kernels.validate` (`discrete_validate`) launches. Consequently, a deserialized `Snapshot` with step candidate `16` is rejected on the host and never exercises `if(i<s[1]*16 && position_steps[i]>=16)atomicExch(status,10u);` (`discrete.cu:514`) on the GPU.
- Inspecting `discrete.cu:180-215` shows `U prefix[MAX_POSITIONS]` (`972` B) and `U transport_root[MAX_NODES], transport[MAX_NODES]` (`2,912` B) use 4-byte words for values bounded by `< 64` and `< 16` (`3,884` B total).
- **Verdict:** **CONTINUE — instrument** (untested GPU position modes/validation and reducible per-thread local stack).

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/labs/m1-position-mechanism/**`
- `docs/research/notebook/m1-discrete-attention.md`
- `docs/loop/tasks/m1-position-mechanism/antigravity-62/**`

## Done when

1. `Device::from_snapshot` checks `position` shape/mode inventory on the host and uploads `position_steps` before launching `discrete_validate`, so a deserialized `Snapshot` with step candidate `16` is refused on the GPU with CUDA status `10` while `Device::new_with_position` continues to validate `PositionConfig` on the host.
2. GPU tests in `crates/uor-r4-discrete/src/device.rs` (`position_device_tests`) and a CUDA qualification binary `position-qualify` verify on an RTX 5090:
   - exact match between GPU traced actions, carried roots, and output vectors and the finite algebraic reference (`relative_holonomy`, `apply_frame_to_root`) across `PrefixHolonomyV1` (`s[14]=1`), `EndpointControlV1` (`s[14]=2`), and `IdentityControlV1` (`s[14]=3`) in both `Flock` and `Relation`;
   - interior-order sensitivity (`PrefixHolonomyV1` changes under an interior token swap while `EndpointControlV1` is invariant and `IdentityControlV1` matches an all-zero step table);
   - GPU `Device::from_snapshot` refusal (`status 10`) of a deserialized `Snapshot` with step candidate `16` and of `Route` with a position table;
   - `ptxas` local-stack bytes and GPU `tok/s` compared across pre-`#3111` (`5844d911c`), `#3111` (`dc5f409b0`), and compacted `discrete.cu`.
3. `uor-harness check`, `uor-harness loop gates --full`, and `uor-harness loop pr-check --local` pass at the exact head, and the D66 fresh-context review is recorded.

## Stop and ask if

- Any change would require touching a CHARTER §3 rule, owner path, or GPU cap (`<= 4` pods, `<= $8/h`).
