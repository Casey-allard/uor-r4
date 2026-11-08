import { useState } from 'preact/hooks';
import type { SessionSummary } from '../adapters/workflow.js';
import type { AgentSessionController } from '../adapters/agentSession.js';
import type { LifecycleStore } from '../state/lifecycle.js';
import { useLifecycle } from '../state/useLifecycle.js';
import { Modal } from './Modal.js';

interface SessionListProps {
    store: LifecycleStore;
    agent: AgentSessionController;
    download(name: string, content: string): void;
    onError(message: string, operation: string): void;
    onChange?(operation: string): void;
    onSuccess?(operation: string): void;
}

export function SessionList({ store, agent, download, onError, onChange, onSuccess }: SessionListProps) {
    const state = useLifecycle(store);
    const [, setRevision] = useState(0);
    const [renaming, setRenaming] = useState<SessionSummary | null>(null);
    const [deleting, setDeleting] = useState<SessionSummary | null>(null);
    const [title, setTitle] = useState('');
    const [error, setError] = useState<{ message: string; operation: string } | null>(null);
    const generating = state.session.phase === 'generating';
    const refresh = () => setRevision((value) => value + 1);
    const succeed = (operation: string, callback?: (operation: string) => void) => {
        setError((current) => current?.operation === operation ? null : current);
        refresh();
        callback?.(operation);
    };
    const activate = (operation: string) => { succeed(operation, onChange); };
    const reportError = (message: string, operation: string) => {
        if (operation.startsWith('session:rename:') || operation.startsWith('session:delete:')) setError({ message, operation });
        onError(message, operation);
    };
    const items = state.session.items;

    function rename(session: SessionSummary) {
        setError(null);
        setRenaming(session);
        setTitle(session.title);
    }

    return (
        <section aria-label="Sessions">
            <h2>Sessions</h2>
            <button type="button" disabled={generating} onClick={() => { if (agent.create()) activate('session:new'); else reportError('Could not create a session.', 'session:new'); }}>New session</button>
            {items.length === 0 && <p>No sessions yet</p>}
            <ul>
                {items.map((session) => (
                    <li data-session-id={session.id} aria-current={session.id === state.session.activeId ? 'true' : undefined}>
                        <strong>{session.title}</strong>
                        <span>{session.messageCount} messages · {session.preview}</span>
                        <button type="button" disabled={generating} onClick={() => { if (agent.resume(session.id)) activate(`session:select:${session.id}`); else reportError('Could not select that session.', `session:select:${session.id}`); }}>{`Select ${session.title}`}</button>
                        <button type="button" disabled={generating} onClick={() => rename(session)}>{`Rename ${session.title}`}</button>
                        <button type="button" disabled={generating} onClick={() => {
                            if (agent.duplicate(session.id)) succeed(`session:duplicate:${session.id}`, onChange);
                            else reportError('Could not duplicate that session.', `session:duplicate:${session.id}`);
                        }}>{`Duplicate ${session.title}`}</button>
                        <button type="button" onClick={() => {
                            const operation = `session:export:${session.id}`;
                            const exported = agent.export(session.id);
                            if (exported) { download(exported.name, exported.content); succeed(operation, onSuccess); }
                            else reportError('Session export is unavailable.', operation);
                        }}>{`Export ${session.title}`}</button>
                        <button type="button" disabled={generating} onClick={() => { setError(null); setDeleting(session); }}>{`Delete ${session.title}`}</button>
                    </li>
                ))}
            </ul>
            {renaming && (
                <Modal title="Rename session" onClose={() => setRenaming(null)}>
                    {error && <p role="alert">{error.message}</p>}
                    <form onSubmit={(event) => {
                        event.preventDefault();
                        if (title.trim() && agent.rename(renaming.id, title.trim())) {
                            setRenaming(null);
                            succeed(`session:rename:${renaming.id}`, onChange);
                        } else {
                            reportError('Could not rename that session.', `session:rename:${renaming.id}`);
                        }
                    }}>
                        <label>Session title<input aria-label="Session title" autofocus maxlength={80} value={title} onInput={(event) => setTitle(event.currentTarget.value)} /></label>
                        <button type="submit">Save title</button>
                    </form>
                </Modal>
            )}
            {deleting && (
                <Modal title="Delete session" onClose={() => setDeleting(null)}>
                    {error && <p role="alert">{error.message}</p>}
                    <p>Delete {deleting.title}? This affects only the selected local session.</p>
                    <button type="button" autofocus onClick={() => setDeleting(null)}>Keep session</button>
                    <button type="button" onClick={() => {
                        if (agent.remove(deleting.id)) {
                            setDeleting(null);
                            activate(`session:delete:${deleting.id}`);
                        } else {
                            reportError(generating ? 'Could not delete that session. Stop the current reply first.' : 'Could not delete that session or verify its result.', `session:delete:${deleting.id}`);
                        }
                    }}>Confirm delete</button>
                </Modal>
            )}
        </section>
    );
}
