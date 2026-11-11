import type {
    AgentTransport,
    ApprovalReceipt,
    HostCapabilities,
} from './client.js';
import {
    AgentClientError,
    isAgentErrorCode,
    isPlainObject,
    toClientError,
} from './storeTypes.js';
import { canonicalJson } from './generated/agentCanonical.js';
import {
    HOST_RESULT_SCHEMA,
    boundedPreview,
    sha256Canonical,
    type HostApprovalInput,
    type HostArtifactRef,
    type HostClientTimers,
    type HostDataRequest,
    type HostGrant,
    type HostInvokeResponse,
    type HostStreamingKind,
    type HostSubscribeOptions,
    type HostSubscription,
    type HostToolIntent,
    type HostToolResult,
    type HostTransport,
    type HostTransportCapabilities,
    type HostTransportKind,
} from './hostClient.js';

const LOOPBACK_TOKEN_HEADER = 'X-UOR-Desktop-Token';
const LOOPBACK_INTENTS_PATH = '/v1/studio/host/intents';
const LOOPBACK_RESULTS_PATH = '/v1/studio/host/results';
const LOOPBACK_ORIGIN_PATTERN =
    /^http:\/\/(127\.0\.0\.1|localhost|\[::1\])(?::\d+)?$/;
const DEFAULT_POLL_INTERVAL_MS = 250;

function errorCode(value: unknown): string {
    if (isPlainObject(value) && typeof value.code === 'string') {
        return isAgentErrorCode(value.code) ? value.code : 'host_unavailable';
    }
    return 'host_unavailable';
}

function errorForCode(code: string, message: string): AgentClientError {
    if (code === 'quota_exceeded') {
        return new AgentClientError('host_unavailable', message);
    }
    if (isAgentErrorCode(code)) {
        return new AgentClientError(code, message);
    }
    return new AgentClientError('host_unavailable', message);
}

function notifyError(
    options: HostSubscribeOptions,
    error: AgentClientError,
): void {
    try {
        options.onError?.(error);
    } catch {
        // A subscriber cannot break the transport lifecycle.
    }
}

function notifyResync(options: HostSubscribeOptions): void {
    try {
        options.onResync?.('cursor_expired');
    } catch {
        // A subscriber cannot break the transport lifecycle.
    }
}

function notifyEnd(options: HostSubscribeOptions): void {
    try {
        options.onEnd?.();
    } catch {
        // A subscriber cannot break the transport lifecycle.
    }
}


function falseCapabilities(
    kind: HostTransportKind,
    streaming: HostStreamingKind,
    hostAvailable: boolean,
): HostTransportCapabilities {
    return {
        kind,
        hostAvailable,
        streaming,
        tree: false,
        changes: false,
        readArtifact: false,
        terminal: false,
        git: false,
        mcp: false,
        approvals: false,
    };
}

function readCapabilities(
    value: unknown,
    kind: HostTransportKind,
    streaming: HostStreamingKind,
): HostTransportCapabilities {
    const record = isPlainObject(value) ? value : {};
    return {
        kind,
        hostAvailable: true,
        streaming,
        tree: record.tree === true,
        changes: record.changes === true,
        readArtifact: record.readArtifact === true,
        terminal: record.terminal === true,
        git: record.git === true,
        mcp: record.mcp === true,
        approvals: record.approvals === true,
    };
}

function isHostResult(value: unknown): value is HostToolResult {
    return (
        isPlainObject(value) &&
        value.schema === HOST_RESULT_SCHEMA
    );
}

function asHostResult(value: unknown): HostToolResult | null {
    if (isHostResult(value)) return value;
    if (isPlainObject(value) && isHostResult(value.result)) {
        return value.result;
    }
    return null;
}

abstract class UnavailableHostTransport implements HostTransport {
    abstract readonly kind: HostTransportKind;
    protected abstract readonly message: string;

    async capabilities(): Promise<HostTransportCapabilities> {
        return falseCapabilities(this.kind, 'none', false);
    }

    async invoke(_intent: HostToolIntent): Promise<HostToolResult> {
        throw new AgentClientError('host_unavailable', this.message);
    }

    subscribe(): HostSubscription {
        throw new AgentClientError('host_unavailable', this.message);
    }

    async approve(_input: HostApprovalInput): Promise<ApprovalReceipt> {
        throw new AgentClientError('host_unavailable', this.message);
    }
}

export class NullHostTransport extends UnavailableHostTransport {
    readonly kind = 'null' as const;
    protected readonly message =
        'the local host companion is not connected';
}

export class StaticHostTransport extends UnavailableHostTransport {
    readonly kind = 'static' as const;
    protected readonly message =
        'This action needs host approval. Approve it in the local companion.';
}

export interface TauriChannelHandle {
    onmessage: (message: unknown) => void;
    close?(): void;
}

export interface TauriHostBridge {
    readonly hasChannel: boolean;
    invoke(
        command: string,
        payload?: Record<string, unknown>,
    ): Promise<unknown>;
    channel(onMessage: (message: unknown) => void): TauriChannelHandle;
    closeChannel?(channel: TauriChannelHandle): void;
}

export interface TauriCoreLike {
    invoke(command: string, payload?: Record<string, unknown>): Promise<unknown>;
    Channel?: new () => TauriChannelHandle;
}

/**
 * Real Tauri adapter for the injected host bridge.
 *
 * It exposes a Channel only when the runtime actually provides one; the
 * offline invoke shim has no Channel, so the transport uses `agent_host_poll`.
 * No component calls `window.__TAURI__.core.invoke` directly.
 */
export function createTauriHostBridge(
    core?: TauriCoreLike | null,
): TauriHostBridge | null {
    const resolved =
        core ??
        (
            globalThis as {
                __TAURI__?: { core?: TauriCoreLike };
            }
        ).__TAURI__?.core ??
        null;
    if (resolved === null || typeof resolved.invoke !== 'function') {
        return null;
    }
    const Channel = resolved.Channel;
    return {
        hasChannel: typeof Channel === 'function',
        invoke: (command, payload) => resolved.invoke(command, payload),
        channel(onMessage): TauriChannelHandle {
            if (typeof Channel !== 'function') {
                throw new AgentClientError(
                    'host_unavailable',
                    'the Tauri runtime has no Channel transport',
                );
            }
            const channel = new Channel();
            channel.onmessage = onMessage;
            return channel;
        },
        closeChannel(channel): void {
            try {
                channel.close?.();
            } catch {
                // Closing a channel is best effort.
            }
        },
    };
}

export interface TauriHostGrant {
    grant_id: string;
    path: string;
}

export interface TauriHostTransportOptions {
    bridge?: TauriHostBridge | null;
    cursor?: string | null;
    scheduler?: HostClientTimers;
    pollIntervalMs?: number;
}

function readPoll(value: unknown): {
    cursor: string | null;
    results: HostToolResult[];
} {
    const collect = (values: unknown[]): HostToolResult[] =>
        values
            .map((item) => asHostResult(item))
            .filter((item): item is HostToolResult => item !== null);
    if (Array.isArray(value)) {
        return { cursor: null, results: collect(value) };
    }
    if (isPlainObject(value)) {
        return {
            cursor: typeof value.cursor === 'string' ? value.cursor : null,
            results: Array.isArray(value.results)
                ? collect(value.results)
                : [],
        };
    }
    return { cursor: null, results: [] };
}

function isInvokeAccepted(value: unknown): boolean {
    return (
        value === null ||
        (isPlainObject(value) &&
            (value.accepted === true || value.queued === true))
    );
}

interface TauriPollState {
    listener: (result: HostToolResult) => void;
    options: HostSubscribeOptions;
    cursor: string | null;
    closed: boolean;
    timer: unknown;
}

interface TauriChannelState {
    channel: TauriChannelHandle;
    closed: boolean;
}

export class TauriHostTransport implements HostTransport {
    readonly kind = 'tauri' as const;
    readonly supportsStream = true;
    private readonly bridge: TauriHostBridge | null;
    private readonly scheduler: HostClientTimers;
    private readonly pollIntervalMs: number;
    private readonly pollStates = new Set<TauriPollState>();
    private readonly channelStates = new Set<TauriChannelState>();
    private cursor: string | null;
    private closed = false;

    constructor(options: TauriHostTransportOptions = {}) {
        this.bridge =
            options.bridge === undefined
                ? createTauriHostBridge()
                : options.bridge;
        this.cursor = options.cursor ?? null;
        this.scheduler = options.scheduler ?? {
            setTimeout: (handler, timeoutMs) =>
                globalThis.setTimeout(handler, timeoutMs),
            clearTimeout: (handle) =>
                globalThis.clearTimeout(
                    handle as ReturnType<typeof setTimeout>,
                ),
        };
        this.pollIntervalMs =
            options.pollIntervalMs ?? DEFAULT_POLL_INTERVAL_MS;
    }

    async capabilities(): Promise<HostTransportCapabilities> {
        const bridge = this.requireBridge();
        return readCapabilities(
            await bridge.invoke('agent_host_capabilities'),
            'tauri',
            bridge.hasChannel ? 'channel' : 'poll',
        );
    }

    async invoke(intent: HostToolIntent): Promise<HostInvokeResponse> {
        const bridge = this.requireBridge();
        const invoked = await bridge.invoke('agent_host_invoke', { intent });
        const result = asHostResult(invoked);
        if (result !== null) return result;
        if (isInvokeAccepted(invoked)) {
            return { accepted: true, intent_id: intent.intent_id };
        }
        throw new AgentClientError(
            'invalid_event',
            'agent_host_invoke returned an invalid response',
        );
    }

    subscribe(
        listener: (result: HostToolResult) => void,
        options: HostSubscribeOptions = {},
    ): HostSubscription {
        const bridge = this.requireBridge();
        if (this.closed) {
            throw new AgentClientError(
                'transport_closed',
                'the Tauri host transport is closed',
            );
        }
        if (bridge.hasChannel) {
            return this.subscribeChannel(bridge, listener, options);
        }
        return this.subscribePoll(bridge, listener, options);
    }

    async hostGrants(): Promise<TauriHostGrant[]> {
        const bridge = this.requireBridge();
        const value = await bridge.invoke('agent_grants');
        if (!Array.isArray(value)) {
            throw new AgentClientError(
                'invalid_event',
                'agent_grants must return a grant array',
            );
        }
        return value.map((item) => {
            if (
                !isPlainObject(item) ||
                typeof item.grant_id !== 'string' ||
                !/^gr_[0-9a-f]{16}$/.test(item.grant_id) ||
                typeof item.path !== 'string' ||
                item.path.length === 0
            ) {
                throw new AgentClientError(
                    'invalid_event',
                    'agent_grants returned a malformed grant record',
                );
            }
            return { grant_id: item.grant_id, path: item.path };
        });
    }

    // The page-facing grant shape. Validation of the host record happens in
    // hostGrants(), and HostClient re-validates the page-facing values.
    async grants(): Promise<HostGrant[]> {
        return (await this.hostGrants()).map((grant) => ({
            grantId: grant.grant_id,
            path: grant.path,
        }));
    }

    async approve(input: HostApprovalInput): Promise<ApprovalReceipt> {
        const bridge = this.requireBridge();
        const response = await bridge.invoke('agent_host_resolve_approval', {
            input: {
                approval_id: input.approvalId,
                decision: input.decision,
            },
        });
        if (!isPlainObject(response)) {
            throw new AgentClientError(
                'invalid_event',
                'host approval response must be an object',
            );
        }
        if (
            response.approval_id !== input.approvalId ||
            response.decision !== input.decision
        ) {
            throw new AgentClientError(
                'approval_forbidden',
                'host approval response does not match the page decision',
            );
        }
        return {
            approvalId: input.approvalId,
            decision: input.decision,
        };
    }

    close(): void {
        if (this.closed) return;
        this.closed = true;
        for (const state of [...this.pollStates]) {
            if (state.timer !== null) {
                this.scheduler.clearTimeout(state.timer);
                state.timer = null;
            }
            state.closed = true;
            this.pollStates.delete(state);
        }
        for (const state of [...this.channelStates]) {
            state.closed = true;
            this.releaseChannel(state);
        }
    }

    private subscribeChannel(
        bridge: TauriHostBridge,
        listener: (result: HostToolResult) => void,
        options: HostSubscribeOptions,
    ): HostSubscription {
        const state: TauriChannelState = {
            channel: null as unknown as TauriChannelHandle,
            closed: false,
        };
        const channel = bridge.channel((message) => {
            if (state.closed) return;
            const result = asHostResult(message);
            if (result !== null) listener(result);
        });
        state.channel = channel;
        this.channelStates.add(state);
        void bridge
            .invoke('agent_subscribe', { channel })
            .catch((error: unknown) => {
                if (state.closed) return;
                state.closed = true;
                this.channelStates.delete(state);
                this.releaseChannel(state);
                notifyError(options, toClientError(error));
            });
        return {
            close: () => {
                if (state.closed) return;
                state.closed = true;
                this.channelStates.delete(state);
                this.releaseChannel(state);
            },
            cursor: () => this.cursor,
        };
    }

    private subscribePoll(
        bridge: TauriHostBridge,
        listener: (result: HostToolResult) => void,
        options: HostSubscribeOptions,
    ): HostSubscription {
        const state: TauriPollState = {
            listener,
            options,
            cursor: options.cursor ?? this.cursor,
            closed: false,
            timer: null,
        };
        this.pollStates.add(state);
        const tick = async (): Promise<void> => {
            if (state.closed) return;
            try {
                const polled = readPoll(
                    await bridge.invoke('agent_host_poll', {
                        cursor: state.cursor,
                    }),
                );
                if (state.closed) return;
                if (polled.cursor !== null) {
                    state.cursor = polled.cursor;
                    this.cursor = polled.cursor;
                }
                for (const result of polled.results) {
                    if (state.closed) return;
                    state.listener(result);
                }
                if (state.closed) return;
                state.timer = this.scheduler.setTimeout(() => {
                    void tick();
                }, this.pollIntervalMs);
            } catch (error) {
                if (state.closed) return;
                state.closed = true;
                this.pollStates.delete(state);
                notifyError(state.options, toClientError(error));
            }
        };
        void tick();
        return {
            close: () => {
                if (state.closed) return;
                state.closed = true;
                if (state.timer !== null) {
                    this.scheduler.clearTimeout(state.timer);
                    state.timer = null;
                }
                this.pollStates.delete(state);
            },
            cursor: () => state.cursor,
        };
    }

    private releaseChannel(state: TauriChannelState): void {
        state.closed = true;
        this.channelStates.delete(state);
        const bridge = this.bridge;
        try {
            state.channel.close?.();
        } catch {
            // Closing a channel is best effort.
        }
        try {
            bridge?.closeChannel?.(state.channel);
        } catch {
            // Closing a channel is best effort.
        }
    }

    private requireBridge(): TauriHostBridge {
        if (this.bridge === null) {
            throw new AgentClientError(
                'host_unavailable',
                'the Tauri host bridge is not connected',
            );
        }
        return this.bridge;
    }
}

export interface LoopbackHostTransportOptions {
    baseUrl?: string;
    pageOrigin?: string;
    token: string;
    fetch?: typeof fetch | null;
    cursor?: string | null;
    capabilities?: Partial<HostCapabilities>;
    onResync?: (reason: 'cursor_expired') => void;
}

export class LoopbackHostTransport implements HostTransport {
    readonly kind = 'loopback' as const;
    readonly supportsStream = true;
    private readonly baseUrl: string;
    private readonly token: string;
    private readonly fetchImpl: typeof fetch | null;
    private readonly capabilityValues: Partial<HostCapabilities>;
    private readonly onResync:
        | ((reason: 'cursor_expired') => void)
        | undefined;
    private cursor: string | null;

    constructor(options: LoopbackHostTransportOptions) {
        if (typeof options.token !== 'string' || options.token.length === 0) {
            throw new AgentClientError(
                'invalid_arguments',
                'loopback transport requires a per-launch token',
            );
        }
        if (
            options.baseUrl !== undefined &&
            typeof options.baseUrl !== 'string'
        ) {
            throw new AgentClientError(
                'invalid_arguments',
                'loopback base URL must be a string',
            );
        }
        if (
            options.pageOrigin !== undefined &&
            typeof options.pageOrigin !== 'string'
        ) {
            throw new AgentClientError(
                'invalid_arguments',
                'loopback page origin must be a string',
            );
        }
        const supplied = options.baseUrl;
        const pageOrigin =
            options.pageOrigin ??
            (globalThis as { location?: { origin?: string } }).location
                ?.origin;
        const candidate =
            supplied === undefined || supplied === ''
                ? pageOrigin
                : supplied;
        if (candidate === undefined || candidate === '') {
            throw new AgentClientError(
                'invalid_arguments',
                'loopback transport requires a local http page origin',
            );
        }
        const normalized = candidate.replace(/\/+$/, '');
        if (!LOOPBACK_ORIGIN_PATTERN.test(normalized)) {
            throw new AgentClientError(
                'invalid_arguments',
                'loopback base URL must be a local http origin',
            );
        }
        this.baseUrl = normalized;
        this.token = options.token;
        this.fetchImpl = options.fetch ?? null;
        this.capabilityValues = options.capabilities ?? {};
        this.onResync = options.onResync;
        this.cursor = options.cursor ?? null;
    }

    async capabilities(): Promise<HostTransportCapabilities> {
        const available = this.fetchImpl !== null;
        return {
            kind: 'loopback',
            hostAvailable: available,
            streaming: 'ndjson',
            tree: this.capabilityValues.tree === true,
            changes: this.capabilityValues.changes === true,
            readArtifact: this.capabilityValues.readArtifact === true,
            terminal: this.capabilityValues.terminal === true,
            git: this.capabilityValues.git === true,
            mcp: this.capabilityValues.mcp === true,
            // There is no loopback approval route and no native confirmation
            // dialog on this surface: approval-required tools are GUI-only or
            // deny-only here, never page-approved.
            approvals: false,
        };
    }

    async invoke(intent: HostToolIntent): Promise<HostInvokeResponse> {
        const response = await this.requireFetch()(this.url(LOOPBACK_INTENTS_PATH), {
            method: 'POST',
            headers: this.headers('application/json'),
            body: JSON.stringify(intent),
            credentials: 'omit',
            cache: 'no-store',
        });
        await this.ensureOk(response);
        const body = await this.safeJson(response);
        if (isPlainObject(body) && typeof body.code === 'string') {
            throw errorForCode(
                errorCode(body),
                'loopback host request failed',
            );
        }
        const result = asHostResult(body);
        if (result !== null) return result;
        if (body === null || isInvokeAccepted(body)) {
            return { accepted: true, intent_id: intent.intent_id };
        }
        throw new AgentClientError(
            'invalid_event',
            'loopback host response is not a HostToolResult',
        );
    }

    subscribe(
        listener: (result: HostToolResult) => void,
        options: HostSubscribeOptions = {},
    ): HostSubscription {
        const fetchImpl = this.fetchImpl;
        if (fetchImpl === null) {
            throw new AgentClientError(
                'host_unavailable',
                'the loopback host is not connected',
            );
        }
        const controller =
            typeof AbortController === 'function'
                ? new AbortController()
                : null;
        let closed = false;
        const cursor = options.cursor ?? this.cursor;
        void this.consume(fetchImpl, listener, cursor, controller).then(
            () => {
                if (closed) return;
                // A host stream that ends cleanly is a dead subscription, not
                // a reason to wait forever: an evicted subscriber or a closed
                // host must fail accepted pending work and allow a verified
                // resync. A direct-result invoke in flight is left to settle
                // from its own response.
                notifyEnd(options);
            },
            (error: unknown) => {
                if (closed) return;
                const clientError = toClientError(error);
                if (clientError.code === 'cursor_expired') {
                    notifyResync(options);
                    try {
                        this.onResync?.('cursor_expired');
                    } catch {
                        // A subscriber cannot break the transport lifecycle.
                    }
                    return;
                }
                notifyError(options, clientError);
            },
        );
        return {
            close: () => {
                closed = true;
                controller?.abort();
            },
            cursor: () => this.cursor,
        };
    }

    async approve(_input: HostApprovalInput): Promise<ApprovalReceipt> {
        throw new AgentClientError(
            'host_unavailable',
            'loopback host approval is not enabled in this stage',
        );
    }

    private async consume(
        fetchImpl: typeof fetch,
        listener: (result: HostToolResult) => void,
        cursor: string | null,
        controller: AbortController | null,
    ): Promise<void> {
        const query =
            cursor === null ? '' : `?cursor=${encodeURIComponent(cursor)}`;
        const aborted = abortPromise(controller);
        const responsePromise = fetchImpl(
            this.url(`${LOOPBACK_RESULTS_PATH}${query}`),
            {
                method: 'GET',
                headers: this.headers('application/x-ndjson'),
                credentials: 'omit',
                cache: 'no-store',
                signal: controller?.signal,
            },
        );
        // A hostile fetch that ignores the abort signal must not become an
        // unhandled rejection after disposal wins the race.
        const normalizedResponse = Promise.resolve(responsePromise);
        void normalizedResponse.catch(() => undefined);
        const response = await Promise.race([normalizedResponse, aborted]);
        await this.ensureOk(response);
        const headerCursor = response.headers.get('x-uor-host-cursor');
        if (headerCursor !== null && headerCursor.length > 0) {
            this.cursor = headerCursor;
        }
        if (response.body === null) {
            throw new AgentClientError(
                'transport_closed',
                'loopback host result stream has no body',
            );
        }
        const reader = response.body.getReader();
        const decoder = new TextDecoder();
        let buffer = '';
        try {
            for (;;) {
                const readPromise = reader.read();
                void readPromise.catch(() => undefined);
                const read = await Promise.race([readPromise, aborted]);
                if (read.done) break;
                buffer += decoder.decode(read.value, { stream: true });
                const lines = buffer.split('\n');
                buffer = lines.pop() ?? '';
                for (const line of lines) {
                    const trimmed = line.trim();
                    if (trimmed.length === 0) continue;
                    if (controller?.signal.aborted === true) return;
                    listener(this.parseLine(trimmed));
                }
            }
            buffer += decoder.decode();
            const trailing = buffer.trim();
            if (trailing.length > 0 && controller?.signal.aborted !== true) {
                listener(this.parseLine(trailing));
            }
        } finally {
            try {
                void reader.cancel().catch(() => undefined);
            } catch {
                // A disposal cannot block the transport lifecycle.
            }
            try {
                reader.releaseLock();
            } catch {
                // A pending read may already own the lock; disposal still wins.
            }
        }
    }

    private parseLine(line: string): HostToolResult {
        let parsed: unknown;
        try {
            parsed = JSON.parse(line);
        } catch {
            throw new AgentClientError(
                'truncated_frame',
                'loopback host stream line is not complete JSON',
            );
        }
        if (isPlainObject(parsed) && parsed.code === 'cursor_expired') {
            throw new AgentClientError(
                'cursor_expired',
                'host cursor expired; a snapshot resync is required',
            );
        }
        const result = asHostResult(parsed);
        if (result === null) {
            throw new AgentClientError(
                'invalid_event',
                'loopback host stream carried a non-HostToolResult frame',
            );
        }
        if (typeof result.cursor === 'string') {
            this.cursor = result.cursor;
        }
        return result;
    }

    private async ensureOk(response: Response): Promise<void> {
        if (response.status === 410) {
            throw new AgentClientError(
                'cursor_expired',
                'host cursor expired; a snapshot resync is required',
            );
        }
        if (response.ok) return;
        const body = await this.safeJson(response);
        const code = errorCode(body);
        if (code === 'cursor_expired') {
            throw new AgentClientError(
                'cursor_expired',
                'host cursor expired; a snapshot resync is required',
            );
        }
        throw errorForCode(
            response.status === 403 ? 'host_denied' : code,
            'loopback host request was refused',
        );
    }

    private async safeJson(response: Response): Promise<unknown> {
        try {
            return (await response.json()) as unknown;
        } catch {
            return null;
        }
    }

    private requireFetch(): typeof fetch {
        if (this.fetchImpl === null) {
            throw new AgentClientError(
                'host_unavailable',
                'the loopback host is not connected',
            );
        }
        return this.fetchImpl;
    }

    private headers(accept: string): Record<string, string> {
        return {
            accept,
            [LOOPBACK_TOKEN_HEADER]: this.token,
            ...(accept === 'application/json'
                ? { 'content-type': 'application/json' }
                : {}),
        };
    }

    private url(path: string): string {
        return `${this.baseUrl}${path}`;
    }
}

export class AgentTransportHostTransport implements HostTransport {
    readonly kind = 'bridge' as const;
    readonly supportsStream = false;

    constructor(private readonly transport: AgentTransport) {}

    async capabilities(): Promise<HostTransportCapabilities> {
        const capabilities = await this.transport.hostCapabilities();
        return {
            kind: 'bridge',
            hostAvailable:
                capabilities.tree ||
                capabilities.changes ||
                capabilities.readArtifact ||
                capabilities.terminal ||
                capabilities.git ||
                capabilities.mcp ||
                capabilities.approvals,
            streaming: 'none',
            ...capabilities,
        };
    }

    async invoke(
        intent: HostToolIntent,
        context?: HostDataRequest,
    ): Promise<HostToolResult> {
        const args = intent.arguments;
        if (intent.tool === 'file.tree') {
            const entries = await this.transport.tree({
                source: 'host',
                ...(typeof args.path === 'string' ? { path: args.path } : {}),
            });
            return hostResultFor(intent, {
                entries: entries.map((entry) => ({
                    path: entry.path,
                    kind: entry.kind,
                    ...(entry.sizeBytes === undefined
                        ? {}
                        : { size_bytes: entry.sizeBytes }),
                })),
            });
        }
        if (intent.tool === 'git.changes') {
            const taskId = context === undefined ? intent.task_id : context.taskId;
            if (taskId === undefined) {
                throw new AgentClientError(
                    'invalid_arguments',
                    'the host bridge requires a task for a branch snapshot',
                );
            }
            const runId =
                context === undefined ? intent.run_id : context.runId;
            const snapshot = await this.transport.changesSnapshot({
                taskId,
                ...(runId === undefined ? {} : { runId }),
                scope: 'branch',
            });
            return hostResultFor(intent, {
                snapshot:
                    snapshot === null
                        ? null
                        : {
                              scope: snapshot.scope,
                              files: snapshot.files,
                              head_digest: snapshot.headDigest,
                              captured_at_ms: snapshot.capturedAtMs,
                              ...(snapshot.truncated === true
                                  ? { truncated: true }
                                  : {}),
                          },
            });
        }
        if (intent.tool === 'artifact.read') {
            const taskId =
                intent.request_scope === 'data'
                    ? context?.taskId
                    : intent.task_id;
            const content = await this.transport.readArtifact({
                ...(taskId === undefined ? {} : { taskId }),
                ...(typeof args.path === 'string' ? { path: args.path } : {}),
                ...(typeof args.artifact_id === 'string'
                    ? { artifactId: args.artifact_id }
                    : {}),
                ...(typeof args.start === 'number' ? { start: args.start } : {}),
                ...(typeof args.end === 'number' ? { end: args.end } : {}),
            });
            const reportedArtifactId = content.artifactId;
            const wholeSize =
                content.artifactSizeBytes ??
                (content.range === undefined ? content.sizeBytes : undefined);
            const wholeDigest =
                content.wholeDigest ??
                (content.range === undefined ? content.digest : undefined);
            const artifact: HostArtifactRef | undefined =
                reportedArtifactId === undefined ||
                wholeSize === undefined ||
                wholeDigest === undefined
                    ? undefined
                    : {
                          artifact_id: reportedArtifactId,
                          digest: wholeDigest,
                          media_type: content.mediaType,
                          size_bytes: wholeSize,
                      };
            return hostResultFor(
                intent,
                {
                    ...(reportedArtifactId === undefined
                        ? {}
                        : { artifact_id: reportedArtifactId }),
                    path: content.path,
                    media_type: content.mediaType,
                    size_bytes: content.sizeBytes,
                    digest: content.digest,
                    ...(wholeSize === undefined
                        ? {}
                        : { artifact_size_bytes: wholeSize }),
                    ...(wholeDigest === undefined
                        ? {}
                        : { whole_digest: wholeDigest }),
                    ...(content.range === undefined
                        ? {}
                        : { range: content.range }),
                    bytes_base64: base64(content.bytes),
                },
                'ok',
                artifact,
            );
        }
        throw new AgentClientError(
            'host_unavailable',
            `the host bridge does not execute ${intent.tool}`,
        );
    }

    subscribe(): HostSubscription {
        throw new AgentClientError(
            'host_unavailable',
            'an AgentTransport carries AgentEvents, not host results',
        );
    }

    async approve(input: HostApprovalInput): Promise<ApprovalReceipt> {
        return this.transport.resolveApproval({
            approvalId: input.approvalId,
            decision: input.decision,
        });
    }
}

async function hostResultFor(
    intent: HostToolIntent,
    output: Record<string, unknown>,
    status: 'ok' | 'error' = 'ok',
    artifact?: HostArtifactRef,
): Promise<HostToolResult> {
    if (intent.approval_id !== undefined) {
        throw new AgentClientError(
            'approval_forbidden',
            'the fixture host bridge cannot issue host approval receipts',
        );
    }
    let canonical: string;
    try {
        canonical = canonicalJson(output);
    } catch (error) {
        throw new AgentClientError(
            'invalid_arguments',
            `host output is not canonical: ${
                error instanceof Error ? error.message : String(error)
            }`,
        );
    }
    return {
        schema: HOST_RESULT_SCHEMA,
        intent_id: intent.intent_id,
        task_id: intent.task_id,
        run_id: intent.run_id,
        generation: intent.generation,
        tool_call_id: intent.tool_call_id,
        request_scope: intent.request_scope ?? 'task',
        status,
        output: JSON.parse(canonical) as unknown,
        output_digest: await sha256Canonical(output),
        output_preview: boundedPreview(canonical),
        ...(artifact === undefined ? {} : { artifact }),
        occurred_at_ms: Date.now(),
    };
}

function base64(bytes: Uint8Array): string {
    let binary = '';
    for (const byte of bytes) binary += String.fromCharCode(byte);
    return globalThis.btoa(binary);
}

function abortPromise(controller: AbortController | null): Promise<never> {
    if (controller === null) return new Promise<never>(() => undefined);
    const aborted = new Promise<never>((_, reject) => {
        const fail = () => reject(new DOMException('Aborted', 'AbortError'));
        if (controller.signal.aborted) {
            fail();
        } else {
            controller.signal.addEventListener('abort', fail, { once: true });
        }
    });
    void aborted.catch(() => undefined);
    return aborted;
}
