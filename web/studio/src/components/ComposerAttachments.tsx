import type { RefObject } from 'preact';
import type { Attachment } from '../adapters/workflow.js';
// @ts-expect-error The existing plain JavaScript image recognizer has no type declarations.
import { isImageFile } from '../../assets/js/image_input.js';

export const MAX_IMAGE_BYTES = 10 * 1024 * 1024;
export const MAX_DOCUMENT_BYTES = 4 * 1024 * 1024;
export const MAX_ATTACHMENTS = 5;
export const MAX_TOTAL_ATTACHMENT_BYTES = 25 * 1024 * 1024;

export interface AttachmentDraft {
    id: string;
    name: string;
    kind: 'image' | 'document' | 'text';
    bytes: Uint8Array;
    size: number;
    mediaType: string;
    previewUrl?: string;
    memoryOnly?: boolean;
}

export interface AttachmentBudgetItem {
    name: string;
    size: number;
    kind?: AttachmentDraft['kind'];
}

export interface AttachmentFileLike {
    name: string;
    size: number;
    type?: string;
}

export interface PlannedAttachment<T extends AttachmentFileLike> {
    file: T;
    kind: 'image' | 'document';
}

export interface AttachmentPlan<T extends AttachmentFileLike> {
    accepted: PlannedAttachment<T>[];
    errors: string[];
}

const encoder = new TextEncoder();
const storedDrafts = new WeakMap<object, AttachmentDraft>();

export function isImageAttachment(file: AttachmentFileLike): boolean {
    return isImageFile(file);
}

export function attachmentSize(entry: Pick<Attachment, 'size' | 'content'>): number {
    return Number.isFinite(Number(entry.size)) ? Number(entry.size) : encoder.encode(String(entry.content || '')).byteLength;
}

export function draftFromStoredAttachment(entry: Attachment, idPrefix = 'stored'): AttachmentDraft {
    const cached = storedDrafts.get(entry);
    if (cached) return cached;
    const kind: AttachmentDraft['kind'] = entry.kind === 'image' ? 'image' : entry.source === 'file' ? 'document' : 'text';
    const draft: AttachmentDraft = {
        id: `${idPrefix}:${entry.name}`,
        name: entry.name,
        kind,
        bytes: kind === 'image' ? new Uint8Array(0) : encoder.encode(String(entry.content || '')),
        size: attachmentSize(entry),
        mediaType: entry.kind === 'image' ? 'image/*' : 'text/plain',
        previewUrl: entry.previewUrl,
        memoryOnly: false,
    };
    storedDrafts.set(entry, draft);
    return draft;
}

export function planAttachmentBatch<T extends AttachmentFileLike>(
    files: readonly T[],
    existing: readonly AttachmentBudgetItem[],
    limits: { maxImageBytes?: number; maxDocumentBytes?: number; maxAttachments?: number; maxTotalBytes?: number } = {},
): AttachmentPlan<T> {
    const maxImageBytes = limits.maxImageBytes ?? MAX_IMAGE_BYTES;
    const maxDocumentBytes = limits.maxDocumentBytes ?? MAX_DOCUMENT_BYTES;
    const maxAttachments = limits.maxAttachments ?? MAX_ATTACHMENTS;
    const maxTotalBytes = limits.maxTotalBytes ?? MAX_TOTAL_ATTACHMENT_BYTES;
    const retained = new Map<string, number>();
    let totalBytes = 0;
    for (const entry of existing) {
        const name = String(entry.name || 'attachment');
        const size = Math.max(0, Number(entry.size) || 0);
        const previous = retained.get(name);
        if (previous !== undefined) totalBytes -= previous;
        retained.set(name, size);
        totalBytes += size;
    }
    const accepted: PlannedAttachment<T>[] = [];
    const errors: string[] = [];
    for (const file of files) {
        const name = String(file.name || 'attachment');
        const size = Math.max(0, Number(file.size) || 0);
        const kind: 'image' | 'document' = isImageAttachment(file) ? 'image' : 'document';
        const perFileLimit = kind === 'image' ? maxImageBytes : maxDocumentBytes;
        if (size > perFileLimit) {
            errors.push(`${name} exceeds the ${perFileLimit}-byte ${kind} limit (${size} bytes).`);
            continue;
        }
        const previous = retained.get(name);
        const nextTotal = totalBytes - (previous ?? 0) + size;
        const nextCount = retained.size + (previous === undefined ? 1 : 0);
        if (nextCount > maxAttachments) {
            errors.push(`${name} exceeds the ${maxAttachments}-attachment limit (${nextCount} attachments requested).`);
            continue;
        }
        if (nextTotal > maxTotalBytes) {
            errors.push(`${name} exceeds the ${maxTotalBytes}-byte total attachment limit (${nextTotal} bytes).`);
            continue;
        }
        retained.set(name, size);
        totalBytes = nextTotal;
        accepted.push({ file, kind });
    }
    return { accepted, errors };
}

interface ComposerAttachmentsProps {
    attachments: readonly AttachmentDraft[];
    onRemove(id: string): void;
    pickerRef: RefObject<HTMLInputElement | null>;
}

function formatBytes(bytes: number): string {
    if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(bytes % (1024 * 1024) === 0 ? 0 : 1)} MiB`;
    return `${Math.ceil(bytes / 1024)} KiB`;
}

export function ComposerAttachments({ attachments, onRemove, pickerRef }: ComposerAttachmentsProps) {
    const totalBytes = attachments.reduce((sum, attachment) => sum + attachment.size, 0);
    return <div class="composer-attachments">
        <div class="composer-attachment-actions">
            <button type="button" class="composer-attach-button" onClick={() => pickerRef.current?.click()}>Attach files</button>
            <span class="composer-attachment-budget" aria-label="Attachment budget">{attachments.length}/{MAX_ATTACHMENTS} files · {formatBytes(totalBytes)}/{formatBytes(MAX_TOTAL_ATTACHMENT_BYTES)}</span>
            <input ref={pickerRef} type="file" multiple aria-label="Attach files" class="visually-hidden" />
        </div>
        {attachments.length > 0 && <ul class="composer-attachment-list" aria-label="Composer attachments">
            {attachments.map((attachment) => <li key={attachment.id} class="composer-attachment-chip" data-attachment-id={attachment.id} data-attachment-kind={attachment.kind}>
                {attachment.kind === 'image' && attachment.previewUrl && <img class="composer-attachment-preview" src={attachment.previewUrl} alt="" />}
                <span class="composer-attachment-name">{attachment.name}</span>
                <span class="composer-attachment-kind">{attachment.kind === 'image' ? (attachment.memoryOnly ? 'image · memory only' : 'image · stored context') : attachment.kind}</span>
                <span class="composer-attachment-size" data-attachment-size={attachment.size}>{formatBytes(attachment.size)}</span>
                <button type="button" onClick={() => onRemove(attachment.id)}>{`Remove ${attachment.name}`}</button>
            </li>)}
        </ul>}
    </div>;
}
