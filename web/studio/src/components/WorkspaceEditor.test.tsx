import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LifecycleStore } from '../state/lifecycle.js';
import { createOperationGate } from '../adapters/operations.js';
import type { StudioFile, StudioSettings, WorkspaceApi, WorkspaceSummary } from '../adapters/workflow.js';
import type { AgentClient } from '../agent/client.js';
import { WorkspaceEditor } from './WorkspaceEditor.js';

let host: HTMLDivElement;
beforeEach(() => { host = document.createElement('div'); document.body.appendChild(host); });
afterEach(() => { render(null, host); host.remove(); vi.restoreAllMocks(); });

function files() {
    const summary: WorkspaceSummary = { mode: 'none', rootName: '', fileCount: 0, tabs: [], activePath: null };
    let active: StudioFile | null = null;
    const api: WorkspaceApi = {
        openFolder: vi.fn(async () => undefined), importFiles: vi.fn(), openDemo: vi.fn(),
        openFile: vi.fn(async () => undefined), newFile: vi.fn(), closeTab: vi.fn(), closeAll: vi.fn(),
        saveActive: vi.fn(async () => true), saveAsCopy: vi.fn(() => true), insertText: vi.fn(() => true),
        setActiveContent: vi.fn((content: string) => { if (active) active.content = content; }),
        activeFile: vi.fn(() => active), workspaceName: vi.fn(() => summary.rootName), mode: vi.fn(() => summary.mode),
        tabs: vi.fn(() => summary.tabs.map((tab) => ({ ...tab, mode: 'local' }))), treeSummary: vi.fn(() => summary),
    };
    return { api, summary, setActive(file: StudioFile | null) { active = file; }, get active() { return active; } };
}

function mount(settings?: StudioSettings) {
    const store = new LifecycleStore();
    const fileApi = files();
    const onError = vi.fn();
    const onSuccess = vi.fn();
    act(() => { render(<WorkspaceEditor store={store} files={fileApi.api} operations={createOperationGate()} onError={onError} onSuccess={onSuccess} settings={settings} />, host); });
    return { store, fileApi, onError, onSuccess };
}

function emit(fileApi: ReturnType<typeof files>) {
    document.dispatchEvent(new CustomEvent('uor-files-changed', { detail: fileApi.summary }));
}

function button(label: string): HTMLButtonElement {
    const found = Array.from(host.querySelectorAll('button')).find((node) => node.textContent === label);
    if (!found) throw new Error(`Missing button ${label}`);
    return found;
}

describe('WorkspaceEditor', () => {
    it('disables editing and save actions with no folder or file', () => {
        mount();
        expect(host.textContent).toContain('No folder open');
        expect(host.textContent).toContain('No file open');
        expect(button('Save').disabled).toBe(true);
        expect(button('Save as').disabled).toBe(true);
        expect(host.querySelector('textarea')!.disabled).toBe(true);
        expect(host.textContent).not.toContain('Saved');
    });

    it('synchronizes folder, file, tabs, and dirty content through file events', () => {
        const { store, fileApi } = mount();
        fileApi.summary.mode = 'directory';
        fileApi.summary.rootName = 'project';
        act(() => { emit(fileApi); });
        expect(store.getState().file.phase).toBe('no-open-file');
        fileApi.summary.fileCount = 2;
        fileApi.summary.tabs = [{ path: 'a.md', dirty: false }, { path: 'b.md', dirty: false }];
        fileApi.summary.activePath = 'a.md';
        fileApi.setActive({ path: 'a.md', name: 'a.md', content: 'alpha', mode: 'directory' });
        act(() => { emit(fileApi); });
        expect(store.getState().file.phase).toBe('clean');
        expect(host.querySelector('textarea')!.value).toBe('alpha');
        act(() => {
            const editor = host.querySelector('textarea')!;
            editor.value = 'alpha edited';
            editor.dispatchEvent(new Event('input', { bubbles: true }));
        });
        expect(fileApi.api.setActiveContent).toHaveBeenCalledWith('alpha edited');
        expect(store.getState().file.phase).toBe('dirty');
        act(() => { button('Open b.md').click(); });
        expect(fileApi.api.openFile).toHaveBeenCalledWith('b.md');
    });

    it('tracks saving, saved, and save failure states through the existing save method', async () => {
        const { store, fileApi, onError } = mount();
        fileApi.summary.mode = 'directory';
        fileApi.summary.rootName = 'project';
        fileApi.summary.activePath = 'a.md';
        fileApi.setActive({ path: 'a.md', name: 'a.md', content: 'alpha', mode: 'directory' });
        act(() => { emit(fileApi); });
        let finish!: (value: boolean) => void;
        fileApi.api.saveActive = vi.fn(() => new Promise<boolean>((resolve) => { finish = resolve; }));
        act(() => { button('Save').click(); });
        expect(store.getState().file.phase).toBe('saving');
        expect(button('Save').disabled).toBe(true);
        await vi.waitFor(() => expect(fileApi.api.saveActive).toHaveBeenCalledOnce());
        await act(async () => { finish(true); await Promise.resolve(); });
        expect(store.getState().file.phase).toBe('saved');
        fileApi.summary.tabs = [{ path: 'a.md', dirty: true }];
        fileApi.api.saveActive = vi.fn(async () => false);
        await act(async () => { button('Save').click(); await Promise.resolve(); });
        expect(store.getState().file.phase).toBe('error');
        expect(onError).toHaveBeenCalledWith('Save did not complete.', 'file:save:a.md');
        expect(button('Save').disabled).toBe(false);
    });

    it('does not apply an old save result to a newly selected file', async () => {
        const { store, fileApi, onError } = mount();
        fileApi.summary.mode = 'directory';
        fileApi.summary.rootName = 'project';
        fileApi.summary.activePath = 'a.md';
        fileApi.setActive({ path: 'a.md', name: 'a.md', content: 'alpha', mode: 'directory' });
        act(() => { emit(fileApi); });
        let finish!: (value: boolean) => void;
        fileApi.api.saveActive = vi.fn(() => new Promise<boolean>((resolve) => { finish = resolve; }));
        act(() => { button('Save').click(); });
        await vi.waitFor(() => expect(fileApi.api.saveActive).toHaveBeenCalledOnce());
        fileApi.summary.activePath = 'b.md';
        fileApi.setActive({ path: 'b.md', name: 'b.md', content: 'bravo', mode: 'directory' });
        act(() => { emit(fileApi); });
        await act(async () => { finish(false); await Promise.resolve(); });
        expect(store.getState().file).toMatchObject({ path: 'b.md', phase: 'clean', error: null });
        expect(onError).not.toHaveBeenCalled();
    });

    it('applies font, wrap, and Tab settings to the typed editor', () => {
        const { fileApi } = mount({ theme: 'dark', reduceMotion: true, fontSize: 20, tabSize: 2, wordWrap: false });
        fileApi.summary.mode = 'directory';
        fileApi.summary.rootName = 'project';
        fileApi.summary.activePath = 'a.md';
        fileApi.setActive({ path: 'a.md', name: 'a.md', content: 'a', mode: 'directory' });
        act(() => { emit(fileApi); });
        const editor = host.querySelector('textarea')!;
        expect(editor.style.fontSize).toBe('20px');
        expect(editor.getAttribute('wrap')).toBe('off');
        editor.setSelectionRange(1, 1);
        act(() => { editor.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true, cancelable: true })); });
        expect(fileApi.api.setActiveContent).toHaveBeenCalledWith('a  ');
    });

    it('refuses save-as when the store and active file differ', () => {
        const { store, fileApi, onError } = mount();
        fileApi.summary.mode = 'directory';
        fileApi.summary.activePath = 'a.md';
        fileApi.setActive({ path: 'b.md', name: 'b.md', content: 'bravo', mode: 'directory' });
        store.dispatch({ type: 'file/opened', path: 'a.md' });
        store.dispatch({ type: 'file/dirty' });
        act(() => { emit(fileApi); });
        button('Save as').click();
        expect(fileApi.api.saveAsCopy).not.toHaveBeenCalled();
        expect(onError).toHaveBeenCalledWith('The active file changed before Save as.', 'file:save-as:a.md');
    });

    it('reports a rejected save-as instead of treating it as success', () => {
        const { fileApi, onError } = mount();
        fileApi.summary.mode = 'directory';
        fileApi.summary.rootName = 'project';
        fileApi.summary.activePath = 'a.md';
        fileApi.setActive({ path: 'a.md', name: 'a.md', content: 'alpha', mode: 'directory' });
        act(() => { emit(fileApi); });
        fileApi.api.saveAsCopy = vi.fn(() => false);
        button('Save as').click();
        expect(onError).toHaveBeenCalledWith('Save as did not complete.', 'file:save-as:a.md');
    });

    it('uses save-as and closes tabs through the existing API', () => {
        const { store, fileApi } = mount();
        fileApi.summary.mode = 'directory';
        fileApi.summary.rootName = 'project';
        fileApi.summary.activePath = 'a.md';
        fileApi.summary.tabs = [{ path: 'a.md', dirty: false }];
        fileApi.setActive({ path: 'a.md', name: 'a.md', content: 'alpha', mode: 'directory' });
        act(() => { emit(fileApi); });
        button('Save as').click();
        expect(fileApi.api.saveAsCopy).toHaveBeenCalledOnce();
        act(() => { button('Close a.md').click(); });
        expect(fileApi.api.closeTab).toHaveBeenCalledWith('a.md');
        fileApi.summary.activePath = null;
        fileApi.summary.tabs = [];
        fileApi.setActive(null);
        act(() => { emit(fileApi); });
        expect(store.getState().file.phase).toBe('no-open-file');
        expect(button('Save').disabled).toBe(true);
    });

    it('ignores an async workspace rejection after unmount', async () => {
        const { fileApi, onError } = mount();
        let rejectOpen!: (error: Error) => void;
        fileApi.api.openFolder = vi.fn((): Promise<void> => new Promise((_resolve, reject) => { rejectOpen = reject; }));
        act(() => { button('Open folder').click(); });
        act(() => { render(null, host); });
        await act(async () => { rejectOpen(new Error('late open failure')); await Promise.resolve(); });
        expect(onError).not.toHaveBeenCalled();
    });

    it('does not let an older workspace success override a newer failure', async () => {
        const { fileApi, onError, onSuccess } = mount();
        let finishFirst!: () => void;
        let rejectSecond!: (error: Error) => void;
        fileApi.api.openFolder = vi.fn()
            .mockImplementationOnce(() => new Promise<void>((resolve) => { finishFirst = resolve; }))
            .mockImplementationOnce((): Promise<void> => new Promise((_resolve, reject) => { rejectSecond = reject; }));
        act(() => { button('Open folder').click(); });
        act(() => { button('Open folder').click(); });
        await act(async () => { rejectSecond(new Error('newer failure')); await Promise.resolve(); });
        await act(async () => { finishFirst(); await Promise.resolve(); });
        expect(onError).toHaveBeenCalledWith('newer failure', 'workspace:open-folder');
        expect(onSuccess).not.toHaveBeenCalled();
    });
});

describe('WorkspaceEditor host grants', () => {
    const GRANT_A = 'gr_9a3c5e7b1d2f4a60';
    const GRANT_B = 'gr_0b1d2f4a6c8e9a30';

    function hostClient(
        grants: Array<{ grantId: string; path: string }>,
        calls: Array<Record<string, unknown>>,
    ): AgentClient {
        return {
            hostCapabilities: vi.fn(async () => ({
                tree: true,
                changes: true,
                readArtifact: false,
                terminal: false,
                git: true,
                mcp: false,
                approvals: false,
            })),
            hostSupportsGrants: () => true,
            grants: vi.fn(async () => grants),
            treePage: vi.fn(async (input: Record<string, unknown>) => {
                calls.push(input);
                return {
                    entries: [{ path: 'src', kind: 'directory' }, { path: 'src/a.ts', kind: 'file', sizeBytes: 5 }],
                    truncated: false,
                    truncatedReported: false,
                    symlinksSkipped: 0,
                    readErrors: 0,
                };
            }),
            registerLocalFile: vi.fn(),
        } as unknown as AgentClient;
    }

    function mountWith(client: AgentClient): { onError: ReturnType<typeof vi.fn> } {
        const store = new LifecycleStore();
        const fileApi = files();
        const onError = vi.fn();
        act(() => {
            render(
                <WorkspaceEditor
                    store={store}
                    files={fileApi.api}
                    operations={createOperationGate()}
                    onError={onError}
                    onSuccess={vi.fn()}
                    agentClient={client}
                />,
                host,
            );
        });
        return { onError };
    }

    it('single_grant_is_auto_selected_and_passed_to_the_host_tree', async () => {
        const calls: Array<Record<string, unknown>> = [];
        mountWith(hostClient([{ grantId: GRANT_A, path: '/granted/project' }], calls));
        await vi.waitFor(() => expect(calls).toHaveLength(1));
        expect(calls[0]).toMatchObject({ source: 'host', grantId: GRANT_A });
        expect(host.textContent).toContain('/granted/project');
        expect(host.querySelector('select[aria-label="Approved folder"]')).toBeNull();
    });

    it('multiple_grants_require_a_labelled_selection_before_the_host_tree', async () => {
        const calls: Array<Record<string, unknown>> = [];
        mountWith(hostClient([
            { grantId: GRANT_A, path: '/granted/project' },
            { grantId: GRANT_B, path: '/other/project' },
        ], calls));
        await vi.waitFor(() =>
            expect(host.querySelector('select[aria-label="Approved folder"]')).not.toBeNull(),
        );
        expect(calls).toHaveLength(0);
        const select = host.querySelector<HTMLSelectElement>('select[aria-label="Approved folder"]')!;
        expect(select.options).toHaveLength(3);
        expect(select.value).toBe('');
        act(() => {
            select.value = GRANT_B;
            select.dispatchEvent(new Event('change', { bubbles: true }));
        });
        await vi.waitFor(() => expect(calls).toHaveLength(1));
        expect(calls[0]).toMatchObject({ source: 'host', grantId: GRANT_B });
    });

    it('no_grants_reports_unavailable_instead_of_listing_a_page_path', async () => {
        const calls: Array<Record<string, unknown>> = [];
        mountWith(hostClient([], calls));
        await vi.waitFor(() => expect(host.textContent).toContain('reports no approved folders'));
        expect(calls).toHaveLength(0);
    });
});
