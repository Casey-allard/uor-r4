import type { ComponentChildren } from 'preact';
import type { StudioSection } from './types.js';
import { Icon } from './ui/Icon.js';

interface ProjectSidebarProps { activeSection: StudioSection; onSection(section: StudioSection): void; onNewProject(): void; drawerOpen: boolean; sessions?: ComponentChildren }

const destinations: Array<{ id: StudioSection; label: string; icon: string }> = [
    { id: 'agent', label: 'Agent', icon: 'agent' }, { id: 'workspace', label: 'Workspace', icon: 'workspace' },
    { id: 'changes', label: 'Changes', icon: 'changes' }, { id: 'terminal', label: 'Terminal', icon: 'terminal' },
    { id: 'models', label: 'Models', icon: 'models' }, { id: 'guide', label: 'Guide', icon: 'guide' },
];

export function ProjectSidebar({ activeSection, onSection, onNewProject, drawerOpen, sessions }: ProjectSidebarProps) {
    return <aside class="project-sidebar" data-open={drawerOpen ? 'true' : 'false'} hidden={!drawerOpen}>
        <div class="project-brand"><span class="project-mark" aria-hidden="true">U</span><div><strong>UOR Harness</strong><span>R4 workbench</span></div><button type="button" data-action="new-project" onClick={onNewProject}><Icon name="plus" /><span>New project</span></button></div>
        <nav id="studio-navigation" aria-label="Studio" data-open={drawerOpen ? 'true' : 'false'} hidden={!drawerOpen}><span class="sidebar-label">Workbench</span><ul>{destinations.map((destination) => <li><button type="button" aria-current={activeSection === destination.id ? 'page' : undefined} onClick={() => onSection(destination.id)}><Icon name={destination.icon} /><span>{destination.label}</span></button></li>)}</ul></nav>
        <div class="project-sessions">{sessions}</div>
    </aside>;
}
