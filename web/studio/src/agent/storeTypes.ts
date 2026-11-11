import {
    AGENT_ERROR_CODES,
    type AgentErrorCode,
    type RunState,
} from './generated/agentContract.js';
import type { AgentIncomingEvent } from '../state/agent.js';

export const AGENT_DB_NAME = 'uor-r4.studio.agent';
export const AGENT_DB_VERSION = 1;
export const SNAPSHOT_SCHEMA = 'uor-r4.studio.agent-snapshot/1';
export const SNAPSHOT_BUNDLE_SCHEMA = 'uor-r4.studio.agent-snapshot-bundle/1';
export const EXPORT_SCHEMA = 'uor-r4.studio.agent-export/1';
export const LEASE_STALE_MS = 30_000;
export const MAX_SNAPSHOT_BYTES = 8 * 1024 * 1024;
export const MAX_EXPORT_BYTES = 64 * 1024 * 1024;

export const AGENT_DB_STORES = [
    'tasks',
    'runs',
    'events',
    'snapshots',
    'artifacts',
    'approvals',
    'changes',
    'comments',
    'leases',
] as const;

export type AgentDbStoreName = (typeof AGENT_DB_STORES)[number];

export interface TaskRecord {
    task_id: string;
    project_id: string;
    title: string;
    repo?: string;
    branch?: string;
    created_at_ms: number;
    archived: boolean;
    pinned_local: boolean;
    latest_run_id?: string | null;
    hold?: {
        reason: 'waiting_for_background';
        run_id: string;
        tool_call_id: string;
    } | null;
}

export interface RunRecord {
    task_id: string;
    run_id: string;
    generation: number;
    state: RunState;
    retry_of_run_id: string | null;
    started_at_ms: number;
    ended_at_ms?: number;
    last_sequence: number;
    last_event_id: string | null;
    last_digest: string | null;
}

export interface EventRecord {
    event_id: string;
    canonical: string;
    digest: string;
    prev_digest: string | null;
    recorded_at_ms: number;
}

export interface SnapshotRecord {
    schema: string;
    task_id: string;
    run_id: string;
    generation: number;
    last_event_id: string;
    last_sequence: number;
    head_digest: string;
    created_at_ms: number;
}

export interface ChangeRecord {
    task_id: string;
    run_id: string;
    scope: 'last_turn';
    files: Array<Record<string, unknown>>;
    head_digest: string;
    captured_at_ms: number;
}

export interface SnapshotBundleRecord {
    schema: typeof SNAPSHOT_BUNDLE_SCHEMA;
    task_id: string;
    run_id: string;
    latest: SnapshotRecord;
    anchors: SnapshotRecord[];
}

export type StoredSnapshotRecord = SnapshotRecord | SnapshotBundleRecord;

export interface SnapshotAnchorInput {
    task_id: string;
    run_id: string;
    generation: number;
    last_sequence: number;
    last_event_id: string;
    head_digest: string;
    created_at_ms?: number;
}

export interface LeaseRecord {
    task_id: string;
    writer_id: string;
    started_at_ms: number;
    heartbeat_at_ms: number;
}

export interface RestoreIncomplete {
    task_id: string;
    run_id: string | null;
    code: AgentErrorCode;
    message: string;
}

export interface OpenReport {
    tasks: TaskRecord[];
    runs: RunRecord[];
    incomplete: RestoreIncomplete[];
}

export interface AppendEventStoreInput {
    event: AgentIncomingEvent;
    digest?: string;
    writer_id?: string;
    now_ms?: number;
    snapshot?: boolean;
    snapshot_anchor?: SnapshotAnchorInput;
}

export interface AppendStoreResult {
    status: 'applied' | 'ignored' | 'rejected';
    code: AgentErrorCode | null;
    message: string | null;
    event_id: string;
    digest: string | null;
    storage: 'indexeddb' | null;
}

export type ReadResult<T> =
    | { ok: true; record: T }
    | { ok: false; code: AgentErrorCode; message: string };

export type ReadRunResult =
    | { ok: true; run: RunRecord; events: EventRecord[] }
    | { ok: false; code: AgentErrorCode; message: string };

export interface ChangeInput {
    task_id: string;
    run_id: string;
    head_digest: string;
    files: Array<Record<string, unknown>>;
    captured_at_ms: number;
    scope?: 'last_turn';
}

export interface ListTaskOptions {
    includeArchived?: boolean;
    query?: string;
}

export interface CreateTaskStoreInput {
    project_id?: string;
    title: string;
    repo?: string | null;
    branch?: string | null;
    task_id?: string;
    created_at_ms?: number;
}

export interface ImportSummary {
    tasks: number;
    runs: number;
    events: number;
    snapshots: number;
    changes: number;
}

export class AgentStoreError extends Error {
    readonly code: AgentErrorCode;
    readonly storage: 'indexeddb' | null;

    constructor(
        code: AgentErrorCode,
        message: string,
        storage: 'indexeddb' | null = null,
    ) {
        super(`${code}: ${message}`);
        this.name = 'AgentStoreError';
        this.code = code;
        this.storage = storage;
    }
}

export class AgentClientError extends Error {
    readonly code: AgentErrorCode;
    readonly retryable: boolean;
    readonly storage: 'indexeddb' | null;

    constructor(
        code: AgentErrorCode,
        message: string,
        retryable = false,
        storage: 'indexeddb' | null = null,
    ) {
        super(`${code}: ${message}`);
        this.name = 'AgentClientError';
        this.code = code;
        this.retryable = retryable;
        this.storage = storage;
    }
}

export function isPlainObject(
    value: unknown,
): value is Record<string, unknown> {
    if (typeof value !== 'object' || value === null || Array.isArray(value)) {
        return false;
    }
    const prototype = Object.getPrototypeOf(value);
    return prototype === Object.prototype || prototype === null;
}

export function isAgentErrorCode(value: unknown): value is AgentErrorCode {
    return (
        typeof value === 'string' &&
        (AGENT_ERROR_CODES as readonly string[]).includes(value)
    );
}

export function toClientError(error: unknown): AgentClientError {
    if (error instanceof AgentClientError) return error;
    if (error instanceof AgentStoreError) {
        const prefix = `${error.code}: `;
        return new AgentClientError(
            error.code,
            error.message.startsWith(prefix)
                ? error.message.slice(prefix.length)
                : error.message,
            false,
            error.storage,
        );
    }
    return new AgentClientError(
        'internal',
        error instanceof Error ? error.message : String(error),
    );
}
