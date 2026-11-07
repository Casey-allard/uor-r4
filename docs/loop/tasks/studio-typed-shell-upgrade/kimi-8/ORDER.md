# ORDER: repair rich-message link and fence parsing

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `d0d7048f9066ae4786d98769121400edd48f98e3`. Prior reviewed head: `181bd625aceccf4b305bfc8e28164fbc321a3aef`, tree `3273eef6eef649ac03c0958ce0dbdb6eb27a14bf`, PR #3818 draft.

## Finished steps

- kimi-7 added the typed Preact-node rich renderer, integration, styles, and evidence; all 341 typed tests, 112 plain-JS tests, legacy checks, and 11 gates passed.
- Fresh review found no executable-HTML/XSS path but returned CHANGES for truncated/anchorized malformed links and punctuation/spaced fence tags turning code content into active Markdown. The report's broad malformed-link/fence claims require correction through code and tests.

## Goal

Preserve balanced parenthesized HTTP(S) destinations, keep unbalanced/malformed links literal, and recognize optional fence info strings with whitespace/punctuation while emitting only sanitized language classes. Preserve every kimi-7 security and raw-content behavior.

## Cancel probe

Add the review's exact failing cases first. Stop rather than accepting a URL prefix, anchorizing an unbalanced destination, or allowing fence contents to become headings/links/other active Markdown.

## Files in scope

- `web/studio/src/components/RichMessage.tsx` and `RichMessage.test.tsx`.
- `web/studio/README.md`, `docs/products/web-client.md`, and `docs/research/notebook/m4-serving.md` only for corrected counts/evidence.
- This repair folder.

No other paths are authorized. No live page/controller, offline vendor shim, dependency, engine, worker, tokenizer, `studio_api.js`, model artifact, publisher, deployment, `.github/**`, `Cargo.lock`, real browser data, owner-controlled path, or generated roadmap/issue edit.

## Done when

- A Wikipedia-style balanced URL retains its complete destination. Unbalanced/malformed destinations produce no anchor and remain literal. Existing unsafe-scheme/relative-link and raw-HTML negatives stay green.
- `c++` and spaced fence tags produce one code block with a sanitized language class, no active code headings, and ordinary following paragraphs. Unclosed fences remain safe.
- Focused/full Studio checks, plain-JS tests, legacy checks, claim/report/local-policy checks, and full gates pass at the new exact head. Existing disclosed D76 advisories remain disclosed; no new warning is introduced.
- Fresh exact-head review approves before ready/queue. PR keeps `References #3500`; no live cutover, publish, migration, useful-chat, M7/M8, production-readiness, or accessibility-certification claim.

## Stop and report conditions

Stop before changing files outside this scope or weakening a test/verifier. Stop after this repair and protected delivery; do not start another UI unit in this session.
