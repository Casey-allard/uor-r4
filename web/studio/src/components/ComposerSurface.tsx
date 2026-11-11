import { flushSync } from 'preact/compat';
import { useEffect, useRef, useState } from 'preact/hooks';
import { createComposerInput } from '../adapters/composerInput.js';
import type { ComposerInputController } from '../adapters/composerInput.js';
import type { RunSummary } from '../agent/client.js';
import { ComposerAttachments } from './ComposerAttachments.js';
import type { AttachmentDraft } from './ComposerAttachments.js';
import { ComposerMenu, commandOptions, mentionOptions, menuTrigger } from './ComposerMenu.js';
import type { ComposerArtifact, ComposerEditorFile, ComposerMenuState, ComposerOption } from './ComposerMenu.js';
import { EnvironmentPicker } from './EnvironmentPicker.js';
import { PermissionPicker } from './PermissionPicker.js';
import type { PermissionMode } from './PermissionPicker.js';
import { SendModeControls } from './SendModeControls.js';
import { Icon } from './ui/Icon.js';

type SendResult = boolean | Promise<boolean>;
type SendAction = (prompt: string) => SendResult;

export interface ComposerSurfaceProps {
    value: string;
    onChange(value: string): void;
    onDraftChange?(value: string): void;
    attachments: readonly AttachmentDraft[];
    onAttach(files: File[]): void | Promise<void>;
    onRemove(id: string): void;
    onSend(prompt: string): SendResult;
    onQueue?(prompt: string): SendResult;
    onSteer?(prompt: string): SendResult;
    onForce?(prompt: string): SendResult;
    onStop?(): void;
    onStopRun?(): void;
    showGenerationStop?: boolean;
    onActionError?(error: unknown): void;
    activeRun?: RunSummary | null;
    mode: PermissionMode;
    onMode?(mode: PermissionMode): void;
    model: string;
    onModel(): void;
    environment: string;
    onEnvironment(value: string): void;
    disabled?: boolean;
    runtimeAvailable?: boolean;
    initialDraft?: string;
    sendLabel?: string;
    generating?: boolean;
    canStop?: boolean;
    editorFile?: ComposerEditorFile | null;
    artifacts?: readonly ComposerArtifact[];
    templatePrompt?(kind: string): string;
    onMentionAdd?(attachment: { name: string; content: string; source: string }): boolean | Promise<boolean>;
    onAttachmentBarrier?(): Promise<void> | null;
    onController?(controller: ComposerInputController | null): void;
}

export function ComposerSurface(props: ComposerSurfaceProps) {
    const [menu, setMenu] = useState<ComposerMenuState | null>(null);
    const textareaRef = useRef<HTMLTextAreaElement>(null);
    const pickerRef = useRef<HTMLInputElement>(null);
    const controllerRef = useRef<ComposerInputController | null>(null);
    const menuRef = useRef<ComposerMenuState | null>(null);
    const optionsRef = useRef<ComposerOption[]>([]);
    const menuKeyHandlerRef = useRef<(event: KeyboardEvent) => boolean>(() => false);
    const selectOptionRef = useRef<(option: ComposerOption) => void>(() => {});
    const submitRef = useRef<() => void>(() => {});
    const attachRef = useRef<(files: File[]) => void | Promise<void>>(async () => {});
    const draftChangeRef = useRef<(value: string) => void>(() => {});
    const inputChangeRef = useRef<(value: string) => void>(() => {});
    const templatePromptRef = useRef<(kind: string) => string>(() => '');
    const mentionAddRef = useRef<(attachment: { name: string; content: string; source: string }) => boolean | Promise<boolean>>(() => false);
    const editorFileRef = useRef<ComposerEditorFile | null>(null);
    const onControllerRef = useRef<((controller: ComposerInputController | null) => void) | undefined>(undefined);
    const attachmentBarrierRef = useRef<(() => Promise<void> | null) | undefined>(undefined);
    const pendingMentionsRef = useRef<Set<Promise<void>>>(new Set());
    const sendingRef = useRef(false);
    const mountedRef = useRef(true);
    const sendRef = useRef<SendAction>(() => false);
    const queueRef = useRef<SendAction | undefined>(undefined);
    const steerRef = useRef<SendAction | undefined>(undefined);
    const forceRef = useRef<SendAction | undefined>(undefined);
    const actionErrorRef = useRef<((error: unknown) => void) | undefined>(undefined);
    const submittedHistoryRef = useRef<string[]>([]);
    const initialDraftRef = useRef(props.initialDraft ?? '');

    function notifyDraftChange(value: string) {
        draftChangeRef.current(value);
    }

    function setDraft(value: string) {
        const controller = controllerRef.current;
        if (controller) controller.reset(value);
        else if (textareaRef.current) textareaRef.current.value = value;
        inputChangeRef.current(value);
        notifyDraftChange(value);
    }

    function acceptSubmission(prompt: string) {
        if (!mountedRef.current) return;
        const textarea = textareaRef.current;
        const current = textarea?.value ?? props.value;
        if (textarea && current !== prompt) {
            const controller = controllerRef.current;
            if (controller) {
                controller.recordSubmitted(prompt);
                setDraft(current);
            } else {
                notifyDraftChange(current);
                flushSync(() => {
                    inputChangeRef.current(current);
                    setMenu(null);
                });
            }
            textarea.focus();
            return;
        }
        if (controllerRef.current) controllerRef.current.recordSubmitted(prompt);
        else notifyDraftChange('');
        if (textarea) textarea.value = '';
        flushSync(() => {
            inputChangeRef.current('');
            setMenu(null);
        });
        textarea?.focus();
    }

    function executeAction(action: SendAction) {
        if (!mountedRef.current) {
            sendingRef.current = false;
            return;
        }
        const textarea = textareaRef.current;
        const prompt = textarea?.value ?? props.value;
        if (!prompt.trim()) {
            sendingRef.current = false;
            return;
        }
        let result: SendResult;
        try {
            result = action(prompt);
        } catch (error) {
            sendingRef.current = false;
            actionErrorRef.current?.(error);
            return;
        }
        if (result === true) {
            sendingRef.current = false;
            acceptSubmission(prompt);
            return;
        }
        if (result === false) {
            sendingRef.current = false;
            return;
        }
        void Promise.resolve(result).then((accepted) => {
            if (accepted) acceptSubmission(prompt);
        }, () => undefined).finally(() => {
            sendingRef.current = false;
        });
    }

    function runPromptAction(action: SendAction | undefined) {
        if (!action || sendingRef.current || !mountedRef.current) return;
        sendingRef.current = true;
        const pendingMentions = Array.from(pendingMentionsRef.current);
        const barrier = attachmentBarrierRef.current?.() ?? null;
        if (!barrier && !pendingMentions.length) {
            executeAction(action);
            return;
        }
        void Promise.all([barrier ?? Promise.resolve(), ...pendingMentions]).then(() => {
            if (!mountedRef.current) {
                sendingRef.current = false;
                return;
            }
            executeAction(action);
        }, () => {
            sendingRef.current = false;
        });
    }

    function submit() {
        if (props.disabled || sendingRef.current || !mountedRef.current) return;
        const textarea = textareaRef.current;
        const initial = textarea?.value ?? props.value;
        if (!initial.trim()) return;
        runPromptAction(sendRef.current);
    }

    sendRef.current = props.onSend;
    queueRef.current = props.onQueue;
    steerRef.current = props.onSteer;
    forceRef.current = props.onForce;
    actionErrorRef.current = props.onActionError;

    function updateMenu(textarea: HTMLTextAreaElement) {
        const trigger = menuTrigger(textarea);
        setMenu(trigger ? { ...trigger, activeIndex: 0 } : null);
        if (trigger) textarea.focus();
    }

    async function selectOption(option: ComposerOption) {
        const textarea = textareaRef.current;
        if (option.kind === 'command') {
            setDraft(templatePromptRef.current(option.template));
            setMenu(null);
            textarea?.focus();
            return;
        }
        const currentMenu = menuRef.current;
        if (!currentMenu || !textarea) return;
        const mention = `@${option.attachment.name}`;
        if (!option.add) {
            const visiblePrompt = textarea.value;
            setDraft(`${visiblePrompt.slice(0, currentMenu.start)}${mention}${visiblePrompt.slice(currentMenu.end)}`);
            setMenu(null);
            textarea.focus();
            return;
        }
        const editor = editorFileRef.current;
        setMenu(null);
        if (!editor || editor.path !== option.attachment.name) {
            textarea.focus();
            return;
        }
        const original = textarea.value;
        const continuation = (async () => {
            try {
                const accepted = await mentionAddRef.current({ name: editor.path, content: editor.content, source: 'editor' });
                if (!accepted) return;
                const current = textareaRef.current;
                if (!current || current.value !== original) return;
                setDraft(`${current.value.slice(0, currentMenu.start)}${mention}${current.value.slice(currentMenu.end)}`);
            } catch {
                // Keep the user's current draft when the attach callback rejects.
            } finally {
                textareaRef.current?.focus();
            }
        })();
        const tracked = continuation.then(() => undefined, () => undefined);
        pendingMentionsRef.current.add(tracked);
        void tracked.finally(() => pendingMentionsRef.current.delete(tracked));
        await tracked;
    }

    function interceptMenuKey(event: KeyboardEvent): boolean {
        const currentMenu = menuRef.current;
        const currentOptions = optionsRef.current;
        if (!currentMenu || !currentOptions.length || event.isComposing || event.keyCode === 229) return false;
        if (event.key === 'Escape') {
            event.preventDefault();
            event.stopPropagation();
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

    const options = menu?.kind === 'commands'
        ? commandOptions(menu.query)
        : menu?.kind === 'mentions' ? mentionOptions(menu.query, props.editorFile ?? null, props.attachments, props.artifacts ?? []) : [];
    const menuOpen = Boolean(menu && options.length);
    const activeIndex = options.length ? Math.min(menu?.activeIndex ?? 0, options.length - 1) : 0;
    const menuId = menu && menuOpen ? `agent-composer-${menu.kind}` : undefined;

    menuRef.current = menu;
    optionsRef.current = options;
    menuKeyHandlerRef.current = interceptMenuKey;
    selectOptionRef.current = selectOption;
    submitRef.current = submit;
    attachRef.current = props.onAttach;
    draftChangeRef.current = (value) => props.onDraftChange?.(value);
    inputChangeRef.current = props.onChange;
    templatePromptRef.current = props.templatePrompt ?? ((kind: string) => `Prompt for ${kind}`);
    mentionAddRef.current = (attachment) => props.onMentionAdd?.(attachment) ?? false;
    editorFileRef.current = props.editorFile ?? null;
    onControllerRef.current = props.onController;
    attachmentBarrierRef.current = props.onAttachmentBarrier;

    useEffect(() => () => {
        mountedRef.current = false;
        pendingMentionsRef.current.clear();
    }, []);

    useEffect(() => {
        const textarea = textareaRef.current;
        if (!textarea) return;
        const controller = createComposerInput({
            textarea,
            picker: pickerRef.current,
            initialDraft: initialDraftRef.current,
            getHistory: () => submittedHistoryRef.current,
            onHistoryChange: (history) => { submittedHistoryRef.current = history; },
            onSend: () => submitRef.current(),
            onFiles: (files) => { void attachRef.current(files); },
            onDraftChange: (value) => draftChangeRef.current(value),
            interceptKeyDown: (event) => menuKeyHandlerRef.current(event),
        });
        controllerRef.current = controller;
        onControllerRef.current?.(controller);
        return () => {
            controllerRef.current = null;
            onControllerRef.current?.(null);
            controller.destroy();
        };
    }, []);

    return <div class="agent-composer composer-surface" data-composer-surface="true">
        <div class="composer-picker-row">
            <EnvironmentPicker value={props.environment} onChange={props.onEnvironment} disabled={props.disabled} />
            <PermissionPicker mode={props.mode} onChange={props.onMode} hostAvailable={props.runtimeAvailable === true} />
            <button type="button" class="composer-model-button" onClick={() => props.onModel()}>{`Model: ${props.model}`}</button>
        </div>
        <ComposerAttachments attachments={props.attachments} onRemove={props.onRemove} pickerRef={pickerRef} />
        {menuOpen && menu && <ComposerMenu menu={menu} options={options} activeIndex={activeIndex} onSelect={selectOption} />}
        <div class="composer-input-row">
            <label class="composer-prompt-field"><span class="visually-hidden">Prompt</span><textarea
                ref={textareaRef}
                aria-label="Prompt"
                placeholder="Ask R4 Harness, or type / for tasks"
                aria-haspopup="listbox"
                aria-expanded={menuOpen ? 'true' : 'false'}
                aria-controls={menuId}
                aria-activedescendant={menuOpen ? options[activeIndex].id : undefined}
                defaultValue={initialDraftRef.current}
                onInput={(event) => {
                    const element = event.currentTarget;
                    flushSync(() => {
                        inputChangeRef.current(element.value);
                        updateMenu(element);
                    });
                }}
            /></label>
            <button type="button" class="send-button" disabled={props.disabled || !props.value.trim()} onClick={() => submitRef.current()}><Icon name="send" /><span>{props.sendLabel ?? 'Send'}</span></button>
        </div>
        <div class="composer-footer"><span>Enter to send · Shift+Enter for a line · / tasks · @ context</span><SendModeControls
            activeRun={props.activeRun ?? null}
            onQueue={() => runPromptAction(queueRef.current)}
            onSteer={() => runPromptAction(steerRef.current)}
            onForce={() => runPromptAction(forceRef.current)}
            onStop={() => props.onStopRun?.()}
        />{props.showGenerationStop !== false && <button type="button" disabled={!props.generating || props.canStop === false} onClick={() => props.onStop?.()}>Stop</button>}</div>
    </div>;
}
