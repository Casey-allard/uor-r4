import { useState } from 'preact/hooks';
import type { ComponentChildren } from 'preact';
import type { StudioSection } from './types.js';
import { Icon } from './ui/Icon.js';
import { Splitter } from './ui/Splitter.js';

interface PaneDockProps {
    activeSection: StudioSection;
    onSection(section: StudioSection): void;
    onSplit?(): void;
    onCloseSplit?(): void;
    children: ComponentChildren;
    secondarySection?: StudioSection | null;
    secondaryChildren?: ComponentChildren;
}

const targets: Array<{ id: StudioSection | 'browser'; pane: string; label: string; icon: string }> = [
    { id: 'agent', pane: 'chat', label: 'Chat', icon: 'agent' },
    { id: 'workspace', pane: 'editor', label: 'Editor', icon: 'workspace' },
    { id: 'changes', pane: 'diff', label: 'Diff', icon: 'changes' },
    { id: 'terminal', pane: 'terminal', label: 'Terminal', icon: 'terminal' },
    { id: 'browser', pane: 'browser', label: 'Browser', icon: 'browser' },
];

export function paneId(section: StudioSection): string {
    if (section === 'agent') return 'chat';
    if (section === 'workspace') return 'editor';
    if (section === 'changes') return 'diff';
    return section;
}

export function PaneDock({ activeSection, onSection, onSplit, onCloseSplit, children, secondarySection, secondaryChildren }: PaneDockProps) {
    const [primaryWidth, setPrimaryWidth] = useState(50);
    const split = Boolean(secondarySection);
    return <div class="workspace-grid" data-split={split ? 'true' : 'false'}>
        <div class="pane-dock" role="toolbar" aria-label="Workspace panes">
            <div class="pane-tabs">{targets.map((target) => <button type="button" data-pane-target={target.pane} aria-pressed={activeSection === target.id ? 'true' : 'false'} onClick={() => onSection(target.id)}><Icon name={target.icon} /><span>{target.label}</span></button>)}</div>
            <div class="pane-actions">{split ? <button type="button" data-action="close-split" aria-label="Close split pane" onClick={onCloseSplit}><Icon name="close" /></button> : <button type="button" data-action="split-pane" aria-label="Split pane" onClick={onSplit}><Icon name="split" /><span>Split</span></button>}</div>
        </div>
        <div class="pane-columns">
            <section class="pane-primary" data-pane={paneId(activeSection)} style={split ? { flexBasis: `${primaryWidth}%` } : undefined}>{children}</section>
            {split && secondarySection && <><Splitter value={primaryWidth} onChange={setPrimaryWidth} /><section class="pane-secondary" data-pane={paneId(secondarySection)}>{secondaryChildren}</section></>}
        </div>
    </div>;
}
