import { useMemo } from 'preact/hooks';
import type { FilePhase } from '../state/lifecycle.js';
import type { StudioFile, StudioSettings } from '../adapters/workflow.js';
import { MAX_DOCUMENT_BYTES } from './ComposerAttachments.js';

export function fileTooLargeMessage(name: string, sizeBytes: number, limitBytes: number = MAX_DOCUMENT_BYTES): string {
    return `${name} exceeds the ${limitBytes}-byte document limit (${sizeBytes} bytes).`;
}

export function textByteLength(content: string): number {
    return content.length === 0 ? 0 : new TextEncoder().encode(content).byteLength;
}

export interface EditorPaneProps {
    active: StudioFile | null;
    phase: FilePhase;
    settings: StudioSettings;
    onContentChange(content: string): void;
}

export function EditorPane({ active, phase, settings, onContentChange }: EditorPaneProps) {
    const content = active?.content ?? '';
    const bytes = useMemo(() => textByteLength(content), [content]);
    const refused = active !== null && bytes > MAX_DOCUMENT_BYTES;
    const lines = Math.max(1, content.split('\n').length);

    return (
        <section class="editor-pane" aria-label="UOR text editor">
            <header class="editor-pane-heading">
                <h2>UOR text editor</h2>
                <p>Language services, debugger, extensions and notebooks are not available.</p>
            </header>
            {refused ? (
                <p class="workbench-alert" role="alert">{fileTooLargeMessage(active.name, bytes)}</p>
            ) : (
                <div class="code-frame" role="tabpanel" aria-label="Editor contents">
                    <div class="code-gutter" aria-hidden="true">
                        {Array.from({ length: lines }, (_, index) => <div>{index + 1}</div>)}
                    </div>
                    <label class="code-input">
                        <span class="visually-hidden">Editor</span>
                        <textarea
                            aria-label="File contents"
                            placeholder={active ? undefined : 'Open a file to start editing, or choose Open demo.'}
                            disabled={!active || phase === 'saving'}
                            style={{ fontSize: `${settings.fontSize}px` }}
                            wrap={settings.wordWrap ? 'soft' : 'off'}
                            value={content}
                            onKeyDown={(event) => {
                                if (event.isComposing || event.keyCode === 229) return;
                                if (event.key !== 'Tab' || !active) return;
                                event.preventDefault();
                                const input = event.currentTarget;
                                const padding = ' '.repeat(settings.tabSize);
                                const start = input.selectionStart;
                                const end = input.selectionEnd;
                                onContentChange(`${input.value.slice(0, start)}${padding}${input.value.slice(end)}`);
                                input.setSelectionRange(start + padding.length, start + padding.length);
                            }}
                            onInput={(event) => {
                                if (!active || event.isComposing) return;
                                onContentChange(event.currentTarget.value);
                            }}
                            onCompositionEnd={(event) => {
                                if (!active) return;
                                onContentChange(event.currentTarget.value);
                            }}
                        />
                    </label>
                </div>
            )}
        </section>
    );
}
