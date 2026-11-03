const DEFAULT_MAX_FILE_BYTES = 1024 * 1024;
const DEFAULT_MAX_TOTAL_BYTES = 4 * 1024 * 1024;
const HISTORY_LIMIT = 100;
const TEXT_EXTENSIONS = new Set(['txt', 'md', 'markdown', 'rs', 'json', 'html', 'css', 'js', 'mjs', 'cjs', 'ts', 'tsx', 'toml', 'yml', 'yaml', 'py', 'sh', 'csv', 'xml', 'svg', 'sql', 'java', 'c', 'h', 'cpp', 'hpp', 'cs', 'go', 'rb', 'php', 'swift', 'kt', 'lua', 'r', 'tex', 'log']);
const TEXT_MIME_TYPES = new Set(['application/json', 'application/javascript', 'application/xml', 'application/x-sh', 'application/x-yaml', 'application/toml', 'application/sql']);

function extension(name) {
    const parts = String(name || '').toLowerCase().split('.');
    return parts.length > 1 ? parts[parts.length - 1] : '';
}

function isTextFile(file) {
    const type = String(file && file.type || '').toLowerCase();
    return type.startsWith('text/') || TEXT_MIME_TYPES.has(type) || TEXT_EXTENSIONS.has(extension(file && file.name));
}

function limit(value, fallback) {
    return Number.isFinite(Number(value)) && Number(value) > 0 ? Math.floor(Number(value)) : fallback;
}

export async function readTextFiles(files, options = {}) {
    const maxFileBytes = limit(options.maxFileBytes, DEFAULT_MAX_FILE_BYTES);
    const maxTotalBytes = limit(options.maxTotalBytes, DEFAULT_MAX_TOTAL_BYTES);
    const attachments = [];
    const errors = [];
    const retained = new Map();
    for (const entry of options.existingAttachments || []) {
        const size = Number.isFinite(Number(entry.size)) ? Number(entry.size) : String(entry.content || '').length;
        retained.set(String(entry.name || 'attachment.txt'), size);
    }
    let totalBytes = Array.from(retained.values()).reduce((sum, size) => sum + size, 0);
    for (const file of Array.from(files || [])) {
        const name = String(file && file.name || 'attachment.txt');
        const size = Number(file && file.size || 0);
        if (!isTextFile(file)) {
            errors.push(name + ' is not a supported text/source file; binary input is refused.');
            continue;
        }
        if (size > maxFileBytes) {
            errors.push(name + ' exceeds the ' + maxFileBytes + '-byte per-file limit (' + size + ' bytes).');
            continue;
        }
        const nextTotal = totalBytes - (retained.get(name) || 0) + size;
        if (nextTotal > maxTotalBytes) {
            errors.push(name + ' exceeds the total attachment limit (' + maxTotalBytes + ' bytes).');
            continue;
        }
        let content;
        try {
            content = await file.text();
        } catch (error) {
            errors.push(name + ' could not be read: ' + String(error && error.message || error));
            continue;
        }
        if (!content.length) {
            errors.push(name + ' is empty; no attachment was added.');
            continue;
        }
        if (content.includes('\0')) {
            errors.push(name + ' contains NUL bytes and is treated as binary, not text.');
            continue;
        }
        attachments.push({ name, kind: 'text', content, source: 'file', size });
        totalBytes = nextTotal;
        retained.set(name, size);
    }
    return { attachments, errors };
}

export function createComposerInput({ textarea, picker, onSend, onFiles, getHistory, onHistoryChange, onDraftChange, initialDraft = '' } = {}) {
    if (!textarea) throw new Error('A composer textarea is required.');
    const send = typeof onSend === 'function' ? onSend : () => {};
    const filesCallback = typeof onFiles === 'function' ? onFiles : () => {};
    const historyCallback = typeof getHistory === 'function' ? getHistory : () => [];
    const historyChanged = typeof onHistoryChange === 'function' ? onHistoryChange : () => {};
    const draftChanged = typeof onDraftChange === 'function' ? onDraftChange : () => {};
    let submittedHistory = historyCallback().slice();
    let historyIndex = submittedHistory.length;
    let draft = '';
    let settingValue = false;
    let destroyed = false;

    function setValue(value) {
        settingValue = true;
        textarea.value = value;
        textarea.dispatchEvent(new Event('input', { bubbles: true }));
        textarea.setSelectionRange(value.length, value.length);
        settingValue = false;
    }

    function recordSubmitted(prompt) {
        const raw = String(prompt == null ? '' : prompt);
        if (!raw.trim()) return;
        const history = submittedHistory.slice();
        history.push(raw);
        const bounded = history.slice(-HISTORY_LIMIT);
        submittedHistory = bounded;
        historyChanged(bounded);
        historyIndex = bounded.length;
        draft = '';
        draftChanged('');
    }

    function older() {
        const history = submittedHistory;
        if (!history.length) return;
        if (historyIndex >= history.length) draft = textarea.value;
        historyIndex = Math.max(0, Math.min(historyIndex, history.length) - 1);
        setValue(history[historyIndex]);
    }

    function newer() {
        const history = submittedHistory;
        if (historyIndex >= history.length) return;
        historyIndex += 1;
        setValue(historyIndex === history.length ? draft : history[historyIndex]);
    }

    function firstLine() {
        if (textarea.selectionStart !== textarea.selectionEnd) return false;
        return !textarea.value.slice(0, textarea.selectionStart).includes('\n');
    }

    function lastLine() {
        if (textarea.selectionStart !== textarea.selectionEnd) return false;
        return !textarea.value.slice(textarea.selectionStart).includes('\n');
    }

    function keydown(event) {
        if (event.key === 'Enter' && !event.shiftKey) {
            if (event.isComposing || event.keyCode === 229) return;
            event.preventDefault();
            send();
            historyIndex = submittedHistory.length;
            draft = '';
            return;
        }
        if (event.key === 'ArrowUp' && historyIndex > 0 && (historyIndex < submittedHistory.length || firstLine())) {
            event.preventDefault();
            older();
        } else if (event.key === 'ArrowDown' && historyIndex < submittedHistory.length && lastLine()) {
            event.preventDefault();
            newer();
        }
    }

    function input() {
        if (settingValue || historyIndex < submittedHistory.length) return;
        draft = textarea.value;
        draftChanged(draft);
    }

    function paste(event) {
        const files = Array.from(event.clipboardData && event.clipboardData.files || []);
        if (!files.length) return;
        event.preventDefault();
        filesCallback(files);
    }

    function drop(event) {
        const files = Array.from(event.dataTransfer && event.dataTransfer.files || []);
        if (!files.length) return;
        event.preventDefault();
        filesCallback(files);
    }

    function picked(event) {
        filesCallback(Array.from(event.target.files || []));
        event.target.value = '';
    }

    function reset(nextDraft = '') {
        submittedHistory = historyCallback().slice();
        historyIndex = submittedHistory.length;
        draft = String(nextDraft || '');
        setValue(draft);
    }

    textarea.addEventListener('keydown', keydown);
    textarea.addEventListener('input', input);
    textarea.addEventListener('paste', paste);
    textarea.addEventListener('drop', drop);
    if (picker) picker.addEventListener('change', picked);
    reset(initialDraft);

    return {
        recordSubmitted,
        reset,
        destroy() {
            if (destroyed) return;
            destroyed = true;
            textarea.removeEventListener('keydown', keydown);
            textarea.removeEventListener('input', input);
            textarea.removeEventListener('paste', paste);
            textarea.removeEventListener('drop', drop);
            if (picker) picker.removeEventListener('change', picked);
        },
    };
}
