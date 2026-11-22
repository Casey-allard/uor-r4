import { expect, test, type Page } from '@playwright/test';

const TEST_URL = '/?typed=1&studioTest=1';
const DB_NAME = 'uor-r4.studio.agent';
const FULL64_A1 = `art_${'a1'.repeat(32)}`;
const FULL64_B2 = `art_${'b2'.repeat(32)}`;
const FULL64_MISSING = `art_${'f'.repeat(64)}`;
const FULL64_RANGE = `art_${'d4'.repeat(32)}`;
const FULL64_HOST = `art_${'e5'.repeat(32)}`;
const FULL64_CORRUPT_DIGEST = `art_${'c1'.repeat(32)}`;
const FULL64_CORRUPT_SIZE = `art_${'c2'.repeat(32)}`;
const FULL64_CORRUPT_METADATA = `art_${'c3'.repeat(32)}`;
const FULL64_CORRUPT_BLOB_TYPE = `art_${'c4'.repeat(32)}`;
const FULL64_CORRUPT_OVERSIZE = `art_${'c5'.repeat(32)}`;
const FULL64_NORMALIZED = `art_${'f6'.repeat(32)}`;
const FULL64_PARAMS_PARTIAL = `art_${'f7'.repeat(32)}`;
const FULL64_PARAMS_REORDERED = `art_${'f8'.repeat(32)}`;
const FULL64_PARAMS_CONFLICT = `art_${'f9'.repeat(32)}`;
const FULL64_PARAMS_BLOB_ONLY = `art_${'fa'.repeat(32)}`;
const FULL64_PARAMS_QUOTED = `art_${'fb'.repeat(32)}`;

interface StudioTestCall {
    method: string;
    args: unknown[];
}

async function openStudio(page: Page, path = TEST_URL): Promise<void> {
    await page.goto(path);
    await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
}

async function call<T>(page: Page, method: string, ...args: unknown[]): Promise<T> {
    const payload: StudioTestCall = { method, args };
    return page.evaluate(({ method, args }) => {
        const api = (window as unknown as {
            __studioTest?: Record<string, (...callArgs: unknown[]) => Promise<unknown>>;
        }).__studioTest;
        if (!api) throw new Error('window.__studioTest is missing');
        return api[method]!(...args);
    }, payload) as Promise<T>;
}

interface DbSnapshot {
    version: number;
    stores: string[];
    eventKeys: unknown[];
    eventRecords: Array<{
        event_id: string;
        canonical: string;
        digest: string;
        prev_digest: string | null;
        recorded_at_ms: number;
    }>;
    runKeys: unknown[];
    runRecords: Array<Record<string, unknown>>;
    snapshotKeys: unknown[];
    snapshotRecords: Array<Record<string, unknown>>;
    changeKeys: unknown[];
    changeRecords: Array<Record<string, unknown>>;
    taskRecords: Array<Record<string, unknown>>;
    leaseRecords: Array<Record<string, unknown>>;
    byEventId: Record<string, unknown> | null;
    eventIndexNames: string[];
    eventIndexUnique: boolean;
}

async function readDb(page: Page): Promise<DbSnapshot> {
    return page.evaluate(async (dbName: string) => {
        const request = <T>(input: IDBRequest<T>): Promise<T> =>
            new Promise<T>((resolve, reject) => {
                input.onsuccess = () => resolve(input.result);
                input.onerror = () => reject(input.error);
            });
        const done = (tx: IDBTransaction): Promise<void> =>
            new Promise<void>((resolve, reject) => {
                tx.oncomplete = () => resolve();
                tx.onabort = () => reject(tx.error ?? new Error('transaction aborted'));
                tx.onerror = () => reject(tx.error ?? new Error('transaction failed'));
            });
        const db = await new Promise<IDBDatabase>((resolve, reject) => {
            const open = indexedDB.open(dbName);
            open.onsuccess = () => resolve(open.result);
            open.onerror = () => reject(open.error);
        });
        try {
            const tx = db.transaction(['events', 'runs', 'snapshots', 'changes', 'tasks', 'leases'], 'readonly');
            const events = tx.objectStore('events');
            const runs = tx.objectStore('runs');
            const snapshots = tx.objectStore('snapshots');
            const changes = tx.objectStore('changes');
            const tasks = tx.objectStore('tasks');
            const leases = tx.objectStore('leases');
            const eventKeys = await request(events.getAllKeys());
            const eventRecords = await request(events.getAll());
            const byEventId = eventRecords.length > 0
                ? await request(events.index('by_event_id').get(eventRecords[0]!.event_id))
                : null;
            const runKeys = await request(runs.getAllKeys());
            const runRecords = await request(runs.getAll());
            const snapshotKeys = await request(snapshots.getAllKeys());
            const snapshotRecords = await request(snapshots.getAll());
            const changeKeys = await request(changes.getAllKeys());
            const changeRecords = await request(changes.getAll());
            const taskRecords = await request(tasks.getAll());
            const leaseRecords = await request(leases.getAll());
            const byEventIndex = events.index('by_event_id');
            await done(tx);
            return {
                version: db.version,
                stores: [...db.objectStoreNames],
                eventKeys,
                eventRecords,
                runKeys,
                runRecords,
                snapshotKeys,
                snapshotRecords,
                changeKeys,
                changeRecords,
                taskRecords,
                leaseRecords,
                byEventId,
                eventIndexNames: [...events.indexNames],
                eventIndexUnique: byEventIndex.unique,
            };
        } finally {
            db.close();
        }
    }, DB_NAME);
}

interface SeededArtifact {
    artifactId: string;
    mediaType: string;
    bytes: number[];
    digest: string;
}

interface SeedArtifactOptions {
    sizeOverride?: number;
    digestOverride?: string;
    blobOmit?: boolean;
    blobTypeOverride?: string;
}

async function seedArtifact(
    page: Page,
    artifactId: string,
    text: string,
    mediaType = 'text/plain',
    options: SeedArtifactOptions = {},
): Promise<SeededArtifact> {
    return page.evaluate(
        async ({
            dbName,
            artifactId,
            text,
            mediaType,
            sizeOverride,
            digestOverride,
            blobOmit,
            blobTypeOverride,
        }) => {
            const request = <T>(input: IDBRequest<T>): Promise<T> =>
                new Promise<T>((resolve, reject) => {
                    input.onsuccess = () => resolve(input.result);
                    input.onerror = () => reject(input.error);
                });
            const done = (tx: IDBTransaction): Promise<void> =>
                new Promise<void>((resolve, reject) => {
                    tx.oncomplete = () => resolve();
                    tx.onabort = () => reject(tx.error ?? new Error('transaction aborted'));
                    tx.onerror = () => reject(tx.error ?? new Error('transaction failed'));
                });
            const db = await new Promise<IDBDatabase>((resolve, reject) => {
                const open = indexedDB.open(dbName);
                open.onsuccess = () => resolve(open.result);
                open.onerror = () => reject(open.error);
            });
            try {
                const bytes = new TextEncoder().encode(text);
                const digestBytes = new Uint8Array(
                    await crypto.subtle.digest('SHA-256', bytes),
                );
                let hex = '';
                for (const byte of digestBytes) hex += byte.toString(16).padStart(2, '0');
                const digest = digestOverride ?? `sha256:${hex}`;
                const record: Record<string, unknown> = {
                    artifact_id: artifactId,
                    media_type: mediaType,
                    size_bytes: sizeOverride ?? bytes.byteLength,
                    digest,
                };
                if (blobOmit !== true) {
                    record.blob = new Blob([bytes], {
                        type: blobTypeOverride ?? mediaType,
                    });
                }
                const tx = db.transaction('artifacts', 'readwrite');
                await request(
                    tx.objectStore('artifacts').put(record, artifactId),
                );
                await done(tx);
                return {
                    artifactId,
                    mediaType,
                    bytes: [...bytes],
                    digest,
                };
            } finally {
                db.close();
            }
        },
        {
            dbName: DB_NAME,
            artifactId,
            text,
            mediaType,
            sizeOverride: options.sizeOverride,
            digestOverride: options.digestOverride,
            blobOmit: options.blobOmit,
            blobTypeOverride: options.blobTypeOverride,
        },
    );
}

const mediaArtifactId = (slot: number): string =>
    `art_${slot.toString(16).padStart(2, '0').repeat(32)}`;

interface ArtifactReadOutcome {
    code: string | null;
    mediaType?: string;
    bytes?: number[];
}

async function readArtifactOutcomes(
    page: Page,
    ids: Record<string, string>,
): Promise<Record<string, ArtifactReadOutcome>> {
    return page.evaluate(async (artifactIds) => {
        const api = (
            window as unknown as {
                __studioTest?: {
                    clientForTest?: () => {
                        readArtifact(input: {
                            artifactId: string;
                        }): Promise<{
                            mediaType: string;
                            bytes: Uint8Array;
                        }>;
                    } | null;
                };
            }
        ).__studioTest;
        const client = api?.clientForTest?.();
        if (!client) throw new Error('studio test client is missing');
        const outcomes: Record<string, ArtifactReadOutcome> = {};
        for (const [label, artifactId] of Object.entries(artifactIds)) {
            try {
                const content = await client.readArtifact({ artifactId });
                outcomes[label] = {
                    code: null,
                    mediaType: content.mediaType,
                    bytes: [...content.bytes],
                };
            } catch (error) {
                outcomes[label] = {
                    code: (error as { code?: string }).code ?? null,
                };
            }
        }
        return outcomes;
    }, ids);
}

type Mutation =
    | { op: 'corrupt_snapshot'; taskId: string; runId: string }
    | { op: 'truncate_event'; taskId: string; runId: string; generation: number; sequence: number }
    | { op: 'alter_event'; taskId: string; runId: string; generation: number; sequence: number; from: string; to: string }
    | { op: 'delete_event'; taskId: string; runId: string; generation: number; sequence: number }
    | { op: 'delete_run'; taskId: string; runId: string }
    | { op: 'delete_snapshot'; taskId: string; runId: string }
    | { op: 'delete_task'; taskId: string }
    | { op: 'delete_run_events'; taskId: string; runId: string }
    | { op: 'mutate_run_identity'; taskId: string; runId: string; field: 'task_id' | 'run_id'; value: string }
    | { op: 'put_lease'; taskId: string; writerId: string; nowMs: number }
    | { op: 'set_task_latest'; taskId: string; latestRunId: string | null }
    | { op: 'corrupt_anchor'; taskId: string; runId: string; index: number }
    | { op: 'corrupt_change_files'; taskId: string; runId: string; headDigest: string; files: Array<Record<string, unknown>> }
    | { op: 'oversize_snapshot'; taskId: string; runId: string; bytes: number };

async function mutateDb(page: Page, mutation: Mutation): Promise<void> {
    await page.evaluate(async ({ dbName, mutation }) => {
        const request = <T>(input: IDBRequest<T>): Promise<T> =>
            new Promise<T>((resolve, reject) => {
                input.onsuccess = () => resolve(input.result);
                input.onerror = () => reject(input.error);
            });
        const done = (tx: IDBTransaction): Promise<void> =>
            new Promise<void>((resolve, reject) => {
                tx.oncomplete = () => resolve();
                tx.onabort = () => reject(tx.error ?? new Error('transaction aborted'));
                tx.onerror = () => reject(tx.error ?? new Error('transaction failed'));
            });
        const db = await new Promise<IDBDatabase>((resolve, reject) => {
            const open = indexedDB.open(dbName);
            open.onsuccess = () => resolve(open.result);
            open.onerror = () => reject(open.error);
        });
        try {
            const tx = db.transaction(['events', 'runs', 'snapshots', 'tasks', 'leases', 'changes'], 'readwrite');
            const events = tx.objectStore('events');
            const runs = tx.objectStore('runs');
            const snapshots = tx.objectStore('snapshots');
            const tasks = tx.objectStore('tasks');
            const leases = tx.objectStore('leases');
            const changes = tx.objectStore('changes');
            if (mutation.op === 'corrupt_snapshot') {
                const key = [mutation.taskId, mutation.runId];
                const record = await request(snapshots.get(key));
                if (!record) throw new Error('snapshot record is missing');
                const target = Array.isArray(record.anchors) && record.latest
                    ? record.latest
                    : record;
                target.head_digest = `sha256:${'0'.repeat(64)}`;
                await request(snapshots.put(record, key));
            } else if (mutation.op === 'truncate_event') {
                const key = [mutation.taskId, mutation.runId, mutation.generation, mutation.sequence];
                const record = await request(events.get(key));
                if (!record) throw new Error('event record is missing');
                record.canonical = record.canonical.slice(0, Math.max(1, record.canonical.length - 6));
                await request(events.put(record, key));
            } else if (mutation.op === 'alter_event') {
                const key = [mutation.taskId, mutation.runId, mutation.generation, mutation.sequence];
                const record = await request(events.get(key));
                if (!record) throw new Error('event record is missing');
                if (!record.canonical.includes(mutation.from)) throw new Error('alter target is missing');
                record.canonical = record.canonical.replace(mutation.from, mutation.to);
                await request(events.put(record, key));
            } else if (mutation.op === 'delete_event') {
                await request(events.delete([mutation.taskId, mutation.runId, mutation.generation, mutation.sequence]));
            } else if (mutation.op === 'delete_run') {
                await request(runs.delete([mutation.taskId, mutation.runId]));
                await request(snapshots.delete([mutation.taskId, mutation.runId]));
            } else if (mutation.op === 'delete_snapshot') {
                await request(snapshots.delete([mutation.taskId, mutation.runId]));
            } else if (mutation.op === 'delete_task') {
                await request(tasks.delete(mutation.taskId));
            } else if (mutation.op === 'delete_run_events') {
                const range = IDBKeyRange.bound(
                    [mutation.taskId, mutation.runId, 0, 0],
                    [mutation.taskId, mutation.runId, Number.MAX_SAFE_INTEGER, Number.MAX_SAFE_INTEGER],
                );
                const keys = await request(events.getAllKeys(range));
                for (const key of keys) await request(events.delete(key));
            } else if (mutation.op === 'mutate_run_identity') {
                const key = [mutation.taskId, mutation.runId];
                const record = await request(runs.get(key));
                if (!record) throw new Error('run record is missing');
                record[mutation.field] = mutation.value;
                await request(runs.put(record, key));
            } else if (mutation.op === 'put_lease') {
                await request(
                    leases.put(
                        {
                            task_id: mutation.taskId,
                            writer_id: mutation.writerId,
                            started_at_ms: mutation.nowMs,
                            heartbeat_at_ms: mutation.nowMs,
                        },
                        mutation.taskId,
                    ),
                );
            } else if (mutation.op === 'set_task_latest') {
                const record = await request(tasks.get(mutation.taskId));
                if (!record) throw new Error('task record is missing');
                record.latest_run_id = mutation.latestRunId;
                await request(tasks.put(record, mutation.taskId));
            } else if (mutation.op === 'corrupt_anchor') {
                const key = [mutation.taskId, mutation.runId];
                const record = await request(snapshots.get(key));
                if (!record || !Array.isArray(record.anchors) || !record.anchors[mutation.index]) {
                    throw new Error('snapshot anchor is missing');
                }
                record.anchors[mutation.index].head_digest = `sha256:${'0'.repeat(64)}`;
                await request(snapshots.put(record, key));
            } else if (mutation.op === 'corrupt_change_files') {
                const key = [mutation.taskId, mutation.runId, mutation.headDigest];
                const record = await request(changes.get(key));
                if (!record) throw new Error('change record is missing');
                record.files = mutation.files;
                await request(changes.put(record, key));
            } else {
                const key = [mutation.taskId, mutation.runId];
                const record = await request(snapshots.get(key));
                if (!record) throw new Error('snapshot record is missing');
                const target = Array.isArray(record.anchors) && record.latest
                    ? record.latest
                    : record;
                target.last_event_id = 'x'.repeat(mutation.bytes);
                await request(snapshots.put(record, key));
            }
            await done(tx);
        } finally {
            db.close();
        }
    }, { dbName: DB_NAME, mutation });
}

async function upgradeDatabaseVersion(page: Page): Promise<void> {
    await page.evaluate(async (dbName: string) => {
        const db = await new Promise<IDBDatabase>((resolve, reject) => {
            const open = indexedDB.open(dbName, 2);
            open.onupgradeneeded = () => {
                if (!open.result.objectStoreNames.contains('v2_probe')) {
                    open.result.createObjectStore('v2_probe');
                }
            };
            open.onsuccess = () => resolve(open.result);
            open.onerror = () => reject(open.error);
        });
        db.close();
    }, DB_NAME);
}

interface FixtureSummary {
    taskId: string;
    projectId: string;
    runId: string;
    generation: number;
    eventIds: string[];
    digests: string[];
}

interface CursorView {
    taskId: string;
    runId: string;
    generation: number;
    sequence: number;
    eventId: string;
    digest: string;
    runState: string;
    statusProjection: string | null;
    taskState: string;
    messageCount: number;
    lastMessageText: string | null;
}

interface RestoreView {
    phase: 'idle' | 'loading' | 'ready' | 'incomplete' | 'error';
    sendEnabled: boolean;
    canStartRun: boolean;
    error: { code: string; message: string } | null;
    incomplete: Array<{ taskId: string | null; runId: string | null; code: string; message: string }>;
    activeRun: {
        runId: string;
        generation: number;
        lastSequence: number;
        lastEventId: string | null;
        state: string;
        statusProjection: string | null;
    } | null;
}

interface TaskViewSummary {
    taskId: string;
    title: string;
    pinned: boolean;
    archived: boolean;
    state: string;
    latestRunId: string | null;
    hold: {
        reason: 'waiting_for_background';
        run_id: string;
        tool_call_id: string;
    } | null;
    runs: Array<{ runId: string; lastSequence: number; state: string }>;
}

interface Outcome {
    ok: boolean;
    code?: string;
    message?: string;
    status?: string;
    storage?: string | null;
    record?: Record<string, unknown>;
}

test.describe('browser agent task store', () => {
    test.beforeEach(async ({ page }) => {
        await openStudio(page);
        await call(page, 'reset');
    });

    test('indexeddb_save_reload_preserves_cursor', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        const before = await call<CursorView>(page, 'cursor');
        expect(before.sequence).toBe(2);
        expect(before.eventId).toBe(created.eventIds[2]);
        expect(before.runState).toBe('running');
        expect(before.taskState).toBe('running');
        expect(before.messageCount).toBe(1);

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        const restored = await call<RestoreView>(page, 'open');
        expect(restored.phase).toBe('ready');
        const after = await call<CursorView>(page, 'cursor');
        expect(after).toMatchObject({
            taskId: created.taskId,
            runId: created.runId,
            generation: 1,
            sequence: 2,
            eventId: created.eventIds[2],
            digest: created.digests[2],
            runState: 'running',
            taskState: 'running',
            messageCount: 1,
            lastMessageText: 'replayed fixture message',
        });
    });

    test('indexeddb_per_run_records_are_keyed', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        await call(page, 'writeSnapshot');
        const db = await readDb(page);
        expect(db.version).toBe(1);
        expect(db.stores.sort()).toEqual([
            'approvals',
            'artifacts',
            'changes',
            'comments',
            'events',
            'leases',
            'runs',
            'snapshots',
            'tasks',
        ].sort());
        expect(db.eventKeys).toEqual([
            [created.taskId, created.runId, 1, 0],
            [created.taskId, created.runId, 1, 1],
            [created.taskId, created.runId, 1, 2],
        ]);
        expect(db.eventKeys.every((key) => Array.isArray(key) && key.length === 4)).toBe(true);
        expect(db.eventIndexNames).toContain('by_event_id');
        expect(db.eventIndexUnique).toBe(true);
        expect(db.byEventId).toMatchObject({ event_id: created.eventIds[0] });
        expect(db.eventRecords.map((record) => record.event_id)).toEqual(created.eventIds);
        expect(db.eventRecords.map((record) => record.digest)).toEqual(created.digests);
        expect(db.eventRecords[0]!.prev_digest).toBeNull();
        expect(db.eventRecords[1]!.prev_digest).toBe(db.eventRecords[0]!.digest);
        expect(db.eventRecords[2]!.prev_digest).toBe(db.eventRecords[1]!.digest);
        expect(db.runKeys).toEqual([[created.taskId, created.runId]]);
        expect(db.runRecords[0]).toMatchObject({
            task_id: created.taskId,
            run_id: created.runId,
            generation: 1,
            last_sequence: 2,
            last_event_id: created.eventIds[2],
            last_digest: created.digests[2],
        });
        expect(db.snapshotKeys).toEqual([[created.taskId, created.runId]]);
        const storedSnapshot = db.snapshotRecords[0] as Record<string, unknown>;
        const latestSnapshot =
            (storedSnapshot.latest as Record<string, unknown> | undefined) ??
            storedSnapshot;
        expect(latestSnapshot).toMatchObject({
            task_id: created.taskId,
            run_id: created.runId,
            generation: 1,
            last_event_id: created.eventIds[2],
            last_sequence: 2,
            head_digest: created.digests[2],
        });
    });

    test('durable_browser_artifact_is_read_without_host_and_returns_defensive_bytes', async ({ page }) => {
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const seeded = await seedArtifact(
            page,
            FULL64_A1,
            'durable browser artifact',
        );
        await seedArtifact(
            page,
            FULL64_B2,
            'corrupt durable artifact',
            'text/plain',
            { digestOverride: `sha256:${'0'.repeat(64)}` },
        );

        const result = await page.evaluate(
            async ({ artifactId, missingId, corruptId }) => {
                const api = (
                    window as unknown as {
                        __studioTest?: {
                            clientForTest?: () => {
                                readArtifact(input: {
                                    artifactId: string;
                                }): Promise<{
                                    path: string;
                                    mediaType: string;
                                    sizeBytes: number;
                                    digest: string;
                                    bytes: Uint8Array;
                                }>;
                            } | null;
                        };
                    }
                ).__studioTest;
                const client = api?.clientForTest?.();
                if (!client) throw new Error('studio test client is missing');
                let missingCode: string | null = null;
                try {
                    await client.readArtifact({ artifactId: missingId });
                } catch (error) {
                    missingCode =
                        (error as { code?: string }).code ?? null;
                }
                let corruptCode: string | null = null;
                try {
                    await client.readArtifact({ artifactId: corruptId });
                } catch (error) {
                    corruptCode =
                        (error as { code?: string }).code ?? null;
                }
                const first = await client.readArtifact({ artifactId });
                const firstBytes = [...first.bytes];
                first.bytes[0] = (first.bytes[0] ?? 0) ^ 0xff;
                const second = await client.readArtifact({ artifactId });
                return {
                    missingCode,
                    corruptCode,
                    firstBytes,
                    secondBytes: [...second.bytes],
                    path: first.path,
                    mediaType: first.mediaType,
                    sizeBytes: first.sizeBytes,
                    digest: first.digest,
                };
            },
            {
                artifactId: seeded.artifactId,
                missingId: FULL64_MISSING,
                corruptId: FULL64_B2,
            },
        );

        expect(result.missingCode).toBe('host_unavailable');
        expect(result.corruptCode).toBe('snapshot_corrupt');
        expect(result.firstBytes).toEqual(seeded.bytes);
        expect(result.secondBytes).toEqual(seeded.bytes);
        expect(result).toMatchObject({
            path: seeded.artifactId,
            mediaType: seeded.mediaType,
            sizeBytes: seeded.bytes.length,
            digest: seeded.digest,
        });
    });

    test('durable_artifact_ranges_preserve_returned_and_whole_metadata', async ({ page }) => {
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const text = 'r'.repeat(1_572_864);
        const seeded = await seedArtifact(page, FULL64_RANGE, text);
        const result = await page.evaluate(
            async ({ artifactId }) => {
                const api = (
                    window as unknown as {
                        __studioTest?: {
                            clientForTest?: () => {
                                readArtifact(input: {
                                    artifactId: string;
                                    start?: number;
                                    end?: number;
                                }): Promise<{
                                    path: string;
                                    mediaType: string;
                                    sizeBytes: number;
                                    artifactSizeBytes?: number;
                                    digest: string;
                                    wholeDigest?: string;
                                    range?: { start: number; end: number };
                                    bytes: Uint8Array;
                                }>;
                            } | null;
                        };
                    }
                ).__studioTest;
                const client = api?.clientForTest?.();
                if (!client) throw new Error('studio test client is missing');
                const whole = await client.readArtifact({ artifactId });
                const range = await client.readArtifact({
                    artifactId,
                    start: 0,
                    end: 1024 * 1024,
                });
                const firstByte = range.bytes[0];
                range.bytes[0] = (range.bytes[0] ?? 0) ^ 0xff;
                const again = await client.readArtifact({
                    artifactId,
                    start: 0,
                    end: 1024 * 1024,
                });
                let oversizedRangeCode: string | null = null;
                try {
                    await client.readArtifact({
                        artifactId,
                        start: 0,
                        end: 1_572_864,
                    });
                } catch (error) {
                    oversizedRangeCode =
                        (error as { code?: string }).code ?? null;
                }
                let outOfRangeCode: string | null = null;
                try {
                    await client.readArtifact({
                        artifactId,
                        start: 1_500_000,
                        end: 1_600_000,
                    });
                } catch (error) {
                    outOfRangeCode =
                        (error as { code?: string }).code ?? null;
                }
                return {
                    oversizedRangeCode,
                    outOfRangeCode,
                    whole: {
                        sizeBytes: whole.sizeBytes,
                        artifactSizeBytes: whole.artifactSizeBytes,
                        digest: whole.digest,
                        wholeDigest: whole.wholeDigest,
                        range: whole.range,
                    },
                    range: {
                        sizeBytes: range.sizeBytes,
                        artifactSizeBytes: range.artifactSizeBytes,
                        digest: range.digest,
                        wholeDigest: range.wholeDigest,
                        range: range.range,
                    },
                    firstByte,
                    secondFirstByte: again.bytes[0],
                };
            },
            { artifactId: seeded.artifactId },
        );

        expect(result.whole.sizeBytes).toBe(seeded.bytes.length);
        expect(result.whole.artifactSizeBytes).toBe(seeded.bytes.length);
        expect(result.whole.digest).toBe(seeded.digest);
        expect(result.whole.wholeDigest).toBe(seeded.digest);
        expect(result.whole.range).toEqual({
            start: 0,
            end: seeded.bytes.length,
        });
        expect(result.range.sizeBytes).toBe(1024 * 1024);
        expect(result.range.artifactSizeBytes).toBe(seeded.bytes.length);
        expect(result.range.wholeDigest).toBe(seeded.digest);
        expect(result.range.range).toEqual({ start: 0, end: 1024 * 1024 });
        expect(result.range.digest).not.toBe(seeded.digest);
        expect(result.firstByte).toBe(seeded.bytes[0]);
        expect(result.secondFirstByte).toBe(seeded.bytes[0]);
        expect(result.oversizedRangeCode).toBe('limit_exceeded');
        expect(result.outOfRangeCode).toBe('invalid_arguments');
    });

    test('durable_artifact_validation_refuses_corrupt_records', async ({ page }) => {
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        await seedArtifact(page, FULL64_CORRUPT_DIGEST, 'wrong digest', 'text/plain', {
            digestOverride: `sha256:${'0'.repeat(64)}`,
        });
        await seedArtifact(page, FULL64_CORRUPT_SIZE, 'size mismatch', 'text/plain', {
            sizeOverride: 999,
        });
        await seedArtifact(page, FULL64_CORRUPT_METADATA, 'metadata only', 'text/plain', {
            blobOmit: true,
        });
        await seedArtifact(page, FULL64_CORRUPT_BLOB_TYPE, 'blob type', 'text/plain', {
            blobTypeOverride: 'application/json',
        });
        await seedArtifact(page, FULL64_CORRUPT_OVERSIZE, 'oversize', 'text/plain', {
            sizeOverride: 8 * 1024 * 1024 + 1,
        });

        const codes = await page.evaluate(
            async ({ ids }) => {
                const api = (
                    window as unknown as {
                        __studioTest?: {
                            clientForTest?: () => {
                                readArtifact(input: {
                                    artifactId: string;
                                    start?: number;
                                    end?: number;
                                }): Promise<unknown>;
                            } | null;
                        };
                    }
                ).__studioTest;
                const client = api?.clientForTest?.();
                if (!client) throw new Error('studio test client is missing');
                const result: Record<string, string | null> = {};
                for (const [label, artifactId] of Object.entries(ids)) {
                    try {
                        await client.readArtifact({ artifactId });
                        result[label] = null;
                    } catch (error) {
                        result[label] =
                            (error as { code?: string }).code ?? null;
                    }
                }
                try {
                    await client.readArtifact({
                        artifactId: ids.digest,
                        start: 0,
                        end: 5,
                    });
                    result.digestRange = null;
                } catch (error) {
                    result.digestRange =
                        (error as { code?: string }).code ?? null;
                }
                return result;
            },
            {
                ids: {
                    digest: FULL64_CORRUPT_DIGEST,
                    size: FULL64_CORRUPT_SIZE,
                    metadata: FULL64_CORRUPT_METADATA,
                    blobType: FULL64_CORRUPT_BLOB_TYPE,
                    oversize: FULL64_CORRUPT_OVERSIZE,
                },
            },
        );

        expect(codes).toEqual({
            digest: 'snapshot_corrupt',
            size: 'snapshot_corrupt',
            metadata: 'snapshot_corrupt',
            blobType: 'snapshot_corrupt',
            oversize: 'snapshot_corrupt',
            digestRange: 'snapshot_corrupt',
        });
    });

    test('durable_artifact_media_type_normalization_is_accepted', async ({ page }) => {
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const seeded = await seedArtifact(
            page,
            FULL64_NORMALIZED,
            'normalized media type',
            'Text/Plain; Charset=UTF-8',
            { blobTypeOverride: 'text/plain' },
        );
        const result = await page.evaluate(
            async ({ artifactId }) => {
                const api = (
                    window as unknown as {
                        __studioTest?: {
                            clientForTest?: () => {
                                readArtifact(input: {
                                    artifactId: string;
                                }): Promise<{
                                    path: string;
                                    mediaType: string;
                                    sizeBytes: number;
                                    digest: string;
                                    bytes: Uint8Array;
                                }>;
                            } | null;
                        };
                    }
                ).__studioTest;
                const client = api?.clientForTest?.();
                if (!client) throw new Error('studio test client is missing');
                const content = await client.readArtifact({ artifactId });
                return {
                    path: content.path,
                    mediaType: content.mediaType,
                    sizeBytes: content.sizeBytes,
                    digest: content.digest,
                    bytes: [...content.bytes],
                };
            },
            { artifactId: seeded.artifactId },
        );
        expect(result).toMatchObject({
            path: seeded.artifactId,
            mediaType: 'Text/Plain; Charset=UTF-8',
            sizeBytes: seeded.bytes.length,
            digest: seeded.digest,
        });
        expect(result.bytes).toEqual(seeded.bytes);
    });

    test('durable_artifact_media_parameters_are_parsed', async ({ page }) => {
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const declared = 'text/plain; charset=utf-8; format=flowed';
        const partial = await seedArtifact(
            page,
            FULL64_PARAMS_PARTIAL,
            'partial parameters',
            declared,
            { blobTypeOverride: 'text/plain; charset=utf-8' },
        );
        const reordered = await seedArtifact(
            page,
            FULL64_PARAMS_REORDERED,
            'reordered parameters',
            declared,
            { blobTypeOverride: 'text/plain; format=flowed; charset=utf-8' },
        );
        const quoted = await seedArtifact(
            page,
            FULL64_PARAMS_QUOTED,
            'quoted parameters',
            'text/plain; charset="UTF-8"',
            { blobTypeOverride: 'text/plain; charset=utf-8' },
        );
        await seedArtifact(
            page,
            FULL64_PARAMS_CONFLICT,
            'conflicting parameters',
            'text/plain; charset=utf-8',
            { blobTypeOverride: 'text/plain; charset=ascii' },
        );
        await seedArtifact(
            page,
            FULL64_PARAMS_BLOB_ONLY,
            'blob only parameter',
            'text/plain',
            { blobTypeOverride: 'text/plain; charset=utf-8' },
        );

        const result = await page.evaluate(
            async ({ ids }) => {
                const api = (
                    window as unknown as {
                        __studioTest?: {
                            clientForTest?: () => {
                                readArtifact(input: {
                                    artifactId: string;
                                }): Promise<{
                                    mediaType: string;
                                    bytes: Uint8Array;
                                }>;
                            } | null;
                        };
                    }
                ).__studioTest;
                const client = api?.clientForTest?.();
                if (!client) throw new Error('studio test client is missing');
                const read = async (
                    artifactId: string,
                ): Promise<{
                    code: string | null;
                    mediaType?: string;
                    bytes?: number[];
                }> => {
                    try {
                        const content = await client.readArtifact({ artifactId });
                        return {
                            code: null,
                            mediaType: content.mediaType,
                            bytes: [...content.bytes],
                        };
                    } catch (error) {
                        return {
                            code: (error as { code?: string }).code ?? null,
                        };
                    }
                };
                return {
                    partial: await read(ids.partial),
                    reordered: await read(ids.reordered),
                    quoted: await read(ids.quoted),
                    conflict: await read(ids.conflict),
                    blobOnly: await read(ids.blobOnly),
                };
            },
            {
                ids: {
                    partial: FULL64_PARAMS_PARTIAL,
                    reordered: FULL64_PARAMS_REORDERED,
                    quoted: FULL64_PARAMS_QUOTED,
                    conflict: FULL64_PARAMS_CONFLICT,
                    blobOnly: FULL64_PARAMS_BLOB_ONLY,
                },
            },
        );

        expect(result.partial.code).toBeNull();
        expect(result.partial.mediaType).toBe(declared);
        expect(result.partial.bytes).toEqual(partial.bytes);
        expect(result.reordered.code).toBeNull();
        expect(result.reordered.mediaType).toBe(declared);
        expect(result.reordered.bytes).toEqual(reordered.bytes);
        expect(result.quoted.code).toBeNull();
        expect(result.quoted.mediaType).toBe('text/plain; charset="UTF-8"');
        expect(result.quoted.bytes).toEqual(quoted.bytes);
        expect(result.conflict.code).toBe('snapshot_corrupt');
        expect(result.blobOnly.code).toBe('snapshot_corrupt');
    });

    test('durable_artifact_empty_blob_type_still_validates_declared_media_type', async ({ page }) => {
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const accepted = await seedArtifact(
            page,
            mediaArtifactId(1),
            'declared only parameters',
            'text/plain; charset=utf-8; format=flowed',
            { blobTypeOverride: '' },
        );
        const malformed: Array<{ label: string; mediaType: string }> = [
            { label: 'unterminated', mediaType: 'text/plain; charset="unterminated' },
            { label: 'missingEquals', mediaType: 'text/plain; bad-param' },
            { label: 'emptyName', mediaType: 'text/plain; =utf-8' },
            { label: 'emptyPart', mediaType: 'text/plain;;charset=utf-8' },
        ];
        const ids: Record<string, string> = { accepted: accepted.artifactId };
        for (const [index, entry] of malformed.entries()) {
            const artifactId = mediaArtifactId(2 + index);
            ids[entry.label] = artifactId;
            await seedArtifact(page, artifactId, entry.label, entry.mediaType, {
                blobTypeOverride: '',
            });
        }
        const outcomes = await readArtifactOutcomes(page, ids);
        const codes = Object.fromEntries(
            Object.entries(outcomes).map(([label, outcome]) => [
                label,
                outcome.code,
            ]),
        );
        expect(codes).toEqual({
            accepted: null,
            unterminated: 'snapshot_corrupt',
            missingEquals: 'snapshot_corrupt',
            emptyName: 'snapshot_corrupt',
            emptyPart: 'snapshot_corrupt',
        });
        expect(outcomes.accepted!.mediaType).toBe(
            'text/plain; charset=utf-8; format=flowed',
        );
        expect(outcomes.accepted!.bytes).toEqual(accepted.bytes);
    });

    test('durable_artifact_media_type_parameter_syntax_is_validated', async ({ page }) => {
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const escapedMediaType = 'text/plain; note="a\\"b\\\\c"';
        const escaped = await seedArtifact(
            page,
            mediaArtifactId(10),
            'escaped quoted value',
            escapedMediaType,
            { blobTypeOverride: escapedMediaType },
        );
        const blobCases: Array<{
            label: string;
            declared: string;
            blobType: string;
        }> = [
            {
                label: 'tooLong',
                declared: 'text/plain',
                blobType: `text/plain; long=${'a'.repeat(1024)}`,
            },
            {
                label: 'tooMany',
                declared: 'text/plain',
                blobType: `text/plain${Array.from(
                    { length: 17 },
                    (_, index) => `; p${index}=v${index}`,
                ).join('')}`,
            },
            {
                label: 'unterminated',
                declared: 'text/plain',
                blobType: 'text/plain; charset="utf-8',
            },
            {
                label: 'quotedName',
                declared: 'text/plain; ch"ars"et=utf-8',
                blobType: 'text/plain; ch"ars"et=utf-8',
            },
            {
                label: 'backslashName',
                declared: 'text/plain; char\\set=utf-8',
                blobType: 'text/plain; char\\set=utf-8',
            },
            {
                label: 'unquotedBackslash',
                declared: 'text/plain; charset=ut\\f-8',
                blobType: 'text/plain; charset=ut\\f-8',
            },
            {
                label: 'unquotedQuote',
                declared: 'text/plain; charset=ut"f"8',
                blobType: 'text/plain; charset=ut"f"8',
            },
            {
                label: 'invalidQuotePlacement',
                declared: 'text/plain; charset="ut"f""',
                blobType: 'text/plain; charset="ut"f""',
            },
            {
                label: 'duplicateSame',
                declared: 'text/plain; charset=utf-8',
                blobType: 'text/plain; charset=utf-8; charset=utf-8',
            },
            {
                label: 'duplicateConflict',
                declared: 'text/plain; charset=utf-8',
                blobType: 'text/plain; charset=utf-8; charset=ascii',
            },
        ];
        const ids: Record<string, string> = { escaped: escaped.artifactId };
        for (const [index, entry] of blobCases.entries()) {
            const artifactId = mediaArtifactId(11 + index);
            ids[entry.label] = artifactId;
            await seedArtifact(page, artifactId, entry.label, entry.declared, {
                blobTypeOverride: entry.blobType,
            });
        }
        const outcomes = await readArtifactOutcomes(page, ids);
        const codes = Object.fromEntries(
            Object.entries(outcomes).map(([label, outcome]) => [
                label,
                outcome.code,
            ]),
        );
        expect(codes).toEqual({
            escaped: null,
            tooLong: 'snapshot_corrupt',
            tooMany: 'snapshot_corrupt',
            unterminated: 'snapshot_corrupt',
            quotedName: 'snapshot_corrupt',
            backslashName: 'snapshot_corrupt',
            unquotedBackslash: 'snapshot_corrupt',
            unquotedQuote: 'snapshot_corrupt',
            invalidQuotePlacement: 'snapshot_corrupt',
            duplicateSame: 'snapshot_corrupt',
            duplicateConflict: 'snapshot_corrupt',
        });
        expect(outcomes.escaped!.mediaType).toBe(escapedMediaType);
        expect(outcomes.escaped!.bytes).toEqual(escaped.bytes);
    });

    test('full64_host_id_without_durable_record_emits_artifact_read_intent', async ({ page }) => {
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const probe = await call<{
            ok: boolean;
            code?: string;
            record?: {
                intents: Array<{
                    tool: string;
                    requestScope?: string;
                    artifactId?: string;
                }>;
                content: {
                    bytes: number[];
                    digest: string;
                    sizeBytes: number;
                    path: string;
                    mediaType: string;
                };
            };
        }>(page, 'artifactFallbackProbeForTest', FULL64_HOST);
        expect(probe.ok, JSON.stringify(probe)).toBe(true);
        const record = probe.record!;
        expect(record.intents).toHaveLength(1);
        expect(record.intents[0]).toMatchObject({
            tool: 'artifact.read',
            requestScope: 'data',
            artifactId: FULL64_HOST,
        });
        expect(new TextDecoder().decode(new Uint8Array(record.content.bytes))).toBe(
            'host artifact',
        );
    });

    test('indexeddb_corrupt_snapshot_is_refused', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        await call(page, 'writeSnapshot');
        await mutateDb(page, { op: 'corrupt_snapshot', taskId: created.taskId, runId: created.runId });
        const direct = await call<Outcome>(page, 'readSnapshot');
        expect(direct.ok).toBe(false);
        expect(direct.code).toBe('snapshot_corrupt');

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        const restored = await call<RestoreView>(page, 'open');
        expect(restored.phase).toBe('error');
        expect(restored.error?.code).toBe('snapshot_corrupt');
        expect(await call<CursorView | null>(page, 'cursor')).toBeNull();

        const db = await readDb(page);
        expect(db.eventRecords).toHaveLength(3);
        expect(db.snapshotRecords).toHaveLength(1);
    });

    test('indexeddb_truncated_event_record_is_refused', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        const before = await readDb(page);
        const originalLength = before.eventRecords[1]!.canonical.length;
        await mutateDb(page, {
            op: 'truncate_event',
            taskId: created.taskId,
            runId: created.runId,
            generation: 1,
            sequence: 1,
        });

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        const restored = await call<RestoreView>(page, 'open');
        expect(restored.phase).toBe('error');
        expect(restored.error?.code).toBe('snapshot_corrupt');
        expect(await call<CursorView | null>(page, 'cursor')).toBeNull();

        const db = await readDb(page);
        expect(db.eventRecords).toHaveLength(3);
        expect(db.eventRecords[1]!.canonical.length).toBe(originalLength - 6);
    });

    test('indexeddb_altered_event_digest_is_refused', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        await mutateDb(page, {
            op: 'alter_event',
            taskId: created.taskId,
            runId: created.runId,
            generation: 1,
            sequence: 2,
            from: 'replayed fixture message',
            to: 'tampered fixture message',
        });

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        const restored = await call<RestoreView>(page, 'open');
        expect(restored.phase).toBe('error');
        expect(restored.error?.code).toBe('snapshot_corrupt');
        expect(await call<CursorView | null>(page, 'cursor')).toBeNull();
        expect((await readDb(page)).eventRecords).toHaveLength(3);
    });

    test('failed_reopen_preserves_the_last_verified_memory_cursor', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        const before = await call<CursorView>(page, 'cursor');
        await mutateDb(page, {
            op: 'truncate_event',
            taskId: created.taskId,
            runId: created.runId,
            generation: 1,
            sequence: 1,
        });
        const reopened = await call<RestoreView>(page, 'reopen');
        expect(reopened.phase).toBe('error');
        expect(reopened.error?.code).toBe('snapshot_corrupt');
        expect(await call<CursorView | null>(page, 'memoryCursor')).toMatchObject({
            taskId: before.taskId,
            runId: before.runId,
            sequence: before.sequence,
            eventId: before.eventId,
        });
        const task = await call<TaskViewSummary>(page, 'taskView');
        expect(task.runs).toHaveLength(1);
    });

    test('indexeddb_missing_chain_event_is_refused', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        await mutateDb(page, {
            op: 'delete_event',
            taskId: created.taskId,
            runId: created.runId,
            generation: 1,
            sequence: 1,
        });

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        const restored = await call<RestoreView>(page, 'open');
        expect(restored.phase).toBe('error');
        expect(restored.error?.code).toBe('snapshot_corrupt');
        expect((await readDb(page)).eventRecords).toHaveLength(2);
    });

    test('indexeddb_lease_uses_heartbeat_not_started_at', async ({ page }) => {
        await call<FixtureSummary>(page, 'createTaskAndEvents');

        const first = await call<Outcome>(page, 'acquireLease', 'writer-a', 1_000_000);
        expect(first.ok).toBe(true);
        expect(first.record).toMatchObject({
            writer_id: 'writer-a',
            started_at_ms: 1_000_000,
            heartbeat_at_ms: 1_000_000,
        });

        const beat = await call<Outcome>(page, 'heartbeatLease', 'writer-a', 1_020_000);
        expect(beat.ok).toBe(true);
        expect(beat.record).toMatchObject({
            writer_id: 'writer-a',
            started_at_ms: 1_000_000,
            heartbeat_at_ms: 1_020_000,
        });

        const renewed = await call<Outcome>(page, 'acquireLease', 'writer-a', 1_045_000);
        expect(renewed.ok).toBe(true);
        expect(renewed.record).toMatchObject({
            started_at_ms: 1_000_000,
            heartbeat_at_ms: 1_045_000,
        });

        const blocked = await call<Outcome>(page, 'acquireLease', 'writer-b', 1_046_000);
        expect(blocked.ok).toBe(false);
        expect(blocked.code).toBe('stale_generation');
        expect((await call<Outcome>(page, 'lease')).record).toMatchObject({
            writer_id: 'writer-a',
            heartbeat_at_ms: 1_045_000,
        });

        const takeover = await call<Outcome>(page, 'acquireLease', 'writer-b', 1_076_000);
        expect(takeover.ok).toBe(true);
        expect(takeover.record).toMatchObject({
            writer_id: 'writer-b',
            started_at_ms: 1_076_000,
            heartbeat_at_ms: 1_076_000,
        });

        const releaseA = await call<Outcome>(page, 'releaseLease', 'writer-a');
        expect(releaseA.ok).toBe(false);
        expect(releaseA.code).toBe('stale_generation');
        const releaseB = await call<Outcome>(page, 'releaseLease', 'writer-b');
        expect(releaseB.ok).toBe(true);
        expect((await call<Outcome>(page, 'lease')).record).toBeNull();
    });

    test('lease_is_live_at_exactly_30000ms_and_stale_at_30001ms', async ({ page }) => {
        await call<FixtureSummary>(page, 'createTaskAndEvents');
        expect((await call<Outcome>(page, 'acquireLease', 'writer-a', 1_000_000)).ok).toBe(true);

        const exactlyThirty = await call<Outcome>(page, 'acquireLease', 'writer-b', 1_030_000);
        expect(exactlyThirty.ok).toBe(false);
        expect(exactlyThirty.code).toBe('stale_generation');

        const thirtyOne = await call<Outcome>(page, 'acquireLease', 'writer-b', 1_030_001);
        expect(thirtyOne.ok).toBe(true);
        expect(thirtyOne.record).toMatchObject({
            writer_id: 'writer-b',
            started_at_ms: 1_030_001,
            heartbeat_at_ms: 1_030_001,
        });
    });

    test('indexeddb_changes_store_records_snapshot', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        const appended = await call<Outcome>(page, 'appendChanges');
        expect(appended.status).toBe('applied');
        const recorded = await call<Outcome>(page, 'recordChanges');
        expect(recorded.ok).toBe(true);
        expect(recorded.record).toMatchObject({
            task_id: created.taskId,
            run_id: created.runId,
            scope: 'last_turn',
            head_digest: 'turn:3:fixture',
        });

        const db = await readDb(page);
        expect(db.changeKeys).toEqual([[created.taskId, created.runId, 'turn:3:fixture']]);
        expect(db.changeRecords[0]).toMatchObject({
            scope: 'last_turn',
            head_digest: 'turn:3:fixture',
        });
        expect((db.changeRecords[0]!.files as unknown[])).toHaveLength(1);

        const read = await call<Outcome>(page, 'readChanges');
        expect(read.ok).toBe(true);
        expect((read.record as unknown as { records: unknown[] }).records).toHaveLength(1);

        const fabricated = await call<Outcome>(page, 'recordChanges', 'turn:fake');
        expect(fabricated.ok).toBe(false);
        expect(fabricated.code).toBe('invalid_arguments');
        expect((await readDb(page)).changeRecords).toHaveLength(1);

        const fabricatedFiles = await call<Outcome>(
            page,
            'recordChangesWithFiles',
            [{ path: 'src/fabricated.ts', status: 'added' }],
        );
        expect(fabricatedFiles.ok).toBe(false);
        expect(fabricatedFiles.code).toBe('invalid_arguments');
        expect((await readDb(page)).changeRecords).toHaveLength(1);
    });

    test('pin_and_archive_are_reversible_in_store', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        expect((await call<Outcome>(page, 'pin', true)).ok).toBe(true);

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        let task = await call<TaskViewSummary>(page, 'taskView');
        expect(task).toMatchObject({ taskId: created.taskId, pinned: true, archived: false });
        expect((await call<CursorView>(page, 'cursor')).sequence).toBe(2);

        expect((await call<Outcome>(page, 'archive', true)).ok).toBe(true);
        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        task = await call<TaskViewSummary>(page, 'taskView');
        expect(task).toMatchObject({ pinned: true, archived: true, state: 'archived' });
        expect(task.runs).toHaveLength(1);
        expect((await call<CursorView>(page, 'cursor')).sequence).toBe(2);

        expect((await call<Outcome>(page, 'pin', false)).ok).toBe(true);
        expect((await call<Outcome>(page, 'archive', false)).ok).toBe(true);
        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        task = await call<TaskViewSummary>(page, 'taskView');
        expect(task).toMatchObject({ pinned: false, archived: false });
        expect(task.runs).toHaveLength(1);
        expect((await call<CursorView>(page, 'cursor')).sequence).toBe(2);
    });

    test('digest_chain_export_import_verifies_before_apply', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        await call(page, 'writeSnapshot');
        const exported = await call<string>(page, 'exportNdjson');
        const lines = exported.trim().split('\n');
        expect(lines[0]).toContain('"kind":"header"');
        expect(lines.some((line) => line.includes('"kind":"event"'))).toBe(true);
        expect(lines.some((line) => line.includes('"kind":"snapshot"'))).toBe(true);

        await call(page, 'reset');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const imported = await call<Outcome>(page, 'importNdjson', exported);
        expect(imported.ok, JSON.stringify(imported)).toBe(true);
        expect(await call<CursorView>(page, 'cursor')).toMatchObject({
            taskId: created.taskId,
            sequence: 2,
            eventId: created.eventIds[2],
            digest: created.digests[2],
        });
        expect((await call<Outcome>(page, 'readSnapshot')).ok).toBe(true);

        const tampered = lines.map((line) => {
            if (!line.includes('"kind":"event"')) return line;
            const parsed = JSON.parse(line) as { record: { event_id: string; digest: string } };
            if (!parsed.record.event_id.endsWith('000000000002')) return line;
            parsed.record.digest = `sha256:${'1'.repeat(64)}`;
            return JSON.stringify(parsed);
        }).join('\n');
        await call(page, 'reset');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const rejected = await call<Outcome>(page, 'importNdjson', tampered);
        expect(rejected.ok).toBe(false);
        expect(rejected.code).toBe('snapshot_corrupt');
        expect((await readDb(page)).eventRecords).toHaveLength(0);

        const partial = lines.slice(0, -2).join('\n');
        await call(page, 'reset');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const partialResult = await call<Outcome>(page, 'importNdjson', partial);
        expect(partialResult.ok).toBe(false);
        expect((await readDb(page)).eventRecords).toHaveLength(0);
    });

    test('resumed_generation_restores_only_with_its_verified_snapshot', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        expect((await call<Outcome>(page, 'writeSnapshot')).ok).toBe(true);
        const resumed = await call<Outcome>(page, 'appendResume');
        expect(resumed.status).toBe('applied');
        expect(await call<CursorView>(page, 'cursor')).toMatchObject({
            taskId: created.taskId,
            runId: created.runId,
            generation: 2,
            sequence: 3,
            eventId: `evt_${created.runId}_2_000000000003`,
            runState: 'running',
        });

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        expect(await call<CursorView>(page, 'cursor')).toMatchObject({
            generation: 2,
            sequence: 3,
            eventId: `evt_${created.runId}_2_000000000003`,
        });

        await mutateDb(page, { op: 'delete_snapshot', taskId: created.taskId, runId: created.runId });
        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        const refused = await call<RestoreView>(page, 'open');
        expect(refused.phase).toBe('error');
        expect(refused.error?.code).toBe('snapshot_missing');
        expect(await call<CursorView | null>(page, 'cursor')).toBeNull();
        const db = await readDb(page);
        expect(db.eventRecords).toHaveLength(4);
        expect(db.snapshotRecords).toHaveLength(0);
    });

    test('client_resume_commits_pre_resume_snapshot_and_reloads', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createClientRun');
        expect((await call<CursorView>(page, 'cursor')).sequence).toBe(2);

        const resumed = await call<Outcome>(page, 'resumeViaClient');
        expect(resumed.ok, JSON.stringify(resumed)).toBe(true);
        expect(resumed.record).toMatchObject({
            taskId: created.taskId,
            runId: created.runId,
            generation: 1,
            lastSequence: 2,
        });
        expect(await call<CursorView>(page, 'cursor')).toMatchObject({
            generation: 2,
            sequence: 3,
            eventId: `evt_${created.runId}_2_000000000003`,
        });
        const db = await readDb(page);
        expect(db.eventRecords).toHaveLength(4);
        expect(db.snapshotRecords).toHaveLength(1);

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        const restored = await call<RestoreView>(page, 'open');
        expect(restored.phase).toBe('ready');
        expect(await call<CursorView>(page, 'cursor')).toMatchObject({
            taskId: created.taskId,
            runId: created.runId,
            generation: 2,
            sequence: 3,
        });
    });

    test('later_snapshot_preserves_prior_resume_anchors', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        expect((await call<Outcome>(page, 'writeSnapshot')).ok).toBe(true);
        expect((await call<Outcome>(page, 'appendResume')).status).toBe('applied');
        expect((await call<Outcome>(page, 'writeSnapshot')).ok).toBe(true);

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        const restored = await call<RestoreView>(page, 'open');
        expect(restored.phase).toBe('ready');
        expect(await call<CursorView>(page, 'cursor')).toMatchObject({
            taskId: created.taskId,
            generation: 2,
            sequence: 3,
        });

        const db = await readDb(page);
        expect(db.snapshotRecords).toHaveLength(1);
        const stored = db.snapshotRecords[0] as Record<string, unknown>;
        const records = Array.isArray(stored.anchors)
            ? [stored.latest as Record<string, unknown>, ...(stored.anchors as Array<Record<string, unknown>>)]
            : [stored];
        const positions = records.map(
            (record) => `${record.generation}:${record.last_sequence}`,
        );
        expect(positions).toContain('1:2');
        expect(positions).toContain('2:3');
    });

    test('multiple_client_resumes_keep_every_pre_resume_anchor', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createClientRun');
        expect((await call<Outcome>(page, 'resumeViaClient')).ok).toBe(true);
        expect((await call<Outcome>(page, 'writeSnapshot')).ok).toBe(true);
        expect((await call<Outcome>(page, 'resumeViaClient')).ok).toBe(true);
        expect((await call<Outcome>(page, 'writeSnapshot')).ok).toBe(true);
        expect(await call<CursorView>(page, 'cursor')).toMatchObject({
            taskId: created.taskId,
            generation: 3,
            sequence: 4,
        });

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        const restored = await call<RestoreView>(page, 'open');
        expect(restored.phase).toBe('ready');
        expect(await call<CursorView>(page, 'cursor')).toMatchObject({
            taskId: created.taskId,
            generation: 3,
            sequence: 4,
        });

        const db = await readDb(page);
        const stored = db.snapshotRecords[0] as Record<string, unknown>;
        const records = Array.isArray(stored.anchors)
            ? [stored.latest as Record<string, unknown>, ...(stored.anchors as Array<Record<string, unknown>>)]
            : [stored];
        const positions = records.map(
            (record) => `${record.generation}:${record.last_sequence}`,
        );
        expect(positions).toContain('1:2');
        expect(positions).toContain('2:3');
        expect(positions).toContain('3:4');
    });

    test('export_import_round_trips_multiple_resume_anchors', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createClientRun');
        expect((await call<Outcome>(page, 'resumeViaClient')).ok).toBe(true);
        await call(page, 'writeSnapshot');
        expect((await call<Outcome>(page, 'resumeViaClient')).ok).toBe(true);
        await call(page, 'writeSnapshot');
        const exported = await call<string>(page, 'exportNdjson');

        await call(page, 'reset');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const imported = await call<Outcome>(page, 'importNdjson', exported);
        expect(imported.ok, JSON.stringify(imported)).toBe(true);
        expect(await call<CursorView>(page, 'cursor')).toMatchObject({
            taskId: created.taskId,
            generation: 3,
            sequence: 4,
        });
        expect((await readDb(page)).eventRecords).toHaveLength(5);
    });

    test('import_rejects_resume_without_its_anchor_before_writing', async ({ page }) => {
        await call<FixtureSummary>(page, 'createTaskAndEvents');
        await call(page, 'writeSnapshot');
        expect((await call<Outcome>(page, 'appendResume')).status).toBe('applied');
        const exported = await call<string>(page, 'exportNdjson');
        const withoutAnchors =
            exported
                .trim()
                .split('\n')
                .filter((line) => !line.includes('"kind":"snapshot"'))
                .join('\n') + '\n';

        await call(page, 'reset');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const rejected = await call<Outcome>(page, 'importNdjson', withoutAnchors);
        expect(rejected.ok).toBe(false);
        expect(['snapshot_missing', 'snapshot_corrupt']).toContain(rejected.code);
        expect((await readDb(page)).eventRecords).toHaveLength(0);
    });

    test('import_rejects_generation_two_without_generation_one', async ({ page }) => {
        const first = await call<FixtureSummary>(page, 'createTaskAndEvents', {
            mode: 'finished',
        });
        const second = await call<FixtureSummary>(page, 'createSecondRun');
        expect(second.generation).toBe(2);
        const exported = await call<string>(page, 'exportNdjson');
        const withoutFirstRun =
            exported
                .trim()
                .split('\n')
                .filter((line) => !line.includes(first.runId))
                .join('\n') + '\n';

        await call(page, 'reset');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const rejected = await call<Outcome>(page, 'importNdjson', withoutFirstRun);
        expect(rejected.ok).toBe(false);
        expect(['snapshot_corrupt', 'stale_generation']).toContain(rejected.code);
        expect((await readDb(page)).eventRecords).toHaveLength(0);
    });

    test('import_rejects_run_without_a_task_record', async ({ page }) => {
        await call<FixtureSummary>(page, 'createTaskAndEvents');
        const exported = await call<string>(page, 'exportNdjson');
        const withoutTask =
            exported
                .trim()
                .split('\n')
                .filter((line) => !line.includes('"kind":"task"'))
                .join('\n') + '\n';

        await call(page, 'reset');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const rejected = await call<Outcome>(page, 'importNdjson', withoutTask);
        expect(rejected.ok).toBe(false);
        expect(rejected.code).toBe('snapshot_corrupt');
        expect((await readDb(page)).eventRecords).toHaveLength(0);
    });

    test('import_requires_all_nine_stores_empty_including_leases', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        const exported = await call<string>(page, 'exportNdjson');

        await call(page, 'reset');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        await mutateDb(page, {
            op: 'put_lease',
            taskId: created.taskId,
            writerId: 'stale-writer',
            nowMs: 1_000_000,
        });
        expect((await readDb(page)).leaseRecords).toHaveLength(1);

        const rejected = await call<Outcome>(page, 'importNdjson', exported);
        expect(rejected.ok).toBe(false);
        expect(rejected.code).toBe('duplicate_event');
        expect((await readDb(page)).eventRecords).toHaveLength(0);
        expect((await readDb(page)).leaseRecords).toHaveLength(1);
    });

    test('orphan_run_without_a_task_is_refused', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        await mutateDb(page, { op: 'delete_task', taskId: created.taskId });

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        const restored = await call<RestoreView>(page, 'open');
        expect(restored.phase).toBe('error');
        expect(restored.error?.code).toBe('snapshot_corrupt');
        expect((await readDb(page)).eventRecords).toHaveLength(3);
    });

    test('no_event_run_does_not_report_a_latest_run', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        await mutateDb(page, {
            op: 'delete_run_events',
            taskId: created.taskId,
            runId: created.runId,
        });

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        const restored = await call<RestoreView>(page, 'open');
        expect(restored.phase).toBe('incomplete');
        const task = await call<TaskViewSummary>(page, 'taskView');
        expect(task.latestRunId).toBeNull();
        expect(task.unresolvedRunId).toBe(created.runId);
        expect(task.runs).toHaveLength(0);
    });

    test('mismatched_run_record_key_is_refused', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        await mutateDb(page, {
            op: 'mutate_run_identity',
            taskId: created.taskId,
            runId: created.runId,
            field: 'run_id',
            value: `run_${'a'.repeat(16)}`,
        });

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        const restored = await call<RestoreView>(page, 'open');
        expect(restored.phase).toBe('error');
        expect(restored.error?.code).toBe('snapshot_corrupt');
    });

    test('10000_small_events_hit_the_cap_with_scoped_rollback', async ({ page }) => {
        test.setTimeout(300_000);
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        const seeded = await call<{
            events: number;
            sequence: number;
            durationMs: number;
            heapUsed: number | null;
        }>(page, 'seedSmallEvents', 9_999);
        expect(seeded.events).toBe(9_999);
        expect(seeded.sequence).toBe(9_998);
        expect(created.taskId).toBeTruthy();

        await call(page, 'armAbortNextAppend');
        const aborted = await call<Outcome>(page, 'appendAssistant', 'rollback at cap');
        expect(aborted.status).toBe('rejected');
        expect(aborted.code).toBe('limit_exceeded');
        expect(aborted.storage).toBe('indexeddb');
        expect((await call<CursorView>(page, 'cursor')).sequence).toBe(9_998);

        const recovered = await call<Outcome>(page, 'appendAssistant', 'cap event');
        expect(recovered.status).toBe('applied');
        expect((await call<CursorView>(page, 'cursor')).sequence).toBe(9_999);

        const over = await call<Outcome>(page, 'appendAssistant', 'over cap');
        expect(over.status).toBe('rejected');
        expect(over.code).toBe('limit_exceeded');
        expect((await call<CursorView>(page, 'cursor')).sequence).toBe(9_999);

        const metrics = await call<{
            events: number;
            sequence: number;
            durationMs: number;
            heapUsed: number | null;
        } | null>(page, 'scaleMetrics');
        expect(metrics).not.toBeNull();
        console.log('scale_metrics', JSON.stringify(metrics));
    });

    test('stale_stored_latest_cannot_override_the_replayed_active_run', async ({ page }) => {
        const first = await call<FixtureSummary>(page, 'createTaskAndEvents', {
            mode: 'finished',
        });
        const second = await call<FixtureSummary>(page, 'createSecondRun');
        expect(second.generation).toBe(2);
        await mutateDb(page, {
            op: 'set_task_latest',
            taskId: first.taskId,
            latestRunId: first.runId,
        });

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        const restored = await call<RestoreView>(page, 'open');
        expect(restored.phase).toBe('error');
        expect(restored.error?.code).toBe('snapshot_corrupt');
        expect(restored.canStartRun).toBe(false);
        expect(restored.sendEnabled).toBe(false);
    });

    test('null_stored_latest_with_a_replayed_run_is_refused', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        await mutateDb(page, {
            op: 'set_task_latest',
            taskId: created.taskId,
            latestRunId: null,
        });

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        const restored = await call<RestoreView>(page, 'open');
        expect(restored.phase).toBe('error');
        expect(restored.error?.code).toBe('snapshot_corrupt');
        expect(restored.canStartRun).toBe(false);
        expect(restored.sendEnabled).toBe(false);
    });

    test('listTasks_keeps_unloaded_active_runs_as_incomplete', async ({ page }) => {
        for (const mode of ['running', 'cancelling'] as const) {
            const listed = await call<Outcome>(page, 'listUnloadedActiveTask', mode);
            expect(listed.ok, JSON.stringify(listed)).toBe(true);
            const summary = listed.record as {
                taskId: string;
                latestRunId: string;
            };
            const view = await call<TaskViewSummary>(page, 'taskView');
            expect(view.taskId).toBe(summary.taskId);
            expect(view.latestRunId).toBe(summary.latestRunId);
            expect(view.runs).toHaveLength(0);
            const state = await call<RestoreView>(page, 'restoreState');
            expect(state.canStartRun).toBe(false);
            expect(state.sendEnabled).toBe(false);
        }
    });

    test('import_gates_concurrent_append_and_publishes_only_complete_state', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        const exported = await call<string>(page, 'exportNdjson');
        await call(page, 'reset');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        await call(page, 'targetFixture', created.taskId, created.runId, 1);
        await call(page, 'beginImport', exported);
        const during = await call<Outcome>(page, 'appendAssistantSafe', 'during import');
        expect(during.ok).toBe(false);
        expect(during.code).toBe('runtime_unavailable');
        const imported = await call<Outcome>(page, 'whenImport');
        expect(imported.ok, JSON.stringify(imported)).toBe(true);
        expect(await call<CursorView>(page, 'cursor')).toMatchObject({
            taskId: created.taskId,
            sequence: 2,
        });
    });

    test('pre_gate_held_append_cannot_commit_after_import_or_be_deleted', async ({ page }) => {
        await call<FixtureSummary>(page, 'createTaskAndEvents');
        const exported = await call<string>(page, 'exportNdjson');
        await call(page, 'reset');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');

        const heldTask = 'tsk_aaaaaaaaaaaaaaaa';
        const heldRun = 'run_bbbbbbbbbbbbbbbb';
        await call(page, 'beginHeldQueuedAppend', heldTask, heldRun);
        await call(page, 'awaitAppendHeld');
        await call(page, 'beginImport', exported);
        expect((await call<RestoreView>(page, 'restoreState')).phase).toBe('loading');
        await call(page, 'releaseHeldAppend');
        const held = await call<Outcome>(page, 'finishHeldAppend');
        expect(held.status).toBe('applied');
        const imported = await call<Outcome>(page, 'whenImport');
        expect(imported.ok).toBe(false);
        expect(imported.code).toBe('duplicate_event');

        const db = await readDb(page);
        expect(db.eventRecords).toHaveLength(1);
        expect(db.runRecords.some((run) => run.run_id === heldRun)).toBe(true);
        expect(db.eventRecords[0]!.event_id).toContain(heldRun);
    });

    test('pre_gate_append_blocks_a_successful_import_until_it_commits', async ({ page }) => {
        await call<FixtureSummary>(page, 'createTaskAndEvents');
        const exported = await call<string>(page, 'exportNdjson');
        await call(page, 'reset');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');

        const heldTask = 'tsk_aaaaaaaaaaaaaaaa';
        const heldRun = 'run_bbbbbbbbbbbbbbbb';
        await call(page, 'beginHeldQueuedAppend', heldTask, heldRun);
        await call(page, 'awaitAppendHeld');
        await call(page, 'beginImport', exported);
        const importOutcome = call<Outcome>(page, 'whenImport');
        const raced = await Promise.race([
            importOutcome.then((result) => ({ kind: 'import' as const, result })),
            page.waitForTimeout(3_000).then(() => ({ kind: 'blocked' as const })),
        ]);
        // The import must stay fenced while a write that started before it is still in flight.
        expect(raced.kind).toBe('blocked');

        await call(page, 'releaseHeldAppend');
        const held = await call<Outcome>(page, 'finishHeldAppend');
        expect(held.status).toBe('applied');
        const imported = await importOutcome;
        expect(imported.ok).toBe(false);
        expect(imported.code).toBe('duplicate_event');

        const db = await readDb(page);
        expect(db.eventRecords).toHaveLength(1);
        expect(db.eventRecords[0]!.event_id).toContain(heldRun);
        expect(db.leaseRecords).toHaveLength(0);
    });

    test('cross_tab_import_fence_blocks_a_foreign_append', async ({ page, context }) => {
        await call<FixtureSummary>(page, 'createTaskAndEvents');
        const exported = await call<string>(page, 'exportNdjson');
        await call(page, 'reset');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');

        const pageTwo = await context.newPage();
        await openStudio(pageTwo);
        expect((await call<RestoreView>(pageTwo, 'open')).phase).toBe('ready');

        await call(page, 'beginImport', exported);
        expect((await call<RestoreView>(page, 'restoreState')).phase).toBe('loading');
        const blocked = await call<Outcome>(
            pageTwo,
            'appendQueuedSafe',
            'tsk_cccccccccccccccc',
            'run_dddddddddddddddd',
        );
        expect(blocked.ok).toBe(false);
        expect(blocked.code).toBe('stale_generation');
        expect((await call<CursorView | null>(pageTwo, 'cursor'))).toBeNull();

        const imported = await call<Outcome>(page, 'whenImport');
        expect(imported.ok, JSON.stringify(imported)).toBe(true);
        await pageTwo.close();
    });

    test('long_import_heartbeat_keeps_the_fence_live_and_blocks_takeover', async ({ page, context }) => {
        await call<FixtureSummary>(page, 'createTaskAndEvents');
        const exported = await call<string>(page, 'exportNdjson');
        await call(page, 'reset');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');

        const pageTwo = await context.newPage();
        await openStudio(pageTwo);
        expect((await call<RestoreView>(pageTwo, 'open')).phase).toBe('ready');

        await call(page, 'beginImport', exported);
        const acquired = await readDb(page);
        expect(acquired.leaseRecords).toHaveLength(1);
        const leaseBefore = acquired.leaseRecords[0] as {
            started_at_ms: number;
            heartbeat_at_ms: number;
        };
        const advanced = await call<Outcome>(page, 'advanceImportClock', 45_000);
        expect(advanced.ok, JSON.stringify(advanced)).toBe(true);
        expect((await call<Outcome>(pageTwo, 'advanceImportClock', 45_000)).ok).toBe(true);
        const blocked = await call<Outcome>(pageTwo, 'importNdjson', exported);
        expect(blocked.ok).toBe(false);
        expect(blocked.code).toBe('stale_generation');

        const refreshed = (await readDb(page)).leaseRecords[0] as {
            started_at_ms: number;
            heartbeat_at_ms: number;
        };
        expect(refreshed.started_at_ms).toBe(leaseBefore.started_at_ms);
        expect(refreshed.heartbeat_at_ms).toBe(leaseBefore.heartbeat_at_ms + 45_000);
        const untouched = await readDb(page);
        expect(untouched.eventRecords).toHaveLength(0);
        expect(untouched.taskRecords).toHaveLength(0);
        expect(untouched.leaseRecords).toHaveLength(1);

        const imported = await call<Outcome>(page, 'whenImport');
        expect(imported.ok, JSON.stringify(imported)).toBe(true);
        expect(await call<CursorView>(page, 'cursor')).toMatchObject({ sequence: 2 });
        const committed = await readDb(page);
        expect(committed.leaseRecords).toHaveLength(0);
        expect(committed.taskRecords).toHaveLength(1);
        await pageTwo.close();
    });

    test('frozen_importers_late_cleanup_preserves_the_takeover_records', async ({ page, context }) => {
        await call<FixtureSummary>(page, 'createTaskAndEvents');
        const exported = await call<string>(page, 'exportNdjson');
        await call(page, 'reset');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');

        const pageTwo = await context.newPage();
        await openStudio(pageTwo);
        expect((await call<RestoreView>(pageTwo, 'open')).phase).toBe('ready');

        await call(page, 'beginImport', exported);
        expect((await call<Outcome>(page, 'freezeImportClock', 60_000)).ok).toBe(true);
        expect((await call<Outcome>(pageTwo, 'advanceImportClock', 60_000)).ok).toBe(true);
        await call(pageTwo, 'beginImport', exported);
        const takeoverLease = (await readDb(page)).leaseRecords[0] as {
            started_at_ms: number;
            heartbeat_at_ms: number;
        };
        expect(takeoverLease.started_at_ms).toBe(takeoverLease.heartbeat_at_ms);

        const first = await call<Outcome>(page, 'whenImport');
        expect(first.ok).toBe(false);
        expect(first.code).toBe('stale_generation');

        const takeover = await call<Outcome>(pageTwo, 'whenImport');
        expect(takeover.ok, JSON.stringify(takeover)).toBe(true);
        const committed = await readDb(page);
        expect(committed.taskRecords).toHaveLength(1);
        expect(committed.leaseRecords).toHaveLength(0);
        expect(committed.eventRecords.length).toBeGreaterThan(0);
        expect(await call<CursorView>(pageTwo, 'cursor')).toMatchObject({
            sequence: 2,
        });
        await pageTwo.close();
    });

    test('client_active_stale_and_null_summaries_reopen_from_event_derived_latest', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        const stale = await call<Outcome>(
            page,
            'listActiveSummary',
            created.taskId,
            'run_eeeeeeeeeeeeeeee',
            'running',
        );
        expect(stale.ok, JSON.stringify(stale)).toBe(true);
        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        expect(await call<CursorView>(page, 'cursor')).toMatchObject({
            taskId: created.taskId,
            runId: created.runId,
            sequence: 2,
        });

        const nullLatest = await call<Outcome>(
            page,
            'listActiveSummary',
            created.taskId,
            null,
            'running',
        );
        expect(nullLatest.ok, JSON.stringify(nullLatest)).toBe(true);
        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        expect(await call<CursorView>(page, 'cursor')).toMatchObject({
            runId: created.runId,
            sequence: 2,
        });
    });

    test('pin_archive_preserve_unresolved_latest_and_block_send', async ({ page }) => {
        const listed = await call<Outcome>(page, 'listUnloadedActiveTask', 'running');
        expect(listed.ok).toBe(true);
        const summary = listed.record as { taskId: string; latestRunId: string };

        expect((await call<Outcome>(page, 'pin', true)).ok).toBe(true);
        let view = await call<TaskViewSummary>(page, 'taskView');
        expect(view.unresolvedRunId).toBe(summary.latestRunId);
        expect((await call<RestoreView>(page, 'restoreState')).canStartRun).toBe(false);
        expect((await call<Outcome>(page, 'archive', true)).ok).toBe(true);

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        const restored = await call<RestoreView>(page, 'open');
        expect(restored.phase).toBe('incomplete');
        expect(restored.canStartRun).toBe(false);
        view = await call<TaskViewSummary>(page, 'taskView');
        expect(view.unresolvedRunId).toBe(summary.latestRunId);
        expect(view.runs).toHaveLength(0);
    });

    test('oversized_bundle_blocks_reopen', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        expect((await call<Outcome>(page, 'writeSnapshot')).ok).toBe(true);
        await mutateDb(page, {
            op: 'oversize_snapshot',
            taskId: created.taskId,
            runId: created.runId,
            bytes: 9 * 1024 * 1024,
        });
        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        const restored = await call<RestoreView>(page, 'open');
        expect(restored.phase).toBe('error');
        expect(restored.error?.code).toBe('limit_exceeded');
        expect(restored.canStartRun).toBe(false);
    });

    test('corrupt_anchor_blocks_append_commit_and_export', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        expect((await call<Outcome>(page, 'writeSnapshot')).ok).toBe(true);
        expect((await call<Outcome>(page, 'appendResume')).status).toBe('applied');
        expect((await call<Outcome>(page, 'writeSnapshot')).ok).toBe(true);
        await mutateDb(page, {
            op: 'corrupt_anchor',
            taskId: created.taskId,
            runId: created.runId,
            index: 0,
        });
        const append = await call<Outcome>(page, 'appendResume');
        expect(append.status).toBe('rejected');
        expect(append.code).toBe('snapshot_corrupt');
        await expect(call<string>(page, 'exportNdjson')).rejects.toThrow();
    });

    test('export_rejects_oversized_bundle_and_fabricated_change_files', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        expect((await call<Outcome>(page, 'writeSnapshot')).ok).toBe(true);
        await mutateDb(page, {
            op: 'oversize_snapshot',
            taskId: created.taskId,
            runId: created.runId,
            bytes: 9 * 1024 * 1024,
        });
        await expect(call<string>(page, 'exportNdjson')).rejects.toThrow();

        await call(page, 'reset');
        const second = await call<FixtureSummary>(page, 'createTaskAndEvents');
        expect((await call<Outcome>(page, 'appendChanges')).status).toBe('applied');
        expect((await call<Outcome>(page, 'recordChanges')).ok).toBe(true);
        await mutateDb(page, {
            op: 'corrupt_change_files',
            taskId: second.taskId,
            runId: second.runId,
            headDigest: 'turn:3:fixture',
            files: [{ path: 'src/fabricated.ts', status: 'added' }],
        });
        await expect(call<string>(page, 'exportNdjson')).rejects.toThrow();
    });

    test('dispose_during_import_does_not_throw_or_leave_an_unhandled_rejection', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        const exported = await call<string>(page, 'exportNdjson');
        await call(page, 'reset');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        await call(page, 'beginImport', exported);
        await call(page, 'disposeDuringImport');
        await call(page, 'whenImport');
        expect(created.taskId).toBeTruthy();
    });

    test('import_cleanup_failure_is_surfaced_and_blocks_ready', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        const exported = await call<string>(page, 'exportNdjson');
        await call(page, 'reset');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        await call(page, 'armFailImportPublish');
        await call(page, 'armFailImportCleanup');
        await call(page, 'beginImport', exported);
        const failed = await call<Outcome>(page, 'whenImport');
        expect(failed.ok).toBe(false);
        expect(failed.code).toBe('limit_exceeded');
        expect(failed.storage).toBe('indexeddb');
        expect((await call<RestoreView>(page, 'restoreState')).phase).toBe('error');
        const importedEvidence = await readDb(page);
        expect(importedEvidence.eventRecords.length).toBeGreaterThan(0);
        expect(importedEvidence.taskRecords).toHaveLength(1);
        expect(importedEvidence.leaseRecords).toHaveLength(1);

        const reopened = await call<RestoreView>(page, 'open');
        expect(reopened.phase).toBe('error');
        expect(reopened.error?.code).toBe('runtime_unavailable');
        expect(created.taskId).toBeTruthy();
    });

    test('import_publish_failure_leaves_no_ready_or_partial_state', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        const exported = await call<string>(page, 'exportNdjson');
        await call(page, 'reset');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        await call(page, 'armFailImportPublish');
        await call(page, 'beginImport', exported);
        const failed = await call<Outcome>(page, 'whenImport');
        expect(failed.ok).toBe(false);
        const state = await call<RestoreView>(page, 'restoreState');
        expect(state.phase).not.toBe('ready');
        expect(state.canStartRun).toBe(false);
        expect(state.sendEnabled).toBe(false);
        const reopened = await call<RestoreView>(page, 'open');
        expect(reopened.phase).toBe('ready');
        expect(await call<CursorView | null>(page, 'cursor')).toBeNull();
        expect((await readDb(page)).eventRecords).toHaveLength(0);
        expect(created.taskId).toBeTruthy();
    });

    test('corrupt_non_latest_anchor_is_refused_by_read_lease_and_write', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        expect((await call<Outcome>(page, 'writeSnapshot')).ok).toBe(true);
        expect((await call<Outcome>(page, 'appendResume')).status).toBe('applied');
        expect((await call<Outcome>(page, 'writeSnapshot')).ok).toBe(true);
        await mutateDb(page, {
            op: 'corrupt_anchor',
            taskId: created.taskId,
            runId: created.runId,
            index: 0,
        });

        const read = await call<Outcome>(page, 'readSnapshot');
        expect(read.ok).toBe(false);
        expect(read.code).toBe('snapshot_corrupt');
        const lease = await call<Outcome>(page, 'acquireLease', 'writer-a', 1_000_000);
        expect(lease.ok).toBe(false);
        expect(lease.code).toBe('snapshot_corrupt');
        const write = await call<Outcome>(page, 'writeSnapshot');
        expect(write.ok).toBe(false);
        expect(write.code).toBe('snapshot_corrupt');
    });

    test('oversized_snapshot_bundle_is_refused_by_read_lease_write_and_append', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        expect((await call<Outcome>(page, 'writeSnapshot')).ok).toBe(true);
        await mutateDb(page, {
            op: 'oversize_snapshot',
            taskId: created.taskId,
            runId: created.runId,
            bytes: 9 * 1024 * 1024,
        });

        const read = await call<Outcome>(page, 'readSnapshot');
        expect(read.ok).toBe(false);
        expect(read.code).toBe('limit_exceeded');
        const lease = await call<Outcome>(page, 'acquireLease', 'writer-a', 1_000_000);
        expect(lease.ok).toBe(false);
        expect(lease.code).toBe('limit_exceeded');
        const write = await call<Outcome>(page, 'writeSnapshot');
        expect(write.ok).toBe(false);
        expect(write.code).toBe('limit_exceeded');
        const append = await call<Outcome>(page, 'appendResume');
        expect(append.status).toBe('rejected');
        expect(append.code).toBe('limit_exceeded');
    });

    test('import_and_read_rebind_change_files_to_the_snapshot_event', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        expect((await call<Outcome>(page, 'appendChanges')).status).toBe('applied');
        expect((await call<Outcome>(page, 'recordChanges')).ok).toBe(true);
        const exported = await call<string>(page, 'exportNdjson');
        await mutateDb(page, {
            op: 'corrupt_change_files',
            taskId: created.taskId,
            runId: created.runId,
            headDigest: 'turn:3:fixture',
            files: [{ path: 'src/fabricated.ts', status: 'added' }],
        });
        const read = await call<Outcome>(page, 'readChanges');
        expect(read.ok).toBe(false);
        expect(read.code).toBe('snapshot_corrupt');

        const tampered = exported
            .trim()
            .split('\n')
            .map((line) => {
                if (!line.includes('"kind":"changes"')) return line;
                const parsed = JSON.parse(line) as {
                    record: { files: Array<Record<string, unknown>> };
                };
                parsed.record.files = [{ path: 'src/fabricated.ts', status: 'added' }];
                return JSON.stringify(parsed);
            })
            .join('\n') + '\n';
        await call(page, 'reset');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const imported = await call<Outcome>(page, 'importNdjson', tampered);
        expect(imported.ok).toBe(false);
        expect(imported.code).toBe('snapshot_corrupt');
        expect((await readDb(page)).eventRecords).toHaveLength(0);
    });

    test('queued_only_run_persists_the_queued_event_time', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createQueuedOnlyTask');
        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        const restored = await call<RestoreView>(page, 'open');
        expect(restored.phase).toBe('ready');
        const db = await readDb(page);
        const run = db.runRecords.find(
            (record) => record.run_id === created.runId,
        ) as Record<string, unknown>;
        expect(run.started_at_ms).toBe(1_794_283_200_000);
    });

    test('actual_transaction_abort_preserves_memory_and_durable_cursor', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        await call(page, 'armAbortNextAppend');
        const aborted = await call<Outcome>(page, 'appendAssistant', 'never durable');
        expect(aborted.status).toBe('rejected');
        expect(aborted.code).toBe('limit_exceeded');
        expect(aborted.storage).toBe('indexeddb');

        const cursor = await call<CursorView>(page, 'cursor');
        expect(cursor.sequence).toBe(2);
        expect(cursor.messageCount).toBe(1);
        expect(cursor.lastMessageText).toBe('replayed fixture message');

        let db = await readDb(page);
        expect(db.eventRecords).toHaveLength(3);
        expect(db.runRecords[0]!.last_sequence).toBe(2);
        expect(db.runRecords[0]!.last_digest).toBe(created.digests[2]);

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        expect((await call<CursorView>(page, 'cursor')).sequence).toBe(2);

        const recovered = await call<Outcome>(page, 'appendAssistant', 'durable after abort');
        expect(recovered.status).toBe('applied');
        expect((await call<CursorView>(page, 'cursor')).sequence).toBe(3);
        db = await readDb(page);
        expect(db.eventRecords).toHaveLength(4);
        expect(db.runRecords[0]!.last_sequence).toBe(3);
    });

    test('invalid_now_ms_is_invalid_arguments_not_a_storage_limit', async ({ page }) => {
        await call<FixtureSummary>(page, 'createTaskAndEvents');
        const result = await call<Outcome>(page, 'appendAssistant', 'bad timestamp', {
            nowMs: -1,
        });
        expect(result.status).toBe('rejected');
        expect(result.code).toBe('invalid_arguments');
        expect(result.storage).toBeNull();
        expect((await call<CursorView>(page, 'cursor')).sequence).toBe(2);
        expect((await readDb(page)).eventRecords).toHaveLength(3);
    });

    test('invalid_created_at_ms_is_invalid_arguments', async ({ page }) => {
        await call<FixtureSummary>(page, 'createTaskAndEvents');
        const result = await call<Outcome>(page, 'createTaskAt', -1);
        expect(result.ok).toBe(false);
        expect(result.code).toBe('invalid_arguments');
        expect((await call<TaskViewSummary>(page, 'taskView')).title).toBe('Fixture task');
    });

    test('task_write_abort_rolls_back_metadata_in_memory', async ({ page }) => {
        await call<FixtureSummary>(page, 'createTaskAndEvents');
        await call(page, 'armAbortNextTaskWrite');
        const failedPin = await call<Outcome>(page, 'pin', true);
        expect(failedPin.ok).toBe(false);
        expect(failedPin.code).toBe('limit_exceeded');
        expect((await call<TaskViewSummary>(page, 'taskView')).pinned).toBe(false);
        expect((await call<Outcome>(page, 'pin', true)).ok).toBe(true);
        expect((await call<TaskViewSummary>(page, 'taskView')).pinned).toBe(true);

        await call(page, 'reset');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        await call(page, 'armAbortNextTaskWrite');
        const failedCreate = await call<Outcome>(page, 'createTaskAt', 1_794_283_200_000);
        expect(failedCreate.ok).toBe(false);
        expect(failedCreate.code).toBe('limit_exceeded');
        expect(await call<TaskViewSummary | null>(page, 'taskView')).toBeNull();
        expect((await call<Outcome>(page, 'createTaskAt', 1_794_283_200_000)).ok).toBe(true);
    });

    test('versionchange_resets_the_cached_open_state', async ({ page }) => {
        await call<FixtureSummary>(page, 'createTaskAndEvents');
        expect((await call<RestoreView>(page, 'restoreState')).phase).toBe('ready');

        await upgradeDatabaseVersion(page);
        const reopened = await call<RestoreView>(page, 'open');
        expect(reopened.phase).toBe('error');
        expect(reopened.error?.code).toBe('runtime_unavailable');
    });

    test('two_page_lease_fencing_stops_a_stale_writer', async ({ page, context }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        expect((await call<Outcome>(page, 'acquireLease', 'writer-a', 1_000_000)).ok).toBe(true);

        const pageTwo = await context.newPage();
        await openStudio(pageTwo);
        expect((await call<RestoreView>(pageTwo, 'open')).phase).toBe('ready');
        expect((await call<CursorView>(pageTwo, 'cursor')).taskId).toBe(created.taskId);

        const early = await call<Outcome>(pageTwo, 'acquireLease', 'writer-b', 1_020_000);
        expect(early.ok).toBe(false);
        expect(early.code).toBe('stale_generation');

        const takeover = await call<Outcome>(pageTwo, 'acquireLease', 'writer-b', 1_031_000);
        expect(takeover.ok).toBe(true);

        const stale = await call<Outcome>(page, 'appendAssistant', 'from writer a', {
            writerId: 'writer-a',
            nowMs: 1_032_000,
        });
        expect(stale.status).toBe('rejected');
        expect(stale.code).toBe('stale_generation');
        expect((await call<CursorView>(page, 'cursor')).sequence).toBe(2);

        const accepted = await call<Outcome>(pageTwo, 'appendAssistant', 'from writer b', {
            writerId: 'writer-b',
            nowMs: 1_032_000,
        });
        expect(accepted.status).toBe('applied');
        expect((await call<CursorView>(pageTwo, 'cursor')).sequence).toBe(3);

        const db = await readDb(pageTwo);
        expect(db.eventRecords).toHaveLength(4);
        expect(db.eventRecords.filter((record) => record.event_id.endsWith('000000000003'))).toHaveLength(1);
        expect(db.eventRecords[3]!.canonical).toContain('from writer b');
        expect(db.eventRecords[3]!.canonical).not.toContain('from writer a');

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        expect(await call<CursorView>(page, 'cursor')).toMatchObject({
            sequence: 3,
            lastMessageText: 'from writer b',
        });
        await pageTwo.close();
    });

    test('restored_active_run_is_replayed_before_send_can_start', async ({ page }) => {
        await call<FixtureSummary>(page, 'createTaskAndEvents', { mode: 'cancelling' });

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await call(page, 'beginOpen');
        const loading = await call<RestoreView>(page, 'restoreState');
        expect(loading.phase).toBe('loading');
        expect(loading.sendEnabled).toBe(false);
        expect(loading.canStartRun).toBe(false);
        // The production owner opens before the typed mount, so the active run
        // is already replayed when this artificial loading window begins.
        expect(loading.activeRun).toMatchObject({
            generation: 1,
            lastSequence: 2,
            state: 'running',
            statusProjection: 'cancelling',
        });

        const ready = await call<RestoreView>(page, 'whenOpen');
        expect(ready.phase).toBe('ready');
        expect(ready.sendEnabled).toBe(false);
        expect(ready.canStartRun).toBe(false);
        expect(ready.activeRun).toMatchObject({
            generation: 1,
            lastSequence: 2,
            state: 'running',
            statusProjection: 'cancelling',
        });
        expect(await call<CursorView>(page, 'cursor')).toMatchObject({
            sequence: 2,
            runState: 'running',
            statusProjection: 'cancelling',
        });
    });

    test('background_hold_survives_reload_refuses_new_run_and_resolves', async ({ page }) => {
        const seeded = await call<
            Outcome & {
                record?: {
                    taskId: string;
                    runId: string;
                    toolCallId: string;
                };
            }
        >(page, 'seedBackgroundHoldForTest');
        expect(seeded.ok, JSON.stringify(seeded)).toBe(true);
        const hold = seeded.record!;

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        const restored = await call<RestoreView>(page, 'open');
        expect(restored.phase).toBe('ready');
        expect(restored.activeRun).toBeNull();
        expect(restored.canStartRun).toBe(false);
        expect(restored.sendEnabled).toBe(false);

        const heldTask = await call<TaskViewSummary>(page, 'taskView');
        expect(heldTask.state).toBe('waiting_for_background');
        expect(heldTask.hold).toEqual({
            reason: 'waiting_for_background',
            run_id: hold.runId,
            tool_call_id: hold.toolCallId,
        });
        const before = await call<CursorView>(page, 'cursor');

        const refused = await call<Outcome>(page, 'startRunOnHeldTaskForTest', hold.taskId);
        expect(refused.ok, JSON.stringify(refused)).toBe(false);
        expect(refused.code).toBe('out_of_order');

        const resolved = await call<Outcome>(
            page,
            'resolveBackgroundHoldForTest',
            hold.taskId,
            hold.runId,
            hold.toolCallId,
        );
        expect(resolved.ok, JSON.stringify(resolved)).toBe(true);
        const cleared = await call<TaskViewSummary>(page, 'taskView');
        expect(cleared.state).toBe('failed');
        expect(cleared.hold).toBeNull();
        const after = await call<CursorView>(page, 'cursor');
        expect(after).toMatchObject({
            taskId: before!.taskId,
            runId: before!.runId,
            generation: before!.generation,
            sequence: before!.sequence,
            eventId: before!.eventId,
            digest: before!.digest,
            runState: 'failed',
        });
    });

    test('restored_finished_run_allows_send_and_missing_run_is_incomplete', async ({ page }) => {
        await call<FixtureSummary>(page, 'createTaskAndEvents', { mode: 'finished' });
        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        const ready = await call<RestoreView>(page, 'open');
        expect(ready.phase).toBe('ready');
        expect(ready.activeRun).toBeNull();
        expect(ready.canStartRun).toBe(true);
        expect(ready.sendEnabled).toBe(true);

        await call(page, 'reset');
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        await mutateDb(page, { op: 'delete_run', taskId: created.taskId, runId: created.runId });
        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        const incomplete = await call<RestoreView>(page, 'open');
        expect(incomplete.phase).toBe('incomplete');
        expect(incomplete.incomplete).toHaveLength(1);
        expect(incomplete.incomplete[0]).toMatchObject({ taskId: created.taskId, code: 'snapshot_missing' });
        expect(incomplete.canStartRun).toBe(false);
        expect(incomplete.sendEnabled).toBe(false);
        const task = await call<TaskViewSummary>(page, 'taskView');
        expect(task.latestRunId).toBeNull();
        expect(task.runs).toHaveLength(0);
        expect((await readDb(page)).eventRecords).toHaveLength(3);
    });

    test('later_generation_without_its_earlier_run_is_refused', async ({ page }) => {
        const first = await call<FixtureSummary>(page, 'createTaskAndEvents', { mode: 'finished' });
        const second = await call<FixtureSummary>(page, 'createSecondRun');
        expect(second.generation).toBe(2);

        await mutateDb(page, { op: 'delete_run', taskId: first.taskId, runId: first.runId });
        await mutateDb(page, {
            op: 'delete_event',
            taskId: first.taskId,
            runId: first.runId,
            generation: 1,
            sequence: 0,
        });
        await mutateDb(page, {
            op: 'delete_event',
            taskId: first.taskId,
            runId: first.runId,
            generation: 1,
            sequence: 1,
        });
        await mutateDb(page, {
            op: 'delete_event',
            taskId: first.taskId,
            runId: first.runId,
            generation: 1,
            sequence: 2,
        });

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        const restored = await call<RestoreView>(page, 'open');
        expect(restored.phase).toBe('error');
        expect(restored.error?.code).toBe('snapshot_corrupt');
        expect(await call<CursorView | null>(page, 'cursor')).toBeNull();
    });

    test('read_snapshot_missing_is_distinct_from_corruption', async ({ page }) => {
        await call<FixtureSummary>(page, 'createTaskAndEvents');
        const missing = await call<Outcome>(page, 'readSnapshot');
        expect(missing.ok).toBe(false);
        expect(missing.code).toBe('snapshot_missing');

        await call(page, 'writeSnapshot');
        const present = await call<Outcome>(page, 'readSnapshot');
        expect(present.ok).toBe(true);
        expect(present.record).toMatchObject({ last_sequence: 2 });

        await call(page, 'reset');
        const fresh = await call<FixtureSummary>(page, 'createTaskAndEvents');
        await mutateDb(page, { op: 'delete_snapshot', taskId: fresh.taskId, runId: fresh.runId });
        const stillMissing = await call<Outcome>(page, 'readSnapshot');
        expect(stillMissing.ok).toBe(false);
        expect(stillMissing.code).toBe('snapshot_missing');
    });

    test('pre_apply_held_append_is_drained_before_import_and_survives_the_failed_import', async ({ page }) => {
        await call<FixtureSummary>(page, 'createTaskAndEvents');
        const exported = await call<string>(page, 'exportNdjson');
        await call(page, 'reset');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');

        const heldTask = 'tsk_aaaaaaaaaaaaaaaa';
        const heldRun = 'run_bbbbbbbbbbbbbbbb';
        await call(page, 'beginHeldQueuedAppend', heldTask, heldRun);
        await call(page, 'awaitAppendHeld');
        await call(page, 'beginImport', exported);
        const importOutcome = call<Outcome>(page, 'whenImport');
        const raced = await Promise.race([
            importOutcome.then((result) => ({ kind: 'import' as const, result })),
            page.waitForTimeout(5_000).then(() => ({ kind: 'blocked' as const })),
        ]);
        // The barrier starts before the digest and in-memory apply, so the import drains this writer.
        expect(raced.kind).toBe('blocked');

        await call(page, 'releaseHeldAppend');
        const held = await call<Outcome>(page, 'finishHeldAppend');
        expect(held.status).toBe('applied');
        const imported = await importOutcome;
        expect(imported.ok).toBe(false);
        expect(imported.code).toBe('duplicate_event');

        const db = await readDb(page);
        expect(db.eventRecords).toHaveLength(1);
        expect(db.eventRecords[0]!.event_id).toContain(heldRun);
        expect(db.taskRecords).toHaveLength(1);
        expect((db.taskRecords[0] as { task_id: string }).task_id).toBe(heldTask);
        expect(db.leaseRecords).toHaveLength(0);
        // The failed import publishes the post-drain checkpoint, not a stale pre-drain copy.
        expect(await call<CursorView | null>(page, 'memoryCursor')).toMatchObject({
            taskId: heldTask,
            runId: heldRun,
            sequence: 0,
        });
    });

    test('pre_write_task_upsert_is_drained_before_import_and_leaves_no_orphan', async ({ page }) => {
        await call<FixtureSummary>(page, 'createTaskAndEvents');
        const exported = await call<string>(page, 'exportNdjson');
        await call(page, 'reset');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');

        const heldTask = 'tsk_eeeeeeeeeeeeeeee';
        await call(page, 'beginHeldTaskWrite', heldTask);
        await call(page, 'awaitTaskWriteHeld');
        await call(page, 'beginImport', exported);
        const importOutcome = call<Outcome>(page, 'whenImport');
        const raced = await Promise.race([
            importOutcome.then((result) => ({ kind: 'import' as const, result })),
            page.waitForTimeout(5_000).then(() => ({ kind: 'blocked' as const })),
        ]);
        // The in-memory upsert is inside the barrier; the import must wait for its durable write.
        expect(raced.kind).toBe('blocked');

        await call(page, 'releaseTaskWrite');
        const written = await call<Outcome>(page, 'finishHeldTaskWrite');
        expect(written.ok, JSON.stringify(written)).toBe(true);
        const imported = await importOutcome;
        expect(imported.ok).toBe(false);
        expect(imported.code).toBe('duplicate_event');

        const db = await readDb(page);
        expect(db.taskRecords).toHaveLength(1);
        expect((db.taskRecords[0] as { task_id: string }).task_id).toBe(heldTask);
        expect(db.eventRecords).toHaveLength(0);
        expect(db.leaseRecords).toHaveLength(0);
        const view = await call<TaskViewSummary | null>(page, 'taskView');
        expect(view?.taskId).toBe(heldTask);
        expect((await call<RestoreView>(page, 'restoreState')).phase).not.toBe('ready');
    });

    test('import_publishes_before_releasing_the_fence_so_no_cross_tab_write_slips_in', async ({ page, context }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        const exported = await call<string>(page, 'exportNdjson');
        await call(page, 'reset');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');

        const pageTwo = await context.newPage();
        await openStudio(pageTwo);
        expect((await call<RestoreView>(pageTwo, 'open')).phase).toBe('ready');

        await call(page, 'beginImport', exported, { holdPublish: true });
        const importOutcome = call<Outcome>(page, 'whenImport');
        await call(page, 'awaitImportPublishGate');

        // The first tab has published its verified checkpoint and still owns the fence.
        const slipped = await call<Outcome>(
            pageTwo,
            'appendQueuedSafe',
            'tsk_cccccccccccccccc',
            'run_dddddddddddddddd',
        );
        expect(slipped.ok).toBe(false);
        expect(slipped.code).toBe('stale_generation');

        await call(page, 'releaseImportPublishGate');
        const imported = await importOutcome;
        expect(imported.ok, JSON.stringify(imported)).toBe(true);
        expect(await call<CursorView>(page, 'cursor')).toMatchObject({
            taskId: created.taskId,
            runId: created.runId,
            sequence: 2,
        });
        const db = await readDb(page);
        expect(db.taskRecords).toHaveLength(1);
        expect((db.taskRecords[0] as { task_id: string }).task_id).toBe(created.taskId);
        expect(db.eventRecords).toHaveLength(3);
        expect(db.leaseRecords).toHaveLength(0);
        await pageTwo.close();
    });

    test('ordinary_append_rejects_a_corrupt_existing_anchor_without_advancing_the_cursor', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        expect((await call<Outcome>(page, 'writeSnapshot')).ok).toBe(true);
        expect((await call<Outcome>(page, 'appendResume')).status).toBe('applied');
        expect((await call<Outcome>(page, 'writeSnapshot')).ok).toBe(true);
        await mutateDb(page, {
            op: 'corrupt_anchor',
            taskId: created.taskId,
            runId: created.runId,
            index: 0,
        });

        const before = await call<CursorView>(page, 'cursor');
        const append = await call<Outcome>(
            page,
            'appendAssistantSafe',
            'ordinary event after a corrupt anchor',
        );
        expect(append.status).toBe('rejected');
        expect(append.code).toBe('snapshot_corrupt');
        expect(await call<CursorView>(page, 'cursor')).toEqual(before);
        expect((await readDb(page)).eventRecords).toHaveLength(4);
    });

    test('ordinary_append_rejects_an_oversized_existing_bundle_without_advancing_the_cursor', async ({ page }) => {
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        expect((await call<Outcome>(page, 'writeSnapshot')).ok).toBe(true);
        await mutateDb(page, {
            op: 'oversize_snapshot',
            taskId: created.taskId,
            runId: created.runId,
            bytes: 9 * 1024 * 1024,
        });

        const before = await call<CursorView>(page, 'cursor');
        const append = await call<Outcome>(
            page,
            'appendAssistantSafe',
            'ordinary event after an oversized bundle',
        );
        expect(append.status).toBe('rejected');
        expect(append.code).toBe('limit_exceeded');
        expect(await call<CursorView>(page, 'cursor')).toEqual(before);
        expect((await readDb(page)).eventRecords).toHaveLength(3);
    });
});

test('no_query_browser_exposes_no_studio_test_api', async ({ page }) => {
    await page.goto('/?typed=1');
    await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
    expect(await page.evaluate(() => '__studioTest' in window)).toBe(false);
    expect(await page.evaluate(() => Object.keys(window).filter((key) => key.toLowerCase().includes('studiotest')))).toEqual([]);
});

interface LiveTaskSeed {
    taskId: string;
    runId: string;
    generation: number;
    title: string;
    prompt: string;
}

interface LiveRunSnapshot {
    taskId: string;
    title: string;
    taskState: string;
    hold: {
        reason: 'waiting_for_background';
        run_id: string;
        tool_call_id: string;
    } | null;
    latestRunId: string | null;
    runId: string | null;
    generation: number | null;
    lastSequence: number | null;
    lastEventId: string | null;
    lastDigest: string | null;
    runState: string | null;
    messages: string[];
    changes: Array<{
        files: Array<Record<string, unknown>>;
        headDigest: string;
    }>;
    approvals: Array<{
        approvalId: string;
        toolCallId: string;
        requestDigest: string;
        expiresAtMs: number;
        decision: string | null;
    }>;
    unknownEvents: number;
}

interface ReloadTaskSeed {
    taskId: string;
    runId: string;
    generation: number;
    title: string;
    message: string;
    files: Array<Record<string, unknown>>;
    headDigest: string;
    snapshot: {
        generation: number;
        lastSequence: number;
        lastEventId: string;
        headDigest: string;
    };
}

function seededDigest(text: string): string {
    let hex = '';
    for (let index = 0; index < 64; index += 1) {
        const code = text.charCodeAt(index % Math.max(1, text.length));
        hex += ((code + index) % 16).toString(16);
    }
    return `sha256:${hex}`;
}

interface HostileResult {
    mode: string;
    outcomes: Array<{
        kind: string;
        code: string | null;
        requiresResync: boolean;
    }>;
    snapshot: LiveRunSnapshot;
}

test.describe('Task 13 parallel tasks, reload and hostile ingestion', () => {
    test('two_live_tasks_keep_independent_artifacts_and_reload_from_indexeddb', async ({ page }) => {
        await openStudio(page);
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const started = await call<Outcome>(page, 'startTwoLiveTasksForTest');
        expect(started.ok, JSON.stringify(started)).toBe(true);
        const [taskA, taskB] = started.record as LiveTaskSeed[];

        const liveA = (await call<Outcome>(page, 'taskSnapshotForTest', taskA.taskId))
            .record as LiveRunSnapshot;
        const liveB = (await call<Outcome>(page, 'taskSnapshotForTest', taskB.taskId))
            .record as LiveRunSnapshot;
        expect(liveA.runState).toBe('running');
        expect(liveB.runState).toBe('running');
        expect(liveA.runId).not.toBe(liveB.runId);

        expect((await call<Outcome>(page, 'liveTextForTest', taskA.runId, 'alpha live output')).ok)
            .toBe(true);
        const bAfterA = (await call<Outcome>(page, 'taskSnapshotForTest', taskB.taskId))
            .record as LiveRunSnapshot;
        expect(bAfterA.lastSequence).toBe(liveB.lastSequence);
        expect(bAfterA.messages).toEqual([]);

        expect((await call<Outcome>(page, 'liveTextForTest', taskB.runId, 'beta live output')).ok)
            .toBe(true);
        expect(
            (await call<Outcome>(page, 'liveToolForTest', taskA.runId, 'artifacts/live-a.txt')).ok,
        ).toBe(true);
        expect(
            (await call<Outcome>(page, 'liveToolForTest', taskB.runId, 'artifacts/live-b.txt')).ok,
        ).toBe(true);
        expect((await call<Outcome>(page, 'finishLiveRunForTest', taskA.runId)).ok).toBe(true);
        expect((await call<Outcome>(page, 'finishLiveRunForTest', taskB.runId)).ok).toBe(true);

        const finalA = (await call<Outcome>(page, 'taskSnapshotForTest', taskA.taskId))
            .record as LiveRunSnapshot;
        const finalB = (await call<Outcome>(page, 'taskSnapshotForTest', taskB.taskId))
            .record as LiveRunSnapshot;
        expect(finalA.runState).toBe('finished');
        expect(finalB.runState).toBe('finished');
        expect(finalA.messages).toEqual(['alpha live output']);
        expect(finalB.messages).toEqual(['beta live output']);
        expect(finalA.changes[0]?.files).toEqual([
            { path: 'artifacts/live-a.txt', change: 'added' },
        ]);
        expect(finalB.changes[0]?.files).toEqual([
            { path: 'artifacts/live-b.txt', change: 'added' },
        ]);
        expect(finalA.changes[0]?.headDigest).not.toBe(finalB.changes[0]?.headDigest);

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');

        const reloadA = (await call<Outcome>(page, 'taskSnapshotForTest', taskA.taskId))
            .record as LiveRunSnapshot;
        const reloadB = (await call<Outcome>(page, 'taskSnapshotForTest', taskB.taskId))
            .record as LiveRunSnapshot;
        expect(reloadA).toMatchObject({
            taskId: taskA.taskId,
            runId: taskA.runId,
            generation: finalA.generation,
            lastSequence: finalA.lastSequence,
            lastEventId: finalA.lastEventId,
            lastDigest: finalA.lastDigest,
            runState: 'finished',
            messages: ['alpha live output'],
        });
        expect(reloadB).toMatchObject({
            taskId: taskB.taskId,
            runId: taskB.runId,
            generation: finalB.generation,
            lastSequence: finalB.lastSequence,
            lastEventId: finalB.lastEventId,
            lastDigest: finalB.lastDigest,
            runState: 'finished',
            messages: ['beta live output'],
        });
        expect(reloadA.changes[0]?.files).toEqual([
            { path: 'artifacts/live-a.txt', change: 'added' },
        ]);
        expect(reloadB.changes[0]?.files).toEqual([
            { path: 'artifacts/live-b.txt', change: 'added' },
        ]);
        expect(reloadB.lastDigest).not.toBe(reloadA.lastDigest);

        for (const task of [taskA, taskB]) {
            const verified = await call<Outcome>(page, 'verifyRunForTest', task.taskId, task.runId);
            expect(verified.record).toMatchObject({
                runOk: true,
                runCode: null,
            });
            expect((verified.record as { eventCount: number }).eventCount).toBeGreaterThan(2);
        }
        const state = await call<RestoreView>(page, 'restoreState');
        expect(state.phase).toBe('ready');
        expect(state.canStartRun).toBe(true);
    });

    test('hostile_ndjson_bytes_never_become_output_and_active_runs_keep_send_disabled', async ({ page }) => {
        await openStudio(page);
        await call<RestoreView>(page, 'open');
        const started = await call<Outcome>(page, 'startTwoLiveTasksForTest');
        expect(started.ok, JSON.stringify(started)).toBe(true);
        const [taskA, taskB] = started.record as LiveTaskSeed[];

        const modes = [
            'malformed',
            'partial',
            'utf8_split',
            'unknown',
            'cross_task',
            'duplicate',
            'gap',
        ] as const;
        const outcomes = new Map<string, HostileResult>();
        for (const mode of modes) {
            const result = await call<Outcome>(page, 'ingestHostileForTest', taskA.runId, mode);
            expect(result.ok, JSON.stringify(result)).toBe(true);
            outcomes.set(mode, result.record as HostileResult);
        }

        expect(outcomes.get('malformed')!.outcomes[0]).toMatchObject({
            kind: 'rejected',
            code: 'malformed_event',
        });
        expect(outcomes.get('partial')!.outcomes.at(-1)).toMatchObject({
            kind: 'rejected',
            code: 'truncated_frame',
        });
        expect(outcomes.get('utf8_split')!.outcomes.at(-1)).toMatchObject({
            kind: 'applied',
        });
        expect(outcomes.get('unknown')!.outcomes[0]).toMatchObject({
            kind: 'unknown',
            code: null,
        });
        expect(outcomes.get('gap')!.outcomes[0]).toMatchObject({
            kind: 'rejected',
            code: 'out_of_order',
            requiresResync: true,
        });
        expect(outcomes.get('cross_task')!.outcomes[0]).toMatchObject({
            kind: 'rejected',
            code: 'cross_task_event',
        });
        expect(outcomes.get('duplicate')!.outcomes[0]).toMatchObject({
            kind: 'duplicate',
        });
        expect((await call<Outcome>(page, 'resyncLiveRunForTest', taskA.runId)).ok)
            .toBe(true);

        expect((await call<Outcome>(page, 'liveTextForTest', taskA.runId, 'honest after hostile')).ok)
            .toBe(true);
        const finished = await call<Outcome>(page, 'finishLiveRunForTest', taskA.runId);
        expect(finished.ok, JSON.stringify(finished)).toBe(true);
        const cleanA = finished.record as LiveRunSnapshot;
        expect(cleanA.messages).toEqual(['honest after hostile']);

        const forged = await call<Outcome>(
            page,
            'ingestHostileForTest',
            taskB.runId,
            'forged_duplicate',
        );
        expect(forged.ok, JSON.stringify(forged)).toBe(true);
        expect((forged.record as HostileResult).outcomes[0]).toMatchObject({
            kind: 'rejected',
            code: 'duplicate_event',
            requiresResync: true,
        });
        const poisonedB = (await call<Outcome>(page, 'taskSnapshotForTest', taskB.taskId))
            .record as LiveRunSnapshot;
        expect(poisonedB.messages).toEqual([]);
        expect(poisonedB.runState).toBe('running');
        expect(poisonedB.unknownEvents).toBe(0);

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const reloadA = (await call<Outcome>(page, 'taskSnapshotForTest', taskA.taskId))
            .record as LiveRunSnapshot;
        const reloadB = (await call<Outcome>(page, 'taskSnapshotForTest', taskB.taskId))
            .record as LiveRunSnapshot;
        expect(reloadA.messages).toEqual(['honest after hostile']);
        expect(reloadA.unknownEvents).toBe(1);
        expect(reloadA.changes).toEqual([]);
        expect(reloadA.runState).toBe('finished');
        expect(reloadB.messages).toEqual([]);
        expect(reloadB.runState).toBe('running');
        expect(reloadB.unknownEvents).toBe(0);
        expect(reloadB.changes).toEqual([]);

        await call(page, 'targetFixture', taskB.taskId, taskB.runId, taskB.generation);
        const blocked = await call<RestoreView>(page, 'restoreState');
        expect(blocked.activeRun?.runId).toBe(taskB.runId);
        expect(blocked.canStartRun).toBe(false);
        expect(blocked.sendEnabled).toBe(false);
    });

    test('two_finished_tasks_restore_with_verified_chain_snapshot_cursor_and_artifacts', async ({ page }) => {
        await openStudio(page);
        await call<RestoreView>(page, 'open');
        const seeded = await call<Outcome>(page, 'seedTwoFinishedTasksForTest');
        expect(seeded.ok, JSON.stringify(seeded)).toBe(true);
        const [taskA, taskB] = seeded.record as ReloadTaskSeed[];
        const beforeA = (await call<Outcome>(page, 'taskSnapshotForTest', taskA.taskId))
            .record as LiveRunSnapshot;
        const beforeB = (await call<Outcome>(page, 'taskSnapshotForTest', taskB.taskId))
            .record as LiveRunSnapshot;

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');

        for (const [seed, before] of [
            [taskA, beforeA],
            [taskB, beforeB],
        ] as const) {
            const restored = (await call<Outcome>(page, 'taskSnapshotForTest', seed.taskId))
                .record as LiveRunSnapshot;
            expect(restored).toMatchObject({
                runId: seed.runId,
                generation: seed.generation,
                lastSequence: before.lastSequence,
                lastEventId: before.lastEventId,
                lastDigest: before.lastDigest,
                runState: 'finished',
                messages: [seed.message],
            });
            expect(restored.changes[0]?.files).toEqual(seed.files);
            expect(restored.changes[0]?.headDigest).toBe(seed.headDigest);

            const verified = await call<Outcome>(
                page,
                'verifyRunForTest',
                seed.taskId,
                seed.runId,
            );
            expect(verified.ok, JSON.stringify(verified)).toBe(true);
            expect(verified.record).toMatchObject({
                runOk: true,
                snapshotOk: true,
                snapshot: {
                    generation: seed.snapshot.generation,
                    lastSequence: seed.snapshot.lastSequence,
                    lastEventId: seed.snapshot.lastEventId,
                    headDigest: seed.snapshot.headDigest,
                },
            });
        }
        expect(beforeA.lastDigest).not.toBe(beforeB.lastDigest);
        expect((await readDb(page)).taskRecords).toHaveLength(2);
        expect((await readDb(page)).runRecords).toHaveLength(2);
        expect((await readDb(page)).snapshotKeys).toHaveLength(2);
    });

    test('pending_approval_and_retry_restore_honestly_without_fabricated_resolution', async ({ page }) => {
        await openStudio(page);
        await call<RestoreView>(page, 'open');
        const approvalSeed = await call<Outcome>(page, 'seedPendingApprovalForTest');
        expect(approvalSeed.ok, JSON.stringify(approvalSeed)).toBe(true);
        const approvalFixture = approvalSeed.record as {
            taskId: string;
            runId: string;
        };
        const retrySeed = await call<Outcome>(page, 'seedFailedRunForRetryProbe');
        expect(retrySeed.ok, JSON.stringify(retrySeed)).toBe(true);
        const retryFixture = retrySeed.record as { taskId: string; runId: string };

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');

        const pending = await call<Outcome>(
            page,
            'pendingApprovalsForTest',
            approvalFixture.taskId,
        );
        expect(pending.ok, JSON.stringify(pending)).toBe(true);
        const pendingRecord = pending.record as {
            pending: Array<{
                approvalId: string;
                taskId: string;
                runId: string;
                generation: number;
                toolCallId: string;
                requestDigest: string;
                expiresAtMs: number;
                expired: boolean;
            }>;
            view: LiveRunSnapshot['approvals'];
        };
        expect(pendingRecord.pending).toHaveLength(1);
        expect(pendingRecord.pending[0]).toMatchObject({
            approvalId: 'appr_reload_approval',
            taskId: approvalFixture.taskId,
            runId: approvalFixture.runId,
            generation: 1,
            toolCallId: 'tool_reload_approval',
            expired: false,
        });
        expect(pendingRecord.pending[0].requestDigest).toBe(
            seededDigest('reload-approval'),
        );
        expect(pendingRecord.pending[0].expiresAtMs).toBe(
            1_794_283_200_000 + 600_000,
        );
        expect(pendingRecord.view[0]).toMatchObject({
            approvalId: 'appr_reload_approval',
            decision: null,
        });

        const beforeResolve = await call<Outcome>(
            page,
            'verifyRunForTest',
            approvalFixture.taskId,
            approvalFixture.runId,
        );
        const refused = await call<Outcome>(
            page,
            'resolveRestoredApprovalForTest',
            'appr_reload_approval',
        );
        expect(refused.ok, JSON.stringify(refused)).toBe(false);
        expect(refused.code).toBe('approval_forbidden');
        const afterResolve = await call<Outcome>(
            page,
            'verifyRunForTest',
            approvalFixture.taskId,
            approvalFixture.runId,
        );
        expect((afterResolve.record as { eventCount: number }).eventCount).toBe(
            (beforeResolve.record as { eventCount: number }).eventCount,
        );
        const stillPending = (await call<Outcome>(
            page,
            'taskSnapshotForTest',
            approvalFixture.taskId,
        )).record as LiveRunSnapshot;
        expect(stillPending.approvals[0]?.decision).toBeNull();
        expect(stillPending.runState).not.toBe('finished');

        const retry = await call<Outcome>(
            page,
            'retryRestoredRunForTest',
            retryFixture.taskId,
            retryFixture.runId,
        );
        expect(retry.ok, JSON.stringify(retry)).toBe(false);
        expect(retry.code).toBe('invalid_arguments');
        expect(retry.message).toContain('prompt is unavailable');
        const retryTask = (await call<Outcome>(page, 'taskSnapshotForTest', retryFixture.taskId))
            .record as LiveRunSnapshot;
        expect(retryTask.latestRunId).toBe(retryFixture.runId);

        await call(
            page,
            'targetFixture',
            approvalFixture.taskId,
            approvalFixture.runId,
            1,
        );
        const blocked = await call<RestoreView>(page, 'restoreState');
        expect(blocked.activeRun?.runId).toBe(approvalFixture.runId);
        expect(blocked.canStartRun).toBe(false);
        expect(blocked.sendEnabled).toBe(false);
    });

    test('adapter_hold_null_clears_memory_and_durable_records', async ({ page }) => {
        await openStudio(page);
        await call<RestoreView>(page, 'open');
        const seeded = await call<Outcome>(page, 'seedBackgroundHoldForTest');
        expect(seeded.ok, JSON.stringify(seeded)).toBe(true);
        const hold = seeded.record as {
            taskId: string;
            runId: string;
            toolCallId: string;
        };

        const cleared = await call<Outcome>(page, 'clearHoldNullForTest', hold.taskId);
        expect(cleared.ok, JSON.stringify(cleared)).toBe(true);
        expect(cleared.record).toMatchObject({
            memoryHold: null,
            durableHold: null,
            writtenHold: null,
        });
        const before = (await call<Outcome>(page, 'taskSnapshotForTest', hold.taskId))
            .record as LiveRunSnapshot;
        expect(before.hold).toBeNull();
        expect(before.taskState).toBe('failed');

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const restored = (await call<Outcome>(page, 'taskSnapshotForTest', hold.taskId))
            .record as LiveRunSnapshot;
        expect(restored.hold).toBeNull();
        expect(restored.taskState).toBe('failed');
        const durable = (await readDb(page)).taskRecords.find(
            (record) => record.task_id === hold.taskId,
        );
        expect(durable?.hold).toBeUndefined();
    });

    test('reset_and_reopen_dispose_the_cached_runtime_before_the_shared_store', async ({ page }) => {
        await openStudio(page);
        await call<RestoreView>(page, 'open');
        expect((await call<Outcome>(page, 'startTwoLiveTasksForTest')).ok).toBe(true);

        const reopened = await call<RestoreView>(page, 'reopen');
        expect(reopened.phase).toBe('ready');
        const afterReopen = await call<Outcome>(page, 'startTwoLiveTasksForTest');
        expect(afterReopen.ok, JSON.stringify(afterReopen)).toBe(true);
        const [reopenedTask] = afterReopen.record as LiveTaskSeed[];
        expect((await call<Outcome>(page, 'finishLiveRunForTest', reopenedTask.runId)).ok)
            .toBe(true);

        await call(page, 'reset');
        const resetOpen = await call<RestoreView>(page, 'open');
        expect(resetOpen.phase).toBe('ready');
        const afterReset = await call<Outcome>(page, 'startTwoLiveTasksForTest');
        expect(afterReset.ok, JSON.stringify(afterReset)).toBe(true);
        const [resetTask] = afterReset.record as LiveTaskSeed[];
        expect((await call<Outcome>(page, 'finishLiveRunForTest', resetTask.runId)).ok)
            .toBe(true);
    });
});
