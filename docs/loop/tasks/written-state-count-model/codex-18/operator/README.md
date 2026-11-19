# Finite selector and ordinary client replay

The four packed policies use the same ordered exemplar roots, radii, sealed bank and frozen D11 parent. The profile selects the operator. The study-stage build and all four executables are named by [SHA256SUMS](SHA256SUMS), whose paths are relative to `~/uor-codex/20261119-factorial` on Wick.

On the staged RTX 4090 executable, the Rust selector and CUDA matched all 68 raw words per decision across 84 synthetic decisions. Each fixture uses a low original address, a high original address in the 192-token context, and a no-fact row. The table gives the first row of each fixture. [Fixture source](../../codex-17/native/src/factorial.rs)

| Fixture | Ordered | Ranked | Veto | Both |
| --- | --- | --- | --- | --- |
| Ranking, with both starts admitted | Source 5 | Source 4 | Source 5 | Source 4 |
| Known rival outside its radius | Unseen | Unseen | Source 4 | Source 4 |
| Ranking reversal and a third outside start | Unseen | Unseen | Source 5 | Source 4 |
| Missing radius | Unseen | Unseen | Unseen | Unseen |
| Missing class | Unseen | Unseen | Unseen | Unseen |
| Exact positive ratio tie | Tie | Tie | Tie | Tie |
| Zero top supported margin with an outside rival | Unseen | Unseen | Nonpositive | Nonpositive |

The independently built `operator-01` produced the same selector log, packets, banks and traces as the staged build. It also replayed six historical Codex 17 trace sets through the updated ordered, neutral and position implementations. All 6,150 decisions matched the sealed trace bytes and the Rust selector. The staged CUDA executable contains SM89 SASS with no embedded PTX and passes the build script's scan for floating-point and matrix instructions.

The staged packer produced four distinct D11 hashes on the public-client fixture, byte-identical to the independently packed artifacts. Each ordinary protocol-2 client invocation selected original source 8 and emitted `[924,1074,1]` through EOS. The raw copy-step words and prompt IDs matched across the four invocations. These are finite positive-control replays.

## Reproduce on Wick

Use a fresh build root with the same source snapshot and [stage build command](../run/build-wick.sh), or verify and reuse the recorded binaries. A fresh output directory is required.

```bash
cd "$HOME/uor-codex/20261119-factorial"
sha256sum --check build-receipt/executables.sha256
(
  flock 9
  test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
  bin/all-starts-native factorial-finite "$PWD/bin/native-sm89" "$PWD/finite/selector-repeat"
) 9>>"$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock" > finite/selector-repeat.log
cmp finite/selector.log finite/selector-repeat.log
while read -r hash path; do
  printf '%s  %s\n' "$hash" "${path/finite\/selector\//finite\/selector-repeat\/}"
done < finite/selector-artifacts.sha256 | sha256sum --check
```

The staged client replay uses the same synthetic parent, tokenizer, span state, planted bank and profile as the isolated integration test. The checksum command verifies the three historical fixture inputs. The mode-specific `*.witness.json` files in the manifest retain the prompt IDs, selected fact, query and source, original generated IDs, copy-step words and stop.

```bash
prior="$HOME/uor-codex/20261119-fresh-boundary-validity/onset-public-integration/current-binaries-calibration-02/seed1"
sha256sum --check "$prior/inputs.sha256"
carrier="$HOME/uor-codex/20261118-ordered-role-transfer/fresh-role-trace/run-02/seed1-fixtures/favor_correct.d11"
tokenizer="$HOME/uor-codex/20261118-ordered-role-transfer/assets/tokenizer-seed1.json"
span="$HOME/uor-codex/20261118-span-connectors/results/span-01/seed1-state.json"
bank=operator-01/public-finite-01/known-right.bank
text='Mira kept blue green. Nora kept red. Please give what Mira collected before closing, rather than Nora beside the entrance.'
mkdir finite/public-repeat
for mode in ordered ranked veto both; do
  out="finite/public-repeat/$mode"
  bin/pack-all-starts "$carrier" "$span" "$bank" "finite/public/$mode.profile.json" "$out.d11"
  cmp "$out.d11" "finite/public/$mode.d11"
  bin/uor-chat --discrete "$out.d11" --tokenizer "$tokenizer" --protocol 2 \
    --temperature 0 --max-tokens 16 --read-mode enabled --say "$text" |
    jq -S -c '{prompt_ids, selected, generated_ids, copy_step_words, stop}' > "$out.witness.json"
  cmp "$out.witness.json" "finite/public/$mode.witness.json"
done
```

## GPU compatibility and post-transfer replay

The exact executable set needs an x86-64 Linux userspace with glibc 2.39 or later, a usable host `libcuda.so.1`, and an NVIDIA compute capability 8.9 GPU. The staged `native-sm89` executable has two SM89 cubins and no PTX. The frozen `chain` and `action-writer` Rust hosts embed SM89 cubins directly in their host data sections, outside the CUDA fatbin format. The standalone agreement and fence devices contain SM89 cubins and PTX. The post-transfer replay disables PTX JIT for every process.

An RTX 4090, RTX 4080, RTX 6000 Ada, L4, L40 or L40S has the required instruction set. Match the actual GPU identity and free memory before replay. An RTX 5090, H100, RTX 3090 or A100 has a different compute capability from the required target. Moving to one of those GPUs would require ahead-of-time recompilation of the selector and the writer and chain cubins, target-native code for the agreement and fence devices, new executable and embedded-cubin pins, and a fresh finite qualification and dispatch review. [NVIDIA CUDA compatibility guide](https://docs.nvidia.com/cuda/ada-compatibility-guide/) · [NVIDIA compute capability table](https://developer.nvidia.com/cuda/gpus)

Transfer the files in [VAST-FINITE-INPUTS.sha256](VAST-FINITE-INPUTS.sha256) into a separate root with the same relative layout. Pass its absolute path and a fresh absolute output path to [vast-finite.sh](vast-finite.sh) on the leased GPU. The script verifies the exact endpoint, selector, writer and client inputs before running. It replays the synthetic selector rows, tests the frozen chain and direct action writer on one previously exposed prompt with two continuations, and compares four packed artifacts and ordinary client witnesses. The transferred fixture set uses no Codex 18 held text or grading data.

The Vast run passed in 37.62 seconds on RTX 4090 `GPU-f9afc849-5fb5-d7c6-c8d5-3154e9a2bd25`, driver 580.95.05 and glibc 2.39. The 84 selector decisions matched Wick bit-for-bit. The full chain reproduced the archived receipt and 11 raw writer transitions. Direct action-writer replay matched both chain rows, including source 7 emitting `[1481,1]`. The four packed D11 hashes and ordinary client witnesses matched Wick. The pod receipt at `/workspace/vast-finite-extra/run-01/qualification.sha256` verified with SHA-256 `d7bf151b290324cfd336de544ee899193df0b969c1637e099c409ea3ce8fce7e`. Its output manifest has SHA-256 `6aba0a071a02cf6f59886322cf7adfff97e4d0fbe122571456a064166e855cf4`.
