import { createContext } from 'preact';
import type { ComponentChildren } from 'preact';
import { useContext } from 'preact/hooks';
import type { StudioSection } from './types.js';
import { Icon } from './ui/Icon.js';
import { ProjectHome } from './ProjectHome.js';

interface ProjectSidebarProps { activeSection: StudioSection; onSection(section: StudioSection): void; onNewProject(): void; projectName: string; environment: string; railOpen: boolean; sessions?: ComponentChildren }

export const TaskRailCountContext = createContext<boolean | null>(null);

const destinations: Array<{ id: StudioSection; label: string; icon: string }> = [
    { id: 'agent', label: 'Agent', icon: 'agent' }, { id: 'workspace', label: 'Workspace', icon: 'workspace' },
    { id: 'changes', label: 'Changes', icon: 'changes' }, { id: 'terminal', label: 'Terminal', icon: 'terminal' },
    { id: 'models', label: 'Models', icon: 'models' }, { id: 'guide', label: 'Guide', icon: 'guide' },
];

export function ProjectSidebar({ activeSection, onSection, onNewProject, projectName, environment, railOpen, sessions }: ProjectSidebarProps) {
    const taskEmpty = useContext(TaskRailCountContext);
    return <aside class="project-sidebar" data-open={railOpen ? 'true' : 'false'} hidden={!railOpen}>
        <div class="project-brand"><span class="project-mark" aria-hidden="true">U</span><div><strong>UOR Harness</strong><span>R4 workbench</span></div><button type="button" data-action="new-project" onClick={onNewProject}><Icon name="plus" /><span>New project</span></button></div>
        <nav id="studio-navigation" aria-label="Studio" data-open={railOpen ? 'true' : 'false'} hidden={!railOpen}><span class="sidebar-label">Workbench</span><ul>{destinations.map((destination) => <li><button type="button" aria-current={activeSection === destination.id ? 'page' : undefined} onClick={() => onSection(destination.id)}><Icon name={destination.icon} /><span>{destination.label}</span></button></li>)}</ul></nav>
        <ProjectHome projectName={projectName} environment={environment} showEmpty={taskEmpty !== false} />
        <div class="project-sessions">{sessions}</div>
    </aside>;
}
