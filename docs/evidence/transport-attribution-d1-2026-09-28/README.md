# D1 packet: transport attribution at matched MLP width

2026-09-28 · Claude lab track (cloud) · Note: [transport-attribution-d1-2026-09-28.md](../../integration/transport-attribution-d1-2026-09-28.md) · References #820, #973

This packet holds the run records of D1, pre-registered in the whole-project synthesis §4. The question is whether the `rrarra` core's quaternion transport is load-bearing at matched MLP width.

## Contents

| Path | What |
|---|---|
| `<arm>/attempt.json`, `report.json`, `manifest.json`, `model/config.json` | Each arm's final report root: claim record, report (settings, curve, final metrics, samples, input and executable identities), seal manifest (BLAKE3), model configuration |
| `<arm>/train.log` | The training log |
| `sources/run-d1.sh` | The runner: arms, settings, and the idempotent resume logic |
| `chain.log` | Launch and exit times of every arm |
| `packet.json` | The runs, their curves and identities, the pre-registered decision computed from the final scores, and the SHA-256 and size of every file here |

## Runs

| Root | Transport | Seed | Final NLL (512 windows) | Model SHA-256 |
|---|---|---:|---:|---|
| `rot_s1/` | quaternion | 1 | 2.599845 | `898893924c4f9b6a49b31f7d18eed8ef38d246971505227544a0b712292aa4a6` |
| `id_s1/` | identity | 1 | 2.670985 | `879da99addeaebbc111b74af79fb3883306772e93f4d1646e68b274681a22679` |
| `rot_s2/` | quaternion | 2 | 2.580265 | `deb422e442cd6274ca71605f4e22639be73c3196f699ef5471f788531e607c25` |
| `id_s2/` | identity | 2 | 2.657687 | `59d5d3c30e912bf383dd99006ecc4408e25f6d5158b5ec24ccc9a528856e02b7` |

**Decision, computed in `packet.json`:** keep quaternion transport.
- Identity − quaternion: +0.071140 (seed 1) and +0.077422 (seed 2).
- The threshold is 0.02 in both seeds.

## Identities

All four arms share the same executable and inputs.
- **Source:** `4eef03e6` (the `stack_mlp=` option; merged in #1437).
- **Executable:** `geometric-stack`, built with `-C target-cpu=x86-64-v3`, SHA-256 `c90c7346aa9920b82fcd82b27dbc27f0bb59515bb90f58933a5e0603b39f38aa`.
- **Data:** the cycle-4 repository code split and lab BPE.
  - `train.u16` `b12707b012e5447f0c613236ea7f8819f2cc123861dafe49f7f8226ccc457137`
  - `valid.u16` `3f7c50ef98fa707f4523aa3f7c6a97f496ee184fcfa125a84979fed9e7b91cb8`
  - `lens.u16` `edc0a8d6bcabf8c2a789113a71ebdd4ce6a3dd53de08e77aff13c56bc7c2e61f`
  - `merges.txt` `7453bfa086a18d36948d23fd060d2a74bc3ba8d87882d457cb397396dde84405`
- **Host:** the lab sandbox's Cascade Lake Xeon at 2.80 GHz with 4 cores, two threads per arm. The matrix library chooses its blocking from the host's caches, so a rerun on another CPU follows a different floating-point path.
- **Weights:** they stay in the lab sandbox (`scratchpad/d1/runs/<arm>/model/`), pinned by the SHA-256 above. Each is 27.3–28.6 MB.
