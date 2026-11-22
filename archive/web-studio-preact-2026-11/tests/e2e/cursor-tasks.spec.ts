import { expect, test, type BrowserContext, type Page } from '@playwright/test';

const TEST_URL = '/?typed=1&studioTest=1';

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
    latestRunId: string | null;
    runId: string | null;
    runState: string | null;
    retryOfRunId?: string | null;
    messages: string[];
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

function detail(page: Page) {
    return page.locator('#studio-typed-root main[data-region="task-detail"]');
}

async function activeClass(page: Page): Promise<string> {
    return page.evaluate(() => {
        const active = document.activeElement as HTMLElement | null;
        return active === null ? '' : `${active.tagName}:${active.className}`;
    });
}

async function openRailIfClosed(page: Page): Promise<void> {
    const rail = page.locator('#studio-typed-root [data-region="task-rail"]');
    if (await rail.isHidden()) {
        await page.locator('#studio-typed-root [data-action="toggle-projects"]').click();
        await expect(rail).toBeVisible();
    }
}

test.describe('Task 14 browser task detail, timeline, approvals and errors', () => {
    test('replay_detail_collapses_deltas_pairs_tools_and_never_fabricates_output', async ({ page }) => {
        await openStudio(page);
        expect((await call<Outcome>(page, 'open')).ok).not.toBe(false);
        const seeded = await call<{ taskId: string }>(page, 'createTaskAndEvents', {
            mode: 'detail',
            title: 'Replay detail task',
        });
        expect(seeded.taskId).toMatch(/^tsk_[0-9a-f]{16}$/);
        const taskId = seeded.taskId;

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await selectTask(page, taskId);
        const view = detail(page);

        await expect(view.locator('[data-task-detail-header]')).toContainText('Finished');
        await expect(view.locator('[data-task-detail-header]')).toContainText('Replay detail task');
        await expect(view.locator('[data-timeline-kind="assistant"]')).toHaveCount(1);
        await expect(view.locator('[data-timeline-kind="assistant"]')).toContainText('Hello from replay');
        await expect(view.locator('[data-timeline-kind="tool"]')).toHaveCount(1);
        await expect(view.locator('[data-tool-call-id="tool_detail_1"]')).toContainText('{"path":"notes.md"}');
        await expect(view.locator('[data-tool-call-id="tool_detail_1"]')).toContainText('wrote notes');
        await expect(view.locator('[data-timeline-kind="unknown"]')).toContainText('host.instruction');
        await expect(view).not.toContainText('hostile');
        await expect(view.locator('[data-task-error="run"]')).toContainText('malformed_event');
        await expect(view.locator('[data-task-error="run"]')).toContainText('The event stream contained an invalid event. The transcript was not changed.');
        await expect(view.locator('[data-timeline-kind="usage"]')).toContainText('12 input tokens');
        await expect(view.locator('[data-task-plan]')).toContainText('Replay plan');
        await expect(view.locator('[data-task-plan]')).toContainText('Read request');
        await expect(view.locator('[data-task-plan]')).toContainText('Write answer');
        await expect(view).toContainText('Synthetic replay');
        await expect(view).toContainText('not an R4 model');
        await expect(view.locator('.agent-composer')).toBeVisible();
        await expect(view.locator('textarea[aria-label="Prompt"]')).toBeVisible();

        await expect(view.locator('.run-status').getByRole('button', { name: 'Stop', exact: true })).toHaveCount(0);
        await expect(view.getByRole('button', { name: 'Retry', exact: true })).toHaveCount(0);
        await expect(view.getByRole('button', { name: 'Restore Checkpoint', exact: true })).toHaveCount(0);
        await expect(view.getByRole('button', { name: 'Pause', exact: true })).toHaveCount(0);
        await expect(view.getByRole('button', { name: 'Delete', exact: true })).toHaveCount(0);
        await expect(view.getByRole('tab', { name: 'Logs' })).toHaveCount(0);
        await expect(view).not.toContainText(/\$\s?\d/);
    });

    test('approval_card_shows_tool_scope_expiry_and_static_host_without_page_authority', async ({ page }) => {
        await openStudio(page);
        await call<Outcome>(page, 'open');
        const seeded = await call<Outcome>(page, 'seedPendingApprovalForTest');
        expect(seeded.ok, JSON.stringify(seeded)).toBe(true);
        const taskId = (seeded.record as { taskId: string }).taskId;

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await selectTask(page, taskId);
        const card = detail(page).locator('[data-approval-id="appr_reload_approval"]');
        await expect(card).toBeVisible();
        await expect(card).toContainText('write_file');
        await expect(card).toContainText('tool:write_file');
        await expect(card.locator('time')).toHaveAttribute('datetime', new Date(1_794_283_800_000).toISOString());
        await expect(card).toContainText('Host: not connected');
        await expect(card).toContainText('This action needs host approval. Approve it in the local companion.');
        await expect(card.getByRole('button', { name: 'Approve', exact: true })).toBeDisabled();
        await expect(card.getByRole('button', { name: 'Deny', exact: true })).toBeDisabled();
        await expect(detail(page).locator('[data-task-detail-header]')).toContainText('Waiting for approval');
    });

    test('stop_is_active_only_and_delivery_errors_stay_message_scoped', async ({ page }) => {
        await openStudio(page);
        await call<Outcome>(page, 'open');
        const started = await call<Outcome>(page, 'startTwoLiveTasksForTest');
        expect(started.ok, JSON.stringify(started)).toBe(true);
        const [task] = started.record as LiveTaskSeed[];

        await selectTask(page, task.taskId);
        const queued = await call<Outcome>(page, 'queueTaskMessageForTest', task.taskId, 'queued follow-up');
        expect(queued.ok, JSON.stringify(queued)).toBe(true);
        const messageId = (queued.record as { messageId: string }).messageId;

        const stop = detail(page).locator('.run-status').getByRole('button', { name: 'Stop', exact: true });
        await expect(stop).toBeEnabled();
        await stop.click();

        await expect(detail(page).locator('.run-status')).toContainText('Cancelled');
        await expect(detail(page).locator('.run-status').getByRole('button', { name: 'Stop', exact: true })).toHaveCount(0);
        await expect(detail(page).getByRole('button', { name: 'Pause', exact: true })).toHaveCount(0);
        const delivery = detail(page).locator('[data-timeline-kind="delivery"]');
        await expect(delivery).toHaveCount(1);
        await expect(delivery).toContainText(messageId);
        await expect(delivery).toContainText('was not delivered');
        await expect(detail(page).locator('[data-task-error="run"]')).toHaveCount(0);
        await expect(detail(page)).not.toContainText('message_not_delivered:');
    });

    test('retry_appears_only_for_retryable_failure_and_uses_the_real_runtime', async ({ page }) => {
        await openStudio(page);
        await call<Outcome>(page, 'open');
        const started = await call<Outcome>(page, 'startTwoLiveTasksForTest');
        expect(started.ok, JSON.stringify(started)).toBe(true);
        const [task] = started.record as LiveTaskSeed[];

        const failed = await call<Outcome>(page, 'failLiveRunForTest', task.runId);
        expect(failed.ok, JSON.stringify(failed)).toBe(true);
        await selectTask(page, task.taskId);
        await expect(detail(page).locator('[data-task-error="run"]')).toContainText('model_unavailable');
        await expect(detail(page).locator('[data-task-error="run"]')).toContainText('synthetic retryable failure');
        const retry = detail(page).getByRole('button', { name: 'Retry', exact: true });
        await expect(retry).toBeVisible();
        await retry.click();

        await expect(detail(page).locator('.run-status')).toContainText('Running');
        await expect(detail(page).getByRole('button', { name: 'Retry', exact: true })).toHaveCount(0);
        const snapshot = (await call<Outcome>(page, 'taskSnapshotForTest', task.taskId))
            .record as LiveRunSnapshot;
        expect(snapshot.runId).not.toBe(task.runId);
        expect(snapshot.retryOfRunId).toBe(task.runId);
        expect(snapshot.runState).toBe('running');
    });

    test('the_gated_api_and_the_ui_share_one_runtime_owner_across_a_reload', async ({ page }) => {
        await openStudio(page);
        await call<Outcome>(page, 'open');
        const fixture = await call<{ taskId: string; runId: string }>(page, 'createClientRun');
        expect(fixture.taskId).toMatch(/^tsk_[0-9a-f]{16}$/);
        expect(fixture.runId).toMatch(/^run_[0-9a-f]{16}$/);

        await expect(page.locator(`#studio-typed-root [data-task-id="${fixture.taskId}"]`)).toBeVisible();
        await selectTask(page, fixture.taskId);
        const text = await call<Outcome>(page, 'liveTextForTest', fixture.runId, 'shared owner replay text');
        expect(text.ok, JSON.stringify(text)).toBe(true);
        await expect(detail(page).locator('[data-timeline-kind="assistant"]')).toContainText('shared owner replay text');

        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await selectTask(page, fixture.taskId);
        await expect(detail(page).locator('[data-timeline-kind="assistant"]')).toContainText('shared owner replay text');
    });

    test('a_persisted_cancelling_projection_reloads_as_cancelling_without_an_enabled_stop', async ({ page }) => {
        await openStudio(page);
        await call<Outcome>(page, 'open');
        const seeded = await call<{ taskId: string }>(page, 'createTaskAndEvents', { mode: 'cancelling' });
        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await selectTask(page, seeded.taskId);
        await expect(page.locator(`#studio-typed-root [data-task-id="${seeded.taskId}"] .task-state`)).toHaveText('Cancelling');
        await expect(detail(page).locator('.run-status-label')).toHaveText('Cancelling');
        const stop = detail(page).locator('.run-status .run-stop');
        if (await stop.count() > 0) await expect(stop).toBeDisabled();
        await expect(detail(page).locator('.run-progress')).toHaveCount(0);
        await expect(detail(page).locator('.run-status')).not.toContainText('Running');
    });

    test('back_clears_the_runtime_selection_returns_row_focus_and_cannot_send_to_a_hidden_task', async ({ page }) => {
        await openStudio(page);
        await call<Outcome>(page, 'open');
        const seeded = await call<{ taskId: string; runId: string }>(page, 'createTaskAndEvents', {
            mode: 'detail',
            title: 'Back focus task',
        });
        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await selectTask(page, seeded.taskId);
        await expect.poll(() => activeClass(page)).toContain('task-detail-title');

        await detail(page).getByRole('button', { name: 'Back to tasks', exact: true }).click();
        await expect(page.locator('#studio-typed-root h1')).toHaveText('Agent');
        await expect.poll(() => activeClass(page)).toContain('task-select');

        const prompt = page.locator('#studio-typed-root textarea[aria-label="Prompt"]');
        await prompt.fill('back then send');
        await expect(page.getByRole('button', { name: 'Send', exact: true })).toBeDisabled();
        const before = (await call<Outcome>(page, 'taskSnapshotForTest', seeded.taskId))
            .record as LiveRunSnapshot;
        await prompt.press('Enter');
        await page.waitForTimeout(200);
        const after = (await call<Outcome>(page, 'taskSnapshotForTest', seeded.taskId))
            .record as LiveRunSnapshot;
        expect(after.runId).toBe(before.runId);
        await expect(prompt).toHaveValue('back then send');
    });

    test('fresh_shell_send_creates_the_first_runtime_task', async ({ page }) => {
        await openStudio(page);
        const prompt = page.locator('#studio-typed-root textarea[aria-label="Prompt"]');
        await prompt.fill('fresh shell prompt');
        const send = page.getByRole('button', { name: 'Send', exact: true });
        await expect(send).toBeEnabled();
        await prompt.press('Enter');
        await expect(page.locator('#studio-typed-root [data-task-id]')).toHaveCount(1);
        await expect(page.locator('#studio-typed-root [data-task-id] .task-state')).toHaveText(/Running|Finished/);
        await expect(prompt).toHaveValue('');
    });

    test('an_out_of_range_approval_expiry_renders_not_recorded', async ({ page }) => {
        await openStudio(page);
        await call<Outcome>(page, 'open');
        const seeded = await call<{ taskId: string }>(page, 'createTaskAndEvents', {
            mode: 'approval',
            title: 'Out of range approval',
            expiresAtMs: Number.MAX_SAFE_INTEGER,
        });
        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await selectTask(page, seeded.taskId);
        const card = detail(page).locator('[data-approval-id="appr_reload_approval"]');
        await expect(card).toContainText('Not recorded');
        await expect(card.locator('time')).toHaveCount(0);
        await expect(detail(page)).not.toContainText('RangeError');
    });

    test('detail_focus_returns_and_target_sizes_hold_at_390_and_1024', async ({ page }) => {
        await openStudio(page);
        await call<Outcome>(page, 'open');
        const seeded = await call<{ taskId: string }>(page, 'createTaskAndEvents', {
            mode: 'approval',
            title: 'Responsive focus task',
        });
        for (const width of [390, 1024]) {
            await page.setViewportSize({ width, height: 900 });
            await page.reload();
            await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
            await openRailIfClosed(page);
            await selectTask(page, seeded.taskId);
            await expect.poll(() => activeClass(page)).toContain('task-detail-title');
            const outline = await page.locator('#studio-typed-root .task-detail-title').evaluate((element) => {
                const style = getComputedStyle(element);
                return { style: style.outlineStyle, width: style.outlineWidth };
            });
            expect(outline.style).not.toBe('none');
            expect(outline.width).toBe('2px');
            const undersized = await page.evaluate(() => {
                const root = document.querySelector<HTMLElement>('#studio-typed-root main[data-region="task-detail"]');
                if (root === null) return [{ name: 'missing detail', width: 0, height: 0 }];
                const result: Array<{ name: string; width: number; height: number }> = [];
                for (const element of Array.from(root.querySelectorAll<HTMLElement>('button, a[href], summary'))) {
                    const style = getComputedStyle(element);
                    const rect = element.getBoundingClientRect();
                    if (style.display === 'none' || style.visibility === 'hidden' || rect.width === 0 || rect.height === 0) continue;
                    if (rect.width + 0.01 < 24 || rect.height + 0.01 < 24) {
                        result.push({
                            name: (element.getAttribute('aria-label') || element.textContent || element.tagName).trim().slice(0, 60),
                            width: Math.round(rect.width * 10) / 10,
                            height: Math.round(rect.height * 10) / 10,
                        });
                    }
                }
                return result;
            });
            expect(undersized, `width ${width}`).toEqual([]);
            if (await page.locator('#studio-typed-root [data-region="task-rail"]').isHidden()) {
                await page.locator('#studio-typed-root [data-action="toggle-projects"]').click();
                await expect(page.locator('#studio-typed-root [data-region="task-rail"]')).toBeVisible();
            }
            const localSessions = page.locator('#studio-typed-root details[data-task-group="local-sessions"] summary');
            await expect(localSessions).toBeVisible();
            const summaryBox = await localSessions.boundingBox();
            expect(summaryBox).not.toBeNull();
            expect(summaryBox!.width + 0.01).toBeGreaterThanOrEqual(24);
            expect(summaryBox!.height + 0.01).toBeGreaterThanOrEqual(24);
            if (await page.locator('#studio-typed-root [data-region="task-rail"]').isVisible()) {
                await page.locator('#studio-typed-root [data-action="toggle-projects"]').click();
            }
            await detail(page).getByRole('button', { name: 'Back to tasks', exact: true }).click();
            await expect.poll(() => activeClass(page)).toContain('task-select');
        }
    });

    test('a_terminal_status_projection_does_not_make_an_active_run_terminal', async ({ page }) => {
        await openStudio(page);
        await call<Outcome>(page, 'open');
        const seeded = await call<{ taskId: string }>(page, 'createTaskAndEvents', {
            mode: 'false-finished',
            title: 'False finished task',
        });
        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await selectTask(page, seeded.taskId);
        await expect(detail(page).locator('.run-status-label')).toHaveText('Running');
        await expect(detail(page).locator('.run-status')).not.toContainText('Finished');
        await expect(detail(page).locator('.run-status .run-stop')).toBeEnabled();
        await expect(page.getByRole('button', { name: 'Retry', exact: true })).toHaveCount(0);
        await expect(page.locator(`#studio-typed-root [data-task-id="${seeded.taskId}"] .task-state`)).toHaveText('Running');
    });

    test('distinct_durable_title_and_unrecorded_prompt_are_not_conflated', async ({ page }) => {
        await openStudio(page);
        await call<Outcome>(page, 'open');
        const created = await call<{ taskId: string; runId: string }>(page, 'createClientRun', {
            title: 'Distinct task title',
            prompt: 'runtime-prompt-value',
        });
        const row = page.locator(`#studio-typed-root [data-task-id="${created.taskId}"]`);
        await expect(row).toBeVisible();
        await selectTask(page, created.taskId);
        const header = detail(page).locator('[data-task-detail-header]');
        await expect(header).toContainText('Title');
        await expect(header).toContainText('Distinct task title');
        await expect(header).toContainText('Starting prompt not recorded');
        await expect(header).not.toContainText('runtime-prompt-value');
        await expect(header).not.toContainText('Client resume fixture');
    });

    test('a_ready_legacy_source_enables_send_on_an_empty_runtime_task_list', async ({ page }) => {
        await openStudio(page);
        await page.locator('#studio-typed-root').getByRole('button', { name: /Command palette/ }).click();
        const search = page.locator('#studio-typed-root input[aria-label="Search commands"]');
        await search.fill('Load bundled demo');
        await search.press('Enter');
        await expect(page.locator('#studio-typed-root .session-status')).toContainText('Artifact: ready');
        await page.locator('#studio-typed-root textarea[aria-label="Prompt"]').fill('legacy ready prompt');
        await expect(page.getByRole('button', { name: 'Send', exact: true })).toBeEnabled();
    });

    test('local_session_select_and_new_clear_the_runtime_detail', async ({ page }) => {
        await openStudio(page);
        await call<Outcome>(page, 'open');
        const seeded = await call<{ taskId: string; runId: string }>(page, 'createTaskAndEvents', {
            mode: 'detail',
            title: 'Local switch task',
        });
        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await selectTask(page, seeded.taskId);
        await expect(detail(page).locator('[data-task-detail-header]')).toBeVisible();

        const localGroup = page.locator('#studio-typed-root details[data-task-group="local-sessions"]');
        const openLocalGroup = async () => {
            if (!(await localGroup.evaluate((element) => (element as HTMLDetailsElement).open))) {
                await localGroup.locator('summary').click();
            }
        };
        await openLocalGroup();
        await localGroup.locator('[data-action="new-session"]').click();
        await expect(detail(page).locator('[data-task-detail-header]')).toHaveCount(0);
        await expect(page.locator('#studio-typed-root h1')).toHaveText('Agent');
        await expect(page.locator('#studio-typed-root')).toContainText('No messages yet');
        const selectedAfterNew = await page.evaluate(() => (window as unknown as {
            __studioTest: { clientForTest(): { store: { snapshot(): { selectedTaskId: string | null } } } };
        }).__studioTest.clientForTest().store.snapshot().selectedTaskId);
        expect(selectedAfterNew).toBeNull();

        await selectTask(page, seeded.taskId);
        await expect(detail(page).locator('[data-task-detail-header]')).toBeVisible();
        await openLocalGroup();
        await localGroup.locator('[data-action="resume-session"]').first().click();
        await expect(detail(page).locator('[data-task-detail-header]')).toHaveCount(0);
        await expect(page.locator('#studio-typed-root h1')).toHaveText('Agent');
        await expect(page.locator('#studio-typed-root')).toContainText('No messages yet');
        const selectedAfterSelect = await page.evaluate(() => (window as unknown as {
            __studioTest: { clientForTest(): { store: { snapshot(): { selectedTaskId: string | null } } } };
        }).__studioTest.clientForTest().store.snapshot().selectedTaskId);
        expect(selectedAfterSelect).toBeNull();
        const after = (await call<Outcome>(page, 'taskSnapshotForTest', seeded.taskId))
            .record as LiveRunSnapshot;
        expect(after.runId).toBe(seeded.runId);
    });
});

test.describe('Final fix: restored run and approval reconciliation', () => {
    test('restored_run_stop_is_a_durable_safe_boundary_and_allows_a_new_run', async ({ page }) => {
        await openStudio(page);
        await call<Outcome>(page, 'open');
        const seeded = await call<{ taskId: string; runId: string }>(page, 'createTaskAndEvents', {
            mode: 'message',
            title: 'Restored stop task',
        });
        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await selectTask(page, seeded.taskId);
        await expect(detail(page).locator('.run-status-label')).toHaveText('Running');

        const stop = detail(page).locator('.run-status').getByRole('button', { name: 'Stop', exact: true });
        await expect(stop).toBeEnabled();
        await stop.click();
        await expect(detail(page).locator('.run-status-label')).toHaveText('Cancelled');
        await expect(detail(page).locator('.run-status')).not.toContainText('already terminal');
        await expect(detail(page).locator('.run-status').getByRole('button', { name: 'Stop', exact: true })).toHaveCount(0);
        const stopped = (await call<Outcome>(page, 'taskSnapshotForTest', seeded.taskId))
            .record as LiveRunSnapshot;
        expect(stopped.runId).toBe(seeded.runId);
        expect(stopped.runState).toBe('cancelled');

        const newRun = detail(page).getByRole('button', { name: 'New run', exact: true });
        await expect(newRun).toBeVisible();
        await newRun.click();
        await expect(detail(page).locator('[data-task-detail-header]')).toHaveCount(0);
        const prompt = page.locator('#studio-typed-root textarea[aria-label="Prompt"]');
        await prompt.fill('second run after a restored stop');
        await prompt.press('Enter');
        await expect(page.locator(`#studio-typed-root [data-task-id="${seeded.taskId}"] .task-state`)).toHaveText('Running');
        const next = (await call<Outcome>(page, 'taskSnapshotForTest', seeded.taskId))
            .record as LiveRunSnapshot;
        expect(next.runId).not.toBe(seeded.runId);
        expect(next.runState).toBe('running');
        const stopNext = page.locator('#studio-typed-root .composer-run-modes').getByRole('button', { name: 'Stop run', exact: true });
        await expect(stopNext).toBeEnabled();
        await stopNext.click();
        await expect(page.locator(`#studio-typed-root [data-task-id="${seeded.taskId}"] .task-state`)).toHaveText('Cancelled');
    });

    test('restored_approval_reports_no_active_resolver_after_reload', async ({ page }) => {
        await openStudio(page);
        await call<Outcome>(page, 'open');
        const seeded = await call<Outcome>(page, 'seedPendingApprovalForTest');
        expect(seeded.ok, JSON.stringify(seeded)).toBe(true);
        const taskId = (seeded.record as { taskId: string }).taskId;
        await page.reload();
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await selectTask(page, taskId);
        const card = detail(page).locator('[data-approval-id="appr_reload_approval"]');
        await expect(card.getByRole('button', { name: 'Approve', exact: true })).toBeDisabled();
        await expect(card.getByRole('button', { name: 'Deny', exact: true })).toBeDisabled();

        const pending = await call<Outcome>(page, 'pendingApprovalsForTest', taskId);
        expect(pending.ok, JSON.stringify(pending)).toBe(true);
        const record = pending.record as {
            pending: unknown[];
            resolvable: boolean[];
        };
        expect(record.pending).toHaveLength(1);
        expect(record.resolvable).toEqual([false]);

        const refused = await call<Outcome>(page, 'resolveRestoredApprovalForTest', 'appr_reload_approval');
        expect(refused.ok).toBe(false);
        expect(refused.code).toBe('approval_forbidden');
        // The static host has no approval authority, so the existing static
        // guidance stays; the runtime surface proves no active owner exists.
        await expect(card).toContainText('This action needs host approval. Approve it in the local companion.');
    });
});

async function seedFinishedCrossTabTask(
    page: Page,
): Promise<{ taskId: string; runId: string }> {
    await openStudio(page);
    await call<Outcome>(page, 'open');
    const seeded = await call<{ taskId: string; runId: string }>(page, 'createTaskAndEvents', {
        mode: 'finished',
        title: 'Cross-tab lease task',
    });
    await page.reload();
    await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
    await expect(
        page.locator(`#studio-typed-root [data-task-id="${seeded.taskId}"]`),
    ).toBeVisible();
    return seeded;
}

async function startSecondWriter(
    context: BrowserContext,
    taskId: string,
): Promise<{ tabA: Page; runId: string }> {
    const tabA = await context.newPage();
    await openStudio(tabA);
    await selectTask(tabA, taskId);
    await tabA.getByRole('button', { name: 'New run', exact: true }).click();
    const prompt = tabA.locator('#studio-typed-root textarea[aria-label="Prompt"]');
    await prompt.fill('tab A holds the lease');
    await prompt.press('Enter');
    const row = tabA.locator(`#studio-typed-root [data-task-id="${taskId}"]`);
    await expect(row.locator('.task-state')).toHaveText('Running');
    const snapshot = (
        await call<Outcome>(tabA, 'taskSnapshotForTest', taskId)
    ).record as LiveRunSnapshot;
    expect(snapshot.runState).toBe('running');
    expect(snapshot.runId).toMatch(/^run_[0-9a-f]{16}$/);
    return { tabA, runId: snapshot.runId! };
}

async function reopenFreshStore(
    context: BrowserContext,
): Promise<{ phase: string; error: unknown }> {
    const tabC = await context.newPage();
    try {
        await openStudio(tabC);
        return await call<{ phase: string; error: unknown }>(tabC, 'open');
    } finally {
        await tabC.close();
    }
}

test.describe('Residual: cross-tab lease and run generation safety', () => {
    test('stale_tab_cannot_displace_a_fresh_active_writer', async ({ context, page: tabB }) => {
        const seeded = await seedFinishedCrossTabTask(tabB);
        const { tabA, runId } = await startSecondWriter(context, seeded.taskId);
        try {
            await expect(
                tabB.locator(`#studio-typed-root [data-task-id="${seeded.taskId}"] .task-state`),
            ).toHaveText('Finished');
            await selectTask(tabB, seeded.taskId);
            const leaseBefore = (await call<Outcome>(tabB, 'lease')).record as {
                writer_id: string;
            };
            expect(leaseBefore.writer_id.length).toBeGreaterThan(0);

            await tabB.getByRole('button', { name: 'New run', exact: true }).click();
            await tabB.evaluate(() => {
                const target = window as unknown as {
                    __studioToastCalls: string[];
                    __studioUi: { toast(text: string, tone?: string): void };
                };
                target.__studioToastCalls = [];
                const original = target.__studioUi.toast;
                target.__studioUi.toast = (text: string, tone?: string) => {
                    target.__studioToastCalls.push(String(text));
                    original.call(target.__studioUi, text, tone);
                };
            });
            const prompt = tabB.locator('#studio-typed-root textarea[aria-label="Prompt"]');
            await prompt.fill('stale tab colliding run');
            await prompt.press('Enter');
            await tabB.waitForTimeout(500);
            const toasts = await tabB.evaluate(
                () => (window as unknown as { __studioToastCalls: string[] }).__studioToastCalls,
            );
            expect(toasts.some((text) => text.includes('stale_generation'))).toBe(true);
            await expect(
                tabB.locator(`#studio-typed-root [data-task-id="${seeded.taskId}"] .task-state`),
            ).toHaveText('Finished');

            const leaseAfter = (await call<Outcome>(tabB, 'lease')).record as {
                writer_id: string;
            };
            expect(leaseAfter.writer_id).toBe(leaseBefore.writer_id);

            await expect(
                tabA.locator(`#studio-typed-root [data-task-id="${seeded.taskId}"] .task-state`),
            ).toHaveText('Running');
            const stillActive = (
                await call<Outcome>(tabA, 'taskSnapshotForTest', seeded.taskId)
            ).record as LiveRunSnapshot;
            expect(stillActive.runId).toBe(runId);
            expect(stillActive.runState).toBe('running');

            const fresh = await reopenFreshStore(context);
            expect(fresh.phase).toBe('ready');
            expect(fresh.error).toBeNull();
        } finally {
            await tabA.close();
        }
    });

    test('commit_event_rejects_a_colliding_run_without_the_lease_path', async ({ context, page: tabB }) => {
        const seeded = await seedFinishedCrossTabTask(tabB);
        const { tabA, runId } = await startSecondWriter(context, seeded.taskId);
        try {
            const lease = (await call<Outcome>(tabB, 'lease')).record as {
                writer_id: string;
            };
            expect(lease.writer_id.length).toBeGreaterThan(0);

            const collided = await call<Outcome>(
                tabB,
                'appendCollidingRunQueuedForTest',
                seeded.taskId,
                lease.writer_id,
            );
            expect(collided.ok, JSON.stringify(collided)).toBe(false);
            expect(collided.code).toBe('stale_generation');

            const leaseAfter = (await call<Outcome>(tabB, 'lease')).record as {
                writer_id: string;
            };
            expect(leaseAfter.writer_id).toBe(lease.writer_id);
            await expect(
                tabA.locator(`#studio-typed-root [data-task-id="${seeded.taskId}"] .task-state`),
            ).toHaveText('Running');
            const stillActive = (
                await call<Outcome>(tabA, 'taskSnapshotForTest', seeded.taskId)
            ).record as LiveRunSnapshot;
            expect(stillActive.runId).toBe(runId);
            expect(stillActive.runState).toBe('running');

            const fresh = await reopenFreshStore(context);
            expect(fresh.phase).toBe('ready');
            expect(fresh.error).toBeNull();
        } finally {
            await tabA.close();
        }
    });
});
