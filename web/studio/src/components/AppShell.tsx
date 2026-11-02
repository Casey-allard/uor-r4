import { useState } from 'preact/hooks';
import type { ComponentChildren } from 'preact';
import type { ShellStatus, StudioSection } from './types.js';

interface AppShellProps {
    activeSection: StudioSection;
    onSection(section: StudioSection): void;
    status: ShellStatus;
    sessions?: ComponentChildren;
    children: ComponentChildren;
    onPalette?(): void;
    onSettings?(): void;
    onInspector?(): void;
    inspectorOpen?: boolean;
}

const destinations: Array<{ id: StudioSection; label: string }> = [
    { id: 'agent', label: 'Agent' },
    { id: 'workspace', label: 'Workspace' },
    { id: 'models', label: 'Models' },
    { id: 'guide', label: 'Guide' },
];

export function AppShell({ activeSection, onSection, status, sessions, children, onPalette, onSettings, onInspector, inspectorOpen = false }: AppShellProps) {
    const [drawerOpen, setDrawerOpen] = useState(() => window.innerWidth > 800);
    return (
        <div class="studio-shell">
            <header class="studio-toolbar">
                <button type="button" aria-label="Toggle navigation" aria-controls="studio-navigation" aria-expanded={drawerOpen} onClick={() => setDrawerOpen(!drawerOpen)}>Menu</button>
                <strong>UOR-R4 Studio</strong>
                {onPalette && <button type="button" onClick={onPalette}>Command palette</button>}
                {onSettings && <button type="button" onClick={onSettings}>Settings</button>}
                {onInspector && <button type="button" aria-controls="studio-inspector" aria-expanded={inspectorOpen} onClick={onInspector}>Inspector</button>}
                <div role="status" aria-live="polite" class="studio-status">
                    <span>{status.source}</span><span>{status.artifact}</span><span>{status.privacy}</span><span>{status.file}</span><span>{status.session}</span>
                </div>
            </header>
            <div class="studio-body">
                <nav id="studio-navigation" aria-label="Studio" data-open={drawerOpen} hidden={!drawerOpen}>
                    <ul>
                        {destinations.map((destination) => (
                            <li><button type="button" aria-current={activeSection === destination.id ? 'page' : undefined} onClick={() => onSection(destination.id)}>{destination.label}</button></li>
                        ))}
                    </ul>
                    {sessions}
                </nav>
                <main>{children}</main>
            </div>
        </div>
    );
}
