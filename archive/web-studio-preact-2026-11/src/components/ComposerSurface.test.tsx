import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LifecycleStore } from '../state/lifecycle.js';
import type { GenerationAdapter } from '../adapters/generation.js';
import type { Attachment, SessionApi, StudioSettings, StudioUiApi, WorkspaceApi } from '../adapters/workflow.js';
import { AgentClient, type TaskSummary } from '../agent/client.js';
import { FixtureTransport } from '../agent/fixtureTransport.js';
import type { RunView, TaskView } from '../state/agent.js';
import { AgentComposer } from './AgentComposer.js';
import { StudioApp } from './StudioApp.js';
import { makeFiles, makeModelAdapter, makeSessions, makeSources, makeUi } from './studioTestFixtures.js';

vi.mock('@xterm/xterm', () => ({
    Terminal: class {
        options = {};
        open = vi.fn();
        write = vi.fn();
        onData = vi.fn();
        onResize = vi.fn();
        loadAddon = vi.fn();
        focus = vi.fn();
        dispose = vi.fn();
        resize = vi.fn();
    },
}));
vi.mock('@xterm/addon-fit', () => ({ FitAddon: class { fit = vi.fn(); } }));

let host: HTMLDivElement;
let createObjectURL: ReturnType<typeof vi.fn>;
let revokeObjectURL: ReturnType<typeof vi.fn>;
let urlSequence = 0;
const originalCreateObjectURL = Object.getOwnPropertyDescriptor(URL, 'createObjectURL');
const originalRevokeObjectURL = Object.getOwnPropertyDescriptor(URL, 'revokeObjectURL');

beforeEach(() => {
    host = document.createElement('div');
    document.body.appendChild(host);
    urlSequence = 0;
    createObjectURL = vi.fn(() => `blob:test/${++urlSequence}`);
    revokeObjectURL = vi.fn();
    Object.defineProperty(URL, 'createObjectURL', { configurable: true, writable: true, value: createObjectURL });
    Object.defineProperty(URL, 'revokeObjectURL', { configurable: true, writable: true, value: revokeObjectURL });
});

afterEach(() => {
    act(() => { render(null, host); });
    host.remove();
    vi.restoreAllMocks();
    if (originalCreateObjectURL) Object.defineProperty(URL, 'createObjectURL', originalCreateObjectURL);
    else Reflect.deleteProperty(URL, 'createObjectURL');
    if (originalRevokeObjectURL) Object.defineProperty(URL, 'revokeObjectURL', originalRevokeObjectURL);
    else Reflect.deleteProperty(URL, 'revokeObjectURL');
});

function sessions(): SessionApi {
    return {
        getMode: vi.fn((): 'wasm' | 'api' => 'api'), listSessions: vi.fn(() => []), activeSessionId: vi.fn(() => null),
        selectSession: vi.fn(() => true), newSession: vi.fn(() => true), renameSession: vi.fn(() => true),
        duplicateSession: vi.fn(() => null), exportSession: vi.fn(() => null), deleteSession: vi.fn(() => true),
        getConversationMessages: vi.fn(() => []), setComposerText: vi.fn(() => true), sendPrompt: vi.fn(() => 1), predict: vi.fn(() => 1), stopGeneration: vi.fn(),
    };
}

function workspace(active = true): WorkspaceApi {
    return {
        openFolder: vi.fn(async () => undefined), importFiles: vi.fn(), openDemo: vi.fn(), openFile: vi.fn(async () => undefined),
        newFile: vi.fn(), closeTab: vi.fn(), closeAll: vi.fn(), saveActive: vi.fn(async () => true), saveAsCopy: vi.fn(() => true),
        insertText: vi.fn(() => true), setActiveContent: vi.fn(),
        activeFile: vi.fn(() => active ? { path: 'notes.md', name: 'notes.md', content: 'context text', mode: 'local' } : null),
        workspaceName: vi.fn(() => 'project'), mode: vi.fn(() => 'local'), tabs: vi.fn(() => []),
        treeSummary: vi.fn(() => ({ mode: 'local', rootName: 'project', fileCount: 1, tabs: [], activePath: active ? 'notes.md' : null })),
    };
}

function ui() {
    const attachments: Attachment[] = [];
    const settings: StudioSettings = { theme: 'dark', reduceMotion: false, fontSize: 13, tabSize: 4, wordWrap: false };
    return {
        attachments,
        api: {
            setView: vi.fn(), toast: vi.fn(), getSettings: vi.fn(() => settings), updateSettings: vi.fn(),
            getAttachments: vi.fn(() => attachments), addAttachment: vi.fn((entry: Attachment) => {
                const index = attachments.findIndex((item) => item.name === entry.name);
                if (index >= 0) attachments.splice(index, 1);
                attachments.push(entry);
            }),
            removeAttachment: vi.fn((name: string) => { const index = attachments.findIndex((item) => item.name === name); if (index >= 0) attachments.splice(index, 1); }),
            clearAttachments: vi.fn(() => { attachments.splice(0); }), templatePrompt: vi.fn((kind: string) => `Prompt for ${kind}`),
            clearSessions: vi.fn(() => ({ cleared: 0, total: 0 })), clearEditor: vi.fn(), updateStatus: vi.fn(),
        } satisfies StudioUiApi,
    };
}

interface MountOptions {
    selected?: boolean;
    activeFile?: boolean;
    sendThrows?: boolean;
    agentClient?: AgentClient | null;
    agentTask?: TaskView | null;
    activeRun?: RunView | null;
}

function mount(options: MountOptions = {}) {
    const store = new LifecycleStore();
    const sessionApi = sessions();
    const fileApi = workspace(options.activeFile !== false);
    const uiApi = ui();
    const generation: GenerationAdapter = {
        send: vi.fn(() => {
            if (options.sendThrows) throw new Error('generation exploded');
            return true;
        }), stop: vi.fn(), dispose: vi.fn(),
    };
    if (options.selected !== false) store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
    const copy = vi.fn(async () => undefined);
    const download = vi.fn();
    const props = {
        store, sessions: sessionApi, files: fileApi, ui: uiApi.api, generation, copy, download,
        agentClient: options.agentClient ?? null,
        agentTask: options.agentTask ?? null,
        activeRun: options.activeRun ?? null,
    };
    act(() => { render(<AgentComposer {...props} />, host); });
    return { store, sessionApi, fileApi, uiApi, generation, copy, download };
}

function promptElement() {
    const input = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]');
    if (!input) throw new Error('Missing prompt textarea');
    return input;
}

function typePrompt(value: string, caret = value.length) {
    const input = promptElement();
    act(() => {
        input.value = value;
        input.setSelectionRange(caret, caret);
        input.dispatchEvent(new Event('input', { bubbles: true }));
    });
    return input;
}

function pressKey(textarea: HTMLTextAreaElement, key: string, init: { shiftKey?: boolean; isComposing?: boolean; keyCode?: number } = {}) {
    const event = new KeyboardEvent('keydown', { key, shiftKey: init.shiftKey, bubbles: true, cancelable: true });
    Object.defineProperties(event, {
        isComposing: { value: Boolean(init.isComposing) },
        keyCode: { value: init.keyCode || 0 },
    });
    textarea.dispatchEvent(event);
    return event;
}

function pasteFiles(textarea: HTMLTextAreaElement, files: File[], text = '') {
    const event = new Event('paste', { bubbles: true, cancelable: true });
    Object.defineProperty(event, 'clipboardData', { value: { files, getData: () => text } });
    textarea.dispatchEvent(event);
    return event;
}

function button(label: string) {
    const found = Array.from(host.querySelectorAll('button')).find((node) => node.textContent === label);
    if (!found) throw new Error(`Missing button ${label}`);
    return found;
}

function imageFile(name = 'pixel.png', bytes = [137, 80, 78, 71]) {
    return new File([new Uint8Array(bytes)], name, { type: 'image/png' });
}

async function settle() {
    await act(async () => { await Promise.resolve(); });
}

async function loadModule(name: string): Promise<any> {
    return import(/* @vite-ignore */ `./${name}.js`);
}

async function mountSurface(overrides: Record<string, unknown> = {}) {
    const { ComposerSurface } = await loadModule('ComposerSurface');
    const props = {
        value: '', onChange: vi.fn(), attachments: [], onAttach: vi.fn(), onRemove: vi.fn(), onSend: vi.fn(() => true),
        mode: 'ask', onMode: vi.fn(),
        model: 'model-a', onModel: vi.fn(), environment: 'Local browser', onEnvironment: vi.fn(),
        disabled: false, runtimeAvailable: false, templatePrompt: (kind: string) => `Prompt for ${kind}`,
        ...overrides,
    };
    const Surface = ComposerSurface as (surfaceProps: typeof props) => any;
    act(() => { render(<Surface {...props} />, host); });
    return props;
}

function fakeFile(name: string, size: number, type: string): File {
    return { name, size, type } as File;
}

function deferredImage(name: string, size: number) {
    const file = new File([new Uint8Array(1)], name, { type: 'image/png' });
    Object.defineProperty(file, 'size', { value: size });
    let resolve!: () => void;
    const pending = new Promise<ArrayBuffer>((done) => { resolve = () => done(new Uint8Array(size).buffer); });
    Object.defineProperty(file, 'arrayBuffer', { value: () => pending });
    return { file, resolve };
}

function deferredDocument(name: string, size: number, content = 'x') {
    const file = new File([content], name, { type: 'text/plain' });
    Object.defineProperty(file, 'size', { value: size });
    let resolve!: () => void;
    const pending = new Promise<string>((done) => { resolve = () => done(content); });
    Object.defineProperty(file, 'text', { value: () => pending });
    return { file, resolve };
}

async function flushAsync() {
    await act(async () => { await new Promise((resolve) => setTimeout(resolve, 0)); });
}

describe('composer surface regression and editor behaviour', () => {
    it('existing_ime_composition_regression keeps Enter, IME composition, and history intact', () => {
        const { generation } = mount();
        const textarea = promptElement();
        typePrompt('first\nline');
        pressKey(textarea, 'Enter');
        expect(generation.send).toHaveBeenCalledWith('first\nline');
        typePrompt('second');
        pressKey(textarea, 'ArrowUp');
        expect(textarea.value).toBe('first\nline');
        pressKey(textarea, 'ArrowDown');
        expect(textarea.value).toBe('second');
        typePrompt('in progress');
        pressKey(textarea, 'Enter', { isComposing: true });
        pressKey(textarea, 'Enter', { keyCode: 229 });
        expect(generation.send).toHaveBeenCalledTimes(1);
    });

    it('shift_enter_inserts_newline_at_caret', () => {
        const { generation } = mount();
        const textarea = promptElement();
        typePrompt('abcd', 2);
        const enter = pressKey(textarea, 'Enter', { shiftKey: true });
        expect(enter.defaultPrevented).toBe(false);
        expect(textarea.selectionStart).toBe(2);
        act(() => {
            textarea.value = `${textarea.value.slice(0, 2)}\n${textarea.value.slice(2)}`;
            textarea.setSelectionRange(3, 3);
            textarea.dispatchEvent(new Event('input', { bubbles: true }));
        });
        expect(textarea.value).toBe('ab\ncd');
        expect(textarea.selectionStart).toBe(3);
        pressKey(textarea, 'Enter');
        expect(generation.send).toHaveBeenCalledWith('ab\ncd');
    });

    it('visible_send_button_and_prompt_label_are_preserved', () => {
        const { store, generation } = mount({ selected: false });
        expect(host.querySelector('textarea[aria-label="Prompt"]')).not.toBeNull();
        expect(button('Send').disabled).toBe(true);
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        typePrompt('Hello');
        expect(button('Send').disabled).toBe(false);
        act(() => { button('Send').click(); });
        expect(generation.send).toHaveBeenCalledWith('Hello');
        expect(host.querySelector('.agent-composer[data-composer-surface="true"]')).not.toBeNull();
    });

    it('keeps one generation send path through the integrated composer', () => {
        const { generation } = mount();
        typePrompt('single path');
        act(() => { button('Send').click(); });
        expect(generation.send).toHaveBeenCalledTimes(1);
        expect(generation.send).toHaveBeenCalledWith('single path');
    });

    it('reports a synchronous generation send exception and preserves the draft', () => {
        const { uiApi } = mount({ sendThrows: true });
        typePrompt('keep this draft');
        act(() => { button('Send').click(); });
        expect(uiApi.api.toast).toHaveBeenCalledWith(expect.stringContaining('generation exploded'), 'error');
        expect(promptElement().value).toBe('keep this draft');
    });
});

describe('bounded shared attachment budget', () => {
    it('accepts exact per-file boundaries and rejects limit plus one with exact sizes', async () => {
        const { planAttachmentBatch, MAX_IMAGE_BYTES, MAX_DOCUMENT_BYTES } = await loadModule('ComposerAttachments');
        const exact = planAttachmentBatch([
            fakeFile('exact.png', MAX_IMAGE_BYTES, 'image/png'),
            fakeFile('exact.md', MAX_DOCUMENT_BYTES, 'text/markdown'),
        ], []);
        expect(exact.errors).toEqual([]);
        expect(exact.accepted.map((entry: { kind: string }) => entry.kind)).toEqual(['image', 'document']);

        const over = planAttachmentBatch([
            fakeFile('photo.png', MAX_IMAGE_BYTES + 1, 'image/png'),
            fakeFile('report.md', MAX_DOCUMENT_BYTES + 1, 'text/markdown'),
        ], []);
        expect(over.accepted).toEqual([]);
        expect(over.errors).toEqual([
            'photo.png exceeds the 10485760-byte image limit (10485761 bytes).',
            'report.md exceeds the 4194304-byte document limit (4194305 bytes).',
        ]);
    });

    it('counts existing text and images for the five-attachment limit and allows same-name replacement', async () => {
        const { planAttachmentBatch } = await loadModule('ComposerAttachments');
        const existing = Array.from({ length: 5 }, (_, index) => ({ name: `file-${index}.md`, size: 1, kind: 'text' as const }));
        const overflow = planAttachmentBatch([fakeFile('six.md', 1, 'text/markdown')], existing);
        expect(overflow.accepted).toEqual([]);
        expect(overflow.errors).toEqual(['six.md exceeds the 5-attachment limit (6 attachments requested).']);
        const replaced = planAttachmentBatch([fakeFile('file-4.md', 2, 'text/markdown')], existing);
        expect(replaced.errors).toEqual([]);
        expect(replaced.accepted).toHaveLength(1);
    });

    it('accepts the exact 25 MiB total across mixed images and documents and rejects one byte over', async () => {
        const { planAttachmentBatch, MAX_IMAGE_BYTES, MAX_DOCUMENT_BYTES, MAX_TOTAL_ATTACHMENT_BYTES } = await loadModule('ComposerAttachments');
        const exactBatch = [
            fakeFile('a.png', MAX_IMAGE_BYTES, 'image/png'),
            fakeFile('b.png', MAX_IMAGE_BYTES, 'image/png'),
            fakeFile('c.md', MAX_DOCUMENT_BYTES, 'text/markdown'),
            fakeFile('d.png', MAX_TOTAL_ATTACHMENT_BYTES - (2 * MAX_IMAGE_BYTES) - MAX_DOCUMENT_BYTES, 'image/png'),
        ];
        const exact = planAttachmentBatch(exactBatch, []);
        expect(exact.errors).toEqual([]);
        expect(exact.accepted).toHaveLength(4);
        const bytes = exactBatch.reduce((sum, file) => sum + file.size, 0);
        expect(bytes).toBe(MAX_TOTAL_ATTACHMENT_BYTES);

        const over = planAttachmentBatch([...exactBatch, fakeFile('one-byte.txt', 1, 'text/plain')], []);
        expect(over.accepted).toHaveLength(4);
        expect(over.errors).toEqual(['one-byte.txt exceeds the 26214400-byte total attachment limit (26214401 bytes).']);
    });

    it('counts a mixed batch sequentially and keeps same-name replacements from growing the budget', async () => {
        const { planAttachmentBatch, MAX_IMAGE_BYTES } = await loadModule('ComposerAttachments');
        const existing = [
            { name: 'a.png', size: 5 * 1024 * 1024, kind: 'image' as const },
            { name: 'b.png', size: 5 * 1024 * 1024, kind: 'image' as const },
            { name: 'c.md', size: 1024 * 1024, kind: 'text' as const },
            { name: 'd.md', size: 1024 * 1024, kind: 'text' as const },
        ];
        const overflow = planAttachmentBatch([
            fakeFile('e.png', MAX_IMAGE_BYTES, 'image/png'),
            fakeFile('e.md', 4 * 1024 * 1024, 'text/markdown'),
        ], existing);
        expect(overflow.accepted.map((entry: { file: { name: string } }) => entry.file.name)).toEqual(['e.png']);
        expect(overflow.errors).toEqual(['e.md exceeds the 5-attachment limit (6 attachments requested).']);

        const replacement = planAttachmentBatch([
            fakeFile('c.md', 2 * 1024 * 1024, 'text/markdown'),
            fakeFile('e.png', MAX_IMAGE_BYTES, 'image/png'),
        ], existing);
        expect(replacement.errors).toEqual([]);
        expect(replacement.accepted).toHaveLength(2);
    });

    it('applies the shared count budget to current-file and pasted text attachments', async () => {
        const { uiApi } = mount();
        for (let index = 0; index < 5; index += 1) uiApi.attachments.push({ name: `seed-${index}.md`, content: 'x', size: 1, source: 'file' });
        act(() => { document.dispatchEvent(new CustomEvent('uor-attachments-changed')); });
        act(() => { button('Attach current file').click(); });
        await flushAsync();
        expect(uiApi.api.addAttachment).not.toHaveBeenCalled();
        expect(uiApi.api.toast).toHaveBeenCalledWith('notes.md exceeds the 5-attachment limit (6 attachments requested).', 'error');
    });
});

describe('memory-only image and document attachments', () => {
    it('paste_image_creates_memory_only_attachment without browser persistence', async () => {
        const { uiApi } = mount();
        const textarea = promptElement();
        const setItem = vi.spyOn(Storage.prototype, 'setItem');
        const indexedOpen = vi.fn();
        const originalIndexedDB = Object.getOwnPropertyDescriptor(window, 'indexedDB');
        Object.defineProperty(window, 'indexedDB', { configurable: true, value: { open: indexedOpen } });
        pasteFiles(textarea, [imageFile()]);
        await vi.waitFor(() => { expect(host.querySelector('[data-attachment-kind="image"]')).not.toBeNull(); });
        expect(host.textContent).toContain('pixel.png');
        expect(createObjectURL).toHaveBeenCalledOnce();
        expect(host.querySelector<HTMLImageElement>('.composer-attachment-preview')?.src).toBe('blob:test/1');
        expect(uiApi.api.addAttachment).not.toHaveBeenCalled();
        expect(uiApi.attachments).toEqual([]);
        expect(setItem).not.toHaveBeenCalled();
        expect(indexedOpen).not.toHaveBeenCalled();
        expect(host.innerHTML).not.toContain('data:image');
        if (originalIndexedDB) Object.defineProperty(window, 'indexedDB', originalIndexedDB);
        else Reflect.deleteProperty(window, 'indexedDB');
    });

    it('revokes preview URLs on remove, replacement, and unmount', async () => {
        const { } = mount();
        let textarea = promptElement();
        pasteFiles(textarea, [imageFile('photo.png', [1])]);
        await vi.waitFor(() => { expect(host.querySelector('[data-attachment-kind="image"]')).not.toBeNull(); });
        expect(createObjectURL).toHaveReturnedWith('blob:test/1');

        pasteFiles(textarea, [imageFile('photo.png', [2])]);
        await vi.waitFor(() => { expect(revokeObjectURL).toHaveBeenCalledWith('blob:test/1'); });
        expect(host.querySelectorAll('[data-attachment-kind="image"]')).toHaveLength(1);

        act(() => { button('Remove photo.png').click(); });
        expect(revokeObjectURL).toHaveBeenCalledWith('blob:test/2');
        expect(host.querySelector('[data-attachment-kind="image"]')).toBeNull();

        pasteFiles(textarea, [imageFile('remaining.png', [3])]);
        await vi.waitFor(() => { expect(host.querySelector('[data-attachment-kind="image"]')).not.toBeNull(); });
        act(() => { render(null, host); });
        expect(revokeObjectURL).toHaveBeenCalledWith('blob:test/3');
    });

    it('document_over_limit_reports_exact_size and does not read or attach it', async () => {
        const { uiApi } = mount();
        const textarea = promptElement();
        const oversized = new File([new Uint8Array(4 * 1024 * 1024 + 1)], 'report.md', { type: 'text/markdown' });
        pasteFiles(textarea, [oversized]);
        await vi.waitFor(() => {
            expect(uiApi.api.toast).toHaveBeenCalledWith('report.md exceeds the 4194304-byte document limit (4194305 bytes).', 'error');
        });
        expect(uiApi.api.addAttachment).not.toHaveBeenCalled();
    });

    it('accepts an exact four MiB document through readTextFiles', async () => {
        const { uiApi } = mount();
        const textarea = promptElement();
        const exact = new File(['a'.repeat(4 * 1024 * 1024)], 'exact.md', { type: 'text/markdown' });
        pasteFiles(textarea, [exact]);
        await vi.waitFor(() => {
            expect(uiApi.api.addAttachment).toHaveBeenCalledWith({ name: 'exact.md', kind: 'text', content: expect.any(String), source: 'file', size: 4194304 });
        });
        expect(uiApi.api.toast).not.toHaveBeenCalled();
        expect(host.querySelector('[data-attachment-kind="document"]')?.textContent).toContain('exact.md');
    });

    it('counts a mixed image and document batch against one budget before attaching', async () => {
        const { uiApi } = mount();
        const textarea = promptElement();
        const image = imageFile('mixed.png');
        const document = new File(['mixed document'], 'mixed.md', { type: 'text/markdown' });
        pasteFiles(textarea, [image, document]);
        await vi.waitFor(() => { expect(uiApi.api.addAttachment).toHaveBeenCalledWith(expect.objectContaining({ name: 'mixed.md' })); });
        expect(host.querySelectorAll('[data-attachment-kind="image"]')).toHaveLength(1);
        expect(host.querySelectorAll('[data-attachment-kind="document"]')).toHaveLength(1);
    });
});

describe('serialized attachment operations', () => {
    it('serializes overlapping file batches so four retained plus two 1-byte documents cannot exceed five', async () => {
        const { uiApi } = mount();
        for (let index = 0; index < 4; index += 1) uiApi.attachments.push({ name: `seed-${index}.md`, content: 'x', source: 'file', size: 1 });
        act(() => { document.dispatchEvent(new CustomEvent('uor-attachments-changed')); });
        const textarea = promptElement();
        const first = deferredDocument('first.md', 1);
        const second = deferredDocument('second.md', 1);
        pasteFiles(textarea, [first.file]);
        pasteFiles(textarea, [second.file]);
        first.resolve();
        await flushAsync();
        second.resolve();
        await flushAsync();
        expect(uiApi.attachments).toHaveLength(5);
        expect(uiApi.attachments.map((attachment) => attachment.name)).toContain('first.md');
        expect(uiApi.attachments.map((attachment) => attachment.name)).not.toContain('second.md');
        expect(uiApi.api.toast).toHaveBeenCalledWith('second.md exceeds the 5-attachment limit (6 attachments requested).', 'error');
    });

    it('serializes overlapping image batches so the shared 25 MiB total is never exceeded', async () => {
        const { uiApi } = mount();
        uiApi.attachments.push({ name: 'existing.png', kind: 'image', content: '', size: 10 * 1024 * 1024 });
        act(() => { document.dispatchEvent(new CustomEvent('uor-attachments-changed')); });
        const textarea = promptElement();
        const first = deferredImage('first.png', 10 * 1024 * 1024);
        const second = deferredImage('second.png', 10 * 1024 * 1024);
        pasteFiles(textarea, [first.file]);
        pasteFiles(textarea, [second.file]);
        first.resolve();
        await flushAsync();
        second.resolve();
        await flushAsync();
        expect(host.querySelectorAll('[data-attachment-kind="image"]')).toHaveLength(2);
        expect(uiApi.api.toast).toHaveBeenCalledWith('second.png exceeds the 26214400-byte total attachment limit (31457280 bytes).', 'error');
    });

    it('serializes same-name overlapping images, keeps one draft, and revokes the replaced URL', async () => {
        mount();
        const textarea = promptElement();
        const first = deferredImage('same.png', 4);
        const second = deferredImage('same.png', 5);
        pasteFiles(textarea, [first.file]);
        pasteFiles(textarea, [second.file]);
        first.resolve();
        second.resolve();
        await flushAsync();
        expect(host.querySelectorAll('[data-attachment-kind="image"]')).toHaveLength(1);
        expect(createObjectURL).toHaveBeenCalledTimes(2);
        expect(revokeObjectURL).toHaveBeenCalledWith('blob:test/1');
        expect(revokeObjectURL).not.toHaveBeenCalledWith('blob:test/2');
        expect(host.querySelector<HTMLImageElement>('.composer-attachment-preview')?.src).toBe('blob:test/2');
    });

    it('queues current-file and pasted additions behind a pending file batch', async () => {
        const { uiApi } = mount();
        for (let index = 0; index < 4; index += 1) uiApi.attachments.push({ name: `seed-${index}.md`, content: 'x', source: 'file', size: 1 });
        act(() => { document.dispatchEvent(new CustomEvent('uor-attachments-changed')); });
        const pending = deferredDocument('pending.md', 1);
        pasteFiles(promptElement(), [pending.file]);
        act(() => { button('Attach current file').click(); });
        const pasted = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Pasted context"]')!;
        act(() => {
            pasted.value = 'pasted text';
            pasted.dispatchEvent(new Event('input', { bubbles: true }));
        });
        act(() => { button('Attach pasted text').click(); });
        pending.resolve();
        await flushAsync();
        expect(uiApi.attachments).toHaveLength(5);
        expect(uiApi.attachments.map((attachment) => attachment.name)).toContain('pending.md');
        expect(uiApi.attachments.map((attachment) => attachment.name)).not.toContain('notes.md');
        expect(uiApi.attachments.map((attachment) => attachment.name)).not.toContain('pasted-context.txt');
        expect(uiApi.api.toast).toHaveBeenCalledWith('notes.md exceeds the 5-attachment limit (6 attachments requested).', 'error');
        expect(uiApi.api.toast).toHaveBeenCalledWith('pasted-context.txt exceeds the 5-attachment limit (6 attachments requested).', 'error');
    });

    it('queues an active-file mention behind a pending batch and preserves the draft on rejection', async () => {
        const { uiApi } = mount();
        for (let index = 0; index < 4; index += 1) uiApi.attachments.push({ name: `seed-${index}.md`, content: 'x', source: 'file', size: 1 });
        act(() => { document.dispatchEvent(new CustomEvent('uor-attachments-changed')); });
        const pending = deferredDocument('pending.md', 1);
        pasteFiles(promptElement(), [pending.file]);
        typePrompt('@');
        const option = host.querySelector<HTMLElement>('[role="option"][data-attachment="notes.md"]')!;
        act(() => { option.click(); });
        pending.resolve();
        await flushAsync();
        expect(uiApi.attachments).toHaveLength(5);
        expect(uiApi.attachments.map((attachment) => attachment.name)).toContain('pending.md');
        expect(uiApi.attachments.map((attachment) => attachment.name)).not.toContain('notes.md');
        expect(promptElement().value).toBe('@');
        expect(uiApi.api.toast).toHaveBeenCalledWith('notes.md exceeds the 5-attachment limit (6 attachments requested).', 'error');
    });

    it('drops a late image completion after clear without creating or adding a preview', async () => {
        const { uiApi } = mount();
        uiApi.attachments.push({ name: 'seed.md', content: 'x', source: 'file', size: 1 });
        act(() => { document.dispatchEvent(new CustomEvent('uor-attachments-changed')); });
        const pending = deferredImage('late.png', 4);
        pasteFiles(promptElement(), [pending.file]);
        act(() => { button('Clear attachments').click(); });
        pending.resolve();
        await flushAsync();
        expect(host.querySelector('[data-attachment-kind="image"]')).toBeNull();
        expect(uiApi.api.addAttachment).not.toHaveBeenCalled();
        expect(createObjectURL).not.toHaveBeenCalled();
        expect(uiApi.api.toast).not.toHaveBeenCalled();
    });

    it('revokes an already-created preview URL when the remaining batch becomes stale', async () => {
        const { uiApi } = mount();
        uiApi.attachments.push({ name: 'seed.md', content: 'x', source: 'file', size: 1 });
        act(() => { document.dispatchEvent(new CustomEvent('uor-attachments-changed')); });
        const first = deferredImage('first.png', 4);
        const second = deferredImage('second.png', 4);
        pasteFiles(promptElement(), [first.file, second.file]);
        first.resolve();
        await flushAsync();
        expect(createObjectURL).toHaveBeenCalledOnce();
        act(() => { button('Clear attachments').click(); });
        second.resolve();
        await flushAsync();
        expect(host.querySelector('[data-attachment-kind="image"]')).toBeNull();
        expect(revokeObjectURL).toHaveBeenCalledWith('blob:test/1');
        expect(createObjectURL).toHaveBeenCalledOnce();
    });

    it('does not attach or toast a late document after unmount', async () => {
        const { uiApi } = mount();
        const pending = deferredDocument('late-document.md', 1);
        pasteFiles(promptElement(), [pending.file]);
        act(() => { render(null, host); });
        pending.resolve();
        await flushAsync();
        expect(uiApi.api.addAttachment).not.toHaveBeenCalled();
        expect(uiApi.api.toast).not.toHaveBeenCalled();
    });

    it('shows stored image size and labels it differently from a memory-only image', async () => {
        const { uiApi } = mount();
        uiApi.attachments.push({ name: 'legacy.png', kind: 'image', content: '', size: 10 * 1024 * 1024 });
        act(() => { document.dispatchEvent(new CustomEvent('uor-attachments-changed')); });
        const budget = host.querySelector('[aria-label="Attachment budget"]')!;
        expect(budget.textContent).toContain('10 MiB/25 MiB');
        const legacy = host.querySelector<HTMLElement>('[data-attachment-id="stored:legacy.png"]')!;
        expect(legacy.textContent).toContain('image · stored context');
        expect(legacy.querySelector('[data-attachment-size]')?.getAttribute('data-attachment-size')).toBe(String(10 * 1024 * 1024));
        expect(host.textContent).toContain('Stored image context');
        pasteFiles(promptElement(), [new File([new Uint8Array(1024 * 1024)], 'new.png', { type: 'image/png' })]);
        await vi.waitFor(() => { expect(host.querySelector('[data-attachment-id^="image:new.png:"]')).not.toBeNull(); });
        const memory = host.querySelector<HTMLElement>('[data-attachment-id^="image:new.png:"]')!;
        expect(memory.textContent).toContain('image · memory only');
        expect(budget.textContent).toContain('11 MiB/25 MiB');
    });
});

describe('mention send ordering and selective removal', () => {
    it('waits for an accepted mention queued behind a pending batch before sending the final prompt', async () => {
        const { uiApi, generation } = mount();
        uiApi.attachments.push({ name: 'seed-1.md', content: 'x', source: 'file', size: 1 }, { name: 'seed-2.md', content: 'x', source: 'file', size: 1 });
        act(() => { document.dispatchEvent(new CustomEvent('uor-attachments-changed')); });
        const pending = deferredDocument('pending.md', 1);
        pasteFiles(promptElement(), [pending.file]);
        typePrompt('@');
        const option = host.querySelector<HTMLElement>('[role="option"][data-attachment="notes.md"]')!;
        act(() => { option.click(); });
        pressKey(promptElement(), 'Enter');
        pending.resolve();
        await flushAsync();
        expect(generation.send).toHaveBeenCalledTimes(1);
        expect(generation.send).toHaveBeenCalledWith('@notes.md');
        expect(promptElement().value).toBe('');
        expect(uiApi.attachments.map((attachment) => attachment.name)).toEqual(expect.arrayContaining(['pending.md', 'notes.md']));
    });

    it('does not insert an accepted mention into a cleared draft', async () => {
        mount();
        const pending = deferredDocument('pending.md', 1);
        pasteFiles(promptElement(), [pending.file]);
        typePrompt('@');
        const option = host.querySelector<HTMLElement>('[role="option"][data-attachment="notes.md"]')!;
        act(() => { option.click(); });
        act(() => {
            const textarea = promptElement();
            textarea.value = '';
            textarea.dispatchEvent(new Event('input', { bubbles: true }));
        });
        pending.resolve();
        await flushAsync();
        expect(promptElement().value).toBe('');
    });

    it('does not insert an accepted mention into a shortened trigger', async () => {
        mount();
        const pending = deferredDocument('pending.md', 1);
        pasteFiles(promptElement(), [pending.file]);
        typePrompt('hello @');
        const option = host.querySelector<HTMLElement>('[role="option"][data-attachment="notes.md"]')!;
        act(() => { option.click(); });
        act(() => {
            const textarea = promptElement();
            textarea.value = 'hello ';
            textarea.dispatchEvent(new Event('input', { bubbles: true }));
        });
        pending.resolve();
        await flushAsync();
        expect(promptElement().value).toBe('hello ');
    });

    it('catches a rejected mention callback without stale insertion', async () => {
        await mountSurface({
            editorFile: { path: 'notes.md', content: 'context text' },
            attachments: [],
            onMentionAdd: vi.fn(() => Promise.reject(new Error('mention rejected'))),
        });
        typePrompt('@');
        const option = host.querySelector<HTMLElement>('[role="option"][data-attachment="notes.md"]')!;
        act(() => { option.click(); });
        await flushAsync();
        expect(promptElement().value).toBe('@');
    });

    it('removing an unrelated attachment does not cancel a pending multi-file batch', async () => {
        const { uiApi } = mount();
        uiApi.attachments.push({ name: 'seed-0.md', content: 'x', source: 'file', size: 1 }, { name: 'seed-1.md', content: 'x', source: 'file', size: 1 });
        act(() => { document.dispatchEvent(new CustomEvent('uor-attachments-changed')); });
        const first = deferredDocument('first.md', 1);
        const second = deferredDocument('second.md', 1);
        pasteFiles(promptElement(), [first.file, second.file]);
        await flushAsync();
        act(() => { button('Remove seed-0.md').click(); });
        first.resolve();
        second.resolve();
        await flushAsync();
        expect(uiApi.attachments.map((attachment) => attachment.name)).toEqual(expect.arrayContaining(['first.md', 'second.md', 'seed-1.md']));
        expect(uiApi.attachments.map((attachment) => attachment.name)).not.toContain('seed-0.md');
        expect(uiApi.api.toast).not.toHaveBeenCalled();
    });

    it('skips a same-name replacement after its existing attachment is removed and reports it', async () => {
        const { uiApi } = mount();
        pasteFiles(promptElement(), [imageFile('same.png', [1])]);
        await vi.waitFor(() => { expect(host.querySelector('[data-attachment-kind="image"]')).not.toBeNull(); });
        expect(createObjectURL).toHaveBeenCalledTimes(1);
        const replacement = deferredImage('same.png', 4);
        pasteFiles(promptElement(), [replacement.file]);
        await flushAsync();
        act(() => { button('Remove same.png').click(); });
        expect(revokeObjectURL).toHaveBeenCalledWith('blob:test/1');
        replacement.resolve();
        await flushAsync();
        expect(host.querySelector('[data-attachment-kind="image"]')).toBeNull();
        expect(createObjectURL).toHaveBeenCalledTimes(1);
        expect(uiApi.api.toast).toHaveBeenCalledWith('same.png was removed before it could be attached.', 'error');
    });
});

describe('composer menus and host state', () => {
    it('at_menu_lists_editor_file_and_attachments_only', () => {
        const { uiApi } = mount();
        uiApi.attachments.push({ name: 'external.txt', content: 'external context', source: 'workspace' });
        act(() => { document.dispatchEvent(new CustomEvent('uor-attachments-changed')); });
        typePrompt('@');
        const menu = host.querySelector<HTMLElement>('[role="listbox"][aria-label="Attachment mentions"]');
        expect(menu).not.toBeNull();
        const options = Array.from(menu!.querySelectorAll<HTMLElement>('[role="option"]'));
        expect(options.map((option) => option.textContent)).toEqual(['notes.md (Active file)', 'external.txt (Attachment)']);
        expect(host.textContent).not.toContain('(Artifact)');
    });

    it('lists only explicitly supplied artifacts in the mention menu', async () => {
        const props = await mountSurface({ artifacts: [{ name: 'spec.md', content: 'artifact body', source: 'task' }] });
        typePrompt('@');
        const options = Array.from(host.querySelectorAll<HTMLElement>('[role="option"]'));
        expect(options.map((option) => option.textContent)).toEqual(['spec.md (Artifact)']);
        act(() => { options[0].click(); });
        expect(promptElement().value).toBe('@spec.md');
        expect(props.onAttach).not.toHaveBeenCalled();
    });

    it('slash_menu_lists_the_six_local_templates', async () => {
        const props = await mountSurface();
        typePrompt('/');
        const options = Array.from(host.querySelectorAll<HTMLElement>('[role="option"]'));
        expect(options.map((option) => option.dataset.template)).toEqual(['ux', 'api', 'audit', 'report', 'error', 'test']);
        act(() => { options[3].click(); });
        expect(promptElement().value).toBe('Prompt for report');
        expect(props.onSend).not.toHaveBeenCalled();
    });

    it('permission_picker_is_disabled_without_host', async () => {
        const props = await mountSurface();
        const permission = host.querySelector<HTMLSelectElement>('select[aria-label="Permission mode"]');
        expect(permission).not.toBeNull();
        expect(permission!.value).toBe('ask');
        const autonomous = Array.from(permission!.options).find((option) => option.textContent?.startsWith('Autonomous'));
        expect(autonomous?.disabled).toBe(true);
        act(() => { permission!.value = 'autonomous'; permission!.dispatchEvent(new Event('change', { bubbles: true })); });
        expect(props.onMode).not.toHaveBeenCalled();
        expect(permission!.value).toBe('ask');
    });

    it('environment_picker_shows_only_the_real_local_state', async () => {
        const props = await mountSurface();
        const environment = host.querySelector<HTMLSelectElement>('select[aria-label="Composer environment"]');
        expect(environment).not.toBeNull();
        expect(environment!.value).toBe('Local browser');
        const service = Array.from(environment!.options).find((option) => option.textContent?.includes('Harness service'));
        expect(service?.disabled).toBe(true);
        act(() => { environment!.value = 'harness'; environment!.dispatchEvent(new Event('change', { bubbles: true })); });
        expect(props.onEnvironment).not.toHaveBeenCalled();
    });

    it('model_choice_uses_the_existing_model_action', async () => {
        const props = await mountSurface();
        act(() => { button('Model: model-a').click(); });
        expect(props.onModel).toHaveBeenCalledOnce();
        expect(host.querySelectorAll('select[aria-label="Model"]')).toHaveLength(0);
    });

    it('keeps one Send and exposes disabled run send modes without a selected runtime', async () => {
        const onQueue = vi.fn(() => true);
        const onSteer = vi.fn(() => true);
        const onForce = vi.fn(() => true);
        const onStopRun = vi.fn();
        const props = await mountSurface({ value: 'send me', onQueue, onSteer, onForce, onStopRun });
        const sendButtons = Array.from(host.querySelectorAll('button')).filter((node) => node.textContent === 'Send');
        expect(sendButtons).toHaveLength(1);
        for (const label of ['Queue', 'Steer', 'Force send', 'Stop run']) {
            const mode = button(label);
            expect(mode.disabled, label).toBe(true);
            expect(mode.getAttribute('title'), label).toContain('Select a task');
        }
        typePrompt('send me');
        act(() => { button('Send').click(); });
        expect(props.onSend).toHaveBeenCalledWith('send me');
        expect(onQueue).not.toHaveBeenCalled();
        expect(onStopRun).not.toHaveBeenCalled();
    });

    it('treats cancelling as active and disables only repeated stop', async () => {
        const activeRun = { taskId: 'tsk_0000000000000001', runId: 'run_0000000000000001', generation: 1, state: 'cancelling' as const, retryOfRunId: null };
        await mountSurface({ value: 'wait', activeRun, onQueue: vi.fn(() => true), onSteer: vi.fn(() => true), onForce: vi.fn(() => true), onStopRun: vi.fn() });
        expect(button('Queue').disabled).toBe(false);
        expect(button('Steer').disabled).toBe(false);
        expect(button('Force send').disabled).toBe(false);
        expect(button('Stop run').disabled).toBe(true);
    });

    it('passes the visible prompt only to enabled run send modes', async () => {
        const activeRun = { taskId: 'tsk_0000000000000001', runId: 'run_0000000000000001', generation: 1, state: 'running' as const, retryOfRunId: null };
        const onQueue = vi.fn(() => true);
        const onSteer = vi.fn(() => true);
        const onForce = vi.fn(() => true);
        const onStopRun = vi.fn();
        await mountSurface({
            value: 'follow up', activeRun, onQueue, onSteer, onForce, onStopRun,
        });
        typePrompt('follow up');
        act(() => { button('Queue').click(); });
        typePrompt('follow up');
        act(() => { button('Steer').click(); });
        typePrompt('follow up');
        act(() => { button('Force send').click(); });
        act(() => { button('Stop run').click(); });
        expect(onQueue).toHaveBeenCalledWith('follow up');
        expect(onSteer).toHaveBeenCalledWith('follow up');
        expect(onForce).toHaveBeenCalledWith('follow up');
        expect(onStopRun).toHaveBeenCalledOnce();
    });

    it('uses the existing model view action from AgentComposer', () => {
        const { uiApi } = mount();
        act(() => { button('Model: api-a').click(); });
        expect(uiApi.api.setView).toHaveBeenCalledWith('model');
    });
});

describe('asynchronous send receipts and draft preservation', () => {
    it('waits for an async receipt before clearing and blocks duplicate submissions while pending', async () => {
        let accept!: (value: boolean) => void;
        const onSend = vi.fn(() => new Promise<boolean>((resolve) => { accept = resolve; }));
        await mountSurface({ value: 'first draft', onSend });
        typePrompt('first draft');
        act(() => { button('Send').click(); });
        expect(onSend).toHaveBeenCalledWith('first draft');
        expect(promptElement().value).toBe('first draft');
        act(() => { button('Send').click(); });
        pressKey(promptElement(), 'Enter');
        expect(onSend).toHaveBeenCalledTimes(1);
        accept(true);
        await flushAsync();
        expect(promptElement().value).toBe('');
    });

    it('preserves a newer draft when the user edits during an accepted pending send', async () => {
        let accept!: (value: boolean) => void;
        const onSend = vi.fn(() => new Promise<boolean>((resolve) => { accept = resolve; }));
        await mountSurface({ value: 'first draft', onSend });
        typePrompt('first draft');
        act(() => { button('Send').click(); });
        typePrompt('second draft');
        accept(true);
        await flushAsync();
        expect(promptElement().value).toBe('second draft');
    });

    it('records an accepted async send while restoring a newer draft for history recall', async () => {
        let accept!: (value: boolean) => void;
        const onSend = vi.fn(() => new Promise<boolean>((resolve) => { accept = resolve; }));
        await mountSurface({ value: 'first draft', onSend });
        typePrompt('first draft');
        act(() => { button('Send').click(); });
        typePrompt('second draft');
        accept(true);
        await flushAsync();
        expect(promptElement().value).toBe('second draft');
        pressKey(promptElement(), 'ArrowUp');
        expect(promptElement().value).toBe('first draft');
        pressKey(promptElement(), 'ArrowDown');
        expect(promptElement().value).toBe('second draft');
    });

    it('preserves the draft when an async send is refused or rejects', async () => {
        const refused = await mountSurface({ value: 'keep refused', onSend: vi.fn(() => Promise.resolve(false)) });
        typePrompt('keep refused');
        act(() => { button('Send').click(); });
        await flushAsync();
        expect(refused.onSend).toHaveBeenCalledWith('keep refused');
        expect(promptElement().value).toBe('keep refused');

        const rejected = await mountSurface({ value: 'keep rejected', onSend: vi.fn(() => Promise.reject(new Error('offline'))) });
        typePrompt('keep rejected');
        act(() => { button('Send').click(); });
        await flushAsync();
        expect(rejected.onSend).toHaveBeenCalledWith('keep rejected');
        expect(promptElement().value).toBe('keep rejected');
    });
});

class UnloadedSummaryTransport extends FixtureTransport {
    constructor(private readonly summaryTask: TaskSummary) {
        super();
    }

    override async listTasks(): Promise<TaskSummary[]> {
        return [this.summaryTask];
    }
}

class StaleThenNullSummaryTransport extends FixtureTransport {
    private listCalls = 0;

    constructor(
        private readonly first: TaskSummary,
        private readonly later: TaskSummary,
    ) {
        super();
    }

    override async listTasks(): Promise<TaskSummary[]> {
        this.listCalls += 1;
        return [this.listCalls === 1 ? this.first : this.later];
    }
}

describe('task runtime send identity', () => {
    function fixtureRuntime() {
        const transport = new FixtureTransport();
        return { transport, client: new AgentClient({ transport }) };
    }

    async function settleAsync() {
        await act(async () => { await new Promise((resolve) => setTimeout(resolve, 0)); });
        await act(async () => { await new Promise((resolve) => setTimeout(resolve, 0)); });
    }

    async function mountStudio(client: AgentClient, filesApi = makeFiles()) {
        const sessionsApi = makeSessions();
        const uiApi = makeUi();
        act(() => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={filesApi} sessions={sessionsApi} ui={uiApi} agentClient={client} download={vi.fn()} />, host);
        });
        await settleAsync();
        return { sessionsApi, uiApi };
    }

    function editorFiles() {
        const files = makeFiles();
        files.activeFile = vi.fn(() => ({ path: 'notes.md', name: 'notes.md', content: 'context text', mode: 'local' }));
        return files;
    }

    it('starts a run through only AgentClient for a selected task and blocks a second active run', async () => {
        const { transport, client } = fixtureRuntime();
        const task = await client.createTask({ title: 'Selected task' });
        client.store.selectTask(task.taskId);
        const { sessionsApi } = await mountStudio(client);

        const permission = host.querySelector<HTMLSelectElement>('select[aria-label="Permission mode"]')!;
        expect(permission.value).toBe('ask');
        const autonomous = Array.from(permission.options).find((option) => option.textContent?.startsWith('Autonomous'));
        expect(autonomous?.disabled).toBe(true);

        typePrompt('start this task');
        act(() => { button('Send').click(); });
        await vi.waitFor(() => { expect(promptElement().value).toBe(''); });
        const runs = transport.calls.filter((call) => call.method === 'startRun');
        expect(runs).toHaveLength(1);
        expect(runs[0]!.payload).toEqual({ taskId: task.taskId, prompt: 'start this task' });
        expect(sessionsApi.sendPrompt).not.toHaveBeenCalled();
        await vi.waitFor(() => { expect(button('Send').disabled).toBe(true); });
    });

    it('queues, steers, force-sends and stops with the selected task and actual run identity', async () => {
        const { transport, client } = fixtureRuntime();
        const task = await client.createTask({ title: 'Running task' });
        client.store.selectTask(task.taskId);
        const run = await client.startRun({ taskId: task.taskId, prompt: 'initial' });
        const { sessionsApi } = await mountStudio(client);
        const startsBefore = transport.calls.filter((call) => call.method === 'startRun').length;

        expect(button('Send').disabled).toBe(true);
        typePrompt('queued note');
        act(() => { button('Queue').click(); });
        await vi.waitFor(() => { expect(promptElement().value).toBe(''); });
        expect(transport.calls.filter((call) => call.method === 'queueMessage').at(-1)?.payload).toEqual({ taskId: task.taskId, text: 'queued note' });

        typePrompt('steer note');
        act(() => { button('Steer').click(); });
        await vi.waitFor(() => { expect(promptElement().value).toBe(''); });
        expect(transport.calls.filter((call) => call.method === 'steer').at(-1)?.payload).toEqual({ taskId: task.taskId, text: 'steer note' });

        typePrompt('force note');
        act(() => { button('Force send').click(); });
        await vi.waitFor(() => { expect(promptElement().value).toBe(''); });
        expect(transport.calls.filter((call) => call.method === 'forceSend').at(-1)?.payload).toEqual({ taskId: task.taskId, text: 'force note' });

        act(() => { button('Stop run').click(); });
        await vi.waitFor(() => {
            expect(transport.calls.filter((call) => call.method === 'stop').at(-1)?.payload).toEqual({ taskId: task.taskId, runId: run.runId });
            expect(button('Queue').disabled).toBe(true);
            expect(button('Stop run').disabled).toBe(true);
        });
        expect(sessionsApi.stopGeneration).not.toHaveBeenCalled();
        expect(transport.calls.filter((call) => call.method === 'startRun')).toHaveLength(startsBefore);
    });

    it('blocks Send and Enter when the selected task latest run is not loaded', async () => {
        const { transport, client } = fixtureRuntime();
        const task = await client.createTask({ title: 'Unloaded task' });
        client.store.upsertTask({
            taskId: task.taskId,
            projectId: task.projectId,
            title: task.title,
            createdAtMs: task.createdAtMs,
            latestRunId: 'run_1111111111111111',
        });
        client.store.selectTask(task.taskId);
        const startsBefore = transport.calls.filter((call) => call.method === 'startRun').length;
        await mountStudio(client);

        expect(button('Send').disabled).toBe(true);
        typePrompt('do not start');
        pressKey(promptElement(), 'Enter');
        await settleAsync();
        expect(transport.calls.filter((call) => call.method === 'startRun')).toHaveLength(startsBefore);
        expect(promptElement().value).toBe('do not start');
    });

    it('blocks the real Send and Enter paths after listTasks reports an unloaded run', async () => {
        const summary: TaskSummary = {
            taskId: 'tsk_9999999999999999',
            projectId: 'prj_0000000000000001',
            title: 'Unloaded list task',
            repo: null,
            branch: null,
            createdAtMs: 1,
            archived: false,
            pinned: false,
            latestRunId: 'run_9999999999999999',
            state: 'running',
        };
        const transport = new UnloadedSummaryTransport(summary);
        const client = new AgentClient({ transport });
        await client.listTasks({ includeArchived: true });
        await mountStudio(client);

        expect(button('Send').disabled).toBe(true);
        typePrompt('do not start from list');
        pressKey(promptElement(), 'Enter');
        await settleAsync();
        expect(transport.calls.filter((call) => call.method === 'startRun')).toHaveLength(0);
        expect(promptElement().value).toBe('do not start from list');
    });

    it('keeps a null-latest reload in Restoring and blocks Send for the unloaded run', async () => {
        const first: TaskSummary = {
            taskId: 'tsk_7777777777777777',
            projectId: 'prj_0000000000000001',
            title: 'Stale then null task',
            repo: null,
            branch: null,
            createdAtMs: 1,
            archived: false,
            pinned: false,
            latestRunId: 'run_7777777777777777',
            state: 'running',
        };
        const later: TaskSummary = {
            ...first,
            latestRunId: null,
            state: 'idle',
        };
        const transport = new StaleThenNullSummaryTransport(first, later);
        const client = new AgentClient({ transport });
        await client.listTasks({ includeArchived: true });
        await client.listTasks({ includeArchived: true });
        client.store.selectTask(later.taskId);
        await mountStudio(client);

        expect(button('Send').disabled).toBe(true);
        const row = host.querySelector<HTMLElement>('[data-task-id="tsk_7777777777777777"]');
        expect(row).not.toBeNull();
        expect(row!.textContent).toContain('Restoring');
        expect(row!.textContent).not.toContain('Idle');

        typePrompt('do not start from a null summary');
        pressKey(promptElement(), 'Enter');
        await settleAsync();
        expect(transport.calls.filter((call) => call.method === 'startRun')).toHaveLength(0);
        expect(promptElement().value).toBe('do not start from a null summary');
    });

    it('blocks Enter during a cancelling run without starting another run', async () => {
        const { transport, client } = fixtureRuntime();
        const task = await client.createTask({ title: 'Cancelling task' });
        client.store.selectTask(task.taskId);
        await client.startRun({ taskId: task.taskId, prompt: 'initial' });
        const selected = client.store.snapshot().selectedTask!;
        const latest = selected.runs.find((candidate) => candidate.runId === selected.latestRunId)!;
        const cancelling = { ...latest, state: 'cancelling' as const };
        const startsBefore = transport.calls.filter((call) => call.method === 'startRun').length;
        const { uiApi } = mount({ agentClient: client, agentTask: selected, activeRun: cancelling });

        expect(button('Send').disabled).toBe(true);
        typePrompt('hold this draft');
        pressKey(promptElement(), 'Enter');
        await settleAsync();
        expect(transport.calls.filter((call) => call.method === 'startRun')).toHaveLength(startsBefore);
        expect(uiApi.api.toast).not.toHaveBeenCalled();
        expect(promptElement().value).toBe('hold this draft');
    });

    for (const [label, method] of [['Queue', 'queueMessage'], ['Steer', 'steer'], ['Force send', 'forceSend']] as const) {
        it(`waits for a deferred accepted mention before ${label}`, async () => {
            const { transport, client } = fixtureRuntime();
            const task = await client.createTask({ title: `Mention ${label}` });
            client.store.selectTask(task.taskId);
            await client.startRun({ taskId: task.taskId, prompt: 'initial' });
            const { uiApi } = await mountStudio(client, editorFiles());
            const pending = deferredDocument('pending.md', 1);
            pasteFiles(promptElement(), [pending.file]);
            typePrompt('@');
            const option = host.querySelector<HTMLElement>('[role="option"][data-attachment="notes.md"]');
            expect(option).not.toBeNull();
            act(() => { option!.click(); });
            act(() => { button(label).click(); });
            expect(transport.calls.filter((call) => call.method === method)).toHaveLength(0);
            pending.resolve();
            await vi.waitFor(() => {
                expect(transport.calls.filter((call) => call.method === method)).toHaveLength(1);
            }, { timeout: 3000 });
            expect(transport.calls.filter((call) => call.method === method)[0]!.payload).toEqual({ taskId: task.taskId, text: '@notes.md' });
            const added = vi.mocked(uiApi.addAttachment).mock.calls.map((call) => (call[0] as { name: string }).name);
            expect(added).toEqual(expect.arrayContaining(['pending.md', 'notes.md']));
            await vi.waitFor(() => { expect(promptElement().value).toBe(''); });
        });
    }

    it('preserves the draft and reports a failed queue receipt', async () => {
        const { transport, client } = fixtureRuntime();
        const task = await client.createTask({ title: 'Refusing task' });
        client.store.selectTask(task.taskId);
        await client.startRun({ taskId: task.taskId, prompt: 'initial' });
        vi.spyOn(transport, 'queueMessage').mockRejectedValue(new Error('queue refused'));
        const { uiApi } = await mountStudio(client);

        typePrompt('keep queued');
        act(() => { button('Queue').click(); });
        await settleAsync();
        expect(uiApi.toast).toHaveBeenCalledWith('Queue failed: queue refused', 'error');
        expect(uiApi.toast).toHaveBeenCalledTimes(1);
        expect(promptElement().value).toBe('keep queued');
        expect(button('Queue').disabled).toBe(false);
    });

    it('requires a selected task and preserves the draft without starting anything', async () => {
        const { transport, client } = fixtureRuntime();
        await client.createTask({ title: 'Unselected task' });
        const { uiApi } = await mountStudio(client);
        const before = transport.calls.length;

        typePrompt('keep this draft');
        expect(button('Send').disabled, 'runtime tasks without a selected task disable Send').toBe(true);
        act(() => { button('Send').click(); });
        await settleAsync();
        expect(uiApi.toast).not.toHaveBeenCalledWith('Select a task before sending.', 'error');
        expect(transport.calls.slice(before).some((call) => call.method === 'startRun')).toBe(false);
        expect(promptElement().value).toBe('keep this draft');
    });

    it('surfaces a start refusal and preserves the draft', async () => {
        const { client } = fixtureRuntime();
        const task = await client.createTask({ title: 'Archived task' });
        client.store.selectTask(task.taskId);
        await client.archiveTask({ taskId: task.taskId, archived: true });
        const { uiApi } = await mountStudio(client);

        typePrompt('keep me');
        act(() => { button('Send').click(); });
        await settleAsync();
        expect(uiApi.toast).toHaveBeenCalledWith(expect.stringContaining('task_archived'), 'error');
        expect(promptElement().value).toBe('keep me');
    });

    it('starts a fresh run when the selected task latest run is already terminal', async () => {
        const { transport, client } = fixtureRuntime();
        const task = await client.createTask({ title: 'Finished task' });
        client.store.selectTask(task.taskId);
        const first = await client.startRun({ taskId: task.taskId, prompt: 'first' });
        await client.stop({ taskId: task.taskId, runId: first.runId });
        const { sessionsApi } = await mountStudio(client);

        typePrompt('second run');
        act(() => { button('Send').click(); });
        await settleAsync();
        const runs = transport.calls.filter((call) => call.method === 'startRun');
        expect(runs.at(-1)?.payload).toEqual({ taskId: task.taskId, prompt: 'second run' });
        expect(sessionsApi.sendPrompt).not.toHaveBeenCalled();
    });
});
