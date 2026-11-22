import { useEffect, useState } from 'preact/hooks';
import type { StudioUiApi } from '../adapters/workflow.js';
import type { LifecycleStore } from '../state/lifecycle.js';
import { useLifecycle } from '../state/useLifecycle.js';

interface InspectorProps {
    store: LifecycleStore;
    ui: StudioUiApi;
    open: boolean;
    onClose(): void;
}

export function Inspector({ store, ui, open, onClose }: InspectorProps) {
    const state = useLifecycle(store);
    const [, setRevision] = useState(0);
    useEffect(() => {
        const changed = () => setRevision((value) => value + 1);
        document.addEventListener('uor-attachments-changed', changed);
        return () => document.removeEventListener('uor-attachments-changed', changed);
    }, []);
    if (!open) return null;
    const generation = state.session.activeGeneration || state.session.lastGeneration;
    return (
        <aside id="studio-inspector" aria-label="Workspace inspector">
            <button type="button" onClick={onClose}>Close inspector</button>
            <h2>Inspector</h2>
            <dl>
                <dt>Source</dt><dd>{state.selectedSource ? `${state.selectedSource.kind}: ${state.selectedSource.id}` : 'No source selected'}</dd>
                <dt>Artifact</dt><dd>{state.residentArtifact?.fileName || state.artifact.phase}</dd>
                <dt>File</dt><dd>{state.file.path || 'No file open'}</dd>
                {generation && <><dt>{state.session.activeGeneration ? 'Generation' : 'Last generation'}</dt><dd>{generation.source.id} · {generation.privacy}</dd></>}
                <dt>Attachments</dt><dd>{ui.getAttachments().length ? ui.getAttachments().map((item) => item.name).join(', ') : 'No context attached'}</dd>
            </dl>
        </aside>
    );
}
