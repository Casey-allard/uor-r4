# ORDER: studio-conversational-composer, deepseek-3 (correct unrun checks)

Lab: deepseek. Item: [studio-conversational-composer](../../../../roadmap/items/studio-conversational-composer.md) (#3608).
Base: `c8b6606b8e4d16cb8c5099a8b16d0249bdd7f18d`.

## Goal

Repair the fresh-context CHANGES review of PR #3621 at
`77c0883f1a35646f9f2c7d4d46dc5935b8862e6a`.

The implementation and strengthened Node tests are accepted. Two reports still
say that Vitest, typecheck, Vite or a browser run remain a CI responsibility,
but no current workflow executes those tests. The roadmap also links the
superseded report instead of the active repair.

## Files in scope

- `docs/loop/tasks/studio-conversational-composer/deepseek-1/REPORT.md`
- `docs/loop/tasks/studio-conversational-composer/deepseek-2/**`
- `docs/loop/tasks/studio-conversational-composer/deepseek-3/**`
- `docs/roadmap/items/studio-conversational-composer.md`

## Done when

1. Every unavailable Studio/browser check is labeled `NOT_RUN`.
2. No report says an existing CI job covers those checks.
3. The roadmap links the active deepseek-3 report.
4. Report check, exact-diff check, retained evidence checksums, local PR check
   and full gates pass.
5. A fresh exact-head review approves the record-only repair.

## Stop and ask

Do not change production paste behavior, the accepted test, or claim an
unexecuted Studio/browser result.
