import { expect, test, type Page, type Request } from '@playwright/test';

const TERMINAL_COPY =
    'Interactive terminal requires the local companion. This browser build cannot start a shell.';
const GIT_COPY =
    'Git requires the local companion. Uploaded files can still be reviewed locally.';
const ACCOUNT_COPY =
    'Accounts and team settings require an optional service. Local settings are stored on this device.';
const BACKGROUND_COPY =
    'Background and cross-device tasks require an optional service. Closing this tab stops local work.';

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

async function openStatic(page: Page, path = '/?typed=1'): Promise<void> {
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

function studioDestination(page: Page, name: 'Changes' | 'Terminal') {
    return page
        .locator('#studio-typed-root nav[aria-label="Studio"]')
        .getByRole('button', { name, exact: true });
}

test('static_browser_disables_terminal_with_exact_copy', async ({ page }) => {
    await openStatic(page);
    const root = page.locator('#studio-typed-root');
    await studioDestination(page, 'Terminal').click();

    const status = root.locator('#terminalConsentState');
    await expect(status).toHaveText(TERMINAL_COPY);
    const open = root.getByRole('button', { name: 'Open terminal', exact: true });
    await expect(open).toBeDisabled();
    await open.click({ force: true });

    await expect(status).toHaveText(TERMINAL_COPY);
    await expect(root.locator('.terminal-screen[data-session-id]')).toHaveCount(0);
    await expect(root.getByRole('button', { name: 'Close shell', exact: true })).toHaveCount(0);
    expect(await page.evaluate(() => typeof (window as unknown as { __TAURI__?: unknown }).__TAURI__)).toBe('undefined');
});

test('static_browser_disables_git_with_exact_copy', async ({ page }) => {
    await openStatic(page);
    const root = page.locator('#studio-typed-root');
    await studioDestination(page, 'Changes').click();

    const summary = root.locator('#gitStatusSummary');
    await expect(summary).toHaveText(GIT_COPY);
    await expect(root.getByRole('button', { name: 'Refresh changes', exact: true })).toBeDisabled();
    await expect(root.locator('#gitDiffOutput')).toHaveText('');
    await expect(summary).not.toContainText('connected');
    await expect(root.getByRole('button', { name: 'Copy diff', exact: true })).toBeDisabled();
});

test('static_browser_disables_account_and_background', async ({ page }) => {
    await openStatic(page, '/?typed=1&studioTest=1');
    const root = page.locator('#studio-typed-root');
    expect(await page.evaluate(() => typeof (window as unknown as { __TAURI__?: unknown }).__TAURI__)).toBe('undefined');

    await root.locator('header.session-toolbar button[aria-label="Settings"]').click();
    const settings = root.locator('dialog');
    await expect(settings).toBeVisible();
    await expect(settings).toContainText(ACCOUNT_COPY);
    await expect(settings.getByText('Local settings')).toBeVisible();
    await settings.getByRole('button', { name: 'Close', exact: true }).click();

    const opened = await call<{ phase: string }>(page, 'open');
    expect(opened.phase).toBe('ready');
    const seeded = await call<Outcome>(page, 'seedBackgroundHoldForTest');
    expect(seeded.ok, JSON.stringify(seeded)).toBe(true);

    await page.reload();
    await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
    expect((await call<{ phase: string }>(page, 'open')).phase).toBe('ready');
    await root.getByRole('button', { name: 'Open Fixture task', exact: true }).click();

    const detail = root.locator('[data-task-detail-header]');
    await expect(detail).toBeVisible();
    await expect(root.locator('[data-static-background]')).toHaveText(BACKGROUND_COPY);
    await expect(root.getByRole('button', { name: /background/i })).toHaveCount(0);
});

test('static_browser_makes_zero_external_requests_on_load', async ({ page }) => {
    const requests: Request[] = [];
    page.on('request', (request) => requests.push(request));
    const sockets: string[] = [];
    page.on('websocket', (socket) => sockets.push(socket.url()));

    await openStatic(page);
    await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-host', 'static');
    await expect(page.locator('#studio-typed-root nav[aria-label="Studio"]')).toBeVisible();
    await page.waitForTimeout(500);

    const origin = new URL(page.url()).origin;
    const documentRequests = requests.filter((request) => request.resourceType() === 'document');
    expect(documentRequests).toHaveLength(1);
    const external = requests
        .map((request) => request.url())
        .filter((url) => !url.startsWith(`${origin}/`) && !url.startsWith('data:') && !url.startsWith('blob:'));
    expect(external, `external requests at load: ${external.join(', ')}`).toEqual([]);
    const hostOrServiceRequests = requests
        .map((request) => request.url())
        .filter((url) => /\/v1\/studio\/host|__tauri|agent_host|anthropic|openai|cdn\.|fonts\.googleapis/i.test(url));
    expect(hostOrServiceRequests).toEqual([]);
    expect(sockets).toEqual([]);
    expect(await page.evaluate(() => typeof (window as unknown as { __TAURI__?: unknown }).__TAURI__)).toBe('undefined');
    expect(await page.evaluate(() => typeof (window as unknown as { __studioTest?: unknown }).__studioTest)).toBe('undefined');
});
