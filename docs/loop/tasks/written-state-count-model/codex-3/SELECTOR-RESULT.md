# [codex] Learned occurrence selection

The fitted selector missed its registered role-swap criterion at both released artifact seeds. On the 64 held-out synthetic questions (32 matched pairs) it found the asked value on eight individual questions and got both sides correct in zero of the 32 matched pairs. For 48 questions its selected position belonged to neither value. The two seeds use the same records, so the denominator remains 64 per seed.

| Frozen arm | d | q | TRAIN label hits / 128 | Held-out asked / 64 | Held-out distractor / 64 | Held-out neither / 64 | Both questions correct / 32 |
|---|---:|---:|---:|---:|---:|---:|---:|
| Asked-source fit | 2 | 13 | 48 | 8 | 8 | 48 | 0 |
| Swapped-source fit | 2 | 7 | 128 | 0 | 64 | 0 | 0 |
| No fit | 1 | 0 | — | 0 | 0 | 64 | 0 |

Both artifacts produced these same counts. Each fitted maximum was unique among the 192 coordinates. Swapping the TRAIN labels changed the fitted query offset and produced 64/64 exact distractor hits. Thus this set of relative offsets and root distances can recover one consistently placed entity, yet the asked-source fit did not reliably follow a change in the question. On a post hoc breakdown of the saved predictions, the first held-out wording yielded zero asked/other hits in 32 cases, and the second yielded 8/8 in 32 cases. No parameters were changed after outcomes.

The learned state has two bounded integer coordinates that control prediction. Each state receipt also carries 192 u32 support counts, the input digest, and the artifact digest for validation. All 480 held-out/fixed-panel source decisions (2 seeds × 3 arms × 80 prefixes) matched the independently implemented integer serving argmin exactly. The separate fixed 16-case writer comparison consumes the trained state's source once per prompt and is reported in the main unit report. This unit measures a bounded selector on constructed two-entity questions; chat, reasoning, general reading and ordinary-text BPB remain open.

## Reproduce

On Wick the private run root is `/home/callard/uor-codex/20261118-geometric-copy`. In its `src` checkout, build the standalone instrument using `UOR_NVCC_ARCH=compute_89`, a separate `CARGO_TARGET_DIR` and the CUDA toolkit in PATH.

```sh
cargo build --locked --target x86_64-unknown-linux-gnu \
  --manifest-path docs/labs/m1-d11-artifact-scorer/instrument/Cargo.toml \
  --features occurrence-selector --bin occurrence-selector
```

`occurrence-selector prepare TOKENIZER NEW_DIR` reconstructs the pinned 128/64 split. The independent sheet binds both raw tokenizers and all effective corpus files. The fixed 16 inference file is the JSON projection `jq 'map({prefix})' panel.json` of the codex-1 panel when it has the sheet's exact digest. For each of the two released models, these commands fit the grid on device, generate per-prefix GPU choices and grade held-out replay. Set the model, split, paths and arm variables to the named inputs; use new output locations on each run.

```sh
occurrence-selector fixture
occurrence-selector fit "$model" "$split/train.json" "$out"
for arm in trained swapped no_fit; do
  occurrence-selector predict "$model" "$out/$arm.json" "$split/heldout-prefixes.json" "$out/heldout-$arm.json"
  occurrence-selector predict "$model" "$out/$arm.json" "$fixed_prefixes" "$out/role-$arm.json"
  occurrence-selector grade "$split/heldout.json" "$out/heldout-$arm.json" "$out/grade-$arm.json"
done
```

The first instrument binary was `f3869f5ca25377cd199e90c847bf690fd5af17b7ddab6fac6fab709023ce793e`. Rebuilding after source formatting produced binary `5b0515bd3b8bda428b62194c0ffdf37527d6cb02b10da0462c8c54f620b039b7` and retained the identical sm_89 selector cubin `9847666304ae7e6eeb158f6c93abba6876a7f51c5e5c6aeb8e64b734962ef1b2`. The synthetic finite suite includes all 192 settings, three prefixes, earliest tie, empty domain, displaced source and swapped-label votes. A corrupted TRAIN manifest was refused before GPU fitting. The retained selector directory includes learned/control states, raw predictions, grades, kernel and executable identities, and a manifest covering the small receipts. The generated corpora are reproducible on Wick.

Both per-seed directories in the original `results/selector-run-01` compared byte-for-byte equal to their respective final replay directories in `results/selector-run-02` (13 files per seed). The two seed-bound trained-state SHA256 digests were `5981d5716bb9674646cb5292da5455abf349dea7367be682ef310974c88f1c07` and `c54dc669339bc22a57a8ee676141ec780b5ba5ea3185940de65a96811ca94944`.
