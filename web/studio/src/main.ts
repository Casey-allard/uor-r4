export * from './state/lifecycle.js';
export * from './adapters/modelSourceAdapter.js';
export * from './persistence/migration.js';
export * from './adapters/workflow.js';
export * from './adapters/generation.js';
export * from './adapters/agentSession.js';
export * from './adapters/desktopActions.js';
export * from './adapters/mcpConnections.js';
export * from './adapters/composerInput.js';
export * from './adapters/fileSave.js';
export * from './state/useLifecycle.js';
export * from './components/Modal.js';
export * from './components/AppShell.js';
export * from './components/ProjectSidebar.js';
export * from './components/SessionToolbar.js';
export * from './components/PaneDock.js';
export * from './components/AgentTranscript.js';
export * from './components/ui/Icon.js';
export * from './components/ui/Splitter.js';
export * from './components/CommandPalette.js';
export * from './components/ModelPicker.js';
export * from './components/WorkspaceEditor.js';
export * from './components/AgentComposer.js';
export * from './components/DesktopTools.js';
export * from './components/McpConnections.js';
export * from './components/RichMessage.js';
export * from './components/SessionList.js';
export * from './components/Inspector.js';
export * from './components/Guide.js';
export * from './components/Settings.js';
export * from './components/StudioApp.js';
export * from './components/types.js';
import './studio.css';
import { mountTypedStudio, typedPreviewRequested } from './bootstrap.js';

export const foundation = {};

export function showMountError(root: HTMLElement, error: unknown): void {
    root.hidden = false;
    root.dataset.active = 'true';
    root.textContent = `The typed Studio could not start: ${String(error)}`;
}

if (typedPreviewRequested(window.location.search)) {
    const root = document.getElementById('studio-typed-root');
    if (root) {
        await mountTypedStudio(root).catch((error) => showMountError(root, error));
    }
}
