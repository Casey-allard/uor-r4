import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { AgentClient } from '../agent/client.js';
import {
    HOST_RESULT_SCHEMA,
    HostClient,
    sha256Canonical,
    type HostDataRequest,
    type HostInvokeResponse,
    type HostSubscription,
    type HostToolIntent,
    type HostToolResult,
    type HostTransport,
    type HostTransportCapabilities,
} from '../agent/hostClient.js';
import { AgentClientError } from '../agent/storeTypes.js';
import { createOperationGate } from '../adapters/operations.js';
import type { StudioFile, WorkspaceApi, WorkspaceSummary } from '../adapters/workflow.js';
import { LifecycleStore } from '../state/lifecycle.js';
import { FileTree, MAX_TREE_SEARCH_RESULTS, MAX_TREE_SEARCH_SCAN, searchFileNodes } from './FileTree.js';
import type { FileNode } from './types.js';
import { WorkspaceEditor } from './WorkspaceEditor.js';

const ZERO_TASK_ID = 'tsk_0000000000000000';
const ZERO_RUN_ID = 'run_0000000000000000';

const HOST_CAPS: HostTransportCapabilities = {
    kind: 'bridge', hostAvailable: true, streaming: 'none',
    tree: true, changes: false, readArtifact: false, terminal: false, git: false, mcp: false, approvals: false,
};

async function hostResult(intent: HostToolIntent, output: unknown): Promise<HostToolResult> {
    return {
        schema: HOST_RESULT_SCHEMA,
        intent_id: intent.intent_id,
        task_id: intent.task_id,
        run_id: intent.run_id,
        generation: intent.generation,
        tool_call_id: intent.tool_call_id,
        request_scope: intent.request_scope ?? 'task',
        status: 'ok',
        output,
        output_digest: await sha256Canonical(output),
        output_preview: '',
        occurred_at_ms: 1_800_000_000_000,
    };
}

class RecordingHostTransport implements HostTransport {
    readonly kind = 'bridge' as const;
    readonly supportsStream = false;
    readonly intents: HostToolIntent[] = [];
    readonly contexts: Array<HostDataRequest | undefined> = [];

    constructor(private readonly output: unknown) {}

    async capabilities(): Promise<HostTransportCapabilities> {
        return { ...HOST_CAPS };
    }

    async invoke(intent: HostToolIntent, context?: HostDataRequest): Promise<HostInvokeResponse> {
        this.intents.push(intent);
        this.contexts.push(context);
        return hostResult(intent, this.output);
    }

    subscribe(): HostSubscription {
        return { close: () => undefined, cursor: () => null };
    }

    async approve(input: { approvalId: string; decision: 'approved' | 'denied' }) {
        return { approvalId: input.approvalId, decision: input.decision };
    }
}

class StaticHostTransport implements HostTransport {
    readonly kind = 'static' as const;
    readonly supportsStream = false;
    readonly intents: HostToolIntent[] = [];

    async capabilities(): Promise<HostTransportCapabilities> {
        return {
            kind: 'static', hostAvailable: false, streaming: 'none',
            tree: false, changes: false, readArtifact: false, terminal: false, git: false, mcp: false, approvals: false,
        };
    }

    async invoke(intent: HostToolIntent): Promise<HostInvokeResponse> {
        this.intents.push(intent);
        throw new AgentClientError('host_unavailable', 'the static transport has no host');
    }

    subscribe(): HostSubscription {
        throw new AgentClientError('host_unavailable', 'the static transport has no host');
    }

    async approve(): Promise<never> {
        throw new AgentClientError('host_unavailable', 'the static transport has no host');
    }
}

let host: HTMLDivElement;
beforeEach(() => { host = document.createElement('div'); document.body.appendChild(host); });
afterEach(() => { render(null, host); host.remove(); vi.restoreAllMocks(); });

function workspace(library: Record<string, string> = {}) {
    const summary: WorkspaceSummary = { mode: 'none', rootName: '', fileCount: 0, tabs: [], activePath: null };
    let active: StudioFile | null = null;
    const api: WorkspaceApi = {
        openFolder: vi.fn(async () => undefined), importFiles: vi.fn(), openDemo: vi.fn(),
        openFile: vi.fn(async (path: string) => {
            active = { path, name: path.split('/').pop()!, content: library[path] ?? 'alpha', mode: 'local' };
            summary.activePath = path;
            summary.tabs = [{ path, dirty: false }];
        }),
        newFile: vi.fn(), closeTab: vi.fn(), closeAll: vi.fn(),
        saveActive: vi.fn(async () => true), saveAsCopy: vi.fn(() => true), insertText: vi.fn(() => true),
        setActiveContent: vi.fn(), activeFile: vi.fn(() => active), workspaceName: vi.fn(() => ''),
        mode: vi.fn(() => summary.mode), tabs: vi.fn(() => []), treeSummary: vi.fn(() => summary),
    };
    return { api, summary };
}

function mount(files: WorkspaceApi, agentClient: AgentClient | null) {
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
                agentClient={agentClient}
            />,
            host,
        );
    });
    return { store, onError, onSuccess };
}

function treeButtons(): HTMLButtonElement[] {
    return Array.from(host.querySelectorAll<HTMLButtonElement>('button.tree-file'));
}

function nodes(count: number, path: (index: number) => string): FileNode[] {
    return Array.from({ length: count }, (_, index) => ({ path: path(index), name: path(index).split('/').pop()!, kind: 'file' as const }));
}

describe('FileTree', () => {
    it('file_tree_uses_agent_client_tree', async () => {
        const hostTransport = new RecordingHostTransport({
            grant_id: 'gr_abcd1234',
            path: '',
            entries: [
                { path: 'src', kind: 'directory' },
                { path: 'src/main.ts', kind: 'file', size_bytes: 12 },
                { path: 'readme.md', kind: 'file', size_bytes: 4 },
            ],
            truncated: true,
            symlinks_skipped: 2,
            read_errors: 1,
        });
        const client = new AgentClient({ hostClient: new HostClient({ transport: hostTransport }) });
        const files = workspace();
        mount(files.api, client);

        await vi.waitFor(() => expect(treeButtons().map((button) => button.textContent)).toEqual(['main.ts', 'readme.md']));
        expect(host.textContent).toContain('src');
        expect(hostTransport.intents.map((intent) => intent.tool)).toEqual(['file.tree']);
        expect(hostTransport.intents[0]!.request_scope).toBe('data');
        expect(hostTransport.intents[0]!.task_id).toBe(ZERO_TASK_ID);
        expect(hostTransport.intents[0]!.run_id).toBe(ZERO_RUN_ID);
        expect(hostTransport.intents[0]!.arguments).toEqual({});

        const text = host.textContent ?? '';
        expect(text).toContain('The file list is incomplete: the companion returned the first 3 entries.');
        expect(text).toContain('2 symbolic links were skipped.');
        expect(text).toContain('1 entry could not be read.');

        act(() => { treeButtons().find((button) => button.textContent === 'main.ts')!.click(); });
        expect(files.api.openFile).toHaveBeenCalledWith('src/main.ts');
    });

    it('host_tree_open_refuses_an_unknown_path_instead_of_reporting_success', async () => {
        const transport = new RecordingHostTransport({
            grant_id: 'gr_abcd1234',
            path: '',
            entries: [{ path: 'src/main.ts', kind: 'file', size_bytes: 12 }],
            truncated: false,
            symlinks_skipped: 0,
            read_errors: 0,
        });
        const client = new AgentClient({ hostClient: new HostClient({ transport }) });
        const files = workspace();
        files.api.openFile = vi.fn(async () => undefined);
        const { onError, onSuccess } = mount(files.api, client);

        await vi.waitFor(() => expect(treeButtons().map((button) => button.textContent)).toEqual(['main.ts']));
        act(() => { treeButtons()[0]!.click(); });
        await vi.waitFor(() => expect(onError).toHaveBeenCalledWith(
            'src/main.ts is not available in this editor build.',
            'workspace:open:src/main.ts',
        ));
        expect(onSuccess).not.toHaveBeenCalledWith('workspace:open:src/main.ts');
    });

    it('host_tree_open_refuses_when_another_file_stays_active', async () => {
        const transport = new RecordingHostTransport({
            grant_id: 'gr_abcd1234',
            path: '',
            entries: [{ path: 'src/main.ts', kind: 'file', size_bytes: 12 }],
            truncated: false,
            symlinks_skipped: 0,
            read_errors: 0,
        });
        const client = new AgentClient({ hostClient: new HostClient({ transport }) });
        const files = workspace();
        files.api.openFile = vi.fn(async () => undefined);
        files.api.activeFile = vi.fn(() => ({ path: 'readme.md', name: 'readme.md', content: 'old', mode: 'local' }));
        const { onError, onSuccess } = mount(files.api, client);

        await vi.waitFor(() => expect(treeButtons().map((button) => button.textContent)).toEqual(['main.ts']));
        act(() => { treeButtons()[0]!.click(); });
        await vi.waitFor(() => expect(onError).toHaveBeenCalledWith(
            'src/main.ts is not available in this editor build.',
            'workspace:open:src/main.ts',
        ));
        expect(onSuccess).not.toHaveBeenCalledWith('workspace:open:src/main.ts');
    });

    it('static_mode_lists_registered_uploads_and_never_calls_the_host', async () => {
        const staticTransport = new StaticHostTransport();
        const client = new AgentClient({ hostClient: new HostClient({ transport: staticTransport }) });
        client.registerLocalFile({
            path: 'notes/today.md', mediaType: 'text/markdown',
            bytes: new TextEncoder().encode('today'), digest: `sha256:${'a'.repeat(64)}`,
        });
        const files = workspace();
        mount(files.api, client);

        await vi.waitFor(() => expect(treeButtons().map((button) => button.textContent)).toEqual(['today.md']));
        expect(staticTransport.intents).toEqual([]);
    });

    it('opening_an_uploaded_file_lists_it_through_the_local_agent_tree', async () => {
        const staticTransport = new StaticHostTransport();
        const client = new AgentClient({ hostClient: new HostClient({ transport: staticTransport }) });
        const files = workspace({ 'notes/today.md': 'today' });
        files.summary.mode = 'directory';
        files.summary.rootName = 'notes';
        files.summary.tabs = [{ path: 'notes/today.md', dirty: false }];
        mount(files.api, client);

        await vi.waitFor(() => expect(host.textContent).toContain('No files are listed for this session yet.'));
        act(() => { (Array.from(host.querySelectorAll('button')).find((node) => node.textContent === 'Open notes/today.md') as HTMLButtonElement).click(); });
        await vi.waitFor(() => expect(treeButtons().map((button) => button.textContent)).toEqual(['today.md']));
        await expect(client.tree()).resolves.toEqual([{ path: 'notes', kind: 'directory' }, { path: 'notes/today.md', kind: 'file', sizeBytes: 5 }]);
        expect(staticTransport.intents).toEqual([]);
    });

    it('file_tree_search_is_bounded', () => {
        const late = searchFileNodes(
            [...nodes(MAX_TREE_SEARCH_SCAN, (index) => `dir/file-${index}.md`), { path: 'zzz/last-match.md', name: 'last-match.md', kind: 'file' }],
            'last-match',
        );
        expect(late.matches).toEqual([]);
        expect(late.scanned).toBe(MAX_TREE_SEARCH_SCAN);
        expect(late.truncated).toBe(true);

        const many = searchFileNodes(nodes(MAX_TREE_SEARCH_SCAN, (index) => `a/${index}.md`), 'a');
        expect(many.matches).toHaveLength(MAX_TREE_SEARCH_RESULTS);
        expect(many.scanned).toBe(MAX_TREE_SEARCH_RESULTS);
        expect(many.truncated).toBe(true);

        const small = searchFileNodes(
            [
                { path: 'src/main.ts', name: 'main.ts', kind: 'file' },
                { path: 'src', name: 'src', kind: 'directory' },
                { path: 'readme.md', name: 'readme.md', kind: 'file' },
            ],
            'MAIN',
        );
        expect(small.matches.map((node) => node.path)).toEqual(['src/main.ts']);
        expect(small.scanned).toBe(3);
        expect(small.truncated).toBe(false);
    });

    it('file_tree_search_renders_the_bounded_result_set', () => {
        act(() => {
            render(
                <FileTree
                    nodes={nodes(600, (index) => `a/${index}.md`)}
                    meta={{ truncated: false, truncatedReported: false, symlinksSkipped: 0, readErrors: 0 }}
                    activePath={null}
                    phase="ready"
                    error={null}
                    onOpen={() => undefined}
                />,
                host,
            );
        });
        const search = host.querySelector<HTMLInputElement>('input[type="search"]')!;
        expect(search.getAttribute('aria-label')).toBe('Search files');
        act(() => {
            search.value = 'a';
            search.dispatchEvent(new Event('input', { bubbles: true }));
        });
        expect(host.querySelectorAll('button.tree-file')).toHaveLength(MAX_TREE_SEARCH_RESULTS);
        expect(host.querySelector('[role="status"]')?.textContent).toContain(`first ${MAX_TREE_SEARCH_RESULTS} matches`);
        expect(host.querySelector('[role="status"]')?.textContent).toContain('Refine the search to narrow the results.');

        act(() => {
            search.value = 'zzz';
            search.dispatchEvent(new Event('input', { bubbles: true }));
        });
        expect(host.querySelector('[role="status"]')?.textContent).toContain('No loaded files match "zzz".');

        act(() => {
            render(
                <FileTree
                    nodes={nodes(MAX_TREE_SEARCH_SCAN + 5, (index) => `a/${index}.md`)}
                    meta={{ truncated: true, truncatedReported: false, symlinksSkipped: 0, readErrors: 0 }}
                    activePath={null}
                    phase="ready"
                    error={null}
                    onOpen={() => undefined}
                />,
                host,
            );
        });
        expect(host.querySelector('[role="status"]')?.textContent).toContain(`No matches in the first ${MAX_TREE_SEARCH_SCAN} loaded files.`);
        expect(host.querySelector('[role="status"]')?.textContent).toContain(`The search scans at most ${MAX_TREE_SEARCH_SCAN} entries.`);
    });

    it('an_unlabelled_full_listing_is_reported_as_maybe_capped', () => {
        act(() => {
            render(
                <FileTree
                    nodes={nodes(512, (index) => `a/${index}.md`)}
                    meta={{ truncated: true, truncatedReported: false, symlinksSkipped: 0, readErrors: 0 }}
                    activePath={null}
                    phase="ready"
                    error={null}
                    onOpen={() => undefined}
                />,
                host,
            );
        });
        expect(host.textContent).toContain('The companion returned exactly 512 entries without a truncation flag; the listing may be capped.');
        expect(host.textContent).not.toContain('The file list is incomplete:');
    });
});
