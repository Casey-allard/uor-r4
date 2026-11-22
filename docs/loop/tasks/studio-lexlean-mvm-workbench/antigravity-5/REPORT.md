# REPORT for Qwen3-0.6B Foundry Webharness LoRA Addendum (`d90-foundry-lora-v2`)

Lab: antigravity. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md) (#3939).

## What did not work

- **Run 1 on `vast-62052088` (`~$0.23` spent, cleanly deleted; `0` pods running):**
  1. Initial 3-epoch `lr=2e-4` LoRA triggered the `val_loss` monotonic-rise stop rule (`1.1747 -> 1.1886 -> 1.2152`, while `train_loss` collapsed to `0.0068`/`0.0061`).
  2. One bounded 1-epoch `lr=1.5e-4` retry lifted held-out tool/build validity from `11.56%` (`37/320` baseline) to **`95.62%`** (`306/320`, passing Bar #2 >= `90%`), while held-out Chat scored `66/80` (`82.50%`, below Bar #3 >= `95%`): **`60/60` (`100%`)** across 6 covered Chat topics passed, whereas all `14/14` Chat misses traced to an SFT generator defect where `TRAIN_CHAT_ANSWERS` fell back to an 18-token boilerplate string on underrepresented topics (inducing `token_bounds:18` and missing keywords). All `14` minor tool misses were `prismpm_inspect` (`3`) and `lexlean_check` (`9`) phrasing ambiguity plus `2` `standards_lookup`.
  3. WAT audit on compiled `lexlean_r4_wasm.wasm` verified **`37/37` `HoloV13Model` neural compute functions execute `0` float opcodes** and `holo_geometric_runtime.rs` has `0` non-comment `f32`/`f64` code lines, while flagging one `f64.convert_i32_u` opcode (`WAT:915934`) in `generate_chat_turn_with_history` immediately feeding `wasm_bindgen` `wbg_cast` -> `JsValue` -> `js_sys::Function::call2` for the JS streaming callback step index.

## What changed

- **[ORDER.md](ORDER.md):** Scoped addendum PR inside `docs/loop/tasks/studio-lexlean-mvm-workbench/antigravity-5/**` only.
- **[PREP.md](PREP.md):** Documented Run 1 negatives/diagnostics verbatim, pinned updated SHA-256 digests for `generate_foundry_sft.py` (`7a108f49...`, `24` dedicated `45–85`-word Chat topics removing the 18-token fallback + `lexlean_check`/`prismpm_verify` phrasing disambiguation while keeping `val.jsonl` `8ae0ec5c...` bit-identical), `train_foundry_lora.py` (`1cb4f257...`, `epochs=1`, `lr=1.2e-4`, `dropout=0.10`, `wd=0.05`), and `eval_foundry_v13.py` (`2d7bb71d...`, fail-closed `compute_float_ops == 0` check explicitly isolating verified `wbg_cast`/`JsValue`/`js_sys` ABI casts).

## Measurements

- Local self-test of updated Stage A scripts (`python3 local/antigravity/d90-foundry-lora/eval_foundry_v13.py --self-test`): generated `6,400` train (`sha256: 68991b24ab14b6be0aa9ed4f1a3b956238fb23f996fc8ace76b78fde02bda3a6`) and identical `400`-row held-out `val.jsonl` (`sha256: 8ae0ec5ceddfa0e188af0236a301cdb77c977d068897ce1c632d29c438eab252`), `train_val_overlap_hashes = 0`, `320/320` (`100.0%`) gold tool/build, and `80/80` (`100.0%`) gold Chat.

## Checks run at the head

- Non-author Charter §4.5 / D22 pre-registration fitness review (`f0919869-68ac-4962-b626-8e4cd6d65e90` at `f139cc2c4478c14255a9a6e8ff26bbaaa6cc013a`): `FIT_VERDICT: FIT`, `VERDICT: APPROVE`.
- `uor-harness check --staged`: `0 rules, 0 failed (0 error, 0 warn)`.
- `cargo run -q --locked -p xtask -- links`: `0` broken links in `antigravity-5/{ORDER.md,PREP.md,REPORT.md}`.
- `cargo run -q --locked -p uor-r4-claim-wording -- .`: passed.
- `uor-harness ratchet d23` / `d25` / `d32`: passed (`4451` / `93` / `31`, `0` added against baseline).
- `uor-harness loop pr-check --base origin/main --head HEAD --local`: `verdict: PASS` (`PASS order`, `PASS scope`, `PASS commit-order`, `PASS report`, `PASS owner-paths`, `PASS rule-sources`).

## Skills used

- None outside standard harness / git workflow.

## Still open

- Confirm final exact-head approval, post `[antigravity] uor-review v1`, merge PR #4178 via `uor-harness loop queue`, then execute Run 2 (`d90-foundry-lora-v2`) on a single-GPU Vast.ai pod under `PREP.md`.

## Plain summary

Recorded Run 1 GPU empirical diagnostics (`11.56% -> 95.62%` tool gain, isolated 18-token Chat boilerplate SFT bug on 2 underrepresented topics, and `37/37` `0`-float-op WAT compute proof separating the `js_sys::Function::call2` `JsValue` cast) and pre-registered the updated Run 2 addendum (`PREP.md`, `d90-foundry-lora-v2`) with pinned script SHA-256s and identical held-out `val.jsonl` before launching Pod #2.
