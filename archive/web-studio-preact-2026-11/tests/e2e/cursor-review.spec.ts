import { expect, test, type Page } from '@playwright/test';

const TEST_URL = '/?typed=1&studioTest=1';
const DB_NAME = 'uor-r4.studio.agent';

interface StudioTestCall {
    method: string;
    args: unknown[];
}

interface Outcome {
    ok: boolean;
    code?: string;
    message?: string;
    record?: unknown;
}

interface FixtureSeed {
    taskId: string;
    runId: string;
}

interface StoredComment {
    key: IDBValidKey;
    record: Record<string, unknown>;
}

async function openStudio(page: Page): Promise<void> {
    await page.goto(TEST_URL);
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

async function selectTask(page: Page, taskId: string): Promise<void> {
    const row = page.locator(`#studio-typed-root [data-task-id="${taskId}"] .task-select`);
    await expect(row).toBeVisible();
    await row.click();
    await expect(page.locator('#studio-typed-root [data-task-detail-header]')).toBeVisible();
}

async function openChanges(page: Page): Promise<void> {
    const navigation = page.locator('#studio-typed-root #studio-navigation');
    await navigation.getByRole('button', { name: 'Changes', exact: true }).click();
    await expect(page.locator('#studio-typed-root [data-diff-review]')).toBeVisible();
}

async function readStoredComments(page: Page): Promise<StoredComment[]> {
    return page.evaluate(async (dbName) => {
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
            const tx = db.transaction('comments', 'readonly');
            const store = tx.objectStore('comments');
            const keys = await request(store.getAllKeys());
            const records = await request(store.getAll());
            await done(tx);
            return keys.map((key, index) => ({ key, record: records[index] as Record<string, unknown> }));
        } finally {
            db.close();
        }
    }, DB_NAME);
}

async function clearStoredComments(page: Page): Promise<void> {
    await page.evaluate(async (dbName) => {
        const db = await new Promise<IDBDatabase>((resolve, reject) => {
            const open = indexedDB.open(dbName);
            open.onsuccess = () => resolve(open.result);
            open.onerror = () => reject(open.error);
        });
        try {
            const tx = db.transaction('comments', 'readwrite');
            tx.objectStore('comments').clear();
            await new Promise<void>((resolve, reject) => {
                tx.oncomplete = () => resolve();
                tx.onabort = () => reject(tx.error ?? new Error('transaction aborted'));
                tx.onerror = () => reject(tx.error ?? new Error('transaction failed'));
            });
        } finally {
            db.close();
        }
    }, DB_NAME);
}

async function addCommentDirect(
    page: Page,
    input: {
        taskId: string;
        path: string;
        line: number;
        body: string;
        headDigest: string;
        scope: 'last_turn' | 'branch';
        createdAtMs?: number;
    },
): Promise<{ comment_id: string; body: string; head_digest: string; created_at_ms?: number }> {
    return page.evaluate(async (payload) => {
        const api = (window as unknown as {
            __studioTest?: {
                clientForTest?: () => {
                    addDiffComment(input: typeof payload): Promise<{
                        comment_id: string;
                        body: string;
                        head_digest: string;
                    }>;
                } | null;
            };
        }).__studioTest;
        const client = api?.clientForTest?.();
        if (!client) throw new Error('studio test client is missing');
        return client.addDiffComment(payload);
    }, input);
}

async function addCommentOutcome(
    page: Page,
    input: {
        taskId: string;
        path: string;
        line: number;
        body: string;
        headDigest: string;
        scope: 'last_turn' | 'branch';
        createdAtMs?: number;
    },
): Promise<{ ok: boolean; code?: string; message?: string }> {
    return page.evaluate(async (payload) => {
        const api = (window as unknown as {
            __studioTest?: {
                clientForTest?: () => {
                    addDiffComment(input: typeof payload): Promise<unknown>;
                } | null;
            };
        }).__studioTest;
        const client = api?.clientForTest?.();
        if (!client) return { ok: false, code: 'runtime_unavailable', message: 'client missing' };
        try {
            await client.addDiffComment(payload);
            return { ok: true };
        } catch (error) {
            const failure = error as { code?: string; message?: string };
            return { ok: false, code: failure.code, message: failure.message };
        }
    }, input);
}

async function readCommentsDirect(
    page: Page,
    taskId: string,
): Promise<Array<{ comment_id: string; path: string; line: number; body: string; head_digest: string; created_at_ms: number }>> {
    return page.evaluate(async (id) => {
        const api = (window as unknown as {
            __studioTest?: {
                clientForTest?: () => {
                    readDiffComments(taskId: string): Promise<Array<{
                        comment_id: string;
                        path: string;
                        line: number;
                        body: string;
                        head_digest: string;
                        created_at_ms: number;
                    }>>;
                } | null;
            };
        }).__studioTest;
        const client = api?.clientForTest?.();
        if (!client) throw new Error('studio test client is missing');
        return client.readDiffComments(id);
    }, taskId);
}

async function appendRemoteComment(
    page: Page,
    input: {
        taskId: string;
        runId: string;
        body: unknown;
        path?: string;
        line?: number;
        headDigest?: string;
    },
): Promise<Outcome & { status?: string }> {
    return page.evaluate(async (payload) => {
        const api = (window as unknown as {
            __studioTest?: {
                clientForTest?: () => {
                    store: {
                        getRunCursor(taskId: string, runId: string): {
                            generation: number;
                            lastSequence: number;
                        } | null;
                    };
                    ingestWireEvent(text: string): Promise<Outcome & { status?: string }>;
                } | null;
            };
        }).__studioTest;
        const client = api?.clientForTest?.();
        if (!client) return { ok: false, code: 'runtime_unavailable', message: 'client missing' };
        const cursor = client.store.getRunCursor(payload.taskId, payload.runId);
        if (cursor === null) return { ok: false, code: 'run_not_found', message: 'cursor missing' };
        const generation = cursor.generation;
        const sequence = cursor.lastSequence + 1;
        const data = {
            body: payload.body,
            comment_id: 'cmt_aaaaaaaaaaaaaaaa',
            head_digest: payload.headDigest ?? 'remote:digest',
            line: payload.line ?? 0,
            path: payload.path ?? 'src/a.ts',
        };
        const event = {
            data,
            event_id: `evt_${payload.runId}_${generation}_${String(sequence).padStart(12, '0')}`,
            generation,
            project_id: 'prj_0000000000000001',
            recorded_at_ms: 1_800_000_000_000,
            run_id: payload.runId,
            schema: 'uor-r4.studio.agent-event/1',
            sequence,
            task_id: payload.taskId,
            type: 'diff.comment',
        };
        return client.ingestWireEvent(JSON.stringify(event));
    }, input);
}

function exportedCommentEvents(text: string): Array<{
    task_id: string;
    run_id: string;
    generation: number;
    sequence: number;
    record: { event_id: string; canonical: string; digest: string; prev_digest: string | null; recorded_at_ms: number };
}> {
    return text
        .split('\n')
        .filter((line) => line.length > 0)
        .map((line) => JSON.parse(line) as Record<string, unknown>)
        .filter((line) => line.kind === 'event')
        .map((line) => line as unknown as {
            task_id: string;
            run_id: string;
            generation: number;
            sequence: number;
            record: { event_id: string; canonical: string; digest: string; prev_digest: string | null; recorded_at_ms: number };
        })
        .filter((line) => (JSON.parse(line.record.canonical) as { type?: string }).type === 'diff.comment');
}

function exportedEventCount(text: string): number {
    return text
        .split('\n')
        .filter((line) => line.length > 0)
        .map((line) => JSON.parse(line) as { kind?: string })
        .filter((line) => line.kind === 'event').length;
}

async function runCursor(
    page: Page,
    taskId: string,
    runId: string,
): Promise<{ generation: number; lastSequence: number; lastEventId: string | null; lastDigest: string | null } | null> {
    return page.evaluate(({ taskId: id, runId: rid }) => {
        const api = (window as unknown as {
            __studioTest?: {
                clientForTest?: () => {
                    store: {
                        getRunCursor(taskId: string, runId: string): {
                            generation: number;
                            lastSequence: number;
                            lastEventId: string | null;
                            lastDigest: string | null;
                        } | null;
                    };
                } | null;
            };
        }).__studioTest;
        const client = api?.clientForTest?.();
        if (!client) return null;
        return client.store.getRunCursor(id, rid);
    }, { taskId, runId });
}

test.describe('Task 21 changes review and local comments', () => {
    test('diff_comment_survives_reload', async ({ page }) => {
        await openStudio(page);
        await call<{ phase: string }>(page, 'open');
        const seeded = await call<FixtureSeed>(page, 'createTaskAndEvents', {
            mode: 'message',
            title: 'Review persistence task',
        });
        const appended = await call<Outcome>(page, 'appendChanges');
        expect(appended.ok, JSON.stringify(appended)).toBe(true);
        const finished = await call<Outcome>(page, 'finishFixtureForTest');
        expect(finished.ok, JSON.stringify(finished)).toBe(true);

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await expect.poll(async () => (await call<{ phase: string }>(page, 'open')).phase).toBe('ready');
        await selectTask(page, seeded.taskId);
        await openChanges(page);
        const review = page.locator('#studio-typed-root [data-diff-review]');
        await expect(review.getByRole('button', { name: 'Last agent turn', exact: true })).toHaveAttribute('aria-pressed', 'true');
        await expect(review).toContainText('Browser last turn');
        await expect(review).toContainText('src/a.ts');

        const text = 'keep this note across reload';
        await review.getByLabel('Comment', { exact: true }).fill(text);
        await review.getByRole('button', { name: 'Add local comment', exact: true }).click();
        await expect(review.locator('[data-diff-current-comments]')).toContainText(text);
        await expect(review.locator('[data-diff-stale-comments]')).toHaveCount(0);

        const stored = await readStoredComments(page);
        expect(stored).toHaveLength(1);
        expect(stored[0]!.key).toEqual([seeded.taskId, 'src/a.ts', 0, 'turn:3:fixture']);
        expect(stored[0]!.record).toMatchObject({
            task_id: seeded.taskId,
            path: 'src/a.ts',
            line: 0,
            body: text,
            head_digest: 'turn:3:fixture',
            scope: 'last_turn',
        });

        const exported = await call<string>(page, 'exportNdjson');
        const commentEvents = exportedCommentEvents(exported);
        expect(commentEvents).toHaveLength(1);
        expect(commentEvents[0]!.task_id).toBe(seeded.taskId);
        expect(commentEvents[0]!.run_id).toBe(seeded.runId);
        expect(commentEvents[0]!.record.prev_digest).not.toBeNull();
        const canonical = JSON.parse(commentEvents[0]!.record.canonical) as {
            type: string;
            data: Record<string, unknown>;
        };
        expect(canonical.type).toBe('diff.comment');
        expect(canonical.data).toEqual({
            body: text,
            comment_id: expect.stringMatching(/^cmt_[0-9a-f]+$/) as unknown as string,
            head_digest: 'turn:3:fixture',
            line: 0,
            path: 'src/a.ts',
        });
        expect(JSON.parse(commentEvents[0]!.record.canonical)).not.toHaveProperty('data.scope');

        const rejectedImport = await call<Outcome>(page, 'importNdjson', '{"kind":"header"}\n');
        expect(rejectedImport.ok).toBe(false);
        expect((await call<{ phase: string }>(page, 'open')).phase).toBe('ready');
        expect(await readCommentsDirect(page, seeded.taskId)).toHaveLength(1);

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await expect.poll(async () => (await call<{ phase: string }>(page, 'open')).phase).toBe('ready');
        await selectTask(page, seeded.taskId);
        await openChanges(page);

        const reloaded = page.locator('#studio-typed-root [data-diff-review]');
        await expect(reloaded).toContainText('src/a.ts');
        await expect(reloaded.locator('[data-diff-current-comments]')).toContainText(text);
        await expect(reloaded.locator('[data-diff-stale-comments]')).toHaveCount(0);
        await expect(reloaded).not.toContainText(/\bmerge\b/i);
        await expect(reloaded).not.toContainText(/restore checkpoint|checkpoint/i);

        await clearStoredComments(page);
        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await expect.poll(async () => (await call<{ phase: string }>(page, 'open')).phase).toBe('ready');
        await selectTask(page, seeded.taskId);
        await openChanges(page);
        const rebuilt = page.locator('#studio-typed-root [data-diff-review]');
        await expect(rebuilt.locator('[data-diff-current-comments]')).toContainText(text);
        expect(await readCommentsDirect(page, seeded.taskId)).toHaveLength(1);
    });

    test('finished_run_comment_is_canonical_and_importable', async ({ page }) => {
        await openStudio(page);
        await call<{ phase: string }>(page, 'open');
        const seeded = await call<FixtureSeed>(page, 'createTaskAndEvents', {
            mode: 'message',
            title: 'Canonical comment task',
        });
        expect((await call<Outcome>(page, 'appendChanges')).ok).toBe(true);
        expect((await call<Outcome>(page, 'finishFixtureForTest')).ok).toBe(true);
        const second = await call<FixtureSeed>(page, 'createSecondRun');
        expect(second.taskId).toBe(seeded.taskId);
        expect(second.runId).not.toBe(seeded.runId);
        expect((await call<Outcome>(page, 'finishFixtureForTest')).ok).toBe(true);

        const added = await addCommentDirect(page, {
            taskId: seeded.taskId,
            path: 'src/a.ts',
            line: 0,
            body: 'canonical note',
            headDigest: 'turn:3:fixture',
            scope: 'last_turn',
        });
        expect(added.body).toBe('canonical note');

        const running = await call<FixtureSeed>(page, 'createTaskAndEvents', {
            mode: 'message',
            title: 'Running comment task',
        });
        const runningRejected = await addCommentOutcome(page, {
            taskId: running.taskId,
            path: 'src/a.ts',
            line: 0,
            body: 'too early',
            headDigest: 'turn:3:fixture',
            scope: 'last_turn',
        });
        expect(runningRejected).toMatchObject({ ok: false, code: 'out_of_order' });

        const invalidBody = await addCommentOutcome(page, {
            taskId: seeded.taskId,
            path: 'src/a.ts',
            line: 0,
            body: 'x'.repeat(9000),
            headDigest: 'turn:3:fixture',
            scope: 'last_turn',
        });
        expect(invalidBody).toMatchObject({ ok: false, code: 'invalid_arguments' });

        const exported = await call<string>(page, 'exportNdjson');
        const events = exportedCommentEvents(exported);
        expect(events).toHaveLength(1);
        expect(events[0]!.run_id).toBe(seeded.runId);
        expect(JSON.parse(events[0]!.record.canonical).data).toMatchObject({
            path: 'src/a.ts',
            line: 0,
            body: 'canonical note',
            head_digest: 'turn:3:fixture',
        });

        await call(page, 'reset');
        expect((await call<{ phase: string }>(page, 'open')).phase).toBe('ready');
        const imported = await call<Outcome>(page, 'importNdjson', exported);
        expect(imported.ok, JSON.stringify(imported)).toBe(true);
        const importedComments = await readCommentsDirect(page, seeded.taskId);
        expect(importedComments).toHaveLength(1);
        expect(importedComments[0]).toMatchObject({
            path: 'src/a.ts',
            line: 0,
            body: 'canonical note',
            head_digest: 'turn:3:fixture',
        });

        const reexported = await call<string>(page, 'exportNdjson');
        expect(exportedCommentEvents(reexported)).toHaveLength(1);
    });

    test('generic_ingress_rejects_diff_comment_without_mutation', async ({ page }) => {
        await openStudio(page);
        await call<{ phase: string }>(page, 'open');
        const seeded = await call<FixtureSeed>(page, 'createTaskAndEvents', {
            mode: 'message',
            title: 'Remote comment ingress',
        });
        const beforeExport = await call<string>(page, 'exportNdjson');
        const beforeCount = exportedEventCount(beforeExport);
        const beforeCursor = await runCursor(page, seeded.taskId, seeded.runId);
        expect(beforeCursor).not.toBeNull();

        const remote = await appendRemoteComment(page, {
            taskId: seeded.taskId,
            runId: seeded.runId,
            body: 'remote note',
        });
        expect(remote).toMatchObject({ status: 'rejected', code: 'invalid_event' });

        const malformed = await appendRemoteComment(page, {
            taskId: seeded.taskId,
            runId: seeded.runId,
            body: 42,
        });
        expect(malformed).toMatchObject({ status: 'rejected', code: 'invalid_event' });

        const afterExport = await call<string>(page, 'exportNdjson');
        expect(exportedEventCount(afterExport)).toBe(beforeCount);
        expect(exportedCommentEvents(afterExport)).toHaveLength(0);
        expect(await runCursor(page, seeded.taskId, seeded.runId)).toEqual(beforeCursor);
        expect(await readCommentsDirect(page, seeded.taskId)).toHaveLength(0);

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await expect.poll(async () => (await call<{ phase: string }>(page, 'open')).phase).toBe('ready');
        const reloadedExport = await call<string>(page, 'exportNdjson');
        expect(exportedEventCount(reloadedExport)).toBe(beforeCount);
        expect(exportedCommentEvents(reloadedExport)).toHaveLength(0);
        expect(await readCommentsDirect(page, seeded.taskId)).toHaveLength(0);
    });

    test('future_created_at_cannot_bypass_lease', async ({ page }) => {
        await openStudio(page);
        await call<{ phase: string }>(page, 'open');
        const seeded = await call<FixtureSeed>(page, 'createTaskAndEvents', {
            mode: 'message',
            title: 'Future timestamp task',
        });
        expect((await call<Outcome>(page, 'appendChanges')).ok).toBe(true);
        expect((await call<Outcome>(page, 'finishFixtureForTest')).ok).toBe(true);

        const lease = await call<Outcome>(page, 'acquireLease', 'other-writer');
        expect(lease.ok, JSON.stringify(lease)).toBe(true);
        const future = 9_000_000_000_000;
        const rejected = await addCommentOutcome(page, {
            taskId: seeded.taskId,
            path: 'src/a.ts',
            line: 0,
            body: 'future note',
            headDigest: 'turn:3:fixture',
            scope: 'last_turn',
            createdAtMs: future,
        });
        expect(rejected).toMatchObject({ ok: false, code: 'stale_generation' });
        expect(await readCommentsDirect(page, seeded.taskId)).toHaveLength(0);

        const released = await call<Outcome>(page, 'releaseLease', 'other-writer');
        expect(released.ok, JSON.stringify(released)).toBe(true);
        const accepted = await addCommentDirect(page, {
            taskId: seeded.taskId,
            path: 'src/a.ts',
            line: 0,
            body: 'future note',
            headDigest: 'turn:3:fixture',
            scope: 'last_turn',
            createdAtMs: future,
        });
        expect(accepted.body).toBe('future note');
        const comments = await readCommentsDirect(page, seeded.taskId);
        expect(comments).toHaveLength(1);
        expect(comments[0]!.created_at_ms).toBe(future);
        const exported = await call<string>(page, 'exportNdjson');
        const events = exportedCommentEvents(exported);
        expect(events).toHaveLength(1);
        expect(events[0]!.record.recorded_at_ms).toBe(future);
    });

    test('static_browser_reviews_uploaded_file_locally', async ({ page }) => {
        await openStudio(page);
        await openChanges(page);
        const review = page.locator('#studio-typed-root [data-diff-review]');
        const picker = page.locator('#studio-typed-root .diff-upload input[type="file"]');
        await picker.setInputFiles({
            name: 'notes.diff',
            mimeType: 'text/plain',
            buffer: Buffer.from('diff --git a/notes.md b/notes.md\n+local review line\n'),
        });

        await expect(review.getByRole('button', { name: 'Branch', exact: true })).toHaveAttribute('aria-pressed', 'true');
        await expect(review).toContainText('Uploaded file');
        await expect(review).toContainText('notes.diff');
        await expect(review).toContainText('+local review line');
        await expect(review).toContainText('stored in this browser');
        await expect(review).toContainText('Select a browser task to add a local comment.');
    });
});

interface RecordedRequest {
    method: string;
    url: string;
}

function recordRequests(page: Page): RecordedRequest[] {
    const requests: RecordedRequest[] = [];
    page.on('request', (request) => {
        requests.push({ method: request.method(), url: request.url() });
    });
    return requests;
}

async function settleInitialRequests(page: Page, requests: RecordedRequest[]): Promise<void> {
    await page.waitForLoadState('networkidle', { timeout: 10_000 });
    let last = requests.length;
    let quietSince = Date.now();
    const deadline = Date.now() + 5_000;
    while (Date.now() < deadline) {
        await page.waitForTimeout(50);
        if (requests.length !== last) {
            last = requests.length;
            quietSince = Date.now();
        } else if (Date.now() - quietSince >= 300) {
            return;
        }
    }
}

async function assertNoNewRequests(
    page: Page,
    requests: RecordedRequest[],
    before: number,
    action: string,
): Promise<void> {
    let last = requests.length;
    let quietSince = Date.now();
    const deadline = Date.now() + 3_000;
    while (Date.now() < deadline) {
        await page.waitForTimeout(50);
        if (requests.length !== last) {
            last = requests.length;
            quietSince = Date.now();
        } else if (Date.now() - quietSince >= 300) {
            break;
        }
    }
    const added = requests.slice(before);
    expect(
        added,
        `${action} issued a network request:\n${added
            .map((request) => `${request.method} ${request.url}`)
            .join('\n')}`,
    ).toHaveLength(0);
}

test.describe('Task 22 read-only PR and digest-bound artifacts', () => {
    test('pr_tab_is_read_only_and_explicitly_unavailable_without_a_hosted_integration', async ({ page }) => {
        const requests = recordRequests(page);
        await openStudio(page);
        await settleInitialRequests(page, requests);
        expect(requests.length).toBeGreaterThan(0);

        const before = requests.length;
        await page.locator('#studio-typed-root [data-pane-target="pr"]').click();
        const panel = page.locator('#studio-typed-root [data-pr-panel]');
        await expect(panel).toBeVisible();
        await expect(panel.locator('[data-pr-unavailable]')).toContainText(
            'No local or imported pull request summary is available',
        );
        await expect(panel.locator('a[data-pr-link]')).toHaveCount(0);
        await expect(panel.locator('iframe')).toHaveCount(0);
        await expect(
            panel.getByRole('button', { name: /merge|approve|restore checkpoint/i }),
        ).toHaveCount(0);
        await expect(panel).not.toContainText(/restore checkpoint/i);
        await expect(panel).not.toContainText(/github connected/i);
        await assertNoNewRequests(page, requests, before, 'Opening the PR tab');
    });

    test('artifact_preview_reads_verified_browser_bytes_without_a_network_request', async ({ page }) => {
        const requests = recordRequests(page);
        await openStudio(page);
        expect((await call<{ phase: string }>(page, 'open')).phase).toBe('ready');
        const artifactId = `art_${'a1'.repeat(32)}`;
        const text = 'digest-bound local artifact';
        const seeded = await seedArtifactForReview(page, artifactId, text);
        await settleInitialRequests(page, requests);
        expect(requests.length).toBeGreaterThan(0);

        await page.locator('#studio-typed-root [data-pane-target="artifacts"]').click();
        const panel = page.locator('#studio-typed-root [data-artifact-panel]');
        await expect(panel).toBeVisible();
        await settleInitialRequests(page, requests);
        const identity = panel.getByLabel('Artifact ID', { exact: true });
        await expect(identity).toBeVisible();
        await identity.fill(artifactId);
        await expect(identity).toHaveValue(artifactId);

        const before = requests.length;
        await panel.getByRole('button', { name: 'Load artifact', exact: true }).click();
        const preview = panel.locator('[data-artifact-preview="text"]');
        await expect(preview).toBeVisible();
        await expect(preview).toHaveText(text);
        await expect(panel.locator('[data-artifact-metadata]')).toContainText(seeded.digest);
        await expect(panel.locator('[data-artifact-metadata]')).toContainText('text/plain');
        await expect(panel.locator('iframe')).toHaveCount(0);
        await assertNoNewRequests(page, requests, before, 'Loading a local artifact');

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await expect.poll(async () => (await call<{ phase: string }>(page, 'open')).phase).toBe('ready');
        await page.locator('#studio-typed-root [data-pane-target="artifacts"]').click();
        const reloaded = page.locator('#studio-typed-root [data-artifact-panel]');
        await expect(reloaded).toBeVisible();
        await settleInitialRequests(page, requests);
        const reloadedIdentity = reloaded.getByLabel('Artifact ID', { exact: true });
        await reloadedIdentity.fill(artifactId);
        await expect(reloadedIdentity).toHaveValue(artifactId);

        const beforeReloadLoad = requests.length;
        await reloaded.getByRole('button', { name: 'Load artifact', exact: true }).click();
        await expect(reloaded.locator('[data-artifact-preview="text"]')).toHaveText(text);
        await assertNoNewRequests(page, requests, beforeReloadLoad, 'Loading a reloaded local artifact');
    });

    test('corrupt_artifact_and_expired_link_are_refused_without_preview_or_request', async ({ page }) => {
        const requests = recordRequests(page);
        await openStudio(page);
        expect((await call<{ phase: string }>(page, 'open')).phase).toBe('ready');
        const corruptId = `art_${'c1'.repeat(32)}`;
        await seedArtifactForReview(page, corruptId, 'corrupt artifact', {
            digestOverride: `sha256:${'0'.repeat(64)}`,
        });
        await settleInitialRequests(page, requests);
        expect(requests.length).toBeGreaterThan(0);

        await page.locator('#studio-typed-root [data-pane-target="artifacts"]').click();
        const panel = page.locator('#studio-typed-root [data-artifact-panel]');
        await expect(panel).toBeVisible();
        await settleInitialRequests(page, requests);
        const identity = panel.getByLabel('Artifact ID', { exact: true });
        await identity.fill(corruptId);
        await expect(identity).toHaveValue(corruptId);

        const beforeCorrupt = requests.length;
        await panel.getByRole('button', { name: 'Load artifact', exact: true }).click();
        await expect(panel.locator('[data-artifact-error]')).toContainText('failed verification');
        await expect(panel.locator('[data-artifact-preview]')).toHaveCount(0);
        await assertNoNewRequests(page, requests, beforeCorrupt, 'Loading a corrupt artifact');

        const expired = `https://example.com/artifact?token=expired`;
        await identity.fill(expired);
        await expect(identity).toHaveValue(expired);

        const beforeExpired = requests.length;
        await panel.getByRole('button', { name: 'Load artifact', exact: true }).click();
        await expect(panel.locator('[data-artifact-error]')).toContainText('Artifact ID must be art_');
        await expect(panel.locator('[data-artifact-preview]')).toHaveCount(0);
        await expect(identity).toHaveValue('');
        await expect(panel.locator('a[href*="example.com"]')).toHaveCount(0);
        await assertNoNewRequests(page, requests, beforeExpired, 'Submitting an expired artifact link');
    });
});

async function seedArtifactForReview(
    page: Page,
    artifactId: string,
    text: string,
    options: { digestOverride?: string } = {},
): Promise<{ artifactId: string; digest: string; bytes: number[] }> {
    return page.evaluate(
        async ({ dbName, artifactId: id, text: body, digestOverride }) => {
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
                const bytes = new TextEncoder().encode(body);
                const digestBytes = new Uint8Array(
                    await crypto.subtle.digest('SHA-256', bytes),
                );
                let hex = '';
                for (const byte of digestBytes) hex += byte.toString(16).padStart(2, '0');
                const digest = digestOverride ?? `sha256:${hex}`;
                const tx = db.transaction('artifacts', 'readwrite');
                await request(
                    tx.objectStore('artifacts').put({
                        artifact_id: id,
                        media_type: 'text/plain',
                        size_bytes: bytes.byteLength,
                        digest,
                        blob: new Blob([bytes], { type: 'text/plain' }),
                    }, id),
                );
                await done(tx);
                return { artifactId: id, digest, bytes: [...bytes] };
            } finally {
                db.close();
            }
        },
        { dbName: 'uor-r4.studio.agent', artifactId, text, digestOverride: options.digestOverride },
    );
}
