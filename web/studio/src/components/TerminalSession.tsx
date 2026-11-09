import { useEffect, useRef, useState } from 'preact/hooks';
import type { Terminal as XTerm } from '@xterm/xterm';
import '@xterm/xterm/css/xterm.css';
import type { DesktopActions, TerminalSnapshot } from '../adapters/desktopActions.js';
import type { WorkspaceApi } from '../adapters/workflow.js';
import { useWorkspacePath } from './DesktopTools.js';
import { Icon } from './ui/Icon.js';

interface DesktopPanelProps { actions: DesktopActions; files: WorkspaceApi; idPrefix?: string }
type TerminalLike = Pick<XTerm, 'open' | 'write' | 'onData' | 'onResize' | 'loadAddon' | 'focus' | 'dispose' | 'options'>;
interface TerminalRuntime {
    snapshot: TerminalSnapshot | null;
    sessionRevision: number;
    terminal: TerminalLike | null;
    observer: ResizeObserver | null;
    poller: number | null;
    cursor: number;
    readingFor: string | null;
    disposed: boolean;
    closeRequested: boolean;
    truncationNotified: boolean;
}

function bytesFromHex(value: string): Uint8Array {
    const bytes = new Uint8Array(value.length / 2);
    for (let index = 0; index < value.length; index += 2) bytes[index / 2] = Number.parseInt(value.slice(index, index + 2), 16);
    return bytes;
}

export function TerminalPanel({ actions, files, idPrefix = '' }: DesktopPanelProps) {
    const { path, revision, updatePath } = useWorkspacePath(files);
    const containerRef = useRef<HTMLDivElement>(null);
    const runtime = useRef<TerminalRuntime>({ snapshot: null, sessionRevision: revision, terminal: null, observer: null, poller: null, cursor: 0, readingFor: null, disposed: false, closeRequested: false, truncationNotified: false });
    const mounted = useRef(true);
    const workspaceRevision = useRef(revision);
    const [session, setSession] = useState<TerminalSnapshot | null>(null);
    const [failedSession, setFailedSession] = useState<TerminalSnapshot | null>(null);
    const [opening, setOpening] = useState(false);
    const [closing, setClosing] = useState(false);
    const [consent, setConsent] = useState('');
    const id = (name: string) => `${idPrefix}${name}`;

    function closeSession(restoreOnFailure = true) {
        const current = runtime.current;
        if (current.poller !== null) window.clearInterval(current.poller);
        current.poller = null;
        current.observer?.disconnect();
        current.observer = null;
        current.terminal?.dispose();
        current.terminal = null;
        current.readingFor = null;
        const previous = current.snapshot;
        if (!previous) {
            current.disposed = !mounted.current;
            if (mounted.current) {
                setSession(null);
                setFailedSession(null);
            }
            return;
        }
        current.disposed = true;
        if (current.closeRequested) return;
        current.closeRequested = true;
        if (mounted.current) {
            setClosing(true);
            setSession(null);
            setFailedSession(null);
        }
        const sessionId = previous.sessionId;
        void actions.closeTerminal(sessionId).catch(async error => {
            if (mounted.current) throw error;
            try {
                await actions.closeTerminal(sessionId);
            } catch (retryError) {
                console.error(`Could not close PTY after unmount: ${String(retryError instanceof Error ? retryError.message : retryError)}`);
            }
        }).then(() => {
            if (current.snapshot?.sessionId !== sessionId) return;
            current.snapshot = null;
            current.disposed = true;
            current.closeRequested = false;
            if (mounted.current) {
                setSession(null);
                setFailedSession(null);
            }
        }).catch((error) => {
            if (current.snapshot?.sessionId !== sessionId) return;
            current.closeRequested = false;
            if (mounted.current) {
                const restore = restoreOnFailure && current.sessionRevision === workspaceRevision.current;
                setSession(restore ? previous : null);
                setFailedSession(restore ? null : previous);
                setConsent(`Could not close the PTY: ${String(error instanceof Error ? error.message : error)}`);
            }
        }).finally(() => {
            if (mounted.current) setClosing(false);
        });
    }

    useEffect(() => {
        const changed = () => {
            workspaceRevision.current += 1;
            closeSession();
        };
        document.addEventListener('uor-files-changed', changed);
        return () => document.removeEventListener('uor-files-changed', changed);
    }, []);

    useEffect(() => {
        if (revision > 0) setConsent('Workspace changed. Open a shell in the new project folder.');
    }, [revision]);

    useEffect(() => () => {
        mounted.current = false;
        closeSession();
    }, []);

    useEffect(() => {
        if (!session) return;
        let cancelled = false;
        const sessionId = session.sessionId;
        const current = runtime.current;
        if (current.disposed || current.snapshot?.sessionId !== sessionId) return () => { cancelled = true; };
        current.closeRequested = false;
        current.cursor = session.cursor;
        current.truncationNotified = false;
        const read = async () => {
            if (current.disposed || current.readingFor === sessionId || current.snapshot?.sessionId !== sessionId) return;
            current.readingFor = sessionId;
            try {
                const snapshot = await actions.readTerminal(sessionId, current.cursor);
                if (current.disposed || current.snapshot?.sessionId !== sessionId) return;
                if (snapshot.sessionId !== sessionId) {
                    setConsent('PTY output arrived for the wrong session; it was discarded.');
                    return;
                }
                current.snapshot = snapshot;
                current.cursor = snapshot.cursor;
                for (const event of snapshot.events) {
                    if (event.type === 'output' && event.dataHex) current.terminal?.write(bytesFromHex(event.dataHex));
                    if (event.type === 'exit') setConsent(event.code === null || event.code === undefined ? 'Shell ended.' : `Shell exited with code ${event.code}.`);
                    if (event.type === 'error') setConsent(event.message || 'PTY error.');
                }
                if (snapshot.truncated && !current.truncationNotified) {
                    current.truncationNotified = true;
                    setConsent('PTY output history is capped at 1 MiB; older output is discarded.');
                }
                if (!snapshot.active) {
                    setSession(snapshot);
                    if (current.poller !== null) window.clearInterval(current.poller);
                    current.poller = null;
                }
            } catch (error) {
                if (!current.disposed && current.snapshot?.sessionId === sessionId) setConsent(`Could not read PTY output: ${String(error instanceof Error ? error.message : error)}`);
            } finally {
                if (current.readingFor === sessionId) current.readingFor = null;
            }
        };
        const mount = async () => {
            const [{ Terminal }, { FitAddon }] = await Promise.all([import('@xterm/xterm'), import('@xterm/addon-fit')]);
            if (cancelled || current.disposed || current.snapshot?.sessionId !== sessionId || !containerRef.current) return;
            const element = containerRef.current;
            const style = getComputedStyle(element);
            const terminal = new Terminal({ cols: 80, rows: 24, fontFamily: 'ui-monospace, SFMono-Regular, Consolas, monospace', fontSize: 13, screenReaderMode: true, theme: { background: style.backgroundColor, foreground: style.color, cursor: style.color } });
            const fit = new FitAddon();
            terminal.open(element);
            terminal.loadAddon(fit);
            fit.fit();
            current.terminal = terminal;
            terminal.onData((data) => {
                if (current.disposed || current.snapshot?.sessionId !== sessionId) return;
                void actions.inputTerminal(sessionId, data).catch((error) => {
                    if (!current.disposed && current.snapshot?.sessionId === sessionId) setConsent(`Terminal input failed: ${String(error instanceof Error ? error.message : error)}`);
                });
            });
            terminal.onResize(({ cols, rows }) => {
                if (current.disposed || current.snapshot?.sessionId !== sessionId) return;
                void actions.resizeTerminal(sessionId, rows, cols).catch((error) => {
                    if (!current.disposed && current.snapshot?.sessionId === sessionId) setConsent(`Terminal resize failed: ${String(error instanceof Error ? error.message : error)}`);
                });
            });
            terminal.focus();
            if (typeof ResizeObserver === 'function') {
                current.observer = new ResizeObserver(() => { if (!current.disposed) fit.fit(); });
                current.observer.observe(element);
            }
            void read();
            current.poller = window.setInterval(() => { void read(); }, 50);
        };
        void mount().catch((error) => {
            if (!cancelled && !current.disposed && current.snapshot?.sessionId === sessionId) setConsent(`Could not mount the terminal: ${String(error instanceof Error ? error.message : error)}`);
        });
        return () => { cancelled = true; };
    }, [session?.sessionId]);

    async function open() {
        const current = runtime.current;
        if (opening || closing || session || failedSession || current.snapshot || !actions.available) return;
        const cwd = path.trim();
        if (!cwd) {
            setConsent('Choose a project folder before opening an interactive PTY.');
            return;
        }
        const requestedRevision = workspaceRevision.current;
        current.disposed = false;
        setOpening(true);
        try {
            const enabled = await actions.enableTerminal();
            if (enabled?.enabled !== true) {
                if (!current.disposed && requestedRevision === workspaceRevision.current) setConsent('Terminal enablement was cancelled; no PTY was started.');
                return;
            }
            if (current.disposed || requestedRevision !== workspaceRevision.current) return;
            const snapshot = await actions.openTerminal(cwd, 24, 80);
            current.sessionRevision = requestedRevision;
            current.snapshot = snapshot;
            if (current.disposed || requestedRevision !== workspaceRevision.current) {
                closeSession(false);
                if (mounted.current) setConsent('Workspace changed before the shell opened; close the stale shell before opening another.');
                return;
            }
            current.disposed = false;
            setSession(snapshot);
            setConsent(`Interactive PTY · ${snapshot.shell} · ${snapshot.cwd}`);
        } catch (error) {
            if (current.disposed || requestedRevision !== workspaceRevision.current) return;
            const detail = String(error instanceof Error ? error.message : error);
            setConsent(detail.includes('TerminalConsentRequired') || detail.includes('cancelled') ? 'Shell opening was cancelled; no PTY was started.' : `Could not open the PTY: ${detail}`);
        } finally {
            if (mounted.current) setOpening(false);
        }
    }

    return <section aria-label="Terminal" class="desktop-panel terminal-panel">
        <header class="panel-heading"><div><h1>Terminal</h1><p>Persistent PTY with direct input and live output.</p></div><span class="panel-icon"><Icon name="terminal" /></span></header>
        <p id={id('terminalConsentState')} role="status" aria-live="polite" class="panel-summary">{consent || (actions.available ? 'Open a shell for a persistent interactive PTY.' : 'Interactive PTY is unavailable in this browser build.')}</p>
        <label>Working directory<input id={id('terminalCwd')} aria-label="Working directory" value={path} disabled={Boolean(session) || opening} onInput={(event) => {
            workspaceRevision.current += 1;
            closeSession();
            updatePath(event.currentTarget.value);
        }} /></label>
        <div class="panel-actions">
            {!session && !failedSession && <button id={id('btnTerminalOpen')} type="button" disabled={!actions.available || opening || closing || Boolean(runtime.current.snapshot)} onClick={() => void open()}>{opening ? 'Opening…' : 'Open terminal'}</button>}
            {(session || failedSession) && <button id={id('btnTerminalClose')} type="button" disabled={closing} onClick={() => closeSession()}>{closing ? 'Closing…' : failedSession ? 'Retry close' : 'Close shell'}</button>}
        </div>
        <div id={id('terminalOutput')} ref={containerRef} class="terminal-screen" role="group" aria-label="Interactive PTY terminal" data-session-id={session?.sessionId} onClick={() => runtime.current.terminal?.focus()} />
    </section>;
}
