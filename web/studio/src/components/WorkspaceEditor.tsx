import { useEffect, useRef, useState } from 'preact/hooks';
import type { StudioSettings } from '../adapters/workflow.js';
import { saveActiveFile, saveActiveFileCopy } from '../adapters/fileSave.js';
import type { OperationGate } from '../adapters/operations.js';
import { syncWorkspaceSummary } from '../adapters/workflow.js';
import type { WorkspaceApi, WorkspaceSummary } from '../adapters/workflow.js';
import type { LifecycleStore } from '../state/lifecycle.js';
import { useLifecycle } from '../state/useLifecycle.js';

interface WorkspaceEditorProps {
    store: LifecycleStore;
    files: WorkspaceApi;
    operations: OperationGate;
    onError(message: string, operation: string): void;
    onSuccess(operation: string): void;
    settings?: StudioSettings;
}

const defaultSettings: StudioSettings = { theme: 'dark', reduceMotion: false, fontSize: 13, tabSize: 4, wordWrap: false };

export function WorkspaceEditor({ store, files, operations, onError, onSuccess, settings = defaultSettings }: WorkspaceEditorProps) {
    const state = useLifecycle(store);
    const mounted = useRef(true);
    const [, setRevision] = useState(0);
    const refresh = () => setRevision((value) => value + 1);
    const report = (error: unknown, operation: string) => { if (mounted.current) onError(error instanceof Error ? error.message : String(error), operation); };
    const succeed = (operation: string) => { if (mounted.current) onSuccess(operation); };

    function sync(summary: WorkspaceSummary) {
        syncWorkspaceSummary(store, summary);
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

    function perform(action: () => Promise<unknown>, operation: string) {
        void operations.run(operation, action(), (completed) => {
            sync(files.treeSummary());
            refresh();
            succeed(completed);
        }, report);
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

    const summary = files.treeSummary();
    const active = files.activeFile();
    const canSave = store.canSave();

    return (
        <section aria-label="Workspace editor">
            <h1>Workspace</h1>
            <p>{summary.rootName || 'No folder open'}</p>
            <p>File: {state.file.phase}{state.file.path ? ` · ${state.file.path}` : ' · No file open'}</p>
            {state.file.error && <p role="alert">{state.file.error}</p>}
            <div>
                <button type="button" onClick={() => void perform(files.openFolder, 'workspace:open-folder')}>Open folder</button>
                <button type="button" onClick={() => immediate(files.importFiles, 'workspace:import-files')}>Import files</button>
                <button type="button" onClick={() => immediate(files.openDemo, 'workspace:demo')}>Open demo</button>
                <button type="button" onClick={() => immediate(files.newFile, 'workspace:new-file')}>New file</button>
            </div>
            {summary.tabs.length > 0 && (
                <ul aria-label="Open editor tabs">
                    {summary.tabs.map((tab) => (
                        <li>
                            <button type="button" aria-current={summary.activePath === tab.path ? 'page' : undefined} onClick={() => void perform(() => files.openFile(tab.path), `workspace:open:${tab.path}`)}>{`Open ${tab.path}`}</button>
                            <button type="button" aria-label={`Close ${tab.path}`} onClick={() => immediate(() => files.closeTab(tab.path), `workspace:close:${tab.path}`)}>{`Close ${tab.path}`}</button>
                            {tab.dirty && <span> Unsaved</span>}
                        </li>
                    ))}
                </ul>
            )}
            <label>Editor
                <textarea
                    aria-label="File contents"
                    disabled={!active || state.file.phase === 'saving'}
                    style={{ fontSize: `${settings.fontSize}px` }}
                    wrap={settings.wordWrap ? 'soft' : 'off'}
                    value={active?.content ?? ''}
                    onKeyDown={(event) => {
                        if (event.key !== 'Tab' || !active) return;
                        event.preventDefault();
                        const input = event.currentTarget;
                        const padding = ' '.repeat(settings.tabSize);
                        const start = input.selectionStart;
                        const end = input.selectionEnd;
                        files.setActiveContent(`${input.value.slice(0, start)}${padding}${input.value.slice(end)}`);
                        store.dispatch({ type: 'file/dirty' });
                        input.setSelectionRange(start + padding.length, start + padding.length);
                        refresh();
                    }}
                    onInput={(event) => {
                        if (!active) return;
                        files.setActiveContent(event.currentTarget.value);
                        store.dispatch({ type: 'file/dirty' });
                        refresh();
                    }}
                />
            </label>
            <div>
                <button type="button" disabled={!canSave} onClick={() => void save()}>Save</button>
                <button type="button" disabled={!canSave} onClick={saveCopy}>Save as</button>
            </div>
        </section>
    );
}
