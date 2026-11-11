import { h, render } from 'preact';
import { LifecycleStore } from './state/lifecycle.js';
import { createModelSourceAdapter } from './adapters/modelSourceAdapter.js';
import { createLegacyWorkflowApis } from './adapters/workflow.js';
import type { LegacyWorkflowApis } from './adapters/workflow.js';
import { ModelSources } from '../assets/js/model_sources.js';
import { StudioApp } from './components/StudioApp.js';
import { activateStudioMigration, studioMigrationStatus } from './persistence/migration.js';
import { installLegacyShortcutGuard } from './agent/legacyShortcutGuard.js';
import { AgentClient } from './agent/client.js';
import { HostClient } from './agent/hostClient.js';
import { AgentRuntime, type RuntimeToolExecutor } from './agent/runtime.js';
import { AgentTaskStore } from './agent/store.js';
import { ReplayModelPort, type ModelPort } from './agent/modelPort.js';
import {
    createTauriHostBridge,
    StaticHostTransport,
    TauriHostTransport,
    type TauriCoreLike,
} from './agent/transports.js';
import { createAgentStore, type AgentStore } from './state/agent.js';

interface LegacyAppWindow extends Window {
    __studioApp?: { worker: Worker | null };
}

function retargetSkipLink(href: string): string | null {
    const link = document.querySelector<HTMLAnchorElement>('a.skip-link');
    if (!link) return null;
    const previous = link.getAttribute('href');
    link.setAttribute('href', href);
    return previous;
}

export function typedPreviewRequested(search: string): boolean {
    const params = new URLSearchParams(search);
    return params.get('legacy') !== '1' && params.get('typed') !== '0';
}

export type StudioHostKind = 'tauri' | 'static' | 'injected' | 'none';

export interface StudioAgentOwner {
    store: AgentStore;
    taskStore: AgentTaskStore | null;
    runtime: AgentRuntime | null;
    client: AgentClient;
    modelPort: ModelPort;
    hostKind: StudioHostKind;
    error: { code: string; message: string } | null;
}

export interface StudioAgentOwnerOptions {
    indexedDB?: IDBFactory;
    modelPort?: ModelPort;
    now?: () => number;
    toolExecutor?: RuntimeToolExecutor;
    hostClient?: HostClient | null;
    tauriCore?: TauriCoreLike | null;
}

/**
 * The production host boundary: a real Tauri companion supplies the only host
 * path; any other page composes the explicit static transport. Callers never
 * construct a second host client or a page-side approval authority.
 */
export function createStudioHostClient(
    options: { tauriCore?: TauriCoreLike | null } = {},
): { client: HostClient; kind: 'tauri' | 'static' } {
    const bridge = createTauriHostBridge(options.tauriCore);
    if (bridge !== null) {
        return {
            client: new HostClient({ transport: new TauriHostTransport({ bridge }) }),
            kind: 'tauri',
        };
    }
    return {
        client: new HostClient({ transport: new StaticHostTransport() }),
        kind: 'static',
    };
}

// One production browser owner: the runtime owns the reducer/task adapter and
// the client is the only UI-facing path. A missing or unopenable IndexedDB
// keeps the legacy shell usable through an explicit runtime_unavailable client
// instead of constructing a second store or writer.
export function createStudioAgentOwner(options: StudioAgentOwnerOptions = {}): StudioAgentOwner {
    const store = createAgentStore();
    // A deterministic synthetic replay stream with visible pacing so a local
    // run can be stopped honestly before it finishes. Still not an R4 model.
    const modelPort = options.modelPort ?? new ReplayModelPort({ paceMs: 500 });
    const host =
        options.hostClient !== undefined
            ? options.hostClient === null
                ? null
                : { client: options.hostClient, kind: 'injected' as const }
            : options.toolExecutor !== undefined
                ? null
                : createStudioHostClient({ tauriCore: options.tauriCore });
    const hostKind: StudioHostKind = host === null ? 'none' : host.kind;
    try {
        const taskStore = new AgentTaskStore({
            store,
            ...(options.indexedDB === undefined ? {} : { indexedDB: options.indexedDB }),
            ...(options.now === undefined ? {} : { now: options.now }),
        });
        const runtime = new AgentRuntime({
            store,
            taskStore,
            modelPort,
            ownsTaskStore: false,
            ...(options.now === undefined ? {} : { now: options.now }),
            ...(options.toolExecutor === undefined ? {} : { toolExecutor: options.toolExecutor }),
            ...(host === null ? {} : { hostClient: host.client }),
        });
        return { store, taskStore, runtime, client: new AgentClient({ runtime }), modelPort, hostKind, error: null };
    } catch (error) {
        const code = error !== null && typeof error === 'object' && 'code' in error && typeof (error as { code?: unknown }).code === 'string'
            ? (error as { code: string }).code
            : 'runtime_unavailable';
        return {
            store,
            taskStore: null,
            runtime: null,
            client: new AgentClient({ store }),
            modelPort,
            hostKind: 'none',
            error: { code, message: error instanceof Error ? error.message : String(error) },
        };
    }
}

export function studioTestApiEnabled(
    buildFlag: string | undefined,
    search: string,
): boolean {
    return (
        buildFlag === '1' &&
        new URLSearchParams(search).get('studioTest') === '1'
    );
}

async function waitForWorkflowApis(timeoutMs: number): Promise<LegacyWorkflowApis> {
    const deadline = Date.now() + timeoutMs;
    while (Date.now() < deadline) {
        const apis = createLegacyWorkflowApis();
        if (
            apis.files
            && apis.sessions
            && typeof apis.files.tabs === 'function'
            && typeof apis.files.activeFile === 'function'
            && typeof apis.ui.getSettings === 'function'
            && typeof apis.ui.getAttachments === 'function'
            && typeof apis.sessions.listSessions === 'function'
            && typeof apis.sessions.activeSessionId === 'function'
            && typeof apis.sessions.getConversationMessages === 'function'
        ) return apis;
        await new Promise((resolve) => window.setTimeout(resolve, 25));
    }
    throw new Error('The existing Studio workflow services did not initialize.');
}

export async function mountTypedStudio(root: HTMLElement, options: { timeoutMs?: number } = {}) {
    if (studioMigrationStatus().state === 'inactive') activateStudioMigration(true);
    const migration = studioMigrationStatus();
    if (migration.state === 'error') throw new Error(`Browser-state migration failed: ${migration.error || 'unknown error'}`);
    root.hidden = false;
    const shortcutGuard = installLegacyShortcutGuard(root);
    let apis: LegacyWorkflowApis;
    try {
        apis = await waitForWorkflowApis(options.timeoutMs ?? 5000);
    } catch (error) {
        shortcutGuard.dispose();
        throw error;
    }
    const requestedTheme = new URLSearchParams(window.location.search).get('theme')
        || (window.location.hash === '#theme=light' ? 'light' : (window.location.hash === '#theme=dark' ? 'dark' : null));
    if (requestedTheme === 'light' || requestedTheme === 'dark') apis.ui.updateSettings({ theme: requestedTheme });
    const buildEnablesTestApi = import.meta.env.VITE_STUDIO_TEST === '1';
    const gatedTestApi = buildEnablesTestApi
        && studioTestApiEnabled(import.meta.env.VITE_STUDIO_TEST, window.location.search);
    const testApi = gatedTestApi ? await import('./agent/testApi.js') : null;
    const owner = createStudioAgentOwner({
        ...(testApi === null ? {} : {
            modelPort: testApi.createGatedTestModelPort(),
            now: testApi.studioTestNow,
            toolExecutor: testApi.createGatedTestToolExecutor(),
        }),
    });
    root.dataset.host = owner.hostKind;
    try {
        await owner.client.open();
    } catch {
        // The client stays explicit runtime_unavailable; the legacy shell and
        // composer must still mount.
    }
    const store = new LifecycleStore();
    const legacyWorker = (window as LegacyAppWindow).__studioApp?.worker || null;
    const worker = legacyWorker || new Worker(new URL('../assets/js/uor_model_worker.js', import.meta.url), { type: 'module' });
    const adapter = createModelSourceAdapter(store, ModelSources, worker);
    const forwardMessage = (event: MessageEvent) => adapter.handleWorkerMessage(event.data);
    worker.addEventListener('message', forwardMessage);
    render(h(StudioApp, {
        store,
        adapter,
        sources: ModelSources,
        files: apis.files,
        sessions: apis.sessions,
        ui: apis.ui,
        agentClient: owner.client,
    }), root);
    if (testApi !== null) testApi.installStudioTestApi(owner);
    const previousSkipHref = retargetSkipLink('#harness-main');
    root.dataset.active = 'true';
    return {
        store,
        worker,
        adapter,
        files: apis.files,
        sessions: apis.sessions,
        ui: apis.ui,
        agentClient: owner.client,
        agentStore: owner.store,
        agentTaskStore: owner.taskStore,
        agentRuntime: owner.runtime,
        dispose() {
            shortcutGuard.dispose();
            void owner.client.dispose().finally(() => owner.taskStore?.close());
            worker.removeEventListener('message', forwardMessage);
            if (!legacyWorker) worker.terminate();
            root.dataset.active = 'false';
            delete root.dataset.host;
            root.hidden = true;
            render(null, root);
            if (previousSkipHref !== null) retargetSkipLink(previousSkipHref);
        },
    };
}
