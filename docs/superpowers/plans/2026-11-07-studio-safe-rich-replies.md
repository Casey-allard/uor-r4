# Safe rich replies implementation plan

## Context

`AgentComposer` currently renders message strings as one plain `<p>`. Studio Harness needs readable technical replies, but parsing into HTML strings would create an injection boundary. This unit adds a typed development-library renderer using only Preact nodes, then integrates it without changing raw message actions or the live page.

## Approach

Implement a small block parser for headings, lists, quotes, fenced code, and paragraphs, plus an inline parser for code, strong text, and Markdown links. Only HTTP(S) links become anchors, with `target="_blank"` and `rel="noopener noreferrer"`. All other input, including raw HTML and unsafe schemes, remains literal text. No sanitizer dependency or `dangerouslySetInnerHTML` is needed because Preact creates every node and escapes every text value.

## Steps

1. Commit the ORDER alone, then this plan. Install the pinned Studio packages with `npm ci`.
2. Add `RichMessage.test.tsx` first for structure, inline formatting, safe links, raw-HTML/unsafe-protocol refusal, unclosed/unmatched input, Unicode and long content. Add one `AgentComposer` integration case proving rich display and exact raw copy/insert/save/retry payloads. Run them against the missing component and retain the failure log.
3. Implement `RichMessage.tsx` with bounded line-by-line parsing and stable keys. Preserve text exactly within the chosen structures; do not execute, fetch, highlight, or transform code contents. Render unclosed fences safely to the end of input and unmatched tokens as text.
4. Replace only the typed transcript paragraph with `<RichMessage content={message.content} />`; keep role/action logic untouched. Export the component and add restrained code/link/list styling to the existing typed stylesheet.
5. Rerun focused tests, then `npm test`, `npm run typecheck`, `npm run build`, `node --test assets/js/*.test.mjs`, and all four legacy checks. Keep Cargo output under `target/kimi-lexlean`.
6. Update the typed-library README, product note, notebook, and REPORT with exact counts, red-first evidence, head/tree, and remaining limits. Run claim/report/local-policy/full gates and review the scoped diff.
7. Commit, push a draft PR with `References #3500`, obtain a fresh exact-head delivery review, repair if needed, then use the protected queue and fresh-main verification. Preserve evidence and clean only this worktree's generated outputs. Do not publish or start another unit in this session.

## Assumptions

- Common technical-reply structures are enough for this unit; tables, syntax highlighting, math rendering, raw HTML, images, and custom Markdown extensions remain plain text or later work.
- Only HTTP(S) links are interactive. Relative, protocol-relative, `mailto:`, `javascript:`, `data:`, and `vbscript:` destinations stay inert in this unit.
- Raw HTML is intentionally displayed, not sanitized into active markup. Copy/insert/download/retry always use the unmodified source string.
- This is typed-component evidence only. Full browser/Tauri parity, migration, cutover, real-browser accessibility acceptance, and publication remain under #3500.

## Verification contract

The report must include missing-component red-first evidence, the green focused/full counts, security negatives, exact delivery head/tree, and explicit non-goals. A component test is not a live-page, browser-parity, accessibility-certification, or model-capability claim.
