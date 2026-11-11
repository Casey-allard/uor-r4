import { render } from 'preact';
import { act } from 'preact/test-utils';
import { useState } from 'preact/hooks';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { AgentClient, type TaskSummary } from '../agent/client.js';
import { FixtureTransport } from '../agent/fixtureTransport.js';
import { createAgentStore } from '../state/agent.js';
import { LifecycleStore } from '../state/lifecycle.js';
import { StudioApp } from './StudioApp.js';
import { TaskRail, type TaskRailProps } from './TaskRail.js';
import { makeFiles, makeModelAdapter, makeSessions, makeSources, makeUi } from './studioTestFixtures.js';

vi.mock('@xterm/xterm', () => ({
    Terminal: class {
        options = {};
        open = vi.fn();
        write = vi.fn();
        onData = vi.fn();
        onResize = vi.fn();
        loadAddon = vi.fn();
        focus = vi.fn();
        dispose = vi.fn();
        resize = vi.fn();
    },
}));
vi.mock('@xterm/addon-fit', () => ({ FitAddon: class { fit = vi.fn(); } }));

const PROJECT_A = 'prj_0000000000000001';
const PROJECT_B = 'prj_0000000000000002';
const NOW = new Date(2026, 10, 10, 12, 0, 0).getTime();

function startOfLocalDay(value: number): number {
    const date = new Date(value);
    date.setHours(0, 0, 0, 0);
    return date.getTime();
}

function atLocalDay(offsetDays: number, hour = 12): number {
    const date = new Date(NOW);
    date.setHours(hour, 0, 0, 0);
    date.setDate(date.getDate() + offsetDays);
    return date.getTime();
}

function task(overrides: Partial<TaskSummary> & Pick<TaskSummary, 'taskId' | 'title' | 'createdAtMs'>): TaskSummary {
    return {
        projectId: PROJECT_A,
        repo: 'uor-r4',
        branch: 'main',
        archived: false,
        pinned: false,
        latestRunId: null,
        state: 'idle',
        ...overrides,
    };
}

let host: HTMLDivElement;
beforeEach(() => {
    host = document.createElement('div');
    document.body.appendChild(host);
    localStorage.clear();
});
afterEach(() => {
    render(null, host);
    host.remove();
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
});

function RailHarness(props: TaskRailProps) {
    const [query, setQuery] = useState(props.query);
    return <TaskRail
        {...props}
        query={query}
        onQuery={(value) => { setQuery(value); props.onQuery(value); }}
    />;
}

function renderRail(overrides: Partial<TaskRailProps> = {}): TaskRailProps {
    const props: TaskRailProps = {
        tasks: [],
        query: '',
        onQuery: vi.fn(),
        onSelect: vi.fn(),
        onPin: vi.fn(),
        onArchive: vi.fn(),
        selectedTaskId: null,
        nowMs: NOW,
        ...overrides,
    };
    act(() => { render(<RailHarness {...props} />, host); });
    return props;
}

function rows(group?: string): HTMLElement[] {
    const scope = group === undefined ? host : host.querySelector<HTMLElement>(`[data-task-group="${group}"]`)!;
    return Array.from(scope.querySelectorAll<HTMLElement>('[data-task-id]'));
}

function rowIds(group?: string): string[] {
    return rows(group).map((row) => row.getAttribute('data-task-id')!);
}

function setSearch(value: string): void {
    const input = host.querySelector<HTMLInputElement>('input[aria-label="Search tasks"]')!;
    act(() => {
        input.value = value;
        input.dispatchEvent(new Event('input', { bubbles: true }));
    });
}

describe('TaskRail groups', () => {
    it('labels an unresolved latest run as Restoring and searches by that label', () => {
        renderRail({
            tasks: [
                task({
                    taskId: 'tsk_00000000000000a1',
                    title: 'Resume task',
                    createdAtMs: NOW,
                    latestRunId: 'run_00000000000000a1',
                    unresolvedRunId: 'run_00000000000000a1',
                    state: 'idle',
                }),
            ],
        });
        const row = host.querySelector<HTMLElement>('[data-task-id="tsk_00000000000000a1"]')!;
        expect(row.textContent).toContain('Restoring');
        expect(row.textContent).not.toContain('Idle');
        const input = host.querySelector<HTMLInputElement>('input[aria-label="Search tasks"]')!;
        act(() => {
            input.value = 'restoring';
            input.dispatchEvent(new Event('input', { bubbles: true }));
        });
        expect(host.querySelector('[data-task-id="tsk_00000000000000a1"]')).not.toBeNull();
    });

    it('keeps Restoring after a null-latest record follows an unresolved latest run', () => {
        const store = createAgentStore();
        const record = {
            taskId: 'tsk_00000000000000a2',
            projectId: PROJECT_A,
            title: 'Reloaded task',
            createdAtMs: NOW,
            archived: false,
            pinnedLocal: false,
        };
        store.upsertTask({ ...record, latestRunId: 'run_00000000000000a2' });
        store.upsertTask({ ...record, latestRunId: null });
        const view = store.snapshot().tasks[0]!;
        renderRail({
            tasks: [{
                taskId: view.taskId,
                projectId: view.projectId,
                title: view.title,
                repo: view.repo,
                branch: view.branch,
                createdAtMs: view.createdAtMs,
                archived: view.archived,
                pinned: view.pinnedLocal,
                latestRunId: view.latestRunId,
                unresolvedRunId: view.unresolvedRunId,
                state: view.state,
            }],
        });
        const row = host.querySelector<HTMLElement>('[data-task-id="tsk_00000000000000a2"]')!;
        expect(row.textContent).toContain('Restoring');
        expect(row.textContent).not.toContain('Idle');
    });

    it('treats an explicitly unloaded latest run as Restoring without a marker', () => {
        const summary = {
            ...task({
                taskId: 'tsk_00000000000000a3',
                title: 'Unloaded rail task',
                createdAtMs: NOW,
                latestRunId: 'run_00000000000000a3',
                state: 'idle',
            }),
            latestRunLoaded: false,
        } as TaskSummary;
        renderRail({ tasks: [summary] });
        const row = host.querySelector<HTMLElement>('[data-task-id="tsk_00000000000000a3"]')!;
        expect(row.textContent).toContain('Restoring');
        expect(row.textContent).not.toContain('Idle');
    });

    it('renders_today_last_seven_days_older_groups from task record timestamps', () => {
        const todayStart = startOfLocalDay(NOW);
        const sevenDaysStart = atLocalDay(-7, 0);
        const tasks = [
            task({ taskId: 'tsk_0000000000000001', title: 'Today edge', createdAtMs: todayStart }),
            task({ taskId: 'tsk_0000000000000002', title: 'Yesterday edge', createdAtMs: todayStart - 1 }),
            task({ taskId: 'tsk_0000000000000003', title: 'Seven days edge', createdAtMs: sevenDaysStart }),
            task({ taskId: 'tsk_0000000000000004', title: 'Older edge', createdAtMs: sevenDaysStart - 1 }),
        ];
        renderRail({ tasks });

        expect(host.querySelector('nav[aria-label="Tasks and local sessions"]')).not.toBeNull();
        expect(rowIds('today')).toEqual(['tsk_0000000000000001']);
        expect(rowIds('last-7-days')).toEqual(['tsk_0000000000000002', 'tsk_0000000000000003']);
        const older = host.querySelector<HTMLDetailsElement>('details[data-task-group="older"]')!;
        expect(older).not.toBeNull();
        expect(older.open).toBe(false);
        expect(older.querySelector('summary')?.textContent).toContain('Older');
        expect(older.querySelector('summary')?.textContent).toContain('1');
        act(() => { older.open = true; });
        expect(rowIds('older')).toEqual(['tsk_0000000000000004']);

        const labels = Array.from(host.querySelectorAll('[data-group-label]')).map((node) => node.textContent);
        expect(labels).toEqual(['Today', 'Last 7 days', 'Older']);
        expect(host.textContent).not.toContain('This Week');
        expect(host.textContent).not.toContain('This Month');
    });

    it('groups pinned tasks in their real day group and sorts them first', () => {
        const tasks = [
            task({ taskId: 'tsk_0000000000000010', title: 'Unpinned today', createdAtMs: NOW - 1_000 }),
            task({ taskId: 'tsk_0000000000000011', title: 'Pinned today', createdAtMs: NOW - 2_000, pinned: true }),
        ];
        renderRail({ tasks });
        expect(rowIds('today')).toEqual(['tsk_0000000000000011', 'tsk_0000000000000010']);
        expect(host.querySelector('li[data-task-id="tsk_0000000000000011"]')?.textContent).toContain('This device');
    });

    it('uses singular grammar and heading semantics in collapsed group labels', () => {
        const sevenDaysStart = atLocalDay(-7, 0);
        const tasks = [task({ taskId: 'tsk_0000000000000012', title: 'Old one', createdAtMs: sevenDaysStart - 1 })];
        renderRail({ tasks });
        const older = host.querySelector<HTMLDetailsElement>('details[data-task-group="older"]')!;
        expect(older).not.toBeNull();
        expect(older.querySelector('summary')?.textContent).toContain('1 task');
        expect(older.querySelector('summary')?.textContent).not.toContain('1 tasks');
        expect(older.querySelector('summary [role="heading"]')?.textContent).toBe('Older');
        expect(older.querySelector('summary')?.getAttribute('aria-label')).toBeNull();
    });

    it('derives the visible task state from the loaded run projection', async () => {
        const transport = new FixtureTransport({ nowMs: Date.now() - 60_000, projectId: PROJECT_A });
        const client = new AgentClient({ transport });
        const created = await client.createTask({ projectId: PROJECT_A, title: 'Running task', repo: 'uor-r4' });
        await client.startRun({ taskId: created.taskId, prompt: 'Run it' });

        act(() => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} agentClient={client} />, host);
        });
        await vi.waitFor(() => expect(host.querySelector(`nav[aria-label="Tasks and local sessions"] [data-task-id="${created.taskId}"]`)).not.toBeNull());
        expect(host.querySelector(`nav[aria-label="Tasks and local sessions"] [data-task-id="${created.taskId}"]`)?.textContent).toContain('Running');
        expect(client.store.snapshot().tasks.find((candidate) => candidate.taskId === created.taskId)?.state).toBe('running');
        const home = host.querySelector('section[aria-label="Project home"]');
        expect(home).not.toBeNull();
        expect(home!.textContent).toContain('uor-r4');
        expect(home!.textContent).not.toContain('No tasks yet');
        expect(host.textContent).not.toContain('No tasks yet');
    });

    it('selects a task through AgentStore and marks the current row', async () => {
        const transport = new FixtureTransport({ nowMs: Date.now() - 60_000, projectId: PROJECT_A });
        const client = new AgentClient({ transport });
        const created = await client.createTask({ projectId: PROJECT_A, title: 'Selectable task', repo: 'uor-r4' });

        act(() => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} agentClient={client} />, host);
        });
        await vi.waitFor(() => expect(host.querySelector(`nav[aria-label="Tasks and local sessions"] [data-task-id="${created.taskId}"]`)).not.toBeNull());

        const open = host.querySelector<HTMLButtonElement>('nav[aria-label="Tasks and local sessions"] button[aria-label="Open Selectable task"]')!;
        act(() => { open.click(); });
        await vi.waitFor(() => expect(client.store.snapshot().selectedTaskId).toBe(created.taskId));
        await vi.waitFor(() => expect(host.querySelector(`nav[aria-label="Tasks and local sessions"] [data-task-id="${created.taskId}"]`)?.getAttribute('aria-current')).toBe('true'));
    });
});

describe('TaskRail search', () => {
    it('search_filters_tasks_by_title_and_project', () => {
        const tasks = [
            task({ taskId: 'tsk_0000000000000020', title: 'Fix parser', createdAtMs: NOW - 1_000, projectId: PROJECT_A, repo: 'uor-r4' }),
            task({ taskId: 'tsk_0000000000000021', title: 'Draft notes', createdAtMs: NOW - 2_000, projectId: PROJECT_B, repo: 'spiralcore' }),
            task({ taskId: 'tsk_0000000000000022', title: 'Parser docs', createdAtMs: NOW - 3_000, projectId: PROJECT_B, repo: 'spiralcore' }),
        ];
        const props = renderRail({ tasks });

        setSearch('spiralcore');
        expect(props.onQuery).toHaveBeenLastCalledWith('spiralcore');
        expect(rowIds()).toEqual(['tsk_0000000000000021', 'tsk_0000000000000022']);

        setSearch('fix parser');
        expect(rowIds()).toEqual(['tsk_0000000000000020']);

        setSearch(PROJECT_A);
        expect(rowIds()).toEqual(['tsk_0000000000000020']);

        setSearch('no such task');
        expect(rowIds()).toEqual([]);
        expect(host.querySelector('[role="status"]')?.textContent).toContain('No tasks match');
        expect(host.querySelector('[role="status"]')?.textContent).toContain('no such task');
    });

    it('search input has an accessible label and reports the real result count', () => {
        const tasks = [
            task({ taskId: 'tsk_0000000000000030', title: 'Alpha', createdAtMs: NOW - 1_000 }),
            task({ taskId: 'tsk_0000000000000031', title: 'Beta', createdAtMs: NOW - 2_000 }),
        ];
        renderRail({ tasks });
        const input = host.querySelector<HTMLInputElement>('input[aria-label="Search tasks"]')!;
        expect(input.getAttribute('aria-label')).toBe('Search tasks');
        expect(input.labels?.[0]?.textContent).toContain('Search tasks');
        setSearch('alpha');
        expect(host.textContent).toContain('1 of 2 tasks');
    });
});

describe('TaskRail pins', () => {
    it('pins_are_local_device_only_without_service', () => {
        const fetchMock = vi.fn();
        vi.stubGlobal('fetch', fetchMock);
        const tasks = [task({ taskId: 'tsk_0000000000000040', title: 'Local pin', createdAtMs: NOW - 1_000, pinned: true })];
        renderRail({ tasks });

        const pin = host.querySelector<HTMLButtonElement>('button[aria-label="Unpin Local pin on this device"]');
        expect(pin).not.toBeNull();
        expect(pin?.getAttribute('aria-pressed')).toBe('true');
        expect(pin?.textContent).toBe('This device');
        expect(host.textContent).toContain('Pins stay on this device.');
        expect(host.textContent).not.toMatch(/sync|account|service|hosted/i);
        expect(fetchMock).not.toHaveBeenCalled();
        expect(localStorage.length).toBe(0);
    });

    it('pin_task_calls_agent_client and updates the store projection', async () => {
        const transport = new FixtureTransport({ nowMs: Date.now() - 60_000, projectId: PROJECT_A });
        const client = new AgentClient({ transport });
        const created = await client.createTask({ projectId: PROJECT_A, title: 'Pinned task', repo: 'uor-r4' });

        act(() => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} agentClient={client} />, host);
        });
        await vi.waitFor(() => expect(host.querySelector(`nav[aria-label="Tasks and local sessions"] [data-task-id="${created.taskId}"]`)).not.toBeNull());

        const pin = host.querySelector<HTMLButtonElement>(`nav[aria-label="Tasks and local sessions"] button[aria-label="Pin Pinned task on this device"]`)!;
        act(() => { pin.click(); });
        await vi.waitFor(() => expect(transport.calls.some((call) => call.method === 'pinTask' && JSON.stringify(call.payload) === JSON.stringify({ taskId: created.taskId, pinned: true }))).toBe(true));
        await vi.waitFor(() => expect(host.textContent).toContain('This device'));

        expect(client.store.snapshot().tasks.find((candidate) => candidate.taskId === created.taskId)?.pinnedLocal).toBe(true);
        expect(transport.calls.filter((call) => call.method === 'pinTask')).toHaveLength(1);
    });
});

describe('TaskRail archive', () => {
    it('archive_task_calls_agent_client_and_reopens to restore', async () => {
        const transport = new FixtureTransport({ nowMs: Date.now() - 60_000, projectId: PROJECT_A });
        const client = new AgentClient({ transport });
        const created = await client.createTask({ projectId: PROJECT_A, title: 'Archived task', repo: 'uor-r4' });

        act(() => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} agentClient={client} />, host);
        });
        await vi.waitFor(() => expect(host.querySelector(`nav[aria-label="Tasks and local sessions"] [data-task-id="${created.taskId}"]`)).not.toBeNull());

        const archive = host.querySelector<HTMLButtonElement>(`nav[aria-label="Tasks and local sessions"] button[aria-label="Archive Archived task"]`)!;
        act(() => { archive.click(); });
        await vi.waitFor(() => expect(transport.calls.some((call) => call.method === 'archiveTask' && JSON.stringify(call.payload) === JSON.stringify({ taskId: created.taskId, archived: true }))).toBe(true));
        await vi.waitFor(() => expect(host.querySelector(`nav[aria-label="Tasks and local sessions"] [data-task-group="today"] [data-task-id="${created.taskId}"]`)).toBeNull());
        expect(client.store.snapshot().tasks.find((candidate) => candidate.taskId === created.taskId)?.archived).toBe(true);

        const archived = host.querySelector<HTMLDetailsElement>('nav[aria-label="Tasks and local sessions"] details[data-task-group="archived"]')!;
        expect(archived).not.toBeNull();
        expect(archived.querySelector('summary')?.textContent).toContain('Archived');
        expect(archived.querySelector('summary')?.textContent).toContain('1');
        expect(archived.querySelector(`[data-task-id="${created.taskId}"]`)).not.toBeNull();
        expect(archived.open).toBe(false);
        act(() => { archived.open = true; });

        const restore = archived.querySelector<HTMLButtonElement>('button[aria-label="Restore Archived task"]')!;
        expect(restore).not.toBeNull();
        act(() => { restore.click(); });
        await vi.waitFor(() => expect(transport.calls.some((call) => call.method === 'archiveTask' && JSON.stringify(call.payload) === JSON.stringify({ taskId: created.taskId, archived: false }))).toBe(true));
        await vi.waitFor(() => expect(host.querySelector(`nav[aria-label="Tasks and local sessions"] [data-task-group="today"] [data-task-id="${created.taskId}"]`)).not.toBeNull());
        expect(host.querySelector('nav[aria-label="Tasks and local sessions"] details[data-task-group="archived"]')).toBeNull();
        expect(client.store.snapshot().tasks.find((candidate) => candidate.taskId === created.taskId)?.archived).toBe(false);
    });
});

describe('TaskRail legacy session presentation', () => {
    it('nests the legacy session list inside the single task nav when no client is mounted', () => {
        act(() => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        const navs = host.querySelectorAll('nav[aria-label="Tasks and local sessions"]');
        expect(navs).toHaveLength(1);
        expect(navs[0].querySelector('.session-list')).not.toBeNull();
        expect(host.querySelectorAll('.session-list')).toHaveLength(1);
        expect(host.querySelector('.session-list')?.closest('nav[aria-label="Tasks and local sessions"]')).toBe(navs[0]);
        expect(navs[0].querySelector('.session-list [data-action="new-session"]')).not.toBeNull();
    });

    it('keeps the labelled Local sessions group collapsed when an AgentClient has runtime tasks', async () => {
        const transport = new FixtureTransport({ nowMs: Date.now() - 60_000, projectId: PROJECT_A });
        const client = new AgentClient({ transport });
        const created = await client.createTask({ projectId: PROJECT_A, title: 'Task with client', repo: 'uor-r4' });

        act(() => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} agentClient={client} />, host);
        });
        await vi.waitFor(() => expect(host.querySelector(`nav[aria-label="Tasks and local sessions"] [data-task-id="${created.taskId}"]`)).not.toBeNull());
        const navs = host.querySelectorAll('nav[aria-label="Tasks and local sessions"]');
        expect(navs).toHaveLength(1);
        const heading = navs[0].querySelector<HTMLElement>('h3#task-rail-local-sessions');
        expect(heading).not.toBeNull();
        const group = heading!.closest('details[data-task-group="local-sessions"]') as HTMLDetailsElement | null;
        expect(group).not.toBeNull();
        expect(group!.open).toBe(false);
        expect(navs[0].querySelector('.session-list')).not.toBeNull();
    });
});

describe('TaskRail integration rulings', () => {
    it('renders one Tasks and local sessions nav with a Local sessions group', () => {
        act(() => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        const navs = host.querySelectorAll('nav[aria-label="Tasks and local sessions"]');
        expect(navs).toHaveLength(1);
        const nav = navs[0];
        expect(nav.querySelector('h2')?.textContent).toBe('Tasks');
        expect(nav.querySelector('h3#task-rail-local-sessions')?.textContent).toBe('Local sessions');
        const sessionList = nav.querySelector('.session-list');
        expect(sessionList).not.toBeNull();
        expect(sessionList?.getAttribute('role')).toBe('group');
        expect(sessionList?.getAttribute('aria-labelledby')).toBe('task-rail-local-sessions');
        expect(Array.from(nav.querySelectorAll('h2')).map((node) => node.textContent)).not.toContain('Sessions');
        expect(host.querySelectorAll('nav[aria-label="Tasks"]')).toHaveLength(0);
        expect(host.querySelectorAll('.session-list')).toHaveLength(1);
        expect(sessionList?.closest('nav[aria-label="Tasks and local sessions"]')).toBe(nav);
        expect(nav.querySelector('.task-legacy-sessions')?.textContent).toContain('No sessions yet');
    });

    it('shows the no-tasks message once and leaves it to ProjectHome', () => {
        act(() => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} />, host);
        });
        const nav = host.querySelector('nav[aria-label="Tasks and local sessions"]');
        expect(nav).not.toBeNull();
        expect(nav!.textContent).not.toContain('No tasks yet');
        expect(host.textContent!.match(/No tasks yet/g) ?? []).toHaveLength(1);
        const home = host.querySelector('section[aria-label="Project home"]');
        expect(home).not.toBeNull();
        expect(home!.textContent).toContain('uor-r4');
        expect(home!.textContent).toContain('Local browser');
        expect(home!.textContent).toContain('No tasks yet');
    });

    it('updates the visible task projection when a transport event arrives after mount', async () => {
        const transport = new FixtureTransport({ nowMs: Date.now() - 60_000, projectId: PROJECT_A });
        const client = new AgentClient({ transport });
        const created = await client.createTask({ projectId: PROJECT_A, title: 'Live task', repo: 'uor-r4' });
        const subscribeSpy = vi.spyOn(client, 'subscribe');

        act(() => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} agentClient={client} />, host);
        });
        await vi.waitFor(() => expect(host.querySelector(`[data-task-id="${created.taskId}"]`)?.textContent).toContain('Idle'));

        await client.startRun({ taskId: created.taskId, prompt: 'Run after mount' });
        await vi.waitFor(() => expect(host.querySelector(`[data-task-id="${created.taskId}"]`)?.textContent).toContain('Running'));
        expect(subscribeSpy).toHaveBeenCalledTimes(1);
        expect(client.store.snapshot().tasks.find((candidate) => candidate.taskId === created.taskId)?.state).toBe('running');

        const stream = subscribeSpy.mock.results[0]?.value as { close(): void } | undefined;
        const closeSpy = stream === undefined ? null : vi.spyOn(stream, 'close');
        act(() => { render(null, host); });
        expect(closeSpy).toHaveBeenCalledTimes(1);
    });

    it('clears a failed pin after a successful retry without hiding ProjectHome', async () => {
        const transport = new FixtureTransport({ nowMs: Date.now() - 60_000, projectId: PROJECT_A });
        const client = new AgentClient({ transport });
        const created = await client.createTask({ projectId: PROJECT_A, title: 'Retry task', repo: 'uor-r4' });
        vi.spyOn(transport, 'pinTask').mockRejectedValueOnce(new Error('pin failed once'));

        act(() => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} agentClient={client} />, host);
        });
        await vi.waitFor(() => expect(host.querySelector(`[data-task-id="${created.taskId}"]`)?.textContent).toContain('Idle'));

        const pin = host.querySelector<HTMLButtonElement>('button[aria-label="Pin Retry task on this device"]')!;
        act(() => { pin.click(); });
        await vi.waitFor(() => expect(host.querySelector('[role="alert"]')?.textContent).toContain('pin failed once'));
        expect(host.querySelector('section[aria-label="Project home"]')).not.toBeNull();
        expect(pin.getAttribute('aria-pressed')).toBe('false');

        act(() => { pin.click(); });
        await vi.waitFor(() => expect(host.querySelector('[role="alert"]')).toBeNull());
        await vi.waitFor(() => expect(host.querySelector(`[data-task-id="${created.taskId}"]`)?.textContent).toContain('This device'));
        expect(client.store.snapshot().tasks.find((candidate) => candidate.taskId === created.taskId)?.pinnedLocal).toBe(true);
        expect(host.querySelector('section[aria-label="Project home"]')).not.toBeNull();
    });
});

describe('TaskRail empty and failure states', () => {
    it('empty_state_explains_runtime_unavailable only when initialization failed', async () => {
        renderRail();
        expect(host.querySelector('nav[aria-label="Tasks and local sessions"]')?.textContent).toContain('No tasks yet');
        expect(host.textContent).not.toMatch(/runtime_unavailable|runtime is not initialized/i);

        render(null, host);
        renderRail({ runtimeUnavailable: true });
        const alert = host.querySelector('[role="alert"]');
        expect(alert?.textContent).toContain('task runtime is not initialized');
        expect(host.textContent).not.toContain('No tasks yet');

        render(null, host);
        renderRail({ error: 'host_unavailable: the host companion is not connected' });
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('host_unavailable');
        expect(host.textContent).not.toContain('No tasks yet');

        render(null, host);
        act(() => {
            render(<StudioApp store={new LifecycleStore()} adapter={makeModelAdapter()} sources={makeSources()} files={makeFiles()} sessions={makeSessions()} ui={makeUi()} download={vi.fn()} agentClient={new AgentClient()} />, host);
        });
        await vi.waitFor(() => expect(host.querySelector('nav[aria-label="Tasks and local sessions"] [role="alert"]')?.textContent).toContain('task runtime is not initialized'));
        expect(host.querySelector('nav[aria-label="Tasks and local sessions"]')?.textContent).not.toContain('No tasks yet');
    });

    it('shows no fabricated task rows or counts for an empty list', () => {
        renderRail();
        expect(host.querySelectorAll('[data-task-id]')).toHaveLength(0);
        expect(host.querySelector('details[data-task-group="archived"]')).toBeNull();
        expect(host.querySelector('nav[aria-label="Tasks and local sessions"]')?.textContent).not.toMatch(/\b[1-9]\d* tasks?\b/);
    });
});

describe('TaskRail task creation', () => {
    it('offers_a_native_new_task_action_and_labels_synthetic_replay_output', () => {
        const onCreate = vi.fn();
        renderRail({ onCreate });
        const action = host.querySelector<HTMLButtonElement>('[data-action="new-task"]');
        expect(action).not.toBeNull();
        expect(action!.type).toBe('button');
        expect(action!.textContent).toBe('New task');
        expect(host.querySelector('nav[aria-label="Tasks and local sessions"]')?.textContent).toContain(
            'deterministic synthetic replay',
        );
        act(() => { action!.click(); });
        expect(onCreate).toHaveBeenCalledTimes(1);
    });

    it('disables_creation_while_the_owner_is_creating_or_unavailable', () => {
        renderRail({ onCreate: vi.fn(), createPending: true });
        expect(host.querySelector<HTMLButtonElement>('[data-action="new-task"]')!.disabled).toBe(true);

        render(null, host);
        renderRail({ runtimeUnavailable: true });
        expect(host.querySelector('[data-action="new-task"]')).toBeNull();
    });
});
