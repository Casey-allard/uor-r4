import { expect, test, type Page } from '@playwright/test';

const TEST_URL = '/?typed=1&studioTest=1';

interface StudioTestCall {
    method: string;
    args: unknown[];
}

interface Outcome {
    ok: boolean;
    code?: string;
    message?: string;
    status?: string;
    storage?: string | null;
    record?: unknown;
}

interface OwnerIdentity {
    clientAdopted: boolean;
    runtimeAdopted: boolean;
}

interface RestoreView {
    phase: 'idle' | 'loading' | 'ready' | 'incomplete' | 'error';
    sendEnabled: boolean;
    canStartRun: boolean;
    error: { code: string; message: string } | null;
    activeRun: {
        runId: string;
        generation: number;
        lastSequence: number;
        state: string;
        statusProjection: string | null;
    } | null;
}

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
    retryOfRunId: string | null;
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

interface EventList {
    state: string;
    generation: number;
    lastSequence: number;
    lastEventId: string | null;
    lastDigest: string | null;
    events: Array<{
        sequence: number;
        type: string;
        eventId: string;
    }>;
}

interface CursorView {
    taskId: string;
    runId: string;
    generation: number;
    sequence: number;
    eventId: string;
    digest: string;
    runState: string;
    messageCount: number;
    lastMessageText: string | null;
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

interface TerminalResult {
    type: string;
    eventId: string;
    outcomes: Array<{
        kind: string;
        code: string | null;
        requiresResync: boolean;
    }>;
    snapshot: LiveRunSnapshot;
}

interface FixtureSummary {
    taskId: string;
    projectId: string;
    runId: string;
    generation: number;
    eventIds: string[];
    digests: string[];
}

interface VerifyResult {
    runOk: boolean;
    runCode: string | null;
    snapshotOk: boolean;
    snapshot: {
        generation: number;
        lastSequence: number;
        lastEventId: string;
        headDigest: string;
    } | null;
    eventCount: number;
}

async function openStudio(page: Page, path = TEST_URL): Promise<void> {
    await page.goto(path);
    await expect(page.locator('#studio-typed-root')).toHaveAttribute(
        'data-active',
        'true',
    );
}

async function call<T>(
    page: Page,
    method: string,
    ...args: unknown[]
): Promise<T> {
    const payload: StudioTestCall = { method, args };
    return page.evaluate(({ method, args }) => {
        const api = (
            window as unknown as {
                __studioTest?: Record<
                    string,
                    (...callArgs: unknown[]) => Promise<unknown>
                >;
            }
        ).__studioTest;
        if (!api) throw new Error('window.__studioTest is missing');
        return api[method]!(...args);
    }, payload) as Promise<T>;
}

async function stopRun(
    page: Page,
    taskId: string,
    runId: string,
): Promise<{ acknowledged: boolean }> {
    return page.evaluate(
        async ({ taskId, runId }) => {
            const api = (
                window as unknown as {
                    __studioTest?: {
                        clientForTest(): {
                            stop(input: {
                                taskId: string;
                                runId: string;
                            }): Promise<{ acknowledged: boolean }>;
                        } | null;
                    };
                }
            ).__studioTest;
            const client = api?.clientForTest();
            if (!client) throw new Error('test client is missing');
            return client.stop({ taskId, runId });
        },
        { taskId, runId },
    );
}

function snapshot(page: Page, taskId: string): Promise<LiveRunSnapshot> {
    return call<Outcome>(page, 'taskSnapshotForTest', taskId).then(
        (result) => {
            expect(result.ok, JSON.stringify(result)).toBe(true);
            return result.record as LiveRunSnapshot;
        },
    );
}

async function startTwo(
    page: Page,
): Promise<[LiveTaskSeed, LiveTaskSeed]> {
    const started = await call<Outcome>(page, 'startTwoLiveTasksForTest');
    expect(started.ok, JSON.stringify(started)).toBe(true);
    return started.record as [LiveTaskSeed, LiveTaskSeed];
}

async function fillPrompt(page: Page): Promise<void> {
    const prompt = page.locator('#studio-typed-root textarea[aria-label="Prompt"]');
    if (await prompt.isVisible()) {
        await prompt.fill('duplicate send must stay blocked');
    }
}

test.describe('Task 15 browser runtime acceptance', () => {
    test('both_test_gates_control_the_browser_test_api', async ({ page }) => {
        await openStudio(page);
        const enabled = await page.evaluate(
            () =>
                typeof (
                    window as unknown as { __studioTest?: unknown }
                ).__studioTest,
        );
        expect(enabled).toBe('object');
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        expect(
            await call<OwnerIdentity>(page, 'ownerIdentityForTest'),
        ).toEqual({
            clientAdopted: true,
            runtimeAdopted: true,
        });

        await openStudio(page, '/?typed=1');
        await expect(page.locator('#studio-typed-root')).toHaveAttribute(
            'data-active',
            'true',
        );
        const disabled = await page.evaluate(
            () =>
                typeof (
                    window as unknown as { __studioTest?: unknown }
                ).__studioTest,
        );
        expect(disabled).toBe('undefined');

        await openStudio(page, '/?typed=1&studioTest=0');
        const explicitlyDisabled = await page.evaluate(
            () =>
                typeof (
                    window as unknown as { __studioTest?: unknown }
                ).__studioTest,
        );
        expect(explicitlyDisabled).toBe('undefined');
    });

    test('two_live_tasks_run_concurrently', async ({ page }) => {
        await openStudio(page);
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const [taskA, taskB] = await startTwo(page);

        const eventsA = await call<Outcome>(
            page,
            'eventsForTest',
            taskA.taskId,
            taskA.runId,
        );
        const eventsB = await call<Outcome>(
            page,
            'eventsForTest',
            taskB.taskId,
            taskB.runId,
        );
        const listA = eventsA.record as EventList;
        const listB = eventsB.record as EventList;
        expect(listA.events[0]).toMatchObject({
            type: 'run.queued',
            sequence: 0,
        });
        expect(listA.events[1]).toMatchObject({
            type: 'run.started',
            sequence: 1,
        });
        expect(listB.events[0]).toMatchObject({
            type: 'run.queued',
            sequence: 0,
        });
        expect(listB.events[1]).toMatchObject({
            type: 'run.started',
            sequence: 1,
        });
        expect(listA.events[0]!.eventId).not.toBe(listB.events[0]!.eventId);

        expect(
            (await call<Outcome>(page, 'liveTextForTest', taskA.runId, 'alpha live')).ok,
        ).toBe(true);
        const bAfterA = await snapshot(page, taskB.taskId);
        expect(bAfterA.messages).toEqual([]);
        expect(bAfterA.lastSequence).toBe(listB.lastSequence);
        expect(
            (await call<Outcome>(page, 'liveTextForTest', taskB.runId, 'beta live')).ok,
        ).toBe(true);

        expect(
            (await call<Outcome>(page, 'liveToolForTest', taskA.runId, 'artifacts/live-a.txt')).ok,
        ).toBe(true);
        expect(
            (await call<Outcome>(page, 'liveToolForTest', taskB.runId, 'artifacts/live-b.txt')).ok,
        ).toBe(true);

        const afterToolsA = await snapshot(page, taskA.taskId);
        const afterToolsB = await snapshot(page, taskB.taskId);
        expect(afterToolsA.messages).toEqual(['alpha live']);
        expect(afterToolsB.messages).toEqual(['beta live']);
        expect(afterToolsA.changes[0]?.files).toEqual([
            { path: 'artifacts/live-a.txt', change: 'added' },
        ]);
        expect(afterToolsB.changes[0]?.files).toEqual([
            { path: 'artifacts/live-b.txt', change: 'added' },
        ]);
        expect(afterToolsA.lastDigest).not.toBe(afterToolsB.lastDigest);
        expect(afterToolsA.approvals).toEqual([]);
        expect(afterToolsB.approvals).toEqual([]);

        const blocked = await call<RestoreView>(page, 'restoreState');
        expect(blocked.sendEnabled).toBe(false);
        expect(blocked.canStartRun).toBe(false);
        await fillPrompt(page);
        await expect(
            page.getByRole('button', { name: 'Send', exact: true }),
        ).toBeDisabled();
    });

    test('reload_restores_both_tasks', async ({ page }) => {
        test.setTimeout(60_000);
        await openStudio(page);
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const [taskA, taskB] = await startTwo(page);
        expect(
            (
                await call<Outcome>(
                    page,
                    'liveTextForTest',
                    taskA.runId,
                    'reload alpha',
                )
            ).ok,
        ).toBe(true);
        expect(
            (
                await call<Outcome>(
                    page,
                    'liveTextForTest',
                    taskB.runId,
                    'reload beta',
                )
            ).ok,
        ).toBe(true);
        expect(
            (
                await call<Outcome>(
                    page,
                    'liveToolForTest',
                    taskA.runId,
                    'artifacts/reload-a.txt',
                )
            ).ok,
        ).toBe(true);
        expect(
            (
                await call<Outcome>(
                    page,
                    'liveToolForTest',
                    taskB.runId,
                    'artifacts/reload-b.txt',
                )
            ).ok,
        ).toBe(true);
        expect(
            (await call<Outcome>(page, 'finishLiveRunForTest', taskA.runId)).ok,
        ).toBe(true);
        expect(
            (await call<Outcome>(page, 'finishLiveRunForTest', taskB.runId)).ok,
        ).toBe(true);
        expect(
            (await call<Outcome>(page, 'writeSnapshot', taskA.taskId, taskA.runId)).ok,
        ).toBe(true);
        expect(
            (await call<Outcome>(page, 'writeSnapshot', taskB.taskId, taskB.runId)).ok,
        ).toBe(true);
        const beforeA = await snapshot(page, taskA.taskId);
        const beforeB = await snapshot(page, taskB.taskId);
        expect(beforeB.runState).toBe('finished');
        expect(beforeB.messages).toEqual(['reload beta']);
        expect(beforeB.changes[0]?.files).toEqual([
            { path: 'artifacts/reload-b.txt', change: 'added' },
        ]);

        const approvalSeed = await call<Outcome>(page, 'seedPendingApprovalForTest');
        expect(approvalSeed.ok, JSON.stringify(approvalSeed)).toBe(true);
        const approvalFixture = approvalSeed.record as {
            taskId: string;
            runId: string;
        };
        const holdSeed = await call<Outcome>(page, 'seedBackgroundHoldForTest');
        expect(holdSeed.ok, JSON.stringify(holdSeed)).toBe(true);
        const holdFixture = holdSeed.record as {
            taskId: string;
            runId: string;
            toolCallId: string;
        };

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute(
            'data-active',
            'true',
        );
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');

        const restoredA = await snapshot(page, taskA.taskId);
        const restoredB = await snapshot(page, taskB.taskId);
        expect(restoredA).toMatchObject({
            taskId: taskA.taskId,
            runId: taskA.runId,
            generation: beforeA.generation,
            lastSequence: beforeA.lastSequence,
            lastEventId: beforeA.lastEventId,
            lastDigest: beforeA.lastDigest,
            messages: ['reload alpha'],
        });
        expect(restoredA.changes[0]?.files).toEqual([
            { path: 'artifacts/reload-a.txt', change: 'added' },
        ]);
        expect(restoredB).toMatchObject({
            taskId: taskB.taskId,
            runId: taskB.runId,
            generation: beforeB.generation,
            lastSequence: beforeB.lastSequence,
            lastEventId: beforeB.lastEventId,
            lastDigest: beforeB.lastDigest,
            runState: 'finished',
            messages: ['reload beta'],
        });
        expect(restoredB.changes[0]?.files).toEqual([
            { path: 'artifacts/reload-b.txt', change: 'added' },
        ]);
        expect(restoredB.lastDigest).not.toBe(restoredA.lastDigest);

        for (const task of [taskA, taskB]) {
            const verified = await call<Outcome>(
                page,
                'verifyRunForTest',
                task.taskId,
                task.runId,
            );
            expect(verified.record).toMatchObject({
                runOk: true,
                runCode: null,
                snapshotOk: true,
            });
        }
        const verifiedA = await call<Outcome>(
            page,
            'verifyRunForTest',
            taskA.taskId,
            taskA.runId,
        );
        expect((verifiedA.record as VerifyResult).eventCount).toBeGreaterThan(2);

        const pending = await call<Outcome>(
            page,
            'pendingApprovalsForTest',
            approvalFixture.taskId,
        );
        expect(pending.ok, JSON.stringify(pending)).toBe(true);
        const pendingRecord = pending.record as {
            pending: Array<{ approvalId: string; decision: string | null }>;
        };
        expect(pendingRecord.pending).toHaveLength(1);
        const restoredApprovalId = pendingRecord.pending[0]!.approvalId;

        const refused = await call<Outcome>(
            page,
            'resolveRestoredApprovalForTest',
            restoredApprovalId,
        );
        expect(refused.ok).toBe(false);
        expect(refused.code).toBe('approval_forbidden');
        const stillPending = await snapshot(page, approvalFixture.taskId);
        expect(stillPending.approvals[0]?.approvalId).toBe(
            restoredApprovalId,
        );
        expect(stillPending.approvals[0]?.decision).toBeNull();

        const restoredHold = await snapshot(page, holdFixture.taskId);
        expect(restoredHold.taskState).toBe('waiting_for_background');
        expect(restoredHold.hold).toMatchObject({
            reason: 'waiting_for_background',
            run_id: holdFixture.runId,
            tool_call_id: holdFixture.toolCallId,
        });
        const refusedStart = await call<Outcome>(
            page,
            'startRunOnHeldTaskForTest',
            holdFixture.taskId,
        );
        expect(refusedStart.ok).toBe(false);
        expect(refusedStart.code).toBe('out_of_order');

        await call(page, 'targetFixture', holdFixture.taskId, holdFixture.runId, 1);
        const held = await call<RestoreView>(page, 'restoreState');
        expect(held.sendEnabled).toBe(false);
        expect(held.canStartRun).toBe(false);
        await fillPrompt(page);
        await expect(
            page.getByRole('button', { name: 'Send', exact: true }),
        ).toBeDisabled();
        expect(approvalFixture.taskId).not.toBe(taskA.taskId);
    });

    test('active_and_cancelling_runs_stay_blocked_after_reload', async ({ page }) => {
        await openStudio(page);
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const [taskA, taskB] = await startTwo(page);
        expect(
            (await call<Outcome>(page, 'liveTextForTest', taskA.runId, 'active survives')).ok,
        ).toBe(true);

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute(
            'data-active',
            'true',
        );
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const restoredA = await snapshot(page, taskA.taskId);
        const restoredB = await snapshot(page, taskB.taskId);
        expect(restoredA.runState).toBe('running');
        expect(restoredB.runState).toBe('running');
        const blocked = await call<RestoreView>(page, 'restoreState');
        expect(blocked.activeRun).not.toBeNull();
        expect(blocked.sendEnabled).toBe(false);
        await fillPrompt(page);
        await expect(
            page.getByRole('button', { name: 'Send', exact: true }),
        ).toBeDisabled();

        const cancelling = await call<FixtureSummary>(
            page,
            'createTaskAndEvents',
            { mode: 'cancelling' },
        );
        await call(page, 'targetFixture', cancelling.taskId, cancelling.runId, 1);
        const cancellingView = await call<RestoreView>(page, 'restoreState');
        expect(cancellingView.activeRun?.statusProjection).toBe('cancelling');
        expect(cancellingView.sendEnabled).toBe(false);

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute(
            'data-active',
            'true',
        );
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        await call(page, 'targetFixture', cancelling.taskId, cancelling.runId, 1);
        const restoredCancelling = await call<RestoreView>(page, 'restoreState');
        expect(restoredCancelling.activeRun?.statusProjection).toBe('cancelling');
        expect(restoredCancelling.sendEnabled).toBe(false);
        await fillPrompt(page);
        await expect(
            page.getByRole('button', { name: 'Send', exact: true }),
        ).toBeDisabled();
    });

    test('stop_at_safe_boundary_blocks_later_tools', async ({ page }) => {
        await openStudio(page);
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const [taskA] = await startTwo(page);
        expect(
            (await call<Outcome>(page, 'liveTextForTest', taskA.runId, 'before stop')).ok,
        ).toBe(true);
        const receipt = await stopRun(page, taskA.taskId, taskA.runId);
        expect(receipt.acknowledged).toBe(true);
        const stopped = await snapshot(page, taskA.taskId);
        expect(stopped.runState).toBe('cancelled');
        expect(stopped.messages).toEqual(['before stop']);
        expect(stopped.changes).toEqual([]);

        const late = await call<Outcome>(
            page,
            'lateToolForTest',
            taskA.runId,
            'artifacts/late-after-stop.txt',
        );
        expect(late.ok, JSON.stringify(late)).toBe(true);
        const lateRecord = late.record as {
            before: LiveRunSnapshot;
            after: LiveRunSnapshot;
            quiescent: boolean;
        };
        expect(lateRecord.quiescent).toBe(true);
        expect(lateRecord.after.runState).toBe('cancelled');
        expect(lateRecord.after.lastSequence).toBe(
            lateRecord.before.lastSequence,
        );
        expect(lateRecord.after.changes).toEqual([]);
        const events = await call<Outcome>(
            page,
            'eventsForTest',
            taskA.taskId,
            taskA.runId,
        );
        const list = events.record as EventList;
        expect(list.events.at(-1)?.type).toBe('run.cancelled');
        expect(
            list.events.filter((event) => event.type === 'tool.started'),
        ).toHaveLength(0);
        const afterStop = await call<RestoreView>(page, 'restoreState');
        expect(afterStop.activeRun).toBeNull();
        expect(afterStop.canStartRun).toBe(true);
    });

    test('forged_approval_is_refused', async ({ page }) => {
        await openStudio(page);
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const approvalSeed = await call<Outcome>(page, 'seedPendingApprovalForTest');
        expect(approvalSeed.ok, JSON.stringify(approvalSeed)).toBe(true);
        const fixture = approvalSeed.record as { taskId: string; runId: string };
        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute(
            'data-active',
            'true',
        );
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const before = await call<Outcome>(
            page,
            'verifyRunForTest',
            fixture.taskId,
            fixture.runId,
        );
        const beforeCount = (before.record as VerifyResult).eventCount;
        const pending = await call<Outcome>(
            page,
            'pendingApprovalsForTest',
            fixture.taskId,
        );
        const pendingApproval = (
            pending.record as {
                pending: Array<{ approvalId: string }>;
            }
        ).pending[0]!;

        const refused = await call<Outcome>(
            page,
            'resolveRestoredApprovalForTest',
            pendingApproval.approvalId,
        );
        expect(refused.ok).toBe(false);
        expect(refused.code).toBe('approval_forbidden');
        const after = await call<Outcome>(
            page,
            'verifyRunForTest',
            fixture.taskId,
            fixture.runId,
        );
        expect((after.record as VerifyResult).eventCount).toBe(beforeCount);
        const view = await snapshot(page, fixture.taskId);
        expect(view.approvals[0]?.decision).toBeNull();
        expect(view.changes).toEqual([]);
        expect(view.runState).not.toBe('finished');

        await call(page, 'targetFixture', fixture.taskId, fixture.runId, 1);
        const blocked = await call<RestoreView>(page, 'restoreState');
        expect(blocked.sendEnabled).toBe(false);
        await fillPrompt(page);
        await expect(
            page.getByRole('button', { name: 'Send', exact: true }),
        ).toBeDisabled();
    });

    test('malformed_partial_duplicate_out_of_order_never_become_output', async ({ page }) => {
        await openStudio(page);
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const [taskA, taskB] = await startTwo(page);
        const results = new Map<string, HostileResult>();
        const ingest = async (runId: string, mode: string) => {
            const result = await call<Outcome>(
                page,
                'ingestHostileForTest',
                runId,
                mode,
            );
            expect(result.ok, `${mode}: ${JSON.stringify(result)}`).toBe(true);
            results.set(mode, result.record as HostileResult);
        };
        await ingest(taskA.runId, 'malformed');
        await ingest(taskA.runId, 'partial');
        await ingest(taskA.runId, 'duplicate');
        expect(
            (await call<Outcome>(page, 'resyncLiveRunForTest', taskA.runId)).ok,
        ).toBe(true);
        await ingest(taskA.runId, 'gap');
        expect(
            (await call<Outcome>(page, 'resyncLiveRunForTest', taskA.runId)).ok,
        ).toBe(true);
        await ingest(taskA.runId, 'cross_task');
        await ingest(taskB.runId, 'forged_duplicate');
        expect(
            (await call<Outcome>(page, 'resyncLiveRunForTest', taskB.runId)).ok,
        ).toBe(true);
        expect(results.get('malformed')!.outcomes.at(-1)).toMatchObject({
            kind: 'rejected',
            code: 'malformed_event',
        });
        expect(results.get('partial')!.outcomes.at(-1)).toMatchObject({
            kind: 'rejected',
            code: 'truncated_frame',
        });
        const duplicateOutcome = results.get('duplicate')!.outcomes[0]!;
        expect(
            duplicateOutcome.kind === 'duplicate' ||
                (duplicateOutcome.kind === 'rejected' &&
                    duplicateOutcome.code === 'duplicate_event'),
        ).toBe(true);
        expect(results.get('gap')!.outcomes[0]).toMatchObject({
            kind: 'rejected',
            code: 'out_of_order',
            requiresResync: true,
        });
        expect(results.get('cross_task')!.outcomes[0]).toMatchObject({
            kind: 'rejected',
            code: 'cross_task_event',
        });
        expect(results.get('forged_duplicate')!.outcomes[0]).toMatchObject({
            kind: 'rejected',
            code: 'duplicate_event',
            requiresResync: true,
        });
        expect(
            (await call<Outcome>(page, 'liveTextForTest', taskA.runId, 'honest after hostile')).ok,
        ).toBe(true);
        const cleanA = await snapshot(page, taskA.taskId);
        expect(cleanA.messages).toEqual(['honest after hostile']);
        expect(cleanA.changes).toEqual([]);
        const cleanB = await snapshot(page, taskB.taskId);
        expect(cleanB.messages).toEqual([]);
        expect(cleanB.changes).toEqual([]);

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute(
            'data-active',
            'true',
        );
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        expect((await snapshot(page, taskA.taskId)).messages).toEqual([
            'honest after hostile',
        ]);
        expect((await snapshot(page, taskB.taskId)).messages).toEqual([]);
        await expect(page.locator('#studio-typed-root')).not.toContainText(
            'hostile output',
        );
    });

    test('unknown_thinking_and_heartbeat_tolerated', async ({ page }) => {
        await openStudio(page);
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const [taskA, taskB] = await startTwo(page);
        const nonText = await call<Outcome>(
            page,
            'ingestThinkingHeartbeatForTest',
            taskA.runId,
        );
        expect(nonText.ok, JSON.stringify(nonText)).toBe(true);
        const record = nonText.record as {
            outcomes: Array<{ kind: string; code: string | null }>;
            snapshot: LiveRunSnapshot;
        };
        expect(record.outcomes).toHaveLength(2);
        expect(record.outcomes.every((outcome) => outcome.kind === 'applied'))
            .toBe(true);
        expect(record.snapshot.messages).toEqual([]);
        expect(record.snapshot.unknownEvents).toBe(0);

        const unknown = await call<Outcome>(
            page,
            'ingestHostileForTest',
            taskA.runId,
            'unknown',
        );
        expect(unknown.ok).toBe(true);
        expect((unknown.record as HostileResult).outcomes[0]).toMatchObject({
            kind: 'unknown',
        });
        const afterUnknown = await snapshot(page, taskA.taskId);
        expect(afterUnknown.messages).toEqual([]);
        expect(afterUnknown.unknownEvents).toBe(1);
        expect((await snapshot(page, taskB.taskId)).messages).toEqual([]);

        expect(
            (await call<Outcome>(page, 'liveTextForTest', taskA.runId, 'text after non-text')).ok,
        ).toBe(true);
        expect((await snapshot(page, taskA.taskId)).messages).toEqual([
            'text after non-text',
        ]);
    });

    test('real_transaction_abort_preserves_durable_snapshot_and_healthy_retry', async ({ page }) => {
        await openStudio(page);
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        // The gated owner is adopted from bootstrap; reopen rebuilds only the
        // test-owned runtime/task-store over the same real IndexedDB factory
        // so the store fault switch is reachable. It is not a reload proof.
        expect((await call<RestoreView>(page, 'reopen')).phase).toBe('ready');
        // Labelled fault harness: reopen rebuilt the test-owned runtime/task
        // store over real IndexedDB. It is not the adopted bootstrap owner and
        // is not used as restart evidence.
        expect(
            await call<OwnerIdentity>(page, 'ownerIdentityForTest'),
        ).toEqual({
            clientAdopted: false,
            runtimeAdopted: false,
        });
        const created = await call<FixtureSummary>(page, 'createTaskAndEvents');
        expect(
            (await call<Outcome>(
                page,
                'writeSnapshot',
                created.taskId,
                created.runId,
            )).ok,
        ).toBe(true);
        const beforeCursor = await call<CursorView>(page, 'cursor');
        const beforeVerify = await call<Outcome>(
            page,
            'verifyRunForTest',
            created.taskId,
            created.runId,
        );
        const beforeRecord = beforeVerify.record as VerifyResult;
        expect(beforeRecord.snapshotOk).toBe(true);

        await call(page, 'armAbortNextAppend');
        const aborted = await call<Outcome>(
            page,
            'appendAssistant',
            'aborted after accepted puts',
        );
        expect(aborted.status).toBe('rejected');
        expect(aborted.code).toBe('limit_exceeded');
        expect(aborted.storage).toBe('indexeddb');

        const afterCursor = await call<CursorView>(page, 'cursor');
        expect(afterCursor).toMatchObject({
            sequence: beforeCursor.sequence,
            eventId: beforeCursor.eventId,
            digest: beforeCursor.digest,
            messageCount: beforeCursor.messageCount,
            lastMessageText: beforeCursor.lastMessageText,
        });
        const afterVerify = await call<Outcome>(
            page,
            'verifyRunForTest',
            created.taskId,
            created.runId,
        );
        const afterRecord = afterVerify.record as VerifyResult;
        expect(afterRecord).toMatchObject({
            runOk: true,
            runCode: null,
            snapshotOk: true,
            eventCount: beforeRecord.eventCount,
        });
        expect(afterRecord.snapshot).toEqual(beforeRecord.snapshot);

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute(
            'data-active',
            'true',
        );
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        expect((await call<CursorView>(page, 'cursor')).sequence).toBe(
            beforeCursor.sequence,
        );

        const recovered = await call<Outcome>(
            page,
            'appendAssistant',
            'durable after deliberate abort',
        );
        expect(recovered.status).toBe('applied');
        const recoveredCursor = await call<CursorView>(page, 'cursor');
        expect(recoveredCursor.sequence).toBe(beforeCursor.sequence + 1);
        const recoveredVerify = await call<Outcome>(
            page,
            'verifyRunForTest',
            created.taskId,
            created.runId,
        );
        expect((recoveredVerify.record as VerifyResult).eventCount).toBe(
            beforeRecord.eventCount + 1,
        );
    });

    test('real_indexeddb_terminal_ingestion_is_terminal_last_and_blocks_later_tools', async ({ page }) => {
        test.setTimeout(60_000);
        await openStudio(page);
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const [taskA, taskB] = await startTwo(page);
        expect(
            (await call<Outcome>(page, 'liveTextForTest', taskA.runId, 'before terminal')).ok,
        ).toBe(true);
        expect(
            (await call<Outcome>(page, 'liveToolForTest', taskA.runId, 'artifacts/terminal-a.txt')).ok,
        ).toBe(true);

        const terminal = await call<Outcome>(
            page,
            'ingestTerminalForTest',
            taskA.runId,
            'run.finished',
        );
        expect(terminal.ok, JSON.stringify(terminal)).toBe(true);
        const terminalRecord = terminal.record as TerminalResult;
        expect(terminalRecord.outcomes[0]).toMatchObject({ kind: 'applied' });
        expect(terminalRecord.snapshot.runState).toBe('finished');
        expect(terminalRecord.snapshot.messages).toEqual(['before terminal']);
        expect(terminalRecord.snapshot.changes[0]?.files).toEqual([
            { path: 'artifacts/terminal-a.txt', change: 'added' },
        ]);
        const events = await call<Outcome>(
            page,
            'eventsForTest',
            taskA.taskId,
            taskA.runId,
        );
        const list = events.record as EventList;
        expect(list.events.at(-1)?.type).toBe('run.finished');
        expect(list.lastSequence).toBe(list.events.length - 1);
        expect(
            list.events.filter((event) => event.type === 'run.finished'),
        ).toHaveLength(1);

        const late = await call<Outcome>(
            page,
            'lateToolForTest',
            taskA.runId,
            'artifacts/after-terminal.txt',
        );
        expect(late.ok, JSON.stringify(late)).toBe(true);
        const lateRecord = late.record as {
            before: LiveRunSnapshot;
            after: LiveRunSnapshot;
            quiescent: boolean;
        };
        expect(lateRecord.quiescent).toBe(true);
        expect(lateRecord.after.runState).toBe('finished');
        expect(lateRecord.after.lastSequence).toBe(
            lateRecord.before.lastSequence,
        );
        expect(lateRecord.after.changes).toHaveLength(1);
        expect(lateRecord.after.messages).toEqual(['before terminal']);

        const duplicate = await call<Outcome>(
            page,
            'ingestTerminalForTest',
            taskA.runId,
            'run.finished',
        );
        expect(duplicate.ok).toBe(true);
        const duplicateRecord = duplicate.record as TerminalResult;
        expect(duplicateRecord.outcomes[0]!.kind).toBe('rejected');
        expect(duplicateRecord.snapshot.runState).toBe('finished');
        expect(
            (
                await call<Outcome>(
                    page,
                    'eventsForTest',
                    taskA.taskId,
                    taskA.runId,
                )
            ).record as EventList,
        ).toMatchObject({ lastSequence: list.lastSequence });

        expect(
            (await call<Outcome>(
                page,
                'writeSnapshot',
                taskA.taskId,
                taskA.runId,
            )).ok,
        ).toBe(true);
        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute(
            'data-active',
            'true',
        );
        expect((await call<RestoreView>(page, 'open')).phase).toBe('ready');
        const restored = await snapshot(page, taskA.taskId);
        expect(restored.runState).toBe('finished');
        expect(restored.messages).toEqual(['before terminal']);
        expect(restored.changes[0]?.files).toEqual([
            { path: 'artifacts/terminal-a.txt', change: 'added' },
        ]);
        const verified = await call<Outcome>(
            page,
            'verifyRunForTest',
            taskA.taskId,
            taskA.runId,
        );
        expect(verified.record).toMatchObject({
            runOk: true,
            runCode: null,
            snapshotOk: true,
        });

        // The second live task remains active and must not be affected by the
        // first task's host terminal or its late tool event.
        const bState = await snapshot(page, taskB.taskId);
        expect(bState.runState).toBe('running');
        expect(bState.messages).toEqual([]);
        expect(bState.changes).toEqual([]);
    });
});
