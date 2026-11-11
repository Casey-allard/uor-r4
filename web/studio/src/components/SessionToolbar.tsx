import type { ShellStatus } from './types.js';
import { Icon } from './ui/Icon.js';

interface SessionToolbarProps { status: ShellStatus; projectName: string; environment: string; railOpen: boolean; onToggleProjects(): void; onNewProject(): void; onModels(): void; onPalette?(): void; onSettings?(): void; onInspector?(): void; inspectorOpen?: boolean }

export function SessionToolbar({ status, projectName, environment, railOpen, onToggleProjects, onNewProject, onModels, onPalette, onSettings, onInspector, inspectorOpen = false }: SessionToolbarProps) {
    return <header class="session-toolbar">
        <div class="session-toolbar-main">
            <button type="button" class="toolbar-icon" data-action="toggle-projects" aria-label="Toggle projects" aria-controls="studio-navigation" aria-expanded={railOpen} onClick={onToggleProjects}><Icon name="workspace" /></button>
            <span class="session-wordmark">UOR <span>Harness</span></span>
            <label class="toolbar-selector"><span>Environment</span><select data-control="environment" aria-label="Environment"><option>{environment}</option><option disabled>Harness service (not connected)</option></select></label>
            <button type="button" class="toolbar-selector" data-control="project" onClick={onNewProject}><span>Project</span><strong>{projectName}</strong><Icon name="chevron" /></button>
            <label class="toolbar-selector"><span>Permission</span><select data-control="permission" aria-label="Permission"><option>Ask before write</option><option disabled>Autonomous (requires service)</option></select></label>
            <button type="button" class="toolbar-selector" data-control="model" onClick={onModels}><span>Model</span><strong>{status.source === 'No source selected' ? 'Choose model' : status.source}</strong><Icon name="chevron" /></button>
        </div>
        <div class="session-toolbar-actions">
            <div role="status" aria-live="polite" class="session-status"><span>{status.source}</span><span>{status.artifact}</span><span>{status.privacy}</span><span>{status.file}</span><span>{status.session}</span></div>
            {onPalette && <button type="button" class="toolbar-command" data-shortcut="⌘⇧P" aria-keyshortcuts="Meta+Shift+P Control+Shift+P" onClick={onPalette}><Icon name="command" /><span>Command palette</span></button>}
            {onSettings && <button type="button" class="toolbar-icon" aria-label="Settings" onClick={onSettings}><Icon name="settings" /><span class="visually-hidden">Settings</span></button>}
            {onInspector && <button type="button" class="toolbar-icon" aria-label="Inspector" aria-controls="studio-inspector" aria-expanded={inspectorOpen} onClick={onInspector}><Icon name="inspector" /><span class="visually-hidden">Inspector</span></button>}
        </div>
    </header>;
}
