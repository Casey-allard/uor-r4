import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LifecycleStore } from '../state/lifecycle.js';
import { AgentClient, type TaskSummary } from '../agent/client.js';
import { FixtureTransport } from '../agent/fixtureTransport.js';
import { canonicalJson } from '../agent/generated/agentCanonical.js';
import { createAgentStore, type AgentIncomingEvent } from '../state/agent.js';
import type { Attachment } from '../adapters/workflow.js';
import { createDesktopActions } from '../adapters/desktopActions.js';
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
beforeEach(() => {
    host = document.createElement('div');
    document.body.appendChild(host);
    Object.defineProperty(HTMLDialogElement.prototype, 'showModal', { configurable: true, value: vi.fn(function (this: HTMLDialogElement) { this.open = true; }) });
    Object.defineProperty(HTMLDialogElement.prototype, 'close', { configurable: true, value: vi.fn(function (this: HTMLDialogElement) { this.open = false; this.dispatchEvent(new Event('close')); }) });
});
afterEach(() => { render(null, host); host.remove(); vi.restoreAllMocks(); });

function destination(label: string): HTMLButtonElement {
    const navigation = host.querySelector('nav[aria-label="Studio"]')!;
    return Array.from(navigation.querySelectorAll('button')).find((node) => node.textContent === label)!;
}

function button(label: string): HTMLButtonElement {
    const found = Array.from(host.querySelectorAll('button')).find((node) => node.textContent === label);
    if (!found) throw new Error(`Missing button ${label}`);
    return found;
}

describe('StudioApp', () => {
    it('composes all six destinations with one status source and on-demand inspector', () => {
        const store = new LifecycleStore();
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        expect(host.querySelectorAll('main')).toHaveLength(1);
        expect(host.querySelectorAll('[role="status"]')).toHaveLength(1);
        expect(host.textContent).toContain('No messages yet');
        act(() => { destination('Workspace').click(); });
        expect(host.textContent).toContain('No folder open');
        act(() => { destination('Models').click(); });
        expect(host.textContent).toContain('API providers');
        act(() => { destination('Guide').click(); });
        expect(host.textContent).toContain('next-token prediction');
        expect(host.textContent).toContain('not a conversational assistant');
        act(() => { button('Command palette').click(); });
        const paletteInput = host.querySelector<HTMLInputElement>('input[aria-label="Search commands"]')!;
        act(() => { paletteInput.value = 'Go to Changes'; paletteInput.dispatchEvent(new Event('input', { bubbles: true })); });
        act(() => { paletteInput.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true })); });
        expect(host.textContent).toContain('Git requires the local companion. Uploaded files can still be reviewed locally.');
        act(() => { destination('Terminal').click(); });
        expect(host.textContent).toContain('Interactive terminal requires the local companion. This browser build cannot start a shell.');
        act(() => { destination('Agent').click(); });
        expect(host.textContent).toContain('No messages yet');
        act(() => { button('Inspector').click(); });
        expect(host.querySelector('#studio-inspector')).not.toBeNull();
        act(() => { button('Close inspector').click(); });
        expect(host.querySelector('#studio-inspector')).toBeNull();
        act(() => { button('Settings').click(); });
        expect(host.textContent).toContain('Appearance');
        expect(host.querySelector('dialog')).not.toBeNull();
    });

    it('does not label an API selection without an endpoint as local execution', () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: undefined as unknown as string });
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        expect(host.querySelector('[role="status"]')?.textContent).toContain('Invalid API endpoint');
        expect(host.querySelector('[role="status"]')?.textContent).not.toContain('localWASM');
    });

    it('uses lifecycle transitions for palette saves', async () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'file/opened', path: 'a.md' });
        store.dispatch({ type: 'file/dirty' });
        const files = makeFiles();
        files.activeFile = vi.fn(() => ({ path: 'a.md', name: 'a.md', content: 'edited', mode: 'local' }));
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={files} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        act(() => { button('Command palette').click(); });
        const input = host.querySelector<HTMLInputElement>('input[aria-label="Search commands"]')!;
        act(() => { input.value = 'Save file'; input.dispatchEvent(new Event('input', { bubbles: true })); });
        await act(async () => {
            input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }));
            await Promise.resolve();
        });
        expect(store.getState().file.phase).toBe('saved');
    });

    it('pins the top status source while a generation is active', () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        store.dispatch({ type: 'generation/started', source: { kind: 'api', id: 'api-a' } });
        store.dispatch({ type: 'artifact/selected', sourceId: 'model-b' });
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        expect(host.querySelector('[role="status"]')?.textContent).toContain('api: api-a');
        expect(host.querySelector('[role="status"]')?.textContent).not.toContain('artifact: model-b');
    });

    it('does not report an unmatched resident as ready in the top status', () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'artifact/selected', sourceId: 'model-a' });
        store.dispatch({ type: 'artifact/ready', operation: store.getState().operation!, artifact: { sourceId: 'model-b', schema: 'uor-r4.lut-stack/1', canGenerate: true } });
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        expect(host.querySelector('[role="status"]')?.textContent).toContain('Selected model; not ready');
        expect(host.querySelector('[role="status"]')?.textContent).not.toContain('Artifact: ready');
    });

    it('uses current selection privacy after a previous generation completes', () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        store.dispatch({ type: 'generation/started', source: { kind: 'api', id: 'api-a' } });
        store.dispatch({ type: 'generation/finished', outcome: 'complete', generationId: 1 });
        store.dispatch({ type: 'artifact/selected', sourceId: 'model-a' });
        act(() => {
            store.dispatch({ type: 'artifact/ready', operation: store.getState().operation!, artifact: { schema: 'uor-r4.lut-stack/1', canGenerate: true } });
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        expect(host.querySelector('[role="status"]')?.textContent).toContain('localWASM');
        expect(host.querySelector('[role="status"]')?.textContent).not.toContain('remoteAPI');
    });

    it('follows session and insert navigation events into coherent workflow state', () => {
        const store = new LifecycleStore();
        const sessions = makeSessions();
        const files = makeFiles();
        const items: Array<{ id: string; title: string; createdAt: number; updatedAt: number; messageCount: number; preview: string }> = [];
        let activeId: string | null = null;
        sessions.listSessions = vi.fn(() => items);
        sessions.activeSessionId = vi.fn(() => activeId);
        sessions.newSession = vi.fn(() => {
            items.unshift({ id: 'session-created', title: 'New conversation', createdAt: 1, updatedAt: 1, messageCount: 0, preview: '' });
            activeId = 'session-created';
            document.dispatchEvent(new CustomEvent('uor-view-changed', { detail: { view: 'chat' } }));
            return true;
        });
        sessions.getConversationMessages = vi.fn(() => [{ role: 'assistant', content: 'Answer' }]);
        files.insertText = vi.fn((_text: string, name?: string) => {
            const path = name || 'agent-message.md';
            const summary = {
                mode: 'local', rootName: 'project', fileCount: 1,
                tabs: [{ path, dirty: true }], activePath: path,
            };
            files.treeSummary = vi.fn(() => summary);
            document.dispatchEvent(new CustomEvent('uor-files-changed', { detail: summary }));
            document.dispatchEvent(new CustomEvent('uor-view-changed', { detail: { view: 'workspace' } }));
            return true;
        });
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={files} sessions={sessions} ui={makeUi()} download={vi.fn()} />, host);
        });
        act(() => { destination('Models').click(); });
        act(() => { button('New session').click(); });
        expect(store.getState().session.activeId).toBe('session-created');
        expect(destination('Agent').getAttribute('aria-current')).toBe('page');
        const insert = Array.from(host.querySelectorAll<HTMLButtonElement>('article[data-role="assistant"] button')).find((node) => node.textContent === 'Insert into editor')!;
        act(() => { insert.click(); });
        expect(destination('Workspace').getAttribute('aria-current')).toBe('page');
        expect(store.getState().file.path).toMatch(/^agent-message-\d+-\d+\.md$/);
        expect(store.getState().file.phase).toBe('dirty');
        expect(store.canSave()).toBe(true);
    });

    it('preserves unsent prompt and pasted context across section navigation', () => {
        const store = new LifecycleStore();
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        const prompt = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]')!;
        const pasted = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Pasted context"]')!;
        act(() => { prompt.value = 'Keep this prompt'; prompt.dispatchEvent(new Event('input', { bubbles: true })); });
        act(() => { pasted.value = 'Keep this context'; pasted.dispatchEvent(new Event('input', { bubbles: true })); });
        act(() => { destination('Models').click(); });
        act(() => { destination('Agent').click(); });
        expect(host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]')!.value).toBe('Keep this prompt');
        expect(host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Pasted context"]')!.value).toBe('Keep this context');
    });

    it('propagates committed Settings values to the composed WorkspaceEditor', () => {
        const store = new LifecycleStore();
        const files = makeFiles();
        const ui = makeUi();
        store.dispatch({ type: 'folder/opened', path: 'project' });
        store.dispatch({ type: 'file/opened', path: 'a.md' });
        files.activeFile = vi.fn(() => ({ path: 'a.md', name: 'a.md', content: 'a', mode: 'local' }));
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={files} sessions={makeSessions()} ui={ui} download={vi.fn()} />, host);
        });
        act(() => { destination('Workspace').click(); });
        act(() => { button('Settings').click(); });
        const font = host.querySelector<HTMLInputElement>('input[aria-label="Editor font size"]')!;
        act(() => { font.value = '1'; font.dispatchEvent(new Event('input', { bubbles: true })); });
        expect(font.isConnected).toBe(true);
        act(() => { font.value = '16'; font.dispatchEvent(new Event('input', { bubbles: true })); });
        expect(font.isConnected).toBe(true);
        act(() => { font.focus(); font.blur(); });
        const tab = host.querySelector<HTMLInputElement>('input[aria-label="Tab size"]')!;
        act(() => { tab.value = '2'; tab.dispatchEvent(new Event('input', { bubbles: true })); tab.focus(); tab.blur(); });
        const wrap = host.querySelector<HTMLInputElement>('input[aria-label="Word wrap"]')!;
        act(() => { wrap.checked = true; wrap.dispatchEvent(new Event('change', { bubbles: true })); });
        const reduce = host.querySelector<HTMLInputElement>('input[aria-label="Reduce motion"]')!;
        act(() => { reduce.checked = true; reduce.dispatchEvent(new Event('change', { bubbles: true })); });
        const editor = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="File contents"]')!;
        expect(ui.updateSettings).toHaveBeenCalledWith({ fontSize: 16 });
        expect(host.querySelector('div[data-reduce-motion]')?.getAttribute('data-reduce-motion')).toBe('1');
        expect(editor.style.fontSize).toBe('16px');
        expect(editor.getAttribute('wrap')).toBe('soft');
        editor.setSelectionRange(1, 1);
        act(() => { editor.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true, cancelable: true })); });
        expect(files.setActiveContent).toHaveBeenCalledWith('a  ');
    });

    it('clears a session error after a successful duplicate', () => {
        const store = new LifecycleStore();
        const sessions = makeSessions();
        const items = [{ id: 'session-a', title: 'Session A', createdAt: 1, updatedAt: 2, messageCount: 1, preview: 'message' }];
        sessions.listSessions = vi.fn(() => items);
        sessions.activeSessionId = vi.fn(() => 'session-a');
        sessions.duplicateSession = vi.fn().mockReturnValueOnce(null).mockImplementationOnce(() => {
            items.unshift({ id: 'session-copy', title: 'Session A (copy)', createdAt: 3, updatedAt: 3, messageCount: 1, preview: 'message' });
            return 'session-copy';
        });
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={sessions} ui={makeUi()} download={vi.fn()} />, host);
        });
        act(() => { button('Duplicate Session A').click(); });
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('Could not duplicate');
        act(() => { button('Duplicate Session A').click(); });
        expect(host.querySelector('[role="alert"]')).toBeNull();
    });

    it('clears a provider error after a successful selection retry', () => {
        const store = new LifecycleStore();
        const sources = makeSources();
        const adapter = makeModelAdapter();
        const provider = { id: 'api-a', label: 'API A', preset: 'custom', baseUrl: 'https://api.example.com/v1', model: 'model-a', apiKey: '' };
        sources.list = vi.fn(() => ({ active: { kind: 'wasm', id: 'bundled-demo' }, wasm: {}, api: [provider] }));
        adapter.selectApi = vi.fn()
            .mockImplementationOnce(() => { throw new Error('storage full'); })
            .mockImplementationOnce((id: string, endpoint: string) => store.dispatch({ type: 'api/selected', sourceId: id, endpoint }));
        act(() => {
            render(<StudioApp store={store} adapter={adapter} sources={sources} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        act(() => { destination('Models').click(); });
        act(() => { button('Use API A').click(); });
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('storage full');
        act(() => { button('Use API A').click(); });
        expect(host.querySelector('[role="alert"]')).toBeNull();
    });

    it('clears the top-level error after a successful retry', async () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'file/opened', path: 'a.md' });
        store.dispatch({ type: 'file/dirty' });
        const files = makeFiles();
        files.treeSummary = vi.fn(() => ({ mode: 'local', rootName: 'project', fileCount: 1, tabs: [{ path: 'a.md', dirty: true }], activePath: 'a.md' }));
        files.activeFile = vi.fn(() => ({ path: 'a.md', name: 'a.md', content: 'edited', mode: 'local' }));
        let successful = true;
        files.saveActive = vi.fn(async () => {
            successful = !successful;
            files.treeSummary = vi.fn(() => ({ mode: 'local', rootName: 'project', fileCount: 1, tabs: [{ path: 'a.md', dirty: !successful }], activePath: 'a.md' }));
            return successful;
        });
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={files} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        act(() => { destination('Workspace').click(); });
        await act(async () => { button('Save').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(store.getState().file.phase).toBe('error'));
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('Save did not complete.');
        await act(async () => { button('Save').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(store.getState().file.phase).toBe('saved'));
        expect(host.querySelector('[role="alert"]')).toBeNull();
    });

    it('closes lifecycle file state when editor tabs are cleared outside Workspace', () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'file/opened', path: 'a.md' });
        store.dispatch({ type: 'file/dirty' });
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        act(() => { button('Settings').click(); });
        act(() => { button('Clear editor tabs').click(); });
        act(() => { button('Confirm clear editor tabs').click(); });
        expect(store.getState().file.phase).toBe('no-open-file');
        expect(store.getState().file.path).toBeNull();
    });

    it('refreshes the session list after confirmed clearing', () => {
        const store = new LifecycleStore();
        const sessions = makeSessions();
        const ui = makeUi();
        let items = [{ id: 'session-a', title: 'Session A', createdAt: 1, updatedAt: 2, messageCount: 1, preview: 'message' }];
        sessions.listSessions = vi.fn(() => items);
        sessions.activeSessionId = vi.fn(() => items[0]?.id ?? null);
        ui.clearSessions = vi.fn(() => { items = []; return { cleared: 1, total: 1 }; });
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={sessions} ui={ui} download={vi.fn()} />, host);
        });
        expect(host.textContent).toContain('Session A');
        act(() => { button('Settings').click(); });
        act(() => { button('Clear sessions').click(); });
        act(() => { button('Confirm clear sessions').click(); });
        expect(host.textContent).toContain('No sessions yet');
        expect(store.getState().session.phase).toBe('new');
    });

    it('blocks palette and settings session mutations during generation', () => {
        const store = new LifecycleStore();
        const sessions = makeSessions();
        const ui = makeUi();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        store.dispatch({ type: 'generation/started', source: { kind: 'api', id: 'api-a' } });
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={sessions} ui={ui} download={vi.fn()} />, host);
        });
        act(() => { button('Command palette').click(); });
        const paletteNew = host.querySelector<HTMLButtonElement>('button[data-command-id="new-session"]')!;
        expect(paletteNew.disabled).toBe(true);
        act(() => { host.querySelector<HTMLDialogElement>('dialog')!.close(); });
        act(() => { button('Settings').click(); });
        expect(button('Clear sessions').disabled).toBe(true);
        expect(sessions.newSession).not.toHaveBeenCalled();
        expect(ui.clearSessions).not.toHaveBeenCalled();
        expect(store.getState().session.activeGeneration).not.toBeNull();
    });

    it('clears a parent export error after a successful export retry', () => {
        const store = new LifecycleStore();
        const sessions = makeSessions();
        sessions.listSessions = vi.fn(() => [{ id: 'session-a', title: 'Session A', createdAt: 1, updatedAt: 2, messageCount: 1, preview: 'message' }]);
        sessions.activeSessionId = vi.fn(() => 'session-a');
        sessions.exportSession = vi.fn().mockReturnValueOnce(null).mockReturnValueOnce('{"schema":"uor-r4.studio-session/1"}');
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={sessions} ui={makeUi()} download={vi.fn()} />, host);
        });
        act(() => { button('Export Session A').click(); });
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('Session export is unavailable.');
        act(() => { button('Export Session A').click(); });
        expect(host.querySelector('[role="alert"]')).toBeNull();
    });

    it('clears a parent artifact-load error after a successful retry', async () => {
        const store = new LifecycleStore();
        const adapter = makeModelAdapter();
        adapter.loadBundled = vi.fn().mockRejectedValueOnce(new Error('load failed')).mockResolvedValueOnce({ bytes: new Uint8Array(), fileName: 'model.lut', meta: { schema: 'uor-r4.lut-stack/1' } });
        act(() => {
            render(<StudioApp store={store} adapter={adapter} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        act(() => { destination('Models').click(); });
        await act(async () => { button('Load bundled demo').click(); await Promise.resolve(); });
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('load failed');
        await act(async () => { button('Load bundled demo').click(); await Promise.resolve(); });
        expect(host.querySelector('[role="alert"]')).toBeNull();
    });

    it('clears a palette artifact-load error after a successful retry', async () => {
        const store = new LifecycleStore();
        const adapter = makeModelAdapter();
        adapter.loadBundled = vi.fn().mockRejectedValueOnce(new Error('palette load failed')).mockResolvedValueOnce({ bytes: new Uint8Array(), fileName: 'model.lut', meta: { schema: 'uor-r4.lut-stack/1' } });
        act(() => {
            render(<StudioApp store={store} adapter={adapter} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        act(() => { button('Command palette').click(); });
        await act(async () => { host.querySelector<HTMLButtonElement>('button[data-command-id="load-demo"]')!.click(); await Promise.resolve(); });
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('palette load failed');
        act(() => { button('Command palette').click(); });
        await act(async () => { host.querySelector<HTMLButtonElement>('button[data-command-id="load-demo"]')!.click(); await Promise.resolve(); });
        expect(host.querySelector('[role="alert"]')).toBeNull();
    });

    it('clears a SessionList new-session error after a palette retry', () => {
        const store = new LifecycleStore();
        const sessions = makeSessions();
        const items: Array<{ id: string; title: string; createdAt: number; updatedAt: number; messageCount: number; preview: string }> = [];
        let activeId: string | null = null;
        sessions.listSessions = vi.fn(() => items);
        sessions.activeSessionId = vi.fn(() => activeId);
        sessions.newSession = vi.fn().mockReturnValueOnce(false).mockImplementationOnce(() => {
            items.unshift({ id: 'session-new', title: 'New conversation', createdAt: 1, updatedAt: 1, messageCount: 0, preview: '' });
            activeId = 'session-new';
            return true;
        });
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={sessions} ui={makeUi()} download={vi.fn()} />, host);
        });
        act(() => { button('New session').click(); });
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('Could not create a session.');
        act(() => { button('Command palette').click(); });
        act(() => { host.querySelector<HTMLButtonElement>('button[data-command-id="new-session"]')!.click(); });
        expect(host.querySelector('[role="alert"]')).toBeNull();
    });

    it('clears a palette new-session error after a successful retry', () => {
        const store = new LifecycleStore();
        const sessions = makeSessions();
        const items: Array<{ id: string; title: string; createdAt: number; updatedAt: number; messageCount: number; preview: string }> = [];
        let activeId: string | null = null;
        sessions.listSessions = vi.fn(() => items);
        sessions.activeSessionId = vi.fn(() => activeId);
        sessions.newSession = vi.fn().mockReturnValueOnce(false).mockImplementationOnce(() => {
            items.unshift({ id: 'session-new', title: 'New conversation', createdAt: 1, updatedAt: 1, messageCount: 0, preview: '' });
            activeId = 'session-new';
            return true;
        });
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={sessions} ui={makeUi()} download={vi.fn()} />, host);
        });
        act(() => { button('Command palette').click(); });
        act(() => { host.querySelector<HTMLButtonElement>('button[data-command-id="new-session"]')!.click(); });
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('Could not create a session.');
        act(() => { button('Command palette').click(); });
        act(() => { host.querySelector<HTMLButtonElement>('button[data-command-id="new-session"]')!.click(); });
        expect(host.querySelector('[role="alert"]')).toBeNull();
    });

    it('does not clear file A save failure after file B saves', async () => {
        const store = new LifecycleStore();
        const files = makeFiles();
        let activePath = 'a.md';
        files.activeFile = vi.fn(() => ({ path: activePath, name: activePath, content: 'edited', mode: 'local' }));
        files.treeSummary = vi.fn(() => ({ mode: 'local', rootName: 'project', fileCount: 1, tabs: [{ path: activePath, dirty: true }], activePath }));
        files.saveActive = vi.fn(async () => activePath !== 'a.md');
        store.dispatch({ type: 'file/opened', path: 'a.md' });
        store.dispatch({ type: 'file/dirty' });
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={files} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        act(() => { destination('Workspace').click(); });
        await act(async () => { button('Save').click(); await Promise.resolve(); });
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('Save did not complete.');
        activePath = 'b.md';
        act(() => {
            store.dispatch({ type: 'file/opened', path: 'b.md' });
            store.dispatch({ type: 'file/dirty' });
        });
        await act(async () => { button('Save').click(); await Promise.resolve(); });
        expect(store.getState().file.phase).toBe('saved');
        await act(async () => { await Promise.resolve(); await Promise.resolve(); });
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('Save did not complete.');
    });

    it('keeps a provider selection error after an unrelated successful test', async () => {
        const store = new LifecycleStore();
        const sources = makeSources();
        const adapter = makeModelAdapter();
        const provider = { id: 'api-a', label: 'API A', preset: 'custom', baseUrl: 'https://api.example.com/v1', model: 'model-a', apiKey: '' };
        sources.list = vi.fn(() => ({ active: { kind: 'wasm', id: 'bundled-demo' }, wasm: {}, api: [provider] }));
        sources.testApi = vi.fn(async () => ({ ok: true }));
        adapter.selectApi = vi.fn(() => { throw new Error('selection failed'); });
        act(() => {
            render(<StudioApp store={store} adapter={adapter} sources={sources} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        act(() => { destination('Models').click(); });
        act(() => { button('Use API A').click(); });
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('selection failed');
        await act(async () => { button('Test API A').click(); await Promise.resolve(); });
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('selection failed');
    });

    it('clears a provider test error after that test succeeds', async () => {
        const store = new LifecycleStore();
        const sources = makeSources();
        const provider = { id: 'api-a', label: 'API A', preset: 'custom', baseUrl: 'https://api.example.com/v1', model: 'model-a', apiKey: '' };
        sources.list = vi.fn(() => ({ active: { kind: 'wasm', id: 'bundled-demo' }, wasm: {}, api: [provider] }));
        sources.testApi = vi.fn().mockRejectedValueOnce(new Error('test failed')).mockResolvedValueOnce({ ok: true });
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={sources} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        act(() => { destination('Models').click(); });
        await act(async () => { button('Test API A').click(); await Promise.resolve(); });
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('test failed');
        await act(async () => { button('Test API A').click(); await Promise.resolve(); });
        expect(host.querySelector('[role="alert"]')).toBeNull();
    });

    it('clears a workspace open-folder error only after that operation succeeds', async () => {
        const store = new LifecycleStore();
        const files = makeFiles();
        files.openFolder = vi.fn().mockRejectedValueOnce(new Error('open failed')).mockResolvedValueOnce(undefined);
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={files} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        act(() => { destination('Workspace').click(); });
        await act(async () => { button('Open folder').click(); await Promise.resolve(); });
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('open failed');
        await act(async () => { button('Open folder').click(); await Promise.resolve(); });
        expect(host.querySelector('[role="alert"]')).toBeNull();
    });

    it('replaces a provider-test rejection with the settled failure detail', async () => {
        const store = new LifecycleStore();
        const sources = makeSources();
        const provider = { id: 'api-a', label: 'API A', preset: 'custom', baseUrl: 'https://api.example.com/v1', model: 'model-a', apiKey: '' };
        sources.list = vi.fn(() => ({ active: { kind: 'wasm', id: 'bundled-demo' }, wasm: {}, api: [provider] }));
        sources.testApi = vi.fn().mockRejectedValueOnce(new Error('network down')).mockResolvedValueOnce({ ok: false, detail: 'unauthorized' });
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={sources} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        act(() => { destination('Models').click(); });
        await act(async () => { button('Test API A').click(); await Promise.resolve(); });
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('network down');
        await act(async () => { button('Test API A').click(); await Promise.resolve(); });
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('unauthorized');
        expect(host.querySelector('[role="alert"]')?.textContent).not.toContain('network down');
    });

    it('derives the active session marker from verified lifecycle state', () => {
        const store = new LifecycleStore();
        const sessions = makeSessions();
        const items = [
            { id: 'session-a', title: 'Session A', createdAt: 1, updatedAt: 1, messageCount: 0, preview: '' },
            { id: 'session-b', title: 'Session B', createdAt: 2, updatedAt: 2, messageCount: 0, preview: '' },
        ];
        let activeId = 'session-a';
        sessions.listSessions = vi.fn(() => items);
        sessions.activeSessionId = vi.fn(() => activeId);
        sessions.selectSession = vi.fn((id: string) => { activeId = id; return true; });
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={sessions} ui={makeUi()} download={vi.fn()} />, host);
        });
        expect(host.querySelector('li[aria-current="true"] strong')?.textContent).toBe('Session A');
        act(() => { button('Select Session B').click(); });
        expect(store.getState().session.activeId).toBe('session-b');
        expect(host.querySelector('li[aria-current="true"] strong')?.textContent).toBe('Session B');
    });

    it('keeps the partial session-clear warning after refreshing sessions', () => {
        const store = new LifecycleStore();
        const sessions = makeSessions();
        sessions.listSessions = vi.fn(() => [{ id: 'session-a', title: 'Session A', createdAt: 1, updatedAt: 2, messageCount: 0, preview: '' }]);
        sessions.activeSessionId = vi.fn(() => 'session-a');
        const ui = makeUi();
        ui.clearSessions = vi.fn(() => ({ cleared: 1, total: 2 }));
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={sessions} ui={ui} download={vi.fn()} />, host);
        });
        act(() => { button('Settings').click(); });
        act(() => { button('Clear sessions').click(); });
        act(() => { button('Confirm clear sessions').click(); });
        expect(host.querySelector('main > p[role="alert"]')?.textContent).toContain('Cleared 1 of 2 sessions');
    });

    it('clears a failed provider test after a successful edit', async () => {
        const store = new LifecycleStore();
        const sources = makeSources();
        const provider = { id: 'api-a', label: 'API A', preset: 'custom', baseUrl: 'https://api.example.com/v1', model: 'model-a', apiKey: '' };
        sources.list = vi.fn(() => ({ active: { kind: 'wasm', id: 'bundled-demo' }, wasm: {}, api: [provider] }));
        sources.testApi = vi.fn().mockRejectedValueOnce(new Error('test failed'));
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={sources} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        act(() => { destination('Models').click(); });
        await act(async () => { button('Test API A').click(); await Promise.resolve(); });
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('test failed');
        act(() => { button('Edit API A').click(); });
        act(() => { host.querySelector('form[data-action="save-provider"]')!.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true })); });
        expect(host.querySelector('[role="alert"]')).toBeNull();
    });

    it('ignores a late artifact-load rejection after a newer load succeeds', async () => {
        const store = new LifecycleStore();
        const adapter = makeModelAdapter();
        let rejectFirst!: (error: Error) => void;
        let finishSecond!: (result: { bytes: Uint8Array; fileName: string; meta: { schema: string } }) => void;
        adapter.loadBundled = vi.fn()
            .mockImplementationOnce(() => new Promise((_resolve, reject) => { rejectFirst = reject; }))
            .mockImplementationOnce(() => new Promise((resolve) => { finishSecond = resolve; }));
        act(() => {
            render(<StudioApp store={store} adapter={adapter} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        act(() => { destination('Models').click(); });
        act(() => { button('Load bundled demo').click(); });
        act(() => { button('Load bundled demo').click(); });
        await act(async () => { finishSecond({ bytes: new Uint8Array(), fileName: 'model.lut', meta: { schema: 'uor-r4.lut-stack/1' } }); await Promise.resolve(); });
        await act(async () => { rejectFirst(new Error('late failure')); await Promise.resolve(); });
        expect(host.querySelector('[role="alert"]')).toBeNull();
    });

    it('ignores an older palette load rejection after a newer load succeeds', async () => {
        const store = new LifecycleStore();
        const adapter = makeModelAdapter();
        let rejectFirst!: (error: Error) => void;
        let finishSecond!: (result: { bytes: Uint8Array; fileName: string; meta: { schema: string } }) => void;
        adapter.loadBundled = vi.fn()
            .mockImplementationOnce(() => new Promise((_resolve, reject) => { rejectFirst = reject; }))
            .mockImplementationOnce(() => new Promise((resolve) => { finishSecond = resolve; }));
        act(() => {
            render(<StudioApp store={store} adapter={adapter} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        act(() => { button('Command palette').click(); });
        act(() => { host.querySelector<HTMLButtonElement>('button[data-command-id="load-demo"]')!.click(); });
        act(() => { button('Command palette').click(); });
        act(() => { host.querySelector<HTMLButtonElement>('button[data-command-id="load-demo"]')!.click(); });
        await act(async () => { finishSecond({ bytes: new Uint8Array(), fileName: 'model.lut', meta: { schema: 'uor-r4.lut-stack/1' } }); await Promise.resolve(); });
        await act(async () => { rejectFirst(new Error('late palette failure')); await Promise.resolve(); });
        expect(host.querySelector('[role="alert"]')).toBeNull();
    });

    it('clears a failed provider selection after removing that provider', () => {
        const store = new LifecycleStore();
        const sources = makeSources();
        const provider = { id: 'api-a', label: 'API A', preset: 'custom', baseUrl: 'ftp://example.com/v1', model: 'model-a', apiKey: '' };
        sources.list = vi.fn(() => ({ active: { kind: 'wasm', id: 'bundled-demo' }, wasm: {}, api: [provider] }));
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={sources} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        act(() => { destination('Models').click(); });
        act(() => { button('Use API A').click(); });
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('Only HTTP(S) API endpoints are supported.');
        act(() => { button('Remove API A').click(); });
        act(() => { button('Confirm removal').click(); });
        expect(host.querySelector('[role="alert"]')).toBeNull();
    });

    it('uses an available native host through composed Changes and Terminal views', async () => {
        const snapshot = { sessionId: 'pty-fixture', cursor: 0, active: true, truncated: false, cwd: '/granted/project', shell: '/bin/sh', pid: 123, events: [] };
        const invoke = vi.fn(async (command: string, _args: Record<string, unknown> = {}) => {
            if (command === 'native_git_status') return { branch: 'main', modified_files: ['a.rs'], untracked_files: [], timed_out: false };
            if (command === 'native_git_diff') return { diff: 'composed diff', truncated: false, timed_out: false };
            if (command === 'native_enable_terminal') return { enabled: true };
            if (command === 'native_open_terminal' || command === 'native_read_terminal') return snapshot;
            if (command === 'native_close_terminal') return { ...snapshot, active: false };
            if (command === 'native_terminal_input') return { ok: true };
            throw new Error(`unexpected ${command}`);
        });
        const store = new LifecycleStore();
        const files = makeFiles();
        const summary = files.treeSummary();
        files.treeSummary = vi.fn(() => ({ ...summary, rootPath: '/granted/project' }));
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={files} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} desktop={createDesktopActions({ invoke })} />, host);
        });
        act(() => { destination('Changes').click(); });
        expect(button('Refresh changes').disabled).toBe(false);
        await act(async () => { button('Refresh changes').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('gitDiffOutput')?.textContent).toContain('composed diff'));
        act(() => { destination('Terminal').click(); });
        await act(async () => { button('Open terminal').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('terminalConsentState')?.textContent).toContain('Interactive PTY'));
        expect(invoke).toHaveBeenCalledWith('native_open_terminal', { cwd: '/granted/project', rows: 24, cols: 80 });
        expect(invoke).not.toHaveBeenCalledWith('native_run_terminal_command', expect.anything());
        expect(Array.from(host.querySelectorAll('button')).find((node) => node.textContent === 'Run command')).toBeUndefined();
    });

    it('attaches an MCP result and exposes it through existing @ mentions', async () => {
        const result = { content: [{ type: 'text', text: 'context text' }] };
        const session = {
            listTools: vi.fn(async () => [{ name: 'read_file', description: 'Read a file' }]),
            callTool: vi.fn(async () => result),
            disconnect: vi.fn(async () => undefined),
        };
        const mcpConnect = vi.fn(async () => session);
        const attachments: Attachment[] = [];
        const ui = makeUi();
        ui.getAttachments = vi.fn(() => attachments);
        ui.addAttachment = vi.fn((entry: Attachment) => {
            attachments.push(entry);
            document.dispatchEvent(new CustomEvent('uor-attachments-changed'));
        });
        const store = new LifecycleStore();
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={ui} download={vi.fn()} mcpConnect={mcpConnect} />, host);
        });
        act(() => { destination('Models').click(); });
        const url = host.querySelector<HTMLInputElement>('input[aria-label="MCP server URL"]')!;
        act(() => { url.value = 'http://127.0.0.1:9000/mcp'; url.dispatchEvent(new Event('input', { bubbles: true })); });
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.querySelector('.mcp-tool-button')).not.toBeNull());
        await act(async () => { (document.querySelector('.mcp-tool-button') as HTMLButtonElement).click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('mcpResultOutput')?.textContent).toContain('context text'));
        await act(async () => { button('Attach result').click(); await Promise.resolve(); });
        expect(attachments).toEqual([{ name: 'mcp-read_file.json', kind: 'text', content: JSON.stringify(result, null, 2), source: 'mcp' }]);
        act(() => { destination('Agent').click(); });
        const prompt = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]')!;
        act(() => {
            prompt.value = '@mcp';
            prompt.setSelectionRange(4, 4);
            prompt.dispatchEvent(new Event('input', { bubbles: true }));
        });
        const mentions = host.querySelector('[role="listbox"][aria-label="Attachment mentions"]')!;
        expect(mentions.textContent).toContain('mcp-read_file.json');
        expect(session.callTool).toHaveBeenCalledOnce();
    });
});


describe('navigation command grouping', () => {
    it('keeps navigation commands searchable by their existing group', () => {
        const store = new LifecycleStore();
        act(() => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        act(() => { button('Command palette').click(); });
        const input = host.querySelector<HTMLInputElement>('input[aria-label="Search commands"]')!;
        act(() => { input.value = 'Navigation'; input.dispatchEvent(new Event('input', { bubbles: true })); });
        for (const id of ['agent', 'workspace', 'changes', 'terminal', 'models', 'guide']) {
            expect(host.querySelector(`button[data-command-id="${id}"]`), `missing Navigation command ${id}`).not.toBeNull();
        }
    });
});

const RUNTIME_PROJECT = 'prj_0000000000000001';
const RUNTIME_TASK = 'tsk_0000000000000001';
const RUNTIME_RUN = 'run_0000000000000001';

function runtimeEvent(sequence: number, type: string, data: Record<string, unknown>): AgentIncomingEvent {
    return {
        schema: 'uor-r4.studio.agent-event/1',
        event_id: `evt_${RUNTIME_RUN}_1_${sequence.toString().padStart(12, '0')}`,
        project_id: RUNTIME_PROJECT,
        task_id: RUNTIME_TASK,
        run_id: RUNTIME_RUN,
        generation: 1,
        sequence,
        recorded_at_ms: 1_794_283_200_000 + sequence,
        type,
        data,
    } as unknown as AgentIncomingEvent;
}

function runtimeClientFixture(): { client: AgentClient; transport: FixtureTransport } {
    const store = createAgentStore();
    expect(store.apply(runtimeEvent(0, 'run.queued', { queued_at_ms: 1_794_283_200_000 })).status).toBe('applied');
    expect(store.apply(runtimeEvent(1, 'run.started', {
        started_at_ms: 1_794_283_200_001,
        model: { id: 'synthetic-replay', kind: 'synthetic-replay' },
        mode: 'agent',
        environment: { kind: 'browser-runtime' },
        retry_of_run_id: null,
    })).status).toBe('applied');
    store.upsertTask({
        taskId: RUNTIME_TASK,
        projectId: RUNTIME_PROJECT,
        title: 'Runtime detail task',
        repo: 'uor-r4',
        branch: 'main',
        createdAtMs: 1_794_283_200_000,
        latestRunId: RUNTIME_RUN,
    });
    const transport = new FixtureTransport();
    return { client: new AgentClient({ store, transport }), transport };
}

function runtimeClient(): AgentClient {
    return runtimeClientFixture().client;
}

class PendingListTransport extends FixtureTransport {
    private resolveList!: (tasks: TaskSummary[]) => void;
    private readonly pending = new Promise<TaskSummary[]>((resolve) => { this.resolveList = resolve; });
    override listTasks(): Promise<TaskSummary[]> {
        return this.pending;
    }
    resolve(tasks: TaskSummary[] = []): void {
        this.resolveList(tasks);
    }
}

class RejectingListTransport extends FixtureTransport {
    override async listTasks(): Promise<TaskSummary[]> {
        throw new Error('task list failed');
    }
}

function runtimeClientWithStatus(status: 'finished' | 'cancelling'): AgentClient {
    const store = createAgentStore();
    store.apply(runtimeEvent(0, 'run.queued', { queued_at_ms: 1_794_283_200_000 }));
    store.apply(runtimeEvent(1, 'run.started', {
        started_at_ms: 1_794_283_200_001,
        model: { id: 'synthetic-replay', kind: 'synthetic-replay' },
        mode: 'agent',
        environment: { kind: 'browser-runtime' },
        retry_of_run_id: null,
    }));
    store.apply(runtimeEvent(2, 'run.status', { status }));
    store.upsertTask({
        taskId: RUNTIME_TASK,
        projectId: RUNTIME_PROJECT,
        title: `Status ${status} task`,
        createdAtMs: 1_794_283_200_000,
        latestRunId: RUNTIME_RUN,
    });
    return new AgentClient({ store, transport: new FixtureTransport() });
}

describe('StudioApp runtime task detail', () => {
    it('opens the selected runtime task in the transcript-first detail and returns to the shell', async () => {
        const client = runtimeClient();
        await act(async () => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} agentClient={client} />, host);
            await Promise.resolve();
        });
        const row = host.querySelector<HTMLButtonElement>(`[data-task-id="${RUNTIME_TASK}"] .task-select`)!;
        expect(row).not.toBeNull();
        await act(async () => { row.click(); await Promise.resolve(); });
        expect(host.querySelector('[data-task-detail-header]')).not.toBeNull();
        expect(host.querySelector('.agent-transcript')).not.toBeNull();
        expect(host.querySelector('.agent-composer')).not.toBeNull();
        expect(host.querySelectorAll('h1')).toHaveLength(1);
        expect(host.textContent).toContain('Runtime detail task');
        expect(host.textContent).toContain('Synthetic replay');
        expect(host.textContent).not.toContain('Restore Checkpoint');

        await act(async () => { button('Back to tasks').click(); await Promise.resolve(); });
        expect(host.querySelector('[data-task-detail-header]')).toBeNull();
        expect(host.textContent).toContain('No messages yet');
        expect(host.querySelector('h1')?.textContent).toBe('Agent');
    });

    it('re-renders the detail from the runtime subscription without a reload', async () => {
        const client = runtimeClient();
        await act(async () => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} agentClient={client} />, host);
            await Promise.resolve();
        });
        await act(async () => { host.querySelector<HTMLButtonElement>(`[data-task-id="${RUNTIME_TASK}"] .task-select`)!.click(); await Promise.resolve(); });
        expect(host.textContent).not.toContain('Streamed reply');
        const delta = runtimeEvent(2, 'assistant.delta', { message_id: 'msg_1f2e3d4c5b6a7988', text_delta: 'Streamed reply' });
        await act(async () => { await client.ingestWireEvent(canonicalJson(delta)); await Promise.resolve(); });
        await vi.waitFor(() => expect(host.textContent).toContain('Streamed reply'));
    });

    it('keeps the legacy shell and local sessions usable when the task runtime is unavailable', async () => {
        const client = new AgentClient({ store: createAgentStore() });
        await act(async () => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} agentClient={client} />, host);
            await Promise.resolve();
        });
        await vi.waitFor(() => expect(host.textContent).toContain('The task runtime is not initialized. Reload the page.'));
        expect(host.textContent).toContain('Local sessions');
        expect(host.querySelector('.agent-composer')).not.toBeNull();
        expect(host.querySelector('textarea[aria-label="Prompt"]')).not.toBeNull();
    });

    it('clears the real selection on Back, restores row focus and cannot start a hidden run', async () => {
        const { client, transport } = runtimeClientFixture();
        await act(async () => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} agentClient={client} />, host);
            await Promise.resolve();
        });
        await act(async () => { host.querySelector<HTMLButtonElement>(`[data-task-id="${RUNTIME_TASK}"] .task-select`)!.click(); await Promise.resolve(); });
        expect(host.querySelector('[data-task-detail-header]')).not.toBeNull();
        await act(async () => { button('Back to tasks').click(); await Promise.resolve(); });
        expect(client.store.snapshot().selectedTaskId).toBeNull();
        expect(client.store.snapshot().selectedTask).toBeNull();
        await vi.waitFor(() => expect((document.activeElement as HTMLElement | null)?.className ?? '').toContain('task-select'));
        const startsBefore = transport.calls.filter((call) => call.method === 'startRun').length;
        const prompt = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]')!;
        act(() => { prompt.value = 'back then send'; prompt.dispatchEvent(new Event('input', { bubbles: true })); });
        expect(button('Send').disabled, 'runtime tasks without a selection disable Send').toBe(true);
        await act(async () => { button('Send').click(); await Promise.resolve(); });
        expect(transport.calls.filter((call) => call.method === 'startRun')).toHaveLength(startsBefore);
        expect(prompt.value).toBe('back then send');
    });

    it('disables Send while the runtime task list is loading and uses legacy readiness when it is empty', async () => {
        const store = new LifecycleStore();
        vi.spyOn(store, 'canRun').mockReturnValue(true);
        const transport = new PendingListTransport();
        const client = new AgentClient({ transport });
        await act(async () => {
            render(<StudioApp store={store} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} agentClient={client} />, host);
            await Promise.resolve();
        });
        const prompt = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]')!;
        act(() => { prompt.value = 'ready check'; prompt.dispatchEvent(new Event('input', { bubbles: true })); });
        expect(button('Send').disabled, 'Send must wait for the task list').toBe(true);

        await act(async () => { transport.resolve([]); await new Promise((resolve) => setTimeout(resolve, 0)); });
        await vi.waitFor(() => expect(host.textContent).toContain('No tasks yet'));
        expect(button('Send').disabled, 'an empty runtime list falls back to lifecycle readiness').toBe(false);
    });

    it('keeps Send disabled for an unresolved restored run without starting another path', async () => {
        const store = createAgentStore();
        store.upsertTask({
            taskId: RUNTIME_TASK,
            projectId: RUNTIME_PROJECT,
            title: 'Unresolved restore task',
            createdAtMs: 1_794_283_200_000,
            latestRunId: RUNTIME_RUN,
        });
        const transport = new FixtureTransport();
        const client = new AgentClient({ store, transport });
        await act(async () => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} agentClient={client} />, host);
            await Promise.resolve();
        });
        await act(async () => { host.querySelector<HTMLButtonElement>(`[data-task-id="${RUNTIME_TASK}"] .task-select`)!.click(); await Promise.resolve(); });
        const prompt = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]')!;
        act(() => { prompt.value = 'unresolved send'; prompt.dispatchEvent(new Event('input', { bubbles: true })); });
        expect(button('Send').disabled).toBe(true);
        const startsBefore = transport.calls.filter((call) => call.method === 'startRun').length;
        await act(async () => { button('Send').click(); await Promise.resolve(); });
        expect(transport.calls.filter((call) => call.method === 'startRun')).toHaveLength(startsBefore);
    });

    it('hides the legacy lifecycle Stop while the runtime detail is active', async () => {
        const client = runtimeClient();
        await act(async () => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} agentClient={client} />, host);
            await Promise.resolve();
        });
        await act(async () => { host.querySelector<HTMLButtonElement>(`[data-task-id="${RUNTIME_TASK}"] .task-select`)!.click(); await Promise.resolve(); });
        const composerStops = Array.from(host.querySelectorAll<HTMLButtonElement>('.agent-composer .composer-footer button'))
            .filter((node) => (node.textContent ?? '').trim() === 'Stop');
        expect(composerStops).toHaveLength(0);
        expect(host.querySelector('.run-status .run-stop')).not.toBeNull();
    });

    it('keeps the labelled Local sessions group available when runtime tasks exist', async () => {
        const client = runtimeClient();
        await act(async () => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} agentClient={client} />, host);
            await Promise.resolve();
        });
        await vi.waitFor(() => expect(host.querySelector(`[data-task-id="${RUNTIME_TASK}"]`)).not.toBeNull());
        const heading = host.querySelector<HTMLElement>('nav[aria-label="Tasks and local sessions"] h3#task-rail-local-sessions');
        expect(heading).not.toBeNull();
        const group = heading!.closest('details[data-task-group="local-sessions"]') as HTMLDetailsElement | null;
        expect(group).not.toBeNull();
        expect(group!.open).toBe(false);
    });

    it('shows cancelling in the rail and ignores a false terminal projection', async () => {
        const cancelling = runtimeClientWithStatus('cancelling');
        await act(async () => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} agentClient={cancelling} />, host);
            await Promise.resolve();
        });
        await vi.waitFor(() => expect(host.querySelector(`[data-task-id="${RUNTIME_TASK}"] .task-state`)?.textContent).toBe('Cancelling'));
        expect(cancelling.store.snapshot().tasks[0]?.state).toBe('cancelling');

        const falseFinished = runtimeClientWithStatus('finished');
        await act(async () => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} agentClient={falseFinished} />, host);
            await Promise.resolve();
        });
        await vi.waitFor(() => expect(host.querySelector(`[data-task-id="${RUNTIME_TASK}"] .task-state`)?.textContent).toBe('Running'));
        expect(falseFinished.store.snapshot().tasks[0]?.state).toBe('running');
    });

    it('disables Send instead of falling back to legacy generation when the task list errors with runtime tasks', async () => {
        const store = createAgentStore();
        store.apply(runtimeEvent(0, 'run.queued', { queued_at_ms: 1_794_283_200_000 }));
        store.apply(runtimeEvent(1, 'run.started', {
            started_at_ms: 1_794_283_200_001,
            model: { id: 'synthetic-replay', kind: 'synthetic-replay' },
            mode: 'agent',
            environment: { kind: 'browser-runtime' },
            retry_of_run_id: null,
        }));
        store.upsertTask({
            taskId: RUNTIME_TASK,
            projectId: RUNTIME_PROJECT,
            title: 'List error task',
            createdAtMs: 1_794_283_200_000,
            latestRunId: RUNTIME_RUN,
        });
        const transport = new RejectingListTransport();
        const client = new AgentClient({ store, transport });
        const lifecycle = new LifecycleStore();
        lifecycle.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        const sessionsApi = makeSessions();
        await act(async () => {
            render(<StudioApp store={lifecycle} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={sessionsApi} ui={makeUi()} download={vi.fn()} agentClient={client} />, host);
            await new Promise((resolve) => setTimeout(resolve, 0));
        });
        await vi.waitFor(() => expect(host.querySelector(`[data-task-id="${RUNTIME_TASK}"]`)).not.toBeNull());
        await act(async () => { host.querySelector<HTMLButtonElement>(`[data-task-id="${RUNTIME_TASK}"] .task-select`)!.click(); await Promise.resolve(); });
        const prompt = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]')!;
        act(() => { prompt.value = 'must not leak to legacy'; prompt.dispatchEvent(new Event('input', { bubbles: true })); });
        expect(button('Send').disabled, 'a task-list error must block Send while runtime tasks exist').toBe(true);
        const startsBefore = transport.calls.filter((call) => call.method === 'startRun').length;
        await act(async () => { button('Send').click(); await Promise.resolve(); });
        expect(transport.calls.filter((call) => call.method === 'startRun')).toHaveLength(startsBefore);
        expect(sessionsApi.sendPrompt).not.toHaveBeenCalled();
    });

    it('opens a collapsed containing task group before restoring Back focus', async () => {
        const store = createAgentStore();
        store.upsertTask({
            taskId: RUNTIME_TASK,
            projectId: RUNTIME_PROJECT,
            title: 'Older task',
            createdAtMs: Date.now() - 30 * 24 * 60 * 60 * 1000,
            latestRunId: RUNTIME_RUN,
        });
        store.apply(runtimeEvent(0, 'run.queued', { queued_at_ms: 1_794_283_200_000 }));
        store.apply(runtimeEvent(1, 'run.started', {
            started_at_ms: 1_794_283_200_001,
            model: { id: 'synthetic-replay', kind: 'synthetic-replay' },
            mode: 'agent',
            environment: { kind: 'browser-runtime' },
            retry_of_run_id: null,
        }));
        const client = new AgentClient({ store, transport: new FixtureTransport() });
        await act(async () => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} agentClient={client} />, host);
            await Promise.resolve();
        });
        const group = host.querySelector<HTMLDetailsElement>('details[data-task-group="older"]')!;
        expect(group).not.toBeNull();
        expect(group.open).toBe(false);
        act(() => { group.open = true; });
        await act(async () => { host.querySelector<HTMLButtonElement>(`[data-task-id="${RUNTIME_TASK}"] .task-select`)!.click(); await Promise.resolve(); });
        act(() => { group.open = false; });
        await act(async () => { button('Back to tasks').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(group.open).toBe(true));
        await vi.waitFor(() => expect((document.activeElement as HTMLElement | null)?.className ?? '').toContain('task-select'));
    });

    it('clears the runtime detail when a Local session is selected', async () => {
        const { client, transport } = runtimeClientFixture();
        const sessionsApi = makeSessions();
        sessionsApi.newSession();
        await act(async () => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={sessionsApi} ui={makeUi()} download={vi.fn()} agentClient={client} />, host);
            await Promise.resolve();
        });
        await act(async () => { host.querySelector<HTMLButtonElement>(`[data-task-id="${RUNTIME_TASK}"] .task-select`)!.click(); await Promise.resolve(); });
        expect(host.querySelector('[data-task-detail-header]')).not.toBeNull();
        const localGroup = host.querySelector<HTMLDetailsElement>('details[data-task-group="local-sessions"]')!;
        localGroup.open = true;
        const startsBefore = transport.calls.filter((call) => call.method === 'startRun').length;
        await act(async () => { host.querySelector<HTMLButtonElement>('[data-task-group="local-sessions"] [data-action="resume-session"]')!.click(); await Promise.resolve(); });
        expect(client.store.snapshot().selectedTaskId).toBeNull();
        expect(client.store.snapshot().selectedTask).toBeNull();
        expect(host.querySelector('[data-task-detail-header]')).toBeNull();
        expect(host.querySelector('h1')?.textContent).toBe('Agent');
        expect(host.textContent).toContain('No messages yet');
        expect(sessionsApi.selectSession).toHaveBeenCalledWith('session-new-1');
        expect(transport.calls.filter((call) => call.method === 'startRun')).toHaveLength(startsBefore);
    });

    it('clears the runtime detail for a Local new session, rename and delete', async () => {
        const { client } = runtimeClientFixture();
        const sessionsApi = makeSessions();
        sessionsApi.newSession();
        await act(async () => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={sessionsApi} ui={makeUi()} download={vi.fn()} agentClient={client} />, host);
            await Promise.resolve();
        });
        const openRuntime = async () => {
            await act(async () => { host.querySelector<HTMLButtonElement>(`[data-task-id="${RUNTIME_TASK}"] .task-select`)!.click(); await Promise.resolve(); });
            expect(host.querySelector('[data-task-detail-header]')).not.toBeNull();
            const group = host.querySelector<HTMLDetailsElement>('details[data-task-group="local-sessions"]')!;
            group.open = true;
        };

        await openRuntime();
        await act(async () => { host.querySelector<HTMLButtonElement>('[data-task-group="local-sessions"] [data-action="new-session"]')!.click(); await Promise.resolve(); });
        expect(client.store.snapshot().selectedTaskId).toBeNull();
        expect(host.querySelector('[data-task-detail-header]')).toBeNull();
        expect(sessionsApi.newSession).toHaveBeenCalled();

        await openRuntime();
        await act(async () => {
            host.querySelector<HTMLButtonElement>('[data-task-group="local-sessions"] summary[aria-label^="Actions for"]')!.click();
            await Promise.resolve();
        });
        await act(async () => {
            const rename = Array.from(host.querySelectorAll<HTMLButtonElement>('[data-task-group="local-sessions"] button'))
                .find((node) => node.textContent?.startsWith('Rename '))!;
            rename.click();
            await Promise.resolve();
        });
        const title = host.querySelector<HTMLInputElement>('dialog input[aria-label="Session title"]')!;
        act(() => { title.value = 'Renamed local'; title.dispatchEvent(new Event('input', { bubbles: true })); });
        await act(async () => {
            host.querySelector<HTMLButtonElement>('dialog button[type="submit"]')!.click();
            await Promise.resolve();
        });
        expect(client.store.snapshot().selectedTaskId).toBeNull();
        expect(host.querySelector('[data-task-detail-header]')).toBeNull();
        expect(sessionsApi.renameSession).toHaveBeenCalled();

        await openRuntime();
        await act(async () => {
            host.querySelector<HTMLButtonElement>('[data-task-group="local-sessions"] summary[aria-label^="Actions for"]')!.click();
            await Promise.resolve();
        });
        await act(async () => {
            const remove = Array.from(host.querySelectorAll<HTMLButtonElement>('[data-task-group="local-sessions"] button'))
                .find((node) => node.textContent?.startsWith('Delete '))!;
            remove.click();
            await Promise.resolve();
        });
        await act(async () => { button('Confirm delete').click(); await Promise.resolve(); });
        expect(client.store.snapshot().selectedTaskId).toBeNull();
        expect(host.querySelector('[data-task-detail-header]')).toBeNull();
        expect(sessionsApi.deleteSession).toHaveBeenCalled();
    });

    it('disables Send when runtime tasks exist but no runtime task is selected', async () => {
        const { client, transport } = runtimeClientFixture();
        await act(async () => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} agentClient={client} />, host);
            await Promise.resolve();
        });
        await vi.waitFor(() => expect(host.querySelector(`[data-task-id="${RUNTIME_TASK}"]`)).not.toBeNull());
        await vi.waitFor(() => expect(host.querySelector('.task-loading')).toBeNull());
        const prompt = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Prompt"]')!;
        act(() => { prompt.value = 'no selected runtime task'; prompt.dispatchEvent(new Event('input', { bubbles: true })); });
        expect(button('Send').disabled).toBe(true);
        const startsBefore = transport.calls.filter((call) => call.method === 'startRun').length;
        await act(async () => { button('Send').click(); await Promise.resolve(); });
        expect(transport.calls.filter((call) => call.method === 'startRun')).toHaveLength(startsBefore);
        expect(prompt.value).toBe('no selected runtime task');
    });

    it('focuses the group summary when the selected row is hidden', async () => {
        const store = createAgentStore();
        store.upsertTask({
            taskId: RUNTIME_TASK,
            projectId: RUNTIME_PROJECT,
            title: 'Hidden older task',
            createdAtMs: Date.now() - 30 * 24 * 60 * 60 * 1000,
            latestRunId: RUNTIME_RUN,
        });
        store.apply(runtimeEvent(0, 'run.queued', { queued_at_ms: 1_794_283_200_000 }));
        store.apply(runtimeEvent(1, 'run.started', {
            started_at_ms: 1_794_283_200_001,
            model: { id: 'synthetic-replay', kind: 'synthetic-replay' },
            mode: 'agent',
            environment: { kind: 'browser-runtime' },
            retry_of_run_id: null,
        }));
        const client = new AgentClient({ store, transport: new FixtureTransport() });
        await act(async () => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} agentClient={client} />, host);
            await Promise.resolve();
        });
        const initialGroup = host.querySelector<HTMLDetailsElement>('details[data-task-group="older"]')!;
        initialGroup.open = true;
        await act(async () => { host.querySelector<HTMLButtonElement>(`[data-task-id="${RUNTIME_TASK}"] .task-select`)!.click(); await Promise.resolve(); });
        const group = host.querySelector<HTMLDetailsElement>('details[data-task-group="older"]')!;
        const row = host.querySelector<HTMLButtonElement>(`[data-task-id="${RUNTIME_TASK}"] .task-select`)!;
        group.open = false;
        row.disabled = true;
        await act(async () => { button('Back to tasks').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(group.open).toBe(true));
        await vi.waitFor(() => expect(document.activeElement).toBe(group.querySelector('summary')));
    });
});

describe('StudioApp runtime task creation', () => {
    it('creates_and_selects_a_first_task_through_the_existing_agent_owner', async () => {
        const transport = new FixtureTransport();
        const client = new AgentClient({ transport });
        const store = new LifecycleStore();
        act(() => {
            render(
                <StudioApp
                    store={store}
                    adapter={makeModelAdapter()}
                    sources={makeSources()}
                    files={makeFiles()}
                    sessions={makeSessions()}
                    ui={makeUi()}
                    download={vi.fn()}
                    agentClient={client}
                />,
                host,
            );
        });
        await vi.waitFor(() => expect(host.querySelector('[data-action="new-task"]')).not.toBeNull());
        const action = host.querySelector<HTMLButtonElement>('[data-action="new-task"]')!;
        await act(async () => {
            action.click();
            await Promise.resolve();
        });
        expect(transport.calls.filter((call) => call.method === 'createTask')).toHaveLength(1);
        expect(transport.calls.find((call) => call.method === 'createTask')?.payload).toMatchObject({
            title: 'New task',
        });
        await vi.waitFor(() =>
            expect(client.store.snapshot().selectedTask?.title).toBe('New task'),
        );
        expect(host.querySelector('#studio-typed-root [data-task-detail-header]')).toBeNull();
        expect(host.querySelector('textarea[aria-label="Prompt"]')).not.toBeNull();
        await vi.waitFor(() =>
            expect(host.querySelector<HTMLButtonElement>('.send-button')?.disabled).toBe(true),
        );
        expect(host.textContent).not.toContain('Select a task before sending');
    });
});
