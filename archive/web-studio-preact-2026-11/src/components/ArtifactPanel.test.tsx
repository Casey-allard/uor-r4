import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest';

import {
    AgentClient,
    type AgentTransport,
    type ArtifactContent,
    type ReadArtifactInput,
} from '../agent/client.js';
import {
    HOST_RESULT_SCHEMA,
    HostClient,
    sha256Bytes,
    sha256Canonical,
    type HostInvokeResponse,
    type HostSubscription,
    type HostToolIntent,
    type HostToolResult,
    type HostTransport,
    type HostTransportCapabilities,
} from '../agent/hostClient.js';
import { AgentTransportHostTransport } from '../agent/transports.js';

const FULL64 = `art_${'a'.repeat(64)}`;
const OTHER64 = `art_${'b'.repeat(64)}`;
const ALL_FALSE: HostTransportCapabilities = {
    kind: 'bridge',
    hostAvailable: true,
    streaming: 'none',
    tree: false,
    changes: false,
    readArtifact: true,
    terminal: false,
    git: false,
    mcp: false,
    approvals: false,
};

let host: HTMLDivElement;

beforeEach(() => {
    host = document.createElement('div');
    document.body.appendChild(host);
    localStorage.clear();
});

afterEach(() => {
    render(null, host);
    host.remove();
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    localStorage.clear();
});

function base64(bytes: Uint8Array): string {
    let binary = '';
    for (const byte of bytes) binary += String.fromCharCode(byte);
    return globalThis.btoa(binary);
}

class ArtifactHostTransport implements HostTransport {
    readonly kind = 'bridge' as const;
    readonly supportsStream = false;
    readonly intents: HostToolIntent[] = [];
    readonly output: Record<string, unknown>;

    constructor(output: Record<string, unknown>) {
        this.output = output;
    }

    async capabilities(): Promise<HostTransportCapabilities> {
        return ALL_FALSE;
    }

    async invoke(intent: HostToolIntent): Promise<HostInvokeResponse> {
        this.intents.push(intent);
        const result: HostToolResult = {
            schema: HOST_RESULT_SCHEMA,
            intent_id: intent.intent_id,
            task_id: intent.task_id,
            run_id: intent.run_id,
            generation: intent.generation,
            tool_call_id: intent.tool_call_id,
            request_scope: intent.request_scope ?? 'task',
            status: 'ok',
            output: this.output,
            output_digest: await sha256Canonical(this.output),
            output_preview: 'artifact',
            occurred_at_ms: 1_800_000_000_000,
        };
        return result;
    }

    subscribe(): HostSubscription {
        return { close: () => undefined, cursor: () => null };
    }

    async approve(input: { approvalId: string; decision: 'approved' | 'denied' }) {
        return input;
    }
}

async function readThroughHost(
    output: Record<string, unknown>,
    input: ReadArtifactInput,
): Promise<ArtifactContent> {
    const transport = new ArtifactHostTransport(output);
    const client = new AgentClient({
        hostClient: new HostClient({ transport }),
    });
    return client.readArtifact(input);
}

async function bridgeArtifactRead(
    content: ArtifactContent,
    requestedArtifactId = FULL64,
): Promise<{ result: HostToolResult; client: AgentClient }> {
    const bridge = new AgentTransportHostTransport({
        readArtifact: async () => content,
        hostCapabilities: async () => ({
            tree: false,
            changes: false,
            readArtifact: true,
            terminal: false,
            git: false,
            mcp: false,
            approvals: false,
        }),
    } as unknown as AgentTransport);
    const hostClient = new HostClient({ transport: bridge });
    const result = await hostClient.execute({
        tool: 'artifact.read',
        arguments: {
            artifact_id: requestedArtifactId,
            start: 2,
            end: 5,
        },
    });
    return { result, client: new AgentClient({ hostClient }) };
}

async function renderArtifactPanel(props: {
    client: { readArtifact(input: ReadArtifactInput): Promise<ArtifactContent> } | null;
    reference?: unknown;
}): Promise<void> {
    const { ArtifactPanel } = await import('./ArtifactPanel.js');
    act(() => {
        render(<ArtifactPanel client={props.client as never} reference={props.reference as never} />, host);
    });
}

async function submitArtifactForm(): Promise<void> {
    const form = host.querySelector('form');
    if (form === null) throw new Error('artifact form is missing');
    await act(async () => {
        form.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }));
        await Promise.resolve();
    });
}

function artifactContent(overrides: Partial<ArtifactContent> = {}): ArtifactContent {
    const bytes = new TextEncoder().encode('hello');
    return {
        path: 'artifact.txt',
        mediaType: 'text/plain',
        sizeBytes: bytes.byteLength,
        digest: `sha256:${'0'.repeat(64)}`,
        bytes,
        ...overrides,
    };
}

describe('ArtifactClient authority and bridge propagation', () => {
    test('artifact_read_preserves_host_artifact_id_and_whole_metadata', async () => {
        const fullBytes = new TextEncoder().encode('0123456789');
        const returned = fullBytes.slice(2, 5);
        const digest = await sha256Bytes(returned);
        const wholeDigest = await sha256Bytes(fullBytes);
        const content = await readThroughHost(
            {
                artifact_id: FULL64,
                path: 'data.bin',
                media_type: 'application/octet-stream',
                size_bytes: returned.byteLength,
                artifact_size_bytes: fullBytes.byteLength,
                digest,
                whole_digest: wholeDigest,
                bytes_base64: base64(returned),
                range: { start: 2, end: 5 },
            },
            { artifactId: FULL64, start: 2, end: 5 },
        );

        expect(content).toMatchObject({
            artifactId: FULL64,
            sizeBytes: 3,
            artifactSizeBytes: 10,
            wholeDigest,
            range: { start: 2, end: 5 },
        });
        expect(content.digest).toBe(digest);
        expect(new TextDecoder().decode(content.bytes)).toBe('234');
    });

    test('artifact_read_rejects_mismatched_host_identity', async () => {
        const bytes = new TextEncoder().encode('hello');
        const digest = await sha256Bytes(bytes);
        await expect(
            readThroughHost(
                {
                    artifact_id: OTHER64,
                    path: 'artifact.txt',
                    media_type: 'text/plain',
                    size_bytes: bytes.byteLength,
                    digest,
                    bytes_base64: base64(bytes),
                },
                { artifactId: FULL64 },
            ),
        ).rejects.toMatchObject({ code: 'invalid_event' });
    });

    test('artifact_bridge_propagates_range_and_whole_metadata', async () => {
        const fullBytes = new TextEncoder().encode('0123456789');
        const returned = fullBytes.slice(2, 5);
        const digest = await sha256Bytes(returned);
        const wholeDigest = await sha256Bytes(fullBytes);
        const calls: ReadArtifactInput[] = [];
        const bridgeContent: ArtifactContent = {
            artifactId: FULL64,
            path: 'data.bin',
            mediaType: 'application/octet-stream',
            sizeBytes: returned.byteLength,
            artifactSizeBytes: fullBytes.byteLength,
            digest,
            wholeDigest,
            range: { start: 2, end: 5 },
            bytes: returned,
        };
        const bridge = new AgentTransportHostTransport({
            readArtifact: async (input: ReadArtifactInput) => {
                calls.push(input);
                return bridgeContent;
            },
            hostCapabilities: async () => ({
                tree: false,
                changes: false,
                readArtifact: true,
                terminal: false,
                git: false,
                mcp: false,
                approvals: false,
            }),
        } as unknown as AgentTransport);
        const client = new AgentClient({
            hostClient: new HostClient({ transport: bridge }),
        });

        const content = await client.readArtifact({
            artifactId: FULL64,
            start: 2,
            end: 5,
        });

        expect(calls[0]).toMatchObject({
            artifactId: FULL64,
            start: 2,
            end: 5,
        });
        expect(content).toMatchObject({
            artifactId: FULL64,
            sizeBytes: 3,
            artifactSizeBytes: 10,
            wholeDigest,
            range: { start: 2, end: 5 },
        });
    });

    test('bridge_omits_artifact_reference_when_ranged_whole_digest_is_missing', async () => {
        const returned = new TextEncoder().encode('234');
        const digest = await sha256Bytes(returned);
        const { result, client } = await bridgeArtifactRead({
            artifactId: FULL64,
            path: 'data.bin',
            mediaType: 'application/octet-stream',
            sizeBytes: returned.byteLength,
            artifactSizeBytes: 10,
            digest,
            range: { start: 2, end: 5 },
            bytes: returned,
        });
        const output = result.output as Record<string, unknown>;

        expect(result.status).toBe('ok');
        expect(output).toMatchObject({
            artifact_id: FULL64,
            artifact_size_bytes: 10,
            size_bytes: 3,
            digest,
            range: { start: 2, end: 5 },
        });
        expect(output).not.toHaveProperty('whole_digest');
        expect(result.artifact).toBeUndefined();

        const content = await client.readArtifact({
            artifactId: FULL64,
            start: 2,
            end: 5,
        });
        expect(content).toMatchObject({
            sizeBytes: 3,
            artifactSizeBytes: 10,
            range: { start: 2, end: 5 },
        });
        expect(content.wholeDigest).toBeUndefined();
    });

    test('bridge_does_not_echo_requested_identity_when_source_omits_it', async () => {
        const bytes = new TextEncoder().encode('hello');
        const digest = await sha256Bytes(bytes);
        const { result, client } = await bridgeArtifactRead({
            path: 'legacy.txt',
            mediaType: 'text/plain',
            sizeBytes: bytes.byteLength,
            digest,
            bytes,
        });
        const output = result.output as Record<string, unknown>;

        expect(result.status).toBe('ok');
        expect(output).not.toHaveProperty('artifact_id');
        expect(result.artifact).toBeUndefined();

        const content = await client.readArtifact({ artifactId: FULL64 });
        expect(content.artifactId).toBeUndefined();
        expect(new TextDecoder().decode(content.bytes)).toBe('hello');
    });

    test('bridge_preserves_reported_mismatched_identity_and_client_refuses', async () => {
        const bytes = new TextEncoder().encode('hello');
        const digest = await sha256Bytes(bytes);
        const { result, client } = await bridgeArtifactRead({
            artifactId: OTHER64,
            path: 'artifact.txt',
            mediaType: 'text/plain',
            sizeBytes: bytes.byteLength,
            digest,
            bytes,
        });
        const output = result.output as Record<string, unknown>;

        expect(output).toMatchObject({ artifact_id: OTHER64 });
        expect(result.artifact?.artifact_id).toBe(OTHER64);
        await expect(
            client.readArtifact({ artifactId: FULL64 }),
        ).rejects.toMatchObject({ code: 'invalid_event' });
    });

    test('bridge_preserves_fully_known_ranged_reference', async () => {
        const fullBytes = new TextEncoder().encode('0123456789');
        const returned = fullBytes.slice(2, 5);
        const digest = await sha256Bytes(returned);
        const wholeDigest = await sha256Bytes(fullBytes);
        const { result } = await bridgeArtifactRead({
            artifactId: FULL64,
            path: 'data.bin',
            mediaType: 'application/octet-stream',
            sizeBytes: returned.byteLength,
            artifactSizeBytes: fullBytes.byteLength,
            digest,
            wholeDigest,
            range: { start: 2, end: 5 },
            bytes: returned,
        });
        const output = result.output as Record<string, unknown>;

        expect(output).toMatchObject({
            artifact_id: FULL64,
            artifact_size_bytes: 10,
            whole_digest: wholeDigest,
            range: { start: 2, end: 5 },
        });
        expect(result.artifact).toMatchObject({
            artifact_id: FULL64,
            digest: wholeDigest,
            size_bytes: 10,
        });
    });

    test('bridge_preserves_whole_read_reference', async () => {
        const bytes = new TextEncoder().encode('hello');
        const digest = await sha256Bytes(bytes);
        const { result } = await bridgeArtifactRead({
            artifactId: FULL64,
            path: 'whole.txt',
            mediaType: 'text/plain',
            sizeBytes: bytes.byteLength,
            digest,
            bytes,
        });
        const output = result.output as Record<string, unknown>;

        expect(output).toMatchObject({
            artifact_id: FULL64,
            artifact_size_bytes: bytes.byteLength,
            whole_digest: digest,
            size_bytes: bytes.byteLength,
        });
        expect(output).not.toHaveProperty('range');
        expect(result.artifact).toMatchObject({
            artifact_id: FULL64,
            digest,
            media_type: 'text/plain',
            size_bytes: bytes.byteLength,
        });
    });

    test('artifact_read_prefers_browser_bytes_before_host', async () => {
        const transport = new ArtifactHostTransport({});
        const client = new AgentClient({
            hostClient: new HostClient({ transport }),
        });
        const bytes = new TextEncoder().encode('browser artifact');
        const digest = await sha256Bytes(bytes);
        client.registerLocalFile({
            path: 'local.txt',
            mediaType: 'text/plain',
            bytes,
            digest,
            artifactId: FULL64,
        });

        const content = await client.readArtifact({ artifactId: FULL64 });

        expect(content.artifactId).toBe(FULL64);
        expect(new TextDecoder().decode(content.bytes)).toBe('browser artifact');
        expect(transport.intents).toHaveLength(0);
    });
});

describe('ArtifactPanel digest-bound reads', () => {
    test('artifact_read_verifies_digest', async () => {
        const readArtifact = vi.fn(async () => artifactContent());
        await renderArtifactPanel({
            client: { readArtifact },
            reference: { artifactId: FULL64 },
        });

        expect(host.querySelector<HTMLInputElement>('input[aria-label="Artifact ID"]')?.value).toBe(FULL64);
        expect(readArtifact).not.toHaveBeenCalled();

        await submitArtifactForm();
        await vi.waitFor(() => {
            expect(host.querySelector('[data-artifact-error]')).not.toBeNull();
        });

        expect(readArtifact).toHaveBeenCalledOnce();
        expect(host.querySelector('[data-artifact-error]')?.textContent).toMatch(/digest/i);
        expect(host.querySelector('[data-artifact-preview]')).toBeNull();
        expect(host.textContent).toContain('failed verification');
    });

    test('artifact_panel_refuses_corrupt_size_and_media_metadata', async () => {
        const sizeMismatch = vi.fn(async () =>
            artifactContent({ sizeBytes: 99, digest: `sha256:${'1'.repeat(64)}` }),
        );
        await renderArtifactPanel({
            client: { readArtifact: sizeMismatch },
            reference: { artifactId: FULL64 },
        });
        await submitArtifactForm();
        await vi.waitFor(() => {
            expect(host.querySelector('[data-artifact-error]')?.textContent).toMatch(/size/i);
        });
        expect(host.querySelector('[data-artifact-preview]')).toBeNull();

        render(null, host);
        const identityMismatch = vi.fn(async () =>
            artifactContent({ artifactId: OTHER64 }),
        );
        await renderArtifactPanel({
            client: { readArtifact: identityMismatch },
            reference: { artifactId: FULL64 },
        });
        await submitArtifactForm();
        await vi.waitFor(() => {
            expect(host.querySelector('[data-artifact-error]')?.textContent).toMatch(/identity/i);
        });
        expect(host.querySelector('[data-artifact-preview]')).toBeNull();
    });

    test('artifact_panel_request_is_explicit_and_bounded', async () => {
        const bytes = new TextEncoder().encode('234');
        const digest = await sha256Bytes(bytes);
        const readArtifact = vi.fn(async (input: ReadArtifactInput) => ({
            artifactId: FULL64,
            path: 'data.bin',
            mediaType: 'text/plain',
            sizeBytes: bytes.byteLength,
            artifactSizeBytes: 10,
            digest,
            wholeDigest: `sha256:${'c'.repeat(64)}`,
            range: { start: input.start ?? 0, end: input.end ?? 0 },
            bytes,
        }));
        await renderArtifactPanel({
            client: { readArtifact },
            reference: { artifactId: FULL64, range: { start: 2, end: 5 } },
        });

        expect(readArtifact).not.toHaveBeenCalled();
        await submitArtifactForm();
        await vi.waitFor(() => {
            expect(host.querySelector('[data-artifact-preview]')).not.toBeNull();
        });
        expect(readArtifact).toHaveBeenCalledWith({
            artifactId: FULL64,
            start: 2,
            end: 5,
        });
        expect(host.textContent).toContain('bytes 2–5 of 10');
        expect(host.textContent).toContain(digest);

        render(null, host);
        const overCap = vi.fn(async () => artifactContent());
        await renderArtifactPanel({
            client: { readArtifact: overCap },
            reference: {
                artifactId: FULL64,
                range: { start: 0, end: 1024 * 1024 + 1 },
            },
        });
        await submitArtifactForm();
        await vi.waitFor(() => {
            expect(host.querySelector('[data-artifact-error]')).not.toBeNull();
        });
        expect(overCap).not.toHaveBeenCalled();
        expect(host.querySelector('[data-artifact-preview]')).toBeNull();
    });

    test('artifact_panel_separates_requested_and_reported_identity', async () => {
        const bytes = new TextEncoder().encode('hello');
        const digest = await sha256Bytes(bytes);
        const missingReported = vi.fn(async () => ({
            path: 'artifact.txt',
            mediaType: 'text/plain',
            sizeBytes: bytes.byteLength,
            digest,
            bytes,
        }));
        await renderArtifactPanel({
            client: { readArtifact: missingReported },
            reference: { artifactId: FULL64 },
        });
        await submitArtifactForm();
        await vi.waitFor(() => {
            expect(host.querySelector('[data-artifact-metadata]')).not.toBeNull();
        });
        const missingText = host.querySelector('[data-artifact-metadata]')?.textContent ?? '';
        expect(missingText).toContain('Reported artifact');
        expect(missingText).toContain('not reported by source');
        expect(missingText).toContain('Requested artifact');
        expect(missingText).toContain(FULL64);

        render(null, host);
        const reported = vi.fn(async () => ({
            artifactId: FULL64,
            path: 'artifact.txt',
            mediaType: 'text/plain',
            sizeBytes: bytes.byteLength,
            digest,
            bytes,
        }));
        await renderArtifactPanel({
            client: { readArtifact: reported },
            reference: { artifactId: FULL64 },
        });
        await submitArtifactForm();
        await vi.waitFor(() => {
            expect(host.querySelector('[data-artifact-metadata]')).not.toBeNull();
        });
        const reportedText = host.querySelector('[data-artifact-metadata]')?.textContent ?? '';
        expect(reportedText).toContain('Reported artifact');
        expect(reportedText).toContain('Requested artifact');
        expect(reportedText).toContain(FULL64);
    });

    test('expired_artifact_link_is_not_reused', async () => {
        const readArtifact = vi.fn(async () => artifactContent());
        await renderArtifactPanel({
            client: { readArtifact },
            reference: {
                artifactId: FULL64,
                url: 'https://example.com/artifact?sig=expired',
                token: 'secret-token',
                expiresAtMs: 1,
            },
        });

        await vi.waitFor(() => {
            expect(host.querySelector('[data-artifact-error]')).not.toBeNull();
        });
        expect(readArtifact).not.toHaveBeenCalled();
        expect(host.querySelector('[data-artifact-preview]')).toBeNull();
        expect(host.querySelector('a[href]')).toBeNull();
        expect(host.textContent).not.toContain('https://example.com/artifact');
        expect(host.textContent).not.toContain('secret-token');
        expect(localStorage.length).toBe(0);

        const input = host.querySelector<HTMLInputElement>('input[aria-label="Artifact ID"]');
        expect(input?.value).toBe('');

        await submitArtifactForm();
        expect(readArtifact).not.toHaveBeenCalled();
        expect(host.querySelector('[data-artifact-preview]')).toBeNull();
    });

    test('invalid_artifact_identifiers_are_refused_without_a_read', async () => {
        const readArtifact = vi.fn(async () => artifactContent());
        await renderArtifactPanel({
            client: { readArtifact },
            reference: null,
        });
        const input = host.querySelector<HTMLInputElement>('input[aria-label="Artifact ID"]');
        expect(input).not.toBeNull();
        await act(async () => {
            input!.value = 'https://example.com/artifact?token=leak';
            input!.dispatchEvent(new Event('input', { bubbles: true }));
            await Promise.resolve();
        });
        await submitArtifactForm();
        await vi.waitFor(() => {
            expect(host.querySelector('[data-artifact-error]')).not.toBeNull();
        });
        expect(readArtifact).not.toHaveBeenCalled();
        expect(host.querySelector('[data-artifact-preview]')).toBeNull();
        expect(host.querySelector<HTMLInputElement>('input[aria-label="Artifact ID"]')?.value).toBe('');
        expect(host.textContent).not.toContain('token=leak');
    });

    test('typed_identity_is_not_cleared_by_a_null_reference', async () => {
        const readArtifact = vi.fn(async () => artifactContent());
        await renderArtifactPanel({
            client: { readArtifact },
            reference: null,
        });
        const input = host.querySelector<HTMLInputElement>('input[aria-label="Artifact ID"]');
        expect(input).not.toBeNull();
        await act(async () => {
            input!.value = FULL64;
            input!.dispatchEvent(new Event('input', { bubbles: true }));
            await Promise.resolve();
        });
        expect(input?.value).toBe(FULL64);
        expect(readArtifact).not.toHaveBeenCalled();
    });

    test('host_unavailable_is_reported_honestly', async () => {
        const { AgentClientError } = await import('../agent/client.js');
        const readArtifact = vi.fn(async () => {
            throw new AgentClientError(
                'host_unavailable',
                'the local host companion is not connected',
            );
        });
        await renderArtifactPanel({
            client: { readArtifact },
            reference: { artifactId: FULL64 },
        });
        await submitArtifactForm();
        await vi.waitFor(() => {
            expect(host.querySelector('[data-artifact-unavailable]')).not.toBeNull();
        });
        expect(host.textContent).toMatch(/companion/i);
        expect(host.querySelector('[data-artifact-preview]')).toBeNull();
    });
});

describe('ArtifactPreview inertness', () => {
    async function renderPreview(content: ArtifactContent): Promise<void> {
        const { ArtifactPreview } = await import('./ArtifactPreview.js');
        act(() => {
            render(<ArtifactPreview content={content} />, host);
        });
    }

    test('artifact_preview_is_inert', async () => {
        const payload = '<script>alert(1)</script><img src=x onerror=alert(2)></pre>';
        const hostile = new TextEncoder().encode(payload);
        await renderPreview({
            path: 'note.txt',
            mediaType: 'text/plain',
            sizeBytes: hostile.byteLength,
            digest: await sha256Bytes(hostile),
            bytes: hostile,
        });
        const pre = host.querySelector('pre[data-artifact-preview="text"]');
        expect(pre?.textContent).toBe(payload);
        expect(host.querySelector('script')).toBeNull();
        expect(host.querySelector('img')).toBeNull();
        expect(host.querySelector('iframe')).toBeNull();
        expect(host.querySelector('object')).toBeNull();
        expect(host.querySelector('embed')).toBeNull();

        render(null, host);
        const html = new TextEncoder().encode('<h1>active</h1>');
        await renderPreview({
            path: 'note.html',
            mediaType: 'text/html',
            sizeBytes: html.byteLength,
            digest: await sha256Bytes(html),
            bytes: html,
        });
        expect(host.querySelector('h1')).toBeNull();
        expect(host.querySelector('[data-artifact-preview="unsupported"]')?.textContent).toMatch(
            /not rendered/i,
        );

        render(null, host);
        const svg = new TextEncoder().encode('<svg onload="alert(1)"></svg>');
        await renderPreview({
            path: 'image.svg',
            mediaType: 'image/svg+xml',
            sizeBytes: svg.byteLength,
            digest: await sha256Bytes(svg),
            bytes: svg,
        });
        expect(host.querySelector('svg')).toBeNull();
        expect(host.querySelector('[data-artifact-preview="unsupported"]')).not.toBeNull();

        render(null, host);
        await renderPreview({
            path: 'bad.txt',
            mediaType: 'text/plain',
            sizeBytes: 2,
            digest: `sha256:${'0'.repeat(64)}`,
            bytes: new Uint8Array([0xc3, 0x28]),
        });
        expect(host.querySelector('[data-artifact-preview="unsupported"]')?.textContent).toMatch(
            /encoding/i,
        );
    });

    test('artifact_preview_rejects_oversize_and_media_mismatch', async () => {
        const oversized = new Uint8Array(64 * 1024 + 1).fill(0x61);
        await renderPreview({
            path: 'large.txt',
            mediaType: 'text/plain',
            sizeBytes: oversized.byteLength,
            digest: `sha256:${'0'.repeat(64)}`,
            bytes: oversized,
        });
        expect(host.querySelector('[data-artifact-preview="unsupported"]')?.textContent).toMatch(
            /size/i,
        );

        render(null, host);
        const jpeg = new Uint8Array([0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10]);
        await renderPreview({
            path: 'fake.png',
            mediaType: 'image/png',
            sizeBytes: jpeg.byteLength,
            digest: `sha256:${'0'.repeat(64)}`,
            bytes: jpeg,
        });
        expect(host.querySelector('img')).toBeNull();
        expect(host.querySelector('[data-artifact-preview="unsupported"]')?.textContent).toMatch(
            /media type/i,
        );
    });

    test('artifact_preview_renders_only_verified_raster_bytes_locally', async () => {
        const png = new Uint8Array([
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a,
            0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
        ]);
        await renderPreview({
            path: 'pixel.png',
            mediaType: 'image/png',
            sizeBytes: png.byteLength,
            digest: `sha256:${'0'.repeat(64)}`,
            bytes: png,
        });
        const image = host.querySelector<HTMLImageElement>('img[data-artifact-preview="image"]');
        expect(image).not.toBeNull();
        expect(image?.getAttribute('src')?.startsWith('data:image/png;base64,')).toBe(true);
        expect(image?.getAttribute('src')).not.toContain('http');
        expect(host.querySelector('iframe')).toBeNull();
    });
});

describe('Task 22 prohibited controls', () => {
    test('no_checkpoint_restore_control_is_rendered', async () => {
        const { PrPanel } = await import('./PrPanel.js');
        const { ArtifactPanel } = await import('./ArtifactPanel.js');
        const bytes = new TextEncoder().encode('hello');
        const readArtifact = vi.fn(async () => ({
            artifactId: FULL64,
            path: 'artifact.txt',
            mediaType: 'text/plain',
            sizeBytes: bytes.byteLength,
            digest: await sha256Bytes(bytes),
            bytes,
        }));
        expect(readArtifact).not.toHaveBeenCalled();
        act(() => {
            render(
                <div>
                    <PrPanel
                        summary={{
                            title: 'Read-only summary',
                            url: 'https://github.com/casey/uor-r4/pull/2042',
                        }}
                    />
                    <ArtifactPanel client={{ readArtifact }} reference={{ artifactId: FULL64 }} />
                </div>,
                host,
            );
        });

        expect(host.textContent).not.toMatch(/restore checkpoint/i);
        expect(host.textContent).not.toMatch(/\bcheckpoint\b/i);
        const actionable = [...host.querySelectorAll('button, a')]
            .map((node) => `${node.textContent ?? ''} ${node.getAttribute('aria-label') ?? ''}`)
            .join(' ');
        expect(actionable).not.toMatch(/\bmerge\b/i);
        expect(actionable).not.toMatch(/\bapprove\b/i);
        expect(host.querySelector('[data-action*="restore"], [data-action*="checkpoint"], [data-action*="merge"]')).toBeNull();
    });
});
