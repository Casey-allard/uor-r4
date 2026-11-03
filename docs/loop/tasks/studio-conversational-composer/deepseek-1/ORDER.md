# ORDER: studio-conversational-composer, deepseek-1 (mixed clipboard paste)

Lab: deepseek. Item: [studio-conversational-composer](../../../../roadmap/items/studio-conversational-composer.md) (#3608).
Base: `6729e5d507bd39ef400c6515bc29b75f97e682d7`.

## Goal

Preserve ordinary pasted text when the clipboard event also contains a
supported text/source file. The file is still offered to the attachment
callback, and the browser inserts the text normally at the caret.

## Cancel probe

At the base, `composer_input.js` calls `preventDefault()` whenever a paste
event contains any file. A clipboard event that carries both text and a file
therefore attaches the file but drops the text. The existing test covers file
paste and text-only paste separately, not their simultaneous case.

This is a focused repair to a done item. No model, API, provider, stored
credential, engine or image behavior changes.

## Files in scope

- `web/studio/assets/js/composer_input.js`
- `web/studio/assets/js/composer_input.test.mjs`
- `docs/loop/tasks/studio-conversational-composer/deepseek-1/**`
- `docs/roadmap/items/studio-conversational-composer.md`

## Done when

1. File-only paste is still prevented and attached.
2. Mixed file-plus-text paste attaches the file and does not prevent the text.
3. Text-only paste remains native and sends nothing.
4. Focused Node tests, Studio tests and exact-head repository checks pass.

## Stop if

- The browser cannot distinguish text from files in one paste event.
- The change sends a prompt implicitly, truncates text, or weakens attachment
  validation.
