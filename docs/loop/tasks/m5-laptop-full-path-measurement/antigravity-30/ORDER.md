# ORDER: m5-laptop-full-path-measurement, antigravity-30

Lab: antigravity. Item: [m5-laptop-full-path-measurement](../../../../roadmap/items/m5-laptop-full-path-measurement.md). Base: `687d020b0`.

## Goal

Repair all 6 findings (plus the note 7 disclosure) from [Claude's re-review (`#3012` comment `6354784049` and `6354837170`)](https://github.com/UOR-Foundation/uor-r4/pull/3012#issuecomment-6354784049):
1. **Fix column-position mnemonic parsing so all-hex mnemonics (`fadd`, `fabd`, `fcadd`) are detected (`report_output.rs`):** Extract the mnemonic by column position after the address and raw encoding bytes (never via `.find(|tok| !tok.chars().all(|c| c.is_ascii_hexdigit()))`), and plant `fadd` alongside `mul` and `fmul` in `crates/uor-r4-integer/tests/m5_full_path_cost_controls.rs`.
2. **Walk the reachable call graph and require all 43 mandatory `uor_r4_integer::stack` symbols (`report_output.rs`):** When `require_full_call_graph` is true, require all 43 `STACK_MANDATORY_SYMBOLS` (`IntegerStackSession::step` + 13 `stack::session::stack_*` + 29 `stack::kernels::stack_*`, 0 missing) AND traverse the transitive call graph (`bl`/`b`/`call`/`jmp`) across reachable callees in `uor_r4_integer`, `core`, `alloc`, and `compiler_builtins` (`__multi3`, `__udivti3`, etc.) plus all `uor_r4_integer::stack` closure/scheduler instantiations; state the exact scope and the D23 ratchet-tracked legacy integer-table status of `stack_exp_neg` and `stack_sigmoid_q31` (`scripts/no-softmax-baseline.json`) in the receipt's `disassembly_audit`.
3. **Complete the forbidden-opcode set (`report_output.rs`):** Add AArch64 `fcvt*`, `frint*`, `smnegl`/`umnegl`, `<f...|bf...>.<lanes>`, and `s0..s31` float scalar loads/stores, plus x86-64 `div`, `mulx`, `mulss`/`mulsd`/`addss`/`addsd`, `cvtsi2sd`/`cvttsd2si`, `pmulld`/`pmuludq`/`pmaddwd`, and `vmulpd`/`vaddpd`.
4. **Remove the circular `(1/mean_s)*mean_s == 1` check in `uor-r4-stack cost` (`uor-r4-stack.rs`):** Require `--energy-receipt` in strict qualification mode (`!allow_provisional`) and set `work_rate_identity_verified` to `validated_energy_receipt.is_some()` (verified against the real physical identity $W = (\text{J/tok}) \times (\text{tok/s})$).
5. **Fix the CI95 validator (`report_output.rs`):** Never treat `[min, mean, max]` as samples, never fall back from `net` to `gross` on v3 metrics, and never silently skip `ci95` validation when `samples` and `(sd, n)` are absent (emit `InvalidSampleCount`); for the 3-run archived v2 `measurement_report_rnd.json`, reconstruct the 3 gross observations via exact sum algebra $x_2 = 3\bar{x} - x_{\min} - x_{\max}$ (`[3.135, 3.182, 3.199]` mJ) to diagnose the v2 $z$-vs-$t_2$ defect.
6. **Complete the task records (`antigravity-27..30`):** Record the `getrusage ru_maxrss` $\to$ `vmmap` / `VmHWM` (`#![forbid(unsafe_code)]`) ORDER note in `antigravity-27/ORDER.md` and `antigravity-27/REPORT.md`, and explicitly record the correction of `antigravity-28/ORDER.md:11` across `antigravity-28/ORDER.md`, `antigravity-29/REPORT.md`, and `antigravity-30/REPORT.md`.

## Cancel probe (run first)

Injecting `100001000:\t1e612800\tfadd\td0, d0, d1` into `IntegerStackSession::step` must fail `verify_step_disassembly_text` with an error naming `fadd`.

## Files in scope

- `crates/uor-r4-integer/**`
- `docs/compute/energy-measurement-protocol-v3.md`
- `docs/labs/m5-laptop-full-path-measurement/**`
- `docs/loop/tasks/m5-laptop-full-path-measurement/**`

## Done when

- `verify_step_disassembly_text` catches `fadd`, `mul`, `fmul`, `__multi3` / `__udivti3` reachable callees, and missing mandatory stack symbols.
- `cargo test -p uor-r4-integer --test m5_full_path_cost_controls` (without and with `--features serving-wasm`), `uor-harness check`, `uor-harness loop report --check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass.

## Stop and ask if

- Any file outside `scope` must be modified.
