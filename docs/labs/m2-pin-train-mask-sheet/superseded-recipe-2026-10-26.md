## Steps

**Training and M2 evaluation remain `HELD` (`training_hold: true`, `rental_hold: true`); this sheet executes nothing.** When a discrete M2 run is dispatched after its pre-run review (CHARTER §5.1, D67), the exact discrete CLI steps (replacing the frozen Amendment 6 float `geometric-stack train` CLI, which must not be reused on the discrete path) are:

```sh
# 1. Build the discrete CUDA binaries ahead of time via nvcc (D24, D29: no NVRTC, no CPU model compute)
UOR_NVCC_ARCH=compute_120 CARGO_TARGET_DIR=/root/build/target-sm120 \
  cargo build --release --locked -p uor-r4-discrete --features cuda --bin stage-t-run --bin trunk-run

# 2. Step 0 finite GPU & data-binding controls (fails closed on any mask/token/count/digest mismatch)
/workspace/bin/stage-t-run step0 --data-root /workspace/uor-r4/local/driver/data --out /workspace/out/stage-t-step0.json
/workspace/bin/trunk-run step0 --data-root /workspace/uor-r4/local/driver/data --stage-t-root /workspace/stage-t --out /workspace/out/trunk-step0.json

# 3. Two-seed discrete BVG-K Stage T + Stage 3 TrunkAssemblyV1 + D11 integer export (Seeds 1 & 2, T16_fz & T16_rel)
for ARM in t16-frozen-zero-seed1 t16-frozen-zero-seed2 t16-relation-seed1 t16-relation-seed2; do
  /workspace/bin/stage-t-run train --config docs/labs/m1-bvg-k-stage-t-training/configs/${ARM}.json --out /workspace/stage-t/${ARM}
  /workspace/bin/trunk-run arm --name ${ARM} --data-root /workspace/uor-r4/local/driver/data --stage-t-root /workspace/stage-t --out /workspace/trunk/${ARM}
done
/workspace/bin/trunk-run grade --out-root /workspace/trunk --out /workspace/trunk/grade.json

# 4. Deterministic M2 grounded-reply evaluation on m2-reply512 and sealed m2-fresh256 (only after DEV bars pass across both seeds)
```

- Pinned driver config digests (`docs/labs/m1-bvg-k-stage-t-training/configs/`):
  - `t16-frozen-zero-seed1.json`: sha256 `77d61451125b04827821216e221d548885af188bad6d9d301234d126fefa3b7d`
  - `t16-frozen-zero-seed2.json`: sha256 `d169d4ef347f28f4a289d131c3664e56a40a62a29774bf87cf0313ab358592cf`
  - `t16-relation-seed1.json`: sha256 `ab41788a5e72a1f11370f875b090653e0d1b36be091393861b0e147688e9d746`
  - `t16-relation-seed2.json`: sha256 `a0bfa6c5fe7497d85d2a3d8a68d86157a8dcaa72a5d5b7eca193d904c480f263`

## TEST FITNESS line

```text
TEST FITNESS: FIT - M2 #2030 discrete-geometric-learning
sheet: docs/harness/prep/m2-pin-train-mask-sheet-discrete-2026-10-26.md
reviewer: antigravity (fresh-context non-author subagent d929c424-0c35-4319-9260-6c9e41913c06)
note: Four Amendment 6 count-reference pins (mw-balp/train/response_mask.u8 7512797 B sha256 b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3, expected_train_targets=2425925, frozen count-reference commits 2fd6358bbc52d2a202a872ad0954ebb12894606f and 9e904a83448c451f738d5c1d45224dfdf73eb84e, and current discrete CLI stage-t-run / trunk-run at c695ad21a72c5b91def22458eebbe796e07d5392 / 6f38dd393865a0b4320a2e28bee3be675130dc16), rung-0 and M2 bars (m2-reply512 >= 256/512, sealed m2-fresh256 pairs_both_complete_distinct >= 52/128 at HF commit fb8c5e4fcd3568cc59be8be3073151295b98069c), and D11 zero-gap integer export verified against origin/main; sheet runs nothing and releases no hold.
date: 2026-10-26
A FIT verdict clears compute for the configuration in that sheet only (D22).

Every mechanism pre-registration gets a TEST FITNESS review on its milestone before compute.
```
