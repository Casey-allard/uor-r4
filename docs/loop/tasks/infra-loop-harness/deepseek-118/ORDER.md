# ORDER: infra-loop-harness, deepseek-118 (mandatory artifact binding)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Base: `73f707d29`.
Setup row: 2 (record maintenance and model-contract tooling).

## Goal

Make contract and tokenizer binding mandatory for future FBS artifacts without
breaking the historical `/6` through `/9` artifacts already used as serving
evidence.

A new `uor-r4.lut-stack/10` artifact records the contract source hash, contract
semantic hash and tokenizer SHA-256. New exports require those inputs by
default. The product chat path refuses a `/10` artifact when the supplied
tokenizer does not match. Legacy exports remain reproducible only through an
explicit `--legacy-artifact` switch, and legacy serving remains supported.

## Cancel probe

At the base, provenance is optional source metadata, artifacts do not change
schema when it is present, and `uor-chat` does not compare the embedded
tokenizer hash. The unit stands.

## Legacy migration

Artifacts made under `/6` through `/9` remain readable and exactly
reproducible. Their contracts must declare `legacy-grandfathered` and name this
decision. New exports default to `/10` and require all three bindings.

## Files in scope

- `conformance/stack/**`
- `conformance/generated/**`
- `crates/repo-model/src/stack.rs`
- `crates/uor-r4-integer/src/bin/fbs-export.rs`
- `crates/uor-r4-integer/src/bin/uor-chat.rs`
- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `docs/model/STACK-CONTRACT.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-118/**`
- `xtask/src/main.rs`
- `xtask/src/stack_audit.rs`
- `xtask/src/stack_contract.rs`

## Done when

1. Schema `/10` exists and parser validation requires all three bindings.
2. `fbs-export` defaults to `/10` and requires contract source, contract
   semantic hash and tokenizer identity.
3. `fbs-export --legacy-artifact` reproduces historical `/6` through `/9`
   bytes and refuses a mixed request with binding flags.
4. `uor-chat` hashes the supplied tokenizer and refuses a mismatched `/10`
   artifact before generation; legacy artifacts still serve.
5. `stack-audit` treats `/10` tokenizer and contract bindings as required,
   while reporting them `NOT_RUN` only for legacy artifacts.
6. Contracts explicitly mark whether their artifact is binding-required or
   legacy-grandfathered. Current and candidate contracts may remain legacy only
   through that explicit designation.
7. Tests cover `/10` round trip, missing/malformed provenance, tokenizer
   mismatch, mandatory flags, legacy reproduction, and no regression in `/6`
   through `/9`.
8. Real arm-r and arm-rt legacy audits retain their prior results.
9. Gates, exact-head review, CI, merge and fresh-main verification pass.

## Stop and report

Stop if the change requires training, GPU compute, rewrites an existing
artifact, or changes geometric serving arithmetic. This unit changes artifact
identity and admission only.
