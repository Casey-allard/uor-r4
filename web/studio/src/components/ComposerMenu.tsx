import type { AttachmentDraft } from './ComposerAttachments.js';

export interface ComposerEditorFile {
    path: string;
    content: string;
}

export interface ComposerArtifact {
    name: string;
    content: string;
    source?: string;
}

export interface ComposerMenuState {
    kind: 'commands' | 'mentions';
    query: string;
    start: number;
    end: number;
    activeIndex: number;
}

export type ComposerOption =
    | { id: string; kind: 'command'; label: string; template: string }
    | { id: string; kind: 'mention'; label: string; attachment: AttachmentDraft; add: boolean };

export const LOCAL_TEMPLATES = [
    ['ux', 'Studio UX improvement brief'],
    ['api', 'API integration checklist'],
    ['audit', 'Audit this file'],
    ['report', 'Write a milestone REPORT'],
    ['error', 'Explain this error'],
    ['test', 'Write a verification plan'],
] as const;

const encoder = new TextEncoder();

function editorDraft(file: ComposerEditorFile): AttachmentDraft {
    return { id: `editor:${file.path}`, name: file.path, kind: 'document', bytes: new Uint8Array(0), size: encoder.encode(file.content).byteLength, mediaType: 'text/plain' };
}

function artifactDraft(artifact: ComposerArtifact): AttachmentDraft {
    const bytes = encoder.encode(artifact.content);
    return { id: `artifact:${artifact.name}`, name: artifact.name, kind: 'document', bytes, size: bytes.byteLength, mediaType: 'text/plain' };
}

export function commandOptions(query: string): ComposerOption[] {
    const normalized = query.toLowerCase();
    return LOCAL_TEMPLATES
        .filter(([kind, label]) => `${kind} ${label}`.toLowerCase().includes(normalized))
        .map(([template, label]) => ({ id: `agent-composer-command-${template}`, kind: 'command', label, template }));
}

export function mentionOptions(
    query: string,
    editorFile: ComposerEditorFile | null,
    attachments: readonly AttachmentDraft[],
    artifacts: readonly ComposerArtifact[] = [],
): ComposerOption[] {
    const normalized = query.toLowerCase();
    const retained = new Set(attachments.map((attachment) => attachment.name));
    const candidates: Array<{ attachment: AttachmentDraft; add: boolean; context: string }> = [];
    if (editorFile) candidates.push({ attachment: editorDraft(editorFile), add: !retained.has(editorFile.path), context: 'Active file' });
    for (const attachment of attachments) candidates.push({ attachment, add: false, context: 'Attachment' });
    for (const artifact of artifacts) candidates.push({ attachment: artifactDraft(artifact), add: false, context: 'Artifact' });
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

export function menuTrigger(textarea: HTMLTextAreaElement): Omit<ComposerMenuState, 'activeIndex'> | null {
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

interface ComposerMenuProps {
    menu: ComposerMenuState;
    options: readonly ComposerOption[];
    activeIndex: number;
    onSelect(option: ComposerOption): void;
}

export function ComposerMenu({ menu, options, activeIndex, onSelect }: ComposerMenuProps) {
    return <div id={`agent-composer-${menu.kind}`} class="composer-menu" role="listbox" aria-label={menu.kind === 'commands' ? 'Build task commands' : 'Attachment mentions'}>
        {options.map((option, index) => <div
            key={option.id}
            id={option.id}
            role="option"
            aria-selected={index === activeIndex ? 'true' : 'false'}
            data-template={option.kind === 'command' ? option.template : undefined}
            data-attachment={option.kind === 'mention' ? option.attachment.name : undefined}
            onMouseDown={(event) => event.preventDefault()}
            onClick={() => onSelect(option)}
        >{option.label}</div>)}
    </div>;
}
