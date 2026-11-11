import { describe, expect, test } from 'vitest';

import { AgentClient } from './client.js';
import {
    HOST_RESULT_SCHEMA,
    HostClient,
    sha256Bytes,
    sha256Canonical,
    type HostDataRequest,
    type HostToolIntent,
    type HostToolResult,
    type HostTransport,
    type HostTransportCapabilities,
} from './hostClient.js';
import type { AgentTaskStore } from './store.js';
import { AgentStoreError } from './storeTypes.js';

const TASK_ID = 'tsk_0123456789abcdef';
const FULL64_A = `art_${'a1'.repeat(32)}`;
const FULL64_B = `art_${'b2'.repeat(32)}`;
const FULL64_C = `art_${'c3'.repeat(32)}`;
const FULL64_MISSING = `art_${'f'.repeat(64)}`;
const MAX_HOST_TREE_ENTRIES = 512;
const MAX_HOST_DIFF_FILES = 256;
const MAX_HOST_ARTIFACT_RETURN_BYTES = 1024 * 1024;
const MAX_HOST_ARTIFACT_BASE64_CHARS = 4 * Math.ceil(MAX_HOST_ARTIFACT_RETURN_BYTES / 3);
const RUN_ID = 'run_0123456789abcdef';
const ZERO_TASK_ID = 'tsk_0000000000000000';
const ZERO_RUN_ID = 'run_0000000000000000';
const ALL_CAPS: HostTransportCapabilities = {
    kind: 'bridge',
    hostAvailable: true,
    streaming: 'none',
    tree: true,
    changes: true,
    readArtifact: true,
    terminal: false,
    git: false,
    mcp: false,
    approvals: false,
};

function base64(bytes: Uint8Array): string {
    let binary = '';
    for (const byte of bytes) binary += String.fromCharCode(byte);
    return btoa(binary);
}

async function resultFor(
    intent: HostToolIntent,
    output: unknown,
): Promise<HostToolResult> {
    return {
        schema: HOST_RESULT_SCHEMA,
        intent_id: intent.intent_id,
        task_id: intent.task_id,
        run_id: intent.run_id,
        generation: intent.generation,
        tool_call_id: intent.tool_call_id,
        request_scope: intent.request_scope ?? 'task',
        status: 'ok',
        output,
        output_digest: await sha256Canonical(output),
        output_preview: '',
        occurred_at_ms: 1_800_000_000_000,
    };
}

class RecordingHostTransport implements HostTransport {
    readonly kind = 'bridge' as const;
    readonly supportsStream = false;
    readonly intents: HostToolIntent[] = [];
    readonly contexts: Array<HostDataRequest | undefined> = [];

    constructor(
        private readonly result: (
            intent: HostToolIntent,
        ) => Promise<HostToolResult>,
    ) {}

    async capabilities(): Promise<HostTransportCapabilities> {
        return { ...ALL_CAPS };
    }

    async invoke(
        intent: HostToolIntent,
        context?: HostDataRequest,
    ): Promise<HostToolResult> {
        this.intents.push(intent);
        this.contexts.push(context);
        return this.result(intent);
    }

    subscribe() {
        return { close: () => undefined, cursor: () => null };
    }

    async approve(input: {
        approvalId: string;
        decision: 'approved' | 'denied';
    }) {
        return { approvalId: input.approvalId, decision: input.decision };
    }
}

function clientWith(transport: RecordingHostTransport): AgentClient {
    return new AgentClient({
        hostClient: new HostClient({ transport }),
    });
}

function expectDataIdentity(intent: HostToolIntent): void {
    expect(intent.request_scope).toBe('data');
    expect(intent.task_id).toBe(ZERO_TASK_ID);
    expect(intent.run_id).toBe(ZERO_RUN_ID);
    expect(intent.generation).toBe(1);
}

function expectNoUrlArguments(intent: HostToolIntent): void {
    for (const key of [
        'url',
        'artifact_url',
        'download_url',
        'token',
        'expires_at',
        'link',
    ]) {
        expect(intent.arguments).not.toHaveProperty(key);
    }
}

interface DurableArtifactFixture {
    artifact_id: string;
    media_type: string;
    size_bytes: number;
    digest: string;
    bytes: Uint8Array;
    artifact_size_bytes?: number;
    whole_digest?: string;
    range?: { start: number; end: number };
}

function clientWithDurableArtifact(
    read: (artifactId: string) => Promise<DurableArtifactFixture | null>,
    transport: RecordingHostTransport,
): { client: AgentClient; reads: string[] } {
    const reads: string[] = [];
    const taskStore = {
        readArtifact: async (artifactId: string) => {
            reads.push(artifactId);
            return read(artifactId);
        },
    } as unknown as AgentTaskStore;
    return {
        client: new AgentClient({
            taskStore,
            hostClient: new HostClient({ transport }),
        }),
        reads,
    };
}

describe('AgentClient companion grants', () => {
    test('exposes_validated_host_grants_without_a_second_authority', async () => {
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {}),
        );
        (
            transport as unknown as { grants: () => Promise<unknown[]> }
        ).grants = async () => [
            { grantId: 'gr_9a3c5e7b1d2f4a60', path: '/tmp/studio-project' },
            { grantId: 'gr_0b1d2f4a6c8e9a30', path: '/tmp/other-project' },
        ];
        const client = clientWith(transport);
        await expect(client.grants()).resolves.toEqual([
            { grantId: 'gr_9a3c5e7b1d2f4a60', path: '/tmp/studio-project' },
            { grantId: 'gr_0b1d2f4a6c8e9a30', path: '/tmp/other-project' },
        ]);
    });

    test('a_host_without_grants_reports_none_instead_of_inventing_paths', async () => {
        const client = clientWith(
            new RecordingHostTransport(async (intent) => resultFor(intent, {})),
        );
        await expect(client.grants()).resolves.toEqual([]);
    });
});

describe('AgentClient companion data routing', () => {
    test('tree_uses_explicit_data_scope_and_grant_identity', async () => {
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                grant_id: 'gr_abcd1234',
                path: 'src',
                entries: [
                    { path: 'src', kind: 'directory' },
                    { path: 'src/a.ts', kind: 'file', size_bytes: 5 },
                ],
                truncated: false,
            }),
        );
        const client = clientWith(transport);

        await expect(
            client.tree({
                source: 'host',
                path: 'src',
                grantId: 'gr_abcd1234',
                taskId: TASK_ID,
            }),
        ).resolves.toEqual([
            { path: 'src', kind: 'directory' },
            { path: 'src/a.ts', kind: 'file', sizeBytes: 5 },
        ]);

        expect(transport.intents).toHaveLength(1);
        const intent = transport.intents[0]!;
        expect(intent.tool).toBe('file.tree');
        expectDataIdentity(intent);
        expect(intent.arguments).toEqual({
            grant_id: 'gr_abcd1234',
            path: 'src',
        });
        expectNoUrlArguments(intent);
        expect(transport.contexts[0]?.taskId).toBe(TASK_ID);
        expect(transport.contexts[0]?.runId).toBeUndefined();
    });

    test('tree_without_grant_keeps_the_existing_argument_shape', async () => {
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, { entries: [] }),
        );
        const client = clientWith(transport);

        await expect(
            client.tree({ source: 'host', path: 'src' }),
        ).resolves.toEqual([]);
        expect(transport.intents[0]!.arguments).toEqual({ path: 'src' });
    });

    test('tree_page_surfaces_truncation_symlinks_and_read_errors', async () => {
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                grant_id: 'gr_abcd1234',
                path: 'src',
                entries: [
                    { path: 'src/a.ts', kind: 'file', size_bytes: 5 },
                ],
                truncated: true,
                symlinks_skipped: 2,
                read_errors: 1,
            }),
        );
        const client = clientWith(transport);

        await expect(client.treePage({ source: 'host', path: 'src' })).resolves.toEqual({
            entries: [{ path: 'src/a.ts', kind: 'file', sizeBytes: 5 }],
            truncated: true,
            truncatedReported: true,
            symlinksSkipped: 2,
            readErrors: 1,
        });
        await expect(client.tree({ source: 'host', path: 'src' })).resolves.toEqual([
            { path: 'src/a.ts', kind: 'file', sizeBytes: 5 },
        ]);
    });

    test('tree_page_treats_a_full_listing_without_metadata_as_capped', async () => {
        const entries = Array.from({ length: MAX_HOST_TREE_ENTRIES }, (_, index) => ({
            path: `src/${index}.ts`,
            kind: 'file',
        }));
        const transport = new RecordingHostTransport(async (intent) => resultFor(intent, { entries }));
        const client = clientWith(transport);

        const page = await client.treePage({ source: 'host' });
        expect(page.entries).toHaveLength(MAX_HOST_TREE_ENTRIES);
        expect(page.truncated).toBe(true);
        expect(page.truncatedReported).toBe(false);
        expect(page.symlinksSkipped).toBe(0);
        expect(page.readErrors).toBe(0);
    });

    test('tree_page_rejects_malformed_page_metadata', async () => {
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, { entries: [], truncated: 'yes' }),
        );
        const client = clientWith(transport);

        await expect(client.treePage({ source: 'host' })).rejects.toMatchObject({
            code: 'invalid_event',
        });
        await expect(client.treePage({ source: 'host' })).rejects.toThrow('truncated flag is malformed');
    });

    test('branch_changes_use_host_but_last_turn_never_does', async () => {
        const headDigest = `sha256:${'a'.repeat(64)}`;
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                snapshot: {
                    scope: 'branch',
                    files: [{ path: 'src/a.ts', status: 'added' }],
                    head_digest: headDigest,
                    captured_at_ms: 1_800_000_000_000,
                },
            }),
        );
        const client = clientWith(transport);

        const branch = await client.changesSnapshot({
            taskId: TASK_ID,
            scope: 'branch',
            grantId: 'gr_abcd1234',
        });
        expect(branch).toEqual({
            scope: 'branch',
            files: [{ path: 'src/a.ts', status: 'added' }],
            headDigest,
            capturedAtMs: 1_800_000_000_000,
        });

        expect(transport.intents).toHaveLength(1);
        const intent = transport.intents[0]!;
        expect(intent.tool).toBe('git.changes');
        expectDataIdentity(intent);
        expect(intent.arguments).toEqual({
            grant_id: 'gr_abcd1234',
            scope: 'branch',
        });
        expectNoUrlArguments(intent);

        await expect(
            client.changesSnapshot({ taskId: TASK_ID, scope: 'last_turn' }),
        ).resolves.toBeNull();
        expect(transport.intents).toHaveLength(1);
    });

    test('read_artifact_checks_browser_artifact_before_host', async () => {
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {}),
        );
        const client = clientWith(transport);
        const bytes = new TextEncoder().encode('browser artifact');
        const digest = await sha256Bytes(bytes);
        client.registerLocalFile({
            path: 'src/browser.txt',
            mediaType: 'text/plain',
            bytes,
            digest,
            artifactId: 'art_browser',
        });

        const byId = await client.readArtifact({ artifactId: 'art_browser' });
        expect(byId).toMatchObject({
            path: 'src/browser.txt',
            mediaType: 'text/plain',
            sizeBytes: bytes.byteLength,
            digest,
        });
        expect(new TextDecoder().decode(byId.bytes)).toBe('browser artifact');

        const byPath = await client.readArtifact({ path: 'src/browser.txt' });
        expect(byPath).toMatchObject({ path: 'src/browser.txt', digest });
        expect(transport.intents).toHaveLength(0);
    });

    test('host_artifact_fallback_verifies_digest_and_size', async () => {
        const bytes = new TextEncoder().encode('host artifact');
        const digest = await sha256Bytes(bytes);
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                path: 'host.txt',
                media_type: 'text/plain',
                size_bytes: bytes.byteLength,
                digest,
                bytes_base64: base64(bytes),
            }),
        );
        const client = clientWith(transport);

        const content = await client.readArtifact({
            artifactId: 'art_host',
            taskId: TASK_ID,
        });
        expect(content).toMatchObject({
            path: 'host.txt',
            mediaType: 'text/plain',
            sizeBytes: bytes.byteLength,
            digest,
        });
        expect(new TextDecoder().decode(content.bytes)).toBe('host artifact');
        expect(Object.keys(content).sort()).toEqual([
            'bytes',
            'digest',
            'mediaType',
            'path',
            'sizeBytes',
        ]);

        expect(transport.intents).toHaveLength(1);
        const intent = transport.intents[0]!;
        expect(intent.tool).toBe('artifact.read');
        expectDataIdentity(intent);
        expect(intent.arguments).toEqual({ artifact_id: 'art_host' });
        expectNoUrlArguments(intent);
        expect(transport.contexts[0]?.taskId).toBe(TASK_ID);
    });

    test('host_artifact_range_verifies_returned_bytes', async () => {
        const bytes = new TextEncoder().encode('host ');
        const digest = await sha256Bytes(bytes);
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                path: 'host.txt',
                media_type: 'text/plain',
                size_bytes: bytes.byteLength,
                artifact_size_bytes: 13,
                digest,
                whole_digest: `sha256:${'b'.repeat(64)}`,
                range: { start: 0, end: bytes.byteLength },
                bytes_base64: base64(bytes),
            }),
        );
        const client = clientWith(transport);

        const content = await client.readArtifact({ artifactId: 'art_host' });
        expect(new TextDecoder().decode(content.bytes)).toBe('host ');
        expect(content.sizeBytes).toBe(5);
        expect(content.digest).toBe(digest);
    });

    test('host_artifact_rejects_corrupt_base64_digest_and_size', async () => {
        const bytes = new TextEncoder().encode('host artifact');
        const digest = await sha256Bytes(bytes);

        const corruptBase64 = clientWith(
            new RecordingHostTransport(async (intent) =>
                resultFor(intent, {
                    path: 'host.txt',
                    media_type: 'text/plain',
                    size_bytes: bytes.byteLength,
                    digest,
                    bytes_base64: '***',
                }),
            ),
        );
        await expect(
            corruptBase64.readArtifact({ artifactId: 'art_host' }),
        ).rejects.toMatchObject({ code: 'invalid_event' });

        const corruptDigest = clientWith(
            new RecordingHostTransport(async (intent) =>
                resultFor(intent, {
                    path: 'host.txt',
                    media_type: 'text/plain',
                    size_bytes: bytes.byteLength,
                    digest: `sha256:${'0'.repeat(64)}`,
                    bytes_base64: base64(bytes),
                }),
            ),
        );
        await expect(
            corruptDigest.readArtifact({ artifactId: 'art_host' }),
        ).rejects.toMatchObject({ code: 'invalid_event' });

        const corruptSize = clientWith(
            new RecordingHostTransport(async (intent) =>
                resultFor(intent, {
                    path: 'host.txt',
                    media_type: 'text/plain',
                    size_bytes: bytes.byteLength + 1,
                    digest,
                    bytes_base64: base64(bytes),
                }),
            ),
        );
        await expect(
            corruptSize.readArtifact({ artifactId: 'art_host' }),
        ).rejects.toMatchObject({ code: 'invalid_event' });
    });

    test('host_artifact_result_must_not_carry_urls_or_expiring_links', async () => {
        const bytes = new TextEncoder().encode('host artifact');
        const digest = await sha256Bytes(bytes);
        for (const field of [
            { url: 'https://example.invalid/artifact' },
            { download_url: 'https://example.invalid/artifact' },
            { expires_at: 1_900_000_000_000 },
            { token: 'page-secret' },
        ]) {
            const client = clientWith(
                new RecordingHostTransport(async (intent) =>
                    resultFor(intent, {
                        path: 'host.txt',
                        media_type: 'text/plain',
                        size_bytes: bytes.byteLength,
                        digest,
                        bytes_base64: base64(bytes),
                        ...field,
                    }),
                ),
            );
            await expect(
                client.readArtifact({ artifactId: 'art_host' }),
            ).rejects.toMatchObject({ code: 'invalid_event' });
        }
    });

    test('host_data_calls_keep_static_host_unavailable_behavior', async () => {
        const client = new AgentClient();
        await expect(client.tree()).resolves.toEqual([]);
        await expect(
            client.tree({ source: 'host' }),
        ).rejects.toMatchObject({ code: 'host_unavailable' });
        await expect(
            client.changesSnapshot({ taskId: TASK_ID, scope: 'branch' }),
        ).rejects.toMatchObject({ code: 'host_unavailable' });
        await expect(
            client.changesSnapshot({ taskId: TASK_ID, scope: 'last_turn' }),
        ).resolves.toBeNull();
        await expect(
            client.readArtifact({ artifactId: 'art_missing' }),
        ).rejects.toMatchObject({ code: 'host_unavailable' });
    });

    test('host_data_arguments_never_carry_unscoped_or_secret_fields', async () => {
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, { entries: [] }),
        );
        const client = clientWith(transport);
        await client.tree({
            source: 'host',
            path: 'src',
            grantId: 'gr_abcd1234',
        });
        const intent = transport.intents[0]!;
        expect(Object.keys(intent.arguments).sort()).toEqual([
            'grant_id',
            'path',
        ]);
        expect(JSON.stringify(intent)).not.toContain('granted');
        expect(JSON.stringify(intent)).not.toContain('approval_receipt');
    });

    test('read_artifact_prefers_durable_indexeddb_record', async () => {
        const bytes = new TextEncoder().encode('durable browser bytes');
        const digest = await sha256Bytes(bytes);
        const durable: DurableArtifactFixture = {
            artifact_id: FULL64_A,
            media_type: 'text/plain',
            size_bytes: bytes.byteLength,
            digest,
            bytes,
        };
        const hostBytes = new TextEncoder().encode('host artifact');
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                path: 'host.txt',
                media_type: 'text/plain',
                size_bytes: hostBytes.byteLength,
                digest: await sha256Bytes(hostBytes),
                bytes_base64: base64(hostBytes),
            }),
        );
        const { client, reads } = clientWithDurableArtifact(
            async (artifactId) =>
                artifactId === durable.artifact_id ? durable : null,
            transport,
        );

        const content = await client.readArtifact({
            artifactId: durable.artifact_id,
        });
        expect(content).toMatchObject({
            path: durable.artifact_id,
            mediaType: 'text/plain',
            sizeBytes: bytes.byteLength,
            digest,
        });
        expect(new TextDecoder().decode(content.bytes)).toBe(
            'durable browser bytes',
        );
        expect(reads).toEqual([durable.artifact_id]);
        expect(transport.intents).toHaveLength(0);
    });

    test('read_artifact_missing_durable_record_falls_back_to_host', async () => {
        const hostBytes = new TextEncoder().encode('host artifact');
        const hostDigest = await sha256Bytes(hostBytes);
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                path: 'host.txt',
                media_type: 'text/plain',
                size_bytes: hostBytes.byteLength,
                digest: hostDigest,
                bytes_base64: base64(hostBytes),
            }),
        );
        const { client, reads } = clientWithDurableArtifact(
            async () => null,
            transport,
        );

        const content = await client.readArtifact({
            artifactId: FULL64_B,
        });
        expect(content.path).toBe('host.txt');
        expect(reads).toEqual([FULL64_B]);
        expect(transport.intents).toHaveLength(1);

        // Path-only reads keep the existing local-then-host behavior and do
        // not consult the durable artifact store.
        const byPath = await client.readArtifact({ path: 'src/missing.txt' });
        expect(byPath.path).toBe('host.txt');
        expect(reads).toEqual([FULL64_B]);
        expect(transport.intents).toHaveLength(2);
    });

    test('read_artifact_runtime_unavailable_falls_back_to_host', async () => {
        const hostBytes = new TextEncoder().encode('host artifact');
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                path: 'host.txt',
                media_type: 'text/plain',
                size_bytes: hostBytes.byteLength,
                digest: await sha256Bytes(hostBytes),
                bytes_base64: base64(hostBytes),
            }),
        );
        const { client, reads } = clientWithDurableArtifact(async () => {
            throw new AgentStoreError(
                'runtime_unavailable',
                'durable task store is closed',
            );
        }, transport);

        const content = await client.readArtifact({ artifactId: FULL64_A });
        expect(content.path).toBe('host.txt');
        expect(reads).toEqual([FULL64_A]);
        expect(transport.intents).toHaveLength(1);
    });

    test('read_artifact_invalid_arguments_does_not_fall_back', async () => {
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, { path: 'host.txt' }),
        );
        const { client, reads } = clientWithDurableArtifact(async () => {
            throw new AgentStoreError(
                'invalid_arguments',
                'artifact_id is invalid',
            );
        }, transport);

        await expect(
            client.readArtifact({ artifactId: FULL64_B }),
        ).rejects.toMatchObject({ code: 'invalid_arguments' });
        expect(reads).toEqual([FULL64_B]);
        expect(transport.intents).toHaveLength(0);
    });

    test('read_artifact_full64_missing_durable_record_falls_back_to_host', async () => {
        const hostBytes = new TextEncoder().encode('host artifact');
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                path: 'host.txt',
                media_type: 'text/plain',
                size_bytes: hostBytes.byteLength,
                digest: await sha256Bytes(hostBytes),
                bytes_base64: base64(hostBytes),
            }),
        );
        const { client, reads } = clientWithDurableArtifact(
            async () => null,
            transport,
        );

        const content = await client.readArtifact({
            artifactId: FULL64_MISSING,
        });
        expect(content.path).toBe('host.txt');
        expect(reads).toEqual([FULL64_MISSING]);
        expect(transport.intents).toHaveLength(1);
        expect(transport.intents[0]!.arguments).toEqual({
            artifact_id: FULL64_MISSING,
        });
    });

    test('read_artifact_range_is_sent_to_host_and_metadata_preserved', async () => {
        const rangeBytes = new TextEncoder().encode('range');
        const wholeDigest = `sha256:${'d'.repeat(64)}`;
        const rangeDigest = await sha256Bytes(rangeBytes);
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                path: 'host.txt',
                media_type: 'text/plain',
                size_bytes: rangeBytes.byteLength,
                artifact_size_bytes: 21,
                digest: rangeDigest,
                whole_digest: wholeDigest,
                range: { start: 2, end: 7 },
                bytes_base64: base64(rangeBytes),
            }),
        );
        const client = clientWith(transport);

        const content = await client.readArtifact({
            artifactId: FULL64_A,
            start: 2,
            end: 7,
        });
        expect(transport.intents[0]!.arguments).toEqual({
            artifact_id: FULL64_A,
            start: 2,
            end: 7,
        });
        expect(content).toMatchObject({
            path: 'host.txt',
            mediaType: 'text/plain',
            sizeBytes: rangeBytes.byteLength,
            artifactSizeBytes: 21,
            digest: rangeDigest,
            wholeDigest,
            range: { start: 2, end: 7 },
        });
        expect(new TextDecoder().decode(content.bytes)).toBe('range');
    });

    test('read_artifact_malformed_range_is_refused_before_host', async () => {
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                path: 'host.txt',
                media_type: 'text/plain',
                size_bytes: 0,
                digest: `sha256:${'0'.repeat(64)}`,
                bytes_base64: '',
            }),
        );
        const client = clientWith(transport);
        await expect(
            client.readArtifact({ artifactId: FULL64_A, start: 5, end: 2 }),
        ).rejects.toMatchObject({ code: 'invalid_arguments' });
        await expect(
            client.readArtifact({ artifactId: FULL64_A, start: 1 }),
        ).rejects.toMatchObject({ code: 'invalid_arguments' });
        expect(transport.intents).toHaveLength(0);
    });

    test('read_artifact_durable_range_preserves_metadata', async () => {
        const bytes = new TextEncoder().encode('durable range');
        const digest = await sha256Bytes(bytes);
        const wholeDigest = `sha256:${'e'.repeat(64)}`;
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, { path: 'host.txt' }),
        );
        const { client, reads } = clientWithDurableArtifact(
            async () => ({
                artifact_id: FULL64_A,
                media_type: 'text/plain',
                size_bytes: bytes.byteLength,
                artifact_size_bytes: 100,
                digest,
                whole_digest: wholeDigest,
                range: { start: 10, end: 10 + bytes.byteLength },
                bytes,
            }),
            transport,
        );

        const content = await client.readArtifact({
            artifactId: FULL64_A,
            start: 10,
            end: 10 + bytes.byteLength,
        });
        expect(content).toMatchObject({
            sizeBytes: bytes.byteLength,
            artifactSizeBytes: 100,
            digest,
            wholeDigest,
            range: { start: 10, end: 10 + bytes.byteLength },
        });
        expect(reads).toEqual([FULL64_A]);
        expect(transport.intents).toHaveLength(0);
    });

    test('host_tree_result_is_capped_before_mapping', async () => {
        const entries = Array.from(
            { length: MAX_HOST_TREE_ENTRIES + 1 },
            (_, index) => ({ path: `file-${index}.txt`, kind: 'file' }),
        );
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, { entries }),
        );
        const client = clientWith(transport);
        await expect(
            client.tree({ source: 'host', path: 'src' }),
        ).rejects.toMatchObject({ code: 'invalid_event' });
    });

    test('host_changes_result_is_capped_before_mapping', async () => {
        const files = Array.from(
            { length: MAX_HOST_DIFF_FILES + 1 },
            (_, index) => ({ path: `file-${index}.txt` }),
        );
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                snapshot: {
                    scope: 'branch',
                    files,
                    head_digest: `sha256:${'a'.repeat(64)}`,
                    captured_at_ms: 1,
                },
            }),
        );
        const client = clientWith(transport);
        await expect(
            client.changesSnapshot({ taskId: TASK_ID, scope: 'branch' }),
        ).rejects.toMatchObject({ code: 'invalid_event' });
    });

    test('host_artifact_size_cap_is_enforced_before_decode', async () => {
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                path: 'host.txt',
                media_type: 'text/plain',
                size_bytes: MAX_HOST_ARTIFACT_RETURN_BYTES + 1,
                digest: `sha256:${'0'.repeat(64)}`,
                bytes_base64: 'A',
            }),
        );
        const client = clientWith(transport);
        await expect(
            client.readArtifact({ artifactId: FULL64_A }),
        ).rejects.toMatchObject({
            code: 'invalid_event',
            message: expect.stringContaining('client cap'),
        });
    });

    test('host_artifact_base64_cap_is_enforced_before_decode', async () => {
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                path: 'host.txt',
                media_type: 'text/plain',
                size_bytes: 1,
                digest: `sha256:${'0'.repeat(64)}`,
                bytes_base64: 'A'.repeat(MAX_HOST_ARTIFACT_BASE64_CHARS + 1),
            }),
        );
        const client = clientWith(transport);
        await expect(
            client.readArtifact({ artifactId: FULL64_A }),
        ).rejects.toMatchObject({
            code: 'invalid_event',
            message: expect.stringContaining('client cap'),
        });
    });

    test('host_artifact_malformed_range_is_refused', async () => {
        const bytes = new TextEncoder().encode('four');
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                path: 'host.txt',
                media_type: 'text/plain',
                size_bytes: bytes.byteLength,
                artifact_size_bytes: 10,
                digest: await sha256Bytes(bytes),
                whole_digest: `sha256:${'0'.repeat(64)}`,
                range: { start: 0, end: 5 },
                bytes_base64: base64(bytes),
            }),
        );
        const client = clientWith(transport);
        await expect(
            client.readArtifact({ artifactId: FULL64_A }),
        ).rejects.toMatchObject({ code: 'invalid_event' });
    });

    test('host_git_diff_requires_snapshot_wrapper', async () => {
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                scope: 'branch',
                files: [],
                head_digest: `sha256:${'a'.repeat(64)}`,
                captured_at_ms: 1,
            }),
        );
        const client = clientWith(transport);
        await expect(
            client.changesSnapshot({ taskId: TASK_ID, scope: 'branch' }),
        ).rejects.toMatchObject({ code: 'invalid_event' });
    });

    test('read_artifact_local_range_succeeds_with_metadata', async () => {
        const client = new AgentClient();
        const bytes = new TextEncoder().encode('local range artifact');
        const digest = await sha256Bytes(bytes);
        client.registerLocalFile({
            path: 'local.txt',
            mediaType: 'text/plain',
            bytes,
            digest,
        });

        const content = await client.readArtifact({
            path: 'local.txt',
            start: 6,
            end: 11,
        });
        expect(content).toMatchObject({
            path: 'local.txt',
            mediaType: 'text/plain',
            sizeBytes: 5,
            artifactSizeBytes: bytes.byteLength,
            wholeDigest: digest,
            range: { start: 6, end: 11 },
        });
        expect(new TextDecoder().decode(content.bytes)).toBe('range');
        expect(content.digest).toBe(await sha256Bytes(bytes.slice(6, 11)));
    });

    test('read_artifact_local_range_over_one_mib_is_refused', async () => {
        const client = new AgentClient();
        const bytes = new Uint8Array(MAX_HOST_ARTIFACT_RETURN_BYTES + 1);
        const digest = await sha256Bytes(bytes);
        client.registerLocalFile({
            path: 'big.bin',
            mediaType: 'application/octet-stream',
            bytes,
            digest,
        });
        await expect(
            client.readArtifact({
                path: 'big.bin',
                start: 0,
                end: bytes.byteLength,
            }),
        ).rejects.toMatchObject({ code: 'limit_exceeded' });
    });

    test('read_artifact_corrupt_range_is_not_hidden_by_host', async () => {
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                path: 'host.txt',
                media_type: 'text/plain',
                size_bytes: 1,
                digest: `sha256:${'0'.repeat(64)}`,
                bytes_base64: 'AA==',
            }),
        );
        const { client, reads } = clientWithDurableArtifact(async () => {
            throw new AgentStoreError(
                'snapshot_corrupt',
                'artifact digest does not match its bytes',
            );
        }, transport);

        await expect(
            client.readArtifact({ artifactId: FULL64_A, start: 0, end: 1 }),
        ).rejects.toMatchObject({ code: 'snapshot_corrupt' });
        expect(reads).toEqual([FULL64_A]);
        expect(transport.intents).toHaveLength(0);
    });

    test('host_full_artifact_whole_digest_mismatch_is_refused', async () => {
        const bytes = new TextEncoder().encode('four');
        const digest = await sha256Bytes(bytes);
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                path: 'host.txt',
                media_type: 'text/plain',
                size_bytes: bytes.byteLength,
                artifact_size_bytes: bytes.byteLength,
                digest,
                whole_digest: `sha256:${'0'.repeat(64)}`,
                range: { start: 0, end: bytes.byteLength },
                bytes_base64: base64(bytes),
            }),
        );
        const client = clientWith(transport);
        await expect(
            client.readArtifact({ artifactId: FULL64_A }),
        ).rejects.toMatchObject({ code: 'invalid_event' });
    });

    test('host_full_artifact_missing_whole_digest_is_refused', async () => {
        const bytes = new TextEncoder().encode('four');
        const digest = await sha256Bytes(bytes);
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                path: 'host.txt',
                media_type: 'text/plain',
                size_bytes: bytes.byteLength,
                artifact_size_bytes: bytes.byteLength,
                digest,
                range: { start: 0, end: bytes.byteLength },
                bytes_base64: base64(bytes),
            }),
        );
        const client = clientWith(transport);
        await expect(
            client.readArtifact({ artifactId: FULL64_A }),
        ).rejects.toMatchObject({ code: 'invalid_event' });
    });

    test('read_artifact_corrupt_durable_record_is_refused_without_host', async () => {
        const hostBytes = new TextEncoder().encode('host artifact');
        const transport = new RecordingHostTransport(async (intent) =>
            resultFor(intent, {
                path: 'host.txt',
                media_type: 'text/plain',
                size_bytes: hostBytes.byteLength,
                digest: await sha256Bytes(hostBytes),
                bytes_base64: base64(hostBytes),
            }),
        );
        const { client, reads } = clientWithDurableArtifact(async () => {
            throw new AgentStoreError(
                'snapshot_corrupt',
                'artifact digest does not match its bytes',
            );
        }, transport);

        await expect(
            client.readArtifact({ artifactId: FULL64_C }),
        ).rejects.toMatchObject({ code: 'snapshot_corrupt' });
        expect(reads).toEqual([FULL64_C]);
        expect(transport.intents).toHaveLength(0);
    });
});
