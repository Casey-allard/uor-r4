import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { AgentClient } from '../agent/client.js';
import {
    HOST_RESULT_SCHEMA,
    HostClient,
    sha256Canonical,
    type HostInvokeResponse,
    type HostSubscription,
    type HostToolIntent,
    type HostToolResult,
    type HostTransport,
    type HostTransportCapabilities,
} from '../agent/hostClient.js';
import { createOperationGate } from '../adapters/operations.js';
import type { StudioFile, StudioSettings, WorkspaceApi, WorkspaceSummary } from '../adapters/workflow.js';
import { LifecycleStore } from '../state/lifecycle.js';
import { MAX_DOCUMENT_BYTES } from './ComposerAttachments.js';
import { fileTooLargeMessage } from './EditorPane.js';
import { WorkspaceEditor } from './WorkspaceEditor.js';

const settings: StudioSettings = { theme: 'dark', reduceMotion: false, fontSize: 13, tabSize: 4, wordWrap: false };

interface WorkspaceOptions {
    files: Record<string, string>;
    tabs: Array<{ path: string; dirty?: boolean }>;
    activePath?: string | null;
    mode?: string;
    deferredSaves?: boolean;
}

function workspace(options: WorkspaceOptions) {
    let activePath: string | null = options.activePath ?? null;
    const tabs = options.tabs.map((tab) => ({
        path: tab.path,
        name: tab.path.split('/').pop()!,
        content: options.files[tab.path] ?? '',
        dirty: Boolean(tab.dirty),
    }));
    const summary: WorkspaceSummary = { mode: 'none', rootName: '', fileCount: 0, tabs: [], activePath: null };
    const writes: Array<string | null> = [];
    const pending: Array<(value: boolean) => void> = [];
    const refresh = () => {
        summary.mode = options.mode ?? 'directory';
        summary.rootName = 'project';
        summary.fileCount = Object.keys(options.files).length;
        summary.tabs = tabs.map((tab) => ({ path: tab.path, dirty: tab.dirty }));
        summary.activePath = activePath;
    };
    const emit = () => document.dispatchEvent(new CustomEvent('uor-files-changed', { detail: summary }));
    const activeTab = () => tabs.find((tab) => tab.path === activePath) ?? null;
    const api: WorkspaceApi = {
        openFolder: vi.fn(async () => undefined), importFiles: vi.fn(), openDemo: vi.fn(),
        openFile: vi.fn(async (path: string) => {
            let tab = tabs.find((candidate) => candidate.path === path);
            if (!tab) {
                tab = { path, name: path.split('/').pop()!, content: options.files[path] ?? '', dirty: false };
                tabs.push(tab);
            }
            activePath = path;
            refresh();
            emit();
        }),
        newFile: vi.fn(), closeTab: vi.fn((path: string) => {
            const index = tabs.findIndex((tab) => tab.path === path);
            if (index >= 0) tabs.splice(index, 1);
            if (activePath === path) activePath = tabs[Math.min(index, tabs.length - 1)]?.path ?? null;
            refresh();
            emit();
        }), closeAll: vi.fn(),
        saveActive: vi.fn(() => {
            writes.push(activePath);
            return options.deferredSaves ? new Promise<boolean>((resolve) => { pending.push(resolve); }) : Promise.resolve(true);
        }), saveAsCopy: vi.fn(() => true), insertText: vi.fn(() => true),
        setActiveContent: vi.fn((content: string) => {
            const tab = activeTab();
            if (tab) { tab.content = content; tab.dirty = true; refresh(); }
        }),
        activeFile: vi.fn((): StudioFile | null => {
            const tab = activeTab();
            return tab ? { path: tab.path, name: tab.name, content: tab.content, mode: options.mode ?? 'directory' } : null;
        }),
        workspaceName: vi.fn(() => 'project'), mode: vi.fn(() => options.mode ?? 'directory'),
        tabs: vi.fn(() => []), treeSummary: vi.fn(() => summary),
    };
    refresh();
    return {
        api, summary, writes, pending,
        setActive(path: string | null) { activePath = path; refresh(); },
        tab(path: string) { return tabs.find((candidate) => candidate.path === path)!; },
    };
}

function mount(files: WorkspaceApi, agentClient: AgentClient | null = null) {
    const store = new LifecycleStore();
    const onError = vi.fn();
    const onSuccess = vi.fn();
    act(() => {
        render(
            <WorkspaceEditor
                store={store}
                files={files}
                operations={createOperationGate()}
                onError={onError}
                onSuccess={onSuccess}
                settings={settings}
                agentClient={agentClient}
            />,
            host,
        );
    });
    return { store, onError, onSuccess };
}

function button(label: string): HTMLButtonElement {
    const found = Array.from(host.querySelectorAll('button')).find((node) => node.textContent === label || node.getAttribute('aria-label') === label);
    if (!found) throw new Error(`missing button ${label}`);
    return found;
}

let showModal: ReturnType<typeof vi.fn>;
let closeDialog: ReturnType<typeof vi.fn>;
let invokerFocus: HTMLElement | null;
let host: HTMLDivElement;

beforeEach(() => {
    host = document.createElement('div');
    document.body.appendChild(host);
    invokerFocus = null;
    showModal = vi.fn(function (this: HTMLDialogElement) {
        invokerFocus = document.activeElement as HTMLElement | null;
        this.open = true;
        this.querySelector<HTMLElement>('[autofocus]')?.focus();
    });
    closeDialog = vi.fn(function (this: HTMLDialogElement, value?: string) {
        if (value !== undefined) this.returnValue = value;
        this.open = false;
        invokerFocus?.focus();
        this.dispatchEvent(new Event('close'));
    });
    Object.defineProperty(HTMLDialogElement.prototype, 'showModal', { configurable: true, value: showModal });
    Object.defineProperty(HTMLDialogElement.prototype, 'close', { configurable: true, value: closeDialog });
});

afterEach(() => {
    render(null, host);
    host.remove();
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
});

describe('EditorPane and tabs', () => {
    it('editor_labels_it_is_not_vs_code_or_lsp', () => {
        mount(workspace({ files: {}, tabs: [] }).api);
        expect(host.querySelector('.editor-pane h2')?.textContent).toBe('UOR text editor');
        expect(host.textContent).toContain('Language services, debugger, extensions and notebooks are not available.');
        expect(host.textContent).not.toMatch(/VS Code|LSP|Monaco|CodeMirror|language server/i);
    });

    it('dirty_tab_close_requires_confirmation', () => {
        const files = workspace({
            files: { 'a.md': 'alpha', 'b.md': 'bravo' },
            tabs: [{ path: 'a.md', dirty: true }, { path: 'b.md' }],
            activePath: 'a.md',
        });
        mount(files.api);

        const close = button('Close a.md');
        close.focus();
        act(() => { close.click(); });
        expect(showModal).toHaveBeenCalledOnce();
        expect(files.api.closeTab).not.toHaveBeenCalled();
        const dialog = host.querySelector('dialog')!;
        expect(dialog.getAttribute('role')).toBeNull();
        expect(dialog.getAttribute('aria-modal')).toBeNull();
        expect(dialog.textContent).toContain('Discard changes to a.md?');

        act(() => { dialog.close('cancel'); });
        expect(files.api.closeTab).not.toHaveBeenCalled();
        expect(document.activeElement).toBe(button('Close a.md'));

        act(() => { button('Close a.md').click(); });
        act(() => { host.querySelector('dialog')!.close('discard'); });
        expect(files.api.closeTab).toHaveBeenCalledWith('a.md');
        expect(document.activeElement?.textContent).toContain('Open b.md');
    });

    it('dirty_tab_close_without_remaining_tabs_focuses_the_tree_search', () => {
        const files = workspace({ files: { 'a.md': 'alpha' }, tabs: [{ path: 'a.md', dirty: true }], activePath: 'a.md' });
        mount(files.api);
        act(() => { button('Close a.md').click(); });
        act(() => { host.querySelector('dialog')!.close('discard'); });
        expect(files.api.closeTab).toHaveBeenCalledWith('a.md');
        expect(document.activeElement).toBe(host.querySelector('input[type="search"]'));
    });

    it('arrow_keys_move_selection_and_focus', () => {
        const files = workspace({
            files: { 'a.md': 'alpha', 'b.md': 'bravo' },
            tabs: [{ path: 'a.md' }, { path: 'b.md' }],
            activePath: 'a.md',
        });
        mount(files.api);
        const first = button('Open a.md');
        first.focus();
        act(() => { first.dispatchEvent(keydown('ArrowRight')); });
        expect(files.api.openFile).toHaveBeenCalledWith('b.md');
        expect(document.activeElement?.textContent).toContain('Open b.md');
    });

    it('clean_tab_close_does_not_prompt', () => {
        const files = workspace({
            files: { 'a.md': 'alpha', 'b.md': 'bravo' },
            tabs: [{ path: 'a.md' }, { path: 'b.md' }],
            activePath: 'b.md',
        });
        mount(files.api);
        act(() => { button('Close a.md').click(); });
        expect(showModal).not.toHaveBeenCalled();
        expect(files.api.closeTab).toHaveBeenCalledWith('a.md');
        expect(document.activeElement?.textContent).toContain('Open b.md');
    });

    it('save_serializes_and_verifies_active_path', async () => {
        const files = workspace({
            files: { 'a.md': 'alpha', 'b.md': 'bravo' },
            tabs: [{ path: 'a.md' }, { path: 'b.md' }],
            activePath: 'a.md',
            deferredSaves: true,
        });
        const pending = files.pending;
        const { store, onError, onSuccess } = mount(files.api);

        act(() => { button('Save').click(); });
        await vi.waitFor(() => expect(pending).toHaveLength(1));
        expect(store.getState().file.phase).toBe('saving');

        act(() => { button('Open b.md').click(); });
        await vi.waitFor(() => expect(store.getState().file.path).toBe('b.md'));
        act(() => { button('Save').click(); });
        expect(pending).toHaveLength(1);

        await act(async () => { pending[0]!(true); await Promise.resolve(); });
        await vi.waitFor(() => expect(pending).toHaveLength(2));
        await act(async () => { pending[1]!(true); await Promise.resolve(); });

        expect(files.writes).toEqual(['a.md', 'b.md']);
        expect(store.getState().file).toMatchObject({ path: 'b.md', phase: 'saved', error: null });
        expect(onSuccess).toHaveBeenCalledWith('file:save:b.md');
        expect(onSuccess).not.toHaveBeenCalledWith('file:save:a.md');
        expect(onError).not.toHaveBeenCalled();
    });

    it('save_refuses_a_stale_active_path_without_writing', async () => {
        const files = workspace({
            files: { 'a.md': 'alpha', 'b.md': 'bravo' },
            tabs: [{ path: 'a.md' }, { path: 'b.md' }],
            activePath: 'a.md',
        });
        const { store, onError } = mount(files.api);
        act(() => {
            store.dispatch({ type: 'file/opened', path: 'a.md' });
            store.dispatch({ type: 'file/dirty' });
        });
        files.setActive('b.md');
        act(() => { button('Save').click(); });
        await vi.waitFor(() => expect(onError).toHaveBeenCalledWith('The active file changed before saving.', 'file:save:a.md'));
        expect(files.api.saveActive).not.toHaveBeenCalled();
        expect(files.writes).toEqual([]);
    });

    it('large_file_is_refused', async () => {
        const big = { path: 'big.md', kind: 'file' as const, size_bytes: MAX_DOCUMENT_BYTES + 1 };
        const transport = new RecordingTreeTransport([big]);
        const client = new AgentClient({ hostClient: new HostClient({ transport }) });
        const files = workspace({ files: { 'big.md': '' }, tabs: [], activePath: null });
        const { onError } = mount(files.api, client);

        await vi.waitFor(() => expect(host.querySelectorAll('button.tree-file')).toHaveLength(1));
        act(() => { (host.querySelector('button.tree-file') as HTMLButtonElement).click(); });
        expect(files.api.openFile).not.toHaveBeenCalled();
        expect(onError).toHaveBeenCalledWith(
            'big.md exceeds the 4194304-byte document limit (4194305 bytes).',
            'file:open:big.md',
        );
        expect(fileTooLargeMessage('big.md', MAX_DOCUMENT_BYTES + 1)).toBe('big.md exceeds the 4194304-byte document limit (4194305 bytes).');
    });

    it('oversized_content_is_refused_instead_of_truncated', () => {
        const content = 'x'.repeat(MAX_DOCUMENT_BYTES + 1);
        const files = workspace({ files: { 'big.md': content }, tabs: [{ path: 'big.md' }], activePath: 'big.md' });
        mount(files.api);
        expect(host.querySelector('textarea[aria-label="File contents"]')).toBeNull();
        expect(host.querySelector('[role="alert"]')?.textContent).toBe('big.md exceeds the 4194304-byte document limit (4194305 bytes).');
        expect(host.textContent).not.toContain(content.slice(0, 32));
    });

    it('uses_the_native_editor_surface_with_settings_and_ime_safe_tab_insertion', () => {
        const files = workspace({ files: { 'a.md': 'a' }, tabs: [{ path: 'a.md' }], activePath: 'a.md' });
        mount(files.api);
        expect(host.querySelector('input[aria-label="Search files"]')).not.toBeNull();
        expect(host.querySelector('.code-frame')).not.toBeNull();
        expect(host.querySelector('.code-gutter')).not.toBeNull();
        const editor = host.querySelector<HTMLTextAreaElement>('.code-input textarea[aria-label="File contents"]')!;
        expect(editor.disabled).toBe(false);
        editor.setSelectionRange(1, 1);
        act(() => { editor.dispatchEvent(keydown('Tab', { isComposing: true, keyCode: 229 })); });
        expect(files.api.setActiveContent).not.toHaveBeenCalled();
        act(() => { editor.dispatchEvent(keydown('Tab')); });
        expect(files.api.setActiveContent).toHaveBeenCalledWith('a    ');
    });
});

function keydown(key: string, init: { isComposing?: boolean; keyCode?: number } = {}): KeyboardEvent {
    const event = new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true });
    Object.defineProperty(event, 'isComposing', { value: Boolean(init.isComposing) });
    Object.defineProperty(event, 'keyCode', { value: init.keyCode ?? 0 });
    return event;
}

class RecordingTreeTransport implements HostTransport {
    readonly kind = 'bridge' as const;
    readonly supportsStream = false;
    readonly intents: HostToolIntent[] = [];

    constructor(private readonly entries: unknown[]) {}

    async capabilities(): Promise<HostTransportCapabilities> {
        return {
            kind: 'bridge', hostAvailable: true, streaming: 'none',
            tree: true, changes: false, readArtifact: false, terminal: false, git: false, mcp: false, approvals: false,
        };
    }

    async invoke(intent: HostToolIntent): Promise<HostInvokeResponse> {
        this.intents.push(intent);
        const output = { grant_id: 'gr_abcd1234', path: '', entries: this.entries, truncated: false, symlinks_skipped: 0, read_errors: 0 };
        const result: HostToolResult = {
            schema: HOST_RESULT_SCHEMA, intent_id: intent.intent_id, task_id: intent.task_id, run_id: intent.run_id,
            generation: intent.generation, tool_call_id: intent.tool_call_id, request_scope: intent.request_scope ?? 'task',
            status: 'ok', output, output_digest: await sha256Canonical(output), output_preview: '', occurred_at_ms: 1_800_000_000_000,
        };
        return result;
    }

    subscribe(): HostSubscription {
        return { close: () => undefined, cursor: () => null };
    }

    async approve(input: { approvalId: string; decision: 'approved' | 'denied' }) {
        return { approvalId: input.approvalId, decision: input.decision };
    }
}
