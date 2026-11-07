import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LifecycleStore } from '../state/lifecycle.js';
import { createDesktopActions } from '../adapters/desktopActions.js';
import { StudioApp } from './StudioApp.js';
import { makeFiles, makeModelAdapter, makeSessions, makeSources, makeUi } from './studioTestFixtures.js';

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
        expect(host.textContent).toContain('Native Git is unavailable in this browser build; use the desktop build.');
        act(() => { destination('Terminal').click(); });
        expect(host.textContent).toContain('Guarded terminal actions are unavailable in this browser build.');
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
        sessions.newSession = vi.fn(() => {
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
        sessions.listSessions = vi.fn(() => [{ id: 'session-a', title: 'Session A', createdAt: 1, updatedAt: 2, messageCount: 1, preview: 'message' }]);
        sessions.duplicateSession = vi.fn().mockReturnValueOnce(null).mockReturnValueOnce('session-copy');
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
        sessions.newSession = vi.fn().mockReturnValueOnce(false).mockReturnValueOnce(true);
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
        sessions.newSession = vi.fn().mockReturnValueOnce(false).mockReturnValueOnce(true);
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

    it('keeps the partial session-clear warning after refreshing sessions', () => {
        const store = new LifecycleStore();
        const sessions = makeSessions();
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
        const invoke = vi.fn(async (command: string, _args: Record<string, unknown> = {}) => {
            if (command === 'native_git_status') return { branch: 'main', modified_files: ['a.rs'], untracked_files: [], timed_out: false };
            if (command === 'native_git_diff') return { diff: 'composed diff', truncated: false, timed_out: false };
            if (command === 'native_enable_terminal') return { enabled: true };
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
        await act(async () => { button('Enable terminal').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('terminalConsentState')?.textContent).toContain('enabled'));
        expect(invoke.mock.calls.map((call) => call[0])).toEqual(['native_git_status', 'native_git_diff', 'native_enable_terminal']);
    });
});
