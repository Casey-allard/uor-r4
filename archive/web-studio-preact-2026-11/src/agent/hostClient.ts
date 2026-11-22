import { canonicalJson } from './generated/agentCanonical.js';
import type { ApprovalReceipt, HostCapabilities } from './client.js';
import {
    AgentClientError,
    isPlainObject,
    toClientError,
} from './storeTypes.js';

export const HOST_INTENT_SCHEMA = 'uor-r4.studio.host-intent/1' as const;
export const HOST_RESULT_SCHEMA = 'uor-r4.studio.host-result/1' as const;

const DIGEST_PATTERN = /^sha256:[0-9a-f]{64}$/;
const RECEIPT_PATTERN = /^hmac-sha256:[0-9a-f]{64}$/;
const TOOL_PATTERN = /^[a-z][a-z0-9_.-]*$/;
const TASK_ID_PATTERN = /^tsk_[0-9a-f]{16}$/;
const RUN_ID_PATTERN = /^run_[0-9a-f]{16}$/;
const APPROVAL_ID_PATTERN = /^[A-Za-z0-9_-]+$/;
const GRANT_ID_PATTERN = /^gr_[0-9a-f]{16}$/;
const MAX_GRANT_PATH_CHARS = 4096;
const MAX_PREVIEW_LENGTH = 512;
const DATA_TASK_ID = 'tsk_0000000000000000';
const DATA_RUN_ID = 'run_0000000000000000';
const DATA_GENERATION = 1;
const AUTHORITY_KEYS = [
    'granted',
    'scope',
    'command',
    'key',
    'receipt',
    'path',
] as const;

export type HostToolStatus = 'ok' | 'error' | 'cancelled';
export type HostRequestScope = 'task' | 'data';
export type HostTransportKind =
    | 'null'
    | 'static'
    | 'tauri'
    | 'loopback'
    | 'bridge';
export type HostStreamingKind = 'none' | 'channel' | 'poll' | 'ndjson';
export type HostDataTool = 'file.tree' | 'git.changes' | 'artifact.read';

export interface HostArtifactRef {
    artifact_id: string;
    digest: string;
    media_type: string;
    size_bytes: number;
}

/** A host-approved folder grant. The page never enumerates paths itself. */
export interface HostGrant {
    grantId: string;
    path: string;
}

export const MAX_HOST_GRANTS = 64;

export interface HostToolIntent {
    schema: typeof HOST_INTENT_SCHEMA;
    intent_id: string;
    task_id: string;
    run_id: string;
    generation: number;
    tool_call_id: string;
    tool: string;
    arguments: Record<string, unknown>;
    arguments_digest: string;
    request_scope?: HostRequestScope;
    approval_id?: string;
}

export interface HostToolResult {
    schema: typeof HOST_RESULT_SCHEMA;
    intent_id: string;
    task_id: string;
    run_id: string;
    generation: number;
    tool_call_id: string;
    request_scope?: HostRequestScope;
    status: HostToolStatus;
    output?: unknown;
    output_digest: string;
    output_preview: string;
    artifact?: HostArtifactRef;
    approval_receipt?: string;
    occurred_at_ms: number;
    code?: string;
    message?: string;
    truncated?: boolean;
    cursor?: string;
}

export interface HostInvokeAccepted {
    accepted: true;
    intent_id?: string;
}

export type HostInvokeResponse =
    | HostToolResult
    | HostInvokeAccepted
    | null;

export interface HostToolRequest {
    taskId: string;
    runId: string;
    generation: number;
    toolCallId: string;
    tool: string;
    arguments: Record<string, unknown>;
    approvalId?: string;
}

export interface HostDataRequest {
    tool: HostDataTool;
    arguments: Record<string, unknown>;
    taskId?: string;
    runId?: string;
    generation?: number;
}

export interface HostApprovalInput {
    approvalId: string;
    decision: 'approved' | 'denied';
}

export interface HostSubscription {
    close(): void;
    cursor(): string | null;
}

export interface HostSubscribeOptions {
    cursor?: string | null;
    onResync?: (reason: 'cursor_expired') => void;
    onError?: (error: AgentClientError) => void;
    onEnd?: () => void;
}

export interface HostTransportCapabilities extends HostCapabilities {
    kind: HostTransportKind;
    hostAvailable: boolean;
    streaming: HostStreamingKind;
}

export interface HostTransport {
    readonly kind: HostTransportKind;
    readonly supportsStream?: boolean;
    capabilities(): Promise<HostTransportCapabilities>;
    invoke(
        intent: HostToolIntent,
        context?: HostDataRequest,
    ): Promise<HostInvokeResponse>;
    subscribe(
        listener: (result: HostToolResult) => void,
        options?: HostSubscribeOptions,
    ): HostSubscription;
    approve(input: HostApprovalInput): Promise<ApprovalReceipt>;
    /** Host-approved folders. A transport without this capability returns none. */
    grants?(): Promise<HostGrant[]>;
}

export interface HostClientTimers {
    setTimeout(handler: () => void, timeoutMs: number): unknown;
    clearTimeout(handle: unknown): void;
}

export interface HostClientOptions {
    transport: HostTransport;
    pendingTimeoutMs?: number;
    timers?: HostClientTimers;
}

export interface HostExecuteOptions {
    signal?: AbortSignal;
    timeoutMs?: number;
}

function errorMessage(error: unknown): string {
    return error instanceof Error ? error.message : String(error);
}

function invalidArguments(message: string): never {
    throw new AgentClientError('invalid_arguments', message);
}

function invalidEvent(message: string): never {
    throw new AgentClientError('invalid_event', message);
}

export function canonicalToolArguments(
    value: unknown,
    label = 'tool arguments',
): Record<string, unknown> {
    if (!isPlainObject(value)) {
        return invalidArguments(`${label} must be a canonical JSON object`);
    }
    let canonical: string;
    try {
        canonical = canonicalJson(value);
    } catch (error) {
        return invalidArguments(`${label} are not canonical: ${errorMessage(error)}`);
    }
    return JSON.parse(canonical) as Record<string, unknown>;
}

export function boundedPreview(
    text: string,
    maxLength = MAX_PREVIEW_LENGTH,
): string {
    if (text.length <= maxLength) return text;
    let end = Math.max(0, maxLength);
    const code = text.charCodeAt(end - 1);
    if (code >= 0xd800 && code <= 0xdbff) end -= 1;
    return text.slice(0, end);
}

export async function sha256Bytes(bytes: Uint8Array): Promise<string> {
    const subtle = globalThis.crypto?.subtle;
    if (subtle === undefined) {
        throw new AgentClientError(
            'runtime_unavailable',
            'Web Crypto SHA-256 is not available in this environment',
        );
    }
    const digest = new Uint8Array(
        await subtle.digest('SHA-256', bytes as unknown as BufferSource),
    );
    let hex = '';
    for (const byte of digest) hex += byte.toString(16).padStart(2, '0');
    return `sha256:${hex}`;
}

export async function sha256Canonical(value: unknown): Promise<string> {
    return sha256Bytes(new TextEncoder().encode(canonicalJson(value)));
}

async function argumentsDigest(value: Record<string, unknown>): Promise<string> {
    try {
        return await sha256Canonical(value);
    } catch (error) {
        if (error instanceof AgentClientError) throw error;
        return invalidArguments(
            `tool arguments are not canonical: ${errorMessage(error)}`,
        );
    }
}

function assertNoAuthorityFields(
    value: Record<string, unknown>,
    label: string,
): void {
    for (const key of AUTHORITY_KEYS) {
        if (key in value) {
            invalidArguments(
                `${label} may not carry host authority field ${key}`,
            );
        }
    }
}

export async function validateHostToolIntent(
    input: HostToolIntent,
): Promise<HostToolIntent> {
    if (!isPlainObject(input)) {
        return invalidArguments('host tool intent must be an object');
    }
    assertNoAuthorityFields(input, 'host tool intent');
    if (input.schema !== HOST_INTENT_SCHEMA) {
        return invalidArguments('host tool intent schema is not host-intent/1');
    }
    if (typeof input.intent_id !== 'string' || input.intent_id.length === 0) {
        return invalidArguments('host tool intent requires an intent_id');
    }
    const scope = input.request_scope ?? 'task';
    if (scope !== 'task' && scope !== 'data') {
        return invalidArguments(
            'host tool intent request_scope must be task | data',
        );
    }
    if (
        typeof input.task_id !== 'string' ||
        !TASK_ID_PATTERN.test(input.task_id)
    ) {
        return invalidArguments('host tool intent requires a canonical task_id');
    }
    if (
        typeof input.run_id !== 'string' ||
        !RUN_ID_PATTERN.test(input.run_id)
    ) {
        return invalidArguments('host tool intent requires a canonical run_id');
    }
    if (scope === 'data') {
        if (
            input.task_id !== DATA_TASK_ID ||
            input.run_id !== DATA_RUN_ID ||
            input.generation !== DATA_GENERATION
        ) {
            return invalidArguments(
                'host data intent must not claim a real task, run or generation',
            );
        }
    } else if (
        input.task_id === DATA_TASK_ID ||
        input.run_id === DATA_RUN_ID
    ) {
        return invalidArguments(
            'zero task and run placeholders are data-only',
        );
    } else if (
        !Number.isSafeInteger(input.generation) ||
        input.generation < 1
    ) {
        return invalidArguments(
            'host tool intent generation must be a positive safe integer',
        );
    }
    if (
        typeof input.tool_call_id !== 'string' ||
        input.tool_call_id.length === 0
    ) {
        return invalidArguments('host tool intent requires a tool_call_id');
    }
    if (typeof input.tool !== 'string' || !TOOL_PATTERN.test(input.tool)) {
        return invalidArguments('host tool intent requires a canonical tool name');
    }
    const argumentsValue = canonicalToolArguments(input.arguments);
    if (
        typeof input.arguments_digest !== 'string' ||
        !DIGEST_PATTERN.test(input.arguments_digest)
    ) {
        return invalidArguments(
            'host tool intent arguments_digest must be sha256:<64 lowercase hex>',
        );
    }
    if ((await argumentsDigest(argumentsValue)) !== input.arguments_digest) {
        return invalidArguments(
            'host tool intent arguments_digest does not match the arguments',
        );
    }
    if (
        input.approval_id !== undefined &&
        (typeof input.approval_id !== 'string' ||
            !APPROVAL_ID_PATTERN.test(input.approval_id))
    ) {
        return invalidArguments(
            'host tool intent approval_id must be a non-empty identity',
        );
    }
    try {
        canonicalJson({ ...input, arguments: argumentsValue });
    } catch (error) {
        return invalidArguments(
            `host tool intent is not canonical: ${errorMessage(error)}`,
        );
    }
    return { ...input, request_scope: scope, arguments: argumentsValue };
}

export async function validateHostToolResult(
    intent: HostToolIntent,
    value: unknown,
): Promise<HostToolResult> {
    if (!isPlainObject(value)) {
        return invalidEvent('host tool result must be an object');
    }
    if (
        'type' in value ||
        'event_id' in value ||
        'sequence' in value ||
        'recorded_at_ms' in value
    ) {
        return invalidEvent(
            'host tool result is not an AgentEvent and may not carry event fields',
        );
    }
    if (value.schema !== HOST_RESULT_SCHEMA) {
        return invalidEvent('host tool result schema is not host-result/1');
    }
    if (value.intent_id !== intent.intent_id) {
        throw new AgentClientError(
            'out_of_order',
            'host tool result does not match the intent_id',
        );
    }
    const intentScope = intent.request_scope ?? 'task';
    const resultScope = value.request_scope ?? 'task';
    if (resultScope !== intentScope) {
        throw new AgentClientError(
            'out_of_order',
            'host tool result request_scope does not match the intent',
        );
    }
    if (intentScope === 'task') {
        if (value.task_id !== intent.task_id) {
            throw new AgentClientError(
                'cross_task_event',
                'host tool result belongs to a different task',
            );
        }
        if (value.run_id !== intent.run_id) {
            throw new AgentClientError(
                'out_of_order',
                'host tool result belongs to a different run',
            );
        }
        if (value.generation !== intent.generation) {
            throw new AgentClientError(
                'stale_generation',
                'host tool result belongs to a stale generation',
            );
        }
    } else {
        if (value.task_id !== intent.task_id) {
            throw new AgentClientError(
                'cross_task_event',
                'host data result requires the explicit zero task identity',
            );
        }
        if (value.run_id !== intent.run_id) {
            throw new AgentClientError(
                'out_of_order',
                'host data result requires the explicit zero run identity',
            );
        }
        if (value.generation !== intent.generation) {
            throw new AgentClientError(
                'stale_generation',
                'host data result requires generation 1',
            );
        }
    }
    if (value.tool_call_id !== intent.tool_call_id) {
        throw new AgentClientError(
            'out_of_order',
            'host tool result does not match the tool_call_id',
        );
    }
    if (
        value.status !== 'ok' &&
        value.status !== 'error' &&
        value.status !== 'cancelled'
    ) {
        return invalidEvent('host tool result status is invalid');
    }
    if (
        typeof value.occurred_at_ms !== 'number' ||
        !Number.isSafeInteger(value.occurred_at_ms) ||
        value.occurred_at_ms < 0
    ) {
        return invalidEvent(
            'host tool result occurred_at_ms must be a non-negative safe integer',
        );
    }
    try {
        canonicalJson(value);
    } catch (error) {
        return invalidArguments(
            `host tool result is not canonical: ${errorMessage(error)}`,
        );
    }
    const output = value.output === undefined ? {} : value.output;
    let digest: string;
    try {
        digest = await sha256Canonical(output);
    } catch (error) {
        return invalidArguments(
            `host tool result output is not canonical: ${errorMessage(error)}`,
        );
    }
    if (value.output_digest !== digest) {
        return invalidArguments(
            'host tool result output_digest does not match the output',
        );
    }
    if (
        typeof value.output_preview !== 'string' ||
        value.output_preview.length > MAX_PREVIEW_LENGTH
    ) {
        return invalidEvent(
            'host tool result output_preview must be a bounded string',
        );
    }
    if (value.code !== undefined && typeof value.code !== 'string') {
        return invalidEvent('host tool result code must be a string');
    }
    if (value.code === 'quota_exceeded') {
        throw new AgentClientError(
            'host_unavailable',
            'quota_exceeded is a future service code and is not a local host result',
        );
    }
    if (value.message !== undefined && typeof value.message !== 'string') {
        return invalidEvent('host tool result message must be a string');
    }
    if (value.truncated !== undefined && typeof value.truncated !== 'boolean') {
        return invalidEvent('host tool result truncated must be a boolean');
    }
    if (value.cursor !== undefined && typeof value.cursor !== 'string') {
        return invalidEvent('host tool result cursor must be a string');
    }
    if (value.artifact !== undefined) {
        if (!isPlainObject(value.artifact)) {
            return invalidEvent('host tool result artifact must be an object');
        }
        const artifact = value.artifact;
        if (
            typeof artifact.artifact_id !== 'string' ||
            artifact.artifact_id.length === 0
        ) {
            return invalidEvent(
                'host tool result artifact requires an artifact_id',
            );
        }
        if (
            typeof artifact.digest !== 'string' ||
            !DIGEST_PATTERN.test(artifact.digest)
        ) {
            return invalidEvent(
                'host tool result artifact digest must be sha256:<64 lowercase hex>',
            );
        }
        if (
            typeof artifact.media_type !== 'string' ||
            artifact.media_type.length === 0
        ) {
            return invalidEvent(
                'host tool result artifact requires a media_type',
            );
        }
        if (
            typeof artifact.size_bytes !== 'number' ||
            !Number.isSafeInteger(artifact.size_bytes) ||
            artifact.size_bytes < 0
        ) {
            return invalidEvent(
                'host tool result artifact size_bytes must be a non-negative safe integer',
            );
        }
    }
    if (intent.approval_id !== undefined) {
        if (
            typeof value.approval_receipt !== 'string' ||
            !RECEIPT_PATTERN.test(value.approval_receipt)
        ) {
            throw new AgentClientError(
                'approval_forbidden',
                'host tool result lacks a host approval receipt',
            );
        }
    } else if (value.approval_receipt !== undefined) {
        throw new AgentClientError(
            'approval_forbidden',
            'host tool result carries an approval receipt that was not requested',
        );
    }
    return value as unknown as HostToolResult;
}

function isHostToolIntent(value: unknown): value is HostToolIntent {
    return isPlainObject(value) && value.schema === HOST_INTENT_SCHEMA;
}

function isHostToolRequest(value: unknown): value is HostToolRequest {
    return isPlainObject(value) && 'toolCallId' in value;
}

function isHostToolResultLike(value: unknown): value is HostToolResult {
    return isPlainObject(value) && value.schema === HOST_RESULT_SCHEMA;
}

function asHostToolResult(value: unknown): HostToolResult | null {
    if (isHostToolResultLike(value)) return value;
    if (isPlainObject(value) && isHostToolResultLike(value.result)) {
        return value.result;
    }
    return null;
}

function isInvokeAccepted(value: unknown): boolean {
    return (
        value === null ||
        (isPlainObject(value) &&
            (value.accepted === true || value.queued === true))
    );
}

interface PendingEntry {
    intent: HostToolIntent;
    promise: Promise<HostToolResult>;
    resolve: (result: HostToolResult) => void;
    reject: (error: AgentClientError) => void;
    settled: boolean;
    accepted: boolean;
    timer: unknown;
    explicitTimeout: boolean;
    removeAbort: (() => void) | null;
}

interface StreamListenerEntry {
    listener: (result: HostToolResult) => void;
    onError?: (error: AgentClientError) => void;
    onResync?: (reason: 'cursor_expired') => void;
    onEnd?: () => void;
}

const DEFAULT_PENDING_TIMEOUT_MS = 30_000;

const defaultHostTimers: HostClientTimers = {
    setTimeout: (handler, timeoutMs) =>
        globalThis.setTimeout(handler, timeoutMs),
    clearTimeout: (handle) =>
        globalThis.clearTimeout(handle as ReturnType<typeof setTimeout>),
};

export class HostClient {
    private readonly transport: HostTransport;
    private readonly pending = new Map<string, PendingEntry>();
    private readonly streamListeners = new Set<StreamListenerEntry>();
    private readonly timers: HostClientTimers;
    private readonly pendingTimeoutMs: number;
    private subscription: HostSubscription | null = null;
    private subscriptionGeneration = 0;
    private streamDead = false;
    private closed = false;
    private intentCounter = 0;

    constructor(options: HostClientOptions) {
        this.transport = options.transport;
        this.timers = options.timers ?? defaultHostTimers;
        this.pendingTimeoutMs =
            options.pendingTimeoutMs ?? DEFAULT_PENDING_TIMEOUT_MS;
    }

    async capabilities(): Promise<HostCapabilities> {
        try {
            const capabilities = await this.transport.capabilities();
            return {
                tree: capabilities.tree === true,
                changes: capabilities.changes === true,
                readArtifact: capabilities.readArtifact === true,
                terminal: capabilities.terminal === true,
                git: capabilities.git === true,
                mcp: capabilities.mcp === true,
                approvals: capabilities.approvals === true,
            };
        } catch (error) {
            if (
                error instanceof AgentClientError &&
                error.code === 'host_unavailable'
            ) {
                return {
                    tree: false,
                    changes: false,
                    readArtifact: false,
                    terminal: false,
                    git: false,
                    mcp: false,
                    approvals: false,
                };
            }
            throw error;
        }
    }

    /** True only when the transport can supply host-approved folder grants. */
    supportsGrants(): boolean {
        return typeof this.transport.grants === 'function';
    }

    async grants(): Promise<HostGrant[]> {
        if (this.closed) {
            throw new AgentClientError(
                'transport_closed',
                'host client is closed',
            );
        }
        const readGrants = this.transport.grants;
        if (readGrants === undefined) return [];        let value: unknown;
        try {
            value = await readGrants.call(this.transport);
        } catch (error) {
            throw toClientError(error);
        }
        if (!Array.isArray(value)) {
            return invalidEvent('host grants must be an array');
        }
        if (value.length > MAX_HOST_GRANTS) {
            return invalidEvent('host grants exceed the client cap');
        }
        const seen = new Set<string>();
        const grants: HostGrant[] = [];
        for (const item of value) {
            if (
                !isPlainObject(item) ||
                typeof item.grantId !== 'string' ||
                !GRANT_ID_PATTERN.test(item.grantId) ||
                typeof item.path !== 'string' ||
                item.path.length === 0 ||
                item.path.length > MAX_GRANT_PATH_CHARS ||
                /[\u0000-\u001f\u007f-\u009f]/.test(item.path)
            ) {
                return invalidEvent('host grant record is malformed');
            }
            if (seen.has(item.grantId)) {
                return invalidEvent('host grants must not repeat a grant id');
            }
            seen.add(item.grantId);
            grants.push({ grantId: item.grantId, path: item.path });
        }
        return grants;
    }

    async createIntent(input: HostToolRequest): Promise<HostToolIntent> {
        if (!isPlainObject(input)) {
            return invalidArguments('host tool request must be an object');
        }
        if (
            typeof input.taskId !== 'string' ||
            typeof input.runId !== 'string' ||
            typeof input.toolCallId !== 'string' ||
            typeof input.tool !== 'string' ||
            !Number.isSafeInteger(input.generation)
        ) {
            return invalidArguments(
                'host tool request requires task, run, generation, tool call and tool',
            );
        }
        const argumentsValue = canonicalToolArguments(input.arguments);
        const digest = await argumentsDigest(argumentsValue);
        const counter = this.intentCounter++;
        const intentId = `hti_${input.runId.replace(/^run_/, '')}_${String(
            counter,
        ).padStart(12, '0')}`;
        return validateHostToolIntent({
            schema: HOST_INTENT_SCHEMA,
            intent_id: intentId,
            task_id: input.taskId,
            run_id: input.runId,
            generation: input.generation,
            tool_call_id: input.toolCallId,
            tool: input.tool,
            arguments: argumentsValue,
            arguments_digest: digest,
            request_scope: 'task',
            ...(input.approvalId === undefined
                ? {}
                : { approval_id: input.approvalId }),
        });
    }

    async execute(
        input: HostToolIntent,
        options?: HostExecuteOptions,
    ): Promise<HostToolResult>;
    async execute(
        input: HostToolRequest,
        options?: HostExecuteOptions,
    ): Promise<HostToolResult>;
    async execute(
        input: HostDataRequest,
        options?: HostExecuteOptions,
    ): Promise<HostToolResult>;
    async execute(
        input: unknown,
        options: HostExecuteOptions = {},
    ): Promise<HostToolResult> {
        if (!isPlainObject(input)) {
            return invalidArguments('host tool request must be an object');
        }
        if (this.closed) {
            throw new AgentClientError(
                'transport_closed',
                'host client is closed',
            );
        }
        const dataContext = isHostToolIntent(input) || isHostToolRequest(input)
            ? undefined
            : (input as unknown as HostDataRequest);
        const intent = isHostToolIntent(input)
            ? await validateHostToolIntent(input)
            : isHostToolRequest(input)
              ? await this.createIntent(input)
              : await this.createDataIntent(input as unknown as HostDataRequest);
        if (this.pending.has(intent.intent_id)) {
            throw new AgentClientError(
                'out_of_order',
                'a host intent with this intent_id is already in flight',
            );
        }
        const entry = this.createPending(intent, options);
        if (entry.settled) return entry.promise;
        let streaming: boolean;
        try {
            streaming = this.transport.supportsStream === true;
            this.ensureSubscription(streaming);
        } catch (error) {
            this.rejectPending(entry, toClientError(error));
            return entry.promise;
        }
        if (entry.settled) return entry.promise;
        let invocation: HostInvokeResponse | Promise<HostInvokeResponse>;
        try {
            invocation = this.transport.invoke(intent, dataContext);
        } catch (error) {
            this.rejectPending(entry, toClientError(error));
            return entry.promise;
        }
        let normalized: Promise<HostInvokeResponse>;
        try {
            normalized = Promise.resolve(invocation);
        } catch (error) {
            this.rejectPending(entry, toClientError(error));
            return entry.promise;
        }
        void normalized.then(
            (response) => {
                try {
                    this.handleInvocation(entry, response, streaming);
                } catch (error) {
                    this.rejectPending(entry, toClientError(error));
                }
            },
            (error: unknown) => {
                this.rejectPending(entry, toClientError(error));
            },
        );
        return entry.promise;
    }

    private handleInvocation(
        entry: PendingEntry,
        response: HostInvokeResponse,
        streaming: boolean,
    ): void {
        if (entry.settled) return;
        const direct = asHostToolResult(response);
        if (direct !== null) {
            void this.settleFromResult(entry, direct);
            return;
        }
        if (!isInvokeAccepted(response)) {
            this.rejectPending(
                entry,
                new AgentClientError(
                    'invalid_event',
                    'host invoke returned an invalid response',
                ),
            );
            return;
        }
        if (!streaming) {
            this.rejectPending(
                entry,
                new AgentClientError(
                    'host_unavailable',
                    'the host transport cannot deliver a streamed result',
                ),
            );
            return;
        }
        this.markHostAccepted(entry);
    }

    private markHostAccepted(entry: PendingEntry): void {
        if (entry.settled) return;
        if (this.streamDead) {
            this.rejectPending(
                entry,
                new AgentClientError(
                    'transport_closed',
                    'host result stream ended before the result arrived',
                ),
            );
            return;
        }
        entry.accepted = true;
        if (entry.explicitTimeout) return;
        if (entry.timer !== null) {
            this.timers.clearTimeout(entry.timer);
            entry.timer = null;
        }
    }

    subscribe(
        listener: (result: HostToolResult) => void,
        options: HostSubscribeOptions = {},
    ): HostSubscription {
        if (this.closed) {
            throw new AgentClientError(
                'transport_closed',
                'host client is closed',
            );
        }
        let streaming: boolean;
        try {
            streaming = this.transport.supportsStream === true;
        } catch (error) {
            throw toClientError(error);
        }
        if (!streaming) {
            throw new AgentClientError(
                'host_unavailable',
                'the host transport has no result stream',
            );
        }
        this.ensureSubscription(streaming);
        const entry: StreamListenerEntry = {
            listener,
            ...(options.onError === undefined
                ? {}
                : { onError: options.onError }),
            ...(options.onResync === undefined
                ? {}
                : { onResync: options.onResync }),
            ...(options.onEnd === undefined ? {} : { onEnd: options.onEnd }),
        };
        this.streamListeners.add(entry);
        let closed = false;
        return {
            close: () => {
                if (closed) return;
                closed = true;
                this.streamListeners.delete(entry);
            },
            cursor: () => this.subscription?.cursor() ?? null,
        };
    }

    close(): void {
        if (this.closed) return;
        this.closed = true;
        this.subscriptionGeneration += 1;
        const subscription = this.subscription;
        this.subscription = null;
        try {
            subscription?.close();
        } catch {
            // A transport close failure must not block pending cleanup.
        } finally {
            this.failAll(
                new AgentClientError(
                    'transport_closed',
                    'host client closed before the result arrived',
                ),
            );
            this.streamListeners.clear();
        }
    }

    async approve(input: HostApprovalInput): Promise<ApprovalReceipt> {
        if (this.closed) {
            throw new AgentClientError(
                'transport_closed',
                'host client is closed',
            );
        }
        if (!isPlainObject(input)) {
            return invalidArguments('page approval input must be an object');
        }
        const keys = Object.keys(input).sort();
        if (
            keys.length !== 2 ||
            keys[0] !== 'approvalId' ||
            keys[1] !== 'decision'
        ) {
            return invalidArguments(
                'page approval accepts only approvalId and decision',
            );
        }
        if (
            typeof input.approvalId !== 'string' ||
            !APPROVAL_ID_PATTERN.test(input.approvalId)
        ) {
            return invalidArguments('approvalId must be a non-empty identity');
        }
        if (input.decision !== 'approved' && input.decision !== 'denied') {
            return invalidArguments(
                'approval decision must be approved | denied',
            );
        }
        return this.transport.approve({
            approvalId: input.approvalId,
            decision: input.decision,
        });
    }

    private createPending(
        intent: HostToolIntent,
        options: HostExecuteOptions,
    ): PendingEntry {
        let resolve!: (result: HostToolResult) => void;
        let reject!: (error: AgentClientError) => void;
        const promise = new Promise<HostToolResult>((innerResolve, innerReject) => {
            resolve = innerResolve;
            reject = innerReject;
        });
        const entry: PendingEntry = {
            intent,
            promise,
            resolve,
            reject,
            settled: false,
            accepted: false,
            timer: null,
            explicitTimeout: options.timeoutMs !== undefined,
            removeAbort: null,
        };
        this.pending.set(intent.intent_id, entry);
        entry.timer = this.timers.setTimeout(() => {
            this.rejectPending(
                entry,
                new AgentClientError(
                    'transport_closed',
                    'host result timed out before it arrived',
                ),
            );
        }, options.timeoutMs ?? this.pendingTimeoutMs);
        if (options.signal !== undefined) {
            const abort = () => {
                this.rejectPending(
                    entry,
                    new AgentClientError(
                        'transport_closed',
                        'host result was aborted',
                    ),
                );
            };
            if (options.signal.aborted) {
                abort();
            } else {
                options.signal.addEventListener('abort', abort, {
                    once: true,
                });
                entry.removeAbort = () =>
                    options.signal?.removeEventListener('abort', abort);
            }
        }
        return entry;
    }

    private ensureSubscription(streaming: boolean): void {
        if (this.subscription !== null || this.closed) return;
        if (!streaming) return;
        const generation = ++this.subscriptionGeneration;
        const subscription = this.transport.subscribe(
            (result) => {
                if (generation !== this.subscriptionGeneration) return;
                void this.onStreamResult(result);
            },
            {
                onResync: () => {
                    if (generation !== this.subscriptionGeneration) return;
                    this.failStream(
                        new AgentClientError(
                            'cursor_expired',
                            'host result stream requires a snapshot resync',
                        ),
                        'resync',
                    );
                },
                onError: (error) => {
                    if (generation !== this.subscriptionGeneration) return;
                    this.failStream(error);
                },
                onEnd: () => {
                    if (generation !== this.subscriptionGeneration) return;
                    this.endStream();
                },
            },
        );
        if (generation !== this.subscriptionGeneration || this.closed) {
            subscription.close();
            return;
        }
        this.subscription = subscription;
        this.streamDead = false;
    }

    private endStream(): void {
        this.subscriptionGeneration += 1;
        const subscription = this.subscription;
        this.subscription = null;
        this.streamDead = true;
        try {
            subscription?.close();
        } catch {
            // A transport close failure must not block pending cleanup.
        }
        for (const entry of [...this.pending.values()]) {
            if (!entry.accepted) continue;
            this.rejectPending(
                entry,
                new AgentClientError(
                    'transport_closed',
                    'host result stream ended before the result arrived',
                ),
            );
        }
        for (const listener of this.streamListeners) {
            try {
                listener.onEnd?.();
            } catch {
                // A consumer cannot break the transport lifecycle.
            }
        }
    }

    private failStream(
        error: AgentClientError,
        reason?: 'resync',
    ): void {
        this.subscriptionGeneration += 1;
        const subscription = this.subscription;
        this.subscription = null;
        this.streamDead = false;
        try {
            subscription?.close();
        } catch {
            // A transport close failure must not block pending cleanup.
        } finally {
            this.failAll(error, reason);
        }
    }

    private async onStreamResult(result: HostToolResult): Promise<void> {
        const entry = this.pending.get(result.intent_id);
        if (entry === undefined) return;
        await this.settleFromResult(entry, result);
    }

    private async settleFromResult(
        entry: PendingEntry,
        value: unknown,
    ): Promise<void> {
        if (entry.settled) return;
        entry.settled = true;
        this.cleanupPending(entry);
        try {
            const validated = await validateHostToolResult(
                entry.intent,
                value,
            );
            entry.resolve(validated);
            for (const listener of this.streamListeners) {
                try {
                    listener.listener(validated);
                } catch {
                    // A consumer cannot break result delivery for other consumers.
                }
            }
        } catch (error) {
            entry.reject(toClientError(error));
        }
    }

    private rejectPending(
        entry: PendingEntry,
        error: AgentClientError,
    ): void {
        if (entry.settled) return;
        entry.settled = true;
        this.cleanupPending(entry);
        entry.reject(error);
    }

    private cleanupPending(entry: PendingEntry): void {
        if (entry.timer !== null) {
            this.timers.clearTimeout(entry.timer);
            entry.timer = null;
        }
        entry.removeAbort?.();
        entry.removeAbort = null;
        this.pending.delete(entry.intent.intent_id);
    }

    private failAll(
        error: AgentClientError,
        reason?: 'resync',
    ): void {
        for (const entry of [...this.pending.values()]) {
            this.rejectPending(entry, error);
        }
        for (const listener of this.streamListeners) {
            try {
                if (reason === 'resync') {
                    listener.onResync?.('cursor_expired');
                } else {
                    listener.onError?.(error);
                }
            } catch {
                // Listener failures do not mask the transport failure.
            }
        }
    }

    private async createDataIntent(
        input: HostDataRequest,
    ): Promise<HostToolIntent> {
        if (!isPlainObject(input)) {
            return invalidArguments('host data request must be an object');
        }
        const argumentsValue = canonicalToolArguments(input.arguments);
        const capability =
            input.tool === 'file.tree'
                ? 'tree'
                : input.tool === 'git.changes'
                  ? 'changes'
                  : input.tool === 'artifact.read'
                    ? 'readArtifact'
                    : null;
        if (capability === null) {
            return invalidArguments('host data tool is not supported');
        }
        const capabilities = await this.transport.capabilities();
        if (capabilities[capability] !== true) {
            throw new AgentClientError(
                'host_unavailable',
                `the local host companion does not provide ${input.tool}`,
            );
        }
        const digest = await argumentsDigest(argumentsValue);
        const counter = this.intentCounter++;
        const suffix = String(counter).padStart(12, '0');
        return validateHostToolIntent({
            schema: HOST_INTENT_SCHEMA,
            intent_id: `hti_data_${suffix}`,
            task_id: DATA_TASK_ID,
            run_id: DATA_RUN_ID,
            generation: DATA_GENERATION,
            tool_call_id: `tc_data_${suffix}`,
            tool: input.tool,
            arguments: argumentsValue,
            arguments_digest: digest,
            request_scope: 'data',
        });
    }
}
