import {
    AGENT_EVENT_SCHEMA,
    AGENT_EVENT_TYPES,
    AGENT_RUN_STATES,
    MAX_EVENT_BYTES,
    MAX_EVENTS_PER_RUN,
    type AgentErrorCode,
    type AgentEvent,
    type RunState,
    type UnknownAgentEvent,
} from '../agent/generated/agentContract.js';
import {
    CanonicalJsonError,
    canonicalJson,
    digestEvent,
    parseCanonicalJson,
} from '../agent/generated/agentCanonical.js';

const KNOWN_EVENT_TYPES: ReadonlySet<string> = new Set(AGENT_EVENT_TYPES);
const RUN_STATES: ReadonlySet<string> = new Set(AGENT_RUN_STATES);

const PROJECT_ID_PATTERN = /^prj_[0-9a-f]{16}$/;
const TASK_ID_PATTERN = /^tsk_[0-9a-f]{16}$/;
const RUN_ID_PATTERN = /^run_[0-9a-f]{16}$/;
const DIGEST_PATTERN = /^sha256:[0-9a-f]{64}$/;

export type AgentIncomingEvent = AgentEvent | UnknownAgentEvent;

export type TaskState =
    | 'idle'
    | 'queued'
    | 'running'
    | 'waiting_for_approval'
    | 'waiting_for_background'
    | 'cancelling'
    | 'finished'
    | 'failed'
    | 'cancelled'
    | 'expired'
    | 'archived'
    | 'restoring';

export type ToolCallStatus = 'requested' | 'started' | 'completed' | 'failed';

export interface AgentMessageView {
    messageId: string;
    text: string;
    sequence: number;
    finalized: boolean;
}

export interface AgentToolCallView {
    toolCallId: string;
    tool: string;
    status: ToolCallStatus;
    argumentsDigest: string | null;
    argumentsPreview: string | null;
    outputDigest: string | null;
    outputPreview: string | null;
    resultStatus: 'ok' | 'error' | 'cancelled' | null;
    code: string | null;
    message: string | null;
}

export interface AgentApprovalView {
    approvalId: string;
    toolCallId: string;
    tool: string;
    scope: string;
    requestDigest: string;
    expiresAtMs: number;
    decision: 'approved' | 'denied' | 'expired' | null;
}

export interface AgentChangeSnapshotView {
    scope: 'last_turn' | 'branch';
    files: Array<Record<string, unknown>>;
    headDigest: string;
    capturedAtMs: number;
    sequence: number;
}

export interface AgentUnknownEventView {
    eventId: string;
    type: string;
    data: Record<string, unknown>;
    generation: number;
    sequence: number;
    recordedAtMs: number;
}

export interface AgentAppliedEventView {
    eventId: string;
    type: string;
    generation: number;
    sequence: number;
    recordedAtMs: number;
    data: Record<string, unknown>;
}

export interface AgentGenerationTransition {
    from: number;
    to: number;
    atSequence: number;
    previousGeneration: number;
    snapshotDigest: string;
}

export interface RunErrorView {
    code: string;
    message: string;
    retryable: boolean;
}

// Delivery failures are message-scoped records, not run-level failures. The
// runtime emits them before a terminal event; keeping them out of RunErrorView
// means a successful or cancelled run cannot present a retained failure.
export interface MessageDeliveryErrorView {
    code: 'message_not_delivered';
    messageId: string | null;
    message: string;
}

export interface RunView {
    taskId: string;
    runId: string;
    generation: number;
    state: RunState;
    statusProjection: RunState | null;
    retryOfRunId: string | null;
    startedAtMs: number | null;
    endedAtMs: number | null;
    lastSequence: number;
    lastEventId: string | null;
    lastDigest: string | null;
    needsResync: boolean;
    resyncCode: AgentErrorCode | null;
    error: RunErrorView | null;
    deliveryErrors: MessageDeliveryErrorView[];
    messages: AgentMessageView[];
    toolCalls: AgentToolCallView[];
    approvals: AgentApprovalView[];
    changesSnapshots: AgentChangeSnapshotView[];
    unknownEvents: AgentUnknownEventView[];
    events: AgentAppliedEventView[];
    generationTransitions: AgentGenerationTransition[];
}

export interface TaskView {
    taskId: string;
    projectId: string;
    title: string;
    repo: string | null;
    branch: string | null;
    createdAtMs: number;
    archived: boolean;
    pinnedLocal: boolean;
    state: TaskState;
    latestRunId: string | null;
    unresolvedRunId: string | null;
    hold: {
        reason: 'waiting_for_background';
        run_id: string;
        tool_call_id: string;
    } | null;
    runs: RunView[];
}

export interface AgentStoreSnapshot {
    tasks: TaskView[];
    runs: RunView[];
    selectedTaskId: string | null;
    selectedTask: TaskView | null;
}

export interface AgentApplyResult {
    status: 'applied' | 'ignored' | 'rejected';
    code: AgentErrorCode | null;
    message: string | null;
    requiresResync: boolean;
}

export interface AgentAppendResult extends AgentApplyResult {
    eventId: string;
    digest: string | null;
    storage?: 'indexeddb' | null;
}

export interface AgentTaskRecord {
    taskId: string;
    projectId: string;
    title: string;
    repo?: string | null;
    branch?: string | null;
    createdAtMs: number;
    archived?: boolean;
    pinnedLocal?: boolean;
    latestRunId?: string | null;
    hold?: {
        reason: 'waiting_for_background';
        run_id: string;
        tool_call_id: string;
    } | null;
}

export interface AgentSnapshotVerification {
    taskId: string;
    runId: string;
    generation: number;
    lastSequence: number;
    lastEventId: string;
    headDigest: string;
}

export interface AgentVerifySnapshotResult {
    ok: boolean;
    code: AgentErrorCode | null;
    message: string | null;
}

export type AgentValidationResult =
    | { ok: true; event: AgentIncomingEvent }
    | { ok: false; code: AgentErrorCode; message: string };

export class AgentProtocolError extends Error {
    readonly code: AgentErrorCode;

    constructor(code: AgentErrorCode, message: string) {
        super(`${code}: ${message}`);
        this.name = 'AgentProtocolError';
        this.code = code;
    }
}

interface SeenEvent {
    canonical: string;
    digest: string | null;
}

interface InternalRun {
    taskId: string;
    projectId: string;
    runId: string;
    generation: number;
    state: RunState;
    statusProjection: RunState | null;
    retryOfRunId: string | null;
    startedAtMs: number | null;
    endedAtMs: number | null;
    lastSequence: number;
    lastEventId: string | null;
    lastDigest: string | null;
    needsResync: boolean;
    resyncCode: AgentErrorCode | null;
    error: RunErrorView | null;
    deliveryErrors: MessageDeliveryErrorView[];
    messages: AgentMessageView[];
    messagesById: Map<string, AgentMessageView>;
    deltasById: Map<string, string>;
    toolCalls: AgentToolCallView[];
    toolsById: Map<string, AgentToolCallView>;
    approvals: AgentApprovalView[];
    approvalsById: Map<string, AgentApprovalView>;
    changesSnapshots: AgentChangeSnapshotView[];
    unknownEvents: AgentUnknownEventView[];
    events: AgentAppliedEventView[];
    generationTransitions: AgentGenerationTransition[];
    seen: Map<string, SeenEvent>;
    verifiedSnapshot: AgentSnapshotVerification | null;
}

interface InternalTask {
    taskId: string;
    projectId: string;
    title: string;
    repo: string | null;
    branch: string | null;
    createdAtMs: number;
    archived: boolean;
    pinnedLocal: boolean;
    latestRunId: string | null;
    unresolvedRunId: string | null;
    maxGeneration: number;
    hold: {
        reason: 'waiting_for_background';
        run_id: string;
        tool_call_id: string;
    } | null;
}

function runKey(taskId: string, runId: string): string {
    return `${taskId}\u0000${runId}`;
}

function byteLength(text: string): number {
    return new TextEncoder().encode(text).byteLength;
}

function isObject(value: unknown): value is Record<string, unknown> {
    if (typeof value !== 'object' || value === null || Array.isArray(value)) {
        return false;
    }
    const prototype = Object.getPrototypeOf(value);
    return prototype === Object.prototype || prototype === null;
}

function isNonEmptyString(value: unknown): value is string {
    return typeof value === 'string' && value.length > 0;
}

function isSafeNonNegativeInteger(value: unknown): value is number {
    return typeof value === 'number' && Number.isSafeInteger(value) && value >= 0;
}

function isSafePositiveInteger(value: unknown): value is number {
    return typeof value === 'number' && Number.isSafeInteger(value) && value >= 1;
}

function invalid(message: string): AgentValidationResult {
    return { ok: false, code: 'invalid_event', message };
}

function terminalStateOf(type: string): RunState | null {
    switch (type) {
        case 'run.finished':
            return 'finished';
        case 'run.failed':
            return 'failed';
        case 'run.cancelled':
            return 'cancelled';
        case 'run.expired':
            return 'expired';
        default:
            return null;
    }
}

function isTerminalState(state: RunState): boolean {
    return (
        state === 'finished' ||
        state === 'failed' ||
        state === 'cancelled' ||
        state === 'expired'
    );
}

// run.status is a projection, never an authoritative terminal. Only a
// non-terminal cancelling projection changes the display; finished, failed and
// expired projections are ignored until a real terminal event arrives.
export function displayRunState(
    run: Pick<RunView, 'state' | 'statusProjection'>,
): RunState {
    if (isTerminalState(run.state)) return run.state;
    return run.statusProjection === 'cancelling' ? 'cancelling' : run.state;
}

function cloneJson<T>(value: T): T {
    return JSON.parse(JSON.stringify(value)) as T;
}

export interface AgentStoreCheckpoint {
    readonly opaque: unknown;
}

export interface AgentJournaledApply {
    result: AgentApplyResult;
    rollback(): void;
}

export interface AgentTaskCheckpoint {
    readonly opaque: unknown;
}

class AgentMutationJournal {
    private readonly undos: Array<() => void> = [];

    push(undo: () => void): void {
        this.undos.push(undo);
    }

    rollback(): void {
        while (this.undos.length > 0) {
            this.undos.pop()!();
        }
    }
}

function captureRunState(
    journal: AgentMutationJournal | null,
    run: InternalRun,
): void {
    if (journal === null) return;
    const state = {
        generation: run.generation,
        state: run.state,
        statusProjection: run.statusProjection,
        retryOfRunId: run.retryOfRunId,
        startedAtMs: run.startedAtMs,
        endedAtMs: run.endedAtMs,
        lastSequence: run.lastSequence,
        lastEventId: run.lastEventId,
        lastDigest: run.lastDigest,
        needsResync: run.needsResync,
        resyncCode: run.resyncCode,
        error: run.error === null ? null : { ...run.error },
        deliveryErrors: run.deliveryErrors.map((entry) => ({ ...entry })),
        verifiedSnapshot:
            run.verifiedSnapshot === null ? null : { ...run.verifiedSnapshot },
    };
    journal.push(() => {
        run.generation = state.generation;
        run.state = state.state;
        run.statusProjection = state.statusProjection;
        run.retryOfRunId = state.retryOfRunId;
        run.startedAtMs = state.startedAtMs;
        run.endedAtMs = state.endedAtMs;
        run.lastSequence = state.lastSequence;
        run.lastEventId = state.lastEventId;
        run.lastDigest = state.lastDigest;
        run.needsResync = state.needsResync;
        run.resyncCode = state.resyncCode;
        run.error = state.error === null ? null : { ...state.error };
        run.deliveryErrors = state.deliveryErrors.map((entry) => ({ ...entry }));
        run.verifiedSnapshot =
            state.verifiedSnapshot === null ? null : { ...state.verifiedSnapshot };
    });
}

function captureTaskState(
    journal: AgentMutationJournal | null,
    task: InternalTask,
): void {
    if (journal === null) return;
    const state = {
        title: task.title,
        repo: task.repo,
        branch: task.branch,
        archived: task.archived,
        pinnedLocal: task.pinnedLocal,
        latestRunId: task.latestRunId,
        unresolvedRunId: task.unresolvedRunId,
        maxGeneration: task.maxGeneration,
        hold: task.hold === null ? null : { ...task.hold },
    };
    journal.push(() => {
        task.title = state.title;
        task.repo = state.repo;
        task.branch = state.branch;
        task.archived = state.archived;
        task.pinnedLocal = state.pinnedLocal;
        task.latestRunId = state.latestRunId;
        task.unresolvedRunId = state.unresolvedRunId;
        task.maxGeneration = state.maxGeneration;
        task.hold = state.hold;
    });
}

function captureNewRun(
    journal: AgentMutationJournal | null,
    runs: Map<string, InternalRun>,
    runOrder: string[],
    runOwners: Map<string, string>,
    key: string,
    runId: string,
): void {
    if (journal === null) return;
    const orderLength = runOrder.length;
    journal.push(() => {
        runs.delete(key);
        runOwners.delete(runId);
        runOrder.length = orderLength;
    });
}

function captureArrayLength(
    journal: AgentMutationJournal | null,
    array: unknown[],
): void {
    if (journal === null) return;
    const length = array.length;
    journal.push(() => {
        array.length = length;
    });
}

function captureMapValue<K, V>(
    journal: AgentMutationJournal | null,
    map: Map<K, V>,
    key: K,
): void {
    if (journal === null) return;
    const had = map.has(key);
    const value = map.get(key);
    journal.push(() => {
        if (had) {
            map.set(key, value as V);
        } else {
            map.delete(key);
        }
    });
}

function captureFields<T extends object>(
    journal: AgentMutationJournal | null,
    target: T,
    fields: Array<keyof T>,
): void {
    if (journal === null) return;
    const values = fields.map((field) => target[field]);
    journal.push(() => {
        fields.forEach((field, index) => {
            target[field] = values[index]!;
        });
    });
}

interface AgentStoreInternalState {
    tasks: Map<string, InternalTask>;
    runs: Map<string, InternalRun>;
    runOwners: Map<string, string>;
    taskOrder: string[];
    runOrder: string[];
    selectedTaskId: string | null;
}

function cloneInternalRun(run: InternalRun): InternalRun {
    const messages = run.messages.map((message) => ({ ...message }));
    const toolCalls = run.toolCalls.map((toolCall) => ({ ...toolCall }));
    const approvals = run.approvals.map((approval) => ({ ...approval }));
    const changesSnapshots = run.changesSnapshots.map((snapshot) => ({
        ...snapshot,
        files: cloneJson(snapshot.files),
    }));
    const unknownEvents = run.unknownEvents.map((event) => ({
        ...event,
        data: cloneJson(event.data),
    }));
    const events = run.events.map((event) => ({
        ...event,
        data: cloneJson(event.data),
    }));
    return {
        taskId: run.taskId,
        projectId: run.projectId,
        runId: run.runId,
        generation: run.generation,
        state: run.state,
        statusProjection: run.statusProjection,
        retryOfRunId: run.retryOfRunId,
        startedAtMs: run.startedAtMs,
        endedAtMs: run.endedAtMs,
        lastSequence: run.lastSequence,
        lastEventId: run.lastEventId,
        lastDigest: run.lastDigest,
        needsResync: run.needsResync,
        resyncCode: run.resyncCode,
        error: run.error === null ? null : { ...run.error },
        deliveryErrors: run.deliveryErrors.map((entry) => ({ ...entry })),
        messages,
        messagesById: new Map(messages.map((message) => [message.messageId, message])),
        deltasById: new Map(run.deltasById),
        toolCalls,
        toolsById: new Map(toolCalls.map((toolCall) => [toolCall.toolCallId, toolCall])),
        approvals,
        approvalsById: new Map(approvals.map((approval) => [approval.approvalId, approval])),
        changesSnapshots,
        unknownEvents,
        events,
        generationTransitions: run.generationTransitions.map(
            (transition) => ({ ...transition }),
        ),
        seen: new Map(
            [...run.seen].map(([eventId, seen]) => [eventId, { ...seen }]),
        ),
        verifiedSnapshot:
            run.verifiedSnapshot === null ? null : { ...run.verifiedSnapshot },
    };
}

function applied(): AgentApplyResult {
    return {
        status: 'applied',
        code: null,
        message: null,
        requiresResync: false,
    };
}

function ignored(): AgentApplyResult {
    return {
        status: 'ignored',
        code: null,
        message: 'duplicate event ignored',
        requiresResync: false,
    };
}

function rejected(
    code: AgentErrorCode,
    message: string,
    requiresResync: boolean,
): AgentApplyResult {
    return {
        status: 'rejected',
        code,
        message,
        requiresResync,
    };
}

function requireSafeField(
    data: Record<string, unknown>,
    field: string,
): AgentValidationResult | null {
    if (!isSafeNonNegativeInteger(data[field])) {
        return invalid(`${field} must be a non-negative safe integer`);
    }
    return null;
}

export function validateAgentEvent(value: unknown): AgentValidationResult {
    if (!isObject(value)) {
        return invalid('an agent event must be a JSON object');
    }
    if (value.schema !== AGENT_EVENT_SCHEMA) {
        return invalid(`schema must be ${AGENT_EVENT_SCHEMA}`);
    }
    const projectId = value.project_id;
    const taskId = value.task_id;
    const runId = value.run_id;
    const generation = value.generation;
    const sequence = value.sequence;
    const recordedAtMs = value.recorded_at_ms;
    const type = value.type;
    const data = value.data;

    if (typeof projectId !== 'string' || !PROJECT_ID_PATTERN.test(projectId)) {
        return invalid('project_id must match prj_<16 lowercase hex>');
    }
    if (typeof taskId !== 'string' || !TASK_ID_PATTERN.test(taskId)) {
        return invalid('task_id must match tsk_<16 lowercase hex>');
    }
    if (typeof runId !== 'string' || !RUN_ID_PATTERN.test(runId)) {
        return invalid('run_id must match run_<16 lowercase hex>');
    }
    if (!isSafePositiveInteger(generation)) {
        return invalid('generation must be a positive safe integer');
    }
    if (!isSafeNonNegativeInteger(sequence)) {
        return invalid('sequence must be a non-negative safe integer');
    }
    if (!isSafeNonNegativeInteger(recordedAtMs)) {
        return invalid('recorded_at_ms must be a non-negative safe integer');
    }
    if (typeof type !== 'string' || type.length === 0) {
        return invalid('type must be a non-empty string');
    }
    if (!isObject(data)) {
        return invalid('event data must be an object');
    }
    const expectedEventId = `evt_${runId}_${generation}_${sequence
        .toString()
        .padStart(12, '0')}`;
    if (value.event_id !== expectedEventId) {
        return invalid(`event_id must be ${expectedEventId}`);
    }

    if (type === 'run.queued') {
        const problem = requireSafeField(data, 'queued_at_ms');
        if (problem) return problem;
    }
    if (type === 'run.started') {
        const retry = data.retry_of_run_id;
        if (
            !(
                retry === null ||
                (typeof retry === 'string' && RUN_ID_PATTERN.test(retry))
            )
        ) {
            return invalid('run.started retry_of_run_id must be string | null');
        }
        if (data.mode !== 'plan' && data.mode !== 'agent') {
            return invalid('run.started mode must be plan | agent');
        }
        const problem = requireSafeField(data, 'started_at_ms');
        if (problem) return problem;
        if (!isObject(data.model) || typeof data.model.id !== 'string' || typeof data.model.kind !== 'string') {
            return invalid('run.started model must carry id and kind');
        }
        if (!isObject(data.environment) || typeof data.environment.kind !== 'string') {
            return invalid('run.started environment must carry kind');
        }
    }
    if (type === 'run.resumed') {
        if (!isSafePositiveInteger(data.previous_generation)) {
            return invalid('run.resumed previous_generation must be a positive safe integer');
        }
        const problem = requireSafeField(data, 'resumed_at_ms');
        if (problem) return problem;
        if (
            typeof data.snapshot_digest !== 'string' ||
            !DIGEST_PATTERN.test(data.snapshot_digest)
        ) {
            return invalid('run.resumed snapshot_digest must be sha256:<64 lowercase hex>');
        }
    }
    if (type === 'run.status') {
        if (typeof data.status !== 'string' || !RUN_STATES.has(data.status)) {
            return invalid('run.status status must be a run state');
        }
    }
    if (type === 'run.finished') {
        if (data.status !== 'finished') {
            return invalid('run.finished status must be finished');
        }
        const finished = requireSafeField(data, 'finished_at_ms');
        if (finished) return finished;
        const duration = requireSafeField(data, 'duration_ms');
        if (duration) return duration;
    }
    if (type === 'run.failed') {
        if (typeof data.code !== 'string' || typeof data.message !== 'string') {
            return invalid('run.failed code and message must be strings');
        }
        if (typeof data.retryable !== 'boolean') {
            return invalid('run.failed retryable must be a boolean');
        }
    }
    if (type === 'run.cancelled') {
        if (data.safe_boundary !== true) {
            return invalid('run.cancelled safe_boundary must be true');
        }
        const finished = requireSafeField(data, 'finished_at_ms');
        if (finished) return finished;
    }
    if (type === 'run.expired') {
        const expired = requireSafeField(data, 'expired_at_ms');
        if (expired) return expired;
        if (typeof data.reason !== 'string') {
            return invalid('run.expired reason must be a string');
        }
    }
    if (type === 'assistant.message') {
        if (typeof data.message_id !== 'string' || typeof data.text !== 'string') {
            return invalid('assistant.message requires message_id and text strings');
        }
    }
    if (type === 'assistant.delta') {
        if (
            typeof data.message_id !== 'string' ||
            typeof data.text_delta !== 'string'
        ) {
            return invalid('assistant.delta requires message_id and text_delta strings');
        }
    }
    if (type === 'changes.snapshot') {
        if (data.scope !== 'last_turn' && data.scope !== 'branch') {
            return invalid('changes.snapshot scope must be last_turn | branch');
        }
        if (
            !Array.isArray(data.files) ||
            !data.files.every((file) => isObject(file))
        ) {
            return invalid('changes.snapshot files must be an array of objects');
        }
        if (!isNonEmptyString(data.head_digest)) {
            return invalid('changes.snapshot head_digest must be a non-empty string');
        }
        const captured = requireSafeField(data, 'captured_at_ms');
        if (captured) return captured;
    }
    if (type === 'tool.requested') {
        if (
            !isNonEmptyString(data.tool_call_id) ||
            !isNonEmptyString(data.tool)
        ) {
            return invalid('tool.requested requires tool_call_id and tool strings');
        }
        if (
            typeof data.arguments_digest !== 'string' ||
            !DIGEST_PATTERN.test(data.arguments_digest)
        ) {
            return invalid(
                'tool.requested arguments_digest must be sha256:<64 lowercase hex>',
            );
        }
        if (typeof data.arguments_preview !== 'string') {
            return invalid('tool.requested arguments_preview must be a string');
        }
    }
    if (type === 'tool.started') {
        if (!isNonEmptyString(data.tool_call_id)) {
            return invalid('tool.started requires tool_call_id');
        }
    }
    if (type === 'tool.completed') {
        if (!isNonEmptyString(data.tool_call_id)) {
            return invalid('tool.completed requires tool_call_id');
        }
        if (
            data.status !== 'ok' &&
            data.status !== 'error' &&
            data.status !== 'cancelled'
        ) {
            return invalid('tool.completed status is not valid');
        }
        if (
            typeof data.output_digest !== 'string' ||
            !DIGEST_PATTERN.test(data.output_digest)
        ) {
            return invalid(
                'tool.completed output_digest must be sha256:<64 lowercase hex>',
            );
        }
        if (typeof data.output_preview !== 'string') {
            return invalid('tool.completed output_preview must be a string');
        }
        if (typeof data.truncated !== 'boolean') {
            return invalid('tool.completed truncated must be a boolean');
        }
    }
    if (type === 'tool.failed') {
        if (!isNonEmptyString(data.tool_call_id) || !isNonEmptyString(data.code)) {
            return invalid('tool.failed requires tool_call_id and code strings');
        }
        if (typeof data.message !== 'string') {
            return invalid('tool.failed message must be a string');
        }
    }
    if (type === 'tool.approval_required') {
        if (
            !isNonEmptyString(data.approval_id) ||
            !isNonEmptyString(data.tool_call_id) ||
            !isNonEmptyString(data.tool) ||
            !isNonEmptyString(data.scope)
        ) {
            return invalid(
                'tool.approval_required requires approval_id, tool_call_id, tool and scope strings',
            );
        }
        if (
            typeof data.request_digest !== 'string' ||
            !DIGEST_PATTERN.test(data.request_digest)
        ) {
            return invalid(
                'tool.approval_required request_digest must be sha256:<64 lowercase hex>',
            );
        }
        const expires = requireSafeField(data, 'expires_at_ms');
        if (expires) return expires;
    }
    if (type === 'tool.approval_resolved') {
        if (!isNonEmptyString(data.approval_id)) {
            return invalid('tool.approval_resolved requires approval_id');
        }
        if (
            data.decision !== 'approved' &&
            data.decision !== 'denied' &&
            data.decision !== 'expired'
        ) {
            return invalid('tool.approval_resolved decision is not valid');
        }
        if (data.resolver !== 'host') {
            return invalid('tool.approval_resolved resolver must be host');
        }
    }
    if (type === 'error') {
        if (!isNonEmptyString(data.code)) {
            return invalid('error requires a code string');
        }
        if (typeof data.message !== 'string') {
            return invalid('error message must be a string');
        }
        if (typeof data.retryable !== 'boolean') {
            return invalid('error retryable must be a boolean');
        }
    }

    return { ok: true, event: value as unknown as AgentIncomingEvent };
}

export function parseAgentEventWire(text: string): AgentIncomingEvent {
    if (typeof text !== 'string') {
        throw new AgentProtocolError('malformed_event', 'wire frame must be a string');
    }
    if (byteLength(text) > MAX_EVENT_BYTES) {
        throw new AgentProtocolError(
            'limit_exceeded',
            'wire frame exceeds max_event_bytes',
        );
    }

    let canonical: string;
    try {
        canonical = parseCanonicalJson(text);
    } catch (error) {
        if (error instanceof CanonicalJsonError) {
            const prefix = `${error.code}: `;
            throw new AgentProtocolError(
                error.code,
                error.message.startsWith(prefix)
                    ? error.message.slice(prefix.length)
                    : error.message,
            );
        }
        throw new AgentProtocolError(
            'malformed_event',
            'wire frame is not canonical JSON',
        );
    }

    // parseCanonicalJson has already rejected duplicate keys, lexical floats,
    // negative or unsafe integers, BOM, trailing content and malformed text.
    // JSON.parse only materializes the already-canonical value; it never
    // performs raw lexical validation itself.
    let value: unknown;
    try {
        value = JSON.parse(canonical);
    } catch {
        throw new AgentProtocolError(
            'malformed_event',
            'canonical frame could not be materialized',
        );
    }

    const validation = validateAgentEvent(value);
    if (!validation.ok) {
        throw new AgentProtocolError(validation.code, validation.message);
    }
    return validation.event;
}

function createRun(task: InternalTask, event: AgentIncomingEvent): InternalRun {
    return {
        taskId: task.taskId,
        projectId: task.projectId,
        runId: event.run_id,
        generation: event.generation,
        state: 'queued',
        statusProjection: null,
        retryOfRunId: null,
        startedAtMs: null,
        endedAtMs: null,
        lastSequence: -1,
        lastEventId: null,
        lastDigest: null,
        needsResync: false,
        resyncCode: null,
        error: null,
        deliveryErrors: [],
        messages: [],
        messagesById: new Map(),
        deltasById: new Map(),
        toolCalls: [],
        toolsById: new Map(),
        approvals: [],
        approvalsById: new Map(),
        changesSnapshots: [],
        unknownEvents: [],
        events: [],
        generationTransitions: [],
        seen: new Map(),
        verifiedSnapshot: null,
    };
}

function upsertMessage(
    run: InternalRun,
    messageId: string,
    text: string,
    sequence: number,
    finalized: boolean,
    journal: AgentMutationJournal | null,
): void {
    const existing = run.messagesById.get(messageId);
    if (existing !== undefined) {
        captureFields(journal, existing, ['text', 'sequence', 'finalized']);
        existing.text = text;
        existing.sequence = sequence;
        existing.finalized = finalized;
        return;
    }
    const message: AgentMessageView = {
        messageId,
        text,
        sequence,
        finalized,
    };
    captureArrayLength(journal, run.messages);
    captureMapValue(journal, run.messagesById, messageId);
    run.messagesById.set(messageId, message);
    run.messages.push(message);
}

function upsertToolCall(
    run: InternalRun,
    toolCallId: string,
    journal: AgentMutationJournal | null,
): AgentToolCallView {
    const existing = run.toolsById.get(toolCallId);
    if (existing !== undefined) {
        captureFields(journal, existing, [
            'tool',
            'status',
            'argumentsDigest',
            'argumentsPreview',
            'outputDigest',
            'outputPreview',
            'resultStatus',
            'code',
            'message',
        ]);
        return existing;
    }
    const toolCall: AgentToolCallView = {
        toolCallId,
        tool: '',
        status: 'requested',
        argumentsDigest: null,
        argumentsPreview: null,
        outputDigest: null,
        outputPreview: null,
        resultStatus: null,
        code: null,
        message: null,
    };
    captureArrayLength(journal, run.toolCalls);
    captureMapValue(journal, run.toolsById, toolCallId);
    run.toolsById.set(toolCallId, toolCall);
    run.toolCalls.push(toolCall);
    return toolCall;
}

const MESSAGE_ID_PATTERN = /msg_[0-9a-f]{16}/;

function deliveryMessageId(message: string): string | null {
    const match = MESSAGE_ID_PATTERN.exec(message);
    return match === null ? null : match[0];
}

function reduceKnownEvent(
    run: InternalRun,
    event: AgentIncomingEvent,
    journal: AgentMutationJournal | null,
): void {
    const data = event.data as Record<string, unknown>;
    switch (event.type) {
        case 'run.queued':
            run.state = 'queued';
            break;
        case 'run.started':
            run.state = 'running';
            run.retryOfRunId = data.retry_of_run_id as string | null;
            run.startedAtMs = data.started_at_ms as number;
            break;
        case 'run.status':
            run.statusProjection = data.status as RunState;
            break;
        case 'run.finished':
            run.state = 'finished';
            run.endedAtMs = data.finished_at_ms as number;
            break;
        case 'run.failed':
            run.state = 'failed';
            run.endedAtMs = event.recorded_at_ms;
            run.error = {
                code: data.code as string,
                message: data.message as string,
                retryable: data.retryable as boolean,
            };
            break;
        case 'run.cancelled':
            run.state = 'cancelled';
            run.endedAtMs = data.finished_at_ms as number;
            break;
        case 'run.expired':
            run.state = 'expired';
            run.endedAtMs = data.expired_at_ms as number;
            break;
        case 'error': {
            const code = data.code as string;
            if (code === 'message_not_delivered') {
                const message = data.message as string;
                captureArrayLength(journal, run.deliveryErrors);
                run.deliveryErrors.push({
                    code: 'message_not_delivered',
                    messageId: deliveryMessageId(message),
                    message,
                });
                break;
            }
            run.error = {
                code,
                message: data.message as string,
                retryable: data.retryable as boolean,
            };
            break;
        }
        case 'assistant.message': {
            const messageId = data.message_id as string;
            const text = data.text as string;
            captureMapValue(journal, run.deltasById, messageId);
            run.deltasById.set(messageId, text);
            upsertMessage(run, messageId, text, event.sequence, true, journal);
            break;
        }
        case 'assistant.delta': {
            const messageId = data.message_id as string;
            const delta = `${run.deltasById.get(messageId) ?? ''}${
                data.text_delta as string
            }`;
            captureMapValue(journal, run.deltasById, messageId);
            run.deltasById.set(messageId, delta);
            upsertMessage(run, messageId, delta, event.sequence, false, journal);
            break;
        }
        case 'tool.requested': {
            const toolCall = upsertToolCall(run, data.tool_call_id as string, journal);
            toolCall.tool = data.tool as string;
            toolCall.status = 'requested';
            toolCall.argumentsDigest = data.arguments_digest as string;
            toolCall.argumentsPreview = data.arguments_preview as string;
            break;
        }
        case 'tool.started': {
            const toolCall = upsertToolCall(run, data.tool_call_id as string, journal);
            toolCall.status = 'started';
            break;
        }
        case 'tool.completed': {
            const toolCall = upsertToolCall(run, data.tool_call_id as string, journal);
            toolCall.status = 'completed';
            toolCall.outputDigest = data.output_digest as string;
            toolCall.outputPreview = data.output_preview as string;
            toolCall.resultStatus = data.status as 'ok' | 'error' | 'cancelled';
            break;
        }
        case 'tool.failed': {
            const toolCall = upsertToolCall(run, data.tool_call_id as string, journal);
            toolCall.status = 'failed';
            toolCall.code = data.code as string;
            toolCall.message = data.message as string;
            break;
        }
        case 'tool.approval_required': {
            const approvalId = data.approval_id as string;
            if (!run.approvalsById.has(approvalId)) {
                const approval: AgentApprovalView = {
                    approvalId,
                    toolCallId: data.tool_call_id as string,
                    tool: data.tool as string,
                    scope: data.scope as string,
                    requestDigest: data.request_digest as string,
                    expiresAtMs: data.expires_at_ms as number,
                    decision: null,
                };
                captureArrayLength(journal, run.approvals);
                captureMapValue(journal, run.approvalsById, approvalId);
                run.approvalsById.set(approvalId, approval);
                run.approvals.push(approval);
            }
            run.state = 'waiting_for_approval';
            break;
        }
        case 'tool.approval_resolved': {
            const approval = run.approvalsById.get(data.approval_id as string);
            if (approval !== undefined) {
                captureFields(journal, approval, ['decision']);
                approval.decision = data.decision as
                    | 'approved'
                    | 'denied'
                    | 'expired';
            }
            if (run.state === 'waiting_for_approval') {
                run.state = 'running';
            }
            break;
        }
        case 'changes.snapshot':
            captureArrayLength(journal, run.changesSnapshots);
            run.changesSnapshots.push({
                scope: data.scope as 'last_turn' | 'branch',
                files: cloneJson(data.files as Array<Record<string, unknown>>),
                headDigest: data.head_digest as string,
                capturedAtMs: data.captured_at_ms as number,
                sequence: event.sequence,
            });
            break;
        default:
            break;
    }
}

function taskStateFor(
    task: InternalTask,
    latestRun: RunView | null,
    unresolvedRunId: string | null,
): TaskState {
    if (task.archived) {
        return 'archived';
    }
    if (latestRun === null) {
        if (unresolvedRunId !== null) return 'restoring';
        return task.hold === null ? 'idle' : 'waiting_for_background';
    }
    if (
        task.hold !== null &&
        task.hold.run_id === latestRun.runId &&
        isTerminalState(latestRun.state)
    ) {
        return 'waiting_for_background';
    }
    switch (displayRunState(latestRun)) {
        case 'queued':
            return 'queued';
        case 'running':
            return 'running';
        case 'waiting_for_approval':
            return 'waiting_for_approval';
        case 'waiting_for_background':
            return 'waiting_for_background';
        case 'cancelling':
            return 'cancelling';
        case 'finished':
        case 'failed':
        case 'cancelled':
        case 'expired':
            return displayRunState(latestRun);
        default:
            return 'idle';
    }
}

function copyRun(run: InternalRun): RunView {
    return {
        taskId: run.taskId,
        runId: run.runId,
        generation: run.generation,
        state: run.state,
        statusProjection: run.statusProjection,
        retryOfRunId: run.retryOfRunId,
        startedAtMs: run.startedAtMs,
        endedAtMs: run.endedAtMs,
        lastSequence: run.lastSequence,
        lastEventId: run.lastEventId,
        lastDigest: run.lastDigest,
        needsResync: run.needsResync,
        resyncCode: run.resyncCode,
        error: run.error === null ? null : { ...run.error },
        deliveryErrors: run.deliveryErrors.map((entry) => ({ ...entry })),
        messages: run.messages.map((message) => ({ ...message })),
        toolCalls: run.toolCalls.map((toolCall) => ({ ...toolCall })),
        approvals: run.approvals.map((approval) => ({ ...approval })),
        changesSnapshots: run.changesSnapshots.map((snapshot) => ({
            ...snapshot,
            files: cloneJson(snapshot.files),
        })),
        unknownEvents: run.unknownEvents.map((unknown) => ({
            ...unknown,
            data: cloneJson(unknown.data),
        })),
        events: run.events.map((event) => ({
            ...event,
            data: cloneJson(event.data),
        })),
        generationTransitions: run.generationTransitions.map(
            (transition) => ({ ...transition }),
        ),
    };
}

export class AgentStore {
    private readonly tasks = new Map<string, InternalTask>();
    private readonly runs = new Map<string, InternalRun>();
    private readonly runOwners = new Map<string, string>();
    private readonly taskOrder: string[] = [];
    private readonly runOrder: string[] = [];
    private selectedTaskId: string | null = null;

    // Durable persistence owns a two-phase commit around apply(): it captures
    // this exact checkpoint, applies synchronously, and restores the whole
    // reducer state if the IndexedDB transaction aborts. The checkpoint is an
    // opaque state copy, not a second task projection.
    checkpoint(): AgentStoreCheckpoint {
        const state: AgentStoreInternalState = {
            tasks: new Map(
                [...this.tasks].map(([taskId, task]) => [taskId, { ...task }]),
            ),
            runs: new Map(
                [...this.runs].map(([key, run]) => [key, cloneInternalRun(run)]),
            ),
            runOwners: new Map(this.runOwners),
            taskOrder: [...this.taskOrder],
            runOrder: [...this.runOrder],
            selectedTaskId: this.selectedTaskId,
        };
        return { opaque: state };
    }

    captureTask(taskId: string): AgentTaskCheckpoint {
        const task = this.tasks.get(taskId);
        return {
            opaque: {
                taskId,
                task: task === undefined ? null : { ...task },
            },
        };
    }

    restoreTask(checkpoint: AgentTaskCheckpoint): void {
        const state = checkpoint.opaque as {
            taskId: string;
            task: InternalTask | null;
        };
        if (state.task === null) {
            this.tasks.delete(state.taskId);
            const index = this.taskOrder.indexOf(state.taskId);
            if (index >= 0) this.taskOrder.splice(index, 1);
            return;
        }
        this.tasks.set(state.taskId, state.task);
    }

    reset(): void {
        this.tasks.clear();
        this.runs.clear();
        this.runOwners.clear();
        this.taskOrder.splice(0, this.taskOrder.length);
        this.runOrder.splice(0, this.runOrder.length);
        this.selectedTaskId = null;
    }

    getRun(taskId: string, runId: string): RunView | null {
        const run = this.runs.get(runKey(taskId, runId));
        return run === undefined ? null : copyRun(run);
    }

    getRunCursor(taskId: string, runId: string): {
        generation: number;
        lastSequence: number;
        lastEventId: string | null;
        lastDigest: string | null;
    } | null {
        const run = this.runs.get(runKey(taskId, runId));
        return run === undefined
            ? null
            : {
                  generation: run.generation,
                  lastSequence: run.lastSequence,
                  lastEventId: run.lastEventId,
                  lastDigest: run.lastDigest,
              };
    }

    getRunRecordState(taskId: string, runId: string): {
        generation: number;
        state: RunState;
        retryOfRunId: string | null;
        startedAtMs: number | null;
        firstRecordedAtMs: number | null;
        endedAtMs: number | null;
        lastSequence: number;
        lastEventId: string | null;
        lastDigest: string | null;
    } | null {
        const run = this.runs.get(runKey(taskId, runId));
        return run === undefined
            ? null
            : {
                  generation: run.generation,
                  state: run.state,
                  retryOfRunId: run.retryOfRunId,
                  startedAtMs: run.startedAtMs,
                  firstRecordedAtMs: run.events[0]?.recordedAtMs ?? null,
                  endedAtMs: run.endedAtMs,
                  lastSequence: run.lastSequence,
                  lastEventId: run.lastEventId,
                  lastDigest: run.lastDigest,
              };
    }

    getTaskLatestRunId(taskId: string): string | null {
        return this.tasks.get(taskId)?.latestRunId ?? null;
    }

    hasTask(taskId: string): boolean {
        return this.tasks.has(taskId);
    }

    getTaskUnresolvedRunId(taskId: string): string | null {
        return this.tasks.get(taskId)?.unresolvedRunId ?? null;
    }

    setTaskUnresolvedRunId(taskId: string, runId: string | null): void {
        const task = this.tasks.get(taskId);
        if (task !== undefined) task.unresolvedRunId = runId;
    }

    restore(checkpoint: AgentStoreCheckpoint): void {
        const state = checkpoint.opaque as AgentStoreInternalState;
        this.tasks.clear();
        for (const [taskId, task] of state.tasks) {
            this.tasks.set(taskId, task);
        }
        this.runs.clear();
        for (const [key, run] of state.runs) {
            this.runs.set(key, run);
        }
        this.runOwners.clear();
        for (const [runId, taskId] of state.runOwners) {
            this.runOwners.set(runId, taskId);
        }
        this.taskOrder.splice(0, this.taskOrder.length, ...state.taskOrder);
        this.runOrder.splice(0, this.runOrder.length, ...state.runOrder);
        this.selectedTaskId = state.selectedTaskId;
    }

    // Synchronous reducer: validation, duplicate identity, ordering and
    // reduction only. Digest computation and persistence live in appendEvent
    // and are awaited by the caller.
    apply(event: AgentIncomingEvent, digest?: string): AgentApplyResult {
        return this.applyInternal(event, digest, null);
    }

    // Durable append uses this scoped journal instead of a whole-store
    // checkpoint. The journal records only the task, run, cursor, message,
    // tool, approval and change mutations this event actually touches.
    applyJournaled(
        event: AgentIncomingEvent,
        digest?: string,
    ): AgentJournaledApply {
        const journal = new AgentMutationJournal();
        try {
            const result = this.applyInternal(event, digest, journal);
            return { result, rollback: () => journal.rollback() };
        } catch (error) {
            journal.rollback();
            throw error;
        }
    }

    private applyInternal(
        event: AgentIncomingEvent,
        digest: string | undefined,
        journal: AgentMutationJournal | null,
    ): AgentApplyResult {
        const validation = validateAgentEvent(event);
        if (!validation.ok) {
            return rejected(validation.code, validation.message, false);
        }
        if (event.sequence >= MAX_EVENTS_PER_RUN) {
            return rejected(
                'limit_exceeded',
                'run exceeds max_events_per_run',
                false,
            );
        }
        if (digest !== undefined && !DIGEST_PATTERN.test(digest)) {
            return rejected(
                'invalid_event',
                'digest must be sha256:<64 lowercase hex>',
                false,
            );
        }

        let canonical: string;
        try {
            canonical = canonicalJson(event);
        } catch (error) {
            if (error instanceof CanonicalJsonError) {
                return rejected(error.code, error.message, false);
            }
            return rejected('invalid_event', 'event is not canonical JSON', false);
        }
        if (byteLength(canonical) > MAX_EVENT_BYTES) {
            return rejected(
                'limit_exceeded',
                'event exceeds max_event_bytes',
                false,
            );
        }

        const typed = validation.event;
        const taskId = typed.task_id;
        const runId = typed.run_id;
        const owner = this.runOwners.get(runId);
        if (owner !== undefined && owner !== taskId) {
            return rejected(
                'cross_task_event',
                `run_id ${runId} belongs to another task`,
                false,
            );
        }

        let task = this.tasks.get(taskId);
        if (task !== undefined && task.projectId !== typed.project_id) {
            return rejected(
                'cross_task_event',
                'event project does not match the task project',
                false,
            );
        }

        const key = runKey(taskId, runId);
        const run = this.runs.get(key);
        if (run !== undefined) {
            captureRunState(journal, run);
            const seen = run.seen.get(typed.event_id);
            if (seen !== undefined) {
                const sameDigest =
                    digest !== undefined && seen.digest !== null
                        ? seen.digest === digest
                        : null;
                const same =
                    sameDigest !== null
                        ? sameDigest
                        : seen.canonical === canonical;
                if (same) {
                    return ignored();
                }
                run.needsResync = true;
                run.resyncCode = 'duplicate_event';
                return rejected(
                    'duplicate_event',
                    'same event_id with a different canonical digest',
                    true,
                );
            }

            if (run.needsResync) {
                return rejected(
                    run.resyncCode ?? 'out_of_order',
                    'run requires a verified snapshot resync',
                    true,
                );
            }
            if (typed.generation < run.generation) {
                run.needsResync = true;
                run.resyncCode = 'stale_generation';
                return rejected(
                    'stale_generation',
                    'event generation is older than the run generation',
                    true,
                );
            }
            if (isTerminalState(run.state)) {
                return rejected(
                    'out_of_order',
                    'no event may follow a terminal run event',
                    false,
                );
            }
            if (typed.generation > run.generation) {
                return this.applyResume(run, typed, canonical, digest, journal);
            }
            if (typed.type === 'run.resumed') {
                run.needsResync = true;
                run.resyncCode = 'out_of_order';
                return rejected(
                    'out_of_order',
                    'run.resumed must advance the run generation',
                    true,
                );
            }
            if (typed.sequence !== run.lastSequence + 1) {
                run.needsResync = true;
                run.resyncCode = 'out_of_order';
                return rejected(
                    'out_of_order',
                    typed.sequence < run.lastSequence + 1
                        ? 'event sequence is not the next sequence'
                        : 'event sequence leaves a gap',
                    true,
                );
            }
            if (
                typed.type === 'run.queued' ||
                (typed.type === 'run.started' && run.lastSequence !== 0)
            ) {
                run.needsResync = true;
                run.resyncCode = 'out_of_order';
                return rejected(
                    'out_of_order',
                    `${typed.type} is only valid at the start of a run`,
                    true,
                );
            }
            if (run.lastSequence === 0 && typed.type !== 'run.started') {
                run.needsResync = true;
                run.resyncCode = 'out_of_order';
                return rejected(
                    'out_of_order',
                    'run.started must be sequence 1 after run.queued',
                    true,
                );
            }

            this.recordApplied(run, typed, canonical, digest, journal);
            reduceKnownEvent(run, typed, journal);
            return applied();
        }

        if (typed.type !== 'run.queued' || typed.sequence !== 0) {
            return rejected(
                'out_of_order',
                'a run must begin with run.queued at sequence 0',
                true,
            );
        }
        const maxGeneration = task?.maxGeneration ?? 0;
        if (typed.generation !== maxGeneration + 1) {
            if (typed.generation <= maxGeneration) {
                return rejected(
                    'stale_generation',
                    'new run generation is not greater than prior runs',
                    false,
                );
            }
            return rejected(
                'out_of_order',
                'new run generation skips ahead',
                true,
            );
        }
        if (task?.archived === true) {
            return rejected('task_archived', 'task is archived', false);
        }
        if (task === undefined) {
            if (journal !== null) {
                const orderLength = this.taskOrder.length;
                journal.push(() => {
                    this.tasks.delete(taskId);
                    this.taskOrder.length = orderLength;
                });
            }
            task = {
                taskId,
                projectId: typed.project_id,
                title: taskId,
                repo: null,
                branch: null,
                createdAtMs: typed.recorded_at_ms,
                archived: false,
                pinnedLocal: false,
                latestRunId: null,
                unresolvedRunId: null,
                maxGeneration: 0,
                hold: null,
            };
            this.tasks.set(taskId, task);
            this.taskOrder.push(taskId);
        } else {
            captureTaskState(journal, task);
        }

        const created = createRun(task, typed);
        captureNewRun(
            journal,
            this.runs,
            this.runOrder,
            this.runOwners,
            key,
            runId,
        );
        this.runs.set(key, created);
        this.runOrder.push(key);
        this.runOwners.set(runId, taskId);
        task.maxGeneration = typed.generation;
        task.latestRunId = runId;
        task.unresolvedRunId = null;
        this.recordApplied(created, typed, canonical, digest, journal);
        reduceKnownEvent(created, typed, journal);
        return applied();
    }

    async appendEvent(event: AgentIncomingEvent): Promise<AgentAppendResult> {
        let digest: string | null = null;
        try {
            digest = await digestEvent(event as AgentEvent);
        } catch (error) {
            const code =
                error instanceof CanonicalJsonError
                    ? error.code
                    : ('invalid_event' as const);
            const message =
                error instanceof Error ? error.message : 'event digest failed';
            return {
                ...rejected(code, message, false),
                eventId: event.event_id,
                digest: null,
            };
        }
        const result = this.apply(event, digest);
        return {
            ...result,
            eventId: event.event_id,
            digest,
        };
    }

    verifySnapshot(
        snapshot: AgentSnapshotVerification,
    ): AgentVerifySnapshotResult {
        const key = runKey(snapshot.taskId, snapshot.runId);
        const run = this.runs.get(key);
        if (run === undefined) {
            return {
                ok: false,
                code: 'snapshot_missing',
                message: 'run is not present in the store',
            };
        }
        const fail = (
            code: AgentErrorCode,
            message: string,
        ): AgentVerifySnapshotResult => {
            run.needsResync = true;
            run.resyncCode = code;
            return { ok: false, code, message };
        };

        if (
            !TASK_ID_PATTERN.test(snapshot.taskId) ||
            !RUN_ID_PATTERN.test(snapshot.runId) ||
            !isSafePositiveInteger(snapshot.generation) ||
            !isSafeNonNegativeInteger(snapshot.lastSequence) ||
            typeof snapshot.lastEventId !== 'string' ||
            !DIGEST_PATTERN.test(snapshot.headDigest)
        ) {
            return fail('snapshot_corrupt', 'snapshot fields are not contract-valid');
        }
        if (
            snapshot.taskId !== run.taskId ||
            snapshot.runId !== run.runId
        ) {
            return fail('snapshot_corrupt', 'snapshot identity does not match the run');
        }
        if (run.lastDigest === null) {
            return fail(
                'snapshot_corrupt',
                'run has no verified event digest chain',
            );
        }
        if (
            snapshot.generation !== run.generation ||
            snapshot.lastSequence !== run.lastSequence ||
            snapshot.lastEventId !== run.lastEventId
        ) {
            return fail(
                'snapshot_corrupt',
                'snapshot position does not match the last applied event',
            );
        }
        if (snapshot.headDigest !== run.lastDigest) {
            return fail(
                'snapshot_corrupt',
                'snapshot head digest does not match the event chain',
            );
        }

        run.verifiedSnapshot = { ...snapshot };
        run.needsResync = false;
        run.resyncCode = null;
        return { ok: true, code: null, message: null };
    }

    snapshot(): AgentStoreSnapshot {
        const runs = this.runOrder.map((key) => copyRun(this.runs.get(key)!));
        const runsByTask = new Map<string, RunView[]>();
        for (const run of runs) {
            const existing = runsByTask.get(run.taskId);
            if (existing === undefined) {
                runsByTask.set(run.taskId, [run]);
            } else {
                existing.push(run);
            }
        }
        const tasks = this.taskOrder.map((taskId) => {
            const task = this.tasks.get(taskId)!;
            const taskRuns = runsByTask.get(taskId) ?? [];
            const latestRun =
                task.latestRunId === null
                    ? null
                    : taskRuns.find((run) => run.runId === task.latestRunId) ??
                      null;
            const unresolvedRunId =
                task.latestRunId === null
                    ? task.unresolvedRunId
                    : latestRun === null
                      ? task.unresolvedRunId ?? task.latestRunId
                      : null;
            return {
                taskId: task.taskId,
                projectId: task.projectId,
                title: task.title,
                repo: task.repo,
                branch: task.branch,
                createdAtMs: task.createdAtMs,
                archived: task.archived,
                pinnedLocal: task.pinnedLocal,
                state: taskStateFor(task, latestRun, unresolvedRunId),
                latestRunId: task.latestRunId,
                unresolvedRunId,
                hold: task.hold === null ? null : { ...task.hold },
                runs: taskRuns,
            };
        });
        const selectedTask =
            this.selectedTaskId === null
                ? null
                : tasks.find((task) => task.taskId === this.selectedTaskId) ??
                  null;
        return {
            tasks,
            runs,
            selectedTaskId: this.selectedTaskId,
            selectedTask,
        };
    }

    selectTask(taskId: string): TaskView | null {
        this.selectedTaskId = taskId;
        const snapshot = this.snapshot();
        return (
            snapshot.tasks.find((task) => task.taskId === taskId) ?? null
        );
    }

    clearSelection(): void {
        this.selectedTaskId = null;
    }

    upsertTask(record: AgentTaskRecord): TaskView {
        if (
            !TASK_ID_PATTERN.test(record.taskId) ||
            !PROJECT_ID_PATTERN.test(record.projectId) ||
            !isSafeNonNegativeInteger(record.createdAtMs)
        ) {
            throw new AgentProtocolError(
                'invalid_event',
                'task record identity is not contract-valid',
            );
        }
        let task = this.tasks.get(record.taskId);
        if (task === undefined) {
            task = {
                taskId: record.taskId,
                projectId: record.projectId,
                title: record.title,
                repo: record.repo ?? null,
                branch: record.branch ?? null,
                createdAtMs: record.createdAtMs,
                archived: record.archived ?? false,
                pinnedLocal: record.pinnedLocal ?? false,
                latestRunId: record.latestRunId ?? null,
                unresolvedRunId:
                    record.latestRunId === null || record.latestRunId === undefined
                        ? null
                        : record.latestRunId,
                maxGeneration: 0,
                hold: record.hold ?? null,
            };
            this.tasks.set(record.taskId, task);
            this.taskOrder.push(record.taskId);
        } else {
            if (task.projectId !== record.projectId) {
                throw new AgentProtocolError(
                    'cross_task_event',
                    'task record project does not match the existing task',
                );
            }
            task.title = record.title;
            task.repo = record.repo ?? null;
            task.branch = record.branch ?? null;
            task.archived = record.archived ?? task.archived;
            task.pinnedLocal = record.pinnedLocal ?? task.pinnedLocal;
            if (record.hold !== undefined) {
                task.hold = record.hold;
            }
            if (record.latestRunId === null) {
                if (
                    task.latestRunId !== null &&
                    !this.runs.has(runKey(task.taskId, task.latestRunId))
                ) {
                    task.unresolvedRunId = task.latestRunId;
                }
            } else if (record.latestRunId !== undefined) {
                const exists = this.runs.has(
                    runKey(task.taskId, record.latestRunId),
                );
                task.unresolvedRunId = exists ? null : record.latestRunId;
                if (!this.hasRunsForTask(task.taskId)) {
                    task.latestRunId = record.latestRunId;
                }
            }
        }
        const snapshot = this.snapshot();
        return snapshot.tasks.find(
            (candidate) => candidate.taskId === record.taskId,
        )!;
    }

    private hasRunsForTask(taskId: string): boolean {
        for (const run of this.runs.values()) {
            if (run.taskId === taskId) {
                return true;
            }
        }
        return false;
    }

    latestChangesSnapshot(
        taskId: string,
        runId?: string,
        scope?: 'last_turn' | 'branch',
    ): AgentChangeSnapshotView | null {
        for (let index = this.runOrder.length - 1; index >= 0; index -= 1) {
            const run = this.runs.get(this.runOrder[index]!);
            if (
                run === undefined ||
                run.taskId !== taskId ||
                (runId !== undefined && run.runId !== runId)
            ) {
                continue;
            }
            for (
                let snapshotIndex = run.changesSnapshots.length - 1;
                snapshotIndex >= 0;
                snapshotIndex -= 1
            ) {
                const snapshot = run.changesSnapshots[snapshotIndex]!;
                if (scope !== undefined && snapshot.scope !== scope) {
                    continue;
                }
                return {
                    ...snapshot,
                    files: cloneJson(snapshot.files),
                };
            }
        }
        return null;
    }

    private recordApplied(
        run: InternalRun,
        event: AgentIncomingEvent,
        canonical: string,
        digest: string | undefined,
        journal: AgentMutationJournal | null,
    ): void {
        captureRunState(journal, run);
        captureMapValue(journal, run.seen, event.event_id);
        captureArrayLength(journal, run.events);
        captureArrayLength(journal, run.unknownEvents);
        run.lastSequence = event.sequence;
        run.lastEventId = event.event_id;
        run.lastDigest = digest ?? null;
        run.seen.set(event.event_id, {
            canonical,
            digest: digest ?? null,
        });
        run.events.push({
            eventId: event.event_id,
            type: event.type,
            generation: event.generation,
            sequence: event.sequence,
            recordedAtMs: event.recorded_at_ms,
            data: cloneJson(event.data as Record<string, unknown>),
        });
        if (!KNOWN_EVENT_TYPES.has(event.type)) {
            run.unknownEvents.push({
                eventId: event.event_id,
                type: event.type,
                data: cloneJson(event.data as Record<string, unknown>),
                generation: event.generation,
                sequence: event.sequence,
                recordedAtMs: event.recorded_at_ms,
            });
        }
    }

    private applyResume(
        run: InternalRun,
        event: AgentIncomingEvent,
        canonical: string,
        digest: string | undefined,
        journal: AgentMutationJournal | null,
    ): AgentApplyResult {
        if (event.type !== 'run.resumed') {
            run.needsResync = true;
            run.resyncCode = 'out_of_order';
            return rejected(
                'out_of_order',
                'a generation jump requires run.resumed, not a fresh run',
                true,
            );
        }
        const data = event.data as {
            previous_generation: number;
            snapshot_digest: string;
        };
        if (
            event.generation !== run.generation + 1 ||
            event.sequence !== run.lastSequence + 1
        ) {
            run.needsResync = true;
            run.resyncCode = 'out_of_order';
            return rejected(
                'out_of_order',
                'run.resumed must use generation G+1 and sequence N+1',
                true,
            );
        }
        if (data.previous_generation !== run.generation) {
            run.needsResync = true;
            run.resyncCode = 'stale_generation';
            return rejected(
                'stale_generation',
                'run.resumed previous_generation does not match the run generation',
                true,
            );
        }
        const snapshot = run.verifiedSnapshot;
        if (snapshot === null) {
            run.needsResync = true;
            run.resyncCode = 'snapshot_missing';
            return rejected(
                'snapshot_missing',
                'run.resumed requires a verified snapshot',
                true,
            );
        }
        if (
            snapshot.generation !== run.generation ||
            snapshot.lastSequence !== run.lastSequence ||
            snapshot.lastEventId !== run.lastEventId ||
            snapshot.headDigest !== data.snapshot_digest ||
            (run.lastDigest !== null && snapshot.headDigest !== run.lastDigest)
        ) {
            run.needsResync = true;
            run.resyncCode = 'snapshot_corrupt';
            return rejected(
                'snapshot_corrupt',
                'verified snapshot does not match the run chain or resume digest',
                true,
            );
        }

        const previousGeneration = run.generation;
        run.generation = event.generation;
        const task = this.tasks.get(run.taskId);
        if (task !== undefined) {
            captureTaskState(journal, task);
            task.maxGeneration = Math.max(
                task.maxGeneration,
                event.generation,
            );
            task.unresolvedRunId = null;
        }
        run.state = 'running';
        run.verifiedSnapshot = null;
        run.needsResync = false;
        run.resyncCode = null;
        captureArrayLength(journal, run.generationTransitions);
        run.generationTransitions.push({
            from: previousGeneration,
            to: event.generation,
            atSequence: event.sequence,
            previousGeneration: data.previous_generation,
            snapshotDigest: data.snapshot_digest,
        });
        this.recordApplied(run, event, canonical, digest, journal);
        return applied();
    }
}

export function createAgentStore(): AgentStore {
    return new AgentStore();
}
