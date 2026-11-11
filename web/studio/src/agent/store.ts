import {
    AGENT_EVENT_SCHEMA,
    AGENT_RUN_STATES,
    MAX_EVENT_BYTES,
    MAX_EVENTS_PER_RUN,
    type AgentErrorCode,
    type AgentEvent,
    type RunState,
} from './generated/agentContract.js';
import {
    CanonicalJsonError,
    canonicalJson,
    digestEvent,
    parseCanonicalJson,
} from './generated/agentCanonical.js';
import {
    AgentStore,
    createAgentStore,
    validateAgentEvent,
    type AgentIncomingEvent,
    type AgentTaskRecord,
    type RunView,
} from '../state/agent.js';
import {
    AGENT_DB_NAME,
    AGENT_DB_STORES,
    AGENT_DB_VERSION,
    AgentStoreError,
    EXPORT_SCHEMA,
    LEASE_STALE_MS,
    MAX_EXPORT_BYTES,
    MAX_SNAPSHOT_BYTES,
    SNAPSHOT_BUNDLE_SCHEMA,
    SNAPSHOT_SCHEMA,
    type AgentDbStoreName,
    type AppendEventStoreInput,
    type AppendStoreResult,
    type ChangeInput,
    type ChangeRecord,
    type CreateTaskStoreInput,
    type EventRecord,
    type ImportSummary,
    type LeaseRecord,
    type ListTaskOptions,
    type OpenReport,
    type ReadResult,
    type ReadRunResult,
    type RestoreIncomplete,
    type RunRecord,
    type SnapshotBundleRecord,
    type SnapshotRecord,
    type StoredSnapshotRecord,
    type TaskRecord,
} from './storeTypes.js';

const PROJECT_ID_PATTERN = /^prj_[0-9a-f]{16}$/;
const TASK_ID_PATTERN = /^tsk_[0-9a-f]{16}$/;
const RUN_ID_PATTERN = /^run_[0-9a-f]{16}$/;
const ARTIFACT_ID_PATTERN = /^art_[0-9a-f]{64}$/;
const DIGEST_PATTERN = /^sha256:[0-9a-f]{64}$/;
const RUN_STATES: ReadonlySet<string> = new Set(AGENT_RUN_STATES);
const TERMINAL_RUN_STATES: ReadonlySet<string> = new Set([
    'finished',
    'failed',
    'cancelled',
    'expired',
]);
const IMPORT_FENCE_KEY = '__import__';
const IMPORT_HEARTBEAT_MS = Math.max(1, Math.floor(LEASE_STALE_MS / 3));
const MAX_DURABLE_ARTIFACT_BYTES = 8 * 1024 * 1024;
const MAX_MEDIA_TYPE_LENGTH = 1024;
const MAX_MEDIA_PARAMETERS = 16;
const MEDIA_TOKEN_PATTERN = /^[!#$%&'*+\-.^_`|~0-9A-Za-z]+$/;
const MAX_DURABLE_ARTIFACT_RETURN_BYTES = 1024 * 1024;
const MAX_DIFF_COMMENT_BODY = 8192;
const MAX_DIFF_COMMENT_PATH = 1024;
const MAX_DIFF_COMMENT_DIGEST = 256;

export interface AppendDiffCommentInput {
    task_id: string;
    path: string;
    line: number;
    body: string;
    head_digest: string;
    scope: 'last_turn' | 'branch';
    comment_id?: string;
    created_at_ms?: number;
}

export interface AppendedDiffComment {
    comment: DiffCommentRecord;
    run_id: string;
    sequence: number;
    event_id: string;
    digest: string;
    prev_digest: string | null;
}

export interface DiffCommentRecord {
    task_id: string;
    comment_id: string;
    path: string;
    line: number;
    body: string;
    head_digest: string;
    scope: 'last_turn' | 'branch';
    created_at_ms: number;
}

interface StoreFaults {
    abort_next_append: boolean;
    abort_next_task_write: boolean;
    fail_import_publish: boolean;
    fail_import_cleanup: boolean;
}

/**
 * Durable browser artifact record, keyed by `artifact_id` in the existing
 * `artifacts` object store. The blob is optional in the declared schema; a
 * record without bytes cannot serve a read and is refused as corrupt.
 */
export interface ArtifactRecord {
    artifact_id: string;
    media_type: string;
    size_bytes: number;
    digest: string;
    blob?: Blob;
}

/** Inclusive-start/exclusive-end durable artifact range. */
export interface DurableArtifactRange {
    start: number;
    end: number;
}

/** Validated durable artifact bytes returned by `AgentTaskStore.readArtifact`. */
export interface DurableArtifact {
    artifact_id: string;
    media_type: string;
    /** Number of bytes returned, not necessarily the whole artifact size. */
    size_bytes: number;
    artifact_size_bytes: number;
    /** SHA-256 of the returned bytes. */
    digest: string;
    /** SHA-256 of the whole artifact recorded at registration. */
    whole_digest: string;
    range: DurableArtifactRange;
    bytes: Uint8Array;
}

export interface AgentTaskStoreTimers {
    setInterval(
        handler: () => void | Promise<void>,
        timeout: number,
    ): unknown;
    clearInterval(handle: unknown): void;
}

export interface AgentTaskStoreOptions {
    store: AgentStore;
    indexedDB?: IDBFactory;
    now?: () => number;
    writerId?: string;
    faults?: StoreFaults;
    timers?: AgentTaskStoreTimers;
}

interface EventPair {
    record: EventRecord;
    key: IDBValidKey;
}

interface ReplayInput {
    tasks: TaskRecord[];
    runs: RunRecord[];
    snapshots: Map<string, SnapshotBundleRecord>;
    events: Map<string, EventPair[]>;
    orphanRuns: Array<{ taskId: string; runId: string }>;
    strict: boolean;
}

interface ReplayResult {
    report: OpenReport;
    replayed: RunRecord[];
    comments: DiffCommentRecord[];
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

function isNonNegativeInteger(value: unknown): value is number {
    return typeof value === 'number' && Number.isSafeInteger(value) && value >= 0;
}

function isPositiveInteger(value: unknown): value is number {
    return typeof value === 'number' && Number.isSafeInteger(value) && value >= 1;
}

function isNonEmptyString(value: unknown): value is string {
    return typeof value === 'string' && value.length > 0;
}

function corrupt(message: string): never {
    throw new AgentStoreError('snapshot_corrupt', message);
}

function invalidArgument(message: string): never {
    throw new AgentStoreError('invalid_arguments', message);
}

function runKey(taskId: string, runId: string): string {
    return `${taskId}\u0000${runId}`;
}

function jsonClean<T>(value: T): T {
    return JSON.parse(JSON.stringify(value)) as T;
}

function errorMessage(error: unknown): string {
    if (error instanceof AgentStoreError) {
        const prefix = `${error.code}: `;
        return error.message.startsWith(prefix)
            ? error.message.slice(prefix.length)
            : error.message;
    }
    return error instanceof Error ? error.message : String(error);
}

function storageError(error: unknown): AgentStoreError {
    if (error instanceof AgentStoreError) {
        return error;
    }
    return new AgentStoreError(
        'limit_exceeded',
        error instanceof Error ? error.message : 'indexeddb transaction failed',
        'indexeddb',
    );
}

function randomHex(bytes: number): string {
    const values = new Uint8Array(bytes);
    const crypto = globalThis.crypto;
    if (crypto === undefined || typeof crypto.getRandomValues !== 'function') {
        throw new AgentStoreError('runtime_unavailable', 'secure random ids are unavailable');
    }
    crypto.getRandomValues(values);
    return [...values]
        .map((value) => value.toString(16).padStart(2, '0'))
        .join('');
}

async function sha256Bytes(bytes: Uint8Array): Promise<string> {
    const subtle = globalThis.crypto?.subtle;
    if (subtle === undefined) {
        throw new AgentStoreError(
            'runtime_unavailable',
            'Web Crypto SHA-256 is not available',
        );
    }
    const digest = new Uint8Array(
        await subtle.digest('SHA-256', bytes as unknown as BufferSource),
    );
    let hex = '';
    for (const byte of digest) hex += byte.toString(16).padStart(2, '0');
    return `sha256:${hex}`;
}

function requestResult<T>(request: IDBRequest<T>): Promise<T> {
    return new Promise<T>((resolve, reject) => {
        request.onsuccess = () => resolve(request.result);
        request.onerror = () =>
            reject(
                new AgentStoreError(
                    'limit_exceeded',
                    request.error?.name ?? 'indexeddb request failed',
                    'indexeddb',
                ),
            );
    });
}

function transactionDone(transaction: IDBTransaction): Promise<void> {
    return new Promise<void>((resolve, reject) => {
        let settled = false;
        const fail = () => {
            if (settled) return;
            settled = true;
            reject(
                new AgentStoreError(
                    'limit_exceeded',
                    transaction.error?.name ?? 'indexeddb transaction aborted',
                    'indexeddb',
                ),
            );
        };
        transaction.oncomplete = () => {
            if (settled) return;
            settled = true;
            resolve();
        };
        transaction.onabort = fail;
        transaction.onerror = fail;
    });
}

function openAgentDatabase(factory: IDBFactory): Promise<IDBDatabase> {
    return new Promise<IDBDatabase>((resolve, reject) => {
        const request = factory.open(AGENT_DB_NAME, AGENT_DB_VERSION);
        request.onupgradeneeded = () => {
            const db = request.result;
            const transaction = request.transaction;
            if (transaction === null) {
                reject(new AgentStoreError('internal', 'missing versionchange transaction'));
                return;
            }
            const ensure = (name: AgentDbStoreName): IDBObjectStore =>
                db.objectStoreNames.contains(name)
                    ? transaction.objectStore(name)
                    : db.createObjectStore(name);
            for (const name of AGENT_DB_STORES) {
                ensure(name);
            }
            const events = transaction.objectStore('events');
            if (!events.indexNames.contains('by_event_id')) {
                events.createIndex('by_event_id', 'event_id', { unique: true });
            }
        };
        request.onsuccess = () => {
            const db = request.result;
            db.onversionchange = () => db.close();
            resolve(db);
        };
        request.onerror = () =>
            reject(
                new AgentStoreError(
                    'runtime_unavailable',
                    request.error?.name ?? 'could not open the agent database',
                ),
            );
        request.onblocked = () =>
            reject(
                new AgentStoreError(
                    'runtime_unavailable',
                    'agent database upgrade is blocked by another connection',
                ),
            );
    });
}

function deleteAgentDatabase(factory: IDBFactory): Promise<void> {
    return new Promise<void>((resolve, reject) => {
        const request = factory.deleteDatabase(AGENT_DB_NAME);
        request.onsuccess = () => resolve();
        request.onerror = () =>
            reject(
                new AgentStoreError(
                    'runtime_unavailable',
                    request.error?.name ?? 'could not delete the agent database',
                ),
            );
        request.onblocked = () =>
            reject(
                new AgentStoreError(
                    'runtime_unavailable',
                    'agent database deletion is blocked by another connection',
                ),
            );
    });
}

function requireStoredTask(value: unknown): asserts value is TaskRecord {
    if (!isObject(value)) corrupt('task record is not an object');
    if (typeof value.task_id !== 'string' || !TASK_ID_PATTERN.test(value.task_id)) {
        corrupt('task record task_id is invalid');
    }
    if (typeof value.project_id !== 'string' || !PROJECT_ID_PATTERN.test(value.project_id)) {
        corrupt('task record project_id is invalid');
    }
    if (typeof value.title !== 'string' || !isNonNegativeInteger(value.created_at_ms)) {
        corrupt('task record title or created_at_ms is invalid');
    }
    if (typeof value.archived !== 'boolean' || typeof value.pinned_local !== 'boolean') {
        corrupt('task record flags are invalid');
    }
    if (
        value.latest_run_id !== undefined &&
        value.latest_run_id !== null &&
        (typeof value.latest_run_id !== 'string' || !RUN_ID_PATTERN.test(value.latest_run_id))
    ) {
        corrupt('task record latest_run_id is invalid');
    }
    if (value.repo !== undefined && typeof value.repo !== 'string') {
        corrupt('task record repo is invalid');
    }
    if (value.branch !== undefined && typeof value.branch !== 'string') {
        corrupt('task record branch is invalid');
    }
    if (value.hold !== undefined && value.hold !== null) {
        if (
            !isObject(value.hold) ||
            value.hold.reason !== 'waiting_for_background' ||
            typeof value.hold.run_id !== 'string' ||
            !RUN_ID_PATTERN.test(value.hold.run_id) ||
            typeof value.hold.tool_call_id !== 'string'
        ) {
            corrupt('task record hold is invalid');
        }
    }
}

function requireStoredRun(value: unknown): asserts value is RunRecord {
    if (!isObject(value)) corrupt('run record is not an object');
    if (typeof value.task_id !== 'string' || !TASK_ID_PATTERN.test(value.task_id)) {
        corrupt('run record task_id is invalid');
    }
    if (typeof value.run_id !== 'string' || !RUN_ID_PATTERN.test(value.run_id)) {
        corrupt('run record run_id is invalid');
    }
    if (!isPositiveInteger(value.generation)) {
        corrupt('run record generation is invalid');
    }
    if (typeof value.state !== 'string' || !RUN_STATES.has(value.state)) {
        corrupt('run record state is invalid');
    }
    if (
        value.retry_of_run_id !== null &&
        (typeof value.retry_of_run_id !== 'string' || !RUN_ID_PATTERN.test(value.retry_of_run_id))
    ) {
        corrupt('run record retry_of_run_id is invalid');
    }
    if (!isNonNegativeInteger(value.started_at_ms)) {
        corrupt('run record started_at_ms is invalid');
    }
    if (value.ended_at_ms !== undefined && !isNonNegativeInteger(value.ended_at_ms)) {
        corrupt('run record ended_at_ms is invalid');
    }
    if (!isNonNegativeInteger(value.last_sequence)) {
        corrupt('run record last_sequence is invalid');
    }
    if (
        value.last_event_id !== null &&
        typeof value.last_event_id !== 'string'
    ) {
        corrupt('run record last_event_id is invalid');
    }
    if (
        value.last_digest !== null &&
        (typeof value.last_digest !== 'string' || !DIGEST_PATTERN.test(value.last_digest))
    ) {
        corrupt('run record last_digest is invalid');
    }
}

function requireStoredEvent(value: unknown): asserts value is EventRecord {
    if (!isObject(value)) corrupt('event record is not an object');
    if (typeof value.event_id !== 'string' || value.event_id.length === 0) {
        corrupt('event record event_id is invalid');
    }
    if (
        typeof value.canonical !== 'string' ||
        byteLength(value.canonical) > MAX_EVENT_BYTES
    ) {
        corrupt('event record canonical text is invalid');
    }
    if (typeof value.digest !== 'string' || !DIGEST_PATTERN.test(value.digest)) {
        corrupt('event record digest is invalid');
    }
    if (
        value.prev_digest !== null &&
        (typeof value.prev_digest !== 'string' || !DIGEST_PATTERN.test(value.prev_digest))
    ) {
        corrupt('event record prev_digest is invalid');
    }
    if (!isNonNegativeInteger(value.recorded_at_ms)) {
        corrupt('event record recorded_at_ms is invalid');
    }
}

function diffCommentKey(record: DiffCommentRecord): [string, string, number, string] {
    return [record.task_id, record.path, record.line, record.head_digest];
}

function requireStoredDiffComment(value: unknown): asserts value is DiffCommentRecord {
    if (!isObject(value)) corrupt('diff comment record is not an object');
    if (typeof value.task_id !== 'string' || !TASK_ID_PATTERN.test(value.task_id)) {
        corrupt('diff comment task_id is invalid');
    }
    if (typeof value.comment_id !== 'string' || value.comment_id.length === 0 || value.comment_id.length > 128) {
        corrupt('diff comment comment_id is invalid');
    }
    if (typeof value.path !== 'string' || value.path.length === 0 || value.path.length > MAX_DIFF_COMMENT_PATH) {
        corrupt('diff comment path is invalid');
    }
    if (!isNonNegativeInteger(value.line)) {
        corrupt('diff comment line is invalid');
    }
    if (typeof value.body !== 'string' || value.body.length > MAX_DIFF_COMMENT_BODY) {
        corrupt('diff comment body is invalid');
    }
    if (typeof value.head_digest !== 'string' || value.head_digest.length === 0 || value.head_digest.length > MAX_DIFF_COMMENT_DIGEST) {
        corrupt('diff comment head_digest is invalid');
    }
    if (value.scope !== 'last_turn' && value.scope !== 'branch') {
        corrupt('diff comment scope is invalid');
    }
    if (!isNonNegativeInteger(value.created_at_ms)) {
        corrupt('diff comment created_at_ms is invalid');
    }
}

function diffCommentRecordFromEvent(
    event: AgentIncomingEvent,
    lastTurnDigests: ReadonlySet<string>,
): DiffCommentRecord {
    const data = event.data as Record<string, unknown>;
    if (
        typeof data.comment_id !== 'string' ||
        data.comment_id.length === 0 ||
        data.comment_id.length > 128 ||
        typeof data.path !== 'string' ||
        data.path.length === 0 ||
        data.path.length > MAX_DIFF_COMMENT_PATH ||
        !isNonNegativeInteger(data.line) ||
        typeof data.body !== 'string' ||
        data.body.length > MAX_DIFF_COMMENT_BODY ||
        typeof data.head_digest !== 'string' ||
        data.head_digest.length === 0 ||
        data.head_digest.length > MAX_DIFF_COMMENT_DIGEST
    ) {
        corrupt('diff.comment event payload is malformed');
    }
    return {
        task_id: event.task_id,
        comment_id: data.comment_id,
        path: data.path,
        line: data.line,
        body: data.body,
        head_digest: data.head_digest,
        scope: lastTurnDigests.has(data.head_digest) ? 'last_turn' : 'branch',
        created_at_ms: event.recorded_at_ms,
    };
}

function lastTurnDigestsOf(events: AgentIncomingEvent[]): Set<string> {
    const digests = new Set<string>();
    for (const event of events) {
        if (event.type !== 'changes.snapshot') continue;
        const data = event.data as Record<string, unknown>;
        if (
            data.scope === 'last_turn' &&
            typeof data.head_digest === 'string'
        ) {
            digests.add(data.head_digest);
        }
    }
    return digests;
}

function requireStoredSnapshot(value: unknown): asserts value is SnapshotRecord {
    if (!isObject(value)) corrupt('snapshot record is not an object');
    if (value.schema !== SNAPSHOT_SCHEMA) {
        corrupt('snapshot schema is not supported');
    }
    if (typeof value.task_id !== 'string' || !TASK_ID_PATTERN.test(value.task_id)) {
        corrupt('snapshot task_id is invalid');
    }
    if (typeof value.run_id !== 'string' || !RUN_ID_PATTERN.test(value.run_id)) {
        corrupt('snapshot run_id is invalid');
    }
    if (!isPositiveInteger(value.generation)) {
        corrupt('snapshot generation is invalid');
    }
    if (typeof value.last_event_id !== 'string' || value.last_event_id.length === 0) {
        corrupt('snapshot last_event_id is invalid');
    }
    if (!isNonNegativeInteger(value.last_sequence)) {
        corrupt('snapshot last_sequence is invalid');
    }
    if (typeof value.head_digest !== 'string' || !DIGEST_PATTERN.test(value.head_digest)) {
        corrupt('snapshot head_digest is invalid');
    }
    if (!isNonNegativeInteger(value.created_at_ms)) {
        corrupt('snapshot created_at_ms is invalid');
    }
}

function sameSnapshot(left: SnapshotRecord, right: SnapshotRecord): boolean {
    return (
        left.task_id === right.task_id &&
        left.run_id === right.run_id &&
        left.generation === right.generation &&
        left.last_sequence === right.last_sequence &&
        left.last_event_id === right.last_event_id &&
        left.head_digest === right.head_digest
    );
}

function snapshotOrder(left: SnapshotRecord, right: SnapshotRecord): number {
    return (
        left.generation - right.generation ||
        left.last_sequence - right.last_sequence ||
        left.created_at_ms - right.created_at_ms
    );
}

function normalizeSnapshotBundle(value: unknown): SnapshotBundleRecord {
    if (!isObject(value)) {
        corrupt('snapshot record is not an object');
    }
    if (value.schema === SNAPSHOT_SCHEMA) {
        requireStoredSnapshot(value);
        return {
            schema: SNAPSHOT_BUNDLE_SCHEMA,
            task_id: value.task_id,
            run_id: value.run_id,
            latest: value,
            anchors: [],
        };
    }
    if (value.schema !== SNAPSHOT_BUNDLE_SCHEMA) {
        corrupt('snapshot record schema is not supported');
    }
    if (typeof value.task_id !== 'string' || !TASK_ID_PATTERN.test(value.task_id)) {
        corrupt('snapshot bundle task_id is invalid');
    }
    if (typeof value.run_id !== 'string' || !RUN_ID_PATTERN.test(value.run_id)) {
        corrupt('snapshot bundle run_id is invalid');
    }
    requireStoredSnapshot(value.latest);
    if (!Array.isArray(value.anchors)) {
        corrupt('snapshot bundle anchors are invalid');
    }
    const anchors = value.anchors.map((anchor) => {
        requireStoredSnapshot(anchor);
        return anchor;
    });
    const allRecords = [value.latest, ...anchors];
    for (let left = 0; left < allRecords.length; left += 1) {
        for (let right = left + 1; right < allRecords.length; right += 1) {
            if (sameSnapshot(allRecords[left]!, allRecords[right]!)) {
                corrupt('snapshot bundle repeats an anchor');
            }
        }
    }
    for (const record of allRecords) {
        if (record.task_id !== value.task_id || record.run_id !== value.run_id) {
            corrupt('snapshot bundle record identity does not match its key');
        }
    }
    return {
        schema: SNAPSHOT_BUNDLE_SCHEMA,
        task_id: value.task_id,
        run_id: value.run_id,
        latest: value.latest,
        anchors,
    };
}

function snapshotRecordsOf(bundle: SnapshotBundleRecord): SnapshotRecord[] {
    const records: SnapshotRecord[] = [bundle.latest];
    for (const anchor of bundle.anchors) {
        if (!records.some((record) => sameSnapshot(record, anchor))) {
            records.push(anchor);
        }
    }
    return records;
}

function bundleFromRecords(
    taskId: string,
    runId: string,
    records: SnapshotRecord[],
): SnapshotBundleRecord {
    const unique: SnapshotRecord[] = [];
    for (const record of records) {
        if (record.task_id !== taskId || record.run_id !== runId) {
            corrupt('snapshot record identity does not match its stored run');
        }
        if (!unique.some((candidate) => sameSnapshot(candidate, record))) {
            unique.push(record);
        }
    }
    unique.sort(snapshotOrder);
    if (unique.length === 0) {
        corrupt('snapshot bundle has no records');
    }
    const latest = unique[unique.length - 1]!;
    return {
        schema: SNAPSHOT_BUNDLE_SCHEMA,
        task_id: taskId,
        run_id: runId,
        latest,
        anchors: unique.filter((record) => !sameSnapshot(record, latest)),
    };
}

function mergeSnapshotAnchor(
    existing: StoredSnapshotRecord | undefined,
    anchor: SnapshotRecord,
): SnapshotBundleRecord {
    const bundle =
        existing === undefined ? null : normalizeSnapshotBundle(existing);
    if (
        bundle !== null &&
        (bundle.task_id !== anchor.task_id || bundle.run_id !== anchor.run_id)
    ) {
        corrupt('snapshot anchor identity does not match the stored run');
    }
    const records = bundle === null ? [] : snapshotRecordsOf(bundle);
    if (!records.some((record) => sameSnapshot(record, anchor))) {
        records.push(anchor);
    }
    return bundleFromRecords(anchor.task_id, anchor.run_id, records);
}

function requireStoredChange(value: unknown): asserts value is ChangeRecord {
    if (!isObject(value)) corrupt('change record is not an object');
    if (typeof value.task_id !== 'string' || !TASK_ID_PATTERN.test(value.task_id)) {
        corrupt('change record task_id is invalid');
    }
    if (typeof value.run_id !== 'string' || !RUN_ID_PATTERN.test(value.run_id)) {
        corrupt('change record run_id is invalid');
    }
    if (value.scope !== 'last_turn') {
        corrupt('change record scope is invalid');
    }
    if (!Array.isArray(value.files) || !value.files.every((file) => isObject(file))) {
        corrupt('change record files are invalid');
    }
    if (!isNonEmptyString(value.head_digest)) {
        corrupt('change record head_digest is invalid');
    }
    if (!isNonNegativeInteger(value.captured_at_ms)) {
        corrupt('change record captured_at_ms is invalid');
    }
}

function mediaTypeEssence(value: string): string {
    return value.split(';', 1)[0]!.trim().toLowerCase();
}

function parseQuotedMediaParameter(value: string): string {
    let parsed = '';
    for (let index = 1; index < value.length; index += 1) {
        const char = value[index]!;
        if (char === '"') {
            if (index !== value.length - 1) {
                corrupt('artifact record media parameter quote is invalid');
            }
            return parsed;
        }
        if (char === '\\') {
            const escaped = value[index + 1];
            if (escaped === undefined) {
                corrupt('artifact record media parameter escape is invalid');
            }
            parsed += escaped;
            index += 1;
            continue;
        }
        parsed += char;
    }
    corrupt('artifact record media parameter quote is invalid');
}

function parseMediaTypeParameters(value: string): Map<string, string> {
    if (value.length > MAX_MEDIA_TYPE_LENGTH) {
        corrupt('artifact record media type is unbounded');
    }
    const parts: string[] = [];
    let current = '';
    let quoted = false;
    let escaped = false;
    for (const char of value) {
        if (escaped) {
            current += char;
            escaped = false;
            continue;
        }
        if (quoted && char === '\\') {
            escaped = true;
            current += char;
            continue;
        }
        if (char === '"') {
            quoted = !quoted;
            current += char;
            continue;
        }
        if (char === ';' && !quoted) {
            parts.push(current);
            current = '';
            continue;
        }
        current += char;
    }
    if (quoted || escaped) {
        corrupt('artifact record media type has an unterminated quote');
    }
    parts.push(current);
    const parameters = new Map<string, string>();
    for (const part of parts.slice(1)) {
        const trimmed = part.trim();
        if (trimmed.length === 0) {
            corrupt('artifact record media parameter is empty');
        }
        if (parameters.size >= MAX_MEDIA_PARAMETERS) {
            corrupt('artifact record has too many media parameters');
        }
        const equals = trimmed.indexOf('=');
        if (equals <= 0) {
            corrupt('artifact record media parameter is malformed');
        }
        const name = trimmed.slice(0, equals).trim().toLowerCase();
        if (!MEDIA_TOKEN_PATTERN.test(name)) {
            corrupt('artifact record media parameter name is invalid');
        }
        const rawValue = trimmed.slice(equals + 1).trim();
        let normalized: string;
        if (rawValue.startsWith('"')) {
            normalized = parseQuotedMediaParameter(rawValue).toLowerCase();
        } else {
            if (!MEDIA_TOKEN_PATTERN.test(rawValue)) {
                corrupt('artifact record media parameter value is invalid');
            }
            normalized = rawValue.toLowerCase();
        }
        if (parameters.has(name)) {
            corrupt('artifact record has duplicate media parameters');
        }
        parameters.set(name, normalized);
    }
    return parameters;
}

function requireStoredArtifact(
    value: unknown,
    key: string,
): asserts value is ArtifactRecord & { blob: Blob } {
    if (!isObject(value)) corrupt('artifact record is not an object');
    if (
        typeof value.artifact_id !== 'string' ||
        !ARTIFACT_ID_PATTERN.test(value.artifact_id)
    ) {
        corrupt('artifact record artifact_id is invalid');
    }
    if (value.artifact_id !== key) {
        corrupt('artifact record identity does not match its key');
    }
    if (
        !isNonEmptyString(value.media_type) ||
        value.media_type.length > 128 ||
        !value.media_type.includes('/') ||
        /[\u0000-\u001f\u007f]/.test(value.media_type)
    ) {
        corrupt('artifact record media_type is invalid');
    }
    if (!isNonNegativeInteger(value.size_bytes)) {
        corrupt('artifact record size_bytes is invalid');
    }
    if (value.size_bytes > MAX_DURABLE_ARTIFACT_BYTES) {
        corrupt('artifact record exceeds the durable size limit');
    }
    if (typeof value.digest !== 'string' || !DIGEST_PATTERN.test(value.digest)) {
        corrupt('artifact record digest is invalid');
    }
    if (!(value.blob instanceof Blob)) {
        corrupt('artifact record has no durable bytes');
    }
    // Parse the declared media type unconditionally so malformed parameters
    // are refused even when the Blob carries no type. Compare parameters only
    // when the Blob type is present: declared-only parameters and reordering
    // are legal, while blob-only names or conflicting values are refused.
    const declaredParameters = parseMediaTypeParameters(value.media_type);
    if (value.blob.type !== '') {
        if (
            mediaTypeEssence(value.blob.type) !==
            mediaTypeEssence(value.media_type)
        ) {
            corrupt('artifact record blob type does not match its media_type');
        }
        const blobParameters = parseMediaTypeParameters(value.blob.type);
        for (const [name, blobValue] of blobParameters) {
            const declaredValue = declaredParameters.get(name);
            if (declaredValue === undefined) {
                corrupt('artifact record has a blob-only media parameter');
            }
            if (declaredValue !== blobValue) {
                corrupt('artifact record media parameter conflicts');
            }
        }
    }
    if (value.blob.size !== value.size_bytes) {
        corrupt('artifact record size does not match its bytes');
    }
}

function requireStoredLease(value: unknown): asserts value is LeaseRecord {
    if (!isObject(value)) corrupt('lease record is not an object');
    if (
        typeof value.task_id !== 'string' ||
        (value.task_id !== IMPORT_FENCE_KEY && !TASK_ID_PATTERN.test(value.task_id))
    ) {
        corrupt('lease task_id is invalid');
    }
    if (!isNonEmptyString(value.writer_id)) {
        corrupt('lease writer_id is invalid');
    }
    if (
        !isNonNegativeInteger(value.started_at_ms) ||
        !isNonNegativeInteger(value.heartbeat_at_ms)
    ) {
        corrupt('lease timestamps are invalid');
    }
}

function taskToAgentRecord(record: TaskRecord): AgentTaskRecord {
    return {
        taskId: record.task_id,
        projectId: record.project_id,
        title: record.title,
        repo: record.repo ?? null,
        branch: record.branch ?? null,
        createdAtMs: record.created_at_ms,
        archived: record.archived,
        pinnedLocal: record.pinned_local,
        latestRunId: record.latest_run_id ?? null,
        // An absent durable hold is an explicit clear, not "leave the in-memory
        // projection untouched". The adapter's writeTaskRecord({hold:null})
        // path therefore clears both copies consistently.
        hold: record.hold ?? null,
    };
}

function snapshotBundleSize(bundle: SnapshotBundleRecord): number {
    return byteLength(canonicalJson(jsonClean(bundle)));
}

function snapshotVerification(snapshot: SnapshotRecord): {
    taskId: string;
    runId: string;
    generation: number;
    lastSequence: number;
    lastEventId: string;
    headDigest: string;
} {
    return {
        taskId: snapshot.task_id,
        runId: snapshot.run_id,
        generation: snapshot.generation,
        lastSequence: snapshot.last_sequence,
        lastEventId: snapshot.last_event_id,
        headDigest: snapshot.head_digest,
    };
}

function snapshotMatchesEvent(
    snapshot: SnapshotRecord,
    event: AgentIncomingEvent,
    digest: string,
): boolean {
    return (
        snapshot.generation === event.generation &&
        snapshot.last_sequence === event.sequence &&
        snapshot.last_event_id === event.event_id &&
        snapshot.head_digest === digest
    );
}

export class AgentTaskStore {
    readonly store: AgentStore;

    private readonly factory: IDBFactory;
    private readonly clock: () => number;
    private readonly writerId: string;
    private readonly faults: StoreFaults;
    private readonly timers: AgentTaskStoreTimers;
    private db: IDBDatabase | null = null;
    private opened = false;
    private importInProgress = false;
    private importClosing = false;
    private importGate: Promise<void> | null = null;
    private importHeartbeatHandle: unknown = null;
    private importHeartbeatTask: Promise<void> = Promise.resolve();
    private importHeartbeatError: AgentStoreError | null = null;
    private activeWrites = 0;
    private readonly writeWaiters: Array<() => void> = [];
    private appendGate: Promise<void> | null = null;
    private appendGateReached: (() => void) | null = null;
    private taskWriteGate: Promise<void> | null = null;
    private taskWriteGateReached: (() => void) | null = null;
    private importPublishGate: Promise<void> | null = null;
    private importPublishGateReached: (() => void) | null = null;
    private closeRequested = false;
    private openPromise: Promise<OpenReport> | null = null;
    private lastReport: OpenReport | null = null;

    constructor(options: AgentTaskStoreOptions) {
        this.store = options.store;
        this.factory = options.indexedDB ?? globalThis.indexedDB;
        if (this.factory === undefined) {
            throw new AgentStoreError(
                'runtime_unavailable',
                'IndexedDB is not available in this browser',
            );
        }
        this.clock = options.now ?? (() => Date.now());
        this.writerId = options.writerId ?? `writer-${randomHex(8)}`;
        this.timers = options.timers ?? {
            setInterval: (handler, timeout) => globalThis.setInterval(handler, timeout),
            clearInterval: (handle) => {
                globalThis.clearInterval(
                    handle as ReturnType<typeof globalThis.setInterval>,
                );
            },
        };
        this.faults = options.faults ?? {
            abort_next_append: false,
            abort_next_task_write: false,
            fail_import_publish: false,
            fail_import_cleanup: false,
        };
    }

    get isOpen(): boolean {
        return this.opened && this.db !== null;
    }

    get defaultWriterId(): string {
        return this.writerId;
    }

    private get importFenceWriter(): string {
        return `import:${this.writerId}`;
    }

    setImportGate(gate: Promise<void> | null): void {
        this.importGate = gate;
    }

    setAppendGate(gate: Promise<void> | null): Promise<void> {
        this.appendGate = gate;
        return new Promise((resolve) => {
            this.appendGateReached = resolve;
        });
    }

    setTaskWriteGate(gate: Promise<void> | null): Promise<void> {
        this.taskWriteGate = gate;
        return new Promise((resolve) => {
            this.taskWriteGateReached = resolve;
        });
    }

    setImportPublishGate(gate: Promise<void> | null): Promise<void> {
        this.importPublishGate = gate;
        return new Promise((resolve) => {
            this.importPublishGateReached = resolve;
        });
    }

    private beginWrite(): void {
        this.activeWrites += 1;
    }

    private endWrite(): void {
        this.activeWrites -= 1;
        if (this.activeWrites === 0) {
            for (const resolve of this.writeWaiters.splice(0)) resolve();
        }
    }

    private drainWrites(): Promise<void> {
        if (this.activeWrites === 0) return Promise.resolve();
        return new Promise((resolve) => this.writeWaiters.push(resolve));
    }

    private startImportHeartbeat(): void {
        this.stopImportHeartbeat();
        this.importHeartbeatError = null;
        this.importHeartbeatTask = Promise.resolve();
        this.importHeartbeatHandle = this.timers.setInterval(() => {
            if (!this.importInProgress || this.importHeartbeatError !== null) {
                return;
            }
            this.importHeartbeatTask = this.importHeartbeatTask.then(async () => {
                if (!this.importInProgress || this.importHeartbeatError !== null) {
                    return;
                }
                try {
                    await this.renewImportFence(this.clock());
                } catch (error) {
                    this.importHeartbeatError =
                        error instanceof AgentStoreError
                            ? error
                            : new AgentStoreError(
                                  'runtime_unavailable',
                                  errorMessage(error),
                              );
                }
            });
            return this.importHeartbeatTask;
        }, IMPORT_HEARTBEAT_MS);
    }

    private stopImportHeartbeat(): void {
        if (this.importHeartbeatHandle !== null) {
            this.timers.clearInterval(this.importHeartbeatHandle);
            this.importHeartbeatHandle = null;
        }
    }

    private async settleImportHeartbeat(): Promise<void> {
        this.stopImportHeartbeat();
        await this.importHeartbeatTask;
    }

    private takeImportHeartbeatError(): AgentStoreError | null {
        const error = this.importHeartbeatError;
        this.importHeartbeatError = null;
        return error;
    }

    private throwImportHeartbeatError(): void {
        const error = this.takeImportHeartbeatError();
        if (error !== null) throw error;
    }

    async open(): Promise<OpenReport> {
        if (this.importInProgress || this.importClosing) {
            throw new AgentStoreError(
                'runtime_unavailable',
                'an import is in progress; open is unavailable',
            );
        }
        if (this.opened && this.lastReport !== null) {
            return this.lastReport;
        }
        if (this.openPromise !== null) {
            return this.openPromise;
        }
        this.openPromise = this.restore();
        try {
            const report = await this.openPromise;
            this.opened = true;
            this.lastReport = report;
            return report;
        } finally {
            this.openPromise = null;
        }
    }

    close(): void {
        if (this.importInProgress || this.importClosing) {
            this.closeRequested = true;
            return;
        }
        this.stopImportHeartbeat();
        this.db?.close();
        this.db = null;
        this.opened = false;
        this.openPromise = null;
        this.lastReport = null;
    }

    async deleteDatabase(): Promise<void> {
        if (this.importInProgress || this.importClosing) {
            throw new AgentStoreError(
                'runtime_unavailable',
                'an import is in progress; deleteDatabase is unavailable',
            );
        }
        this.close();
        await deleteAgentDatabase(this.factory);
    }

    async createTask(input: CreateTaskStoreInput): Promise<TaskRecord> {
        this.requireOpen();
        const title = input.title.trim();
        if (title.length === 0) invalidArgument('task title must not be empty');
        const projectId = input.project_id ?? 'prj_0000000000000001';
        if (!PROJECT_ID_PATTERN.test(projectId)) {
            invalidArgument('project_id must match prj_<16 lowercase hex>');
        }
        const taskId = input.task_id ?? `tsk_${randomHex(8)}`;
        if (!TASK_ID_PATTERN.test(taskId)) {
            invalidArgument('task_id must match tsk_<16 lowercase hex>');
        }
        if (this.store.hasTask(taskId)) {
            invalidArgument('task_id already exists');
        }
        if (input.repo !== undefined && input.repo !== null && typeof input.repo !== 'string') {
            invalidArgument('repo must be a string');
        }
        if (input.branch !== undefined && input.branch !== null && typeof input.branch !== 'string') {
            invalidArgument('branch must be a string');
        }
        if (
            input.created_at_ms !== undefined &&
            !isNonNegativeInteger(input.created_at_ms)
        ) {
            invalidArgument('created_at_ms must be a non-negative safe integer');
        }
        this.beginWrite();
        try {
        const record: TaskRecord = {
            task_id: taskId,
            project_id: projectId,
            title,
            ...(input.repo === undefined || input.repo === null ? {} : { repo: input.repo }),
            ...(input.branch === undefined || input.branch === null ? {} : { branch: input.branch }),
            created_at_ms: input.created_at_ms ?? this.clock(),
            archived: false,
            pinned_local: false,
            latest_run_id: null,
        };
        await this.persistTaskWithRollback(record);
        return record;
        } finally {
            this.endWrite();
        }
    }

    async writeTaskRecord(record: TaskRecord): Promise<TaskRecord> {
        this.requireOpen();
        requireStoredTask(record);
        this.beginWrite();
        try {
        const marker = this.store.getTaskUnresolvedRunId(record.task_id);
        let latest = record.latest_run_id ?? null;
        if (
            marker !== null &&
            (latest === null ||
                this.store.getRunCursor(record.task_id, latest) === null)
        ) {
            latest = marker;
        }
        const stored = (await this.readOne<TaskRecord>(
            'tasks',
            record.task_id,
        )) as TaskRecord | undefined;
        const hold =
            record.hold === undefined
                ? stored?.hold
                : record.hold === null
                  ? null
                  : record.hold;
        const normalized: TaskRecord = {
            ...jsonClean(record),
            latest_run_id: latest,
        };
        if (hold === null || hold === undefined) {
            delete (normalized as { hold?: unknown }).hold;
        } else {
            normalized.hold = hold;
        }
        await this.persistTaskWithRollback(normalized);
        if (
            marker !== null &&
            this.store.getRunCursor(record.task_id, marker) === null
        ) {
            this.store.setTaskUnresolvedRunId(record.task_id, marker);
        }
        return normalized;
        } finally {
            this.endWrite();
        }
    }

    async pinTask(taskId: string, pinned: boolean): Promise<TaskRecord> {
        return this.updateTaskFlag(taskId, (record) => {
            record.pinned_local = pinned;
        });
    }

    async archiveTask(taskId: string, archived: boolean): Promise<TaskRecord> {
        return this.updateTaskFlag(taskId, (record) => {
            record.archived = archived;
        });
    }

    async listTasks(options: ListTaskOptions = {}): Promise<TaskRecord[]> {
        this.requireOpen();
        const records = (await this.readAll<TaskRecord>('tasks'))
            .map((record) => {
                requireStoredTask(record);
                return record;
            })
            .filter((record) => options.includeArchived === true || !record.archived)
            .filter((record) => {
                const query = options.query?.trim().toLowerCase() ?? '';
                return query.length === 0 || record.title.toLowerCase().includes(query);
            })
            .map((record) => ({ ...record }));
        records.sort((left, right) =>
            left.created_at_ms === right.created_at_ms
                ? left.task_id.localeCompare(right.task_id)
                : left.created_at_ms - right.created_at_ms,
        );
        return records;
    }

    async readRun(taskId: string, runId: string): Promise<ReadRunResult> {
        this.requireOpen();
        const run = await this.readRunRecord(taskId, runId);
        if (run === null) {
            return { ok: false, code: 'run_not_found', message: 'run record is missing' };
        }
        const pairs = await this.readEventPairs(taskId, runId);
        if (pairs.length === 0) {
            return {
                ok: false,
                code: 'snapshot_missing',
                message: 'run record has no canonical event records',
            };
        }
        try {
            const verified = await this.verifyEventChain(taskId, runId, pairs);
            this.assertRunRecordMatches(run, verified);
            return {
                ok: true,
                run,
                events: pairs.map((pair) => pair.record),
            };
        } catch (error) {
            if (error instanceof AgentStoreError) {
                return { ok: false, code: error.code, message: errorMessage(error) };
            }
            return {
                ok: false,
                code: 'snapshot_corrupt',
                message: errorMessage(error),
            };
        }
    }

    async writeSnapshot(taskId: string, runId: string): Promise<SnapshotRecord> {
        this.requireOpen();
        if (this.importInProgress) {
            throw new AgentStoreError(
                'runtime_unavailable',
                'an import is in progress; snapshot writes are unavailable',
            );
        }
        this.beginWrite();
        try {
        const view = this.store.getRun(taskId, runId);
        if (view === null || view.lastEventId === null || view.lastDigest === null) {
            throw new AgentStoreError(
                'snapshot_missing',
                'run has no verified event chain to snapshot',
            );
        }
        const read = await this.readRun(taskId, runId);
        if (!read.ok) {
            throw new AgentStoreError(read.code, read.message);
        }
        if (
            read.run.last_event_id !== view.lastEventId ||
            read.run.last_sequence !== view.lastSequence ||
            read.run.last_digest !== view.lastDigest
        ) {
            throw new AgentStoreError(
                'snapshot_corrupt',
                'in-memory run cursor does not match the durable event chain',
            );
        }
        const snapshot = this.buildSnapshot(view);
        const pairs = await this.readEventPairs(taskId, runId);
        const verified = await this.verifyEventChain(taskId, runId, pairs);
        const resumeDigests = new Set(
            verified.events
                .filter((event) => event.type === 'run.resumed')
                .map(
                    (event) =>
                        (event.data as Record<string, unknown>)
                            .snapshot_digest as string,
                ),
        );
        const db = this.requireDb();
        const transaction = db.transaction(['snapshots', 'runs', 'leases'], 'readwrite');
        try {
            await this.checkImportFence(
                transaction.objectStore('leases'),
                this.clock(),
            );
            const storedRun = (await requestResult(
                transaction.objectStore('runs').get([taskId, runId]),
            )) as RunRecord | undefined;
            if (
                storedRun === undefined ||
                storedRun.last_event_id !== snapshot.last_event_id ||
                storedRun.last_sequence !== snapshot.last_sequence ||
                storedRun.last_digest !== snapshot.head_digest
            ) {
                throw new AgentStoreError(
                    'snapshot_corrupt',
                    'durable run record moved before the snapshot was written',
                );
            }
            const existingSnapshot = (await requestResult(
                transaction.objectStore('snapshots').get([taskId, runId]),
            )) as StoredSnapshotRecord | undefined;
            const existingBundle =
                existingSnapshot === undefined
                    ? null
                    : normalizeSnapshotBundle(existingSnapshot);
            if (existingBundle !== null) {
                this.verifySnapshotBundleAgainstChain(existingBundle, verified);
            }
            const retained =
                existingBundle === null
                    ? []
                    : snapshotRecordsOf(existingBundle).filter((record) =>
                          resumeDigests.has(record.head_digest),
                      );
            retained.push(snapshot);
            const bundle = bundleFromRecords(taskId, runId, retained);
            this.assertSnapshotBundleSize(bundle);
            await requestResult(
                transaction
                    .objectStore('snapshots')
                    .put(jsonClean(bundle), [taskId, runId]),
            );
            await requestResult(
                transaction.objectStore('runs').put(jsonClean(storedRun), [taskId, runId]),
            );
            await transactionDone(transaction);
        } catch (error) {
            try {
                transaction.abort();
            } catch {
                // The transaction may already be finished.
            }
            throw storageError(error);
        }
        const verification = this.store.verifySnapshot(snapshotVerification(snapshot));
        if (!verification.ok) {
            throw new AgentStoreError(
                verification.code ?? 'snapshot_corrupt',
                verification.message ?? 'snapshot verification failed',
            );
        }
        return snapshot;
        } finally {
            this.endWrite();
        }
    }

    async readSnapshot(taskId: string, runId: string): Promise<ReadResult<SnapshotRecord>> {
        this.requireOpen();
        const storedValue = (await this.readOne<StoredSnapshotRecord>(
            'snapshots',
            [taskId, runId],
        )) as StoredSnapshotRecord | undefined;
        if (storedValue === undefined) {
            return { ok: false, code: 'snapshot_missing', message: 'snapshot record is missing' };
        }
        try {
            const bundle = normalizeSnapshotBundle(storedValue);
            if (bundle.task_id !== taskId || bundle.run_id !== runId) {
                corrupt('snapshot record identity does not match its key');
            }
            const run = await this.readRunRecord(taskId, runId);
            if (run === null) {
                return {
                    ok: false,
                    code: 'snapshot_corrupt',
                    message: 'snapshot record exists without a run record',
                };
            }
            const pairs = await this.readEventPairs(taskId, runId);
            const verified = await this.verifyEventChain(taskId, runId, pairs);
            this.assertRunRecordMatches(run, verified);
            this.verifySnapshotBundleAgainstChain(bundle, verified);
            return { ok: true, record: bundle.latest };
        } catch (error) {
            if (error instanceof AgentStoreError) {
                return {
                    ok: false,
                    code: error.code,
                    message: errorMessage(error),
                };
            }
            return {
                ok: false,
                code: 'snapshot_corrupt',
                message: errorMessage(error),
            };
        }
    }

    async recordChanges(input: ChangeInput): Promise<ChangeRecord> {
        this.requireOpen();
        if (this.importInProgress) {
            throw new AgentStoreError(
                'runtime_unavailable',
                'an import is in progress; change writes are unavailable',
            );
        }
        this.beginWrite();
        try {
        if (!TASK_ID_PATTERN.test(input.task_id)) invalidArgument('task_id is invalid');
        if (!RUN_ID_PATTERN.test(input.run_id)) invalidArgument('run_id is invalid');
        if (!isNonEmptyString(input.head_digest)) invalidArgument('head_digest must be non-empty');
        if (
            input.scope !== undefined &&
            input.scope !== 'last_turn'
        ) {
            invalidArgument('change scope must be last_turn');
        }
        if (!Array.isArray(input.files) || !input.files.every((file) => isObject(file))) {
            invalidArgument('change files must be an array of objects');
        }
        if (!isNonNegativeInteger(input.captured_at_ms)) {
            invalidArgument('captured_at_ms must be a non-negative safe integer');
        }
        const view = this.store.getRun(input.task_id, input.run_id);
        if (view === null) {
            throw new AgentStoreError('run_not_found', 'run is not present in AgentStore');
        }
        const bound = view.changesSnapshots.find(
            (snapshot) =>
                snapshot.scope === 'last_turn' &&
                snapshot.headDigest === input.head_digest,
        );
        if (bound === undefined) {
            invalidArgument(
                'head_digest is not bound to an applied changes.snapshot event',
            );
        }
        let inputFiles: string;
        let boundFiles: string;
        try {
            inputFiles = canonicalJson(input.files);
            boundFiles = canonicalJson(bound.files);
        } catch {
            invalidArgument('change files are not canonical JSON objects');
        }
        if (inputFiles !== boundFiles) {
            invalidArgument(
                'change files do not match the applied changes.snapshot event',
            );
        }
        const record: ChangeRecord = {
            task_id: input.task_id,
            run_id: input.run_id,
            scope: 'last_turn',
            files: jsonClean(bound.files),
            head_digest: bound.headDigest,
            captured_at_ms: bound.capturedAtMs,
        };
        const transaction = this.requireDb().transaction(['changes', 'leases'], 'readwrite');
        try {
            await this.checkImportFence(
                transaction.objectStore('leases'),
                this.clock(),
            );
            await requestResult(
                transaction
                    .objectStore('changes')
                    .put(jsonClean(record), [
                        record.task_id,
                        record.run_id,
                        record.head_digest,
                    ]),
            );
            await transactionDone(transaction);
        } catch (error) {
            throw storageError(error);
        }
        return record;
        } finally {
            this.endWrite();
        }
    }

    async readChanges(taskId: string, runId?: string): Promise<ChangeRecord[]> {
        this.requireOpen();
        const records = (await this.readAll<ChangeRecord>('changes'))
            .map((record) => {
                requireStoredChange(record);
                return record;
            })
            .filter(
                (record) =>
                    record.task_id === taskId &&
                    (runId === undefined || record.run_id === runId),
            )
            .map((record) => {
                const view = this.store.getRun(record.task_id, record.run_id);
                if (view === null) {
                    corrupt('change record belongs to a missing run');
                }
                const event = view.events.find((candidate) => {
                    const data = candidate.data as Record<string, unknown>;
                    return (
                        candidate.type === 'changes.snapshot' &&
                        data.scope === 'last_turn' &&
                        data.head_digest === record.head_digest
                    );
                });
                if (event === undefined) {
                    corrupt('change record has no matching changes.snapshot event');
                }
                let recordFiles: string;
                let eventFiles: string;
                try {
                    recordFiles = canonicalJson(record.files);
                    eventFiles = canonicalJson(
                        (event.data as Record<string, unknown>).files,
                    );
                } catch {
                    corrupt('change record files are not canonical JSON');
                }
                if (recordFiles !== eventFiles) {
                    corrupt('change record files do not match the snapshot event');
                }
                return {
                    ...record,
                    files: jsonClean(
                        (event.data as Record<string, unknown>).files as Array<
                            Record<string, unknown>
                        >,
                    ),
                };
            });
        records.sort((left, right) =>
            left.captured_at_ms === right.captured_at_ms
                ? left.head_digest.localeCompare(right.head_digest)
                : left.captured_at_ms - right.captured_at_ms,
        );
        return records;
    }

    /**
     * Persist one local diff comment.
     *
     * Comments are keyed by the exact task/path/line/head tuple, so a comment
     * on a moved or changed line is never silently attached to a new line.
     * They are local browser records: no remote service or share path exists.
     */
    /**
     * Append one canonical local `diff.comment` event and its projection.
     *
     * The event is the authority: it follows the run's digest chain even when
     * the run is terminal, because a comment is an annotation and never a run
     * state transition. The `comments` store is written in the same transaction
     * and is rebuilt from canonical events on open/import.
     */
    async appendDiffComment(input: AppendDiffCommentInput): Promise<AppendedDiffComment> {
        this.requireOpen();
        if (this.importInProgress) {
            throw new AgentStoreError(
                'runtime_unavailable',
                'an import is in progress; comment writes are unavailable',
            );
        }
        if (!TASK_ID_PATTERN.test(input.task_id)) invalidArgument('task_id is invalid');
        if (!isNonEmptyString(input.path) || input.path.length > MAX_DIFF_COMMENT_PATH) {
            invalidArgument('comment path must be a bounded non-empty string');
        }
        if (!isNonNegativeInteger(input.line)) {
            invalidArgument('comment line must be a non-negative safe integer');
        }
        if (typeof input.body !== 'string' || input.body.length > MAX_DIFF_COMMENT_BODY) {
            invalidArgument('comment body must be a bounded string');
        }
        if (!isNonEmptyString(input.head_digest) || input.head_digest.length > MAX_DIFF_COMMENT_DIGEST) {
            invalidArgument('comment head_digest must be a bounded non-empty string');
        }
        if (input.scope !== 'last_turn' && input.scope !== 'branch') {
            invalidArgument('comment scope must be last_turn | branch');
        }
        if (
            input.comment_id !== undefined &&
            (!isNonEmptyString(input.comment_id) || input.comment_id.length > 128)
        ) {
            invalidArgument('comment_id must be a bounded non-empty string');
        }
        if (input.created_at_ms !== undefined && !isNonNegativeInteger(input.created_at_ms)) {
            invalidArgument('created_at_ms must be a non-negative safe integer');
        }
        this.beginWrite();
        try {
            const task = await this.readOne<TaskRecord>('tasks', input.task_id);
            if (task === undefined) {
                throw new AgentStoreError('task_not_found', 'task is missing');
            }
            requireStoredTask(task);
            const finishedRuns = (await this.readAll<RunRecord>('runs'))
                .filter((candidate) => candidate.task_id === input.task_id)
                .map((candidate) => {
                    requireStoredRun(candidate);
                    return candidate;
                })
                .filter((candidate) => TERMINAL_RUN_STATES.has(candidate.state))
                .sort((left, right) =>
                    right.started_at_ms - left.started_at_ms ||
                    right.generation - left.generation ||
                    right.run_id.localeCompare(left.run_id),
                );
            if (finishedRuns.length === 0) {
                throw new AgentStoreError(
                    'out_of_order',
                    'local comments are available after the run finishes',
                );
            }
            let run: RunRecord | null = null;
            for (const candidate of finishedRuns) {
                const pairs = await this.readEventPairs(
                    candidate.task_id,
                    candidate.run_id,
                );
                const candidateEvents: AgentIncomingEvent[] = [];
                for (const pair of pairs) {
                    candidateEvents.push(await this.materializeEvent(pair.record));
                }
                const candidateScope = lastTurnDigestsOf(candidateEvents).has(input.head_digest)
                    ? 'last_turn'
                    : 'branch';
                if (candidateScope === input.scope) {
                    run = candidate;
                    break;
                }
            }
            if (run === null) {
                invalidArgument(
                    'comment does not bind to a snapshot in any finished run',
                );
            }
            const derivedScope = input.scope;

            const generation = run.generation;
            const sequence = run.last_sequence + 1;
            if (sequence >= MAX_EVENTS_PER_RUN) {
                throw new AgentStoreError('limit_exceeded', 'run event limit is reached');
            }
            const fenceNow = this.clock();
            const now = input.created_at_ms ?? fenceNow;
            const eventId = `evt_${run.run_id}_${generation}_${sequence
                .toString()
                .padStart(12, '0')}`;
            const commentId = input.comment_id ?? `cmt_${randomHex(16)}`;
            const event = {
                schema: AGENT_EVENT_SCHEMA,
                event_id: eventId,
                project_id: task.project_id,
                task_id: run.task_id,
                run_id: run.run_id,
                generation,
                sequence,
                recorded_at_ms: now,
                type: 'diff.comment',
                data: {
                    comment_id: commentId,
                    path: input.path,
                    line: input.line,
                    body: input.body,
                    head_digest: input.head_digest,
                },
            } as unknown as AgentIncomingEvent;
            const validation = validateAgentEvent(event);
            if (!validation.ok) {
                throw new AgentStoreError(validation.code, validation.message);
            }
            const canonical = canonicalJson(event);
            if (byteLength(canonical) > MAX_EVENT_BYTES) {
                throw new AgentStoreError('limit_exceeded', 'event exceeds max_event_bytes');
            }
            const digest = await digestEvent(event as AgentEvent);
            const record: DiffCommentRecord = {
                task_id: run.task_id,
                comment_id: commentId,
                path: input.path,
                line: input.line,
                body: input.body,
                head_digest: input.head_digest,
                scope: derivedScope,
                created_at_ms: now,
            };
            const db = this.requireDb();
            const transaction = db.transaction(
                ['tasks', 'runs', 'events', 'comments', 'leases'],
                'readwrite',
            );
            try {
                const leases = transaction.objectStore('leases');
                await this.checkImportFence(leases, fenceNow);
                const lease = (await requestResult(leases.get(run.task_id))) as
                    | LeaseRecord
                    | undefined;
                if (lease !== undefined) {
                    requireStoredLease(lease);
                    if (
                        lease.writer_id !== this.writerId &&
                        fenceNow - lease.heartbeat_at_ms <= LEASE_STALE_MS
                    ) {
                        throw new AgentStoreError(
                            'stale_generation',
                            'task lease is held by another active writer',
                        );
                    }
                }
                const current = (await requestResult(
                    transaction.objectStore('runs').get([run.task_id, run.run_id]),
                )) as RunRecord | undefined;
                if (current === undefined) {
                    throw new AgentStoreError('run_not_found', 'run disappeared before append');
                }
                requireStoredRun(current);
                if (
                    current.last_sequence !== run.last_sequence ||
                    current.last_digest !== run.last_digest
                ) {
                    throw new AgentStoreError(
                        'out_of_order',
                        'run cursor changed before the comment was appended',
                    );
                }
                const existing = (await requestResult(
                    transaction
                        .objectStore('events')
                        .get([run.task_id, run.run_id, generation, sequence]),
                )) as EventRecord | undefined;
                if (existing !== undefined) {
                    throw new AgentStoreError(
                        'duplicate_event',
                        'comment event key is already present',
                    );
                }
                await requestResult(
                    transaction.objectStore('events').put(
                        jsonClean({
                            event_id: eventId,
                            canonical,
                            digest,
                            prev_digest: current.last_digest,
                            recorded_at_ms: now,
                        } satisfies EventRecord),
                        [run.task_id, run.run_id, generation, sequence],
                    ),
                );
                await requestResult(
                    transaction.objectStore('runs').put(
                        jsonClean({
                            ...current,
                            last_sequence: sequence,
                            last_event_id: eventId,
                            last_digest: digest,
                        } satisfies RunRecord),
                        [run.task_id, run.run_id],
                    ),
                );
                await requestResult(
                    transaction
                        .objectStore('comments')
                        .put(jsonClean(record), diffCommentKey(record)),
                );
                await transactionDone(transaction);
            } catch (error) {
                try { transaction.abort(); } catch { /* already finished */ }
                throw storageError(error);
            }
            return {
                comment: jsonClean(record),
                run_id: run.run_id,
                sequence,
                event_id: eventId,
                digest,
                prev_digest: run.last_digest,
            };
        } finally {
            this.endWrite();
        }
    }

    async readDiffComments(taskId: string): Promise<DiffCommentRecord[]> {
        this.requireOpen();
        if (!TASK_ID_PATTERN.test(taskId)) invalidArgument('task_id is invalid');
        const records = (await this.readAll<DiffCommentRecord>('comments'))
            .filter((record) => record.task_id === taskId)
            .map((record) => {
                requireStoredDiffComment(record);
                return jsonClean(record);
            });
        records.sort((left, right) =>
            left.path.localeCompare(right.path) ||
            left.line - right.line ||
            left.created_at_ms - right.created_at_ms,
        );
        return records;
    }

    /**
     * Read one durable browser artifact by identity.
     *
     * Returns `null` only when no record exists. A present record is validated
     * against the declared media type, size, digest and blob bytes; corrupt or
     * metadata-only records throw `snapshot_corrupt` instead of being treated
     * as missing. The returned bytes are a fresh copy.
     */
    async readArtifact(
        artifactId: string,
        range?: DurableArtifactRange,
    ): Promise<DurableArtifact | null> {
        this.requireOpen();
        if (
            typeof artifactId !== 'string' ||
            !ARTIFACT_ID_PATTERN.test(artifactId)
        ) {
            invalidArgument('artifact_id is invalid');
        }
        const record = await this.readOne<ArtifactRecord>('artifacts', artifactId);
        if (record === undefined) return null;
        requireStoredArtifact(record, artifactId);
        const total = record.size_bytes;
        let start = 0;
        let end = total;
        if (range !== undefined) {
            if (
                !isNonNegativeInteger(range.start) ||
                !isNonNegativeInteger(range.end) ||
                range.start > range.end ||
                range.end > total
            ) {
                invalidArgument('artifact range is outside the artifact');
            }
            start = range.start;
            end = range.end;
            if (end - start > MAX_DURABLE_ARTIFACT_RETURN_BYTES) {
                throw new AgentStoreError(
                    'limit_exceeded',
                    'artifact range exceeds the durable return limit',
                );
            }
        }
        const length = end - start;
        let wholeBytes: Uint8Array;
        try {
            const bounded = record.blob.slice(0, total);
            wholeBytes = new Uint8Array(await bounded.arrayBuffer());
        } catch {
            corrupt('artifact record bytes are unavailable');
        }
        if (wholeBytes.byteLength !== total) {
            corrupt('artifact record size does not match its bytes');
        }
        // Verify the declared whole digest before returning any slice. A
        // corrupt whole record must fail for whole and subrange reads alike.
        const wholeDigest = await sha256Bytes(wholeBytes);
        if (wholeDigest !== record.digest) {
            corrupt('artifact record digest does not match its bytes');
        }
        const bytes =
            start === 0 && end === total
                ? wholeBytes
                : wholeBytes.slice(start, end);
        if (bytes.byteLength !== length) {
            corrupt('artifact record size does not match its bytes');
        }
        const digest =
            start === 0 && end === total
                ? wholeDigest
                : await sha256Bytes(bytes);
        return {
            artifact_id: record.artifact_id,
            media_type: record.media_type,
            size_bytes: length,
            artifact_size_bytes: total,
            digest,
            whole_digest: wholeDigest,
            range: { start, end },
            bytes,
        };
    }

    async acquireLease(
        taskId: string,
        writerId: string,
        nowMs = this.clock(),
    ): Promise<LeaseRecord> {
        this.requireOpen();
        if (this.importInProgress) {
            throw new AgentStoreError('runtime_unavailable', 'an import is in progress');
        }
        this.beginWrite();
        try {
        this.requireWriter(writerId);
        if (taskId === IMPORT_FENCE_KEY) invalidArgument('task_id is reserved');
        if (!TASK_ID_PATTERN.test(taskId)) invalidArgument('task_id is invalid');
        const task = this.store.snapshot().tasks.find(
            (candidate) => candidate.taskId === taskId,
        );
        if (task === undefined) {
            throw new AgentStoreError('task_not_found', 'task is not present in AgentStore');
        }
        if (task.latestRunId !== null) {
            const read = await this.readRun(taskId, task.latestRunId);
            if (!read.ok) {
                throw new AgentStoreError(read.code, read.message);
            }
            const storedSnapshot = await this.readOne<SnapshotRecord>(
                'snapshots',
                [taskId, task.latestRunId],
            );
            if (storedSnapshot !== undefined) {
                const snapshotRead = await this.readSnapshot(
                    taskId,
                    task.latestRunId,
                );
                if (!snapshotRead.ok) {
                    throw new AgentStoreError(
                        snapshotRead.code,
                        snapshotRead.message,
                        snapshotRead.code === 'limit_exceeded'
                            ? 'indexeddb'
                            : null,
                    );
                }
            }
        }
        if (!isNonNegativeInteger(nowMs)) invalidArgument('now_ms is invalid');
        const transaction = this.requireDb().transaction(
            ['tasks', 'runs', 'leases'],
            'readwrite',
        );
        try {
            const tasks = transaction.objectStore('tasks');
            const runs = transaction.objectStore('runs');
            const stores = transaction.objectStore('leases');
            await this.checkImportFence(stores, nowMs);
            const durableTask = (await requestResult(
                tasks.get(taskId),
            )) as TaskRecord | undefined;
            if (durableTask === undefined) {
                throw new AgentStoreError(
                    'task_not_found',
                    'task is not present in durable storage',
                );
            }
            requireStoredTask(durableTask);
            const durableLatest = durableTask.latest_run_id ?? null;
            if (durableLatest !== task.latestRunId) {
                // The local view is stale: another tab committed a different
                // latest run. Never let a local terminal projection displace a
                // fresh active writer.
                throw new AgentStoreError(
                    'stale_generation',
                    'durable task latest run does not match the local task view',
                );
            }
            let durableLatestState: string | null = null;
            if (durableLatest !== null) {
                const durableRun = (await requestResult(
                    runs.get([taskId, durableLatest]),
                )) as RunRecord | undefined;
                if (durableRun === undefined) {
                    throw new AgentStoreError(
                        'snapshot_missing',
                        'durable task latest run record is missing',
                    );
                }
                requireStoredRun(durableRun);
                durableLatestState = durableRun.state;
            }
            const existing = (await requestResult(stores.get(taskId))) as LeaseRecord | undefined;
            if (existing !== undefined) {
                requireStoredLease(existing);
            }
            const activeOther =
                existing !== undefined &&
                existing.writer_id !== writerId &&
                nowMs - existing.heartbeat_at_ms <= LEASE_STALE_MS &&
                // A lease cannot belong to a terminal run: the writer that
                // owned it has nothing left to do. The terminal exception is
                // derived only from the durable latest run, never a stale
                // in-memory projection.
                !(durableLatestState !== null && TERMINAL_RUN_STATES.has(durableLatestState));
            if (activeOther) {
                throw new AgentStoreError(
                    'stale_generation',
                    'task lease is held by another active writer',
                );
            }
            const record: LeaseRecord =
                existing !== undefined && existing.writer_id === writerId
                    ? {
                          task_id: taskId,
                          writer_id: writerId,
                          started_at_ms: existing.started_at_ms,
                          heartbeat_at_ms: nowMs,
                      }
                    : {
                          task_id: taskId,
                          writer_id: writerId,
                          started_at_ms: nowMs,
                          heartbeat_at_ms: nowMs,
                      };
            await requestResult(stores.put(jsonClean(record), taskId));
            await transactionDone(transaction);
            return record;
        } catch (error) {
            try {
                transaction.abort();
            } catch {
                // The transaction may already be finished.
            }
            throw storageError(error);
        }
        } finally {
            this.endWrite();
        }
    }

    async heartbeatLease(
        taskId: string,
        writerId: string,
        nowMs = this.clock(),
    ): Promise<LeaseRecord> {
        this.requireOpen();
        if (this.importInProgress) {
            throw new AgentStoreError('runtime_unavailable', 'an import is in progress');
        }
        this.beginWrite();
        try {
        this.requireWriter(writerId);
        if (taskId === IMPORT_FENCE_KEY) invalidArgument('task_id is reserved');
        if (!isNonNegativeInteger(nowMs)) invalidArgument('now_ms is invalid');
        const transaction = this.requireDb().transaction('leases', 'readwrite');
        try {
            const stores = transaction.objectStore('leases');
            await this.checkImportFence(stores, nowMs);
            const existing = (await requestResult(stores.get(taskId))) as LeaseRecord | undefined;
            if (existing === undefined || existing.writer_id !== writerId) {
                throw new AgentStoreError(
                    'stale_generation',
                    'lease is missing or held by another writer',
                );
            }
            requireStoredLease(existing);
            const record: LeaseRecord = {
                ...existing,
                heartbeat_at_ms: nowMs,
            };
            await requestResult(stores.put(jsonClean(record), taskId));
            await transactionDone(transaction);
            return record;
        } catch (error) {
            try {
                transaction.abort();
            } catch {
                // The transaction may already be finished.
            }
            throw storageError(error);
        }
        } finally {
            this.endWrite();
        }
    }

    async releaseLease(taskId: string, writerId: string): Promise<{ released: boolean }> {
        this.requireOpen();
        if (this.importInProgress) {
            throw new AgentStoreError('runtime_unavailable', 'an import is in progress');
        }
        this.beginWrite();
        try {
        this.requireWriter(writerId);
        if (taskId === IMPORT_FENCE_KEY) invalidArgument('task_id is reserved');
        const transaction = this.requireDb().transaction('leases', 'readwrite');
        try {
            const stores = transaction.objectStore('leases');
            await this.checkImportFence(stores, this.clock());
            const existing = (await requestResult(stores.get(taskId))) as LeaseRecord | undefined;
            if (existing === undefined) {
                await transactionDone(transaction);
                return { released: false };
            }
            requireStoredLease(existing);
            if (existing.writer_id !== writerId) {
                throw new AgentStoreError(
                    'stale_generation',
                    'lease is held by another writer',
                );
            }
            await requestResult(stores.delete(taskId));
            await transactionDone(transaction);
            return { released: true };
        } catch (error) {
            try {
                transaction.abort();
            } catch {
                // The transaction may already be finished.
            }
            throw storageError(error);
        }
        } finally {
            this.endWrite();
        }
    }

    async readLease(taskId: string): Promise<LeaseRecord | null> {
        this.requireOpen();
        const record = (await this.readOne<LeaseRecord>('leases', taskId)) as
            | LeaseRecord
            | undefined;
        if (record === undefined) return null;
        requireStoredLease(record);
        if (record.task_id !== taskId) {
            corrupt('lease record task_id does not match its key');
        }
        return record;
    }

    async appendEvent(input: AppendEventStoreInput): Promise<AppendStoreResult> {
        this.requireOpen();
        this.beginWrite();
        try {
        if (this.appendGate !== null) {
            const gate = this.appendGate;
            this.appendGate = null;
            const reached = this.appendGateReached;
            this.appendGateReached = null;
            reached?.();
            await gate;
        }
        const event = input.event;
        if (event.type === 'diff.comment') {
            return {
                status: 'rejected',
                code: 'invalid_event',
                message: 'diff.comment is local-only; use the dedicated comment append path',
                event_id: event.event_id,
                digest: null,
                storage: null,
            };
        }
        if (input.now_ms !== undefined && !isNonNegativeInteger(input.now_ms)) {
            return {
                status: 'rejected',
                code: 'invalid_arguments',
                message: 'now_ms must be a non-negative safe integer',
                event_id: event.event_id,
                digest: null,
                storage: null,
            };
        }
        const now = input.now_ms ?? this.clock();
        let canonical: string;
        let digest: string;
        try {
            canonical = canonicalJson(event);
            digest = await digestEvent(event as AgentEvent);
        } catch (error) {
            const code: AgentErrorCode =
                error instanceof CanonicalJsonError ? error.code : 'invalid_event';
            return {
                status: 'rejected',
                code,
                message: errorMessage(error),
                event_id: event.event_id,
                digest: null,
                storage: null,
            };
        }
        if (byteLength(canonical) > MAX_EVENT_BYTES) {
            return {
                status: 'rejected',
                code: 'limit_exceeded',
                message: 'event exceeds max_event_bytes',
                event_id: event.event_id,
                digest,
                storage: null,
            };
        }
        if (input.digest !== undefined && input.digest !== digest) {
            return {
                status: 'rejected',
                code: 'invalid_event',
                message: 'provided digest does not match the recomputed canonical digest',
                event_id: event.event_id,
                digest,
                storage: null,
            };
        }
        try {
            if (input.snapshot_anchor !== undefined || input.snapshot === true) {
                await this.verifyExistingSnapshotForRun(event.task_id, event.run_id);
            } else {
                await this.verifyExistingSnapshotAnchors(event.task_id, event.run_id);
            }
        } catch (error) {
            if (error instanceof AgentStoreError) {
                return {
                    status: 'rejected',
                    code: error.code,
                    message: errorMessage(error),
                    event_id: event.event_id,
                    digest,
                    storage: error.storage,
                };
            }
            return {
                status: 'rejected',
                code: 'snapshot_corrupt',
                message: errorMessage(error),
                event_id: event.event_id,
                digest,
                storage: null,
            };
        }

        let journaled;
        try {
            journaled = this.store.applyJournaled(event, digest);
        } catch (error) {
            return {
                status: 'rejected',
                code: 'internal',
                message: errorMessage(error),
                event_id: event.event_id,
                digest,
                storage: null,
            };
        }
        const result = journaled.result;
        if (result.status !== 'applied') {
            return {
                ...result,
                event_id: event.event_id,
                digest,
                storage: null,
            };
        }

        try {
            await this.commitEvent(event, canonical, digest, input, now);
        } catch (error) {
            journaled.rollback();
            if (error instanceof AgentStoreError && error.storage === null) {
                return {
                    status: 'rejected',
                    code: error.code,
                    message: errorMessage(error),
                    event_id: event.event_id,
                    digest,
                    storage: null,
                };
            }
            return {
                status: 'rejected',
                code: 'limit_exceeded',
                message: errorMessage(error),
                event_id: event.event_id,
                digest,
                storage: 'indexeddb',
            };
        }
        return {
            status: 'applied',
            code: null,
            message: null,
            event_id: event.event_id,
            digest,
            storage: null,
        };
        } finally {
            this.endWrite();
        }
    }

    async exportNdjson(): Promise<string> {
        this.requireOpen();
        const tasks = (await this.readAll<TaskRecord>('tasks')).map((record) => {
            requireStoredTask(record);
            return record;
        });
        const runs = (await this.readAll<RunRecord>('runs')).map((record) => {
            requireStoredRun(record);
            return record;
        });
        const snapshots: SnapshotRecord[] = [];
        for (const { key, record } of await this.readAllPairs<StoredSnapshotRecord>(
            'snapshots',
        )) {
            const bundle = normalizeSnapshotBundle(record);
            if (
                !Array.isArray(key) ||
                key.length !== 2 ||
                key[0] !== bundle.task_id ||
                key[1] !== bundle.run_id
            ) {
                corrupt('snapshot record key does not match its task/run identity');
            }
            this.assertSnapshotBundleSize(bundle);
            for (const manifest of snapshotRecordsOf(bundle)) {
                if (!snapshots.some((existing) => sameSnapshot(existing, manifest))) {
                    snapshots.push(manifest);
                }
            }
        }
        const changes = (await this.readAll<ChangeRecord>('changes')).map((record) => {
            requireStoredChange(record);
            return record;
        });
        const eventPairs = await this.readAllEventPairs();
        const eventGroups = new Map<string, EventPair[]>();
        for (const pair of eventPairs) {
            const key = this.pairRunKey(pair);
            const group = eventGroups.get(key);
            if (group === undefined) {
                eventGroups.set(key, [pair]);
            } else {
                group.push(pair);
            }
        }
        const runKeys = new Set(runs.map((run) => runKey(run.task_id, run.run_id)));
        for (const key of eventGroups.keys()) {
            if (!runKeys.has(key)) {
                corrupt('event records exist without a run record');
            }
        }
        const sortedRuns = [...runs].sort(
            (left, right) =>
                left.task_id.localeCompare(right.task_id) ||
                left.generation - right.generation ||
                left.run_id.localeCompare(right.run_id),
        );
        const verifiedByRun = new Map<
            string,
            { events: AgentIncomingEvent[]; digests: string[] }
        >();
        for (const run of sortedRuns) {
            const pairs = eventGroups.get(runKey(run.task_id, run.run_id)) ?? [];
            if (pairs.length === 0) {
                corrupt('run record has no canonical event records');
            }
            const verified = await this.verifyEventChain(run.task_id, run.run_id, pairs);
            this.assertRunRecordMatches(run, verified);
            verifiedByRun.set(runKey(run.task_id, run.run_id), verified);
        }
        const expectedComments = new Map<string, DiffCommentRecord>();
        for (const run of sortedRuns) {
            const verified = verifiedByRun.get(runKey(run.task_id, run.run_id))!;
            const lastTurnDigests = lastTurnDigestsOf(verified.events);
            for (const event of verified.events) {
                if (event.type !== 'diff.comment') continue;
                const record = diffCommentRecordFromEvent(event, lastTurnDigests);
                expectedComments.set(diffCommentKey(record).join('\u0000'), record);
            }
        }
        const storedComments = (await this.readAll<DiffCommentRecord>('comments')).map((record) => {
            requireStoredDiffComment(record);
            return record;
        });
        if (storedComments.length !== expectedComments.size) {
            corrupt('comments projection does not match canonical diff.comment events');
        }
        for (const record of storedComments) {
            const expected = expectedComments.get(diffCommentKey(record).join('\u0000'));
            if (
                expected === undefined ||
                expected.comment_id !== record.comment_id ||
                expected.body !== record.body ||
                expected.scope !== record.scope ||
                expected.created_at_ms !== record.created_at_ms
            ) {
                corrupt('comments projection does not match canonical diff.comment events');
            }
        }
        for (const change of changes) {
            const verified = verifiedByRun.get(
                runKey(change.task_id, change.run_id),
            );
            if (verified === undefined) {
                corrupt('change record belongs to a missing run record');
            }
            const event = verified.events.find((candidate) => {
                const data = candidate.data as Record<string, unknown>;
                return (
                    candidate.type === 'changes.snapshot' &&
                    data.scope === 'last_turn' &&
                    data.head_digest === change.head_digest
                );
            });
            if (event === undefined) {
                corrupt('change record has no matching changes.snapshot event');
            }
            let changeFiles: string;
            let eventFiles: string;
            try {
                changeFiles = canonicalJson(change.files);
                eventFiles = canonicalJson(
                    (event.data as Record<string, unknown>).files,
                );
            } catch {
                corrupt('change files are not canonical JSON');
            }
            if (changeFiles !== eventFiles) {
                corrupt('change files do not match the applied snapshot event');
            }
        }
        const runMap = new Map(
            sortedRuns.map((run) => [runKey(run.task_id, run.run_id), run]),
        );
        for (const snapshot of snapshots) {
            const run = runMap.get(runKey(snapshot.task_id, snapshot.run_id));
            if (run === undefined) {
                corrupt('snapshot record exists without a run record');
            }
            const pairs = eventGroups.get(runKey(snapshot.task_id, snapshot.run_id)) ?? [];
            const verified = await this.verifyEventChain(
                snapshot.task_id,
                snapshot.run_id,
                pairs,
            );
            const index = verified.events.findIndex(
                (event) =>
                    event.generation === snapshot.generation &&
                    event.sequence === snapshot.last_sequence,
            );
            if (
                index < 0 ||
                verified.events[index]!.event_id !== snapshot.last_event_id ||
                verified.digests[index] !== snapshot.head_digest
            ) {
                corrupt('snapshot position does not match the exported event chain');
            }
        }
        const lines: string[] = [];
        const push = (value: unknown) => {
            lines.push(canonicalJson(jsonClean(value)));
        };
        push({
            kind: 'header',
            schema: EXPORT_SCHEMA,
            exported_at_ms: this.clock(),
        });
        for (const task of [...tasks].sort(
            (left, right) =>
                left.created_at_ms - right.created_at_ms ||
                left.task_id.localeCompare(right.task_id),
        )) {
            push({ kind: 'task', record: task });
        }
        for (const run of sortedRuns) {
            push({ kind: 'run', record: run });
        }
        for (const pair of [...eventPairs].sort((left, right) =>
            this.comparePairs(left, right),
        )) {
            const record = pair.record;
            const event = await this.materializeEvent(record);
            push({
                kind: 'event',
                task_id: event.task_id,
                run_id: event.run_id,
                generation: event.generation,
                sequence: event.sequence,
                record,
            });
        }
        for (const snapshot of snapshots) {
            push({ kind: 'snapshot', record: snapshot });
        }
        for (const change of changes) {
            push({ kind: 'changes', record: change });
        }
        const text = `${lines.join('\n')}\n`;
        if (byteLength(text) > MAX_EXPORT_BYTES) {
            throw new AgentStoreError(
                'limit_exceeded',
                'agent export exceeds max_export_bytes',
            );
        }
        return text;
    }

    async importNdjson(text: string): Promise<ImportSummary> {
        this.requireOpen();
        this.importClosing = true;
        this.opened = false;
        this.lastReport = null;
        let checkpoint = this.store.checkpoint();
        let fenceAcquired = false;
        let cleanupFailed = false;
        let cleanupError: unknown = null;
        let written = false;
        let tasks: TaskRecord[] = [];
        let runs: RunRecord[] = [];
        let events: EventPair[] = [];
        let snapshots = new Map<string, SnapshotBundleRecord>();
        let changes: ChangeRecord[] = [];
        let commentRecords: DiffCommentRecord[] = [];
        try {
        await this.drainWrites();
        checkpoint = this.store.checkpoint();
        this.importInProgress = true;
        await this.acquireImportFence(this.clock());
        fenceAcquired = true;
        this.startImportHeartbeat();
        if (this.importGate !== null) {
            const gate = this.importGate;
            this.importGate = null;
            await gate;
        }
        this.throwImportHeartbeatError();
        if (typeof text !== 'string' || byteLength(text) > MAX_EXPORT_BYTES) {
            throw new AgentStoreError('limit_exceeded', 'import exceeds max_export_bytes');
        }
        const rawLines = text.split('\n');
        while (rawLines.length > 0 && rawLines[rawLines.length - 1] === '') {
            rawLines.pop();
        }
        if (rawLines.length === 0) invalidArgument('import contains no NDJSON records');
        const lines = rawLines.map((line) => {
            if (byteLength(line) > MAX_EVENT_BYTES + 65536) {
                throw new AgentStoreError('limit_exceeded', 'import line exceeds its bound');
            }
            let canonical: string;
            try {
                canonical = parseCanonicalJson(line);
            } catch (error) {
                throw new AgentStoreError(
                    'snapshot_corrupt',
                    `import line is not canonical JSON: ${errorMessage(error)}`,
                );
            }
            if (canonical !== line) {
                corrupt('import line is canonicalizable but not in canonical form');
            }
            try {
                return JSON.parse(canonical) as Record<string, unknown>;
            } catch {
                corrupt('import line could not be materialized');
            }
        });
        const header = lines[0]!;
        if (header.kind !== 'header' || header.schema !== EXPORT_SCHEMA) {
            invalidArgument('import must begin with the agent export header');
        }
        tasks = [];
        runs = [];
        const snapshotRecords: SnapshotRecord[] = [];
        changes = [];
        events = [];
        const seenEventKeys = new Set<string>();
        const seenRunKeys = new Set<string>();
        const seenSnapshotIdentities = new Set<string>();
        const seenChangeKeys = new Set<string>();
        for (let index = 1; index < lines.length; index += 1) {
            const line = lines[index]!;
            if (line.kind === 'task') {
                const record = line.record;
                requireStoredTask(record);
                tasks.push(record);
            } else if (line.kind === 'run') {
                const record = line.record;
                requireStoredRun(record);
                const key = runKey(record.task_id, record.run_id);
                if (seenRunKeys.has(key)) corrupt('import repeats a run record');
                seenRunKeys.add(key);
                runs.push(record);
            } else if (line.kind === 'snapshot') {
                const record = line.record;
                requireStoredSnapshot(record);
                const identity = `${record.task_id}\u0000${record.run_id}\u0000${record.generation}\u0000${record.last_sequence}\u0000${record.last_event_id}\u0000${record.head_digest}`;
                if (seenSnapshotIdentities.has(identity)) {
                    corrupt('import repeats a snapshot identity');
                }
                seenSnapshotIdentities.add(identity);
                snapshotRecords.push(record);
            } else if (line.kind === 'changes') {
                const record = line.record;
                requireStoredChange(record);
                const key = `${record.task_id}\u0000${record.run_id}\u0000${record.head_digest}`;
                if (seenChangeKeys.has(key)) corrupt('import repeats a change record');
                seenChangeKeys.add(key);
                changes.push(record);
            } else if (line.kind === 'event') {
                const record = line.record;
                requireStoredEvent(record);
                const taskId = line.task_id;
                const runId = line.run_id;
                const generation = line.generation;
                const sequence = line.sequence;
                if (
                    typeof taskId !== 'string' ||
                    !TASK_ID_PATTERN.test(taskId) ||
                    typeof runId !== 'string' ||
                    !RUN_ID_PATTERN.test(runId) ||
                    !isPositiveInteger(generation) ||
                    !isNonNegativeInteger(sequence)
                ) {
                    corrupt('import event key is invalid');
                }
                const key = `${taskId}\u0000${runId}\u0000${generation}\u0000${sequence}`;
                if (seenEventKeys.has(key)) corrupt('import repeats an event key');
                seenEventKeys.add(key);
                events.push({
                    record,
                    key: [taskId, runId, generation, sequence],
                });
            } else {
                invalidArgument(`unknown import record kind ${String(line.kind)}`);
            }
        }

        const taskIds = new Set<string>();
        for (const task of tasks) {
            if (taskIds.has(task.task_id)) corrupt('import repeats a task record');
            taskIds.add(task.task_id);
        }
        const runMap = new Map(
            runs.map((run) => [runKey(run.task_id, run.run_id), run]),
        );
        for (const run of runs) {
            if (!taskIds.has(run.task_id)) {
                corrupt('import run record has no task record');
            }
        }
        const eventGroups = new Map<string, EventPair[]>();
        for (const event of events) {
            const [taskId, runId] = event.key as [string, string, number, number];
            const key = runKey(taskId, runId);
            if (!runMap.has(key)) {
                corrupt('import event belongs to a missing run record');
            }
            const group = eventGroups.get(key);
            if (group === undefined) {
                eventGroups.set(key, [event]);
            } else {
                group.push(event);
            }
        }
        const snapshotGroups = new Map<string, SnapshotRecord[]>();
        for (const snapshot of snapshotRecords) {
            const key = runKey(snapshot.task_id, snapshot.run_id);
            if (!runMap.has(key)) {
                corrupt('import snapshot belongs to a missing run record');
            }
            const group = snapshotGroups.get(key);
            if (group === undefined) {
                snapshotGroups.set(key, [snapshot]);
            } else {
                group.push(snapshot);
            }
        }
        snapshots = new Map<string, SnapshotBundleRecord>();
        for (const [key, records] of snapshotGroups) {
            const run = runMap.get(key)!;
            snapshots.set(
                key,
                bundleFromRecords(run.task_id, run.run_id, records),
            );
        }
        const input: ReplayInput = {
            tasks,
            runs,
            snapshots,
            events: eventGroups,
            orphanRuns: [],
            strict: true,
        };
        for (const bundle of snapshots.values()) {
            this.assertSnapshotBundleSize(bundle);
        }
        this.throwImportHeartbeatError();
        const scratch = createAgentStore();
        const scratchResult = await this.replayInto(scratch, input);
        commentRecords = scratchResult.comments;
        for (const change of changes) {
            const view = scratch.getRun(change.task_id, change.run_id);
            if (view === null) {
                corrupt('import change record belongs to a missing run record');
            }
            const event = view.events.find((candidate) => {
                const data = candidate.data as Record<string, unknown>;
                return (
                    candidate.type === 'changes.snapshot' &&
                    data.scope === 'last_turn' &&
                    data.head_digest === change.head_digest
                );
            });
            if (event === undefined) {
                corrupt('import change record has no matching changes.snapshot event');
            }
            let changeFiles: string;
            let eventFiles: string;
            try {
                changeFiles = canonicalJson(change.files);
                eventFiles = canonicalJson(
                    (event.data as Record<string, unknown>).files,
                );
            } catch {
                corrupt('import change files are not canonical JSON');
            }
            if (changeFiles !== eventFiles) {
                corrupt('import change files do not match the snapshot event');
            }
            change.files = jsonClean(
                (event.data as Record<string, unknown>).files as Array<
                    Record<string, unknown>
                >,
            );
        }

        await this.settleImportHeartbeat();
        this.throwImportHeartbeatError();
        const db = this.requireDb();
        const transaction = db.transaction([...AGENT_DB_STORES], 'readwrite');
        try {
            let fenceRecord: LeaseRecord | null = null;
            for (const name of AGENT_DB_STORES) {
                const store = transaction.objectStore(name);
                const count = await requestResult(store.count());
                if (count === 0) continue;
                if (name !== 'leases') {
                    throw new AgentStoreError(
                        'duplicate_event',
                        'import requires all nine agent stores to be empty',
                    );
                }
                const keys = await requestResult(store.getAllKeys());
                if (
                    keys.length !== 1 ||
                    keys[0] !== IMPORT_FENCE_KEY
                ) {
                    throw new AgentStoreError(
                        'duplicate_event',
                        'import requires all nine agent stores to be empty',
                    );
                }
                const fence = (await requestResult(
                    store.get(IMPORT_FENCE_KEY),
                )) as LeaseRecord | undefined;
                if (
                    fence === undefined ||
                    fence.writer_id !== this.importFenceWriter
                ) {
                    throw new AgentStoreError(
                        'stale_generation',
                        'import fence ownership changed before commit',
                    );
                }
                fenceRecord = fence;
            }
            if (fenceRecord === null) {
                throw new AgentStoreError(
                    'stale_generation',
                    'import fence is missing before commit',
                );
            }
            await requestResult(
                transaction.objectStore('leases').put(
                    jsonClean({
                        ...fenceRecord,
                        heartbeat_at_ms: this.clock(),
                    } satisfies LeaseRecord),
                    IMPORT_FENCE_KEY,
                ),
            );
            for (const task of tasks) {
                await requestResult(
                    transaction.objectStore('tasks').put(jsonClean(task), task.task_id),
                );
            }
            for (const run of runs) {
                await requestResult(
                    transaction
                        .objectStore('runs')
                        .put(jsonClean(run), [run.task_id, run.run_id]),
                );
            }
            for (const event of events) {
                await requestResult(
                    transaction
                        .objectStore('events')
                        .put(jsonClean(event.record), event.key),
                );
            }
            for (const [key, bundle] of snapshots) {
                const run = runMap.get(key)!;
                await requestResult(
                    transaction
                        .objectStore('snapshots')
                        .put(jsonClean(bundle), [run.task_id, run.run_id]),
                );
            }
            for (const change of changes) {
                await requestResult(
                    transaction.objectStore('changes').put(jsonClean(change), [
                        change.task_id,
                        change.run_id,
                        change.head_digest,
                    ]),
                );
            }
            for (const comment of commentRecords) {
                await requestResult(
                    transaction
                        .objectStore('comments')
                        .put(jsonClean(comment), diffCommentKey(comment)),
                );
            }
            await transactionDone(transaction);
            written = true;
        } catch (error) {
            try {
                transaction.abort();
            } catch {
                // The transaction may already be finished.
            }
            throw storageError(error);
        }

        if (this.faults.fail_import_publish) {
            this.faults.fail_import_publish = false;
            throw new AgentStoreError(
                'limit_exceeded',
                'injected import publish failure',
                'indexeddb',
            );
        }
        if (fenceAcquired) {
            await this.settleImportHeartbeat();
            this.takeImportHeartbeatError();
        }
        this.store.restore(scratch.checkpoint());
        this.lastReport = scratchResult.report;
        if (this.importPublishGate !== null) {
            const gate = this.importPublishGate;
            this.importPublishGate = null;
            const reached = this.importPublishGateReached;
            this.importPublishGateReached = null;
            reached?.();
            await gate;
        }
        if (fenceAcquired) {
            await this.releaseImportFence();
            fenceAcquired = false;
        }
        this.importInProgress = false;
        this.importClosing = false;
        this.opened = true;
        if (this.closeRequested) {
            this.closeRequested = false;
            this.db?.close();
            this.db = null;
            this.opened = false;
            this.lastReport = null;
        }
        return {
            tasks: tasks.length,
            runs: runs.length,
            events: events.length,
            snapshots: snapshotRecords.length,
            changes: changes.length,
        };
        } catch (error) {
            await this.settleImportHeartbeat();
            this.takeImportHeartbeatError();
            this.store.restore(checkpoint);
            if (written) {
                try {
                    await this.deleteImportedRecords(
                        tasks,
                        runs,
                        events,
                        snapshots,
                        changes,
                    );
                } catch (cleanupFailure) {
                    cleanupFailed = true;
                    cleanupError = cleanupFailure;
                }
            }
            if (fenceAcquired && !cleanupFailed) {
                try {
                    await this.releaseImportFence();
                } catch (releaseFailure) {
                    cleanupFailed = true;
                    cleanupError = releaseFailure;
                }
                fenceAcquired = false;
            }
            this.closeRequested = false;
            this.db?.close();
            this.db = null;
            this.importInProgress = false;
            this.importClosing = false;
            this.opened = false;
            this.lastReport = null;
            if (cleanupFailed) {
                throw storageError(cleanupError);
            }
            throw storageError(error);
        }
    }

    private async deleteImportedRecords(
        tasks: TaskRecord[],
        runs: RunRecord[],
        events: EventPair[],
        snapshots: Map<string, SnapshotBundleRecord>,
        changes: ChangeRecord[],
    ): Promise<void> {
        const db = this.requireDb();
        const transaction = db.transaction([...AGENT_DB_STORES], 'readwrite');
        try {
            for (const task of tasks) {
                await requestResult(
                    transaction.objectStore('tasks').delete(task.task_id),
                );
            }
            for (const run of runs) {
                await requestResult(
                    transaction
                        .objectStore('runs')
                        .delete([run.task_id, run.run_id]),
                );
            }
            for (const event of events) {
                await requestResult(
                    transaction.objectStore('events').delete(event.key),
                );
            }
            for (const key of snapshots.keys()) {
                const run = runs.find(
                    (candidate) => runKey(candidate.task_id, candidate.run_id) === key,
                );
                if (run !== undefined) {
                    await requestResult(
                        transaction
                            .objectStore('snapshots')
                            .delete([run.task_id, run.run_id]),
                    );
                }
            }
            for (const change of changes) {
                await requestResult(
                    transaction.objectStore('changes').delete([
                        change.task_id,
                        change.run_id,
                        change.head_digest,
                    ]),
                );
            }
            for (const name of [
                'artifacts',
                'approvals',
                'comments',
                'leases',
            ] as AgentDbStoreName[]) {
                await requestResult(transaction.objectStore(name).clear());
            }
            if (this.faults.fail_import_cleanup) {
                this.faults.fail_import_cleanup = false;
                transaction.abort();
            }
            await transactionDone(transaction);
        } catch (error) {
            try {
                transaction.abort();
            } catch {
                // The transaction may already be finished.
            }
            throw storageError(error);
        }
    }

    private requireOpen(): void {
        if (this.importInProgress || this.importClosing) {
            throw new AgentStoreError(
                'runtime_unavailable',
                'an import is in progress; the task store is unavailable',
            );
        }
        if (!this.opened || this.db === null) {
            throw new AgentStoreError(
                'runtime_unavailable',
                'durable task store is not open',
            );
        }
    }

    private requireDb(): IDBDatabase {
        if (this.db === null) {
            throw new AgentStoreError(
                'runtime_unavailable',
                'durable task store connection is unavailable',
            );
        }
        return this.db;
    }

    private requireWriter(writerId: string): void {
        if (!isNonEmptyString(writerId) || writerId.length > 128) {
            invalidArgument('writer_id must be a non-empty string');
        }
    }

    private async checkImportFence(
        leases: IDBObjectStore,
        now: number,
    ): Promise<void> {
        const fence = (await requestResult(
            leases.get(IMPORT_FENCE_KEY),
        )) as LeaseRecord | undefined;
        if (fence === undefined) return;
        requireStoredLease(fence);
        if (
            fence.writer_id !== this.importFenceWriter &&
            now - fence.heartbeat_at_ms <= LEASE_STALE_MS
        ) {
            throw new AgentStoreError(
                'stale_generation',
                'an import fence is held by another writer',
            );
        }
    }

    private async renewImportFence(now: number): Promise<void> {
        if (this.db === null) {
            throw new AgentStoreError(
                'runtime_unavailable',
                'durable task store connection is unavailable',
            );
        }
        const transaction = this.db.transaction('leases', 'readwrite');
        try {
            const leases = transaction.objectStore('leases');
            const existing = (await requestResult(
                leases.get(IMPORT_FENCE_KEY),
            )) as LeaseRecord | undefined;
            if (
                existing === undefined ||
                existing.writer_id !== this.importFenceWriter
            ) {
                throw new AgentStoreError(
                    'stale_generation',
                    'import fence ownership changed before its heartbeat',
                );
            }
            requireStoredLease(existing);
            await requestResult(
                leases.put(
                    jsonClean({
                        ...existing,
                        heartbeat_at_ms: now,
                    } satisfies LeaseRecord),
                    IMPORT_FENCE_KEY,
                ),
            );
            await transactionDone(transaction);
        } catch (error) {
            try {
                transaction.abort();
            } catch {
                // The transaction may already be finished.
            }
            throw storageError(error);
        }
    }

    private async acquireImportFence(now: number): Promise<void> {
        const db = this.requireDb();
        const transaction = db.transaction('leases', 'readwrite');
        try {
            const leases = transaction.objectStore('leases');
            const existing = (await requestResult(
                leases.get(IMPORT_FENCE_KEY),
            )) as LeaseRecord | undefined;
            if (existing !== undefined) {
                requireStoredLease(existing);
                if (
                    existing.writer_id !== this.importFenceWriter &&
                    now - existing.heartbeat_at_ms <= LEASE_STALE_MS
                ) {
                    throw new AgentStoreError(
                        'stale_generation',
                        'an import fence is held by another writer',
                    );
                }
            }
            const record: LeaseRecord = {
                task_id: IMPORT_FENCE_KEY,
                writer_id: this.importFenceWriter,
                started_at_ms:
                    existing !== undefined &&
                    existing.writer_id === this.importFenceWriter
                        ? existing.started_at_ms
                        : now,
                heartbeat_at_ms: now,
            };
            await requestResult(leases.put(jsonClean(record), IMPORT_FENCE_KEY));
            await transactionDone(transaction);
        } catch (error) {
            try {
                transaction.abort();
            } catch {
                // The transaction may already be finished.
            }
            throw storageError(error);
        }
    }

    private async releaseImportFence(): Promise<void> {
        if (this.db === null) return;
        const transaction = this.db.transaction('leases', 'readwrite');
        try {
            const leases = transaction.objectStore('leases');
            const existing = (await requestResult(
                leases.get(IMPORT_FENCE_KEY),
            )) as LeaseRecord | undefined;
            if (
                existing !== undefined &&
                existing.writer_id === this.importFenceWriter
            ) {
                await requestResult(leases.delete(IMPORT_FENCE_KEY));
            }
            await transactionDone(transaction);
        } catch (error) {
            try {
                transaction.abort();
            } catch {
                // The transaction may already be finished.
            }
            throw storageError(error);
        }
    }

    private async restore(): Promise<OpenReport> {
        const db = await openAgentDatabase(this.factory);
        this.db = db;
        const fence = await this.readOne<LeaseRecord>(
            'leases',
            IMPORT_FENCE_KEY,
            db,
        );
        if (fence !== undefined) {
            db.close();
            this.db = null;
            throw new AgentStoreError(
                'runtime_unavailable',
                'an unfinished import fence blocks this database; reset before opening',
            );
        }
        db.onversionchange = () => {
            db.close();
            if (this.db === db) {
                this.db = null;
                this.opened = false;
                this.openPromise = null;
                this.lastReport = null;
            }
        };
        const checkpoint = this.store.checkpoint();
        this.store.reset();
        try {
            const taskPairs = await this.readAllPairs<TaskRecord>('tasks', db);
            const runPairs = await this.readAllPairs<RunRecord>('runs', db);
            const snapshotPairs = await this.readAllPairs<StoredSnapshotRecord>(
                'snapshots',
                db,
            );
            const taskRecords = taskPairs.map(({ key, record }) => {
                requireStoredTask(record);
                if (key !== record.task_id) {
                    corrupt('task record key does not match its task_id');
                }
                return record;
            });
            const runRecords = runPairs.map(({ key, record }) => {
                requireStoredRun(record);
                if (
                    !Array.isArray(key) ||
                    key.length !== 2 ||
                    key[0] !== record.task_id ||
                    key[1] !== record.run_id
                ) {
                    corrupt('run record key does not match its task/run identity');
                }
                return record;
            });
            const runKeys = new Set(
                runRecords.map((run) => runKey(run.task_id, run.run_id)),
            );
            const snapshots = new Map<string, SnapshotBundleRecord>();
            for (const { key, record } of snapshotPairs) {
                const bundle = normalizeSnapshotBundle(record);
                if (
                    !Array.isArray(key) ||
                    key.length !== 2 ||
                    key[0] !== bundle.task_id ||
                    key[1] !== bundle.run_id
                ) {
                    corrupt('snapshot record key does not match its task/run identity');
                }
                const composite = runKey(bundle.task_id, bundle.run_id);
                if (!runKeys.has(composite)) {
                    corrupt('snapshot record exists without a run record');
                }
                snapshots.set(composite, bundle);
            }
            const eventKeys = await this.readAllKeys('events', db);
            const orphanRuns: Array<{ taskId: string; runId: string }> = [];
            const seenOrphans = new Set<string>();
            for (const key of eventKeys) {
                if (!Array.isArray(key) || key.length !== 4) {
                    corrupt('event store key is not [task_id, run_id, generation, sequence]');
                }
                if (
                    typeof key[0] !== 'string' ||
                    typeof key[1] !== 'string' ||
                    typeof key[2] !== 'number' ||
                    typeof key[3] !== 'number'
                ) {
                    corrupt('event store key fields are invalid');
                }
                const composite = runKey(key[0], key[1]);
                if (!runKeys.has(composite) && !seenOrphans.has(composite)) {
                    seenOrphans.add(composite);
                    orphanRuns.push({ taskId: key[0], runId: key[1] });
                }
            }
            const events = new Map<string, EventPair[]>();
            for (const run of runRecords) {
                events.set(
                    runKey(run.task_id, run.run_id),
                    await this.readEventPairs(run.task_id, run.run_id, db),
                );
            }
            const result = await this.replayInto(this.store, {
                tasks: taskRecords,
                runs: runRecords,
                snapshots,
                events,
                orphanRuns,
                strict: false,
            });
            await this.replaceCommentProjection(result.comments, db);
            return result.report;
        } catch (error) {
            this.store.restore(checkpoint);
            db.close();
            this.db = null;
            throw storageError(error);
        }
    }

    private async replaceCommentProjection(
        records: DiffCommentRecord[],
        db?: IDBDatabase,
    ): Promise<void> {
        const database = db ?? this.requireDb();
        const transaction = database.transaction('comments', 'readwrite');
        try {
            const store = transaction.objectStore('comments');
            await requestResult(store.clear());
            for (const record of records) {
                await requestResult(
                    store.put(jsonClean(record), diffCommentKey(record)),
                );
            }
            await transactionDone(transaction);
        } catch (error) {
            try { transaction.abort(); } catch { /* already finished */ }
            throw storageError(error);
        }
    }

    private async replayInto(
        target: AgentStore,
        input: ReplayInput,
    ): Promise<ReplayResult> {        const taskIds = new Set<string>();
        for (const task of input.tasks) {
            if (taskIds.has(task.task_id)) {
                corrupt('duplicate task record in replay input');
            }
            taskIds.add(task.task_id);
        }
        const runKeys = new Set<string>();
        for (const run of input.runs) {
            const composite = runKey(run.task_id, run.run_id);
            if (runKeys.has(composite)) {
                corrupt('duplicate run record in replay input');
            }
            runKeys.add(composite);
            if (!taskIds.has(run.task_id)) {
                corrupt('run record has no task record');
            }
        }
        for (const key of input.events.keys()) {
            if (!runKeys.has(key)) {
                if (input.strict) {
                    corrupt('event records exist without a run record');
                }
            }
        }
        const incomplete: RestoreIncomplete[] = [];
        const replayedComments = new Map<string, DiffCommentRecord>();
        const addIncomplete = (entry: RestoreIncomplete) => {
            if (
                !incomplete.some(
                    (existing) =>
                        existing.task_id === entry.task_id &&
                        existing.run_id === entry.run_id &&
                        existing.code === entry.code,
                )
            ) {
                incomplete.push(entry);
            }
        };
        for (const orphan of input.orphanRuns) {
            if (input.strict) {
                corrupt('event records exist without a run record');
            }
            addIncomplete({
                task_id: orphan.taskId,
                run_id: orphan.runId,
                code: 'snapshot_missing',
                message: 'event records exist without a run record',
            });
        }
        const tasks = [...input.tasks].sort((left, right) =>
            left.created_at_ms === right.created_at_ms
                ? left.task_id.localeCompare(right.task_id)
                : left.created_at_ms - right.created_at_ms,
        );
        for (const task of tasks) {
            target.upsertTask({
                ...taskToAgentRecord(task),
                archived: false,
                pinnedLocal: false,
                latestRunId: null,
            });
        }
        const runsByTask = new Map<string, RunRecord[]>();
        for (const run of input.runs) {
            const existing = runsByTask.get(run.task_id);
            if (existing === undefined) {
                runsByTask.set(run.task_id, [run]);
            } else {
                existing.push(run);
            }
        }
        const replayed: RunRecord[] = [];
        const replayedKeys = new Set<string>();
        for (const task of tasks) {
            const taskRuns = (runsByTask.get(task.task_id) ?? []).sort(
                (left, right) =>
                    left.generation - right.generation ||
                    left.started_at_ms - right.started_at_ms ||
                    left.run_id.localeCompare(right.run_id),
            );
            for (const run of taskRuns) {
                const composite = runKey(run.task_id, run.run_id);
                const pairs = input.events.get(composite) ?? [];
                if (pairs.length === 0) {
                    if (input.strict) {
                        corrupt('run record has no canonical event records');
                    }
                    addIncomplete({
                        task_id: run.task_id,
                        run_id: run.run_id,
                        code: 'snapshot_missing',
                        message: 'run record has no canonical event records',
                    });
                    continue;
                }
                const runComments = await this.verifyAndReplay(
                    target,
                    run,
                    pairs,
                    input.snapshots.get(composite) ?? null,
                );
                for (const comment of runComments) {
                    replayedComments.set(
                        diffCommentKey(comment).join('\u0000'),
                        comment,
                    );
                }
                replayed.push(run);
                replayedKeys.add(composite);
            }
            const latest = task.latest_run_id ?? null;
            const latestExists =
                latest !== null &&
                replayedKeys.has(runKey(task.task_id, latest));
            const replayedForTask = [...replayedKeys].some((key) =>
                key.startsWith(`${task.task_id}\u0000`),
            );
            const derivedLatest = target.getTaskLatestRunId(task.task_id);
            if (latest !== null && !latestExists) {
                if (input.strict) {
                    corrupt('latest_run_id has no replayed run');
                }
                addIncomplete({
                    task_id: task.task_id,
                    run_id: latest,
                    code: 'snapshot_missing',
                    message: 'latest_run_id has no replayed run',
                });
            } else if (latest !== null && derivedLatest !== latest) {
                corrupt(
                    'stored latest_run_id does not match the replayed run order',
                );
            } else if (latest === null && replayedForTask) {
                corrupt('stored latest_run_id is null with a replayed run');
            }
            target.upsertTask({
                ...taskToAgentRecord(task),
                archived: task.archived,
                pinnedLocal: task.pinned_local,
                latestRunId: latestExists ? latest : null,
            });
            target.setTaskUnresolvedRunId(
                task.task_id,
                latest !== null && !latestExists ? latest : null,
            );
        }
        const snapshot = target.snapshot();
        return {
            report: {
                tasks: tasks.map((task) => {
                    const view = snapshot.tasks.find(
                        (candidate) => candidate.taskId === task.task_id,
                    );
                    return {
                        ...task,
                        latest_run_id: view?.latestRunId ?? null,
                    };
                }),
                runs: replayed,
                incomplete,
            },
            replayed,
            comments: [...replayedComments.values()],
        };
    }

    private async verifyAndReplay(
        target: AgentStore,
        run: RunRecord,
        pairs: EventPair[],
        bundle: SnapshotBundleRecord | null,
    ): Promise<DiffCommentRecord[]> {
        const verified = await this.verifyEventChain(run.task_id, run.run_id, pairs);
        if (bundle !== null) this.assertSnapshotBundleSize(bundle);
        const lastTurnDigests = lastTurnDigestsOf(verified.events);
        const comments = new Map<string, DiffCommentRecord>();
        const anchors = bundle === null ? [] : snapshotRecordsOf(bundle);
        const matched = new Set<SnapshotRecord>();
        const verifyAnchor = (anchor: SnapshotRecord): void => {
            const check = target.verifySnapshot(snapshotVerification(anchor));
            if (!check.ok) {
                throw new AgentStoreError(
                    check.code ?? 'snapshot_corrupt',
                    check.message ?? 'snapshot verification failed',
                );
            }
            matched.add(anchor);
        };
        for (let index = 0; index < verified.events.length; index += 1) {
            const event = verified.events[index]!;
            const digest = verified.digests[index]!;
            if (event.type === 'diff.comment') {
                const record = diffCommentRecordFromEvent(event, lastTurnDigests);
                comments.set(diffCommentKey(record).join('\u0000'), record);
                continue;
            }
            if (event.type === 'run.resumed') {
                const current = target.getRun(run.task_id, run.run_id);
                if (current === null) {
                    corrupt('replayed run is missing before run.resumed');
                }
                const data = event.data as Record<string, unknown>;
                const anchor = anchors.find(
                    (candidate) =>
                        candidate.generation === current.generation &&
                        candidate.last_sequence === current.lastSequence &&
                        candidate.last_event_id === current.lastEventId &&
                        candidate.head_digest === data.snapshot_digest,
                );
                if (anchor === undefined) {
                    throw new AgentStoreError(
                        'snapshot_missing',
                        'run.resumed requires its persisted pre-resume snapshot anchor',
                    );
                }
                verifyAnchor(anchor);
            }
            const applied = target.apply(event, digest);
            if (applied.status !== 'applied') {
                throw new AgentStoreError(
                    'snapshot_corrupt',
                    `replay rejected ${event.event_id}: ${applied.code ?? 'unknown'} ${applied.message ?? ''}`,
                );
            }
            for (const anchor of anchors) {
                if (snapshotMatchesEvent(anchor, event, digest)) {
                    verifyAnchor(anchor);
                }
            }
        }
        if (bundle !== null && !matched.has(bundle.latest)) {
            corrupt('latest snapshot position is not part of the replayed chain');
        }
        for (const anchor of anchors) {
            if (!matched.has(anchor)) {
                corrupt('snapshot anchor position is not part of the replayed chain');
            }
        }
        const view = target.getRun(run.task_id, run.run_id);
        if (view === null) {
            corrupt('replayed run is missing from AgentStore');
        }
        this.assertRunRecordMatches(run, verified, view);
        return [...comments.values()];
    }

    private assertRunRecordMatches(
        run: RunRecord,
        verified: { events: AgentIncomingEvent[]; digests: string[] },
        view?: RunView,
    ): void {
        const count = verified.events.length;
        const lastEvent = verified.events[count - 1]!;
        const lastDigest = verified.digests[count - 1]!;
        if (
            run.generation !== lastEvent.generation ||
            run.last_sequence !== lastEvent.sequence ||
            run.last_event_id !== lastEvent.event_id ||
            run.last_digest !== lastDigest
        ) {
            corrupt('run record cursor does not match the verified event chain');
        }
        if (view !== undefined) {
            let stateIndex = -1;
            for (let index = count - 1; index >= 0; index -= 1) {
                if (verified.events[index]!.type !== 'diff.comment') {
                    stateIndex = index;
                    break;
                }
            }
            if (stateIndex < 0) {
                corrupt('run has no state transition before its comment events');
            }
            const stateEvent = verified.events[stateIndex]!;
            const stateDigest = verified.digests[stateIndex]!;
            if (
                run.generation !== stateEvent.generation ||
                run.state !== view.state ||
                stateEvent.sequence !== view.lastSequence ||
                stateEvent.event_id !== view.lastEventId ||
                stateDigest !== view.lastDigest
            ) {
                corrupt('run record does not match the replayed run');
            }
        }
    }

    private async verifyEventChain(
        taskId: string,
        runId: string,
        pairs: EventPair[],
    ): Promise<{ events: AgentIncomingEvent[]; digests: string[] }> {
        const events: AgentIncomingEvent[] = [];
        const digests: string[] = [];
        const seen = new Set<string>();
        let previousDigest: string | null = null;
        let expectedSequence = 0;
        let generation: number | null = null;
        for (let index = 0; index < pairs.length; index += 1) {
            const pair = pairs[index]!;
            const key = pair.key;
            if (!Array.isArray(key) || key.length !== 4) {
                corrupt('event store key is not [task_id, run_id, generation, sequence]');
            }
            if (
                typeof key[0] !== 'string' ||
                typeof key[1] !== 'string' ||
                typeof key[2] !== 'number' ||
                typeof key[3] !== 'number'
            ) {
                corrupt('event store key fields are invalid');
            }
            requireStoredEvent(pair.record);
            const event = await this.materializeEvent(pair.record);
            if (event.task_id !== taskId || event.run_id !== runId) {
                corrupt('event identity does not match its per-run key');
            }
            if (event.generation !== key[2] || event.sequence !== key[3]) {
                corrupt('event generation or sequence does not match its key');
            }
            if (seen.has(event.event_id)) {
                corrupt('duplicate event_id in the per-run log');
            }
            seen.add(event.event_id);
            if (events.length === 0) {
                if (pair.record.prev_digest !== null) {
                    corrupt('first event must have a null prev_digest');
                }
                if (event.sequence !== 0 || event.type !== 'run.queued') {
                    corrupt('first event must be run.queued at sequence 0');
                }
                generation = event.generation;
            } else {
                if (pair.record.prev_digest !== previousDigest) {
                    corrupt('event prev_digest does not link to the prior event');
                }
                if (event.sequence !== expectedSequence) {
                    corrupt('event sequence is not contiguous');
                }
                if (event.generation !== generation) {
                    if (
                        generation === null ||
                        event.generation !== generation + 1 ||
                        event.type !== 'run.resumed'
                    ) {
                        corrupt('generation changes require run.resumed');
                    }
                    generation = event.generation;
                }
            }
            previousDigest = pair.record.digest;
            expectedSequence = event.sequence + 1;
            events.push(event);
            digests.push(pair.record.digest);
            if (events.length > MAX_EVENTS_PER_RUN) {
                throw new AgentStoreError('limit_exceeded', 'run exceeds max_events_per_run');
            }
        }
        return { events, digests };
    }

    private assertSnapshotBundleSize(bundle: SnapshotBundleRecord): void {
        if (snapshotBundleSize(bundle) > MAX_SNAPSHOT_BYTES) {
            throw new AgentStoreError(
                'limit_exceeded',
                'snapshot bundle exceeds max_snapshot_bytes',
                'indexeddb',
            );
        }
    }

    private verifySnapshotBundleAgainstChain(
        bundle: SnapshotBundleRecord,
        verified: { events: AgentIncomingEvent[]; digests: string[] },
    ): void {
        this.assertSnapshotBundleSize(bundle);
        for (const record of snapshotRecordsOf(bundle)) {
            const index = verified.events.findIndex(
                (event) =>
                    event.generation === record.generation &&
                    event.sequence === record.last_sequence,
            );
            if (
                index < 0 ||
                verified.events[index]!.event_id !== record.last_event_id ||
                verified.digests[index] !== record.head_digest
            ) {
                corrupt('snapshot anchor position does not match the event chain');
            }
        }
    }

    private async verifyExistingSnapshotForRun(
        taskId: string,
        runId: string,
    ): Promise<void> {
        const stored = (await this.readOne<StoredSnapshotRecord>(
            'snapshots',
            [taskId, runId],
        )) as StoredSnapshotRecord | undefined;
        if (stored === undefined) return;
        const bundle = normalizeSnapshotBundle(stored);
        const pairs = await this.readEventPairs(taskId, runId);
        const verified = await this.verifyEventChain(taskId, runId, pairs);
        this.verifySnapshotBundleAgainstChain(bundle, verified);
    }

    // Ordinary appends verify every persisted latest/anchor at its exact event
    // position without re-reading the whole chain; the append path keeps the
    // chain verified incrementally.
    private async verifyExistingSnapshotAnchors(
        taskId: string,
        runId: string,
    ): Promise<void> {
        const stored = (await this.readOne<StoredSnapshotRecord>(
            'snapshots',
            [taskId, runId],
        )) as StoredSnapshotRecord | undefined;
        if (stored === undefined) return;
        const bundle = normalizeSnapshotBundle(stored);
        this.assertSnapshotBundleSize(bundle);
        for (const record of snapshotRecordsOf(bundle)) {
            const event = (await this.readOne<EventRecord>('events', [
                taskId,
                runId,
                record.generation,
                record.last_sequence,
            ])) as EventRecord | undefined;
            if (
                event === undefined ||
                event.event_id !== record.last_event_id ||
                event.digest !== record.head_digest
            ) {
                corrupt('snapshot anchor does not match the stored event record');
            }
        }
    }

    private async materializeEvent(record: EventRecord): Promise<AgentIncomingEvent> {
        if (record.prev_digest !== null && !DIGEST_PATTERN.test(record.prev_digest)) {
            corrupt('event prev_digest is not a SHA-256 digest');
        }
        let canonical: string;
        try {
            canonical = parseCanonicalJson(record.canonical);
        } catch (error) {
            corrupt(`event canonical text is not canonical JSON: ${errorMessage(error)}`);
        }
        if (canonical !== record.canonical) {
            corrupt('event canonical text is not canonical');
        }
        let value: unknown;
        try {
            value = JSON.parse(canonical);
        } catch {
            corrupt('event canonical text could not be materialized');
        }
        const validation = validateAgentEvent(value);
        if (!validation.ok) {
            corrupt(`event violates the contract: ${validation.code} ${validation.message}`);
        }
        const event = validation.event;
        if (record.event_id !== event.event_id) {
            corrupt('event_id does not match the canonical event');
        }
        const digest = await digestEvent(event as AgentEvent);
        if (digest !== record.digest) {
            corrupt('event digest does not match the recomputed canonical digest');
        }
        return event;
    }

    private async commitEvent(
        event: AgentIncomingEvent,
        canonical: string,
        digest: string,
        input: AppendEventStoreInput,
        now: number,
    ): Promise<void> {
        if (this.importInProgress) {
            throw new AgentStoreError(
                'runtime_unavailable',
                'an import is in progress; append commit is unavailable',
            );
        }
        const db = this.requireDb();
        const transaction = db.transaction(
            ['tasks', 'runs', 'events', 'snapshots', 'leases'],
            'readwrite',
        );
        try {
            const tasks = transaction.objectStore('tasks');
            const runs = transaction.objectStore('runs');
            const events = transaction.objectStore('events');
            const snapshots = transaction.objectStore('snapshots');
            const leases = transaction.objectStore('leases');
            await this.checkImportFence(leases, now);

            const lease = (await requestResult(leases.get(event.task_id))) as
                | LeaseRecord
                | undefined;
            if (lease !== undefined) {
                requireStoredLease(lease);
                if (
                    lease.writer_id !== input.writer_id &&
                    now - lease.heartbeat_at_ms <= LEASE_STALE_MS
                ) {
                    throw new AgentStoreError(
                        'stale_generation',
                        'task lease is held by another active writer',
                    );
                }
            }

            const existing = (await requestResult(
                events.get([event.task_id, event.run_id, event.generation, event.sequence]),
            )) as EventRecord | undefined;
            if (existing !== undefined && existing.digest !== digest) {
                throw new AgentStoreError(
                    'duplicate_event',
                    'same event key already holds a different digest',
                );
            }
            const storedRun = (await requestResult(
                runs.get([event.task_id, event.run_id]),
            )) as RunRecord | undefined;
            if (storedRun !== undefined) {
                if (event.sequence !== storedRun.last_sequence + 1) {
                    throw new AgentStoreError(
                        'out_of_order',
                        'event sequence does not follow the durable run cursor',
                    );
                }
            } else {
                if (event.type !== 'run.queued' || event.sequence !== 0) {
                    throw new AgentStoreError(
                        'snapshot_missing',
                        'durable run record is missing for a non-initial event',
                    );
                }
                // Independent defense: a new run may only follow the durable
                // task's current latest run when that run is absent or
                // terminal, and its generation must be strictly newer. This
                // rejects a stale writer's colliding generation even if it
                // managed to acquire a lease.
                const durableTask = (await requestResult(
                    tasks.get(event.task_id),
                )) as TaskRecord | undefined;
                if (durableTask !== undefined) {
                    requireStoredTask(durableTask);
                    const durableLatest = durableTask.latest_run_id ?? null;
                    if (durableLatest !== null) {
                        const durableLatestRun = (await requestResult(
                            runs.get([event.task_id, durableLatest]),
                        )) as RunRecord | undefined;
                        if (durableLatestRun === undefined) {
                            throw new AgentStoreError(
                                'snapshot_missing',
                                'durable task latest run record is missing',
                            );
                        }
                        requireStoredRun(durableLatestRun);
                        if (
                            !TERMINAL_RUN_STATES.has(durableLatestRun.state) ||
                            durableLatestRun.generation >= event.generation
                        ) {
                            throw new AgentStoreError(
                                'stale_generation',
                                'new run collides with the durable task latest run',
                            );
                        }
                    }
                }
            }
            let anchorRecord: SnapshotRecord | undefined;
            if (input.snapshot_anchor !== undefined) {
                if (event.type !== 'run.resumed') {
                    throw new AgentStoreError(
                        'invalid_arguments',
                        'snapshot_anchor is only valid with run.resumed',
                    );
                }
                if (storedRun === undefined) {
                    throw new AgentStoreError(
                        'snapshot_missing',
                        'run.resumed has no durable pre-resume run record',
                    );
                }
                const anchor = input.snapshot_anchor;
                if (
                    anchor.task_id !== event.task_id ||
                    anchor.run_id !== event.run_id ||
                    anchor.generation !== storedRun.generation ||
                    anchor.last_sequence !== storedRun.last_sequence ||
                    anchor.last_event_id !== storedRun.last_event_id ||
                    anchor.head_digest !== storedRun.last_digest
                ) {
                    throw new AgentStoreError(
                        'snapshot_corrupt',
                        'pre-resume anchor does not match the durable run cursor',
                    );
                }
                anchorRecord = {
                    schema: SNAPSHOT_SCHEMA,
                    task_id: anchor.task_id,
                    run_id: anchor.run_id,
                    generation: anchor.generation,
                    last_event_id: anchor.last_event_id,
                    last_sequence: anchor.last_sequence,
                    head_digest: anchor.head_digest,
                    created_at_ms: anchor.created_at_ms ?? now,
                };
                requireStoredSnapshot(anchorRecord);
            } else if (event.type === 'run.resumed') {
                throw new AgentStoreError(
                    'snapshot_missing',
                    'run.resumed requires a pre-resume snapshot anchor',
                );
            }
            const view = this.store.getRunRecordState(event.task_id, event.run_id);
            if (view === null) {
                throw new AgentStoreError('internal', 'applied run is missing from AgentStore');
            }
            const latestRunId =
                this.store.getTaskLatestRunId(event.task_id) ?? event.run_id;
            const runRecord = this.buildRunRecordFromState(
                event.task_id,
                event.run_id,
                view,
            );
            await requestResult(
                events.put(
                    jsonClean({
                        event_id: event.event_id,
                        canonical,
                        digest,
                        prev_digest: storedRun?.last_digest ?? null,
                        recorded_at_ms: now,
                    } satisfies EventRecord),
                    [event.task_id, event.run_id, event.generation, event.sequence],
                ),
            );
            await requestResult(
                runs.put(jsonClean(runRecord), [event.task_id, event.run_id]),
            );
            const task = (await requestResult(tasks.get(event.task_id))) as
                | TaskRecord
                | undefined;
            if (task === undefined) {
                await requestResult(
                    tasks.put(
                        jsonClean({
                            task_id: event.task_id,
                            project_id: event.project_id,
                            title: event.task_id,
                            created_at_ms: event.recorded_at_ms,
                            archived: false,
                            pinned_local: false,
                            latest_run_id: latestRunId,
                        } satisfies TaskRecord),
                        event.task_id,
                    ),
                );
            } else {
                requireStoredTask(task);
                await requestResult(
                    tasks.put(
                        jsonClean({
                            ...task,
                            latest_run_id: latestRunId,
                        } satisfies TaskRecord),
                        event.task_id,
                    ),
                );
            }
            if (anchorRecord !== undefined || input.snapshot === true) {
                const existingSnapshot = (await requestResult(
                    snapshots.get([event.task_id, event.run_id]),
                )) as StoredSnapshotRecord | undefined;
                let bundle =
                    existingSnapshot === undefined
                        ? undefined
                        : normalizeSnapshotBundle(existingSnapshot);
                if (anchorRecord !== undefined) {
                    bundle = mergeSnapshotAnchor(bundle, anchorRecord);
                }
                if (input.snapshot === true) {
                    bundle = mergeSnapshotAnchor(
                        bundle,
                        this.buildSnapshotFromState(event.task_id, event.run_id, view),
                    );
                }
                if (bundle !== undefined) {
                    this.assertSnapshotBundleSize(bundle);
                    await requestResult(
                        snapshots.put(jsonClean(bundle), [
                            event.task_id,
                            event.run_id,
                        ]),
                    );
                }
            }
            if (this.faults.abort_next_append) {
                this.faults.abort_next_append = false;
                transaction.abort();
            }
            await transactionDone(transaction);
        } catch (error) {
            try {
                transaction.abort();
            } catch {
                // The transaction may already be finished.
            }
            throw storageError(error);
        }
    }

    private buildRunRecordFromState(
        taskId: string,
        runId: string,
        view: {
            generation: number;
            state: RunRecord['state'];
            retryOfRunId: string | null;
            startedAtMs: number | null;
            firstRecordedAtMs: number | null;
            endedAtMs: number | null;
            lastSequence: number;
            lastEventId: string | null;
            lastDigest: string | null;
        },
    ): RunRecord {
        return {
            task_id: taskId,
            run_id: runId,
            generation: view.generation,
            state: view.state,
            retry_of_run_id: view.retryOfRunId,
            started_at_ms:
                view.startedAtMs ?? view.firstRecordedAtMs ?? 0,
            ...(view.endedAtMs === null ? {} : { ended_at_ms: view.endedAtMs }),
            last_sequence: view.lastSequence,
            last_event_id: view.lastEventId,
            last_digest: view.lastDigest,
        };
    }

    private buildSnapshotFromState(
        taskId: string,
        runId: string,
        view: {
            generation: number;
            lastEventId: string | null;
            lastSequence: number;
            lastDigest: string | null;
        },
    ): SnapshotRecord {
        if (view.lastEventId === null || view.lastDigest === null) {
            throw new AgentStoreError(
                'snapshot_missing',
                'run has no verified event to snapshot',
            );
        }
        return {
            schema: SNAPSHOT_SCHEMA,
            task_id: taskId,
            run_id: runId,
            generation: view.generation,
            last_event_id: view.lastEventId,
            last_sequence: view.lastSequence,
            head_digest: view.lastDigest,
            created_at_ms: this.clock(),
        };
    }

    private buildSnapshot(view: RunView): SnapshotRecord {
        if (view.lastEventId === null || view.lastDigest === null) {
            throw new AgentStoreError(
                'snapshot_missing',
                'run has no verified event to snapshot',
            );
        }
        return {
            schema: SNAPSHOT_SCHEMA,
            task_id: view.taskId,
            run_id: view.runId,
            generation: view.generation,
            last_event_id: view.lastEventId,
            last_sequence: view.lastSequence,
            head_digest: view.lastDigest,
            created_at_ms: this.clock(),
        };
    }

    private async persistTaskWithRollback(record: TaskRecord): Promise<void> {
        const checkpoint = this.store.captureTask(record.task_id);
        try {
            this.store.upsertTask(taskToAgentRecord(record));
            if (this.taskWriteGate !== null) {
                const gate = this.taskWriteGate;
                this.taskWriteGate = null;
                const reached = this.taskWriteGateReached;
                this.taskWriteGateReached = null;
                reached?.();
                await gate;
            }
            await this.putRecord('tasks', jsonClean(record), record.task_id);
        } catch (error) {
            this.store.restoreTask(checkpoint);
            throw storageError(error);
        }
    }

    private async updateTaskFlag(
        taskId: string,
        mutate: (record: TaskRecord) => void,
    ): Promise<TaskRecord> {
        this.requireOpen();
        if (!TASK_ID_PATTERN.test(taskId)) invalidArgument('task_id is invalid');
        this.beginWrite();
        try {
        const stored = (await this.readOne<TaskRecord>('tasks', taskId)) as
            | TaskRecord
            | undefined;
        if (stored === undefined) {
            throw new AgentStoreError('task_not_found', 'task record is missing');
        }
        requireStoredTask(stored);
        if (!this.store.hasTask(taskId)) {
            throw new AgentStoreError('task_not_found', 'task is not present in AgentStore');
        }
        const record: TaskRecord = {
            ...jsonClean(stored),
            latest_run_id: this.store.getTaskLatestRunId(taskId),
        };
        const marker = this.store.getTaskUnresolvedRunId(taskId);
        mutate(record);
        const checkpoint = this.store.captureTask(taskId);
        try {
            this.store.upsertTask(taskToAgentRecord(record));
            await this.putRecord('tasks', jsonClean(record), taskId);
        } catch (error) {
            this.store.restoreTask(checkpoint);
            throw storageError(error);
        }
        if (
            marker !== null &&
            this.store.getRunCursor(taskId, marker) === null
        ) {
            this.store.setTaskUnresolvedRunId(taskId, marker);
        }
        return record;
        } finally {
            this.endWrite();
        }
    }

    private async putRecord(
        storeName: AgentDbStoreName,
        value: unknown,
        key: IDBValidKey,
    ): Promise<void> {
        if (this.importInProgress) {
            throw new AgentStoreError(
                'runtime_unavailable',
                'an import is in progress; task writes are unavailable',
            );
        }
        const transaction = this.requireDb().transaction([storeName, 'leases'], 'readwrite');
        try {
            await this.checkImportFence(
                transaction.objectStore('leases'),
                this.clock(),
            );
            await requestResult(
                transaction.objectStore(storeName).put(jsonClean(value), key),
            );
            if (this.faults.abort_next_task_write) {
                this.faults.abort_next_task_write = false;
                transaction.abort();
            }
            await transactionDone(transaction);
        } catch (error) {
            try {
                transaction.abort();
            } catch {
                // The transaction may already be finished.
            }
            throw storageError(error);
        }
    }

    private async readOne<T>(
        storeName: AgentDbStoreName,
        key: IDBValidKey,
        db?: IDBDatabase,
    ): Promise<T | undefined> {
        const database = db ?? this.requireDb();
        const transaction = database.transaction(storeName, 'readonly');
        const record = (await requestResult(
            transaction.objectStore(storeName).get(key),
        )) as T | undefined;
        await transactionDone(transaction);
        return record;
    }

    private async readAll<T>(
        storeName: AgentDbStoreName,
        db?: IDBDatabase,
    ): Promise<T[]> {
        const database = db ?? this.requireDb();
        const transaction = database.transaction(storeName, 'readonly');
        const records = (await requestResult(
            transaction.objectStore(storeName).getAll(),
        )) as T[];
        await transactionDone(transaction);
        return records;
    }

    private async readAllPairs<T>(
        storeName: AgentDbStoreName,
        db?: IDBDatabase,
    ): Promise<Array<{ key: IDBValidKey; record: T }>> {
        const database = db ?? this.requireDb();
        const transaction = database.transaction(storeName, 'readonly');
        const records = (await requestResult(
            transaction.objectStore(storeName).getAll(),
        )) as T[];
        const keys = await requestResult(
            transaction.objectStore(storeName).getAllKeys(),
        );
        await transactionDone(transaction);
        if (records.length !== keys.length) {
            corrupt('record values and keys have different lengths');
        }
        return records.map((record, index) => ({
            key: keys[index]!,
            record,
        }));
    }

    private async readAllKeys(
        storeName: AgentDbStoreName,
        db?: IDBDatabase,
    ): Promise<IDBValidKey[]> {
        const database = db ?? this.requireDb();
        const transaction = database.transaction(storeName, 'readonly');
        const keys = await requestResult(
            transaction.objectStore(storeName).getAllKeys(),
        );
        await transactionDone(transaction);
        return keys;
    }

    private async countStore(storeName: AgentDbStoreName): Promise<number> {
        const database = this.requireDb();
        const transaction = database.transaction(storeName, 'readonly');
        const count = await requestResult(
            transaction.objectStore(storeName).count(),
        );
        await transactionDone(transaction);
        return count;
    }

    private async readRunRecord(taskId: string, runId: string): Promise<RunRecord | null> {
        const record = (await this.readOne<RunRecord>('runs', [taskId, runId])) as
            | RunRecord
            | undefined;
        if (record === undefined) return null;
        requireStoredRun(record);
        if (record.task_id !== taskId || record.run_id !== runId) {
            corrupt('run record fields do not match their IndexedDB key');
        }
        return record;
    }

    private async readEventPairs(
        taskId: string,
        runId: string,
        db?: IDBDatabase,
    ): Promise<EventPair[]> {
        const database = db ?? this.requireDb();
        const transaction = database.transaction('events', 'readonly');
        const store = transaction.objectStore('events');
        const range = IDBKeyRange.bound(
            [taskId, runId, 0, 0],
            [taskId, runId, Number.MAX_SAFE_INTEGER, Number.MAX_SAFE_INTEGER],
        );
        const records = (await requestResult(store.getAll(range))) as EventRecord[];
        const keys = await requestResult(store.getAllKeys(range));
        await transactionDone(transaction);
        if (records.length !== keys.length) {
            corrupt('event records and keys have different lengths');
        }
        return records.map((record, index) => ({
            record,
            key: keys[index]!,
        }));
    }

    private async readAllEventPairs(): Promise<EventPair[]> {
        const database = this.requireDb();
        const transaction = database.transaction('events', 'readonly');
        const store = transaction.objectStore('events');
        const records = (await requestResult(store.getAll())) as EventRecord[];
        const keys = await requestResult(store.getAllKeys());
        await transactionDone(transaction);
        if (records.length !== keys.length) {
            corrupt('event records and keys have different lengths');
        }
        return records.map((record, index) => ({
            record,
            key: keys[index]!,
        }));
    }

    private pairRunKey(pair: EventPair): string {
        const key = pair.key;
        if (!Array.isArray(key) || key.length !== 4) {
            corrupt('event store key is not [task_id, run_id, generation, sequence]');
        }
        return runKey(String(key[0]), String(key[1]));
    }

    private comparePairs(left: EventPair, right: EventPair): number {
        const leftKey = left.key as [string, string, number, number];
        const rightKey = right.key as [string, string, number, number];
        return (
            leftKey[0].localeCompare(rightKey[0]) ||
            leftKey[1].localeCompare(rightKey[1]) ||
            leftKey[2] - rightKey[2] ||
            leftKey[3] - rightKey[3]
        );
    }
}
