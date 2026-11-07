import { render } from 'preact';
import { useState } from 'preact/hooks';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LifecycleStore } from '../state/lifecycle.js';
import type { GenerationAdapter } from '../adapters/generation.js';
import type { Attachment, SessionApi, StudioSettings, StudioUiApi, WorkspaceApi } from '../adapters/workflow.js';
import { AgentComposer } from './AgentComposer.js';

let host: HTMLDivElement;
beforeEach(() => { host = document.createElement('div'); document.body.appendChild(host); });
afterEach(() => { render(null, host); host.remove(); vi.restoreAllMocks(); });

function sessions(): SessionApi {
    return {
        getMode: vi.fn((): 'wasm' | 'api' => 'api'), listSessions: vi.fn(() => []), activeSessionId: vi.fn(() => null),
        selectSession: vi.fn(() => true), newSession: vi.fn(() => true), renameSession: vi.fn(() => true),
        duplicateSession: vi.fn(() => null), exportSession: vi.fn(() => null), deleteSession: vi.fn(() => true),
        getConversationMessages: vi.fn(() => []), setComposerText: vi.fn(() => true), sendPrompt: vi.fn(() => 1), predict: vi.fn(() => 1), stopGeneration: vi.fn(),
    };
}

function workspace(): WorkspaceApi {
    return {
        openFolder: vi.fn(async () => undefined), importFiles: vi.fn(), openDemo: vi.fn(), openFile: vi.fn(async () => undefined),
        newFile: vi.fn(), closeTab: vi.fn(), closeAll: vi.fn(), saveActive: vi.fn(async () => true), saveAsCopy: vi.fn(() => true),
        insertText: vi.fn(() => true), setActiveContent: vi.fn(), activeFile: vi.fn(() => ({ path: 'notes.md', name: 'notes.md', content: 'context text', mode: 'local' })),
        workspaceName: vi.fn(() => 'project'), mode: vi.fn(() => 'local'), tabs: vi.fn(() => []),
        treeSummary: vi.fn(() => ({ mode: 'local', rootName: 'project', fileCount: 1, tabs: [], activePath: 'notes.md' })),
    };
}

function ui() {
    const attachments: Attachment[] = [];
    const settings: StudioSettings = { theme: 'dark', reduceMotion: false, fontSize: 13, tabSize: 4, wordWrap: false };
    return {
        attachments,
        api: {
            setView: vi.fn(), toast: vi.fn(), getSettings: vi.fn(() => settings), updateSettings: vi.fn(),
            getAttachments: vi.fn(() => attachments), addAttachment: vi.fn((entry: Attachment) => { attachments.push(entry); }),
            removeAttachment: vi.fn((name: string) => { const index = attachments.findIndex((item) => item.name === name); if (index >= 0) attachments.splice(index, 1); }),
            clearAttachments: vi.fn(() => { attachments.splice(0); }), templatePrompt: vi.fn((kind: string) => `Prompt for ${kind}`),
            clearSessions: vi.fn(() => ({ cleared: 0, total: 0 })), clearEditor: vi.fn(), updateStatus: vi.fn(),
        } satisfies StudioUiApi,
    };
}

interface MountOverrides {
    controlled?: boolean;
    initialDraft?: string;
    initialPasted?: string;
    onDraftChange?(value: string): void;
    onPastedChange?(value: string): void;
}

function ControlledAgentComposer(props: Parameters<typeof AgentComposer>[0]) {
    const [draft, setDraft] = useState(props.draft ?? '');
    const [pasted, setPasted] = useState(props.pasted ?? '');
    return <AgentComposer
        {...props}
        draft={draft}
        pasted={pasted}
        onDraftChange={(value) => { props.onDraftChange?.(value); setDraft(value); }}
        onPastedChange={(value) => { props.onPastedChange?.(value); setPasted(value); }}
    />;
}

function mount(overrides: MountOverrides = {}) {
    const store = new LifecycleStore();
    const sessionApi = sessions();
    const fileApi = workspace();
    const uiApi = ui();
    const generation: GenerationAdapter = {
        send: vi.fn(() => {
            const selected = store.getState().selectedSource;
            if (!selected || !store.canRun()) return false;
            store.dispatch({ type: 'generation/started', source: selected });
            return true;
        }),
        stop: vi.fn(), dispose: vi.fn(),
    };
    const copy = vi.fn(async () => undefined);
    const download = vi.fn();
    let props = {
        store, sessions: sessionApi, files: fileApi, ui: uiApi.api, generation, copy, download,
        draft: overrides.initialDraft, pasted: overrides.initialPasted,
        onDraftChange: overrides.onDraftChange, onPastedChange: overrides.onPastedChange,
    };
    const renderComposer = () => {
        render(overrides.controlled ? <ControlledAgentComposer {...props} /> : <AgentComposer {...props} />, host);
    };
    act(renderComposer);
    return {
        store, sessionApi, fileApi, uiApi, generation, copy, download,
        update(next: { draft?: string; pasted?: string }) {
            props = { ...props, ...next };
            act(renderComposer);
        },
    };
}

function button(label: string): HTMLButtonElement {
    const found = Array.from(host.querySelectorAll('button')).find((node) => node.textContent === label);
    if (!found) throw new Error(`Missing button ${label}`);
    return found;
}

function prompt(value: string) {
    const input = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]')!;
    act(() => {
        input.value = value;
        input.setSelectionRange(value.length, value.length);
        input.dispatchEvent(new Event('input', { bubbles: true }));
    });
}

describe('AgentComposer', () => {
    it('requires a ready source and keeps generation source/privacy pinned', () => {
        const { store, generation } = mount();
        expect(host.textContent).toContain('No source selected');
        expect(button('Send').disabled).toBe(true);
        act(() => { store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' }); });
        prompt('Hello');
        act(() => { button('Send').click(); });
        expect(generation.send).toHaveBeenCalledWith('Hello');
        act(() => { store.dispatch({ type: 'api/selected', sourceId: 'api-b', endpoint: 'http://127.0.0.1:8000/v1' }); });
        expect(host.textContent).toContain('Generating with api-a (remoteAPI)');
        act(() => { button('Stop').click(); });
        expect(generation.stop).toHaveBeenCalledOnce();
        act(() => { store.dispatch({ type: 'generation/finished', outcome: 'error', generationId: 1 }); });
        expect(host.textContent).toContain('Session: error');
    });

    it('disables send and reports an invalid selection when the API endpoint is missing', () => {
        const { store } = mount();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: undefined as unknown as string });
        prompt('Hello');
        expect(button('Send').disabled).toBe(true);
        expect(store.canRun()).toBe(false);
        expect(host.textContent).toContain('Invalid API endpoint');
        expect(host.textContent).not.toContain('API chat');
    });

    it('pins stop capability to the active generation rather than the picker', () => {
        const { store } = mount();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        prompt('API generation');
        act(() => { button('Send').click(); });
        store.dispatch({ type: 'artifact/selected', sourceId: 'model-a' });
        act(() => {
            store.dispatch({ type: 'artifact/ready', operation: store.getState().operation!, artifact: {
                schema: 'uor-r4.lut-discrete-trunk/1', canGenerate: false, canPredict: true, engineSupported: true,
            } });
        });
        expect(button('Stop').disabled).toBe(false);
        act(() => { store.dispatch({ type: 'generation/finished', outcome: 'complete', generationId: 1 }); });
        prompt('Discrete prediction');
        act(() => { button('Predict next tokens').click(); });
        act(() => { store.dispatch({ type: 'api/selected', sourceId: 'api-b', endpoint: 'https://other.example.com/v1' }); });
        expect(button('Stop').disabled).toBe(true);
    });

    it('disables generation for unsupported or prediction-incompatible artifacts', () => {
        const { store } = mount();
        store.dispatch({ type: 'artifact/selected', sourceId: 'model-a' });
        act(() => {
            store.dispatch({ type: 'artifact/ready', operation: store.getState().operation!, artifact: {
                schema: 'uor-r4.lut-stack/99', engineSupported: false, canGenerate: true,
            } });
        });
        expect(store.canRun()).toBe(false);
        expect(host.textContent).toContain('Generation unavailable');
        prompt('Hello');
        expect(button('Send').disabled).toBe(true);
    });

    it('does not offer prediction for an engine-unsupported artifact', () => {
        const { store } = mount();
        store.dispatch({ type: 'artifact/selected', sourceId: 'model-a' });
        act(() => {
            store.dispatch({ type: 'artifact/ready', operation: store.getState().operation!, artifact: {
                schema: 'uor-r4.lut-discrete-trunk/1', engineSupported: false, canGenerate: false, canPredict: true,
            } });
        });
        expect(host.textContent).toContain('Generation unavailable');
        expect(host.textContent).not.toContain('Local next-token prediction');
        expect(button('Send').disabled).toBe(true);
    });

    it('does not label or enable an unmatched resident artifact as ready local generation', () => {
        const { store, generation } = mount();
        act(() => { store.dispatch({ type: 'artifact/selected', sourceId: 'model-a' }); });
        expect(host.textContent).toContain('Selected model; not ready');
        act(() => {
            store.dispatch({ type: 'artifact/ready', operation: store.getState().operation!, artifact: {
                sourceId: 'model-b', schema: 'uor-r4.lut-stack/1', canGenerate: true, engineSupported: true,
            } });
        });
        expect(host.textContent).toContain('Selected model; not ready');
        expect(host.textContent).not.toContain('Local generation');
        prompt('Hello');
        expect(store.canRun()).toBe(false);
        expect(button('Send').disabled).toBe(true);
        act(() => { button('Send').click(); });
        expect(generation.send).not.toHaveBeenCalled();
    });

    it('pins the active generation mode label across source changes', () => {
        const { store } = mount();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        prompt('API generation');
        act(() => { button('Send').click(); });
        store.dispatch({ type: 'artifact/selected', sourceId: 'model-a' });
        act(() => {
            store.dispatch({ type: 'artifact/ready', operation: store.getState().operation!, artifact: {
                schema: 'uor-r4.lut-discrete-trunk/1', canGenerate: false, canPredict: true, engineSupported: false,
            } });
        });
        expect(host.textContent).toContain('API chat');
        expect(host.textContent).not.toContain('Local generation');
        expect(host.textContent).not.toContain('Generation unavailable');
    });

    it('labels discrete local execution as next-token prediction', () => {
        const { store } = mount();
        store.dispatch({ type: 'artifact/selected', sourceId: 'url' });
        act(() => {
            store.dispatch({ type: 'artifact/ready', operation: store.getState().operation!, artifact: {
                schema: 'uor-r4.lut-discrete-trunk/1', canGenerate: false, canPredict: true, engineSupported: true,
            } });
        });
        expect(host.textContent).toContain('Local next-token prediction');
        expect(host.textContent).not.toContain('Conversational chat');
        expect(button('Predict next tokens').disabled).toBe(true);
        prompt('Hello');
        act(() => { button('Predict next tokens').click(); });
        expect(button('Stop').disabled).toBe(true);
    });

    it('rejects a non-HTTP API endpoint', () => {
        const { store } = mount();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'ftp://example.com/v1' });
        prompt('Hello');
        expect(button('Send').disabled).toBe(true);
        expect(store.canRun()).toBe(false);
        expect(host.textContent).toContain('Invalid API endpoint');
        expect(host.textContent).not.toContain('API chat');
    });

    it('supports current-file, pasted, remove, and clear attachments plus six templates', () => {
        const { uiApi } = mount();
        act(() => { button('Attach current file').click(); });
        expect(uiApi.api.addAttachment).toHaveBeenCalledWith({ name: 'notes.md', content: 'context text', source: 'editor' });
        act(() => {
            const input = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Pasted context"]')!;
            input.value = 'pasted text';
            input.dispatchEvent(new Event('input', { bubbles: true }));
        });
        act(() => { button('Attach pasted text').click(); });
        expect(uiApi.api.addAttachment).toHaveBeenCalledWith({ name: 'pasted-context.txt', content: 'pasted text', source: 'paste' });
        act(() => { button('Remove notes.md').click(); });
        expect(uiApi.api.removeAttachment).toHaveBeenCalledWith('notes.md');
        act(() => { button('Clear attachments').click(); });
        expect(uiApi.api.clearAttachments).toHaveBeenCalledOnce();
        const templates = host.querySelectorAll<HTMLButtonElement>('button[data-template]');
        expect(Array.from(templates).map((node) => node.dataset.template)).toEqual(['ux', 'api', 'audit', 'report', 'error', 'test']);
        for (const template of Array.from(templates)) act(() => { template.click(); });
        expect(uiApi.api.templatePrompt).toHaveBeenCalledTimes(6);
        expect(host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]')!.value).toBe('Prompt for test');
        uiApi.attachments.push({ name: 'external.txt', content: 'external context', source: 'workspace' });
        act(() => { document.dispatchEvent(new CustomEvent('uor-attachments-changed')); });
        expect(host.textContent).toContain('external.txt');
        expect(button('Remove external.txt')).not.toBeNull();
    });

    it('keeps inserted filenames unique when timestamps collide', () => {
        const { store, sessionApi, fileApi } = mount();
        sessionApi.getConversationMessages = vi.fn(() => [{ role: 'assistant', content: 'First' }, { role: 'assistant', content: 'Second' }]);
        const now = vi.spyOn(Date, 'now').mockReturnValue(123);
        act(() => { store.dispatch({ type: 'session/activated' }); });
        const buttons = Array.from(host.querySelectorAll('article[data-role="assistant"]')).map((article) => Array.from(article.querySelectorAll('button')).find((node) => node.textContent === 'Insert into editor')!);
        act(() => { buttons[0].click(); buttons[1].click(); });
        const names = vi.mocked(fileApi.insertText).mock.calls.map((call) => call[1]);
        expect(names[0]).not.toBe(names[1]);
        expect(names[0]).toMatch(/^agent-message-123-\d+\.md$/);
        expect(names[1]).toMatch(/^agent-message-123-\d+\.md$/);
        now.mockRestore();
    });

    it('forwards message copy, insert, save, and retry actions', () => {
        const { store, sessionApi, fileApi, generation, copy, download } = mount();
        const messages = [{ role: 'user', content: 'Original prompt' }, { role: 'assistant', content: 'Assistant answer' }];
        sessionApi.getConversationMessages = vi.fn(() => messages);
        act(() => { store.dispatch({ type: 'session/activated' }); });
        const assistant = host.querySelector('article[data-role="assistant"]')!;
        const action = (label: string) => Array.from(assistant.querySelectorAll('button')).find((node) => node.textContent === label)!;
        act(() => { action('Copy').click(); });
        expect(copy).toHaveBeenCalledWith('Assistant answer');
        act(() => { action('Insert into editor').click(); });
        expect(fileApi.insertText).toHaveBeenCalledWith('Assistant answer', expect.stringMatching(/^agent-message-\d+-\d+\.md$/));
        act(() => { action('Save as file').click(); });
        expect(download).toHaveBeenCalledWith('studio-message.md', 'Assistant answer');
        act(() => { store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' }); });
        act(() => { action('Retry').click(); });
        expect(generation.send).toHaveBeenCalledWith('Original prompt');
    });

    it('renders rich assistant content while preserving exact raw message actions', () => {
        const { store, sessionApi, fileApi, generation, copy, download } = mount();
        const assistantContent = '# Answer\n\nUse **bold** and `code`.';
        sessionApi.getConversationMessages = vi.fn(() => [{ role: 'user', content: 'Original prompt' }, { role: 'assistant', content: assistantContent }]);
        act(() => { store.dispatch({ type: 'session/activated' }); });
        const assistant = host.querySelector('article[data-role="assistant"]')!;
        expect(assistant.querySelector('h1')?.textContent).toBe('Answer');
        expect(assistant.querySelector('.rich-message strong')?.textContent).toBe('bold');
        expect(assistant.querySelector('.rich-message code')?.textContent).toBe('code');
        const action = (label: string) => Array.from(assistant.querySelectorAll('button')).find((node) => node.textContent === label)!;
        act(() => { action('Copy').click(); });
        expect(copy).toHaveBeenCalledWith(assistantContent);
        act(() => { action('Insert into editor').click(); });
        expect(fileApi.insertText).toHaveBeenCalledWith(assistantContent, expect.stringMatching(/^agent-message-\d+-\d+\.md$/));
        act(() => { action('Save as file').click(); });
        expect(download).toHaveBeenCalledWith('studio-message.md', assistantContent);
        act(() => { store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' }); });
        act(() => { action('Retry').click(); });
        expect(generation.send).toHaveBeenCalledWith('Original prompt');
    });

    it('preserves the current unsent draft when retrying an older message', () => {
        const { store, sessionApi } = mount();
        sessionApi.getConversationMessages = vi.fn(() => [{ role: 'user', content: 'Original prompt' }, { role: 'assistant', content: 'Assistant answer' }]);
        act(() => { store.dispatch({ type: 'session/activated' }); });
        act(() => { store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' }); });
        prompt('Unsent current draft');
        const assistant = host.querySelector('article[data-role="assistant"]')!;
        const retry = Array.from(assistant.querySelectorAll('button')).find((node) => node.textContent === 'Retry')!;
        act(() => { retry.click(); });
        expect(host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]')!.value).toBe('Unsent current draft');
    });
});

function pressKey(textarea: HTMLTextAreaElement, key: string, init: { shiftKey?: boolean; isComposing?: boolean; keyCode?: number } = {}) {
    const event = new KeyboardEvent('keydown', { key, shiftKey: init.shiftKey, bubbles: true, cancelable: true });
    Object.defineProperties(event, {
        isComposing: { value: Boolean(init.isComposing) },
        keyCode: { value: init.keyCode || 0 },
    });
    textarea.dispatchEvent(event);
    return event;
}

describe('composer interactions', () => {
    it('submits on Enter, recalls submitted prompts, restores the draft, and refuses IME confirmation', () => {
        const { store, generation } = mount();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        const textarea = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]')!;
        prompt('first\nline');
        pressKey(textarea, 'Enter');
        expect(generation.send).toHaveBeenCalledWith('first\nline');
        prompt('second');
        pressKey(textarea, 'ArrowUp');
        expect(textarea.value).toBe('first\nline');
        pressKey(textarea, 'ArrowDown');
        expect(textarea.value).toBe('second');
        prompt('in progress');
        pressKey(textarea, 'Enter', { isComposing: true });
        pressKey(textarea, 'Enter', { keyCode: 229 });
        expect(generation.send).toHaveBeenCalledTimes(1);
    });

    it('exposes the six build-task commands with accessible keyboard and mouse selection', () => {
        const { store, generation } = mount();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        const textarea = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]')!;
        prompt('/');
        let menu = host.querySelector<HTMLElement>('[role="listbox"][aria-label="Build task commands"]')!;
        expect(menu).not.toBeNull();
        expect(textarea.getAttribute('aria-expanded')).toBe('true');
        expect(textarea.getAttribute('aria-controls')).toBe(menu.id);
        let options = Array.from(menu.querySelectorAll<HTMLElement>('[role="option"]'));
        expect(options).toHaveLength(6);
        expect(options[0].getAttribute('aria-selected')).toBe('true');
        expect(textarea.getAttribute('aria-activedescendant')).toBe(options[0].id);
        expect(document.activeElement).toBe(textarea);
        pressKey(textarea, 'ArrowDown');
        expect(textarea.getAttribute('aria-activedescendant')).toBe(options[1].id);
        pressKey(textarea, 'ArrowUp');
        expect(textarea.getAttribute('aria-activedescendant')).toBe(options[0].id);
        pressKey(textarea, 'End');
        expect(textarea.getAttribute('aria-activedescendant')).toBe(options[5].id);
        expect(options[5].getAttribute('aria-selected')).toBe('true');
        pressKey(textarea, 'Home');
        expect(textarea.getAttribute('aria-activedescendant')).toBe(options[0].id);
        pressKey(textarea, 'End');
        pressKey(textarea, 'Enter');
        expect(textarea.value).toBe('Prompt for test');
        expect(document.activeElement).toBe(textarea);
        expect(generation.send).not.toHaveBeenCalled();
        prompt('/rep');
        menu = host.querySelector<HTMLElement>('[role="listbox"][aria-label="Build task commands"]')!;
        const report = Array.from(menu.querySelectorAll<HTMLElement>('[role="option"]')).find((option) => option.dataset.template === 'report')!;
        const mouseDown = new MouseEvent('mousedown', { bubbles: true, cancelable: true });
        act(() => { report.dispatchEvent(mouseDown); report.click(); });
        expect(mouseDown.defaultPrevented).toBe(true);
        expect(textarea.value).toBe('Prompt for report');
        expect(document.activeElement).toBe(textarea);
        expect(host.querySelector('[role="listbox"][aria-label="Build task commands"]')).toBeNull();
        prompt('/');
        pressKey(textarea, 'Escape');
        expect(host.querySelector('[role="listbox"][aria-label="Build task commands"]')).toBeNull();
        expect(document.activeElement).toBe(textarea);
        expect(generation.send).not.toHaveBeenCalled();
    });

    it('preserves normal Enter submission for unmatched slash and mention tokens', () => {
        const { store, generation } = mount();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        const textarea = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]')!;
        prompt('/unknown');
        expect(host.querySelector('[role="listbox"]')).toBeNull();
        pressKey(textarea, 'Enter');
        expect(generation.send).toHaveBeenCalledWith('/unknown');
        prompt('@unknown');
        expect(host.querySelector('[role="listbox"]')).toBeNull();
        pressKey(textarea, 'Enter');
        expect(generation.send).toHaveBeenCalledWith('@unknown');
    });

    it('inserts mentions with complete keyboard, mouse, focus, and empty-state behavior', () => {
        const { store, generation, fileApi, uiApi } = mount();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        const textarea = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]')!;
        uiApi.attachments.push({ name: 'external.txt', content: 'external context', source: 'workspace' });
        prompt('@');
        let menu = host.querySelector<HTMLElement>('[role="listbox"][aria-label="Attachment mentions"]')!;
        expect(menu).not.toBeNull();
        const options = Array.from(menu.querySelectorAll<HTMLElement>('[role="option"]'));
        expect(options).toHaveLength(2);
        expect(options[0].textContent).toContain('notes.md');
        expect(options[0].getAttribute('aria-selected')).toBe('true');
        pressKey(textarea, 'End');
        expect(textarea.getAttribute('aria-activedescendant')).toBe(options[1].id);
        pressKey(textarea, 'Home');
        expect(textarea.getAttribute('aria-activedescendant')).toBe(options[0].id);
        pressKey(textarea, 'ArrowUp');
        expect(textarea.getAttribute('aria-activedescendant')).toBe(options[1].id);
        pressKey(textarea, 'ArrowDown');
        expect(textarea.getAttribute('aria-activedescendant')).toBe(options[0].id);
        pressKey(textarea, 'Enter');
        expect(textarea.value).toBe('@notes.md');
        expect(document.activeElement).toBe(textarea);
        expect(uiApi.api.addAttachment).toHaveBeenCalledTimes(1);
        expect(uiApi.api.addAttachment).toHaveBeenCalledWith({ name: 'notes.md', content: 'context text', source: 'editor' });
        expect(generation.send).not.toHaveBeenCalled();
        prompt('@no');
        menu = host.querySelector<HTMLElement>('[role="listbox"][aria-label="Attachment mentions"]')!;
        const again = Array.from(menu.querySelectorAll<HTMLElement>('[role="option"]')).find((option) => option.textContent?.includes('notes.md'))!;
        act(() => { again.click(); });
        expect(document.activeElement).toBe(textarea);
        expect(uiApi.api.addAttachment).toHaveBeenCalledTimes(1);
        prompt('@ext');
        menu = host.querySelector<HTMLElement>('[role="listbox"][aria-label="Attachment mentions"]')!;
        const existing = Array.from(menu.querySelectorAll<HTMLElement>('[role="option"]')).find((option) => option.textContent?.includes('external.txt'))!;
        act(() => { existing.click(); });
        expect(textarea.value).toBe('@external.txt');
        expect(document.activeElement).toBe(textarea);
        expect(uiApi.api.addAttachment).not.toHaveBeenCalledWith({ name: 'external.txt', content: 'external context', source: 'workspace' });
        prompt('@no');
        pressKey(textarea, 'Escape');
        expect(host.querySelector('[role="listbox"]')).toBeNull();
        expect(document.activeElement).toBe(textarea);
        uiApi.attachments.splice(0);
        fileApi.activeFile = vi.fn(() => null);
        act(() => { document.dispatchEvent(new CustomEvent('uor-attachments-changed')); });
        prompt('@');
        expect(host.querySelector('[role="listbox"]')).toBeNull();
        expect(textarea.getAttribute('aria-expanded')).toBe('false');
        expect(generation.send).not.toHaveBeenCalled();
    });

    it('forwards ordinary draft input once without persisting history edits or duplicate send clears', () => {
        const onDraftChange = vi.fn();
        const { store, generation } = mount({ onDraftChange });
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        const textarea = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]')!;
        const calls = () => onDraftChange.mock.calls.map(([value]) => value);
        prompt('first');
        expect(calls()).toEqual(['first']);
        pressKey(textarea, 'Enter');
        expect(generation.send).toHaveBeenCalledWith('first');
        expect(calls()).toEqual(['first', '']);
        act(() => { store.dispatch({ type: 'generation/finished', outcome: 'complete', generationId: 1 }); });
        onDraftChange.mockClear();
        prompt('in progress');
        pressKey(textarea, 'ArrowUp');
        expect(textarea.value).toBe('first');
        expect(calls()).toEqual(['in progress']);
        act(() => { textarea.value = 'edited history'; textarea.dispatchEvent(new Event('input', { bubbles: true })); });
        expect(calls()).toEqual(['in progress']);
        pressKey(textarea, 'ArrowDown');
        expect(textarea.value).toBe('in progress');
        expect(calls()).toEqual(['in progress']);
        pressKey(textarea, 'Enter');
        expect(generation.send).toHaveBeenCalledWith('in progress');
        expect(calls()).toEqual(['in progress', '']);
    });

    it('preserves the visible draft and history when sending fails', () => {
        const onDraftChange = vi.fn();
        const { store, generation } = mount({ onDraftChange });
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        vi.mocked(generation.send).mockReturnValue(false);
        const textarea = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]')!;
        prompt('keep me');
        onDraftChange.mockClear();
        pressKey(textarea, 'Enter');
        expect(generation.send).toHaveBeenCalledWith('keep me');
        expect(textarea.value).toBe('keep me');
        expect(onDraftChange).not.toHaveBeenCalled();
        pressKey(textarea, 'ArrowUp');
        expect(textarea.value).toBe('keep me');
    });

    it('uses the visible recalled prompt for mouse Send with controlled draft and pasted props', () => {
        const onDraftChange = vi.fn();
        const onPastedChange = vi.fn();
        const { store, generation } = mount({ controlled: true, initialDraft: 'saved', initialPasted: 'saved paste', onDraftChange, onPastedChange });
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        const textarea = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]')!;
        const pasted = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Pasted context"]')!;
        expect(textarea.value).toBe('saved');
        expect(pasted.value).toBe('saved paste');
        act(() => { pasted.value = 'edited paste'; pasted.dispatchEvent(new Event('input', { bubbles: true })); });
        expect(pasted.value).toBe('edited paste');
        expect(onPastedChange).toHaveBeenCalledOnce();
        expect(onPastedChange).toHaveBeenCalledWith('edited paste');
        prompt('first');
        expect(onDraftChange).toHaveBeenCalledWith('first');
        pressKey(textarea, 'Enter');
        expect(generation.send).toHaveBeenCalledWith('first');
        act(() => { store.dispatch({ type: 'generation/finished', outcome: 'complete', generationId: 1 }); });
        expect(textarea.value).toBe('');
        pressKey(textarea, 'ArrowUp');
        expect(textarea.value).toBe('first');
        expect(button('Send').disabled).toBe(false);
        act(() => { button('Send').click(); });
        expect(generation.send).toHaveBeenCalledTimes(2);
        expect(generation.send).toHaveBeenLastCalledWith('first');
        expect(textarea.value).toBe('');
    });

    it('applies external controlled draft and pasted replacements without retention callbacks', () => {
        const onDraftChange = vi.fn();
        const onPastedChange = vi.fn();
        const { update } = mount({ initialDraft: 'saved', initialPasted: 'saved paste', onDraftChange, onPastedChange });
        update({ draft: 'external draft', pasted: 'external paste' });
        expect(host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]')!.value).toBe('external draft');
        expect(host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Pasted context"]')!.value).toBe('external paste');
        expect(onDraftChange).not.toHaveBeenCalled();
        expect(onPastedChange).not.toHaveBeenCalled();
    });

    it('splices mentions into the visible recalled prompt without losing surrounding text', () => {
        const { store, generation } = mount();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        const textarea = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]')!;
        prompt('old');
        pressKey(textarea, 'Enter');
        act(() => { store.dispatch({ type: 'generation/finished', outcome: 'complete', generationId: 1 }); });
        pressKey(textarea, 'ArrowUp');
        expect(textarea.value).toBe('old');
        prompt('old @');
        const option = host.querySelector<HTMLElement>('[role="option"][data-attachment="notes.md"]')!;
        expect(option).not.toBeNull();
        act(() => { option.click(); });
        expect(textarea.value).toBe('old @notes.md');
        expect(generation.send).toHaveBeenCalledTimes(1);
    });

    it('resets history navigation on programmatic template changes without erasing history', () => {
        const { store, generation } = mount();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        const textarea = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]')!;
        prompt('old');
        pressKey(textarea, 'Enter');
        act(() => { store.dispatch({ type: 'generation/finished', outcome: 'complete', generationId: 1 }); });
        prompt('in progress');
        pressKey(textarea, 'ArrowUp');
        expect(textarea.value).toBe('old');
        act(() => { button('Write a verification plan').click(); });
        expect(textarea.value).toBe('Prompt for test');
        pressKey(textarea, 'ArrowDown');
        expect(textarea.value).toBe('Prompt for test');
        pressKey(textarea, 'ArrowUp');
        expect(textarea.value).toBe('old');
        expect(generation.send).toHaveBeenCalledTimes(1);
    });

    it('passes picked files through the typed composer controller', async () => {
        const { uiApi } = mount();
        const picker = host.querySelector<HTMLInputElement>('input[type="file"]')!;
        const file = new File(['contents'], 'note.txt', { type: 'text/plain' });
        Object.defineProperty(picker, 'files', { value: [file], configurable: true });
        await act(async () => {
            picker.dispatchEvent(new Event('change', { bubbles: true }));
            await vi.waitFor(() => {
                expect(uiApi.api.addAttachment).toHaveBeenCalledWith({ name: 'note.txt', kind: 'text', content: 'contents', source: 'file', size: 8 });
            });
        });
    });

    it('does not count retained images against the picked text-file budget', async () => {
        const { uiApi } = mount();
        uiApi.attachments.push({ name: 'photo.png', kind: 'image', content: '', size: 4_194_305 });
        const picker = host.querySelector<HTMLInputElement>('input[type="file"]')!;
        const file = new File(['contents'], 'note.txt', { type: 'text/plain' });
        Object.defineProperty(picker, 'files', { value: [file], configurable: true });
        await act(async () => {
            picker.dispatchEvent(new Event('change', { bubbles: true }));
            await vi.waitFor(() => {
                expect(uiApi.api.addAttachment).toHaveBeenCalledWith({ name: 'note.txt', kind: 'text', content: 'contents', source: 'file', size: 8 });
            });
        });
        expect(uiApi.api.toast).not.toHaveBeenCalled();
    });
});
