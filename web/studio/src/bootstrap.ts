import { h, render } from 'preact';
import { LifecycleStore } from './state/lifecycle.js';
import { createModelSourceAdapter } from './adapters/modelSourceAdapter.js';
import { createLegacyWorkflowApis } from './adapters/workflow.js';
import type { LegacyWorkflowApis } from './adapters/workflow.js';
import { ModelSources } from '../assets/js/model_sources.js';
import { StudioApp } from './components/StudioApp.js';
import { studioMigrationStatus } from './persistence/migration.js';

interface LegacyAppWindow extends Window {
    __studioApp?: { worker: Worker | null };
}

export function typedPreviewRequested(search: string): boolean {
    return new URLSearchParams(search).get('typed') === '1';
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
    const migration = studioMigrationStatus();
    if (migration.state === 'error') throw new Error(`Browser-state migration failed: ${migration.error || 'unknown error'}`);
    root.hidden = false;
    const apis = await waitForWorkflowApis(options.timeoutMs ?? 5000);
    const requestedTheme = new URLSearchParams(window.location.search).get('theme');
    if (requestedTheme === 'light' || requestedTheme === 'dark') apis.ui.updateSettings({ theme: requestedTheme });
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
    }), root);
    root.dataset.active = 'true';
    return {
        store,
        worker,
        adapter,
        files: apis.files,
        sessions: apis.sessions,
        ui: apis.ui,
        dispose() {
            worker.removeEventListener('message', forwardMessage);
            if (!legacyWorker) worker.terminate();
            root.dataset.active = 'false';
            root.hidden = true;
            render(null, root);
        },
    };
}
