# REPORT: resolve final identity and evidence-chain findings, kimi-81

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `3e33a175bcf928e1e42e7bbe8c7a149709095da3`, tree `fc3df19e7e556d5f3c43b385a0525d6184f96255`; merge base `0895aa65ff2f63ed184e78e81c28e67ffa3f34aa`. Fresh reviewer agent-64 returned CHANGES limited to evidence-chain contradictions and retention wording. The PR remains draft and unqueued.

## What did not work

- `kimi-79/REPORT.md` called d399 both earlier source/acceptance and record-only through 3c7176d5f, and called b975 both final and intermediate. `kimi-80/REPORT.md` claimed to bind a final identity that it did not contain.
- Active records then cited that report for a self-referential final hash. The final identity instead belongs in the PR body and durable identity.json, which are written after the actual commit is known.
- Final `identity.json`, `loopback-acceptance.json`, and `loopback-acceptance.png` exist in the main checkout. They were not all duplicated into the worktree; claiming both was inaccurate.
- The mixed-case RED=200 → GREEN=404 observation is retained in `kimi-78/REPORT.md`; no separate raw red log was retained. Do not present it as independently durable raw output. The regression and complete green logs are the retained executable evidence.
- At intermediate `f986f5a7c0682731db411f5fa0fc3832b482bdd4`, local policy and all **11/11** full gates passed. The first loopback invocation then refused to start because its expected binary had been removed by the earlier generated-target cleanup: `local/kimi/studio-final-acceptance/f986f5a7c0682731db411f5fa0fc3832b482bdd4/loopback.log` retains the ENOENT. Rebuilding that expected binary at the same exact source head succeeded. `loopback-success.log` passes at that head/tree, with 15 initial/12 reopened requests, zero external/raw-TypeScript/runtime errors, and exact session/file restoration. This setup failure is not called a product regression or hidden by the later pass.

## What changed

- `d39953a7d4ebab479673eba50e06481151b740bc` is earlier source/acceptance. `988155ae56dc0fdd872b7428018ead566179fe3f`, tree `e6542fb375518c1b38639cff825b9ac65f7b3594`, is the source/assertion head for complete default/GUI tests.
- `3c7176d5f`, `c17105e704e1ee9b02271062a13fc23747649604`, `b9750be653d451691694853b62d4a98b256f48f3`, and `3e33a175bcf928e1e42e7bbe8c7a149709095da3` are intermediate record-only heads. They retain their own evidence and are not the final identity.
- kimi-79, kimi-80, current-state, notebook, desktop and README references now use those same roles and disclose the raw RED retention limit. The final candidate's HEAD/tree is computed after this commit, then written into the PR body and main-checkout `local/kimi/studio-final-acceptance/<final-head>/identity.json`.
- This repair changes records and evidence only. No production code, test assertion, model/API/engine/worker/tokenizer, or new feature is added.

## Measurements and retained evidence

- `988155ae5` complete default **11/3/21/7/13/8** and GUI **13/3/25/8/13/9** logs are under `local/kimi/studio-final-acceptance/988155ae56dc0fdd872b7428018ead566179fe3f/`, byte-identically copied to the main checkout.
- `3e33a175` had local-policy PASS and **11/11** full-gate GREEN, with a clean loopback at its exact head/tree. Those are retained intermediate results, not the new candidate's gate result.
- The main checkout retains final-candidate identity, loopback JSON, screenshot and browser fixture as they are produced; log-only worktree copies are not represented as a complete duplicate evidence packet.
- Fresh policy/full gates and clean loopback will run at the candidate after this record commit. Their paths and actual HEAD/tree go into the PR body before a new fresh-context exact-head review.

## Checks run at the head

- New candidate: `uor-harness loop pr-check --local`, `uor-harness loop gates --full`, `node web/studio/verify_studio_loopback.mjs`, and `git diff --check` are the required next checks, not results inferred from 3e33.
- Source implementation is unchanged since 988155ae5; web/studio/README.md is the only subsequent change under the product trees.
- No source or capability claim is substituted for the evidence-chain repair.
- The first candidate invocation stopped at local policy because this REPORT lacked the required `## Skills used` heading. Full gates and loopback did not run in that invocation. The heading is added here; the failed log is retained at `56becabea49f2657af6f1fa7da83a73fdd9b600f/pr-check-local.log`.

## Skills used

Fresh-context adversarial review reception, exact source/tree identity binding, evidence preservation and limitation disclosure, contradictory-record repair, report-format validation, and verification before completion. No model or production implementation change was introduced.

## Still open

Fresh candidate gates/loopback, fresh exact-head approval, required GitHub checks, protected queue/merge and fresh-main verification. No production publication, signed release, full Cowork parity, useful-chat result, or fresh packaged-GUI acceptance is claimed.

## Plain summary

The intermediate identities are now explicit, and the PR plus main-checkout identity.json owns the final identity instead of a report pretending to know its own future hash. The raw RED log limitation is disclosed. Delivery remains blocked on the new candidate's checks and independent approval.
