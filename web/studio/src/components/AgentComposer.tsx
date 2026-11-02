import { useEffect, useState } from 'preact/hooks';
import { GENERATION_FINISHED_EVENT } from '../adapters/generation.js';
import type { GenerationAdapter } from '../adapters/generation.js';
import type { SessionApi, StudioUiApi, WorkspaceApi } from '../adapters/workflow.js';
import type { LifecycleStore } from '../state/lifecycle.js';
import { useLifecycle } from '../state/useLifecycle.js';

interface AgentComposerProps {
    store: LifecycleStore;
    sessions: SessionApi;
    files: WorkspaceApi;
    ui: StudioUiApi;
    generation: GenerationAdapter;
    copy(text: string): void | Promise<void>;
    download(name: string, content: string): void;
    draft?: string;
    pasted?: string;
    onDraftChange?(value: string): void;
    onPastedChange?(value: string): void;
}

let insertSequence = 0;

function nextInsertName(): string {
    insertSequence += 1;
    return `agent-message-${Date.now()}-${insertSequence}.md`;
}

const templates = [
    ['ux', 'Studio UX improvement brief'],
    ['api', 'API integration checklist'],
    ['audit', 'Audit this file'],
    ['report', 'Write a milestone REPORT'],
    ['error', 'Explain this error'],
    ['test', 'Write a verification plan'],
] as const;

export function AgentComposer({ store, sessions, files, ui, generation, copy, download, draft: controlledDraft, pasted: controlledPasted, onDraftChange, onPastedChange }: AgentComposerProps) {
    const state = useLifecycle(store);
    const [draftState, setDraftState] = useState('');
    const [pastedState, setPastedState] = useState('');
    const draft = controlledDraft ?? draftState;
    const pasted = controlledPasted ?? pastedState;
    const setDraft = (value: string) => { setDraftState(value); onDraftChange?.(value); };
    const setPasted = (value: string) => { setPastedState(value); onPastedChange?.(value); };
    const [, setRevision] = useState(0);
    const refresh = () => setRevision((value) => value + 1);

    useEffect(() => {
        const changed = () => refresh();
        document.addEventListener(GENERATION_FINISHED_EVENT, changed);
        document.addEventListener('uor-attachments-changed', changed);
        return () => {
            document.removeEventListener(GENERATION_FINISHED_EVENT, changed);
            document.removeEventListener('uor-attachments-changed', changed);
        };
    }, []);

    const selected = state.selectedSource;
    const activeGeneration = state.session.activeGeneration;
    const residentMatches = selected?.kind === 'artifact'
        && state.artifact.phase === 'ready'
        && state.residentArtifact?.sourceId === selected.id;
    const prediction = !activeGeneration && residentMatches
        && state.residentArtifact?.canGenerate === false
        && state.residentArtifact?.canPredict === true;
    const generating = state.session.phase === 'generating';
    const messages = sessions.getConversationMessages();
    const attachments = ui.getAttachments();
    const activeFile = files.activeFile();
    const unavailable = !activeGeneration && residentMatches
        && (state.residentArtifact?.engineSupported === false
            || (state.residentArtifact?.canGenerate === false && state.residentArtifact?.canPredict !== true));
    const predictionMode = activeGeneration ? activeGeneration.canStop === false : prediction && !unavailable;
    const selectionLabel = activeGeneration
        ? (predictionMode ? 'Local next-token prediction' : activeGeneration.source.kind === 'artifact' ? 'Local generation' : 'API chat')
        : !selected ? 'No source selected'
        : selected.kind === 'artifact' && !residentMatches ? 'Selected model; not ready'
        : selected.kind === 'api' && !store.canRun() ? 'Invalid API endpoint'
        : unavailable ? 'Generation unavailable'
        : predictionMode ? 'Local next-token prediction'
        : selected.kind === 'artifact' ? 'Local generation' : 'API chat';

    function send(prompt: string) {
        if (generation.send(prompt)) {
            setDraft('');
            refresh();
        }
    }

    return (
        <section aria-label="Agent composer">
            <h1>Agent</h1>
            <p>{selectionLabel}</p>
            <p>Session: {state.session.phase}</p>
            {state.session.error && <p role="alert">{state.session.error}</p>}
            {activeGeneration && <p>Generating with {activeGeneration.source.id} ({activeGeneration.privacy})</p>}

            <div aria-label="Conversation messages">
                {messages.length === 0 && <p>No messages yet</p>}
                {messages.map((message, index) => (
                    <article data-role={message.role}>
                        <strong>{message.role}</strong>
                        <p>{message.content}</p>
                        <button type="button" onClick={() => void copy(message.content)}>Copy</button>
                        <button type="button" onClick={() => files.insertText(message.content, nextInsertName())}>Insert into editor</button>
                        <button type="button" onClick={() => download('studio-message.md', message.content)}>Save as file</button>
                        {message.role === 'assistant' && (
                            <button type="button" disabled={!store.canRun()} onClick={() => {
                                const previous = messages.slice(0, index + 1).reverse().find((item) => item.role === 'user');
                                if (previous) generation.send(previous.content);
                            }}>Retry</button>
                        )}
                    </article>
                ))}
            </div>

            <section aria-label="Context attachments">
                <h2>Context</h2>
                <button type="button" disabled={!activeFile} onClick={() => {
                    const file = files.activeFile();
                    if (file) {
                        ui.addAttachment({ name: file.path, content: file.content, source: 'editor' });
                        refresh();
                    }
                }}>Attach current file</button>
                <label>Pasted context<textarea aria-label="Pasted context" value={pasted} onInput={(event) => setPasted(event.currentTarget.value)} /></label>
                <button type="button" disabled={!pasted.trim()} onClick={() => {
                    ui.addAttachment({ name: 'pasted-context.txt', content: pasted, source: 'paste' });
                    setPasted('');
                    refresh();
                }}>Attach pasted text</button>
                <button type="button" disabled={!attachments.length} onClick={() => { ui.clearAttachments(); refresh(); }}>Clear attachments</button>
                <ul>
                    {attachments.map((attachment) => (
                        <li>{attachment.name}<button type="button" onClick={() => { ui.removeAttachment(attachment.name); refresh(); }}>{`Remove ${attachment.name}`}</button></li>
                    ))}
                </ul>
            </section>

            <section aria-label="Build task templates">
                <h2>Build tasks</h2>
                {templates.map(([kind, label]) => (
                    <button type="button" data-template={kind} onClick={() => setDraft(ui.templatePrompt(kind))}>{label}</button>
                ))}
            </section>

            <label>Prompt<textarea aria-label="Prompt" value={draft} onInput={(event) => setDraft(event.currentTarget.value)} /></label>
            <button type="button" disabled={!draft.trim() || !store.canRun()} onClick={() => send(draft)}>{predictionMode ? 'Predict next tokens' : 'Send'}</button>
            <button type="button" disabled={!generating || activeGeneration?.canStop === false} onClick={() => { generation.stop(); refresh(); }}>Stop</button>
        </section>
    );
}
