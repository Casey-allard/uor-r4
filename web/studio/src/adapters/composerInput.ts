const HISTORY_LIMIT = 100;

export interface ComposerInputController {
    recordSubmitted(prompt: string): void;
    reset(nextDraft?: string): void;
    destroy(): void;
}

export interface ComposerInputOptions {
    textarea: HTMLTextAreaElement;
    picker?: HTMLInputElement | null;
    onSend?: () => unknown;
    onFiles?: (files: File[]) => void;
    getHistory?: () => string[];
    onHistoryChange?: (history: string[]) => void;
    onDraftChange?: (draft: string) => void;
    initialDraft?: string;
    interceptKeyDown?: (event: KeyboardEvent) => boolean;
}

// @ts-expect-error The existing public text-attachment reader is plain JavaScript.
export { readTextFiles } from '../../assets/js/composer_input.js';

export function createComposerInput({
    textarea,
    picker,
    onSend,
    onFiles,
    getHistory,
    onHistoryChange,
    onDraftChange,
    initialDraft = '',
    interceptKeyDown,
}: ComposerInputOptions = {} as ComposerInputOptions): ComposerInputController {
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

    function setValue(value: string): void {
        settingValue = true;
        textarea.value = value;
        textarea.dispatchEvent(new Event('input', { bubbles: true }));
        textarea.setSelectionRange(value.length, value.length);
        settingValue = false;
    }

    function recordSubmitted(prompt: string): void {
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

    function older(): void {
        const history = submittedHistory;
        if (!history.length) return;
        if (historyIndex >= history.length) draft = textarea.value;
        historyIndex = Math.max(0, Math.min(historyIndex, history.length) - 1);
        setValue(history[historyIndex]);
    }

    function newer(): void {
        const history = submittedHistory;
        if (historyIndex >= history.length) return;
        historyIndex += 1;
        setValue(historyIndex === history.length ? draft : history[historyIndex]);
    }

    function firstLine(): boolean {
        if (textarea.selectionStart !== textarea.selectionEnd) return false;
        return !textarea.value.slice(0, textarea.selectionStart).includes('\n');
    }

    function lastLine(): boolean {
        if (textarea.selectionStart !== textarea.selectionEnd) return false;
        return !textarea.value.slice(textarea.selectionStart).includes('\n');
    }

    function keydown(event: KeyboardEvent): void {
        if (interceptKeyDown?.(event)) return;
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

    function input(): void {
        if (settingValue || historyIndex < submittedHistory.length) return;
        draft = textarea.value;
        draftChanged(draft);
    }

    function paste(event: ClipboardEvent): void {
        const clipboard = event.clipboardData;
        const files = Array.from(clipboard?.files || []);
        if (!files.length) return;
        const text = typeof clipboard?.getData === 'function' ? clipboard.getData('text/plain') : '';
        if (!text) event.preventDefault();
        filesCallback(files);
    }

    function drop(event: DragEvent): void {
        const files = Array.from(event.dataTransfer?.files || []);
        if (!files.length) return;
        event.preventDefault();
        filesCallback(files);
    }

    function picked(event: Event): void {
        const target = event.target as HTMLInputElement;
        filesCallback(Array.from(target.files || []));
        target.value = '';
    }

    function reset(nextDraft = ''): void {
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
