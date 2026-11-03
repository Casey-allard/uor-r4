# ORDER: studio-conversational-composer, deepseek-2 (repair paste-test evidence)

Lab: deepseek. Item: [studio-conversational-composer](../../../../roadmap/items/studio-conversational-composer.md) (#3608).
Base: `c8b6606b8e4d16cb8c5099a8b16d0249bdd7f18d`.

## Goal

Repair the fresh-context CHANGES review of PR #3621 at
`66b84750b6748cb4007f4ca8eaa98fe37fa13fc5`.

The production mixed-paste behavior was approved, but the test used a files
array that could not prove a text-only paste made no callback, and it did not
observe send calls. The environment note also attributed the full Node run's
`crypto` failure to Node itself even though the CLI exposes `crypto` and the
test-runner child requires the experimental global flag.

## Files in scope

- `web/studio/assets/js/composer_input.test.mjs`
- `docs/loop/tasks/studio-conversational-composer/deepseek-1/**`
- `docs/loop/tasks/studio-conversational-composer/deepseek-2/**`
- `docs/roadmap/items/studio-conversational-composer.md`

## Done when

1. The test uses invocation counters for attachment and send callbacks.
2. Mixed and file-only paste each call the attachment callback once and send
   zero times.
3. Text-only paste calls neither callback and remains native.
4. The report distinguishes the CLI WebCrypto global from the test-runner
   process and records the explicit experimental flag that fixes it.
5. Focused tests, broader Node tests, syntax checks, evidence checksums and
   exact-head repository checks pass.
6. A fresh exact-head review approves the repaired head.

## Stop and ask

Do not change production paste behavior, attachment validation, send behavior
or any claim about an unexecuted browser run.
