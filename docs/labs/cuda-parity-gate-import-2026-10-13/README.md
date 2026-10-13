# CUDA parity compiles again: the missing `Gate` import

References #2029. Claude lab, session `claude/gatefix`, 13 October 2026. Owner direction:
"fix the Gate import too, fix everything you need to". Test-source repair only.

## Problem

#2260 (OP1) added `StackConfig::gate` and set `gate: Gate::Swiglu` in
`test_step7a_read_lineage_parity` without importing `Gate`
(`crates/uor-r4-training/tests/cuda_stack_ops_parity.rs:2445`). The test is built only
with `--features cuda`, which laptop and CI checks do not compile (the CUDA build script
needs the toolkit), so the error reached `main`. From `7567cf725` on, every pod bootstrap
on either provider reported **parity FAIL** within 3 s: two Vast pods on 13 October
(`7567cf725`, `19500bf22`) showed `error[E0433]: cannot find type Gate in this scope`.

## Change and check

One import (`Gate`) added to that test's `use` list. A Vast RTX 5090 bootstrap at
`c1d53c8038dcadf8862caa71aa602db995465522` built with CUDA and ran the parity test:
**43 passed, 0 failed, 3 ignored** (the last passing parity before #2260 ran 38; the
count includes the tests #2260/#2262 added). The same pod's HF store fetch downloaded all
six data files, confirming #2261's `HF_TOKEN_PATH` fix live. Pod archived and destroyed;
cost about $0.05.

## Limit

The parity count is read from the bootstrap summary line, which does not name the tests.
