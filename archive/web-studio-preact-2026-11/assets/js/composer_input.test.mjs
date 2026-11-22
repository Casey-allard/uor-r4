import test from 'node:test';
import assert from 'node:assert/strict';
import { createComposerInput } from './composer_input.js';

function setup() {
    const textarea = new EventTarget();
    textarea.value = '';
    textarea.selectionStart = 0;
    textarea.selectionEnd = 0;
    textarea.setSelectionRange = (start, end) => {
        textarea.selectionStart = start;
        textarea.selectionEnd = end;
    };
    const files = [];
    const calls = { files: 0, sends: 0 };
    createComposerInput({
        textarea,
        onSend: () => { calls.sends += 1; },
        onFiles: (value) => {
            calls.files += 1;
            files.push(...value);
        },
    });
    return { textarea, files, calls };
}

function paste(textarea, files, text = '') {
    const event = new Event('paste', { bubbles: true, cancelable: true });
    Object.defineProperty(event, 'clipboardData', {
        value: {
            files,
            getData: (type) => type === 'text/plain' ? text : '',
        },
    });
    textarea.dispatchEvent(event);
    return event;
}

test('mixed clipboard paste attaches the file and leaves its text native', () => {
    const { textarea, files, calls } = setup();
    const file = { name: 'note.txt', type: 'text/plain', size: 8 };
    const event = paste(textarea, [file], 'visible text');
    assert.equal(event.defaultPrevented, false);
    assert.deepEqual(files, [file]);
    assert.equal(calls.files, 1);
    assert.equal(calls.sends, 0);
});

test('file-only paste remains prevented and attached without sending', () => {
    const { textarea, files, calls } = setup();
    const file = { name: 'note.txt', type: 'text/plain', size: 8 };
    assert.equal(paste(textarea, [file]).defaultPrevented, true);
    assert.deepEqual(files, [file]);
    assert.equal(calls.files, 1);
    assert.equal(calls.sends, 0);
});

test('text-only paste remains native without attachment or send callbacks', () => {
    const { textarea, files, calls } = setup();
    const textOnly = paste(textarea, [], 'text only');
    assert.equal(textOnly.defaultPrevented, false);
    assert.deepEqual(files, []);
    assert.equal(calls.files, 0);
    assert.equal(calls.sends, 0);
});
