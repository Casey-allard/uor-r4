# Reproduce the archived continuation audit

This audit joins preserved inputs and GPU outputs. The base analysis needs Bash, jq, shasum, tar, and a checkout containing the codex-7 receipt files. It runs on the packaged integer table captures and needs no GPU, D11 model binary, or tokenizer executable.

## Inputs

Put these three canonical private-store archives in one directory under the exact file names below. Repository links are pinned to immutable store commits, and `stage.sh` checks each complete archive against `ARCHIVES.sha256` before reading its members.

- [span-connectors-wick.tar.gz](https://huggingface.co/datasets/caseyallard/uor-r4-store/blob/a46df86d18c1182375000d0f9ca11fd0edc797d7/results/codex/span-connectors-20261118/span-connectors-wick.tar.gz) supplies primary/prior grades and frozen word classes.
- [value-connectors-wick.tar.gz](https://huggingface.co/datasets/caseyallard/uor-r4-store/blob/58b98f50068eafbfe97bd8f89508757a0d63a49b/results/codex/value-connectors-20261118/value-connectors-wick.tar.gz) supplies fresh grades and primary/fresh/prior value decisions.
- [continuation-eos-wick.tar.gz](https://huggingface.co/datasets/caseyallard/uor-r4-store/blob/b05051bc96a52394f77a4e21818f39482caaa104/results/codex/continuation-eos-20261118/continuation-eos-wick.tar.gz) supplies transfer grades/value decisions and both cold runs of CUDA continuation and writer outputs.

Obtain the retained 178 KiB `history-custody.tar.gz` from the #4104 evidence delivery. It holds the two seed oracle selections/grades, the pre-registered cold adjudication receipts, the original #4104 blind tape/grade, and exact frozen root/class/orbit captures. This bundle currently resides under the owned archive scratch path and has no store upload from this audit. Use `CUSTODY.sha256` to verify the complete bundle and `SOURCE-PINS.sha256` to verify each member. The audit refuses missing inputs and altered receipts.

Keep these files together in an analyst directory: `*.jq`, `stage.sh`, `reproduce.sh`, `ARCHIVES.sha256`, `SOURCE-PINS.sha256`, `CUSTODY.sha256`, `table.rs`, and `orbit.rs`. Include `MODEL-PINS.sha256` for optional recapture.

## Run from a new directory

Choose absolute paths. Create the analyst directory and copy the files into it. Leave the staged lab directory, analyst `source/`, and audit output directory unused. Set `checkout` to a checkout containing the original codex-7 evidence. The retrospective source check used revision `2a72175c75c7f68f29c3100ea833a3b319216cdc`.

```sh
bash "$analyst/stage.sh" "$analyst" "$staged_lab" "$archives" "$custody_tar"
bash "$analyst/reproduce.sh" "$analyst" "$staged_lab" "$checkout" "$output"
```

The outputs are `RESULT.json`, per-seed geometry and adjudication JSON, the signed-root comparison, and complete `INPUTS.sha256` and `ANALYSIS.sha256` lists. The original per-row join files preserve original IDs/endpoint metadata. The report table and interpretation are in `../REPORT.md`.

## Optional D11 table recapture

To independently regenerate instead of trusting the retained read-only tables, restore the original D11 seed 1 and seed 2 assets named by the frozen codex-5 sheet and codex-12 writer pack receipts. On Wick they are `~/uor-codex/20261118-value-connectors/inputs/seed1.d11` and `seed2.d11`. Given those assets elsewhere, put them under those two base names in an asset directory and validate:

```sh
(cd "$assets" && shasum -a 256 -c "$analyst/MODEL-PINS.sha256")
CARGO_TARGET_DIR="$build" cargo build --manifest-path "$checkout/Cargo.toml" \
  --release --locked -p uor-r4-integer --lib
for reader in table orbit; do
  rustc --edition 2021 -O -L "dependency=$build/release/deps" \
    --extern "uor_r4_integer=$build/release/libuor_r4_integer.rlib" \
    -o "$build/$reader" "$analyst/$reader.rs"
done
for seed in seed1 seed2; do
  "$build/table" "$assets/$seed.d11" | cmp - "$analyst/source/tables/$seed-roots.json"
  jq -e '.bank.classes' "$staged_lab/span-connectors-20261118/results/span-01/$seed-state.json" \
    | cmp - "$analyst/source/tables/$seed-classes.json"
done
"$build/orbit" | cmp - "$analyst/source/tables/orbit.json"
```

These Rust readers only inspect the saved bytes. The replay reads no issued credentials, generates no answers, and does no fitting.

The #4104 custody bundle has a digest-verified Wick copy at `~/uor-codex/20261118-ordered-role-transfer/history-custody/history-custody.tar.gz`. The workstation copy is `local/codex/ordered-role-transfer-20261118/next-boundary/history/history-custody.tar.gz` under the main checkout. The approved private store's quota prevented a new remote revision. In the staged replay, paths in INPUTS.sha256 refer to the host that recorded them. Compare digests, normalized to the corresponding filenames in the new output tree.
