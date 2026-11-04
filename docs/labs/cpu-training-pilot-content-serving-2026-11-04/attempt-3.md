# Attempt 3: superseded by control-target accounting

Attempt 3 ran implementation pin
`8194823789244096e2db317bb45954d5a7eeaf8f`. Its release builds, focused
integer and loopback tests, both public seeds, seed-1 repeat and prompt-ID CLI
smoke passed. Both seeds completed 103 prompts and 1,648 decisions with zero
token or decision mismatches, and the normalized repeat was byte-identical.

The attempt is instrument evidence, not the deciding run. The control tests
used the chat release target, which grew to **2,246,424,310 bytes** after the
two test suites, above the registered **2,147,483,648-byte** target ceiling.
Attempt 4 uses a separate 3 GiB control target, preserves this completed
evidence under `evidence-attempt-3/`, and rechecks all build and control target
sizes after the tests.
