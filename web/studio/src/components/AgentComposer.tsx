import { flushSync } from 'preact/compat';
import { useEffect, useRef, useState } from 'preact/hooks';
import { createComposerInput, readTextFiles } from '../adapters/composerInput.js';
import type { ComposerInputController } from '../adapters/composerInput.js';
import { GENERATION_FINISHED_EVENT } from '../adapters/generation.js';
import type { GenerationAdapter } from '../adapters/generation.js';
import type { Attachment, SessionApi, StudioUiApi, WorkspaceApi } from '../adapters/workflow.js';
import type { LifecycleStore } from '../state/lifecycle.js';
import { useLifecycle } from '../state/useLifecycle.js';
import { RichMessage } from './RichMessage.js';

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

interface ComposerMenu {
    kind: 'commands' | 'mentions';
    query: string;
    start: number;
    end: number;
    activeIndex: number;
}

type ComposerOption =
    | { id: string; kind: 'command'; label: string; template: string }
    | { id: string; kind: 'mention'; label: string; attachment: Attachment; add: boolean };

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

function commandOptions(query: string): ComposerOption[] {
    const normalized = query.toLowerCase();
    return templates
        .filter(([kind, label]) => `${kind} ${label}`.toLowerCase().includes(normalized))
        .map(([template, label]) => ({ id: `agent-composer-command-${template}`, kind: 'command', label, template }));
}

function mentionOptions(query: string, activeFile: ReturnType<WorkspaceApi['activeFile']>, attachments: Attachment[]): ComposerOption[] {
    const normalized = query.toLowerCase();
    const retained = new Set(attachments.map((attachment) => attachment.name));
    const candidates: Array<{ attachment: Attachment; add: boolean; context: string }> = [];
    if (activeFile) {
        candidates.push({
            attachment: { name: activeFile.path, content: activeFile.content, source: 'editor' },
            add: !retained.has(activeFile.path),
            context: 'Active file',
        });
    }
    for (const attachment of attachments) candidates.push({ attachment, add: false, context: 'Attachment' });
    const seen = new Set<string>();
    return candidates.filter(({ attachment }) => {
        if (seen.has(attachment.name) || !attachment.name.toLowerCase().includes(normalized)) return false;
        seen.add(attachment.name);
        return true;
    }).map(({ attachment, add, context }) => ({
        id: `agent-composer-mention-${encodeURIComponent(attachment.name)}`,
        kind: 'mention',
        label: `${attachment.name} (${context})`,
        attachment,
        add,
    }));
}

function menuTrigger(textarea: HTMLTextAreaElement): Omit<ComposerMenu, 'activeIndex'> | null {
    const caret = textarea.selectionStart;
    const match = /(^|\s)([/@])([^\s]*)$/.exec(textarea.value.slice(0, caret));
    if (!match) return null;
    return {
        kind: match[2] === '/' ? 'commands' : 'mentions',
        query: match[3],
        start: caret - match[2].length - match[3].length,
        end: caret,
    };
}

export function AgentComposer({ store, sessions, files, ui, generation, copy, download, draft: controlledDraft, pasted: controlledPasted, onDraftChange, onPastedChange }: AgentComposerProps) {
    const state = useLifecycle(store);
    const [pastedState, setPastedState] = useState('');
    const pasted = controlledPasted ?? pastedState;
    const setPasted = (value: string) => { setPastedState(value); onPastedChange?.(value); };
    const [visibleDraft, setVisibleDraft] = useState(controlledDraft ?? '');
    const initialDraftRef = useRef(controlledDraft ?? '');
    const previousControlledDraft = useRef(controlledDraft);
    const ownControlledDraft = useRef<string | null>(null);
    const submittedHistoryRef = useRef<string[]>([]);
    const [, setRevision] = useState(0);
    const refresh = () => setRevision((value) => value + 1);
    const [menu, setMenu] = useState<ComposerMenu | null>(null);
    const textareaRef = useRef<HTMLTextAreaElement>(null);
    const pickerRef = useRef<HTMLInputElement>(null);
    const controllerRef = useRef<ComposerInputController | null>(null);
    const menuRef = useRef<ComposerMenu | null>(null);
    const optionsRef = useRef<ComposerOption[]>([]);
    const menuKeyHandlerRef = useRef<(event: KeyboardEvent) => boolean>(() => false);
    const selectOptionRef = useRef<(option: ComposerOption) => void>(() => {});
    const sendRef = useRef<(prompt: string) => void>(() => {});
    const draftChangeRef = useRef(notifyDraftChange);
    const fileHandlerRef = useRef<(files: File[]) => Promise<void>>(async () => {});

    function notifyDraftChange(value: string) {
        ownControlledDraft.current = value;
        onDraftChange?.(value);
    }

    function setDraft(value: string) {
        if (controllerRef.current) controllerRef.current.reset(value);
        else if (textareaRef.current) textareaRef.current.value = value;
        setVisibleDraft(value);
        notifyDraftChange(value);
    }

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
    const options = menu?.kind === 'commands'
        ? commandOptions(menu.query)
        : menu?.kind === 'mentions' ? mentionOptions(menu.query, activeFile, attachments) : [];
    const menuOpen = Boolean(menu && options.length);
    const activeIndex = options.length ? Math.min(menu?.activeIndex ?? 0, options.length - 1) : 0;
    const menuId = menu ? `agent-composer-${menu.kind}` : undefined;

    function send(prompt: string) {
        if (generation.send(prompt)) {
            if (controllerRef.current) controllerRef.current.recordSubmitted(prompt);
            else notifyDraftChange('');
            if (textareaRef.current) textareaRef.current.value = '';
            setVisibleDraft('');
            setMenu(null);
            refresh();
        }
    }

    function updateMenu(textarea: HTMLTextAreaElement) {
        const trigger = menuTrigger(textarea);
        setMenu(trigger ? { ...trigger, activeIndex: 0 } : null);
        if (trigger) textarea.focus();
    }

    function selectOption(option: ComposerOption) {
        const textarea = textareaRef.current;
        flushSync(() => {
            if (option.kind === 'command') {
                setDraft(ui.templatePrompt(option.template));
            } else {
                const currentMenu = menuRef.current;
                if (!currentMenu || !textarea) return;
                const mention = `@${option.attachment.name}`;
                const visiblePrompt = textarea.value;
                setDraft(`${visiblePrompt.slice(0, currentMenu.start)}${mention}${visiblePrompt.slice(currentMenu.end)}`);
                if (option.add && !ui.getAttachments().some((attachment) => attachment.name === option.attachment.name)) {
                    ui.addAttachment(option.attachment);
                    refresh();
                }
            }
            setMenu(null);
        });
        textarea?.focus();
    }

    function interceptMenuKey(event: KeyboardEvent): boolean {
        const currentMenu = menuRef.current;
        const currentOptions = optionsRef.current;
        if (!currentMenu || !currentOptions.length || event.isComposing || event.keyCode === 229) return false;
        if (event.key === 'Escape') {
            event.preventDefault();
            flushSync(() => setMenu(null));
            textareaRef.current?.focus();
            return true;
        }
        if (['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) {
            event.preventDefault();
            const currentIndex = Math.min(currentMenu.activeIndex, currentOptions.length - 1);
            const activeIndex = event.key === 'Home' ? 0
                : event.key === 'End' ? currentOptions.length - 1
                : (currentIndex + (event.key === 'ArrowDown' ? 1 : -1) + currentOptions.length) % currentOptions.length;
            flushSync(() => setMenu({ ...currentMenu, activeIndex }));
            return true;
        }
        if (event.key === 'Enter' && !event.shiftKey) {
            event.preventDefault();
            selectOptionRef.current(currentOptions[Math.min(currentMenu.activeIndex, currentOptions.length - 1)]);
            return true;
        }
        return false;
    }

    function handleFiles(selectedFiles: File[]) {
        const existingAttachments = ui.getAttachments().filter((attachment) => attachment.kind !== 'image');
        return readTextFiles(selectedFiles, { existingAttachments }).then((result: { attachments: Attachment[]; errors: string[] }) => {
            for (const attachment of result.attachments) ui.addAttachment(attachment);
            for (const error of result.errors) ui.toast(error, 'error');
            if (result.attachments.length) refresh();
        }).catch((error: unknown) => {
            ui.toast(`Attachments could not be read: ${String(error)}`, 'error');
        });
    }

    menuRef.current = menu;
    optionsRef.current = options;
    menuKeyHandlerRef.current = interceptMenuKey;
    selectOptionRef.current = selectOption;
    sendRef.current = send;
    draftChangeRef.current = notifyDraftChange;
    fileHandlerRef.current = handleFiles;

    useEffect(() => {
        const textarea = textareaRef.current;
        if (!textarea) return;
        const controller = createComposerInput({
            textarea,
            picker: pickerRef.current,
            initialDraft: initialDraftRef.current,
            getHistory: () => submittedHistoryRef.current,
            onHistoryChange: (history) => { submittedHistoryRef.current = history; },
            onSend: () => {
                if (textareaRef.current) sendRef.current(textareaRef.current.value);
            },
            onFiles: (selectedFiles) => { void fileHandlerRef.current(selectedFiles); },
            onDraftChange: (value) => draftChangeRef.current(value),
            interceptKeyDown: (event) => menuKeyHandlerRef.current(event),
        });
        controllerRef.current = controller;
        return () => {
            controllerRef.current = null;
            controller.destroy();
        };
    }, []);

    useEffect(() => {
        if (controlledDraft === undefined || controlledDraft === previousControlledDraft.current) return;
        previousControlledDraft.current = controlledDraft;
        if (ownControlledDraft.current === controlledDraft) {
            ownControlledDraft.current = null;
            return;
        }
        setVisibleDraft(controlledDraft);
        if (controllerRef.current) controllerRef.current.reset(controlledDraft);
        else if (textareaRef.current) textareaRef.current.value = controlledDraft;
    }, [controlledDraft]);

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
                        <RichMessage content={message.content} />
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
                <input ref={pickerRef} type="file" multiple aria-label="Attach text files" />
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
                    <button type="button" data-template={kind} onClick={() => { setDraft(ui.templatePrompt(kind)); setMenu(null); }}>{label}</button>
                ))}
            </section>

            <label>Prompt<textarea
                ref={textareaRef}
                aria-label="Prompt"
                aria-expanded={menuOpen ? 'true' : 'false'}
                aria-controls={menuOpen ? menuId : undefined}
                aria-activedescendant={menuOpen ? options[activeIndex].id : undefined}
                defaultValue={initialDraftRef.current}
                onInput={(event) => {
                    flushSync(() => setVisibleDraft(event.currentTarget.value));
                    updateMenu(event.currentTarget);
                }}
            /></label>
            {menuOpen && menu && (
                <div id={menuId} role="listbox" aria-label={menu.kind === 'commands' ? 'Build task commands' : 'Attachment mentions'}>
                    {options.map((option, index) => (
                        <div
                            id={option.id}
                            role="option"
                            aria-selected={index === activeIndex ? 'true' : 'false'}
                            data-template={option.kind === 'command' ? option.template : undefined}
                            data-attachment={option.kind === 'mention' ? option.attachment.name : undefined}
                            onMouseDown={(event) => event.preventDefault()}
                            onClick={() => selectOption(option)}
                        >{option.label}</div>
                    ))}
                </div>
            )}
            <button type="button" disabled={!visibleDraft.trim() || !store.canRun()} onClick={() => send(textareaRef.current?.value ?? visibleDraft)}>{predictionMode ? 'Predict next tokens' : 'Send'}</button>
            <button type="button" disabled={!generating || activeGeneration?.canStop === false} onClick={() => { generation.stop(); refresh(); }}>Stop</button>
        </section>
    );
}
