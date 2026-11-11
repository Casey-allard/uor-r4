import { useEffect, useMemo, useState } from 'preact/hooks';
import type { ModelSourceAdapter } from '../adapters/modelSourceAdapter.js';
import type { ModelSourcesApi } from '../../assets/js/model_sources.js';
import { createAgentSessionController } from '../adapters/agentSession.js';
import { saveActiveFile, saveActiveFileCopy } from '../adapters/fileSave.js';
import { createOperationGate } from '../adapters/operations.js';
import { syncWorkspaceSummary } from '../adapters/workflow.js';
import type { SessionApi, StudioUiApi, WorkspaceApi } from '../adapters/workflow.js';
import { AgentClientError, type AgentClient, type TaskSummary } from '../agent/client.js';
import { useScopedShortcuts } from '../hooks/useScopedShortcuts.js';
import { classifyEndpoint } from '../state/lifecycle.js';
import type { LifecycleStore } from '../state/lifecycle.js';
import { useLifecycle } from '../state/useLifecycle.js';
import { AppShell } from './AppShell.js';
import { AgentComposer } from './AgentComposer.js';
import { ArtifactPanel } from './ArtifactPanel.js';
import { CommandPalette } from './CommandPalette.js';
import { Guide } from './Guide.js';
import { Inspector } from './Inspector.js';
import { ModelPicker } from './ModelPicker.js';
import { PrPanel } from './PrPanel.js';
import { ProjectSidebar, TaskRailCountContext } from './ProjectSidebar.js';
import { SessionList } from './SessionList.js';
import { Settings } from './Settings.js';
import { LOCAL_SESSIONS_LABEL_ID, TaskRail } from './TaskRail.js';
import { WorkspaceEditor } from './WorkspaceEditor.js';
import { createDesktopActions } from '../adapters/desktopActions.js';
import type { DesktopActions } from '../adapters/desktopActions.js';
import { ChangesPanel, TerminalPanel } from './DesktopTools.js';
import { McpConnections } from './McpConnections.js';
import type { McpConnect } from '../adapters/mcpConnections.js';
import type { PaletteCommand, ShellStatus, StudioSection } from './types.js';

interface StudioAppProps { store: LifecycleStore; adapter: ModelSourceAdapter; sources: ModelSourcesApi; files: WorkspaceApi; sessions: SessionApi; ui: StudioUiApi; download?(name: string, content: string): void; desktop?: DesktopActions; mcpConnect?: McpConnect; agentClient?: AgentClient | null }

interface TaskListState {
    phase: 'loading' | 'ready' | 'runtime_unavailable' | 'error';
    message: string | null;
}

function browserDownload(name: string, content: string) {
    const url = URL.createObjectURL(new Blob([content], { type: 'text/plain;charset=utf-8' }));
    const link = document.createElement('a'); link.href = url; link.download = name; document.body.appendChild(link); link.click(); link.remove();
    window.setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export function StudioApp({ store, adapter, sources, files, sessions, ui, download = browserDownload, desktop, mcpConnect, agentClient = null }: StudioAppProps) {
    const state = useLifecycle(store);
    const [activeSection, setActiveSection] = useState<StudioSection>('agent');
    const [secondarySection, setSecondarySection] = useState<StudioSection | null>(null);
    const [paletteOpen, setPaletteOpen] = useState(false);
    const [settingsOpen, setSettingsOpen] = useState(false);
    const [inspectorOpen, setInspectorOpen] = useState(false);
    const [error, setError] = useState<{ message: string; surface: 'file' | 'other'; operation: string } | null>(null);
    const [sessionRevision, setSessionRevision] = useState(0);
    const [agentDraft, setAgentDraft] = useState('');
    const [agentPasted, setAgentPasted] = useState('');
    const [taskQuery, setTaskQuery] = useState('');
    const [taskRevision, setTaskRevision] = useState(0);
    const [detailOpen, setDetailOpen] = useState(false);
    const [focusTaskId, setFocusTaskId] = useState<string | null>(null);
    const [taskList, setTaskList] = useState<TaskListState>(() => ({ phase: agentClient === null ? 'ready' : 'loading', message: null }));
    const [editorSettings, setEditorSettings] = useState(() => ui.getSettings());
    useScopedShortcuts({ onPalette: () => setPaletteOpen(true), onSettings: () => setSettingsOpen(true) });
    const agent = useMemo(() => createAgentSessionController(store, sessions), [store, sessions]);
    const desktopActions = useMemo(() => desktop ?? createDesktopActions(), [desktop]);
    useEffect(() => () => agent.dispose(), [agent]);
    useEffect(() => {
        if (agentClient === null) {
            setTaskList({ phase: 'ready', message: null });
            return;
        }
        let active = true;
        setTaskList((current) => ({ ...current, phase: 'loading', message: null }));
        void (async () => {
            try {
                await agentClient.open();
            } catch (failure) {
                if (active) setTaskList({ phase: 'runtime_unavailable', message: failure instanceof Error ? failure.message : String(failure) });
                return;
            }
            try {
                await agentClient.listTasks({ includeArchived: true });
                if (!active) return;
                setTaskList({ phase: 'ready', message: null });
                setTaskRevision((value) => value + 1);
            } catch (failure) {
                if (!active) return;
                if (failure instanceof AgentClientError && failure.code === 'runtime_unavailable') setTaskList({ phase: 'runtime_unavailable', message: null });
                else setTaskList({ phase: 'error', message: failure instanceof Error ? failure.message : String(failure) });
            }
        })();
        return () => { active = false; };
    }, [agentClient]);
    useEffect(() => {
        if (agentClient === null) return;
        let active = true;
        let stream: ReturnType<AgentClient['subscribe']> | null = null;
        void (async () => {
            try {
                await agentClient.open();
            } catch {
                return;
            }
            if (!active) return;
            try {
                stream = agentClient.subscribe();
            } catch {
                return;
            }
            try {
                for await (const _event of stream) {
                    if (!active) return;
                    setTaskRevision((value) => value + 1);
                }
            } catch { /* a closed stream is not a task failure */ }
        })();
        return () => { active = false; stream?.close(); };
    }, [agentClient]);
    const selectedRuntime = useMemo(() => {
        if (agentClient === null) return null;
        const snapshot = agentClient.store.snapshot();
        const task = snapshot.selectedTask;
        if (task === null) return null;
        const activeRun = task.latestRunId === null ? null : task.runs.find((run) => run.runId === task.latestRunId) ?? null;
        const unresolvedRun = task.unresolvedRunId !== null || (task.latestRunId !== null && activeRun === null);
        return { task, activeRun, unresolvedRun };
    }, [agentClient, taskRevision]);
    const selectedTaskId = selectedRuntime?.task.taskId ?? null;
    const tasks = useMemo<TaskSummary[]>(() => {
        if (agentClient === null) return [];
        return agentClient.store.snapshot().tasks.map((task) => ({
            taskId: task.taskId,
            projectId: task.projectId,
            title: task.title,
            repo: task.repo,
            branch: task.branch,
            createdAtMs: task.createdAtMs,
            archived: task.archived,
            pinned: task.pinnedLocal,
            latestRunId: task.latestRunId,
            unresolvedRunId: task.unresolvedRunId,
            latestRunLoaded:
                task.latestRunId === null
                    ? true
                    : task.runs.some((run) => run.runId === task.latestRunId),
            state: task.state,
        }));
    }, [agentClient, taskRevision]);
    const taskReadyZero = taskList.phase === 'ready' && tasks.length === 0;
    useEffect(() => {
        const viewChanged = (event: Event) => {
            const view = (event as CustomEvent<{ view?: string }>).detail?.view;
            if (view === 'chat') setActiveSection('agent'); if (view === 'model') setActiveSection('models');
            if (view === 'workspace' || view === 'editor' || view === 'files') setActiveSection('workspace');
            if (view === 'changes') setActiveSection('changes'); if (view === 'terminal') setActiveSection('terminal'); if (view === 'guide') setActiveSection('guide');
        };
        const filesChanged = (event: Event) => { syncWorkspaceSummary(store, (event as CustomEvent<Parameters<typeof syncWorkspaceSummary>[1]>).detail); };
        document.addEventListener('uor-view-changed', viewChanged); document.addEventListener('uor-files-changed', filesChanged);
        return () => { document.removeEventListener('uor-view-changed', viewChanged); document.removeEventListener('uor-files-changed', filesChanged); };
    }, [store]);
    useEffect(() => {
        if (focusTaskId === null) return;
        const restore = () => {
            const row = document.querySelector<HTMLButtonElement>(`[data-task-id="${focusTaskId}"] .task-select`);
            const group = row?.closest('details') ?? null;
            if (group !== null && !group.open) group.open = true;
            row?.focus();
            if (document.activeElement !== row) {
                group?.querySelector<HTMLElement>('summary')?.focus();
            }
            setFocusTaskId(null);
        };
        const rail = document.querySelector<HTMLElement>('[data-region="task-rail"]');
        if (rail?.hidden === true) {
            document.querySelector<HTMLButtonElement>('[data-action="toggle-projects"]')?.click();
            window.setTimeout(restore, 0);
            return;
        }
        restore();
    }, [focusTaskId, taskRevision]);
    useEffect(() => { if (secondarySection === activeSection) setSecondarySection(null); }, [activeSection, secondarySection]);
    useEffect(() => { if (state.file.phase === 'saved' && error?.surface === 'file' && error.operation === `file:save:${state.file.path}`) setError(null); }, [state.file.phase, state.file.path, error?.surface, error?.operation]);
    const selected = state.selectedSource;
    const activeGeneration = state.session.activeGeneration;
    let privacy: string = activeGeneration?.privacy || 'localWASM';
    if (!activeGeneration && selected?.kind === 'api') {
        if (!selected.endpoint) privacy = 'Invalid API endpoint';
        else { try { privacy = classifyEndpoint(selected.endpoint); } catch { privacy = 'Invalid API endpoint'; } }
    } else if (!activeGeneration && !selected && state.session.lastGeneration) privacy = `Last generation: ${state.session.lastGeneration.privacy}`;
    const status: ShellStatus = {
        source: activeGeneration ? `${activeGeneration.source.kind}: ${activeGeneration.source.id}` : selected ? `${selected.kind}: ${selected.id}` : 'No source selected',
        artifact: selected?.kind === 'artifact' && (state.artifact.phase !== 'ready' || state.residentArtifact?.sourceId !== selected.id) ? 'Selected model; not ready' : `Artifact: ${state.artifact.phase}`,
        privacy, file: `File: ${state.file.phase}`, session: `Session: ${state.session.phase}`,
    };
    const report = (message: unknown, operation = 'other') => setError({ message: String(message), surface: 'other', operation });
    const reportFile = (message: string, operation = 'file:other') => setError({ message, surface: 'file', operation });
    const succeed = (operation: string) => setError((current) => current?.operation === operation ? null : current);
    const operations = useMemo(() => createOperationGate(), []);
    const reportTaskFailure = (failure: unknown) => {
        if (failure instanceof AgentClientError && failure.code === 'runtime_unavailable') setTaskList((current) => ({ ...current, phase: 'runtime_unavailable', message: null }));
        else setTaskList((current) => ({ ...current, phase: 'error', message: failure instanceof Error ? failure.message : String(failure) }));
    };
    const pinTask = (taskId: string, pinned: boolean) => {
        if (agentClient === null) return;
        void agentClient.pinTask({ taskId, pinned }).then(() => { setTaskList({ phase: 'ready', message: null }); setTaskRevision((value) => value + 1); }).catch(reportTaskFailure);
    };
    const archiveTask = (taskId: string, archived: boolean) => {
        if (agentClient === null) return;
        void agentClient.archiveTask({ taskId, archived }).then(() => { setTaskList({ phase: 'ready', message: null }); setTaskRevision((value) => value + 1); }).catch(reportTaskFailure);
    };
    const [creatingTask, setCreatingTask] = useState(false);
    const selectTask = (taskId: string) => {
        if (agentClient === null) return;
        agentClient.store.selectTask(taskId);
        if (window.innerWidth < 1024) {
            const rail = document.querySelector<HTMLElement>('[data-region="task-rail"]');
            if (rail !== null && !rail.hidden) document.querySelector<HTMLButtonElement>('[data-action="toggle-projects"]')?.click();
        }
        setDetailOpen(true);
        setFocusTaskId(null);
        setTaskRevision((value) => value + 1);
    };
    const clearRuntimeForLegacy = () => {
        agentClient?.store.clearSelection();
        setDetailOpen(false);
        setFocusTaskId(null);
        setTaskRevision((value) => value + 1);
    };
    const createRuntimeTask = async (): Promise<TaskSummary> => {
        if (agentClient === null) throw new Error('the browser task runtime is not initialized');
        const task = await agentClient.createTask({ title: 'New task' });
        agentClient.store.selectTask(task.taskId);
        setDetailOpen(false);
        setTaskList({ phase: 'ready', message: null });
        setTaskRevision((value) => value + 1);
        return task;
    };
    const createTask = () => {
        if (agentClient === null || creatingTask) return;
        setCreatingTask(true);
        void createRuntimeTask().catch(reportTaskFailure).finally(() => setCreatingTask(false));
    };
    const createTaskForSend = async (): Promise<string | null> =>
        (await createRuntimeTask()).taskId;
    const startNewRun = () => {
        setActiveSection('agent');
        setDetailOpen(false);
        setTaskRevision((value) => value + 1);
        window.setTimeout(() => {
            document.querySelector<HTMLTextAreaElement>('#studio-typed-root textarea[aria-label="Prompt"]')?.focus();
        }, 0);
    };
    const closeDetail = () => {
        const taskId = agentClient?.store.snapshot().selectedTaskId ?? null;
        agentClient?.store.clearSelection();
        setDetailOpen(false);
        setFocusTaskId(taskId);
        setTaskRevision((value) => value + 1);
    };
    const commands: PaletteCommand[] = [
        { id: 'agent', group: 'Navigation', label: 'Go to Agent', description: 'Chat and context', run: () => setActiveSection('agent') },
        { id: 'workspace', group: 'Navigation', label: 'Go to Workspace', description: 'Files and editor', run: () => setActiveSection('workspace') },
        { id: 'changes', group: 'Navigation', label: 'Go to Changes', description: 'Bounded Git status and diff', run: () => setActiveSection('changes') },
        { id: 'terminal', group: 'Navigation', label: 'Go to Terminal', description: 'Guarded native commands', run: () => setActiveSection('terminal') },
        { id: 'models', group: 'Navigation', label: 'Go to Models', description: 'Artifacts and providers', run: () => setActiveSection('models') },
        { id: 'guide', group: 'Navigation', label: 'Go to Guide', description: 'Workflows and limits', run: () => setActiveSection('guide') },
        { id: 'new-session', group: 'Sessions', label: 'New session', description: 'Start a clean thread', disabled: state.session.phase === 'generating', run: () => { clearRuntimeForLegacy(); if (agent.create()) { succeed('session:new'); setSessionRevision((value) => value + 1); } else report('Could not create a session.', 'session:new'); } },
        { id: 'open-folder', group: 'Workspace', label: 'Open folder', description: 'Use the existing folder picker', run: () => { void operations.run('workspace:open-folder', files.openFolder(), succeed, report); } },
        { id: 'save', group: 'Workspace', label: 'Save file', description: 'Save through the existing file API', shortcut: 'Ctrl+S', disabled: !store.canSave(), run: () => { void saveActiveFile(store, files, reportFile); } },
        { id: 'save-as', group: 'Workspace', label: 'Save as', description: 'Download a copy', disabled: !store.canSave(), run: () => { saveActiveFileCopy(store, files, reportFile, succeed); } },
        { id: 'load-demo', group: 'Models', label: 'Load bundled demo', description: 'Load the synthetic local artifact', run: () => { void operations.run('artifact:load', adapter.loadBundled(), succeed, report); } },
        { id: 'inspector', group: 'Interface', label: 'Toggle inspector', description: 'Show or hide details', run: () => setInspectorOpen((value) => !value) },
        { id: 'settings', group: 'Interface', label: 'Open settings', description: 'Appearance, editor, and data', run: () => setSettingsOpen(true) },
        { id: 'help', group: 'Help', label: 'Open guide', description: 'Read workflows and model limits', run: () => setActiveSection('guide') },
    ];
    const renderSection = (section: StudioSection, secondary = false) => {
        const idPrefix = secondary ? 'secondary-' : '';
        if (section === 'agent') return <AgentComposer key={`${state.session.activeId || 'new'}:${state.session.revision}:${sessionRevision}`} store={store} sessions={sessions} files={files} ui={ui} generation={agent} draft={agentDraft} pasted={agentPasted} onDraftChange={setAgentDraft} onPastedChange={setAgentPasted} agentClient={agentClient} agentTask={selectedRuntime?.task ?? null} activeRun={selectedRuntime?.activeRun ?? null} unresolvedRun={selectedRuntime?.unresolvedRun ?? false} runtimeTasksLoading={taskList.phase === 'loading'} runtimeTasksReady={taskList.phase === 'ready'} runtimeTaskCount={tasks.length} showTaskDetail={detailOpen} onTaskBack={closeDetail} onNewRun={startNewRun} onCreateTask={agentClient !== null ? createTaskForSend : undefined} onOpenSurface={(surface) => setActiveSection(surface === 'changes' ? 'changes' : 'terminal')} copy={(text) => { if (navigator.clipboard) return navigator.clipboard.writeText(text); ui.toast('Clipboard unavailable.', 'error'); }} download={download} />;
        if (section === 'workspace') return <WorkspaceEditor store={store} files={files} operations={operations} onError={reportFile} onSuccess={succeed} settings={editorSettings} agentClient={agentClient} />;
        if (section === 'changes') return <ChangesPanel actions={desktopActions} files={files} idPrefix={idPrefix} agentClient={agentClient} taskId={selectedTaskId} copy={(text) => { if (navigator.clipboard) return navigator.clipboard.writeText(text); ui.toast('Clipboard unavailable.', 'error'); }} />;
        if (section === 'pr') return <PrPanel summary={null} unavailableReason="No local or imported pull request summary is available in this session." />;
        if (section === 'artifacts') return <ArtifactPanel client={agentClient} reference={null} />;
        if (section === 'terminal') return <TerminalPanel actions={desktopActions} files={files} idPrefix={idPrefix} />;
        if (section === 'models') return <><ModelPicker store={store} adapter={adapter} sources={sources} operations={operations} onError={report} onSuccess={succeed} /><McpConnections ui={ui} connectRemote={mcpConnect} /></>;
        if (section === 'browser') return <section class="browser-placeholder" aria-label="Browser preview"><span class="browser-placeholder-icon" aria-hidden="true">↗</span><h1>Browser</h1><p>Browser preview requires a harness service</p><p>This static preview does not launch a browser, import cookies, or give the model access to your existing tabs. Connect the local or hosted harness service to enable an isolated preview.</p></section>;
        return <Guide />;
    };
    return <div data-reduce-motion={editorSettings.reduceMotion ? '1' : '0'}><TaskRailCountContext.Provider value={taskReadyZero}><AppShell
        activeSection={activeSection} onSection={setActiveSection} status={status}
        onPalette={() => setPaletteOpen(true)} onSettings={() => setSettingsOpen(true)} onInspector={() => setInspectorOpen((value) => !value)} inspectorOpen={inspectorOpen}
        onModels={() => setActiveSection('models')} onOpenFolder={async () => { await files.openFolder(); succeed('workspace:open-folder'); return files.treeSummary().rootName || files.workspaceName() || 'selected-folder'; }}
        onSplit={() => setSecondarySection(activeSection === 'agent' ? 'workspace' : 'agent')} onCloseSplit={() => setSecondarySection(null)} secondarySection={secondarySection} secondaryChildren={secondarySection ? renderSection(secondarySection, true) : null}
        sessions={<TaskRail
            tasks={tasks}
            query={taskQuery}
            onQuery={setTaskQuery}
            onSelect={selectTask}
            onPin={pinTask}
            onArchive={archiveTask}
            onCreate={agentClient !== null && taskList.phase === 'ready' ? createTask : undefined}
            createPending={creatingTask}
            selectedTaskId={selectedTaskId}
            runtimeUnavailable={taskList.phase === 'runtime_unavailable'}
            loading={taskList.phase === 'loading'}
            error={taskList.phase === 'error' ? taskList.message : null}
            showEmpty={!taskReadyZero}
            legacySessions={<SessionList store={store} agent={agent} download={download} onError={report} onSuccess={succeed} onChange={(operation) => { succeed(operation); setSessionRevision((value) => value + 1); }} onBeforeActivate={() => clearRuntimeForLegacy()} showHeader={false} showEmpty={false} labelId={LOCAL_SESSIONS_LABEL_ID} />}
            legacySessionsEmpty={state.session.items.length === 0}
            collapseLocalSessions={tasks.length > 0}
        />}
        alert={error && <p role="alert" class="workbench-alert">{error.message}</p>} primary={renderSection(activeSection)}>
        <Inspector store={store} ui={ui} open={inspectorOpen} onClose={() => setInspectorOpen(false)} />
        {paletteOpen && <CommandPalette commands={commands} onClose={() => setPaletteOpen(false)} />}
        {settingsOpen && <Settings ui={ui} generating={state.session.phase === 'generating'} onClose={() => setSettingsOpen(false)} onError={report} onSettingsChange={setEditorSettings} onSuccess={(kind) => succeed(`settings:${kind}`)} onChange={(kind) => { if (kind === 'editor') store.dispatch({ type: 'file/closed' }); else { agent.refresh(); setSessionRevision((value) => value + 1); } }} />}
    </AppShell></TaskRailCountContext.Provider></div>;
}
