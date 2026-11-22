import { useEffect, useState } from 'preact/hooks';
import type { ComponentChildren } from 'preact';
import type { ShellStatus, StudioSection } from './types.js';
import { ProjectSidebar } from './ProjectSidebar.js';
import { SessionToolbar } from './SessionToolbar.js';
import { PaneDock } from './PaneDock.js';
import { Modal } from './Modal.js';
import { Icon } from './ui/Icon.js';

interface AppShellProps {
    activeSection: StudioSection;
    onSection(section: StudioSection): void;
    status: ShellStatus;
    sessions?: ComponentChildren;
    children: ComponentChildren;
    primary?: ComponentChildren;
    secondarySection?: StudioSection | null;
    secondaryChildren?: ComponentChildren;
    onSplit?(): void;
    onCloseSplit?(): void;
    onPalette?(): void;
    onSettings?(): void;
    onInspector?(): void;
    inspectorOpen?: boolean;
    onModels?(): void;
    onOpenFolder?(): Promise<string>;
    alert?: ComponentChildren;
}

export function AppShell({ activeSection, onSection, status, sessions, children, primary, secondarySection, secondaryChildren, onSplit, onCloseSplit, onPalette, onSettings, onInspector, inspectorOpen = false, onModels, onOpenFolder, alert }: AppShellProps) {
    const environment = 'Local browser';
    const [drawerOpen, setDrawerOpen] = useState(() => window.innerWidth > 800);
    const [taskRailOpen, setTaskRailOpen] = useState(() => window.innerWidth >= 1024);
    const [projectOpen, setProjectOpen] = useState(false);
    const [projectName, setProjectName] = useState('uor-r4');
    const [projectDraft, setProjectDraft] = useState('uor-r4');
    const [projectFolder, setProjectFolder] = useState('');
    const [projectError, setProjectError] = useState('');
    const [choosingFolder, setChoosingFolder] = useState(false);
    useEffect(() => {
        const close = (event: KeyboardEvent) => {
            if (event.key !== 'Escape' || event.defaultPrevented) return;
            if (document.querySelector('dialog[open]')) return;
            const target = event.target;
            if (target instanceof Element && target.closest('.agent-composer')) return;
            setDrawerOpen(false);
            setTaskRailOpen(false);
        };
        document.addEventListener('keydown', close);
        return () => document.removeEventListener('keydown', close);
    }, []);
    const toggleTaskRail = () => {
        const next = !taskRailOpen;
        setTaskRailOpen(next);
        setDrawerOpen(next);
    };
    const openProject = () => { setProjectDraft(projectName); setProjectFolder(''); setProjectError(''); setProjectOpen(true); };
    const chooseFolder = async () => {
        setChoosingFolder(true); setProjectError('');
        try { setProjectFolder(onOpenFolder ? await onOpenFolder() : 'demo-folder'); }
        catch (error) { setProjectError(String(error instanceof Error ? error.message : error)); }
        finally { setChoosingFolder(false); }
    };
    return <div class="harness-shell studio-shell task-workspace">
        <SessionToolbar status={status} projectName={projectName} environment={environment} railOpen={taskRailOpen} onToggleProjects={toggleTaskRail} onNewProject={openProject} onModels={onModels || (() => onSection('models'))} onPalette={onPalette} onSettings={onSettings} onInspector={onInspector} inspectorOpen={inspectorOpen} />
        <div class="harness-body" data-rail-open={taskRailOpen ? 'true' : 'false'}>
            <main id="harness-main" data-region="task-detail" tabindex={-1}>{alert}<PaneDock activeSection={activeSection} onSection={onSection} onSplit={onSplit} onCloseSplit={onCloseSplit} secondarySection={secondarySection} secondaryChildren={secondaryChildren ? <div class="task-work-surface" data-region="work-surface">{secondaryChildren}</div> : secondaryChildren}>{primary ?? children}</PaneDock></main>
            <div data-region="task-rail" hidden={!taskRailOpen}>
                <ProjectSidebar activeSection={activeSection} onSection={onSection} onNewProject={openProject} projectName={projectName} environment={environment} railOpen={taskRailOpen} sessions={sessions} />
            </div>
        </div>
        {children !== primary && children}
        {projectOpen && <Modal title="New project" onClose={() => setProjectOpen(false)}><form class="project-form" onSubmit={(event) => { event.preventDefault(); if (!projectDraft.trim() || !projectFolder) return; setProjectName(projectDraft.trim()); setProjectOpen(false); onSection('workspace'); }}>
            {projectError && <p role="alert">{projectError}</p>}
            <label>Project name<input aria-label="Project name" required value={projectDraft} onInput={(event) => setProjectDraft(event.currentTarget.value)} /></label>
            <label>Project folder<span class="folder-picker"><input aria-label="Project folder" required readOnly value={projectFolder} placeholder="Choose a folder" /><button type="button" disabled={choosingFolder} onClick={() => void chooseFolder()}><Icon name="folder" />{choosingFolder ? 'Choosing…' : 'Choose folder'}</button></span></label>
            <p class="form-hint">The selected folder becomes the project workspace. Project memory and scheduled tasks are added by the harness service.</p>
            <div class="dialog-actions"><button type="button" onClick={() => setProjectOpen(false)}>Cancel</button><button type="submit" class="primary-action" disabled={!projectDraft.trim() || !projectFolder}>Create project</button></div>
        </form></Modal>}
    </div>;
}
