import { render } from 'preact';
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
            clearSessions: vi.fn(() => ({ cleared: 0, total: 0 })), clearEditor: vi.fn(),
        } satisfies StudioUiApi,
    };
}

function mount() {
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
    act(() => { render(<AgentComposer store={store} sessions={sessionApi} files={fileApi} ui={uiApi.api} generation={generation} copy={copy} download={download} />, host); });
    return { store, sessionApi, fileApi, uiApi, generation, copy, download };
}

function button(label: string): HTMLButtonElement {
    const found = Array.from(host.querySelectorAll('button')).find((node) => node.textContent === label);
    if (!found) throw new Error(`Missing button ${label}`);
    return found;
}

function prompt(value: string) {
    const input = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]')!;
    act(() => { input.value = value; input.dispatchEvent(new Event('input', { bubbles: true })); });
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
