# Codex's closed PR #3169, kept as a patch (26 October 2026)

Branch `codex/d63-finite-probe-scope`, draft PR #3169, `infra-loop-harness` unit codex-1. It was closed unmerged at 2026-10-26T21:36:57Z, at Casey's instruction in claude's chat at about 21:34Z:

> "Yes: close codex's PR #3169 with a note, and open a PR removing codex from docs/harness/labs.json, quoting this instruction in its ORDER."

Casey had written before that: "codex is not currently working with the rest of the groups, it is working independently on lexlean-r4 so it should not be included." The closing note is on #3169.

| Patch | Base commit | Restored tracked tree | Disposition |
|---|---|---|---|
| [three-commit patch](codex-d63-finite-probe-scope-20261026.patch) | `71d9247a87ba45588ea00a583d5321ae2a3d6082` | `8f0f86882ae254f21b548836ab85dd955f74e77c` | Closed unmerged; kept as history. Nothing in it is applied. |

**The patch.**
- SHA256 `0eae76bb98c0f49be180f5f23cd65f947b473ffe64bbbc6f90d96a27d5b7f631`.
- It is `git format-patch` output for the PR's three commits, all from 2026-10-24 between 23:50 and 23:53Z:
  - `212ab50385863704c8a4fcf7a116bb7cfcade352`: record D63 documentation clarification scope;
  - `84ed36aac6cd7c0d331a050cc9d570192d5a069b`: qualify finite arithmetic probe evidence;
  - `52bab590e3b75ba433a3f605570f85aa9a87aacc`: record D63 documentation checks.
- Applied with `git am` to the base in an isolated index, it reproduces the PR's head tree exactly.

**What it held** (docs only, +90/−1):
- a D75 row in `docs/integration/DECISIONS.md`;
- one cell of `PLAN.md`;
- the unit's ORDER, PLAN and REPORT under `docs/loop/tasks/infra-loop-harness/codex-1/`.

None of it reached `main`, and none is applied by keeping it here. Its last commit was 2026-10-24T23:53Z. Its open ORDER held the `infra-loop-harness` claim until the close.

Archived by claude-138 (`infra-loop-harness`, #2904), the unit that takes codex out of the lab registry.
