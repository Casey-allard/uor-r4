import { expect, test, type Page, type Request } from '@playwright/test';
import { createServer } from 'node:http';
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { extname, join, resolve, sep } from 'node:path';
import { openTyped, openTypedAt } from './support';

const PROTECTED_SELECTORS = [
    '.task-workspace',
    'nav[aria-label="Studio"]',
    'textarea[aria-label="Prompt"]',
    '.agent-workspace',
    '.agent-transcript',
    '.agent-composer',
    '.workspace-grid',
    '.pane-dock',
    '.session-list',
    '[data-control="environment"]',
    '[data-pane-target]',
    '[data-action="toggle-projects"]',
    '[data-region="task-rail"]',
    '[data-region="task-detail"]',
];

const TERMINAL_COPY =
    'Interactive terminal requires the local companion. This browser build cannot start a shell.';
const GIT_COPY =
    'Git requires the local companion. Uploaded files can still be reviewed locally.';
const ACCOUNT_COPY =
    'Accounts and team settings require an optional service. Local settings are stored on this device.';
const BACKGROUND_COPY =
    'Background and cross-device tasks require an optional service. Closing this tab stops local work.';

const REPO_ROOT = resolve(process.cwd(), '../..');
const DEFAULT_BUILD_DIR = resolve(REPO_ROOT, 'local/kimi/cursor-issue-sync/studio-default-dist');

interface FocusDescriptor {
    tag: string;
    label: string;
    text: string;
    id: string;
    className: string;
}

interface StudioTestCall {
    method: string;
    args: unknown[];
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

async function focusDescriptor(page: Page): Promise<FocusDescriptor> {
    return page.evaluate(() => {
        const element = document.activeElement as HTMLElement | null;
        if (element === null) return { tag: '', label: '', text: '', id: '', className: '' };
        return {
            tag: element.tagName,
            label: (element.getAttribute('aria-label') ?? '').trim(),
            text: (element.textContent ?? '').replace(/\s+/g, ' ').trim().slice(0, 140),
            id: element.id,
            className: typeof element.className === 'string' ? element.className : '',
        };
    });
}

async function tabTo(
    page: Page,
    description: string,
    match: (focus: FocusDescriptor) => boolean,
    maxTabs = 500,
): Promise<FocusDescriptor> {
    const visited: string[] = [];
    for (let index = 0; index < maxTabs; index += 1) {
        await page.keyboard.press('Tab');
        const focus = await focusDescriptor(page);
        visited.push(`${focus.tag}:${focus.label || focus.text || focus.id}`);
        if (match(focus)) return focus;
    }
    throw new Error(`Tab did not reach ${description}. Last focus: ${visited.slice(-8).join(' | ')}`);
}

async function activeSection(page: Page): Promise<string> {
    const current = page.locator('#studio-typed-root nav[aria-label="Studio"] button[aria-current="page"]');
    await expect(current).toHaveCount(1);
    return ((await current.textContent()) ?? '').trim();
}

async function openStudio(page: Page, path = '/?typed=1'): Promise<void> {
    await page.goto(path);
    await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
}

async function openRail(page: Page): Promise<void> {
    const rail = page.locator('#studio-typed-root [data-region="task-rail"]');
    if (await rail.isHidden()) {
        await page.locator('#studio-typed-root [data-action="toggle-projects"]').click();
    }
    await expect(rail).toBeVisible();
}

function walkFiles(root: string): string[] {
    const files: string[] = [];
    for (const entry of readdirSync(root)) {
        const path = join(root, entry);
        if (statSync(path).isDirectory()) files.push(...walkFiles(path));
        else files.push(path);
    }
    return files;
}

function contentType(path: string): string {
    switch (extname(path)) {
        case '.html': return 'text/html; charset=utf-8';
        case '.js': return 'text/javascript; charset=utf-8';
        case '.css': return 'text/css; charset=utf-8';
        case '.json': return 'application/json; charset=utf-8';
        case '.wasm': return 'application/wasm';
        case '.svg': return 'image/svg+xml';
        case '.lut': return 'application/octet-stream';
        default: return 'application/octet-stream';
    }
}

// The default build is archived before this suite runs because the Playwright
// webServer rebuilds dist with VITE_STUDIO_TEST. The server below is created
// and closed by this Playwright test; no external process or pattern kill.
async function serveDirectory(root: string): Promise<{ url: string; close(): Promise<void> }> {
    const server = createServer((request, response) => {
        const url = new URL(request.url ?? '/', 'http://127.0.0.1');
        let pathname = decodeURIComponent(url.pathname);
        if (pathname.endsWith('/')) pathname += 'index.html';
        const candidate = resolve(root, `.${pathname}`);
        if (!candidate.startsWith(`${root}${sep}`) || !existsSync(candidate) || !statSync(candidate).isFile()) {
            response.statusCode = 404;
            response.end('not found');
            return;
        }
        response.setHeader('content-type', contentType(candidate));
        response.end(readFileSync(candidate));
    });
    await new Promise<void>((resolveListen, reject) => {
        server.once('error', reject);
        server.listen(0, '127.0.0.1', () => resolveListen());
    });
    const address = server.address();
    if (address === null || typeof address === 'string') throw new Error('static server has no address');
    return {
        url: `http://127.0.0.1:${address.port}`,
        close: () => new Promise<void>((resolveClose) => server.close(() => resolveClose())),
    };
}

test('final_protected_selectors', async ({ page }) => {
    for (const width of [1440, 1024, 390]) {
        await openTypedAt(page, width, width === 390 ? 844 : 900);
        const root = page.locator('#studio-typed-root');
        await expect(page.locator('#app')).toBeHidden();
        await expect(root.locator('.task-workspace')).toBeVisible();
        const classes = (await root.locator('.task-workspace').getAttribute('class') ?? '').split(/\s+/);
        expect(classes, `${width}: task workspace classes`).toEqual(expect.arrayContaining(['harness-shell', 'studio-shell', 'task-workspace']));
        for (const selector of PROTECTED_SELECTORS) {
            await expect(root.locator(selector).first(), `${width}: ${selector} must exist in the typed root`).toHaveCount(1);
        }
        const rail = root.locator('[data-region="task-rail"]');
        if (width === 390) {
            await expect(rail).toBeHidden();
            await openRail(page);
        }
        for (const selector of PROTECTED_SELECTORS) {
            if (width === 390 && selector === '[data-control="environment"]') continue;
            await expect(root.locator(selector).first(), `${width}: ${selector} must be visible`).toBeVisible();
        }
        if (width === 390) {
            await expect(root.locator('[data-control="environment"]')).toHaveCount(1);
        }
        await expect(rail).toBeVisible();

        const navigation = root.locator('nav[aria-label="Studio"]');
        await expect(navigation.getByRole('button')).toHaveCount(6);
        for (const label of ['Agent', 'Workspace', 'Changes', 'Terminal', 'Models', 'Guide']) {
            await expect(navigation.getByRole('button', { name: label, exact: true })).toBeVisible();
        }
        await expect(root.locator('nav[aria-label="Tasks and local sessions"]')).toBeVisible();
        await expect(root.locator('textarea[aria-label="Prompt"]')).toBeVisible();
        await expect(root.getByRole('button', { name: 'Send', exact: true })).toHaveCount(1);
        await expect(page.locator('a.skip-link')).toHaveCount(1);
        await expect(page.locator('a.skip-link')).toHaveAttribute('href', '#harness-main');
        await expect(root.locator('a.skip-link')).toHaveCount(0);
        await expect(page.locator('#harness-main')).toBeVisible();
    }

    await page.goto('/?legacy=1');
    await expect(page.locator('#app')).toBeVisible();
    await expect(page.locator('#studio-typed-root')).toBeHidden();
    await expect(page.locator('a.skip-link')).toHaveCount(1);
    await expect(page.locator('a.skip-link')).toHaveAttribute('href', '#mainColumn');
});

test('final_1024_boundary', async ({ page }) => {
    await openTypedAt(page, 1024);
    const root = page.locator('#studio-typed-root');
    const rail = root.locator('[data-region="task-rail"]');
    const toggle = root.locator('[data-action="toggle-projects"]');
    await expect(rail).toBeVisible();
    await expect(rail).toHaveCSS('position', 'static');
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');
    await page.waitForTimeout(300);
    await expect(rail).toBeVisible();
    expect(await page.evaluate(() => matchMedia('(max-width: 1023px)').matches)).toBe(false);

    await page.setViewportSize({ width: 1440, height: 900 });
    await expect(rail).toBeVisible();
    await page.setViewportSize({ width: 1024, height: 900 });
    await page.waitForTimeout(300);
    await expect(rail).toBeVisible();

    await toggle.click();
    await expect(rail).toBeHidden();
    await expect(toggle).toHaveAttribute('aria-expanded', 'false');
    await page.waitForTimeout(300);
    await expect(rail).toBeHidden();
    await page.setViewportSize({ width: 1100, height: 900 });
    await page.setViewportSize({ width: 1024, height: 900 });
    await page.waitForTimeout(300);
    await expect(rail).toBeHidden();
    await toggle.click();
    await expect(rail).toBeVisible();
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');

    await openTypedAt(page, 1023);
    await expect(rail).toBeHidden();
    await expect(rail).toHaveCSS('position', 'fixed');
    await expect(toggle).toHaveAttribute('aria-expanded', 'false');
    expect(await page.evaluate(() => matchMedia('(max-width: 1023px)').matches)).toBe(true);
    await toggle.click();
    await expect(rail).toBeVisible();
    await page.setViewportSize({ width: 1024, height: 900 });
    await page.waitForTimeout(300);
    await expect(rail).toBeVisible();
    await expect(rail).toHaveCSS('position', 'static');
    expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBe(0);
    expect(await page.evaluate(() => document.body.scrollWidth - document.body.clientWidth)).toBe(0);
});

test('final_keyboard_only_task_flow', async ({ page }) => {
    const pageErrors: string[] = [];
    page.on('pageerror', (error) => pageErrors.push(error.message));
    await page.setViewportSize({ width: 1440, height: 900 });
    await openStudio(page, '/?typed=1&studioTest=1');
    const seeded = await call<{ taskId: string }>(page, 'createTaskAndEvents', {
        mode: 'finished',
        title: 'Keyboard terminal task',
    });
    await page.reload();
    await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
    const row = page.locator(`#studio-typed-root [data-task-id="${seeded.taskId}"]`);
    await expect(row).toBeVisible();
    await expect(row.locator('.task-state'), 'a real terminal run must not project as Idle in the rail').toHaveText('Finished');

    await page.keyboard.press('Tab');
    await expect(page.locator('a.skip-link')).toBeFocused();
    await page.keyboard.press('Enter');
    await expect(page.locator('#harness-main')).toBeFocused();

    await tabTo(page, 'the terminal task row', (focus) => focus.label === 'Open Keyboard terminal task');
    await page.keyboard.press('Enter');
    const detail = page.locator('#studio-typed-root [data-task-detail-header]');
    await expect(detail).toBeVisible();
    await expect(detail.locator('.task-detail-status')).toHaveText('Status: Finished');
    await expect(page.locator('#studio-typed-root .task-detail-title')).toBeFocused();

    await tabTo(page, 'Back to tasks', (focus) => focus.text === 'Back to tasks');
    await page.keyboard.press('Enter');
    await expect(detail).toHaveCount(0);
    await expect(row.locator('.task-select')).toBeFocused();

    const localSessions = page.locator('#studio-typed-root details[data-task-group="local-sessions"]');
    if (!(await localSessions.evaluate((element) => (element as HTMLDetailsElement).open))) {
        await tabTo(page, 'the Local sessions summary', (focus) => focus.text.startsWith('Local sessions'));
        await page.keyboard.press('Enter');
    }
    await tabTo(page, 'New session', (focus) => focus.text.includes('New session'));
    await page.keyboard.press('Enter');
    await expect(localSessions.locator('[data-action="resume-session"]')).toHaveCount(1);
    expect(await activeSection(page)).toBe('Agent');

    await page.keyboard.press('ControlOrMeta+Shift+P');
    const palette = page.getByRole('dialog', { name: 'Command palette' });
    await expect(palette).toBeVisible();
    await page.keyboard.type('Go to Terminal');
    await page.keyboard.press('Enter');
    await expect(palette).toBeHidden();
    expect(await activeSection(page)).toBe('Terminal');
    await expect(page.locator('#studio-typed-root #terminalConsentState')).toBeVisible();

    await tabTo(page, 'the Local session resume button', (focus) => focus.label.startsWith('Select ') || focus.text.startsWith('Select '));
    await page.keyboard.press('Enter');
    expect(await activeSection(page), 'a Local-session action must activate the Agent view').toBe('Agent');
    await expect(page.locator('#studio-typed-root h1')).toHaveText('Agent');
    await expect(page.locator('#studio-typed-root .task-detail-status')).toHaveCount(0);

    await page.keyboard.press('ControlOrMeta+Shift+P');
    await expect(palette).toBeVisible();
    await page.keyboard.type('Go to Terminal');
    await page.keyboard.press('Enter');
    await expect(palette).toBeHidden();
    expect(await activeSection(page)).toBe('Terminal');
    await tabTo(page, 'New session from another section', (focus) => focus.text.includes('New session'));
    await page.keyboard.press('Enter');
    expect(await activeSection(page), 'creating a Local session must activate the Agent view').toBe('Agent');
    await expect(page.locator('#studio-typed-root h1')).toHaveText('Agent');

    const prompt = page.locator('#studio-typed-root textarea[aria-label="Prompt"]');
    await expect(page.locator('#studio-typed-root .send-button')).toBeDisabled();
    await page.evaluate(() => {
        const target = window as unknown as { __toastCalls: string[]; __studioUi: { toast(text: string, tone?: string): void } };
        target.__toastCalls = [];
        target.__studioUi.toast = (text: string) => { target.__toastCalls.push(String(text)); };
    });
    await prompt.focus();
    await page.keyboard.type('should not send');
    await page.keyboard.press('Enter');
    await expect(prompt).toHaveValue('should not send');
    expect(await page.evaluate(() => (window as unknown as { __toastCalls: string[] }).__toastCalls)).toEqual([]);
    expect(pageErrors, `no uncaught error on a disabled Enter: ${pageErrors.join(' | ')}`).toEqual([]);
    await expect(page.locator('#toastRegion .toast')).toHaveCount(0);
    await expect(page.locator('#studio-typed-root .task-detail-status')).toHaveCount(0);
    await expect(row.locator('.task-select')).not.toHaveAttribute('aria-current', 'true');
    await expect(row.locator('.task-state')).toHaveText('Finished');
});

test('final_ime_cjk_dead_key_rtl', async ({ page }) => {
    await openTypedAt(page, 1440);
    const root = page.locator('#studio-typed-root');
    await root.locator('nav[aria-label="Studio"]').getByRole('button', { name: 'Models', exact: true }).click();
    await page.getByRole('button', { name: 'Add API provider' }).click();
    await page.getByLabel('Provider name').fill('Fixture provider');
    await page.getByLabel('Provider base URL').fill('http://127.0.0.1:8787/v1');
    await page.getByLabel('Provider model').fill('fixture');
    await page.getByRole('button', { name: 'Save provider' }).click();
    await page.getByRole('button', { name: 'Use Fixture provider' }).click();
    await root.locator('nav[aria-label="Studio"]').getByRole('button', { name: 'Agent', exact: true }).click();
    await page.evaluate(() => {
        const target = window as unknown as { __studioSendCalls: string[]; __studioUi: Record<string, unknown> };
        target.__studioSendCalls = [];
        target.__studioUi.sendPrompt = (text: string) => {
            target.__studioSendCalls.push(String(text));
            return 424242;
        };
        target.__studioUi.predict = (text: string) => {
            target.__studioSendCalls.push(`predict:${String(text)}`);
            return 424243;
        };
    });
    await page.evaluate(() => {
        const target = window as unknown as { __compositionEvents: Array<Record<string, unknown>> };
        target.__compositionEvents = [];
        for (const type of ['compositionstart', 'compositionupdate', 'compositionend']) {
            window.addEventListener(type, (event) => {
                target.__compositionEvents.push({ type, data: (event as CompositionEvent).data, trusted: event.isTrusted });
            }, true);
        }
        const prompt = document.querySelector<HTMLTextAreaElement>('#studio-typed-root textarea[aria-label="Prompt"]');
        prompt?.addEventListener('keydown', (event) => {
            target.__compositionEvents.push({ type: 'keydown', key: event.key, keyCode: event.keyCode, isComposing: event.isComposing });
        }, true);
    });
    const records = () => page.evaluate(() => (window as unknown as { __compositionEvents: Array<Record<string, unknown>> }).__compositionEvents);
    const sent = () => page.evaluate(() => (window as unknown as { __studioSendCalls: string[] }).__studioSendCalls);
    const prompt = page.locator('#studio-typed-root textarea[aria-label="Prompt"]');
    const cdp = await page.context().newCDPSession(page);

    test.info().annotations.push({
        type: 'limitation',
        description: 'A physical macOS dead-key press cannot be synthesized by CDP. This check drives the same trusted composition event sequence and records physical OS dead-key coverage as manual/NOT_RUN.',
    });
    await page.evaluate(() => {
        const synthetic = new CompositionEvent('compositionupdate', { data: 'x', bubbles: true });
        (document.querySelector('#studio-typed-root textarea[aria-label="Prompt"]') as HTMLElement).dispatchEvent(synthetic);
    });
    expect((await records()).find((record) => record.type === 'compositionupdate' && record.data === 'x')?.trusted).toBe(false);

    await prompt.fill('existing ');
    await prompt.focus();
    await prompt.evaluate((element) => element.setSelectionRange(9, 9));
    await cdp.send('Input.imeSetComposition', { text: 'にほん', selectionStart: 3, selectionEnd: 3 });
    await expect.poll(async () => (await records()).filter((record) => record.type === 'compositionstart').length).toBe(1);
    let events = await records();
    expect(events.find((record) => record.type === 'compositionstart' && record.trusted === true)).toBeDefined();
    expect(events.find((record) => record.type === 'compositionupdate' && record.data === 'にほん' && record.trusted === true)).toBeDefined();
    expect(await prompt.inputValue()).toBe('existing にほん');
    await cdp.send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13, nativeVirtualKeyCode: 13 });
    await cdp.send('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13, nativeVirtualKeyCode: 13 });
    await expect.poll(async () => (await records()).filter((record) => record.type === 'keydown' && record.key === 'Enter').length).toBeGreaterThan(0);
    events = await records();
    expect(events.find((record) => record.type === 'keydown' && record.key === 'Enter')?.isComposing).toBe(true);
    expect(await sent()).toEqual([]);
    await cdp.send('Input.insertText', { text: '日本' });
    await expect(prompt).toHaveValue('existing 日本');
    expect(await prompt.evaluate((element) => element.selectionStart)).toBe('existing 日本'.length);
    expect(await sent()).toEqual([]);

    await prompt.fill('caf');
    await prompt.focus();
    await prompt.evaluate((element) => element.setSelectionRange(3, 3));
    await cdp.send('Input.imeSetComposition', { text: '´', selectionStart: 1, selectionEnd: 1 });
    await cdp.send('Input.imeSetComposition', { text: 'é', selectionStart: 1, selectionEnd: 1 });
    await cdp.send('Input.insertText', { text: 'é' });
    await expect(prompt).toHaveValue('café');
    expect(await prompt.evaluate((element) => element.selectionStart)).toBe(4);
    events = await records();
    const updates = events.filter((record) => record.type === 'compositionupdate' && record.trusted === true).map((record) => record.data);
    expect(updates).toEqual(expect.arrayContaining(['´', 'é']));
    expect(updates.indexOf('´')).toBeLessThan(updates.lastIndexOf('é'));
    expect(events.some((record) => record.type === 'keydown' && record.key === 'Dead')).toBe(false);
    expect(await sent()).toEqual([]);

    await page.context().grantPermissions(['clipboard-read', 'clipboard-write'], { origin: 'http://127.0.0.1:4173' });
    const rtl = 'مرحبا بالعالم';
    await page.evaluate(async (value) => { await navigator.clipboard.writeText(value); }, rtl);
    await prompt.fill('alpha omega');
    await prompt.focus();
    await prompt.evaluate((element) => element.setSelectionRange(6, 11));
    await page.keyboard.press('Meta+V');
    await expect(prompt).toHaveValue(`alpha ${rtl}`);
    expect(await prompt.evaluate((element) => element.selectionStart)).toBe(6 + rtl.length);
    await prompt.evaluate((element) => element.setSelectionRange(6, 6));
    await page.keyboard.press('Meta+V');
    await expect(prompt).toHaveValue(`alpha ${rtl}${rtl}`);
    expect(await prompt.evaluate((element) => element.selectionStart)).toBe(6 + rtl.length);
    expect(await sent()).toEqual([]);

    const longText = Array.from({ length: 320 }, (_, index) => `line ${index} ${'x'.repeat(40)}`).join('\n');
    await page.evaluate(async (value) => { await navigator.clipboard.writeText(value); }, longText);
    await prompt.fill('start ');
    await prompt.focus();
    await prompt.evaluate((element) => element.setSelectionRange(6, 6));
    await page.keyboard.press('Meta+V');
    await expect.poll(async () => (await prompt.inputValue()).length).toBe(`start ${longText}`.length);
    const state = await prompt.evaluate((element) => ({
        caret: element.selectionStart,
        scrollTop: element.scrollTop,
        scrollHeight: element.scrollHeight,
        clientHeight: element.clientHeight,
    }));
    expect(state.caret).toBe(`start ${longText}`.length);
    expect(state.scrollHeight).toBeGreaterThan(state.clientHeight);
    expect(state.scrollTop).toBeGreaterThan(0);
    expect(await sent()).toEqual([]);
    await page.setViewportSize({ width: 390, height: 844 });
    expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBeLessThanOrEqual(0);
    await expect(page.locator('#studio-typed-root .agent-composer')).toBeVisible();
});

test('final_static_unsupported_states', async ({ page }) => {
    const external: string[] = [];
    const websockets: string[] = [];
    page.on('request', (request: Request) => {
        const url = request.url();
        if (url.startsWith('http://127.0.0.1:4173/')) return;
        if (url.startsWith('data:') || url.startsWith('blob:')) return;
        external.push(url);
    });
    page.on('websocket', (socket) => websockets.push(socket.url()));

    for (const width of [1440, 1024, 390]) {
        await openTypedAt(page, width, width === 390 ? 844 : 900);
        await page.waitForLoadState('networkidle');
        const root = page.locator('#studio-typed-root');
        await expect(root).toHaveAttribute('data-host', 'static');
        expect(await page.evaluate(() => typeof (window as unknown as { __TAURI__?: unknown }).__TAURI__)).toBe('undefined');
        expect(await page.evaluate(() => typeof (window as unknown as { __studioTest?: unknown }).__studioTest)).toBe('undefined');
        if (width === 390) await openRail(page);

        await root.locator('nav[aria-label="Studio"]').getByRole('button', { name: 'Terminal', exact: true }).click();
        await expect(root.locator('#terminalConsentState')).toHaveText(TERMINAL_COPY);
        await expect(root.getByRole('button', { name: 'Open terminal', exact: true })).toBeDisabled();
        await expect(root.locator('.terminal-screen[data-session-id]')).toHaveCount(0);

        await root.locator('nav[aria-label="Studio"]').getByRole('button', { name: 'Changes', exact: true }).click();
        await expect(root.locator('#gitStatusSummary')).toHaveText(GIT_COPY);
        await expect(root.locator('#gitStatusSummary')).not.toContainText('connected');
        await expect(root.getByRole('button', { name: 'Refresh changes', exact: true })).toBeDisabled();
        await expect(root.locator('#gitDiffOutput')).toHaveText('');
        await expect(root.getByRole('button', { name: 'Copy diff', exact: true })).toBeDisabled();

        await root.locator('header.session-toolbar button[aria-label="Settings"]').click();
        const settings = page.getByRole('dialog', { name: 'Studio settings', exact: true });
        await expect(settings).toBeVisible();
        await expect(settings.locator('section[aria-label="Account and team"] p')).toHaveText(ACCOUNT_COPY);
        await expect(settings.getByRole('button', { name: /account|team|sign in|billing|upgrade/i })).toHaveCount(0);
        await page.keyboard.press('Escape');
        await expect(settings).toBeHidden();

        await root.locator('nav[aria-label="Studio"]').getByRole('button', { name: 'Agent', exact: true }).click();
        await expect(root).not.toContainText('runtime_unavailable');
        await expect(root).not.toContainText(/\btokens?\s*\/\s*s\b/i);
        await expect(root).not.toContainText('useful conversation is established');
    }

    await page.setViewportSize({ width: 1440, height: 900 });
    await openStudio(page, '/?typed=1&studioTest=1');
    await call(page, 'open');
    const seeded = await call<{ ok: boolean; message?: string }>(page, 'seedBackgroundHoldForTest');
    expect(seeded.ok, JSON.stringify(seeded)).toBe(true);
    await page.reload();
    await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
    await page.locator('#studio-typed-root').getByRole('button', { name: 'Open Fixture task', exact: true }).click();
    await expect(page.locator('#studio-typed-root [data-static-background]')).toHaveText(BACKGROUND_COPY);
    await expect(page.locator('#studio-typed-root').getByRole('button', { name: /background/i })).toHaveCount(0);

    expect(websockets).toEqual([]);
    expect(external, `external requests observed: ${external.join(', ')}`).toEqual([]);
});

test('no_agent_secret_is_written_to_local_storage', async ({ page }) => {
    await openStudio(page, '/?typed=1&studioTest=1');
    await call(page, 'open');
    const seeded = await call<{ taskId: string; runId: string }>(page, 'createTaskAndEvents', {
        mode: 'finished',
        title: 'Local storage task',
    });
    await page.reload();
    await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
    const row = page.locator(`#studio-typed-root [data-task-id="${seeded.taskId}"] .task-select`);
    await expect(row).toBeVisible();
    await row.click();
    await expect(page.locator('#studio-typed-root [data-task-detail-header]')).toBeVisible();
    await page.waitForTimeout(150);

    const storage = await page.evaluate(() => Object.fromEntries(
        Array.from({ length: window.localStorage.length }, (_, index) => {
            const key = window.localStorage.key(index)!;
            return [key, window.localStorage.getItem(key) ?? ''];
        }),
    ));
    const keys = Object.keys(storage);
    expect(keys.length, 'the page must not use localStorage as an empty fallback for agent state').toBeGreaterThan(0);
    for (const key of keys) {
        expect(key, `unexpected localStorage key ${key}`).toMatch(/^uor\.studio\.[a-z0-9.]+$/i);
        expect(key, `agent/security state must not be in localStorage: ${key}`).not.toMatch(/agent|secret|token|host|signing|capability|receipt|bearer|private/i);
    }
    for (const [key, value] of Object.entries(storage)) {
        expect(value, `${key} must not contain agent identifiers`).not.toMatch(/\b(tsk_|run_|evt_|intent_|req_)[0-9a-f]{8,}/);
        expect(value, `${key} must not contain a digest chain`).not.toMatch(/sha256:[0-9a-f]{64}/);
        expect(value, `${key} must not contain a bearer token`).not.toMatch(/bearer\s+[a-z0-9._-]+/i);
        expect(value, `${key} must not contain a signing/private key`).not.toMatch(/signing[_ -]?key|private[_ -]?key|agent[_ -]?secret/i);
        expect(value, `${key} must not contain a non-empty API key without the documented exception`).not.toMatch(/"apiKey"\s*:\s*"([^"]+)"/);
    }
    const databases = await page.evaluate(async () => (await indexedDB.databases()).map((entry) => entry.name));
    expect(databases).toContain('uor-r4.studio.agent');
    const agentStateInStorage = Object.entries(storage).filter(([key]) => key.includes('uor-r4.studio.agent'));
    expect(agentStateInStorage).toEqual([]);
});

test('model_sources_key_is_documented_exception', async ({ page }) => {
    await openStudio(page, '/?typed=1&studioTest=1');
    const root = page.locator('#studio-typed-root');
    await root.locator('nav[aria-label="Studio"]').getByRole('button', { name: 'Models', exact: true }).click();
    await page.getByRole('button', { name: 'Add API provider' }).click();
    await page.getByLabel('Provider name').fill('Exception provider');
    await page.getByLabel('Provider base URL').fill('http://127.0.0.1:8787/v1');
    await page.getByLabel('Provider model').fill('fixture');
    const secret = 'sk-local-documented-exception-123';
    await page.getByLabel('API key').fill(secret);
    await page.getByRole('button', { name: 'Save provider' }).click();
    await expect(page.getByRole('button', { name: 'Use Exception provider' })).toBeVisible();

    const storage = await page.evaluate(() => Object.fromEntries(
        Array.from({ length: window.localStorage.length }, (_, index) => {
            const key = window.localStorage.key(index)!;
            return [key, window.localStorage.getItem(key) ?? ''];
        }),
    ));
    expect(Object.keys(storage)).toContain('uor.studio.modelSources.v1');
    const modelSources = JSON.parse(storage['uor.studio.modelSources.v1']) as {
        api?: Array<{ label?: string; apiKey?: string; ephemeral?: boolean }>;
    };
    const provider = modelSources.api?.find((entry) => entry.label === 'Exception provider');
    expect(provider, 'the documented user-provider store must contain the saved provider').toBeDefined();
    expect(provider?.apiKey).toBe(secret);
    expect(provider?.ephemeral).not.toBe(true);
    for (const [key, value] of Object.entries(storage)) {
        if (key === 'uor.studio.modelSources.v1') continue;
        expect(value.includes(secret), `the provider key must not leak into ${key}`).toBe(false);
    }
    expect(await page.locator('#studio-typed-root').textContent()).not.toContain(secret);
    expect(await page.evaluate(() => document.querySelectorAll('input[type="text"], input:not([type])').length >= 0)).toBe(true);
    const keys = Object.keys(storage);
    expect(keys.filter((key) => key !== 'uor.studio.modelSources.v1' && /secret|token|signing|private|bearer/i.test(key))).toEqual([]);
});

test('default_build_does_not_expose_studio_test_api', async ({ page }) => {
    expect(
        existsSync(join(DEFAULT_BUILD_DIR, 'index.html')),
        `default build archive missing at ${DEFAULT_BUILD_DIR}; build it without VITE_STUDIO_TEST and copy dist before Playwright starts`,
    ).toBe(true);
    const server = await serveDirectory(DEFAULT_BUILD_DIR);
    const requests: string[] = [];
    page.on('request', (request) => requests.push(request.url()));
    try {
        await page.goto(`${server.url}/?typed=1&studioTest=1&theme=dark`);
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        expect(await page.evaluate(() => typeof (window as unknown as { __studioTest?: unknown }).__studioTest)).toBe('undefined');
        const jsFiles = walkFiles(DEFAULT_BUILD_DIR).filter((file) => file.endsWith('.js'));
        expect(jsFiles.length).toBeGreaterThan(0);
        const urls = jsFiles.map((file) => `${server.url}/${file.slice(DEFAULT_BUILD_DIR.length + 1).split(sep).join('/')}`);
        const staticReferences = jsFiles.filter((file) => readFileSync(file, 'utf8').includes('installStudioTestApi'));
        expect(staticReferences, 'default build must not carry test-API code').toEqual([]);
        const results = await page.evaluate(async (chunkUrls) => {
            const loaded: Array<{ url: string; exported: string[]; error: string | null; after: string }> = [];
            for (const url of chunkUrls) {
                let exported: string[] = [];
                let error: string | null = null;
                try {
                    const module = await import(/* @vite-ignore */ url) as Record<string, unknown>;
                    exported = Object.keys(module);
                } catch (failure) {
                    error = failure instanceof Error ? failure.message : String(failure);
                }
                loaded.push({
                    url,
                    exported,
                    error,
                    after: typeof (window as unknown as { __studioTest?: unknown }).__studioTest,
                });
            }
            return loaded;
        }, urls);
        expect(results).toHaveLength(jsFiles.length);
        expect(results.filter((result) => result.exported.includes('installStudioTestApi'))).toEqual([]);
        expect(results.filter((result) => result.after !== 'undefined')).toEqual([]);
        expect(requests.filter((url) => /testApi/i.test(url))).toEqual([]);
        const origin = new URL(server.url).origin;
        expect(requests.filter((url) => !url.startsWith(`${origin}/`) && !url.startsWith('data:') && !url.startsWith('blob:'))).toEqual([]);
    } finally {
        await server.close();
    }

    await page.goto('/?typed=1&studioTest=1');
    await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
    expect(await page.evaluate(() => typeof (window as unknown as { __studioTest?: unknown }).__studioTest)).toBe('object');
    await page.goto('/?typed=1');
    await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
    expect(await page.evaluate(() => typeof (window as unknown as { __studioTest?: unknown }).__studioTest)).toBe('undefined');
});

interface SourceEvidence {
    file: string;
    name: string;
}

interface MatrixRow {
    status: 'executed' | 'not_run' | 'unverified' | 'not_a_uor_target';
    evidence?: SourceEvidence[];
    future?: string[];
    unverified?: string[];
    notRun?: string[];
    reason: string;
}

const MATRIX_EVIDENCE: Record<number, MatrixRow> = {
    1: {
        status: 'not_run',
        future: ['auth.rs', 'tenant.rs'],
        reason: 'NOT_RUN: hosted access, identity, session and tenant isolation are Appendix A service work (`auth.rs`, `tenant.rs`) and are not authorized by this plan.',
    },
    2: {
        status: 'executed',
        evidence: [
            { file: 'web/studio/src/components/TaskRail.test.tsx', name: 'renders_today_last_seven_days_older_groups from task record timestamps' },
            { file: 'web/studio/tests/e2e/cursor-shell.spec.ts', name: 'task_rail_unifies_local_sessions_and_an_honest_empty_state' },
        ],
        reason: 'Task rail, search, pins and groups execute in component and browser acceptance.',
    },
    3: {
        status: 'executed',
        evidence: [
            { file: 'web/studio/src/components/ComposerSurface.test.tsx', name: 'visible_send_button_and_prompt_label_are_preserved' },
            { file: 'web/studio/tests/e2e/cursor-ime.spec.ts', name: 'existing_ime_composition_regression' },
        ],
        reason: 'Composer and IME behavior execute; real placeholder parity with Cursor remains UNVERIFIED.',
    },
    4: {
        status: 'executed',
        evidence: [
            { file: 'web/studio/src/agent/runtime.test.ts', name: 'queue_message_is_delivered_at_next_turn' },
            { file: 'web/studio/tests/e2e/cursor-tasks.spec.ts', name: 'stop_is_active_only_and_delivery_errors_stay_message_scoped' },
        ],
        reason: 'Queue, steer, force, stop and retry execute against the browser runtime.',
    },
    5: {
        status: 'executed',
        evidence: [
            { file: 'crates/uor-r4-studio-agent/tests/contract_events.rs', name: 'contract_schema_id_is_stable' },
            { file: 'web/studio/src/agent/ingest.test.ts', name: 'decode_frame_buffers_partial_frames_at_every_byte_boundary' },
            { file: 'web/studio/src/components/TaskTimeline.test.tsx', name: 'renders events in sequence order rather than array order' },
        ],
        reason: 'Contract, hostile ingest and transcript rendering execute.',
    },
    6: {
        status: 'executed',
        evidence: [
            { file: 'web/studio/src/components/TaskTimeline.test.tsx', name: 'pairs an approval with its tool request and states tool, scope, expiry and host' },
            { file: 'web/studio/tests/e2e/cursor-tasks.spec.ts', name: 'replay_detail_collapses_deltas_pairs_tools_and_never_fabricates_output' },
        ],
        unverified: ['Plan pill'],
        reason: 'Plan/approval rendering executes; the Cursor Plan pill remains UNVERIFIED.',
    },
    7: {
        status: 'not_run',
        reason: 'NOT_RUN: subagents, side chats and child tasks are outside the first slice.',
    },
    8: {
        status: 'not_a_uor_target',
        evidence: [{ file: 'web/studio/tests/e2e/cursor-review.spec.ts', name: 'pr_tab_is_read_only_and_explicitly_unavailable_without_a_hosted_integration' }],
        reason: 'NOT A UOR TARGET (hosted): no checkpoint or restore control is rendered; host checkpoints remain UNVERIFIED.',
    },
    9: {
        status: 'executed',
        evidence: [{ file: 'web/studio/src/components/DiffReview.test.tsx', name: 'changes_snapshot_last_turn_scope' }],
        unverified: ['picked commits'],
        reason: 'Last-turn and branch scopes execute; Cursor picked commits remain UNVERIFIED.',
    },
    10: {
        status: 'executed',
        evidence: [{ file: 'web/studio/src/components/PrPanel.test.tsx', name: 'pr_tab_is_read_only_link_without_integration' }],
        notRun: ['merge'],
        reason: 'Read-only PR review executes; merge is NOT_RUN without service identity.',
    },
    11: {
        status: 'executed',
        evidence: [
            { file: 'src-tauri/tests/agent_authority.rs', name: 'agent_host_terminal_requires_existing_approval' },
            { file: 'src-tauri/tests/interactive_terminal.rs', name: 'approved_pty_supports_multiple_inputs_resize_and_close' },
            { file: 'web/studio/tests/e2e/cursor-static.spec.ts', name: 'static_browser_disables_terminal_with_exact_copy' },
        ],
        notRun: ['web remote PTY'],
        reason: 'Approved local PTY and static unsupported copy execute; web remote PTY is NOT_RUN.',
    },
    12: { status: 'not_a_uor_target', reason: 'NOT A UOR TARGET: Cursor cloud desktop is desktop-only and is not built.' },
    13: {
        status: 'executed',
        evidence: [{ file: 'web/studio/src/components/ArtifactPanel.test.tsx', name: 'artifact_read_preserves_host_artifact_id_and_whole_metadata' }],
        reason: 'Digest-verified artifact reads and inert previews execute.',
    },
    14: {
        status: 'executed',
        evidence: [
            { file: 'web/studio/src/components/FileTree.test.tsx', name: 'file_tree_uses_agent_client_tree' },
            { file: 'web/studio/src/components/EditorPane.test.tsx', name: 'editor_labels_it_is_not_vs_code_or_lsp' },
        ],
        unverified: ['depth'],
        reason: 'Tree and editor execute; Cursor web editing depth remains UNVERIFIED.',
    },
    15: { status: 'not_a_uor_target', reason: 'NOT A UOR TARGET: LSP, debugger, extensions, themes and notebooks are not built.' },
    16: {
        status: 'executed',
        evidence: [{ file: 'web/studio/tests/e2e/cursor-keyboard.spec.ts', name: 'scoped_palette_shortcut_matches_the_toolbar_hint' }],
        reason: 'Scoped UOR shortcuts execute; no global Cmd+K or Tab is claimed.',
    },
    17: { status: 'not_run', reason: 'NOT_RUN: voice chat and dictation are not in this plan.' },
    18: {
        status: 'executed',
        evidence: [
            { file: 'src-tauri/tests/mcp_protocol.rs', name: 'remote_json_and_sse_share_session_protocol_and_cors' },
            { file: 'src-tauri/tests/agent_authority.rs', name: 'agent_host_refuses_page_supplied_grant' },
        ],
        unverified: ['SSE'],
        reason: 'Existing MCP transport and host authority execute; cloud-agent SSE remains UNVERIFIED.',
    },
    19: {
        status: 'not_run',
        future: ['secrets.rs'],
        reason: 'NOT_RUN: the encrypted secret store and injection/redaction (`secrets.rs`) are Appendix A service work.',
    },
    20: {
        status: 'not_run',
        future: ['background.rs', 'notifications.rs'],
        reason: 'NOT_RUN: automations, subscriptions and notification delivery (`background.rs`, `notifications.rs`) are Appendix A service work.',
    },
    21: {
        status: 'not_run',
        future: ['github.rs'],
        reason: 'NOT_RUN: GitHub App install, webhooks and check readback (`github.rs`) are Appendix A service work.',
    },
    22: { status: 'not_run', reason: 'NOT_RUN: other provider integrations are later and not planned.' },
    23: {
        status: 'not_run',
        future: ['auth.rs', 'tenant.rs', 'secrets.rs'],
        reason: 'NOT_RUN: account, admin, billing, SSO, SCIM, audit and privacy (`auth.rs`, `tenant.rs`, `secrets.rs`) are Appendix A service work.',
    },
    24: { status: 'not_a_uor_target', reason: 'NOT A UOR TARGET: iOS, Android, external API/CLI/SDK surfaces are not built.' },
    25: {
        status: 'executed',
        evidence: [
            { file: 'web/studio/tests/e2e/cursor-a11y.spec.ts', name: 'final_a11y_targets_headings_focus' },
            { file: 'web/studio/tests/e2e/cursor-shell.spec.ts', name: '390_has_no_horizontal_overflow' },
        ],
        reason: 'Visual tokens, responsive boundaries and accessibility execute at 390/1024/1440.',
    },
    26: {
        status: 'not_run',
        reason: 'NOT_RUN: self-hosted and managed workers have no executable test; future proposed only.',
    },
    27: {
        status: 'executed',
        evidence: [
            { file: 'web/studio/tests/e2e/cursor-static.spec.ts', name: 'static_browser_makes_zero_external_requests_on_load' },
            { file: 'web/studio/src/agent/agentClient.test.ts', name: 'fixture_start_run_emits_run_queued_zero_and_run_started_one' },
        ],
        reason: 'Local privacy is executed; the model itself is a replay fixture, not a generation claim.',
    },
    28: {
        status: 'unverified',
        evidence: [{ file: 'web/studio/tests/e2e/cursor-review.spec.ts', name: 'pr_tab_is_read_only_and_explicitly_unavailable_without_a_hosted_integration' }],
        reason: 'UNVERIFIED: no documented Cursor Pause/Delete/Logs/cost UI; UOR renders no invented control.',
    },
    29: {
        status: 'unverified',
        evidence: [{ file: 'web/studio/tests/e2e/cursor-review.spec.ts', name: 'pr_tab_is_read_only_and_explicitly_unavailable_without_a_hosted_integration' }],
        reason: 'UNVERIFIED / NOT A UOR TARGET: no documented hosted checkpoint restore feature; none is rendered.',
    },
    30: {
        status: 'not_run',
        future: ['github.rs', 'secrets.rs'],
        reason: 'NOT_RUN: signed commits, installation tokens and webhooks (`github.rs`, `secrets.rs`) are Appendix A service work.',
    },
};

function matrixRows(): string[] {
    const design = readFileSync(
        resolve(REPO_ROOT, 'docs/superpowers/specs/2026-11-09-cursor-hosted-studio-upgrade-design.md'),
        'utf8',
    );
    const start = design.indexOf('## 14. Coverage matrix');
    const end = design.indexOf('**What this matrix deliberately does not do.**');
    if (start < 0 || end < 0) throw new Error('coverage matrix section is missing');
    return design.slice(start, end).split('\n').filter((line) => /^\|\s*\d+\s*\|/.test(line));
}

function rowNumber(line: string): number {
    return Number(/^\|\s*(\d+)\s*\|/.exec(line)?.[1] ?? 0);
}

test('matrix_planned_evidence_names_reconciled', async () => {
    const rows = matrixRows();
    expect(rows.map(rowNumber)).toEqual(Array.from({ length: 30 }, (_, index) => index + 1));
    expect(Object.keys(MATRIX_EVIDENCE).map(Number).sort((left, right) => left - right)).toEqual(
        Array.from({ length: 30 }, (_, index) => index + 1),
    );
    for (const line of rows) {
        const number = rowNumber(line);
        const row = MATRIX_EVIDENCE[number];
        expect(row, `row ${number} needs an evidence decision`).toBeDefined();
        const lower = line.toLowerCase();
        if (row.status === 'executed') {
            expect(lower, `row ${number} must still be planned in the design`).toContain('planned');
            expect(row.evidence?.length ?? 0, `row ${number} needs executed evidence`).toBeGreaterThan(0);
        }
        if (row.status === 'not_run') {
            expect(lower, `row ${number} design status`).toContain('not_run');
            expect(row.reason, `row ${number} reason`).toContain('NOT_RUN');
        }
        if (row.status === 'unverified') {
            expect(lower, `row ${number} design status`).toContain('unverified');
            expect(row.reason, `row ${number} reason`).toContain('UNVERIFIED');
        }
        if (row.status === 'not_a_uor_target') {
            expect(lower, `row ${number} design status`).toContain('not a uor target');
            expect(row.reason, `row ${number} reason`).toContain('NOT A UOR TARGET');
        }
        for (const token of row.notRun ?? []) {
            expect(lower, `row ${number} design must retain NOT_RUN for ${token}`).toContain('not_run');
            expect(row.reason.toLowerCase(), `row ${number} reason must name ${token}`).toContain(token.toLowerCase());
        }
        for (const token of row.unverified ?? []) {
            expect(lower, `row ${number} design must retain UNVERIFIED for ${token}`).toContain('unverified');
            expect(row.reason.toLowerCase(), `row ${number} reason must name ${token}`).toContain(token.toLowerCase());
        }
        for (const future of row.future ?? []) {
            expect(line, `row ${number} must name its planned future file ${future}`).toContain(future);
            expect(row.reason, `row ${number} reason must name ${future}`).toContain(future);
            expect(existsSync(resolve(REPO_ROOT, 'services/studio-service/src', future)), `${future} must remain unbuilt`).toBe(false);
            expect(existsSync(resolve(REPO_ROOT, 'src-tauri/src', future)), `${future} must remain unbuilt`).toBe(false);
        }
        for (const evidence of row.evidence ?? []) {
            const path = resolve(REPO_ROOT, evidence.file);
            expect(existsSync(path), `row ${number} evidence file ${evidence.file} must exist`).toBe(true);
            expect(readFileSync(path, 'utf8'), `row ${number} evidence ${evidence.name} must exist in ${evidence.file}`).toContain(evidence.name);
        }
    }
});

test.describe('Default build independent acceptance', () => {
    test('default_build_protected_shell_and_1024_boundary', async ({ page }) => {
        expect(
            existsSync(join(DEFAULT_BUILD_DIR, 'index.html')),
            `default build archive missing at ${DEFAULT_BUILD_DIR}`,
        ).toBe(true);
        const server = await serveDirectory(DEFAULT_BUILD_DIR);
        try {
            for (const width of [1440, 1024, 390]) {
                await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
                await page.goto(`${server.url}/?typed=1`);
                const root = page.locator('#studio-typed-root');
                await expect(root).toHaveAttribute('data-active', 'true');
                expect(await page.evaluate(() => typeof (window as unknown as { __studioTest?: unknown }).__studioTest)).toBe('undefined');
                await expect(root.locator('.task-workspace')).toBeVisible();
                for (const selector of PROTECTED_SELECTORS) {
                    await expect(root.locator(selector).first(), `${width}: ${selector}`).toHaveCount(1);
                }
                const rail = root.locator('[data-region="task-rail"]');
                if (width === 390) {
                    await expect(rail).toBeHidden();
                    await root.locator('[data-action="toggle-projects"]').click();
                }
                await expect(rail).toBeVisible();
                await expect(root.locator('[data-action="new-task"]')).toBeVisible();
                await expect(root.locator('nav[aria-label="Studio"]').getByRole('button')).toHaveCount(6);
                await expect(page.locator('a.skip-link')).toHaveCount(1);
                await expect(page.locator('a.skip-link')).toHaveAttribute('href', '#harness-main');
            }

            await page.setViewportSize({ width: 1024, height: 900 });
            await page.goto(`${server.url}/?typed=1`);
            const root = page.locator('#studio-typed-root');
            const rail = root.locator('[data-region="task-rail"]');
            const toggle = root.locator('[data-action="toggle-projects"]');
            await expect(rail).toBeVisible();
            await expect(rail).toHaveCSS('position', 'static');
            await expect(toggle).toHaveAttribute('aria-expanded', 'true');
            await page.setViewportSize({ width: 1023, height: 900 });
            await page.goto(`${server.url}/?typed=1`);
            await expect(rail).toBeHidden();
            await expect(rail).toHaveCSS('position', 'fixed');
            await expect(toggle).toHaveAttribute('aria-expanded', 'false');
            await toggle.click();
            await expect(rail).toBeVisible();
            await page.setViewportSize({ width: 1024, height: 900 });
            await page.waitForTimeout(300);
            await expect(rail).toBeVisible();
        } finally {
            await server.close();
        }
    });

    test('default_build_create_run_stop_reload_new_run_without_studio_test', async ({ page }) => {
        expect(existsSync(join(DEFAULT_BUILD_DIR, 'index.html'))).toBe(true);
        const server = await serveDirectory(DEFAULT_BUILD_DIR);
        try {
            await page.setViewportSize({ width: 1440, height: 900 });
            await page.goto(`${server.url}/?typed=1`);
            const root = page.locator('#studio-typed-root');
            await expect(root).toHaveAttribute('data-active', 'true');
            expect(await page.evaluate(() => typeof (window as unknown as { __studioTest?: unknown }).__studioTest)).toBe('undefined');
            await expect(root.locator('[data-action="new-task"]')).toBeVisible();

            const prompt = root.locator('textarea[aria-label="Prompt"]');
            await expect(prompt).toBeVisible();
            await prompt.fill('first default-build prompt');
            await prompt.press('Enter');
            const row = root.locator('[data-task-id]').filter({ hasText: 'New task' }).first();
            await expect(row).toBeVisible();

            const stopRun = root.locator('.composer-run-modes').getByRole('button', { name: 'Stop run', exact: true });
            await expect(stopRun).toBeEnabled();
            await expect(row.locator('.task-state')).toHaveText('Running');
            await stopRun.click();
            await expect(row.locator('.task-state')).toHaveText('Cancelled');

            await page.reload();
            await expect(root).toHaveAttribute('data-active', 'true');
            const restored = root.locator('[data-task-id]').filter({ hasText: 'New task' }).first();
            await expect(restored.locator('.task-state')).toHaveText('Cancelled');
            await restored.locator('.task-select').click();
            await expect(root.locator('[data-task-detail-header]')).toBeVisible();
            await expect(root.locator('.task-detail-status')).toHaveText('Status: Cancelled');
            await root.getByRole('button', { name: 'New run', exact: true }).click();
            await expect(root.locator('[data-task-detail-header]')).toHaveCount(0);
            await expect(prompt).toBeFocused();
            await page.evaluate(() => {
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
            await prompt.fill('second default-build prompt');
            await prompt.press('Enter');
            await page.waitForTimeout(500);
            expect(await page.evaluate(() => (window as unknown as { __studioToastCalls: string[] }).__studioToastCalls)).toEqual([]);
            await expect(restored.locator('.task-state')).toHaveText(/Running|Finished/);
            await restored.locator('.task-select').click();
            await expect(root.locator('.task-detail-status')).toHaveText('Status: Finished', { timeout: 10_000 });
            await expect(root.locator('[data-timeline-kind="assistant"]')).toContainText(
                'Synthetic replay acknowledges: second default-build prompt',
            );
            await expect(root.locator('.task-synthetic')).toContainText('not an R4 model');
        } finally {
            await server.close();
        }
    });

    test('runtime_composer_ime_cjk_and_disabled_enter_on_the_default_build', async ({ page }) => {
        expect(existsSync(join(DEFAULT_BUILD_DIR, 'index.html'))).toBe(true);
        const server = await serveDirectory(DEFAULT_BUILD_DIR);
        try {
            await page.setViewportSize({ width: 1440, height: 900 });
            await page.goto(`${server.url}/?typed=1`);
            const root = page.locator('#studio-typed-root');
            await expect(root).toHaveAttribute('data-active', 'true');
            const prompt = root.locator('textarea[aria-label="Prompt"]');
            const cdp = await page.context().newCDPSession(page);

            await prompt.fill('existing ');
            await prompt.focus();
            await prompt.evaluate((element) => element.setSelectionRange(9, 9));
            await cdp.send('Input.imeSetComposition', { text: 'にほん', selectionStart: 3, selectionEnd: 3 });
            await expect(prompt).toHaveValue('existing にほん');
            await cdp.send('Input.dispatchKeyEvent', {
                type: 'keyDown',
                key: 'Enter',
                code: 'Enter',
                windowsVirtualKeyCode: 13,
                nativeVirtualKeyCode: 13,
            });
            await cdp.send('Input.dispatchKeyEvent', {
                type: 'keyUp',
                key: 'Enter',
                code: 'Enter',
                windowsVirtualKeyCode: 13,
                nativeVirtualKeyCode: 13,
            });
            await expect(root.locator('[data-task-id]')).toHaveCount(0);
            await cdp.send('Input.insertText', { text: '日本' });
            await expect(prompt).toHaveValue('existing 日本');

            await prompt.press('Enter');
            const row = root.locator('[data-task-id]').filter({ hasText: 'New task' }).first();
            await expect(row).toBeVisible();
            await expect(row.locator('.task-state')).toHaveText('Running');
            await expect(root.locator('.send-button')).toBeDisabled();
            await prompt.fill('disabled during the active run');
            await prompt.press('Enter');
            await expect(prompt).toHaveValue('disabled during the active run');
            await expect(row.locator('.task-state')).toHaveText(/Running|Finished/);

            await expect(row.locator('.task-state')).toHaveText('Finished', { timeout: 10_000 });
            await row.locator('.task-select').click();
            await expect(root.locator('[data-timeline-kind="assistant"]')).toContainText(
                'Synthetic replay acknowledges: existing 日本',
            );
            await expect(root.locator('.task-synthetic')).toContainText('not an R4 model');
        } finally {
            await server.close();
        }
    });
});
