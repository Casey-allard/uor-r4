import { useEffect, useRef, useState } from 'preact/hooks';
import type { AgentClient, HostGrant, TreeEntry } from '../agent/client.js';
import { sha256Bytes } from '../agent/hostClient.js';
import type { StudioSettings } from '../adapters/workflow.js';
import { saveActiveFile, saveActiveFileCopy } from '../adapters/fileSave.js';
import type { OperationGate } from '../adapters/operations.js';
import { syncWorkspaceSummary } from '../adapters/workflow.js';
import type { WorkspaceApi, WorkspaceSummary } from '../adapters/workflow.js';
import type { LifecycleStore } from '../state/lifecycle.js';
import { useLifecycle } from '../state/useLifecycle.js';
import { EditorPane, fileTooLargeMessage } from './EditorPane.js';
import { EditorTabs } from './EditorTabs.js';
import { FileTree, type FileTreeMeta } from './FileTree.js';
import type { FileNode } from './types.js';
import { MAX_DOCUMENT_BYTES } from './ComposerAttachments.js';
import { Icon } from './ui/Icon.js';

interface WorkspaceEditorProps {
    store: LifecycleStore;
    files: WorkspaceApi;
    operations: OperationGate;
    onError(message: string, operation: string): void;
    onSuccess(operation: string): void;
    settings?: StudioSettings;
    agentClient?: AgentClient | null;
}

const defaultSettings: StudioSettings = { theme: 'dark', reduceMotion: false, fontSize: 13, tabSize: 4, wordWrap: false };
const emptyMeta: FileTreeMeta = { truncated: false, truncatedReported: false, symlinksSkipped: 0, readErrors: 0 };

interface TreeState {
    phase: 'loading' | 'ready' | 'error';
    nodes: FileNode[];
    meta: FileTreeMeta;
    error: string | null;
}

function fileNodes(entries: TreeEntry[]): FileNode[] {
    return entries.map((entry) => ({
        path: entry.path,
        name: entry.path.split('/').pop() || entry.path,
        kind: entry.kind,
        ...(entry.sizeBytes === undefined ? {} : { sizeBytes: entry.sizeBytes }),
    }));
}

export function WorkspaceEditor({ store, files, operations, onError, onSuccess, settings = defaultSettings, agentClient = null }: WorkspaceEditorProps) {
    const state = useLifecycle(store);
    const mounted = useRef(true);
    const treeVersion = useRef(0);
    const treeKey = useRef('');
    const searchRef = useRef<HTMLInputElement>(null);
    const localUploads = useRef(false);
    const [, setRevision] = useState(0);
    const [tree, setTree] = useState<TreeState>({ phase: 'loading', nodes: [], meta: emptyMeta, error: null });
    const [grants, setGrants] = useState<HostGrant[]>([]);
    const [grantId, setGrantId] = useState('');
    const grantIdRef = useRef('');
    const refresh = () => setRevision((value) => value + 1);
    const report = (error: unknown, operation: string) => { if (mounted.current) onError(error instanceof Error ? error.message : String(error), operation); };
    const succeed = (operation: string) => { if (mounted.current) onSuccess(operation); };

    function sync(summary: WorkspaceSummary) {
        syncWorkspaceSummary(store, summary);
    }

    async function loadTree(requestedGrantId?: string) {
        if (agentClient === null) {
            if (mounted.current) setTree({ phase: 'ready', nodes: [], meta: emptyMeta, error: null });
            return;
        }
        treeVersion.current += 1;
        const version = treeVersion.current;
        setTree((current) => ({ ...current, phase: 'loading', error: null }));
        try {
            const capabilities = await agentClient.hostCapabilities();
            localUploads.current = !capabilities.tree;
            if (!capabilities.tree) {
                if (mounted.current && treeVersion.current === version) {
                    setGrants([]);
                    setGrantId('');
                    grantIdRef.current = '';
                }
                const page = await agentClient.treePage();
                if (!mounted.current || treeVersion.current !== version) return;
                setTree({
                    phase: 'ready',
                    nodes: fileNodes(page.entries),
                    meta: { truncated: page.truncated, truncatedReported: page.truncatedReported, symlinksSkipped: page.symlinksSkipped, readErrors: page.readErrors },
                    error: null,
                });
                return;
            }
            if (typeof agentClient.hostSupportsGrants !== 'function' || !agentClient.hostSupportsGrants()) {
                const page = await agentClient.treePage({ source: 'host' });
                if (!mounted.current || treeVersion.current !== version) return;
                setTree({
                    phase: 'ready',
                    nodes: fileNodes(page.entries),
                    meta: { truncated: page.truncated, truncatedReported: page.truncatedReported, symlinksSkipped: page.symlinksSkipped, readErrors: page.readErrors },
                    error: null,
                });
                return;
            }
            const available = await agentClient.grants();
            if (!mounted.current || treeVersion.current !== version) return;
            setGrants(available);
            if (available.length === 0) {
                setGrantId('');
                grantIdRef.current = '';
                setTree({
                    phase: 'error',
                    nodes: [],
                    meta: emptyMeta,
                    error: 'File browsing requires an approved local folder grant. The local companion reports no approved folders.',
                });
                return;
            }
            const preferred = requestedGrantId ?? grantIdRef.current;
            const selected = available.some((grant) => grant.grantId === preferred)
                ? preferred
                : available.length === 1
                  ? available[0]!.grantId
                  : '';
            setGrantId(selected);
            grantIdRef.current = selected;
            if (selected === '') {
                setTree({ phase: 'ready', nodes: [], meta: emptyMeta, error: null });
                return;
            }
            const page = await agentClient.treePage({ source: 'host', grantId: selected });
            if (!mounted.current || treeVersion.current !== version) return;
            setTree({
                phase: 'ready',
                nodes: fileNodes(page.entries),
                meta: { truncated: page.truncated, truncatedReported: page.truncatedReported, symlinksSkipped: page.symlinksSkipped, readErrors: page.readErrors },
                error: null,
            });
        } catch (error) {
            if (!mounted.current || treeVersion.current !== version) return;
            setTree({ phase: 'error', nodes: [], meta: emptyMeta, error: error instanceof Error ? error.message : String(error) });
        }
    }

    useEffect(() => {
        mounted.current = true;
        return () => { mounted.current = false; };
    }, []);

    useEffect(() => {
        const changed = (event: Event) => {
            sync((event as CustomEvent<WorkspaceSummary>).detail);
            refresh();
        };
        document.addEventListener('uor-files-changed', changed);
        document.addEventListener('uor-workspace-changed', changed);
        sync(files.treeSummary());
        return () => {
            document.removeEventListener('uor-files-changed', changed);
            document.removeEventListener('uor-workspace-changed', changed);
        };
    }, [files, store]);

    useEffect(() => {
        const summary = files.treeSummary();
        treeKey.current = `${summary.mode}:${summary.rootName}:${summary.fileCount}`;
        void loadTree();
        const reload = (event: Event) => {
            const detail = (event as CustomEvent<WorkspaceSummary | undefined>).detail;
            const key = detail === undefined
                ? ''
                : `${detail.mode}:${detail.rootName}:${detail.fileCount}`;
            if (key === treeKey.current) return;
            treeKey.current = key;
            void loadTree();
        };
        document.addEventListener('uor-files-changed', reload);
        document.addEventListener('uor-workspace-changed', reload);
        return () => {
            document.removeEventListener('uor-files-changed', reload);
            document.removeEventListener('uor-workspace-changed', reload);
        };
    }, [agentClient]);

    function perform(action: () => Promise<unknown>, operation: string) {
        void operations.run(operation, action(), (completed) => {
            sync(files.treeSummary());
            refresh();
            succeed(completed);
        }, report);
    }

    async function rememberUploaded(path: string) {
        if (agentClient === null || !localUploads.current || !mounted.current) return;
        const file = files.activeFile();
        if (file === null || file.path !== path) return;
        try {
            const bytes = new TextEncoder().encode(file.content);
            agentClient.registerLocalFile({ path, mediaType: 'text/plain', bytes, digest: await sha256Bytes(bytes) });
            await loadTree();
        } catch {
            return;
        }
    }

    function immediate(action: () => unknown, operation: string) {
        try {
            action();
            sync(files.treeSummary());
            refresh();
            succeed(operation);
        } catch (error) {
            report(error, operation);
        }
    }

    async function save() {
        await saveActiveFile(store, files, report, succeed);
        if (!mounted.current) return;
        sync(files.treeSummary());
        refresh();
    }

    function saveCopy() {
        saveActiveFileCopy(store, files, report, succeed);
        sync(files.treeSummary());
        refresh();
    }

    function openPath(path: string) {
        const node = tree.nodes.find((candidate) => candidate.path === path);
        if (node?.kind === 'file' && node.sizeBytes !== undefined && node.sizeBytes > MAX_DOCUMENT_BYTES) {
            onError(fileTooLargeMessage(node.name, node.sizeBytes), `file:open:${path}`);
            return;
        }
        void operations.run(
            `workspace:open:${path}`,
            (async () => {
                await files.openFile(path);
                const opened = files.activeFile();
                if (opened === null || opened.path !== path) {
                    throw new Error(`${path} is not available in this editor build.`);
                }
            })(),
            (completed) => {
                sync(files.treeSummary());
                refresh();
                succeed(completed);
                void rememberUploaded(path);
            },
            report,
        );
    }

    const summary = files.treeSummary();
    const active = files.activeFile();
    const canSave = store.canSave();

    return (
        <section aria-label="Workspace editor" class="workspace-editor">
            <header class="workspace-heading"><div><h1>Workspace</h1><p>{summary.rootName || 'No folder open'}</p></div><div class="workspace-actions">
                <button type="button" onClick={() => void perform(files.openFolder, 'workspace:open-folder')}><Icon name="folder" />Open folder</button>
                <button type="button" onClick={() => immediate(files.importFiles, 'workspace:import-files')}>Import files</button>
                <button type="button" onClick={() => immediate(files.openDemo, 'workspace:demo')}>Open demo</button>
                <button type="button" onClick={() => immediate(files.newFile, 'workspace:new-file')}><Icon name="plus" />New file</button>
            </div></header>
            <p class="editor-status">File: {state.file.phase}{state.file.path ? ` · ${state.file.path}` : ' · No file open'}</p>
            {state.file.error && <p role="alert" class="workbench-alert">{state.file.error}</p>}
            {grants.length > 1 && <div class="workspace-grant">
                <label for="workspace-grant-select">Approved folder</label>
                <select
                    id="workspace-grant-select"
                    aria-label="Approved folder"
                    value={grantId}
                    onChange={(event) => {
                        const value = event.currentTarget.value;
                        setGrantId(value);
                        grantIdRef.current = value;
                        void loadTree(value);
                    }}
                >
                    <option value="">Select an approved folder</option>
                    {grants.map((grant) => <option key={grant.grantId} value={grant.grantId}>{grant.path}</option>)}
                </select>
                {grantId === '' && <p class="workspace-grant-prompt">Choose an approved folder before browsing host files.</p>}
            </div>}
            {grants.length === 1 && <p class="workspace-grant-single">Approved folder: {grants[0]!.path}</p>}
            <FileTree
                nodes={tree.nodes}
                meta={tree.meta}
                activePath={summary.activePath}
                phase={tree.phase}
                error={tree.error}
                onOpen={openPath}
                searchRef={searchRef}
            />
            <EditorTabs
                tabs={summary.tabs}
                activePath={summary.activePath}
                onSelect={openPath}
                onClose={(path) => immediate(() => files.closeTab(path), `workspace:close:${path}`)}
                onFocusFallback={() => searchRef.current?.focus()}
            />
            <EditorPane
                active={active}
                phase={state.file.phase}
                settings={settings}
                onContentChange={(content) => {
                    files.setActiveContent(content);
                    store.dispatch({ type: 'file/dirty' });
                    refresh();
                }}
            />
            <footer class="workspace-footer"><button type="button" disabled={!canSave} onClick={() => void save()}><Icon name="save" />Save</button><button type="button" disabled={!canSave} onClick={saveCopy}>Save as</button></footer>
        </section>
    );
}
