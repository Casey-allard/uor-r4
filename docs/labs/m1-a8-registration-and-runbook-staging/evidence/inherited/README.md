# Stage 1 inherited-observation records

Sixteen `uor-r4.attention-audit-inherited-command/1` records, one per arm (`relation`, `frozen-zero`) and command name, assembled from pod 1's sealed tar `results/claude/a8-stage1-pod1-20261024-r2/a8s1-pod1.tar` in the `caseyallard/uor-r4-store` dataset (sha256 `01ac2067506abd9a79c9645775075e07137a36c93ea09ee763dedde37fcf0360`).
File names are `<arm>-<name>.json`.

- `execution` copies `stage1/build-identity-<arm>.json`; `base_source` is `ATTENTION_BASE_SOURCE` at `fd409fcdf3bc43d607e768bae6f8cf403aa75901`.
- Every path is the absolute pod path. Files under `/workspace/a8s1` are hashed from the tar; data files under `/workspace/uor-r4/local/driver/data` take their hash from `stage1/data-sha256.txt`.
- `inputs` are the files named in the argv, plus the data files (and, for the nonroot negative, the table) those configs name. `qualify-relation` reads no file, so its input is the arm's context config.
- `report` is the report file in the command's output folder; for the two `propose` probes it is the stdout JSON; for `negative_k16` it is null.
