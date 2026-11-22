import { describe, expect, test, vi } from 'vitest';

import { canonicalJson } from './generated/agentCanonical.js';
import {
    AgentClient,
    AgentClientError,
    type ArtifactContent,
    type ChangesSnapshot,
    type ChangesSnapshotInput,
    type HostCapabilities,
    type ReadArtifactInput,
    type TreeEntry,
} from './client.js';
import {
    HOST_INTENT_SCHEMA,
    HOST_RESULT_SCHEMA,
    HostClient,
    sha256Canonical,
    type HostExecuteOptions,
    type HostSubscribeOptions,
    type HostSubscription,
    type HostToolIntent,
    type HostToolResult,
    type HostTransport,
    type HostTransportCapabilities,
} from './hostClient.js';
import {
    AgentTransportHostTransport,
    LoopbackHostTransport,
    NullHostTransport,
    StaticHostTransport,
    TauriHostTransport,
    createTauriHostBridge,
    type TauriHostBridge,
    type TauriChannelHandle,
    type TauriCoreLike,
} from './transports.js';
import { AgentRuntime } from './runtime.js';
import { ReplayModelPort } from './modelPort.js';
import { FixtureTransport } from './fixtureTransport.js';
import type { AgentTaskStore } from './store.js';
import { createAgentStore } from '../state/agent.js';

const ALL_FALSE: HostCapabilities = {
    tree: false,
    changes: false,
    readArtifact: false,
    terminal: false,
    git: false,
    mcp: false,
    approvals: false,
};

const ALL_CAPS: HostCapabilities = {
    tree: true,
    changes: true,
    readArtifact: true,
    terminal: false,
    git: false,
    mcp: false,
    approvals: true,
};

const TASK_ID = 'tsk_0123456789abcdef';
const RUN_ID = 'run_0123456789abcdef';

class FakeHostTransport implements HostTransport {
    readonly kind = 'bridge' as const;
    readonly intents: HostToolIntent[] = [];
    readonly approvals: Array<{
        approvalId: string;
        decision: 'approved' | 'denied';
    }> = [];
    capabilityValues: HostCapabilities = ALL_FALSE;
    resultFactory: (intent: HostToolIntent) => Promise<HostToolResult> = (
        intent,
    ) => validResult(intent);

    async capabilities(): Promise<HostTransportCapabilities> {
        return {
            kind: 'bridge',
            hostAvailable: true,
            streaming: 'none',
            ...this.capabilityValues,
        };
    }

    async invoke(intent: HostToolIntent): Promise<HostToolResult> {
        this.intents.push(intent);
        return this.resultFactory(intent);
    }

    subscribe(): HostSubscription {
        return { close: () => undefined, cursor: () => null };
    }

    async approve(input: {
        approvalId: string;
        decision: 'approved' | 'denied';
    }) {
        this.approvals.push(input);
        return { approvalId: input.approvalId, decision: input.decision };
    }
}

async function validIntent(
    overrides: Partial<HostToolIntent> = {},
): Promise<HostToolIntent> {
    const args = overrides.arguments ?? { path: 'src/a.ts' };
    return {
        schema: HOST_INTENT_SCHEMA,
        intent_id: 'hti_0123456789abcdef_000000000001',
        task_id: TASK_ID,
        run_id: RUN_ID,
        generation: 2,
        tool_call_id: 'tc_02',
        tool: 'file.read',
        request_scope: 'task',
        ...overrides,
        arguments: args,
        arguments_digest:
            overrides.arguments_digest ?? (await sha256Canonical(args)),
    };
}

async function dataIntent(
    tool: HostToolIntent['tool'] = 'file.tree',
    args: Record<string, unknown> = { path: 'src' },
    overrides: Partial<HostToolIntent> = {},
): Promise<HostToolIntent> {
    return {
        schema: HOST_INTENT_SCHEMA,
        intent_id: 'hti_data_000000000001',
        task_id: 'tsk_0000000000000000',
        run_id: 'run_0000000000000000',
        generation: 1,
        tool_call_id: 'tc_data_000000000001',
        tool,
        request_scope: 'data',
        ...overrides,
        arguments: args,
        arguments_digest:
            overrides.arguments_digest ?? (await sha256Canonical(args)),
    };
}

async function validResult(
    intent: HostToolIntent,
    overrides: Partial<HostToolResult> = {},
): Promise<HostToolResult> {
    const output = overrides.output ?? { content: 'hello' };
    let preview = '';
    let digest = `sha256:${'0'.repeat(64)}`;
    try {
        preview = canonicalJson(output).slice(0, 512);
        digest = await sha256Canonical(output);
    } catch {
        // Hostile fixtures keep their bytes; the validator must reject them.
    }
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
        output_digest: digest,
        output_preview: preview,
        occurred_at_ms: 1_800_000_000_000,
        ...(intent.approval_id === undefined
            ? {}
            : { approval_receipt: `hmac-sha256:${'a'.repeat(64)}` }),
        ...overrides,
    };
}

class ManualScheduler {
    private nextHandle = 1;
    private readonly tasks = new Map<
        number,
        { handler: () => void; due: number }
    >();
    private now = 0;

    setTimeout = (handler: () => void, timeoutMs = 0): unknown => {
        const handle = this.nextHandle++;
        this.tasks.set(handle, {
            handler,
            due: this.now + Math.max(0, timeoutMs),
        });
        return handle;
    };

    clearTimeout = (handle: unknown): void => {
        if (typeof handle === 'number') this.tasks.delete(handle);
    };

    runNext(): boolean {
        let selected: [number, { handler: () => void; due: number }] | null =
            null;
        for (const entry of this.tasks) {
            if (selected === null || entry[1].due < selected[1].due) {
                selected = entry;
            }
        }
        if (selected === null) return false;
        this.tasks.delete(selected[0]);
        this.now = selected[1].due;
        selected[1].handler();
        return true;
    }

    get pending(): number {
        return this.tasks.size;
    }
}

async function sha256Raw(bytes: Uint8Array): Promise<string> {
    const digest = new Uint8Array(
        await globalThis.crypto.subtle.digest(
            'SHA-256',
            bytes as unknown as BufferSource,
        ),
    );
    let hex = '';
    for (const byte of digest) hex += byte.toString(16).padStart(2, '0');
    return `sha256:${hex}`;
}

function hasLoneSurrogate(text: string): boolean {
    for (let index = 0; index < text.length; index += 1) {
        const code = text.charCodeAt(index);
        if (code >= 0xd800 && code <= 0xdbff) {
            const next = text.charCodeAt(index + 1);
            if (!(next >= 0xdc00 && next <= 0xdfff)) return true;
            index += 1;
        } else if (code >= 0xdc00 && code <= 0xdfff) {
            return true;
        }
    }
    return false;
}

async function until(
    predicate: () => boolean,
    label: string,
    timeoutMs = 2_000,
): Promise<void> {
    const deadline = Date.now() + timeoutMs;
    while (!predicate()) {
        if (Date.now() > deadline) {
            throw new Error(`timed out waiting for ${label}`);
        }
        await new Promise((resolve) => setTimeout(resolve, 5));
    }
}

function flush(): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, 0));
}

describe('HostClient transport matrix', () => {
    test('static_host_is_unavailable', async () => {
        const fetchSpy = vi.fn();
        const originalFetch = globalThis.fetch;
        Object.defineProperty(globalThis, 'fetch', {
            value: fetchSpy,
            configurable: true,
            writable: true,
        });
        try {
            const client = new HostClient({
                transport: new StaticHostTransport(),
            });
            await expect(client.capabilities()).resolves.toEqual(ALL_FALSE);
            await expect(client.execute(await validIntent())).rejects.toMatchObject(
                {
                    name: 'AgentClientError',
                    code: 'host_unavailable',
                },
            );
            await expect(
                client.execute(await validIntent()),
            ).rejects.not.toMatchObject({ code: 'runtime_unavailable' });
            await expect(
                client.approve({
                    approvalId: 'apr_5c2e8a1f9b7d3e40',
                    decision: 'approved',
                }),
            ).rejects.toMatchObject({ code: 'host_unavailable' });
            expect(() => client.subscribe(() => undefined)).toThrowError(
                AgentClientError,
            );
            expect(fetchSpy).not.toHaveBeenCalled();

            const nullClient = new HostClient({
                transport: new NullHostTransport(),
            });
            await expect(
                nullClient.execute(await validIntent()),
            ).rejects.toMatchObject({ code: 'host_unavailable' });
            await expect(nullClient.capabilities()).resolves.toEqual(ALL_FALSE);
        } finally {
            Object.defineProperty(globalThis, 'fetch', {
                value: originalFetch,
                configurable: true,
                writable: true,
            });
        }
    });

    test('tauri_transport_uses_channel_when_available', async () => {
        const commands: string[] = [];
        let channelHandler: ((message: unknown) => void) | null = null;
        let channelCreated = 0;
        const bridge: TauriHostBridge = {
            hasChannel: true,
            async invoke(command, payload) {
                commands.push(command);
                if (command === 'agent_host_capabilities') return ALL_CAPS;
                if (command === 'agent_host_invoke') {
                    const intent = (payload as { intent: HostToolIntent }).intent;
                    return validResult(intent);
                }
                if (command === 'agent_subscribe') return null;
                throw new Error(`unexpected Tauri command ${command}`);
            },
            channel(onMessage): TauriChannelHandle {
                channelCreated += 1;
                channelHandler = onMessage;
                return { onmessage: onMessage };
            },
        };
        const transport = new TauriHostTransport({ bridge });
        const intent = await validIntent();
        const result = (await transport.invoke(intent)) as HostToolResult;

        expect(result.intent_id).toBe(intent.intent_id);
        expect(commands).toContain('agent_host_invoke');
        expect(commands).not.toContain('agent_host_poll');

        const received: HostToolResult[] = [];
        const subscription = transport.subscribe((value) => received.push(value));
        await flush();

        expect(commands).toContain('agent_subscribe');
        expect(channelCreated).toBe(1);
        expect(channelHandler).not.toBeNull();
        const emit = channelHandler as unknown as (message: unknown) => void;
        emit(await validResult(intent));
        expect(received).toHaveLength(1);
        subscription.close();
    });

    test('tauri_offline_shim_uses_poll', async () => {
        const commands: string[] = [];
        let channelCreated = 0;
        const pollCursors: Array<string | null> = [];
        const bridge: TauriHostBridge = {
            hasChannel: false,
            async invoke(command, payload) {
                commands.push(command);
                if (command === 'agent_host_capabilities') return ALL_CAPS;
                if (command === 'agent_host_invoke') {
                    const intent = (payload as { intent: HostToolIntent }).intent;
                    return { accepted: true, intent_id: intent.intent_id };
                }
                if (command === 'agent_host_poll') {
                    pollCursors.push(
                        (payload as { cursor?: string | null }).cursor ?? null,
                    );
                    const intent = await validIntent();
                    return {
                        cursor: 'c1',
                        results: [await validResult(intent)],
                    };
                }
                throw new Error(`unexpected Tauri command ${command}`);
            },
            channel(): TauriChannelHandle {
                channelCreated += 1;
                throw new Error('the offline shim has no Channel');
            },
        };
        const transport = new TauriHostTransport({ bridge });
        const intent = await validIntent();
        const invoked = await transport.invoke(intent);
        expect(invoked).toMatchObject({ accepted: true });
        expect(commands).toContain('agent_host_invoke');

        const received: HostToolResult[] = [];
        const subscription = transport.subscribe((value) => received.push(value));
        await until(() => received.length === 1, 'offline poll delivery');
        expect(commands).toContain('agent_host_poll');
        expect(commands).not.toContain('agent_subscribe');
        expect(pollCursors[0]).toBeNull();
        expect(channelCreated).toBe(0);
        subscription.close();
    });

    test('loopback_transport_uses_fetch_stream_not_event_source', async () => {
        const calls: Array<{ url: string; init: RequestInit }> = [];
        const intent = await validIntent();
        const result = await validResult(intent);
        const fetchImpl = (async (
            input: RequestInfo | URL,
            init: RequestInit = {},
        ): Promise<Response> => {
            const url = String(input);
            calls.push({ url, init });
            if (url.includes('/v1/studio/host/intents')) {
                return new Response(JSON.stringify(result), {
                    status: 200,
                    headers: { 'content-type': 'application/json' },
                });
            }
            const body = `${JSON.stringify(result)}\n${JSON.stringify({
                ...result,
                intent_id: 'hti_0123456789abcdef_000000000002',
            })}\n`;
            const stream = new ReadableStream<Uint8Array>({
                start(controller) {
                    controller.enqueue(new TextEncoder().encode(body));
                    controller.close();
                },
            });
            return new Response(stream, {
                status: 200,
                headers: { 'content-type': 'application/x-ndjson' },
            });
        }) as typeof fetch;

        const eventSources: number[] = [];
        const originalEventSource = (globalThis as { EventSource?: unknown })
            .EventSource;
        (globalThis as { EventSource?: unknown }).EventSource = class {
            constructor() {
                eventSources.push(1);
            }
        };
        try {
            const transport = new LoopbackHostTransport({
                token: 'launch-token',
                fetch: fetchImpl,
                cursor: '7',
            });
            await transport.invoke(intent);

            expect(calls[0]?.url).toContain('/v1/studio/host/intents');
            expect(calls[0]?.init.method).toBe('POST');
            const postHeaders = calls[0]?.init.headers as Record<string, string>;
            expect(postHeaders['X-UOR-Desktop-Token']).toBe('launch-token');
            expect(postHeaders['content-type']).toBe('application/json');
            expect(JSON.parse(String(calls[0]?.init.body))).toMatchObject({
                schema: HOST_INTENT_SCHEMA,
                intent_id: intent.intent_id,
            });

            const received: HostToolResult[] = [];
            const subscription = transport.subscribe((value) =>
                received.push(value),
            );
            await until(() => received.length === 2, 'loopback NDJSON results');

            expect(calls[1]?.url).toContain('/v1/studio/host/results?cursor=7');
            expect(calls[1]?.init.method).toBe('GET');
            const getHeaders = calls[1]?.init.headers as Record<string, string>;
            expect(getHeaders['X-UOR-Desktop-Token']).toBe('launch-token');
            expect(getHeaders.accept).toBe('application/x-ndjson');
            const headerNames = Object.keys(getHeaders).map((name) =>
                name.toLowerCase(),
            );
            expect(headerNames).not.toContain('last-event-id');
            expect(eventSources).toHaveLength(0);
            subscription.close();
        } finally {
            (globalThis as { EventSource?: unknown }).EventSource =
                originalEventSource;
        }
    });

    test('loopback_read_stop_aborts_the_stream_and_delivers_no_late_lines', async () => {
        const intent = await validIntent();
        const result = await validResult(intent);
        let controller:
            | ReadableStreamDefaultController<Uint8Array>
            | null = null;
        let streamSignal: AbortSignal | null = null;
        const fetchImpl = (async (
            input: RequestInfo | URL,
            init: RequestInit = {},
        ): Promise<Response> => {
            const url = String(input);
            if (url.includes('/v1/studio/host/intents')) {
                return new Response(JSON.stringify(result), {
                    status: 200,
                    headers: { 'content-type': 'application/json' },
                });
            }
            streamSignal = init.signal ?? null;
            const stream = new ReadableStream<Uint8Array>({
                start(streamController) {
                    controller = streamController;
                    streamController.enqueue(
                        new TextEncoder().encode(
                            `${JSON.stringify(result)}\n`,
                        ),
                    );
                    init.signal?.addEventListener('abort', () => {
                        streamController.error(
                            new DOMException('Aborted', 'AbortError'),
                        );
                    });
                },
            });
            return new Response(stream, {
                status: 200,
                headers: { 'content-type': 'application/x-ndjson' },
            });
        }) as typeof fetch;
        const transport = new LoopbackHostTransport({
            token: 'launch-token',
            fetch: fetchImpl,
            cursor: '7',
        });
        const received: HostToolResult[] = [];
        const errors: AgentClientError[] = [];
        const subscription = transport.subscribe(
            (value) => received.push(value),
            { onError: (error) => errors.push(error) },
        );
        await until(() => received.length === 1, 'first loopback line');
        subscription.close();
        await until(() => streamSignal?.aborted === true, 'loopback abort');
        // The aborted reader is already errored; a transport that ignored the
        // stop would still be able to deliver. Give it a turn to try.
        expect(controller).not.toBeNull();
        await new Promise<void>((resolve) =>
            globalThis.setTimeout(resolve, 20),
        );
        expect(received).toHaveLength(1);
        expect(errors).toHaveLength(0);
    });

    test('loopback_cursor_expired_requests_snapshot_resync', async () => {
        const fetchImpl = (async () =>
            new Response(JSON.stringify({ code: 'cursor_expired' }), {
                status: 410,
                headers: { 'content-type': 'application/json' },
            })) as typeof fetch;
        const transport = new LoopbackHostTransport({
            token: 'launch-token',
            fetch: fetchImpl,
            cursor: '7',
        });
        const received: HostToolResult[] = [];
        const resyncs: string[] = [];
        const errors: AgentClientError[] = [];
        transport.subscribe((value) => received.push(value), {
            onResync: (reason) => resyncs.push(reason),
            onError: (error) => errors.push(error),
        });
        await until(() => resyncs.length === 1, 'cursor_expired resync');
        expect(resyncs).toEqual(['cursor_expired']);
        expect(received).toHaveLength(0);
        expect(errors).toHaveLength(0);
    });
});

describe('HostClient result and approval authority', () => {
    test('host_result_is_validated_against_intent', async () => {
        const intent = await validIntent();
        const cases: Array<[Partial<HostToolResult>, string]> = [
            [{ intent_id: 'hti_other' }, 'out_of_order'],
            [{ task_id: 'tsk_ffffffffffffffff' }, 'cross_task_event'],
            [{ run_id: 'run_ffffffffffffffff' }, 'out_of_order'],
            [{ generation: intent.generation + 1 }, 'stale_generation'],
            [{ tool_call_id: 'tc_other' }, 'out_of_order'],
            [{ output_digest: `sha256:${'0'.repeat(64)}` }, 'invalid_arguments'],
        ];
        for (const [override, code] of cases) {
            const transport = new FakeHostTransport();
            transport.resultFactory = async (received) => ({
                ...(await validResult(received)),
                ...override,
            });
            const client = new HostClient({ transport });
            await expect(client.execute(intent)).rejects.toMatchObject({ code });
        }

        const eventTransport = new FakeHostTransport();
        eventTransport.resultFactory = async (received) => ({
            ...(await validResult(received)),
            type: 'tool.completed',
            event_id: 'evt_run_0123456789abcdef_2_000000000003',
        });
        await expect(
            new HostClient({ transport: eventTransport }).execute(intent),
        ).rejects.toMatchObject({ code: 'invalid_event' });

        const acceptedTransport = new FakeHostTransport();
        const accepted = await new HostClient({
            transport: acceptedTransport,
        }).execute(intent);
        expect(accepted.intent_id).toBe(intent.intent_id);
        expect(accepted.output).toEqual({ content: 'hello' });
    });

    test('approval_receipt_is_required_for_an_approval_intent', async () => {
        const intent = await validIntent({
            approval_id: 'apr_5c2e8a1f9b7d3e40',
        });
        const missing = new FakeHostTransport();
        missing.resultFactory = async (received) => {
            const value = await validResult(received);
            delete value.approval_receipt;
            return value;
        };
        await expect(
            new HostClient({ transport: missing }).execute(intent),
        ).rejects.toMatchObject({ code: 'approval_forbidden' });

        const malformed = new FakeHostTransport();
        malformed.resultFactory = async (received) => ({
            ...(await validResult(received)),
            approval_receipt: 'page-supplied',
        });
        await expect(
            new HostClient({ transport: malformed }).execute(intent),
        ).rejects.toMatchObject({ code: 'approval_forbidden' });

        const accepted = new FakeHostTransport();
        await expect(
            new HostClient({ transport: accepted }).execute(intent),
        ).resolves.toMatchObject({
            approval_receipt: `hmac-sha256:${'a'.repeat(64)}`,
        });
    });

    test('page_approval_input_is_only_id_and_decision', async () => {
        const transport = new FakeHostTransport();
        const client = new HostClient({ transport });
        const approvalId = 'apr_5c2e8a1f9b7d3e40';
        const authorityFields = [
            { granted: true },
            { path: 'src/a.ts' },
            { scope: 'write' },
            { command: 'git commit' },
            { key: 'secret' },
            { receipt: 'host-receipt' },
        ];
        for (const extra of authorityFields) {
            await expect(
                client.approve({
                    approvalId,
                    decision: 'approved',
                    ...extra,
                } as never),
            ).rejects.toMatchObject({ code: 'invalid_arguments' });
        }
        await expect(
            client.approve({
                approvalId,
                decision: 'maybe',
            } as never),
        ).rejects.toMatchObject({ code: 'invalid_arguments' });
        expect(transport.approvals).toHaveLength(0);

        await expect(
            client.approve({ approvalId, decision: 'denied' }),
        ).resolves.toEqual({ approvalId, decision: 'denied' });
        expect(transport.approvals).toEqual([
            { approvalId, decision: 'denied' },
        ]);
    });
});

describe('HostClient canonical tool values', () => {
    test('rejects_non_canonical_tool_arguments', async () => {
        const nonCanonical: Array<Record<string, unknown>> = [
            { path: 'src/a.ts', weight: 1.5 },
            { path: 'src/a.ts', offset: -1 },
            { path: 'src/a.ts', 'café': 'value' },
        ];
        for (const argumentsValue of nonCanonical) {
            const transport = new FakeHostTransport();
            const client = new HostClient({ transport });
            await expect(
                client.execute({
                    taskId: TASK_ID,
                    runId: RUN_ID,
                    generation: 2,
                    toolCallId: 'tc_02',
                    tool: 'file.read',
                    arguments: argumentsValue,
                }),
            ).rejects.toMatchObject({ code: 'invalid_arguments' });
            expect(transport.intents).toHaveLength(0);
        }

        const transport = new FakeHostTransport();
        const client = new HostClient({ transport });
        await expect(
            client.execute(await validIntent({ arguments_digest: `sha256:${'0'.repeat(64)}` })),
        ).rejects.toMatchObject({ code: 'invalid_arguments' });
        expect(transport.intents).toHaveLength(0);
    });

    test('rejects_non_canonical_host_results', async () => {
        const nonCanonical: Array<Record<string, unknown>> = [
            { value: 1.5 },
            { value: -1 },
            { 'café': 1 },
        ];
        for (const output of nonCanonical) {
            const transport = new FakeHostTransport();
            transport.resultFactory = async (intent) =>
                validResult(intent, { output });
            const client = new HostClient({ transport });
            await expect(
                client.execute(await validIntent()),
            ).rejects.toMatchObject({ code: 'invalid_arguments' });
        }
    });
});

describe('Runtime host boundary', () => {
    test('runtime_accepts_a_valid_host_result_before_tool_completed', async () => {
        const transport = new FakeHostTransport();
        transport.capabilityValues = ALL_CAPS;
        const client = new HostClient({ transport });
        const model = new ReplayModelPort({
            script: [
                [
                    {
                        type: 'tool-call',
                        toolCallId: 'tc_02',
                        tool: 'file.read',
                        arguments: { path: 'src/a.ts' },
                    },
                    { type: 'finished', reason: 'tool' },
                ],
                [
                    { type: 'text-delta', text: 'read complete' },
                    { type: 'finished', reason: 'stop' },
                ],
            ],
        });
        const runtime = new AgentRuntime({
            store: createAgentStore(),
            modelPort: model,
            hostClient: client,
            now: () => 1_800_000_000_000,
        });
        await runtime.open();
        try {
            const task = await runtime.createTask({ title: 'host result' });
            const run = await runtime.startRun({
                taskId: task.taskId,
                prompt: 'read a file',
            });
            await until(
                () =>
                    runtime.store.snapshot().runs.find(
                        (candidate) =>
                            candidate.taskId === task.taskId &&
                            candidate.runId === run.runId,
                    )?.state === 'finished',
                'host result run finished',
            );
            const view = runtime.store.snapshot().runs.find(
                (candidate) =>
                    candidate.taskId === task.taskId &&
                    candidate.runId === run.runId,
            );
            const types = view?.events.map((event) => event.type) ?? [];
            expect(types).toContain('tool.completed');
            expect(types).not.toContain('artifact.created');
            expect(transport.intents).toHaveLength(1);
            expect(transport.intents[0]).toMatchObject({
                schema: HOST_INTENT_SCHEMA,
                task_id: task.taskId,
                run_id: run.runId,
                generation: run.generation,
                tool_call_id: 'tc_02',
                tool: 'file.read',
            });
        } finally {
            await runtime.dispose();
        }
    });

    test('runtime_rejects_a_cross_task_host_result_before_events', async () => {
        const transport = new FakeHostTransport();
        transport.capabilityValues = ALL_CAPS;
        transport.resultFactory = async (intent) => ({
            ...(await validResult(intent)),
            task_id: 'tsk_ffffffffffffffff',
        });
        const client = new HostClient({ transport });
        const model = new ReplayModelPort({
            script: [
                [
                    {
                        type: 'tool-call',
                        toolCallId: 'tc_02',
                        tool: 'file.read',
                        arguments: { path: 'src/a.ts' },
                    },
                    { type: 'finished', reason: 'tool' },
                ],
                [
                    { type: 'text-delta', text: 'after rejected result' },
                    { type: 'finished', reason: 'stop' },
                ],
            ],
        });
        const runtime = new AgentRuntime({
            store: createAgentStore(),
            modelPort: model,
            hostClient: client,
            now: () => 1_800_000_000_000,
        });
        await runtime.open();
        try {
            const task = await runtime.createTask({ title: 'cross task' });
            const run = await runtime.startRun({
                taskId: task.taskId,
                prompt: 'read a file',
            });
            await until(
                () =>
                    runtime.store.snapshot().runs.find(
                        (candidate) =>
                            candidate.taskId === task.taskId &&
                            candidate.runId === run.runId,
                    )?.state === 'finished',
                'cross-task run finished',
            );
            const view = runtime.store.snapshot().runs.find(
                (candidate) =>
                    candidate.taskId === task.taskId &&
                    candidate.runId === run.runId,
            );
            const types = view?.events.map((event) => event.type) ?? [];
            expect(types).not.toContain('tool.completed');
            expect(types).not.toContain('artifact.created');
            const failed = view?.events.find(
                (event) => event.type === 'tool.failed',
            );
            expect(failed?.data.code).toBe('cross_task_event');
        } finally {
            await runtime.dispose();
        }
    });
});

describe('AgentClient host delegation', () => {
    test('agent_client_routes_host_data_through_host_client', async () => {
        const transport = new FakeHostTransport();
        transport.capabilityValues = ALL_CAPS;
        transport.resultFactory = async (intent) => {
            if (intent.tool === 'file.tree') {
                return validResult(intent, {
                    output: {
                        entries: [
                            { path: 'src', kind: 'directory' },
                            {
                                path: 'src/a.ts',
                                kind: 'file',
                                size_bytes: 3,
                            },
                        ],
                    },
                });
            }
            if (intent.tool === 'git.changes') {
                return validResult(intent, {
                    output: {
                        snapshot: {
                            scope: 'branch',
                            files: [{ path: 'src/a.ts' }],
                            head_digest: `sha256:${'b'.repeat(64)}`,
                            captured_at_ms: 1_800_000_000_000,
                        },
                    },
                });
            }
            throw new Error(`unexpected data tool ${intent.tool}`);
        };
        const client = new AgentClient({
            transport: new FakeAgentTransport(),
            hostClient: new HostClient({ transport }),
        });
        await expect(
            client.tree({ source: 'host', path: 'src' }),
        ).resolves.toEqual([
            { path: 'src', kind: 'directory' },
            { path: 'src/a.ts', kind: 'file', sizeBytes: 3 },
        ]);
        await expect(
            client.changesSnapshot({
                taskId: TASK_ID,
                runId: RUN_ID,
                scope: 'branch',
            }),
        ).resolves.toMatchObject({
            scope: 'branch',
            headDigest: `sha256:${'b'.repeat(64)}`,
        });
        expect(transport.intents.map((intent) => intent.tool)).toEqual([
            'file.tree',
            'git.changes',
        ]);
    });
});

class FakeAgentTransport extends FixtureTransport {
    override async hostCapabilities(): Promise<HostCapabilities> {
        return ALL_FALSE;
    }
}

class RecordingAgentTransport extends FixtureTransport {
    readonly changesInputs: ChangesSnapshotInput[] = [];
    readonly artifactInputs: ReadArtifactInput[] = [];

    override async hostCapabilities(): Promise<HostCapabilities> {
        return { ...ALL_FALSE, changes: true, readArtifact: true };
    }

    override async changesSnapshot(
        input: ChangesSnapshotInput,
    ): Promise<ChangesSnapshot | null> {
        this.changesInputs.push({ ...input });
        return {
            scope: 'branch',
            files: [{ path: 'a.ts' }],
            headDigest: `sha256:${'c'.repeat(64)}`,
            capturedAtMs: 1,
        };
    }

    override async readArtifact(
        input: ReadArtifactInput,
    ): Promise<ArtifactContent> {
        this.artifactInputs.push({ ...input });
        const bytes = new TextEncoder().encode('hello');
        return {
            path: 'a.txt',
            mediaType: 'text/plain',
            sizeBytes: bytes.byteLength,
            digest: await sha256Raw(bytes),
            bytes,
        };
    }
}

class PendingTreeTransport extends FixtureTransport {
    treeStarted = false;

    override async hostCapabilities(): Promise<HostCapabilities> {
        return { ...ALL_FALSE, tree: true };
    }

    override tree(): Promise<TreeEntry[]> {
        this.treeStarted = true;
        return new Promise<TreeEntry[]>(() => undefined);
    }
}

class ManualStreamTransport implements HostTransport {
    readonly kind = 'bridge' as const;
    readonly supportsStream = true;
    readonly subscriptions: Array<{
        listener: (result: HostToolResult) => void;
        options: HostSubscribeOptions;
    }> = [];

    async capabilities(): Promise<HostTransportCapabilities> {
        return {
            kind: 'bridge',
            hostAvailable: true,
            streaming: 'channel',
            ...ALL_FALSE,
        };
    }

    async invoke(): Promise<{ accepted: true }> {
        return { accepted: true };
    }

    subscribe(
        listener: (result: HostToolResult) => void,
        options: HostSubscribeOptions = {},
    ): HostSubscription {
        this.subscriptions.push({ listener, options });
        return { close: () => undefined, cursor: () => null };
    }

    async approve(input: {
        approvalId: string;
        decision: 'approved' | 'denied';
    }) {
        return { approvalId: input.approvalId, decision: input.decision };
    }
}

class TreeOnlyAgentTransport extends FixtureTransport {
    entries: TreeEntry[] = [];

    override async hostCapabilities(): Promise<HostCapabilities> {
        return { ...ALL_FALSE, tree: true };
    }

    override async tree(): Promise<TreeEntry[]> {
        return this.entries;
    }
}

describe('HostClient fix-1 stream lifecycle', () => {
    test('tauri_channel_result_completes_execute', async () => {
        let handler: ((message: unknown) => void) | null = null;
        let channelClosed = 0;
        const commands: string[] = [];
        const bridge: TauriHostBridge = {
            hasChannel: true,
            async invoke(command) {
                commands.push(command);
                if (command === 'agent_host_invoke') return { accepted: true };
                if (command === 'agent_subscribe') return null;
                if (command === 'agent_host_capabilities') return ALL_CAPS;
                throw new Error(`unexpected Tauri command ${command}`);
            },
            channel(onMessage): TauriChannelHandle {
                handler = onMessage;
                return {
                    onmessage: onMessage,
                    close: () => {
                        channelClosed += 1;
                    },
                };
            },
        };
        const client = new HostClient({
            transport: new TauriHostTransport({ bridge }),
            timers: new ManualScheduler(),
            pendingTimeoutMs: 1_000,
        });
        const intent = await validIntent();
        const pending = client.execute(intent);
        await until(() => handler !== null, 'Tauri channel subscription', 500);
        const emit = handler as unknown as (message: unknown) => void;
        emit(await validResult(intent));
        await expect(pending).resolves.toMatchObject({
            intent_id: intent.intent_id,
        });
        expect(commands).toContain('agent_subscribe');
        client.close();
        expect(channelClosed).toBe(1);
    });

    test('tauri_channel_mismatch_rejects_execute_without_completion', async () => {
        let handler: ((message: unknown) => void) | null = null;
        const bridge: TauriHostBridge = {
            hasChannel: true,
            async invoke(command) {
                if (command === 'agent_host_invoke') return { accepted: true };
                if (command === 'agent_subscribe') return null;
                throw new Error(`unexpected Tauri command ${command}`);
            },
            channel(onMessage): TauriChannelHandle {
                handler = onMessage;
                return { onmessage: onMessage };
            },
        };
        const client = new HostClient({
            transport: new TauriHostTransport({ bridge }),
            timers: new ManualScheduler(),
        });
        const intent = await validIntent();
        const pending = client.execute(intent);
        await until(() => handler !== null, 'Tauri channel subscription', 500);
        const emit = handler as unknown as (message: unknown) => void;
        emit(
            await validResult(intent, {
                task_id: 'tsk_ffffffffffffffff',
            }),
        );
        await expect(pending).rejects.toMatchObject({
            code: 'cross_task_event',
        });
        client.close();
    });

    test('tauri_channel_direct_result_still_completes_execute', async () => {
        const bridge: TauriHostBridge = {
            hasChannel: true,
            async invoke(command, payload) {
                if (command === 'agent_host_invoke') {
                    const intent = (payload as { intent: HostToolIntent }).intent;
                    return validResult(intent);
                }
                if (command === 'agent_subscribe') return null;
                throw new Error(`unexpected Tauri command ${command}`);
            },
            channel(onMessage): TauriChannelHandle {
                return { onmessage: onMessage };
            },
        };
        const client = new HostClient({
            transport: new TauriHostTransport({ bridge }),
            timers: new ManualScheduler(),
        });
        const intent = await validIntent();
        await expect(client.execute(intent)).resolves.toMatchObject({
            intent_id: intent.intent_id,
        });
        client.close();
    });

    test('tauri_channel_close_suppresses_later_delivery', async () => {
        const handlers: Array<(message: unknown) => void> = [];
        const bridge: TauriHostBridge = {
            hasChannel: true,
            async invoke(command) {
                if (command === 'agent_subscribe') return null;
                throw new Error(`unexpected Tauri command ${command}`);
            },
            channel(onMessage): TauriChannelHandle {
                handlers.push(onMessage);
                return { onmessage: onMessage };
            },
        };
        const transport = new TauriHostTransport({ bridge });
        const first: HostToolResult[] = [];
        const firstSubscription = transport.subscribe((result) =>
            first.push(result),
        );
        await flush();
        expect(handlers).toHaveLength(1);
        firstSubscription.close();
        handlers[0]!(await validResult(await validIntent()));
        expect(first).toHaveLength(0);

        const second: HostToolResult[] = [];
        const secondSubscription = transport.subscribe((result) =>
            second.push(result),
        );
        await flush();
        expect(handlers).toHaveLength(2);
        handlers[0]!(await validResult(await validIntent()));
        expect(first).toHaveLength(0);
        expect(second).toHaveLength(0);
        handlers[1]!(
            await validResult(
                await validIntent({
                    intent_id: 'hti_0123456789abcdef_000000000002',
                }),
            ),
        );
        expect(second).toHaveLength(1);
        secondSubscription.close();
    });

    test('tauri_offline_subscription_polls_without_invoke', async () => {
        const scheduler = new ManualScheduler();
        const commands: string[] = [];
        const cursors: Array<string | null> = [];
        const result = await validResult(await validIntent());
        const bridge: TauriHostBridge = {
            hasChannel: false,
            async invoke(command, payload) {
                commands.push(command);
                if (command !== 'agent_host_poll') {
                    throw new Error(`unexpected Tauri command ${command}`);
                }
                const cursor = (payload as { cursor?: string | null }).cursor;
                cursors.push(cursor ?? null);
                return cursor === null
                    ? { cursor: 'c1', results: [result] }
                    : { cursor: 'c1', results: [] };
            },
            channel(): TauriChannelHandle {
                throw new Error('the offline shim has no Channel');
            },
        };
        const transport = new TauriHostTransport({
            bridge,
            scheduler,
            pollIntervalMs: 10,
        });
        const received: HostToolResult[] = [];
        const subscription = transport.subscribe((value) =>
            received.push(value),
        );
        await until(() => received.length === 1, 'offline poll delivery', 500);
        expect(commands).not.toContain('agent_host_invoke');
        expect(cursors).toEqual([null]);
        expect(subscription.cursor()).toBe('c1');
        scheduler.runNext();
        await flush();
        expect(cursors).toEqual([null, 'c1']);
        subscription.close();
        expect(scheduler.pending).toBe(0);
    });

    test('tauri_offline_subscription_reports_poll_error_and_stops', async () => {
        const scheduler = new ManualScheduler();
        const bridge: TauriHostBridge = {
            hasChannel: false,
            async invoke() {
                throw new Error('poll failed');
            },
            channel(): TauriChannelHandle {
                throw new Error('the offline shim has no Channel');
            },
        };
        const transport = new TauriHostTransport({
            bridge,
            scheduler,
            pollIntervalMs: 10,
        });
        const errors: AgentClientError[] = [];
        transport.subscribe(() => undefined, {
            onError: (error) => errors.push(error),
        });
        await until(() => errors.length === 1, 'poll error', 500);
        expect(errors[0]?.code).toBe('internal');
        expect(scheduler.pending).toBe(0);
    });

    test('loopback_stream_result_completes_execute', async () => {
        const intent = await validIntent();
        const result = await validResult(intent);
        const calls: string[] = [];
        const fetchImpl = (async (
            input: RequestInfo | URL,
            init: RequestInit = {},
        ): Promise<Response> => {
            const url = String(input);
            calls.push(`${init.method ?? 'GET'} ${url}`);
            if (url.includes('/v1/studio/host/intents')) {
                return new Response(JSON.stringify({ accepted: true }), {
                    status: 202,
                    headers: { 'content-type': 'application/json' },
                });
            }
            const stream = new ReadableStream<Uint8Array>({
                start(controller) {
                    controller.enqueue(
                        new TextEncoder().encode(`${JSON.stringify(result)}\n`),
                    );
                    controller.close();
                },
            });
            return new Response(stream, {
                status: 200,
                headers: { 'content-type': 'application/x-ndjson' },
            });
        }) as typeof fetch;
        const client = new HostClient({
            transport: new LoopbackHostTransport({
                token: 'launch-token',
                baseUrl: 'http://127.0.0.1:4173',
                fetch: fetchImpl,
            }),
            timers: new ManualScheduler(),
        });
        await expect(client.execute(intent)).resolves.toMatchObject({
            intent_id: intent.intent_id,
        });
        expect(calls[0]).toContain('GET');
        expect(calls[0]).toContain('/v1/studio/host/results');
        expect(calls.some((call) => call.startsWith('POST'))).toBe(true);
        client.close();
    });

    test('loopback_post_result_still_completes_execute', async () => {
        const intent = await validIntent();
        const result = await validResult(intent);
        const fetchImpl = (async (
            input: RequestInfo | URL,
            init: RequestInit = {},
        ): Promise<Response> => {
            if (String(input).includes('/v1/studio/host/intents')) {
                return new Response(JSON.stringify(result), {
                    status: 200,
                    headers: { 'content-type': 'application/json' },
                });
            }
            return new Response(
                new ReadableStream<Uint8Array>({
                    start(controller) {
                        controller.close();
                    },
                }),
                {
                    status: 200,
                    headers: { 'content-type': 'application/x-ndjson' },
                },
            );
        }) as typeof fetch;
        const client = new HostClient({
            transport: new LoopbackHostTransport({
                token: 'launch-token',
                baseUrl: 'http://127.0.0.1:4173',
                fetch: fetchImpl,
            }),
            timers: new ManualScheduler(),
        });
        await expect(client.execute(intent)).resolves.toMatchObject({
            intent_id: intent.intent_id,
        });
        client.close();
    });

    test('loopback_abort_rejects_pending_execute', async () => {
        const intent = await validIntent();
        const fetchImpl = (async (
            input: RequestInfo | URL,
            init: RequestInit = {},
        ): Promise<Response> => {
            if (String(input).includes('/v1/studio/host/intents')) {
                return new Response(JSON.stringify({ accepted: true }), {
                    status: 202,
                    headers: { 'content-type': 'application/json' },
                });
            }
            return new Response(
                new ReadableStream<Uint8Array>({
                    start() {
                        // Leaves the stream open until the client aborts.
                    },
                }),
                {
                    status: 200,
                    headers: { 'content-type': 'application/x-ndjson' },
                    signal: init.signal ?? undefined,
                } as ResponseInit,
            );
        }) as typeof fetch;
        const client = new HostClient({
            transport: new LoopbackHostTransport({
                token: 'launch-token',
                baseUrl: 'http://127.0.0.1:4173',
                fetch: fetchImpl,
            }),
            timers: new ManualScheduler(),
        });
        const controller = new AbortController();
        const pending = client.execute(intent, {
            signal: controller.signal,
        });
        await flush();
        controller.abort();
        await expect(pending).rejects.toMatchObject({
            code: 'transport_closed',
        });
        client.close();
    });

    test('loopback_base_must_be_local_http', () => {
        expect(
            () =>
                new LoopbackHostTransport({
                    token: 'launch-token',
                    pageOrigin: 'https://example.com',
                }),
        ).toThrowError(AgentClientError);
        expect(
            () =>
                new LoopbackHostTransport({
                    token: 'launch-token',
                    baseUrl: 'https://127.0.0.1:4173',
                }),
        ).toThrowError(AgentClientError);
        expect(
            () =>
                new LoopbackHostTransport({
                    token: 'launch-token',
                    baseUrl: 'http://example.com:4173',
                }),
        ).toThrowError(AgentClientError);
        expect(
            () =>
                new LoopbackHostTransport({
                    token: 'launch-token',
                    baseUrl: 'http://127.0.0.1.evil:4173',
                }),
        ).toThrowError(AgentClientError);
        expect(
            () =>
                new LoopbackHostTransport({
                    token: 'launch-token',
                    baseUrl: 'http://localhost:4173/',
                }),
        ).not.toThrow();
        expect(
            () =>
                new LoopbackHostTransport({
                    token: 'launch-token',
                    pageOrigin: 'http://localhost:4173',
                }),
        ).not.toThrow();
    });
});

describe('HostClient fix-1 authority and mapping', () => {
    test('data_scope_marks_wire_and_binds_result', async () => {
        const transport = new FakeHostTransport();
        transport.capabilityValues = { ...ALL_FALSE, tree: true };
        transport.resultFactory = async (intent) =>
            validResult(intent, {
                output: {
                    entries: [{ path: 'src', kind: 'directory' }],
                },
            });
        const client = new AgentClient({
            hostClient: new HostClient({ transport }),
        });
        await expect(
            client.tree({ source: 'host', path: 'src' }),
        ).resolves.toEqual([{ path: 'src', kind: 'directory' }]);
        expect(transport.intents[0]).toMatchObject({
            request_scope: 'data',
            task_id: 'tsk_0000000000000000',
            run_id: 'run_0000000000000000',
            generation: 1,
        });

        const wrongScope = new FakeHostTransport();
        wrongScope.capabilityValues = { ...ALL_FALSE, tree: true };
        wrongScope.resultFactory = async (intent) => ({
            ...(await validResult(intent, {
                output: { entries: [] },
            })),
            request_scope: 'task',
        });
        const wrongClient = new AgentClient({
            hostClient: new HostClient({ transport: wrongScope }),
        });
        await expect(
            wrongClient.tree({ source: 'host' }),
        ).rejects.toMatchObject({ code: 'out_of_order' });
    });

    test('data_context_omission_is_preserved_in_the_legacy_bridge', async () => {
        const recording = new RecordingAgentTransport();
        const client = new AgentClient({
            hostClient: new HostClient({
                transport: new AgentTransportHostTransport(recording),
            }),
        });
        await expect(
            client.changesSnapshot({ taskId: TASK_ID, scope: 'branch' }),
        ).resolves.toMatchObject({
            scope: 'branch',
            headDigest: `sha256:${'c'.repeat(64)}`,
        });
        expect(recording.changesInputs).toHaveLength(1);
        expect(recording.changesInputs[0]).toMatchObject({
            taskId: TASK_ID,
            scope: 'branch',
        });
        expect(recording.changesInputs[0]?.runId).toBeUndefined();
    });

    test('artifact_read_verifies_raw_digest_and_size', async () => {
        const bytes = new TextEncoder().encode('hello');
        const digest = await sha256Raw(bytes);
        const transport = new FakeHostTransport();
        transport.capabilityValues = { ...ALL_FALSE, readArtifact: true };
        transport.resultFactory = async (intent) =>
            validResult(intent, {
                output: {
                    path: 'a.txt',
                    media_type: 'text/plain',
                    size_bytes: bytes.byteLength,
                    digest,
                    bytes_base64: globalThis.btoa('hello'),
                },
            });
        const client = new AgentClient({
            hostClient: new HostClient({ transport }),
        });
        const content = await client.readArtifact({ path: 'a.txt' });
        expect(new TextDecoder().decode(content.bytes)).toBe('hello');
        expect(content.digest).toBe(digest);

        const corrupt = new FakeHostTransport();
        corrupt.capabilityValues = { ...ALL_FALSE, readArtifact: true };
        corrupt.resultFactory = async (intent) =>
            validResult(intent, {
                output: {
                    path: 'a.txt',
                    media_type: 'text/plain',
                    size_bytes: bytes.byteLength,
                    digest: `sha256:${'0'.repeat(64)}`,
                    bytes_base64: globalThis.btoa('hello'),
                },
            });
        const corruptClient = new AgentClient({
            hostClient: new HostClient({ transport: corrupt }),
        });
        await expect(
            corruptClient.readArtifact({ path: 'a.txt' }),
        ).rejects.toMatchObject({ code: 'invalid_event' });

        const wrongSize = new FakeHostTransport();
        wrongSize.capabilityValues = { ...ALL_FALSE, readArtifact: true };
        wrongSize.resultFactory = async (intent) =>
            validResult(intent, {
                output: {
                    path: 'a.txt',
                    media_type: 'text/plain',
                    size_bytes: 99,
                    digest,
                    bytes_base64: globalThis.btoa('hello'),
                },
            });
        const wrongSizeClient = new AgentClient({
            hostClient: new HostClient({ transport: wrongSize }),
        });
        await expect(
            wrongSizeClient.readArtifact({ path: 'a.txt' }),
        ).rejects.toMatchObject({ code: 'invalid_event' });
    });

    test('branch_snapshot_records_digest_and_timestamp_are_validated', async () => {
        const validDigest = `sha256:${'d'.repeat(64)}`;
        const makeClient = (snapshot: Record<string, unknown>) => {
            const transport = new FakeHostTransport();
            transport.capabilityValues = { ...ALL_FALSE, changes: true };
            transport.resultFactory = async (intent) =>
                validResult(intent, { output: { snapshot } });
            return new AgentClient({
                hostClient: new HostClient({ transport }),
            });
        };
        await expect(
            makeClient({
                scope: 'branch',
                files: [42],
                head_digest: validDigest,
                captured_at_ms: 1,
            }).changesSnapshot({ taskId: TASK_ID, scope: 'branch' }),
        ).rejects.toMatchObject({ code: 'invalid_event' });
        await expect(
            makeClient({
                scope: 'branch',
                files: [{ path: 'a.ts' }],
                head_digest: 'not-a-digest',
                captured_at_ms: 1,
            }).changesSnapshot({ taskId: TASK_ID, scope: 'branch' }),
        ).rejects.toMatchObject({ code: 'invalid_event' });
        await expect(
            makeClient({
                scope: 'branch',
                files: [{ path: 'a.ts' }],
                head_digest: validDigest,
                captured_at_ms: -1,
            }).changesSnapshot({ taskId: TASK_ID, scope: 'branch' }),
        ).rejects.toMatchObject({ code: 'invalid_arguments' });
        await expect(
            makeClient({
                scope: 'branch',
                files: [{ path: 'a.ts' }],
                head_digest: validDigest,
                captured_at_ms: 1,
            }).changesSnapshot({ taskId: TASK_ID, scope: 'branch' }),
        ).resolves.toMatchObject({
            scope: 'branch',
            headDigest: validDigest,
            capturedAtMs: 1,
        });
    });

    test('execute_null_and_primitive_inputs_reject_invalid_arguments', async () => {
        const client = new HostClient({ transport: new FakeHostTransport() });
        for (const value of [null, 42, 'intent', []]) {
            await expect(
                Promise.resolve().then(() =>
                    client.execute(value as never),
                ),
            ).rejects.toMatchObject({ code: 'invalid_arguments' });
        }
    });

    test('host_client_only_composition_capabilities_and_approval', async () => {
        const staticClient = new AgentClient({
            hostClient: new HostClient({
                transport: new StaticHostTransport(),
            }),
        });
        await expect(staticClient.hostCapabilities()).resolves.toEqual(
            ALL_FALSE,
        );

        const transport = new FakeHostTransport();
        const approvalClient = new AgentClient({
            hostClient: new HostClient({ transport }),
        });
        await expect(
            approvalClient.resolveApproval({
                approvalId: 'apr_5c2e8a1f9b7d3e40',
                decision: 'approved',
            }),
        ).resolves.toEqual({
            approvalId: 'apr_5c2e8a1f9b7d3e40',
            decision: 'approved',
        });
        expect(transport.approvals).toEqual([
            {
                approvalId: 'apr_5c2e8a1f9b7d3e40',
                decision: 'approved',
            },
        ]);

        await expect(
            new AgentClient().hostCapabilities(),
        ).rejects.toMatchObject({ code: 'runtime_unavailable' });
    });

    test('loopback_error_mapping_uses_the_generated_taxonomy', async () => {
        const intent = await validIntent();
        const makeTransport = (code: string) =>
            new LoopbackHostTransport({
                token: 'launch-token',
                baseUrl: 'http://127.0.0.1:4173',
                fetch: (async () =>
                    new Response(JSON.stringify({ code }), {
                        status: 409,
                        headers: { 'content-type': 'application/json' },
                    })) as typeof fetch,
            });
        await expect(
            makeTransport('snapshot_missing').invoke(intent),
        ).rejects.toMatchObject({ code: 'snapshot_missing' });
        await expect(
            makeTransport('quota_exceeded').invoke(intent),
        ).rejects.toMatchObject({ code: 'host_unavailable' });
    });

    test('static_approval_text_matches_the_spec', async () => {
        const client = new HostClient({
            transport: new StaticHostTransport(),
        });
        await expect(client.execute(await validIntent())).rejects.toMatchObject(
            {
                message: expect.stringContaining(
                    'This action needs host approval. Approve it in the local companion.',
                ),
            },
        );
    });

    test('output_preview_slicing_never_splits_a_surrogate_pair', async () => {
        const transport = new TreeOnlyAgentTransport();
        const adapter = new AgentTransportHostTransport(transport);
        const intent = await dataIntent('file.tree');
        for (let padding = 450; padding < 500; padding += 1) {
            transport.entries = [
                {
                    path: `${'a'.repeat(padding)}😀`,
                    kind: 'file',
                },
            ];
            const result = await adapter.invoke(intent);
            expect(hasLoneSurrogate(result.output_preview)).toBe(false);
        }
    });
});

describe('HostClient fix-1 pending failure', () => {
    test('tauri_channel_malformed_result_rejects_execute', async () => {
        let handler: ((message: unknown) => void) | null = null;
        const bridge: TauriHostBridge = {
            hasChannel: true,
            async invoke(command) {
                if (command === 'agent_host_invoke') return { accepted: true };
                if (command === 'agent_host_subscribe') return null;
                if (command === 'agent_subscribe') return null;
                throw new Error(`unexpected Tauri command ${command}`);
            },
            channel(onMessage): TauriChannelHandle {
                handler = onMessage;
                return { onmessage: onMessage };
            },
        };
        const client = new HostClient({
            transport: new TauriHostTransport({ bridge }),
            timers: new ManualScheduler(),
        });
        const intent = await validIntent();
        const pending = client.execute(intent);
        await until(() => handler !== null, 'Tauri channel subscription', 500);
        const emit = handler as unknown as (message: unknown) => void;
        emit({
            schema: HOST_RESULT_SCHEMA,
            intent_id: intent.intent_id,
            task_id: intent.task_id,
            run_id: intent.run_id,
            generation: intent.generation,
            tool_call_id: intent.tool_call_id,
            status: 'not-a-status',
        });
        await expect(pending).rejects.toMatchObject({ code: 'invalid_event' });
        client.close();
    });

    test('execute_timeout_rejects_pending_call', async () => {
        const scheduler = new ManualScheduler();
        const bridge: TauriHostBridge = {
            hasChannel: true,
            async invoke(command) {
                if (command === 'agent_host_invoke') return { accepted: true };
                if (command === 'agent_subscribe') return null;
                throw new Error(`unexpected Tauri command ${command}`);
            },
            channel(onMessage): TauriChannelHandle {
                return { onmessage: onMessage };
            },
        };
        const client = new HostClient({
            transport: new TauriHostTransport({ bridge }),
            timers: scheduler,
            pendingTimeoutMs: 10,
        });
        const pending = client.execute(await validIntent(), { timeoutMs: 10 });
        await until(
            () => scheduler.pending === 1,
            'pending timeout scheduled',
            500,
        );
        expect(scheduler.runNext()).toBe(true);
        await expect(pending).rejects.toMatchObject({
            code: 'transport_closed',
        });
        client.close();
    });

    test('host_client_close_rejects_pending_and_ignores_later_results', async () => {
        let handler: ((message: unknown) => void) | null = null;
        const bridge: TauriHostBridge = {
            hasChannel: true,
            async invoke(command) {
                if (command === 'agent_host_invoke') return { accepted: true };
                if (command === 'agent_subscribe') return null;
                throw new Error(`unexpected Tauri command ${command}`);
            },
            channel(onMessage): TauriChannelHandle {
                handler = onMessage;
                return { onmessage: onMessage };
            },
        };
        const client = new HostClient({
            transport: new TauriHostTransport({ bridge }),
            timers: new ManualScheduler(),
        });
        const intent = await validIntent();
        const pending = client.execute(intent);
        await until(() => handler !== null, 'Tauri channel subscription', 500);
        client.close();
        await expect(pending).rejects.toMatchObject({
            code: 'transport_closed',
        });
        const emit = handler as unknown as (message: unknown) => void;
        emit(await validResult(intent));
        await flush();
    });

    test('host_result_quota_exceeded_is_not_a_local_code', async () => {
        const transport = new FakeHostTransport();
        transport.resultFactory = async (intent) => ({
            ...(await validResult(intent)),
            status: 'error',
            code: 'quota_exceeded',
            message: 'service-only',
        });
        await expect(
            new HostClient({ transport }).execute(await validIntent()),
        ).rejects.toMatchObject({ code: 'host_unavailable' });
    });
});

async function disposition(promise: Promise<unknown>): Promise<string> {
    let outcome = 'pending';
    void promise.then(
        () => {
            outcome = 'resolved';
        },
        (error: unknown) => {
            outcome = `rejected:${
                error instanceof AgentClientError ? error.code : 'unknown'
            }`;
        },
    );
    await flush();
    return outcome;
}

describe('HostClient fix-2 ownership and recovery', () => {
    test('runtime_dispose_closes_owned_host_client_and_rejects_pending_poll', async () => {
        const scheduler = new ManualScheduler();
        let polls = 0;
        const bridge: TauriHostBridge = {
            hasChannel: false,
            async invoke(command) {
                if (command === 'agent_host_invoke') return { accepted: true };
                if (command === 'agent_host_poll') {
                    polls += 1;
                    return { cursor: 'c1', results: [] };
                }
                throw new Error(`unexpected Tauri command ${command}`);
            },
            channel(): TauriChannelHandle {
                throw new Error('the offline shim has no Channel');
            },
        };
        const host = new HostClient({
            transport: new TauriHostTransport({
                bridge,
                scheduler,
                pollIntervalMs: 10,
            }),
            timers: scheduler,
        });
        const runtime = new AgentRuntime({
            store: createAgentStore(),
            hostClient: host,
            modelPort: new ReplayModelPort(),
        });
        await runtime.open();
        const pending = host.execute(await validIntent());
        void pending.catch(() => undefined);
        await until(() => polls >= 1, 'runtime poll started', 500);
        await runtime.dispose();
        await expect(disposition(pending)).resolves.toBe(
            'rejected:transport_closed',
        );
        const pollsAtDispose = polls;
        scheduler.runNext();
        await flush();
        expect(polls).toBe(pollsAtDispose);
    });

    test('runtime_and_client_dispose_close_runtime_host_client_once', async () => {
        const host = new HostClient({
            transport: new FakeHostTransport(),
        });
        const close = vi.spyOn(host, 'close');
        const runtime = new AgentRuntime({
            store: createAgentStore(),
            hostClient: host,
            modelPort: new ReplayModelPort(),
        });
        const client = new AgentClient({ runtime });
        await runtime.open();
        await client.dispose();
        expect(close).toHaveBeenCalledTimes(1);
    });

    test('client_dispose_closes_client_created_host_client', async () => {
        const transport = new PendingTreeTransport();
        const client = new AgentClient({ transport });
        const pending = client.tree({ source: 'host' });
        void pending.catch(() => undefined);
        await until(() => transport.treeStarted, 'pending tree request', 500);
        await client.dispose();
        await expect(disposition(pending)).resolves.toBe(
            'rejected:transport_closed',
        );
    });

    test('client_dispose_does_not_close_external_host_client', async () => {
        const external = new HostClient({
            transport: new FakeHostTransport(),
        });
        const client = new AgentClient({ hostClient: external });
        await client.dispose();
        await expect(
            external.execute(await validIntent()),
        ).resolves.toMatchObject({ status: 'ok' });
    });

    test('poll_error_resubscribes_for_the_next_execute', async () => {
        const scheduler = new ManualScheduler();
        const firstIntent = await validIntent({
            intent_id: 'hti_0123456789abcdef_000000000011',
        });
        const secondIntent = await validIntent({
            intent_id: 'hti_0123456789abcdef_000000000012',
        });
        const secondResult = await validResult(secondIntent);
        let polls = 0;
        const bridge: TauriHostBridge = {
            hasChannel: false,
            async invoke(command) {
                if (command === 'agent_host_invoke') return { accepted: true };
                if (command === 'agent_host_poll') {
                    polls += 1;
                    if (polls === 1) throw new Error('poll failed');
                    return { cursor: 'c1', results: [secondResult] };
                }
                throw new Error(`unexpected Tauri command ${command}`);
            },
            channel(): TauriChannelHandle {
                throw new Error('the offline shim has no Channel');
            },
        };
        const client = new HostClient({
            transport: new TauriHostTransport({
                bridge,
                scheduler,
                pollIntervalMs: 10,
            }),
            timers: scheduler,
        });
        await expect(client.execute(firstIntent)).rejects.toMatchObject({
            code: 'internal',
        });
        const second = client.execute(secondIntent);
        void second.catch(() => undefined);
        await until(() => polls === 2, 'poll resubscribe', 500);
        await expect(second).resolves.toMatchObject({
            intent_id: secondIntent.intent_id,
        });
        client.close();
    });

    test('loopback_cursor_expired_resubscribes_for_the_next_execute', async () => {
        const scheduler = new ManualScheduler();
        const firstIntent = await validIntent({
            intent_id: 'hti_0123456789abcdef_000000000021',
        });
        const secondIntent = await validIntent({
            intent_id: 'hti_0123456789abcdef_000000000022',
        });
        const secondResult = await validResult(secondIntent);
        let gets = 0;
        const fetchImpl = (async (
            input: RequestInfo | URL,
            init: RequestInit = {},
        ): Promise<Response> => {
            if (String(input).includes('/v1/studio/host/intents')) {
                return new Response(JSON.stringify({ accepted: true }), {
                    status: 202,
                    headers: { 'content-type': 'application/json' },
                });
            }
            gets += 1;
            if (gets === 1) {
                return new Response(
                    JSON.stringify({ code: 'cursor_expired' }),
                    {
                        status: 410,
                        headers: { 'content-type': 'application/json' },
                    },
                );
            }
            const stream = new ReadableStream<Uint8Array>({
                start(controller) {
                    controller.enqueue(
                        new TextEncoder().encode(
                            `${JSON.stringify(secondResult)}\n`,
                        ),
                    );
                    controller.close();
                },
            });
            return new Response(stream, {
                status: 200,
                headers: { 'content-type': 'application/x-ndjson' },
            });
        }) as typeof fetch;
        const client = new HostClient({
            transport: new LoopbackHostTransport({
                token: 'launch-token',
                baseUrl: 'http://127.0.0.1:4173',
                fetch: fetchImpl,
            }),
            timers: scheduler,
        });
        await expect(client.execute(firstIntent)).rejects.toMatchObject({
            code: 'cursor_expired',
        });
        const second = client.execute(secondIntent);
        void second.catch(() => undefined);
        await until(() => gets === 2, 'loopback resubscribe', 500);
        await expect(second).resolves.toMatchObject({
            intent_id: secondIntent.intent_id,
        });
        client.close();
    });

    test('stale_stream_generation_cannot_settle_a_new_execute', async () => {
        const transport = new ManualStreamTransport();
        const client = new HostClient({
            transport,
            timers: new ManualScheduler(),
        });
        const firstIntent = await validIntent({
            intent_id: 'hti_0123456789abcdef_000000000031',
        });
        const secondIntent = await validIntent({
            intent_id: 'hti_0123456789abcdef_000000000032',
        });
        const first = client.execute(firstIntent);
        void first.catch(() => undefined);
        await until(
            () => transport.subscriptions.length === 1,
            'first stream subscription',
            2_000,
        );
        expect(transport.subscriptions).toHaveLength(1);
        transport.subscriptions[0]!.options.onError?.(
            new AgentClientError('internal', 'stream failed'),
        );
        await expect(first).rejects.toMatchObject({ code: 'internal' });

        const second = client.execute(secondIntent);
        void second.catch(() => undefined);
        await until(
            () => transport.subscriptions.length === 2,
            'stream resubscribe',
            2_000,
        );
        let settled = false;
        void second.then(
            () => {
                settled = true;
            },
            () => {
                settled = true;
            },
        );
        transport.subscriptions[0]!.listener(
            await validResult(secondIntent),
        );
        await flush();
        expect(settled).toBe(false);
        transport.subscriptions[1]!.listener(
            await validResult(secondIntent),
        );
        await expect(second).resolves.toMatchObject({
            intent_id: secondIntent.intent_id,
        });
        client.close();
    });

    test('tauri_transport_close_suppresses_later_channel_delivery', async () => {
        let handler: ((message: unknown) => void) | null = null;
        const bridge: TauriHostBridge = {
            hasChannel: true,
            async invoke(command) {
                if (command === 'agent_subscribe') return null;
                throw new Error(`unexpected Tauri command ${command}`);
            },
            channel(onMessage): TauriChannelHandle {
                handler = onMessage;
                return { onmessage: onMessage };
            },
        };
        const transport = new TauriHostTransport({ bridge });
        const received: HostToolResult[] = [];
        transport.subscribe((result) => received.push(result));
        await flush();
        expect(handler).not.toBeNull();
        transport.close();
        const emit = handler as unknown as (message: unknown) => void;
        emit(await validResult(await validIntent()));
        expect(received).toHaveLength(0);
    });

    test('non_streaming_subscribe_is_host_unavailable', () => {
        const client = new HostClient({
            transport: new AgentTransportHostTransport(
                new FixtureTransport(),
            ),
        });
        expect(() => client.subscribe(() => undefined)).toThrowError(
            AgentClientError,
        );
        try {
            client.subscribe(() => undefined);
        } catch (error) {
            expect(error).toMatchObject({ code: 'host_unavailable' });
        }
    });

    test('data_scope_artifact_read_preserves_context_task_id', async () => {
        const recording = new RecordingAgentTransport();
        const client = new AgentClient({
            hostClient: new HostClient({
                transport: new AgentTransportHostTransport(recording),
            }),
        });
        const content = await client.readArtifact({
            path: 'a.txt',
            taskId: TASK_ID,
        });
        expect(new TextDecoder().decode(content.bytes)).toBe('hello');
        expect(recording.artifactInputs[0]?.taskId).toBe(TASK_ID);
    });

    test('task_scope_artifact_read_uses_the_intent_task_id', async () => {
        const recording = new RecordingAgentTransport();
        const adapter = new AgentTransportHostTransport(recording);
        const result = await adapter.invoke(
            await validIntent({
                tool: 'artifact.read',
                arguments: { path: 'a.txt' },
                request_scope: 'task',
            }),
        );
        expect(result.status).toBe('ok');
        expect(recording.artifactInputs[0]?.taskId).toBe(TASK_ID);
    });

    test('task_scope_rejects_zero_task_and_run_placeholders', async () => {
        const client = new HostClient({
            transport: new FakeHostTransport(),
        });
        await expect(
            client.execute(
                await validIntent({
                    task_id: 'tsk_0000000000000000',
                    run_id: 'run_0000000000000000',
                    generation: 1,
                    request_scope: 'task',
                }),
            ),
        ).rejects.toMatchObject({ code: 'invalid_arguments' });
    });

    test('data_result_requires_explicit_zero_identity', async () => {
        const transport = new FakeHostTransport();
        transport.capabilityValues = { ...ALL_FALSE, tree: true };
        transport.resultFactory = async (intent) => {
            const value = await validResult(intent, {
                output: { entries: [] },
            });
            const record = value as unknown as Record<string, unknown>;
            delete record.task_id;
            delete record.run_id;
            delete record.generation;
            return value;
        };
        const client = new AgentClient({
            hostClient: new HostClient({ transport }),
        });
        await expect(
            client.tree({ source: 'host' }),
        ).rejects.toMatchObject({ code: 'cross_task_event' });
    });

    test('loopback_rejects_non_string_base_and_path_bearing_base', () => {
        expect(
            () =>
                new LoopbackHostTransport({
                    token: 'launch-token',
                    baseUrl: 42 as never,
                }),
        ).toThrowError(AgentClientError);
        expect(
            () =>
                new LoopbackHostTransport({
                    token: 'launch-token',
                    pageOrigin: {} as never,
                }),
        ).toThrowError(AgentClientError);
        expect(
            () =>
                new LoopbackHostTransport({
                    token: 'launch-token',
                    baseUrl: 'http://localhost:4173/path',
                }),
        ).toThrowError(AgentClientError);
    });

    test('host_client_execute_through_the_poll_path', async () => {
        const scheduler = new ManualScheduler();
        const intent = await validIntent();
        const result = await validResult(intent);
        let polls = 0;
        const bridge: TauriHostBridge = {
            hasChannel: false,
            async invoke(command) {
                if (command === 'agent_host_invoke') return { accepted: true };
                if (command === 'agent_host_poll') {
                    polls += 1;
                    return { cursor: 'c1', results: [result] };
                }
                throw new Error(`unexpected Tauri command ${command}`);
            },
            channel(): TauriChannelHandle {
                throw new Error('the offline shim has no Channel');
            },
        };
        const client = new HostClient({
            transport: new TauriHostTransport({
                bridge,
                scheduler,
                pollIntervalMs: 10,
            }),
            timers: scheduler,
        });
        await expect(client.execute(intent)).resolves.toMatchObject({
            intent_id: intent.intent_id,
        });
        expect(polls).toBe(1);
        client.close();
    });
});

class ThrowingCloseTransport implements HostTransport {
    readonly kind = 'bridge' as const;
    readonly supportsStream = true;
    readonly subscriptions: Array<{
        options: HostSubscribeOptions;
    }> = [];

    async capabilities(): Promise<HostTransportCapabilities> {
        return {
            kind: 'bridge',
            hostAvailable: true,
            streaming: 'channel',
            ...ALL_FALSE,
        };
    }

    async invoke(): Promise<{ accepted: true }> {
        return { accepted: true };
    }

    subscribe(
        _listener: (result: HostToolResult) => void,
        options: HostSubscribeOptions = {},
    ): HostSubscription {
        this.subscriptions.push({ options });
        return {
            close: () => {
                throw new Error('subscription close failed');
            },
            cursor: () => null,
        };
    }

    async approve(input: {
        approvalId: string;
        decision: 'approved' | 'denied';
    }) {
        return { approvalId: input.approvalId, decision: input.decision };
    }
}

describe('HostClient fix-3 disposal and hardening', () => {
    test('runtime_dispose_closes_host_before_awaiting_a_pending_tool_chain', async () => {
        const scheduler = new ManualScheduler();
        let polls = 0;
        const bridge: TauriHostBridge = {
            hasChannel: false,
            async invoke(command) {
                if (command === 'agent_host_invoke') return { accepted: true };
                if (command === 'agent_host_poll') {
                    polls += 1;
                    return { cursor: 'c1', results: [] };
                }
                throw new Error(`unexpected Tauri command ${command}`);
            },
            channel(): TauriChannelHandle {
                throw new Error('the offline shim has no Channel');
            },
        };
        const host = new HostClient({
            transport: new TauriHostTransport({
                bridge,
                scheduler,
                pollIntervalMs: 10,
            }),
            timers: scheduler,
        });
        const originalExecute = host.execute.bind(host);
        let hostPending: Promise<HostToolResult> | null = null;
        vi.spyOn(host, 'execute').mockImplementation(((
            input: unknown,
            options?: HostExecuteOptions,
        ) => {
            const pending = originalExecute(
                input as HostToolIntent,
                options,
            );
            hostPending = pending;
            return pending;
        }) as typeof host.execute);
        const model = new ReplayModelPort({
            script: [
                [
                    {
                        type: 'tool-call',
                        toolCallId: 'tc_host_dispose',
                        tool: 'file.read',
                        arguments: { path: 'a.ts' },
                    },
                    { type: 'finished', reason: 'tool' },
                ],
                [
                    { type: 'text-delta', text: 'unused' },
                    { type: 'finished', reason: 'stop' },
                ],
            ],
        });
        const runtime = new AgentRuntime({
            store: createAgentStore(),
            modelPort: model,
            hostClient: host,
        });
        await runtime.open();
        const task = await runtime.createTask({ title: 'host dispose' });
        await runtime.startRun({ taskId: task.taskId, prompt: 'read' });
        await until(() => hostPending !== null, 'runtime host call', 500);
        await until(() => polls >= 1, 'runtime host poll', 500);
        const pendingHost = hostPending as unknown as Promise<HostToolResult>;
        void pendingHost.catch(() => undefined);
        // Test-only: hold one runtime chain open so the old dispose ordering
        // (chains before host close) is observable. The model tool call above
        // is still the real ReplayModelPort call awaiting the host.
        const chains = (
            runtime as unknown as { chains: Map<string, Promise<void>> }
        ).chains;
        let releaseChain!: () => void;
        chains.set(
            'fix3-held-chain',
            new Promise<void>((resolve) => {
                releaseChain = resolve;
            }),
        );
        const dispose = runtime.dispose();
        await expect(disposition(pendingHost)).resolves.toBe(
            'rejected:transport_closed',
        );
        const pollsAtDispose = polls;
        scheduler.runNext();
        await flush();
        expect(polls).toBe(pollsAtDispose);
        releaseChain();
        await expect(dispose).resolves.toBeUndefined();
    });

    test('client_dispose_closes_task_store_when_owned_host_close_throws', async () => {
        const close = vi
            .spyOn(HostClient.prototype, 'close')
            .mockImplementation(() => {
                throw new Error('host close failed');
            });
        const taskStoreClose = vi.fn();
        const taskStore = {
            close: taskStoreClose,
            isOpen: true,
            defaultWriterId: 'writer-fix3',
        } as unknown as AgentTaskStore;
        const client = new AgentClient({
            transport: new FixtureTransport(),
            taskStore,
        });
        try {
            await expect(client.dispose()).rejects.toThrow('host close failed');
            expect(taskStoreClose).toHaveBeenCalledTimes(1);
        } finally {
            close.mockRestore();
        }
    });

    test('plain_value_invoke_settles_directly', async () => {
        const intent = await validIntent();
        const result = await validResult(intent);
        const transport: HostTransport = {
            kind: 'bridge',
            supportsStream: true,
            async capabilities() {
                return {
                    kind: 'bridge',
                    hostAvailable: true,
                    streaming: 'channel',
                    ...ALL_FALSE,
                };
            },
            invoke: (() => result) as unknown as HostTransport['invoke'],
            subscribe: () => ({
                close: () => undefined,
                cursor: () => null,
            }),
            async approve(input) {
                return {
                    approvalId: input.approvalId,
                    decision: input.decision,
                };
            },
        };
        const client = new HostClient({
            transport,
            timers: new ManualScheduler(),
        });
        await expect(client.execute(intent)).resolves.toMatchObject({
            intent_id: intent.intent_id,
        });
        client.close();
    });

    test('accepted_invoke_without_stream_support_rejects_without_timeout', async () => {
        const transport: HostTransport = {
            kind: 'bridge',
            async capabilities() {
                return {
                    kind: 'bridge',
                    hostAvailable: true,
                    streaming: 'none',
                    ...ALL_FALSE,
                };
            },
            async invoke() {
                return { accepted: true };
            },
            subscribe: () => ({
                close: () => undefined,
                cursor: () => null,
            }),
            async approve(input) {
                return {
                    approvalId: input.approvalId,
                    decision: input.decision,
                };
            },
        };
        const client = new HostClient({
            transport,
            timers: new ManualScheduler(),
        });
        const pending = client.execute(await validIntent());
        void pending.catch(() => undefined);
        const outcome = await Promise.race([
            pending.then(
                () => 'resolved',
                (error: unknown) =>
                    error instanceof AgentClientError
                        ? `rejected:${error.code}`
                        : 'rejected:unknown',
            ),
            new Promise<string>((resolve) =>
                globalThis.setTimeout(() => resolve('pending'), 2_000),
            ),
        ]);
        expect(outcome).toBe('rejected:host_unavailable');
        client.close();
    });

    test('throwing_subscription_close_does_not_block_pending_cleanup', async () => {
        const transport = new ThrowingCloseTransport();
        const client = new HostClient({
            transport,
            timers: new ManualScheduler(),
        });
        const pending = client.execute(await validIntent());
        void pending.catch(() => undefined);
        await until(
            () => transport.subscriptions.length === 1,
            'throwing close subscription',
            500,
        );
        let thrown: unknown = null;
        try {
            transport.subscriptions[0]!.options.onError?.(
                new AgentClientError('internal', 'stream failed'),
            );
        } catch (error) {
            thrown = error;
        }
        expect(thrown).toBeNull();
        await expect(pending).rejects.toMatchObject({ code: 'internal' });

        const second = client.execute(await validIntent());
        void second.catch(() => undefined);
        await until(
            () => transport.subscriptions.length === 2,
            'throwing close second subscription',
            500,
        );
        let closeThrown: unknown = null;
        try {
            client.close();
        } catch (error) {
            closeThrown = error;
        }
        expect(closeThrown).toBeNull();
        await expect(second).rejects.toMatchObject({
            code: 'transport_closed',
        });
    });

    test('data_artifact_omits_task_when_context_is_absent', async () => {
        const recording = new RecordingAgentTransport();
        const client = new AgentClient({
            hostClient: new HostClient({
                transport: new AgentTransportHostTransport(recording),
            }),
        });
        await client.readArtifact({ path: 'a.txt' });
        expect(recording.artifactInputs).toHaveLength(1);
        expect(recording.artifactInputs[0]?.taskId).toBeUndefined();
        expect('taskId' in (recording.artifactInputs[0] ?? {})).toBe(false);
    });

    test('subscribe_notifies_on_error_and_resync', async () => {
        const transport = new ManualStreamTransport();
        const client = new HostClient({
            transport,
            timers: new ManualScheduler(),
        });
        const errors: AgentClientError[] = [];
        const resyncs: string[] = [];
        const first = client.subscribe(() => undefined, {
            onError: (error) => errors.push(error),
        });
        await until(
            () => transport.subscriptions.length === 1,
            'subscriber subscription',
            500,
        );
        transport.subscriptions[0]!.options.onError?.(
            new AgentClientError('internal', 'stream failed'),
        );
        expect(errors).toHaveLength(1);
        expect(errors[0]?.code).toBe('internal');

        const second = client.subscribe(() => undefined, {
            onResync: (reason) => resyncs.push(reason),
        });
        await until(
            () => transport.subscriptions.length === 2,
            'subscriber resubscription',
            500,
        );
        transport.subscriptions[1]!.options.onResync?.('cursor_expired');
        expect(resyncs).toEqual(['cursor_expired']);
        first.close();
        second.close();
        client.close();
    });
});

describe('Tauri host bridge adapter', () => {
    test('tauri_bridge_uses_real_invoke_and_channel_only_when_exposed', async () => {
        class FakeChannel implements TauriChannelHandle {
            onmessage: (message: unknown) => void = () => undefined;
        }
        const calls: Array<{
            command: string;
            payload?: Record<string, unknown>;
        }> = [];
        const core: TauriCoreLike = {
            async invoke(command, payload) {
                calls.push({ command, payload });
                return { tree: true };
            },
            Channel: FakeChannel as unknown as new () => TauriChannelHandle,
        };

        const bridge = createTauriHostBridge(core);
        expect(bridge).not.toBeNull();
        expect(bridge?.hasChannel).toBe(true);
        await expect(
            bridge?.invoke('agent_host_capabilities'),
        ).resolves.toEqual({ tree: true });
        expect(calls).toEqual([
            { command: 'agent_host_capabilities', payload: undefined },
        ]);

        const handler = vi.fn();
        const channel = bridge?.channel(handler);
        expect(channel).toBeInstanceOf(FakeChannel);
        expect(channel?.onmessage).toBe(handler);

        expect(createTauriHostBridge(null)).toBeNull();
        expect(
            createTauriHostBridge({ invoke: async () => null }),
        ).toMatchObject({ hasChannel: false });

        const globalScope = globalThis as {
            __TAURI__?: { core?: TauriCoreLike };
        };
        const original = globalScope.__TAURI__;
        try {
            delete globalScope.__TAURI__;
            expect(createTauriHostBridge()).toBeNull();
            globalScope.__TAURI__ = { core };
            expect(createTauriHostBridge()).not.toBeNull();
            expect(createTauriHostBridge()?.hasChannel).toBe(true);
        } finally {
            if (original === undefined) delete globalScope.__TAURI__;
            else globalScope.__TAURI__ = original;
        }
    });

    test('tauri_transport_reads_host_grants', async () => {
        const commands: string[] = [];
        const bridge: TauriHostBridge = {
            hasChannel: false,
            async invoke(command) {
                commands.push(command);
                if (command === 'agent_grants') {
                    return [
                        {
                            grant_id: 'gr_9a3c5e7b1d2f4a60',
                            path: '/tmp/studio-project',
                        },
                    ];
                }
                throw new Error(`unexpected Tauri command ${command}`);
            },
            channel(): TauriChannelHandle {
                throw new Error('the offline shim has no Channel');
            },
        };
        const transport = new TauriHostTransport({ bridge });
        const grants = await transport.hostGrants();
        expect(commands).toEqual(['agent_grants']);
        expect(grants).toEqual([
            {
                grant_id: 'gr_9a3c5e7b1d2f4a60',
                path: '/tmp/studio-project',
            },
        ]);

        const broken = new TauriHostTransport({
            bridge: {
                hasChannel: false,
                async invoke() {
                    return [{ grant_id: 'gr_short', path: 7 }];
                },
                channel(): TauriChannelHandle {
                    throw new Error('no channel');
                },
            },
        });
        await expect(broken.hostGrants()).rejects.toMatchObject({
            code: 'invalid_event',
        });
    });

    test('tauri_transport_exposes_validated_grants_through_host_client', async () => {
        const bridge: TauriHostBridge = {
            hasChannel: false,
            async invoke(command) {
                if (command !== 'agent_grants') {
                    throw new Error(`unexpected Tauri command ${command}`);
                }
                return [
                    {
                        grant_id: 'gr_9a3c5e7b1d2f4a60',
                        path: '/tmp/studio-project',
                    },
                    {
                        grant_id: 'gr_0b1d2f4a6c8e9a30',
                        path: '/tmp/other-project',
                    },
                ];
            },
            channel(): TauriChannelHandle {
                throw new Error('no channel');
            },
        };
        const transport = new TauriHostTransport({ bridge });
        const expected = [
            { grantId: 'gr_9a3c5e7b1d2f4a60', path: '/tmp/studio-project' },
            { grantId: 'gr_0b1d2f4a6c8e9a30', path: '/tmp/other-project' },
        ];
        await expect(transport.grants()).resolves.toEqual(expected);
        const client = new HostClient({ transport });
        await expect(client.grants()).resolves.toEqual(expected);
        await expect(
            new HostClient({ transport: new StaticHostTransport() }).grants(),
        ).resolves.toEqual([]);

        const duplicate = {
            kind: 'bridge',
            capabilities: async () => ({
                kind: 'bridge',
                hostAvailable: true,
                streaming: 'none',
                ...ALL_CAPS,
            }),
            invoke: async () => null,
            subscribe: () => ({ close() {}, cursor: () => null }),
            approve: async (input: {
                approvalId: string;
                decision: 'approved' | 'denied';
            }) => ({ approvalId: input.approvalId, decision: input.decision }),
            grants: async () => [
                { grantId: 'gr_9a3c5e7b1d2f4a60', path: '/tmp/a' },
                { grantId: 'gr_9a3c5e7b1d2f4a60', path: '/tmp/b' },
            ],
        } as unknown as HostTransport;
        await expect(
            new HostClient({ transport: duplicate }).grants(),
        ).rejects.toMatchObject({ code: 'invalid_event' });

        const malformed = {
            ...duplicate,
            grants: async () => [{ grantId: 'gr_short', path: '/tmp/a' }],
        } as unknown as HostTransport;
        await expect(
            new HostClient({ transport: malformed }).grants(),
        ).rejects.toMatchObject({ code: 'invalid_event' });
    });

    test('tauri_transport_resolves_approval_with_exact_input', async () => {
        const payloads: Array<Record<string, unknown>> = [];
        const bridge: TauriHostBridge = {
            hasChannel: false,
            async invoke(command, payload) {
                if (command !== 'agent_host_resolve_approval') {
                    throw new Error(`unexpected Tauri command ${command}`);
                }
                payloads.push(payload ?? {});
                const input = (
                    payload as {
                        input: {
                            approval_id: string;
                            decision: 'approved' | 'denied';
                        };
                    }
                ).input;
                return {
                    approval_id: input.approval_id,
                    decision: input.decision,
                };
            },
            channel(): TauriChannelHandle {
                throw new Error('no channel');
            },
        };
        const transport = new TauriHostTransport({ bridge });
        await expect(
            transport.approve({
                approvalId: 'apr_5c2e8a1f9b7d3e40',
                decision: 'approved',
            }),
        ).resolves.toEqual({
            approvalId: 'apr_5c2e8a1f9b7d3e40',
            decision: 'approved',
        });
        expect(payloads).toEqual([
            {
                input: {
                    approval_id: 'apr_5c2e8a1f9b7d3e40',
                    decision: 'approved',
                },
            },
        ]);

        const mismatched = new TauriHostTransport({
            bridge: {
                hasChannel: false,
                async invoke() {
                    return {
                        approval_id: 'apr_0000000000000000',
                        decision: 'approved',
                    };
                },
                channel(): TauriChannelHandle {
                    throw new Error('no channel');
                },
            },
        });
        await expect(
            mismatched.approve({
                approvalId: 'apr_5c2e8a1f9b7d3e40',
                decision: 'approved',
            }),
        ).rejects.toMatchObject({ code: 'approval_forbidden' });
    });
});

describe('HostClient Task 18 loopback and hostile transport', () => {
    test('loopback_capabilities_do_not_advertise_page_approvals', async () => {
        const transport = new LoopbackHostTransport({
            token: 'launch-token',
            baseUrl: 'http://127.0.0.1:4173',
            fetch: (async () => new Response(null, { status: 204 })) as typeof fetch,
            capabilities: { ...ALL_CAPS, approvals: true },
        });
        const capabilities = await transport.capabilities();
        expect(capabilities.streaming).toBe('ndjson');
        expect(capabilities.tree).toBe(true);
        expect(capabilities.approvals).toBe(false);
        await expect(
            transport.approve({
                approvalId: 'apr_5c2e8a1f9b7d3e40',
                decision: 'approved',
            }),
        ).rejects.toMatchObject({ code: 'host_unavailable' });
    });

    test('slow_host_call_is_not_failed_by_the_default_pending_timeout', async () => {
        const scheduler = new ManualScheduler();
        const transport = new ManualStreamTransport();
        const client = new HostClient({ transport, timers: scheduler });
        const intent = await validIntent({
            intent_id: 'hti_0123456789abcdef_000000000101',
        });
        const pending = client.execute(intent);
        void pending.catch(() => undefined);
        await until(
            () => transport.subscriptions.length === 1,
            'slow host subscription',
            500,
        );
        await flush();
        expect(scheduler.pending).toBe(0);
        transport.subscriptions[0]!.listener(await validResult(intent));
        await expect(pending).resolves.toMatchObject({
            intent_id: intent.intent_id,
        });
        client.close();
    });

    test('explicit_timeout_settles_once_without_cancelling_host_work', async () => {
        const scheduler = new ManualScheduler();
        const transport = new ManualStreamTransport();
        const client = new HostClient({
            transport,
            timers: scheduler,
            pendingTimeoutMs: 30_000,
        });
        const intent = await validIntent({
            intent_id: 'hti_0123456789abcdef_000000000102',
        });
        const pending = client.execute(intent, { timeoutMs: 10 });
        let settlements = 0;
        void pending.then(
            () => {
                settlements += 1;
            },
            () => {
                settlements += 1;
            },
        );
        await until(
            () => transport.subscriptions.length === 1,
            'explicit timeout subscription',
            500,
        );
        await until(() => scheduler.pending >= 1, 'explicit timer', 500);
        expect(scheduler.runNext()).toBe(true);
        await expect(pending).rejects.toMatchObject({ code: 'transport_closed' });
        transport.subscriptions[0]!.listener(
            await validResult(intent, { output: { late: true } }),
        );
        await flush();
        expect(settlements).toBe(1);
        client.close();
    });

    test('duplicate_stream_result_settles_execute_once', async () => {
        const scheduler = new ManualScheduler();
        const transport = new ManualStreamTransport();
        const client = new HostClient({ transport, timers: scheduler });
        const intent = await validIntent({
            intent_id: 'hti_0123456789abcdef_000000000103',
        });
        const pending = client.execute(intent);
        let settlements = 0;
        void pending.then(
            () => {
                settlements += 1;
            },
            () => {
                settlements += 1;
            },
        );
        await until(
            () => transport.subscriptions.length === 1,
            'duplicate subscription',
            500,
        );
        const result = await validResult(intent);
        transport.subscriptions[0]!.listener(result);
        transport.subscriptions[0]!.listener(result);
        await expect(pending).resolves.toMatchObject({
            intent_id: intent.intent_id,
        });
        await flush();
        expect(settlements).toBe(1);
        client.close();
    });

    test('loopback_result_preserves_artifact_identity_and_range_metadata', async () => {
        const intent = await validIntent({
            intent_id: 'hti_data_000000000104',
            tool: 'artifact.read',
            request_scope: 'data',
            task_id: 'tsk_0000000000000000',
            run_id: 'run_0000000000000000',
            generation: 1,
            arguments: { artifact_id: `art_${'a'.repeat(64)}`, start: 2, end: 5 },
        });
        const output = {
            artifact_id: `art_${'a'.repeat(64)}`,
            path: 'data.bin',
            media_type: 'application/octet-stream',
            size_bytes: 3,
            artifact_size_bytes: 10,
            digest: `sha256:${'b'.repeat(64)}`,
            whole_digest: `sha256:${'c'.repeat(64)}`,
            bytes_base64: 'MjM0',
            range: { start: 2, end: 5 },
        };
        const result = await validResult(intent, {
            output,
            artifact: {
                artifact_id: `art_${'a'.repeat(64)}`,
                digest: `sha256:${'c'.repeat(64)}`,
                media_type: 'application/octet-stream',
                size_bytes: 10,
            },
        });
        const fetchImpl = (async (
            input: RequestInfo | URL,
            init: RequestInit = {},
        ): Promise<Response> => {
            if (String(input).includes('/v1/studio/host/intents')) {
                return new Response(JSON.stringify(result), {
                    status: 200,
                    headers: { 'content-type': 'application/json' },
                });
            }
            return new Response(
                new ReadableStream<Uint8Array>({
                    start(controller) {
                        controller.close();
                    },
                }),
                {
                    status: 200,
                    headers: { 'content-type': 'application/x-ndjson' },
                },
            );
        }) as typeof fetch;
        const client = new HostClient({
            transport: new LoopbackHostTransport({
                token: 'launch-token',
                baseUrl: 'http://127.0.0.1:4173',
                fetch: fetchImpl,
            }),
            timers: new ManualScheduler(),
        });
        const settled = await client.execute(intent);
        expect(settled.output).toEqual(output);
        expect(settled.artifact).toEqual(result.artifact);
        client.close();
    });

    test('hostile_invoke_getter_error_rejects_without_pending_timeout', async () => {
        const scheduler = new ManualScheduler();
        const transport: HostTransport = {
            kind: 'bridge',
            supportsStream: true,
            async capabilities() {
                return {
                    kind: 'bridge',
                    hostAvailable: true,
                    streaming: 'channel',
                    ...ALL_FALSE,
                };
            },
            invoke() {
                const hostile = new Proxy(
                    {},
                    {
                        get(_target, key) {
                            if (key === 'schema') {
                                throw new Error('hostile response getter');
                            }
                            return undefined;
                        },
                    },
                );
                return hostile as unknown as Promise<{ accepted: true }>;
            },
            subscribe: () => ({
                close: () => undefined,
                cursor: () => null,
            }),
            async approve(input) {
                return {
                    approvalId: input.approvalId,
                    decision: input.decision,
                };
            },
        };
        const client = new HostClient({ transport, timers: scheduler });
        const pending = client.execute(await validIntent());
        void pending.catch(() => undefined);
        const outcome = await Promise.race([
            pending.then(
                () => 'resolved',
                (error: unknown) =>
                    error instanceof AgentClientError ? error.code : 'unknown',
            ),
            new Promise<string>((resolve) =>
                globalThis.setTimeout(() => resolve('pending'), 1_000),
            ),
        ]);
        expect(outcome).toMatch(/^(internal|invalid_event)$/);
        expect(scheduler.pending).toBe(0);
        client.close();
    });

    test('hostile_supports_stream_getter_error_settles_without_timeout', async () => {
        const scheduler = new ManualScheduler();
        let reads = 0;
        const listeners: Array<(result: HostToolResult) => void> = [];
        const transport: HostTransport = {
            kind: 'bridge',
            get supportsStream(): boolean {
                reads += 1;
                if (reads > 1) {
                    throw new Error('hostile supportsStream getter');
                }
                return true;
            },
            async capabilities() {
                return {
                    kind: 'bridge',
                    hostAvailable: true,
                    streaming: 'channel',
                    ...ALL_FALSE,
                };
            },
            async invoke() {
                return { accepted: true };
            },
            subscribe: (listener) => {
                listeners.push(listener);
                return {
                    close: () => undefined,
                    cursor: () => null,
                };
            },
            async approve(input) {
                return {
                    approvalId: input.approvalId,
                    decision: input.decision,
                };
            },
        };
        const client = new HostClient({ transport, timers: scheduler });
        const intent = await validIntent();
        const pending = client.execute(intent);
        void pending.catch(() => undefined);
        await until(() => listeners.length === 1, 'hostile subscription', 2_000);
        await until(
            () => scheduler.pending === 0,
            'accepted hostile transport',
            2_000,
        );
        expect(reads).toBe(1);
        listeners[0]!(await validResult(intent));
        await expect(pending).resolves.toMatchObject({
            intent_id: intent.intent_id,
        });
        client.close();
    });

    test('loopback_clean_stream_end_fails_pending_and_resubscribes', async () => {
        const scheduler = new ManualScheduler();
        let gets = 0;
        const intent = await validIntent({
            intent_id: 'hti_0123456789abcdef_000000000106',
        });
        const secondIntent = await validIntent({
            intent_id: 'hti_0123456789abcdef_000000000107',
        });
        const fetchImpl = (async (
            input: RequestInfo | URL,
        ): Promise<Response> => {
            if (String(input).includes('/v1/studio/host/intents')) {
                return new Response(JSON.stringify({ accepted: true }), {
                    status: 202,
                    headers: { 'content-type': 'application/json' },
                });
            }
            gets += 1;
            return new Response(
                new ReadableStream<Uint8Array>({
                    start(controller) {
                        // The host closed the subscription (for example, an
                        // evicted subscriber) after acceptance.
                        controller.close();
                    },
                }),
                {
                    status: 200,
                    headers: { 'content-type': 'application/x-ndjson' },
                },
            );
        }) as typeof fetch;
        const client = new HostClient({
            transport: new LoopbackHostTransport({
                token: 'launch-token',
                baseUrl: 'http://127.0.0.1:4173',
                fetch: fetchImpl,
            }),
            timers: scheduler,
            pendingTimeoutMs: 30_000,
        });
        const pending = client.execute(intent);
        void pending.catch(() => undefined);
        const outcome = await Promise.race([
            pending.then(
                () => 'resolved',
                (error: unknown) =>
                    error instanceof AgentClientError ? error.code : 'unknown',
            ),
            new Promise<string>((resolve) =>
                globalThis.setTimeout(() => resolve('pending'), 1_000),
            ),
        ]);
        expect(outcome).toBe('transport_closed');
        expect(scheduler.pending).toBe(0);
        const second = client.execute(secondIntent);
        void second.catch(() => undefined);
        await until(() => gets === 2, 'clean-end resubscribe', 2_000);
        client.close();
    });

    test('default_timeout_still_rejects_before_acceptance', async () => {
        const scheduler = new ManualScheduler();
        const transport: HostTransport = {
            kind: 'bridge',
            supportsStream: true,
            async capabilities() {
                return {
                    kind: 'bridge',
                    hostAvailable: true,
                    streaming: 'channel',
                    ...ALL_FALSE,
                };
            },
            invoke() {
                return new Promise<never>(() => undefined);
            },
            subscribe: () => ({
                close: () => undefined,
                cursor: () => null,
            }),
            async approve(input) {
                return {
                    approvalId: input.approvalId,
                    decision: input.decision,
                };
            },
        };
        const client = new HostClient({
            transport,
            timers: scheduler,
            pendingTimeoutMs: 10,
        });
        const pending = client.execute(await validIntent());
        void pending.catch(() => undefined);
        await until(() => scheduler.pending === 1, 'default timer', 2_000);
        expect(scheduler.runNext()).toBe(true);
        await expect(pending).rejects.toMatchObject({ code: 'transport_closed' });
        client.close();
    });

    test('loopback_read_disposal_cancels_the_stream_and_leaves_no_pending', async () => {
        const scheduler = new ManualScheduler();
        let readerStarted = false;
        let cancelled = false;
        const intent = await validIntent({
            intent_id: 'hti_0123456789abcdef_000000000105',
        });
        const fetchImpl = (async (
            input: RequestInfo | URL,
        ): Promise<Response> => {
            if (String(input).includes('/v1/studio/host/intents')) {
                return new Response(JSON.stringify({ accepted: true }), {
                    status: 202,
                    headers: { 'content-type': 'application/json' },
                });
            }
            const stream = new ReadableStream<Uint8Array>({
                start() {
                    readerStarted = true;
                    // A hostile server ignores the abort signal and holds the
                    // read forever; disposal must still release the reader.
                },
                cancel() {
                    cancelled = true;
                },
            });
            return new Response(stream, {
                status: 200,
                headers: { 'content-type': 'application/x-ndjson' },
            });
        }) as typeof fetch;
        const client = new HostClient({
            transport: new LoopbackHostTransport({
                token: 'launch-token',
                baseUrl: 'http://127.0.0.1:4173',
                fetch: fetchImpl,
            }),
            timers: scheduler,
            pendingTimeoutMs: 30_000,
        });
        const pending = client.execute(intent);
        void pending.catch(() => undefined);
        await until(() => readerStarted, 'loopback reader', 500);
        client.close();
        await expect(pending).rejects.toMatchObject({ code: 'transport_closed' });
        await until(() => cancelled, 'loopback read cancellation', 500);
        expect(scheduler.pending).toBe(0);
    });
});
