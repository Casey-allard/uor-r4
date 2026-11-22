import { useEffect, useState } from 'preact/hooks';

import {
    AgentClientError,
    type ArtifactContent,
    type ReadArtifactInput,
} from '../agent/client.js';
import { sha256Bytes } from '../agent/hostClient.js';
import { ArtifactPreview, artifactBytes } from './ArtifactPreview.js';
import { Icon } from './ui/Icon.js';

export const ARTIFACT_ID_PATTERN = /^art_[0-9a-f]{64}$/;
export const MAX_ARTIFACT_RANGE_BYTES = 1024 * 1024;
const DIGEST_PATTERN = /^sha256:[0-9a-f]{64}$/;

const FORBIDDEN_REFERENCE_KEYS = [
    'url',
    'artifact_url',
    'artifactUrl',
    'download_url',
    'downloadUrl',
    'link',
    'href',
    'src',
    'token',
    'signed_url',
    'signedUrl',
    'expires_at',
    'expiresAt',
    'expiresAtMs',
] as const;

/**
 * A local artifact identity. Deliberately carries no URL, token or expiry:
 * bytes are read through the existing browser/companion artifact path only.
 */
export interface ArtifactReference {
    artifactId: string;
    taskId?: string;
    name?: string;
    mediaType?: string;
    sizeBytes?: number;
    range?: { start: number; end: number };
}

export interface ArtifactReader {
    readArtifact(input: ReadArtifactInput): Promise<ArtifactContent>;
}

export interface ArtifactPanelProps {
    client?: ArtifactReader | null;
    reference?: ArtifactReference | null;
}

type ReferenceValidation =
    | { ok: true; reference: ArtifactReference }
    | { ok: false; message: string };

type LoadState =
    | { phase: 'idle' }
    | { phase: 'loading' }
    | { phase: 'ready'; content: ArtifactContent; requestedArtifactId: string }
    | { phase: 'unavailable'; message: string }
    | { phase: 'error'; message: string };

let nextArtifactPanelId = 0;

function isRecord(value: unknown): value is Record<string, unknown> {
    return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function isSafeInteger(value: unknown): value is number {
    return typeof value === 'number' && Number.isSafeInteger(value) && value >= 0;
}

export function validateArtifactReference(value: unknown): ReferenceValidation {
    if (!isRecord(value)) {
        return {
            ok: false,
            message: 'Provide a local artifact identity: art_ followed by 64 lowercase hex characters.',
        };
    }
    for (const key of FORBIDDEN_REFERENCE_KEYS) {
        if (key in value) {
            return {
                ok: false,
                message: 'Artifact links, tokens and expiry fields are not used. Provide a local art_<64 lowercase hex> identity instead.',
            };
        }
    }
    const artifactId = value.artifactId;
    if (typeof artifactId !== 'string' || !ARTIFACT_ID_PATTERN.test(artifactId)) {
        return {
            ok: false,
            message: 'Artifact ID must be art_ followed by exactly 64 lowercase hex characters.',
        };
    }
    if (value.taskId !== undefined && (typeof value.taskId !== 'string' || value.taskId.length === 0)) {
        return { ok: false, message: 'Artifact task identity is malformed.' };
    }
    if (value.name !== undefined && (typeof value.name !== 'string' || value.name.length === 0 || value.name.length > 256)) {
        return { ok: false, message: 'Artifact name is malformed.' };
    }
    if (value.mediaType !== undefined && (typeof value.mediaType !== 'string' || value.mediaType.length === 0 || value.mediaType.length > 256)) {
        return { ok: false, message: 'Artifact media type is malformed.' };
    }
    if (value.sizeBytes !== undefined && !isSafeInteger(value.sizeBytes)) {
        return { ok: false, message: 'Artifact size is malformed.' };
    }
    let range: { start: number; end: number } | undefined;
    if (value.range !== undefined) {
        if (!isRecord(value.range)) {
            return { ok: false, message: 'Artifact range is malformed.' };
        }
        const { start, end } = value.range;
        if (!isSafeInteger(start) || !isSafeInteger(end) || start > end) {
            return { ok: false, message: 'Artifact range is malformed.' };
        }
        if (end - start > MAX_ARTIFACT_RANGE_BYTES) {
            return { ok: false, message: `Artifact ranges are limited to ${MAX_ARTIFACT_RANGE_BYTES} bytes.` };
        }
        range = { start, end };
    }
    return {
        ok: true,
        reference: {
            artifactId,
            ...(value.taskId === undefined ? {} : { taskId: value.taskId as string }),
            ...(value.name === undefined ? {} : { name: value.name as string }),
            ...(value.mediaType === undefined ? {} : { mediaType: value.mediaType as string }),
            ...(value.sizeBytes === undefined ? {} : { sizeBytes: value.sizeBytes as number }),
            ...(range === undefined ? {} : { range }),
        },
    };
}

async function verifyArtifactContent(
    content: ArtifactContent,
    reference: ArtifactReference | null,
): Promise<{ ok: true } | { ok: false; message: string }> {
    const bytes = artifactBytes(content.bytes);
    if (bytes === null) {
        return { ok: false, message: 'Artifact bytes are missing; failed verification.' };
    }
    if (!isSafeInteger(content.sizeBytes) || content.sizeBytes !== bytes.byteLength) {
        return { ok: false, message: 'Artifact size does not match its bytes; failed verification.' };
    }
    if (typeof content.mediaType !== 'string' || content.mediaType.length === 0) {
        return { ok: false, message: 'Artifact media type is missing; failed verification.' };
    }
    if (content.artifactId !== undefined) {
        if (typeof content.artifactId !== 'string' || !ARTIFACT_ID_PATTERN.test(content.artifactId)) {
            return { ok: false, message: 'Artifact identity is malformed; failed verification.' };
        }
        if (reference !== null && content.artifactId !== reference.artifactId) {
            return { ok: false, message: 'Artifact identity does not match the requested artifact; failed verification.' };
        }
    }
    if (content.artifactSizeBytes !== undefined) {
        if (!isSafeInteger(content.artifactSizeBytes) || content.artifactSizeBytes < content.sizeBytes) {
            return { ok: false, message: 'Artifact total size is malformed; failed verification.' };
        }
    }
    if (content.range !== undefined) {
        const { start, end } = content.range;
        if (!isSafeInteger(start) || !isSafeInteger(end) || start > end || end - start !== content.sizeBytes) {
            return { ok: false, message: 'Artifact range does not match its bytes; failed verification.' };
        }
    }
    if (reference?.range !== undefined) {
        if (
            content.range === undefined ||
            content.range.start !== reference.range.start ||
            content.range.end !== reference.range.end
        ) {
            return { ok: false, message: 'Artifact range does not match the requested range; failed verification.' };
        }
        if (content.artifactSizeBytes === undefined || content.artifactSizeBytes < reference.range.end) {
            return { ok: false, message: 'Artifact total size is missing or outside the requested range; failed verification.' };
        }
    } else if (content.range !== undefined) {
        if (
            content.artifactSizeBytes === undefined ||
            content.range.start !== 0 ||
            content.range.end !== content.artifactSizeBytes ||
            content.range.end !== content.sizeBytes
        ) {
            return { ok: false, message: 'Artifact returned an unexpected partial range; failed verification.' };
        }
    }
    const digest = await sha256Bytes(bytes);
    if (typeof content.digest !== 'string' || content.digest !== digest) {
        return { ok: false, message: 'Artifact digest does not match its bytes; failed verification.' };
    }
    if (content.wholeDigest !== undefined) {
        if (typeof content.wholeDigest !== 'string' || !DIGEST_PATTERN.test(content.wholeDigest)) {
            return { ok: false, message: 'Artifact whole digest is malformed; failed verification.' };
        }
        const wholeRead =
            content.range === undefined ||
            (content.range.start === 0 &&
                content.artifactSizeBytes !== undefined &&
                content.range.end === content.artifactSizeBytes);
        if (wholeRead && content.wholeDigest !== digest) {
            return { ok: false, message: 'Artifact whole digest does not match its bytes; failed verification.' };
        }
    }
    return { ok: true };
}

function failureMessage(error: unknown): { unavailable: boolean; message: string } {
    if (error instanceof AgentClientError) {
        if (error.code === 'host_unavailable' || error.code === 'runtime_unavailable') {
            return {
                unavailable: true,
                message: 'Artifact bytes require the local browser store or the local companion. No remote URL or token is used.',
            };
        }
        if (error.code === 'snapshot_corrupt' || error.code === 'invalid_event') {
            return {
                unavailable: false,
                message: 'Artifact bytes failed verification and were not previewed.',
            };
        }
        if (error.code === 'limit_exceeded' || error.code === 'invalid_arguments') {
            return {
                unavailable: false,
                message: `Artifact reference or range was refused: ${error.message}`,
            };
        }
        return {
            unavailable: false,
            message: `Artifact read failed: ${error.message}`,
        };
    }
    return {
        unavailable: false,
        message: `Artifact read failed: ${error instanceof Error ? error.message : String(error)}`,
    };
}

function contentMetadata(content: ArtifactContent, requestedArtifactId: string) {
    const reportedArtifactId = content.artifactId;
    return <dl class="artifact-meta" data-artifact-metadata>
        <dt>Reported artifact</dt><dd style={{ overflowWrap: 'anywhere' }}><code>{reportedArtifactId ?? 'not reported by source'}</code></dd>
        <dt>Requested artifact</dt><dd style={{ overflowWrap: 'anywhere' }}><code>{requestedArtifactId}</code></dd>
        <dt>Media type</dt><dd>{content.mediaType}</dd>
        <dt>Size</dt><dd>
            {content.range === undefined || content.artifactSizeBytes === undefined
                ? `${content.sizeBytes} bytes`
                : `${content.sizeBytes} bytes ${content.range.start}–${content.range.end} of ${content.artifactSizeBytes}`}
        </dd>
        <dt>SHA-256</dt><dd style={{ overflowWrap: 'anywhere' }}><code>{content.digest}</code></dd>
        {content.wholeDigest !== undefined && content.wholeDigest !== content.digest && <>
            <dt>Whole SHA-256</dt><dd style={{ overflowWrap: 'anywhere' }}><code>{content.wholeDigest}</code></dd>
        </>}
    </dl>;
}

export function ArtifactPanel({ client = null, reference = null }: ArtifactPanelProps) {
    const [id] = useState(() => `artifact-panel-${++nextArtifactPanelId}`);
    const headingId = `${id}-heading`;
    const inputId = `${id}-identity`;
    const helpId = `${id}-help`;
    const [value, setValue] = useState(() => reference?.artifactId ?? '');
    const [state, setState] = useState<LoadState>({ phase: 'idle' });

    useEffect(() => {
        if (reference === null) return;
        const validation = validateArtifactReference(reference);
        if (!validation.ok) {
            setValue('');
            setState({ phase: 'error', message: validation.message });
            return;
        }
        setValue(validation.reference.artifactId);
        setState({ phase: 'idle' });
    }, [reference]);

    async function load(event: Event): Promise<void> {
        event.preventDefault();
        const form = event.currentTarget as HTMLFormElement | null;
        const submitted =
            form?.querySelector<HTMLInputElement>('input[aria-label="Artifact ID"]')?.value ??
            value;
        if (submitted !== value) setValue(submitted);
        const requested: Record<string, unknown> = { artifactId: submitted.trim() };
        const referenceValidation = validateArtifactReference(reference);
        if (
            referenceValidation.ok &&
            referenceValidation.reference.artifactId === requested.artifactId
        ) {
            if (referenceValidation.reference.taskId !== undefined) {
                requested.taskId = referenceValidation.reference.taskId;
            }
            if (referenceValidation.reference.range !== undefined) {
                requested.range = referenceValidation.reference.range;
            }
        }
        const validation = validateArtifactReference(requested);
        if (!validation.ok) {
            setValue('');
            setState({ phase: 'error', message: validation.message });
            return;
        }
        if (client === null) {
            setState({
                phase: 'unavailable',
                message: 'Artifact bytes require the local browser store or the local companion. No remote URL or token is used.',
            });
            return;
        }
        setState({ phase: 'loading' });
        const input: ReadArtifactInput = {
            artifactId: validation.reference.artifactId,
            ...(validation.reference.taskId === undefined ? {} : { taskId: validation.reference.taskId }),
            ...(validation.reference.range === undefined
                ? {}
                : { start: validation.reference.range.start, end: validation.reference.range.end }),
        };
        try {
            const content = await client.readArtifact(input);
            const verified = await verifyArtifactContent(content, validation.reference);
            if (!verified.ok) {
                setState({ phase: 'error', message: verified.message });
                return;
            }
            setState({
                phase: 'ready',
                content,
                requestedArtifactId: validation.reference.artifactId,
            });
        } catch (error) {
            const failure = failureMessage(error);
            setState(
                failure.unavailable
                    ? { phase: 'unavailable', message: failure.message }
                    : { phase: 'error', message: failure.message },
            );
        }
    }

    return <section class="desktop-panel artifact-panel" aria-labelledby={headingId} data-artifact-panel>
        <header class="panel-heading">
            <div>
                <h1 id={headingId}>Artifacts</h1>
                <p>Digest-verified local bytes only. Artifacts are read through the browser store or local companion; this preview never uses an artifact URL, token or remote subresource.</p>
            </div>
            <span class="panel-icon" aria-hidden="true"><Icon name="workspace" /></span>
        </header>
        <form class="artifact-form" aria-labelledby={headingId} onSubmit={(event) => { void load(event); }}>
            <label for={inputId}>Artifact ID</label>
            <input
                id={inputId}
                aria-label="Artifact ID"
                value={value}
                placeholder="art_…"
                autocomplete="off"
                spellcheck={false}
                aria-describedby={helpId}
                onInput={(event) => setValue(event.currentTarget.value)}
            />
            <p id={helpId} class="form-hint">Use the full art_ identity followed by 64 lowercase hex characters. URLs, tokens and expired links are refused and are not retained.</p>
            <button type="submit" disabled={state.phase === 'loading'}>
                {state.phase === 'loading' ? 'Verifying…' : 'Load artifact'}
            </button>
        </form>
        {state.phase === 'loading' && <p role="status" class="artifact-loading">Verifying artifact bytes…</p>}
        {state.phase === 'unavailable' && <p class="panel-summary" role="status" data-artifact-unavailable>{state.message}</p>}
        {state.phase === 'error' && <p class="diff-review-error" role="alert" data-artifact-error>{state.message}</p>}
        {state.phase === 'ready' && <div data-artifact-content>
            {contentMetadata(state.content, state.requestedArtifactId)}
            <ArtifactPreview content={state.content} />
        </div>}
    </section>;
}
