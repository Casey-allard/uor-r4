import { useMemo } from 'preact/hooks';

import type { ArtifactContent } from '../agent/client.js';

export const MAX_PREVIEW_TEXT_BYTES = 64 * 1024;
export const MAX_PREVIEW_IMAGE_BYTES = 1024 * 1024;

export const SAFE_TEXT_MEDIA_TYPES = [
    'text/plain',
    'text/markdown',
    'text/csv',
    'application/json',
] as const;
export const SAFE_IMAGE_MEDIA_TYPES = [
    'image/png',
    'image/jpeg',
    'image/gif',
    'image/webp',
] as const;

export type ArtifactPreviewValidation =
    | { ok: true; kind: 'text'; mediaType: string; text: string }
    | { ok: true; kind: 'image'; mediaType: string; src: string }
    | { ok: false; mediaType: string; reason: string };

const MEDIA_TYPE_PATTERN = /^[a-z0-9][a-z0-9!#$&^_.+-]*\/[a-z0-9][a-z0-9!#$&^_.+-]*$/;

function normalizeMediaType(value: unknown): string {
    if (typeof value !== 'string') return '';
    return value.split(';', 1)[0]!.trim().toLowerCase();
}

function textBytesSafe(bytes: Uint8Array): string | null {
    let text: string;
    try {
        text = new TextDecoder('utf-8', { fatal: true }).decode(bytes);
    } catch {
        return null;
    }
    for (const character of text) {
        const code = character.codePointAt(0)!;
        if (
            code === 0 ||
            code === 0x7f ||
            (code < 0x20 && code !== 0x09 && code !== 0x0a && code !== 0x0d)
        ) {
            return null;
        }
    }
    return text;
}

function imageBytesMatch(mediaType: string, bytes: Uint8Array): boolean {
    if (mediaType === 'image/png') {
        const signature = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
        return bytes.length >= signature.length && signature.every((value, index) => bytes[index] === value);
    }
    if (mediaType === 'image/jpeg') {
        return bytes.length >= 3 && bytes[0] === 0xff && bytes[1] === 0xd8 && bytes[2] === 0xff;
    }
    if (mediaType === 'image/gif') {
        const ascii = String.fromCharCode(...bytes.slice(0, 6));
        return ascii === 'GIF87a' || ascii === 'GIF89a';
    }
    if (mediaType === 'image/webp') {
        return (
            bytes.length >= 12 &&
            String.fromCharCode(...bytes.slice(0, 4)) === 'RIFF' &&
            String.fromCharCode(...bytes.slice(8, 12)) === 'WEBP'
        );
    }
    return false;
}

function base64(bytes: Uint8Array): string {
    let binary = '';
    for (const byte of bytes) binary += String.fromCharCode(byte);
    return globalThis.btoa(binary);
}

/**
 * Accepts a byte view from the page realm or a test realm without trusting
 * array-like objects. A non-byte value is refused before any decoding.
 */
export function artifactBytes(value: unknown): Uint8Array | null {
    if (value instanceof Uint8Array) return value;
    if (ArrayBuffer.isView(value)) {
        const view = value as ArrayBufferView;
        return new Uint8Array(
            view.buffer.slice(
                view.byteOffset,
                view.byteOffset + view.byteLength,
            ),
        );
    }
    return null;
}

/**
 * Only declared, bounded text and raster image bytes are previewed. HTML, SVG,
 * scripts, iframes and remote subresources never reach the DOM. Images are
 * encoded from the already verified bytes as a data URL, so the browser cannot
 * issue a request for them.
 */
export function validateArtifactPreview(content: ArtifactContent): ArtifactPreviewValidation {
    const mediaType = normalizeMediaType(content.mediaType);
    if (!MEDIA_TYPE_PATTERN.test(mediaType)) {
        return { ok: false, mediaType, reason: 'The declared artifact media type is malformed; it is not rendered.' };
    }
    const bytes = artifactBytes(content.bytes);
    if (bytes === null) {
        return { ok: false, mediaType, reason: 'The artifact has no verified bytes to preview.' };
    }
    if ((SAFE_TEXT_MEDIA_TYPES as readonly string[]).includes(mediaType)) {
        if (bytes.byteLength > MAX_PREVIEW_TEXT_BYTES) {
            return { ok: false, mediaType, reason: `Text preview size is limited to ${MAX_PREVIEW_TEXT_BYTES} bytes; this artifact is larger.` };
        }
        const text = textBytesSafe(bytes);
        if (text === null) {
            return { ok: false, mediaType, reason: 'The text encoding is not valid UTF-8 or contains unsupported control characters; it is not rendered.' };
        }
        return { ok: true, kind: 'text', mediaType, text };
    }
    if ((SAFE_IMAGE_MEDIA_TYPES as readonly string[]).includes(mediaType)) {
        if (bytes.byteLength > MAX_PREVIEW_IMAGE_BYTES) {
            return { ok: false, mediaType, reason: `Image preview size is limited to ${MAX_PREVIEW_IMAGE_BYTES} bytes; this artifact is larger.` };
        }
        if (!imageBytesMatch(mediaType, bytes)) {
            return { ok: false, mediaType, reason: 'The declared image media type does not match its bytes; it is not rendered.' };
        }
        return {
            ok: true,
            kind: 'image',
            mediaType,
            src: `data:${mediaType};base64,${base64(bytes)}`,
        };
    }
    return {
        ok: false,
        mediaType,
        reason: `The media type ${mediaType || '(missing)'} is not in the declared text or raster image preview set; it is not rendered.`,
    };
}

export function ArtifactPreview({ content }: { content: ArtifactContent }) {
    const validation = useMemo(() => validateArtifactPreview(content), [content]);
    if (!validation.ok) {
        return <p class="panel-summary" role="status" data-artifact-preview="unsupported" data-artifact-media-type={validation.mediaType}>
            {validation.reason}
        </p>;
    }
    if (validation.kind === 'text') {
        return <pre class="desktop-output artifact-preview" data-artifact-preview="text" data-artifact-media-type={validation.mediaType}>{validation.text}</pre>;
    }
    return <figure class="artifact-image-preview">
        <img
            data-artifact-preview="image"
            data-artifact-media-type={validation.mediaType}
            src={validation.src}
            alt={`Preview of ${content.path}`}
            referrerPolicy="no-referrer"
            style={{ maxWidth: '100%', height: 'auto' }}
        />
    </figure>;
}
