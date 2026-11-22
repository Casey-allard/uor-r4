import { expect, test, type BrowserContext, type Page } from '@playwright/test';
import { openTypedAt } from './support';

async function callTest<T>(page: Page, method: string, ...args: unknown[]): Promise<T> {
    return page.evaluate(({ method, args }) => {
        const api = (window as unknown as {
            __studioTest?: Record<string, (...callArgs: unknown[]) => Promise<unknown>>;
        }).__studioTest;
        if (!api) throw new Error('window.__studioTest is missing');
        return api[method]!(...args);
    }, { method, args }) as Promise<T>;
}

async function waitForSecondWriter(context: BrowserContext): Promise<Page> {
    const tabA = await context.newPage();
    await tabA.goto('/?typed=1&studioTest=1');
    await expect(tabA.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
    await callTest(tabA, 'open');
    return tabA;
}

interface TargetViolation {
    label?: string;
    tag: string;
    name: string;
    width: number;
    height: number;
}

async function targetViolations(page: Page, minimum: number, scope = '#studio-typed-root'): Promise<TargetViolation[]> {
    return page.evaluate(({ minimum, scope }) => {
        const root = document.querySelector<HTMLElement>(scope);
        if (!root) return [{ tag: 'SCOPE', name: `missing ${scope}`, width: 0, height: 0 }];
        const visible = (element: HTMLElement) => {
            const style = getComputedStyle(element);
            if (style.display === 'none' || style.visibility === 'hidden' || Number(style.opacity) === 0) return false;
            if (element.closest('[aria-hidden="true"]')) return false;
            const rect = element.getBoundingClientRect();
            return rect.width > 0 && rect.height > 0;
        };
        const violations: TargetViolation[] = [];
        const seen = new Set<Element>();
        for (const element of Array.from(root.querySelectorAll<HTMLElement>('button, a[href], input, select, textarea, summary, [tabindex], [role="option"], [role="menuitem"], [role="tab"]'))) {
            if ((element as HTMLButtonElement).disabled || element.hasAttribute('disabled')) continue;
            if (element.getAttribute('tabindex') === '-1') continue;
            if (element.classList.contains('visually-hidden') || element.closest('.visually-hidden')) continue;
            if (element.classList.contains('skip-link')) continue;
            if (element.matches('input[type="hidden"]')) continue;
            if (!visible(element)) continue;
            let target: HTMLElement = element;
            const type = (element as HTMLInputElement).type;
            if ((type === 'checkbox' || type === 'radio') && element.closest('label')) target = element.closest('label') as HTMLElement;
            if (seen.has(target)) continue;
            seen.add(target);
            const rect = target.getBoundingClientRect();
            if (rect.width + 0.01 < minimum || rect.height + 0.01 < minimum) {
                violations.push({
                    tag: element.tagName,
                    name: (element.getAttribute('aria-label') || element.textContent || '').replace(/\s+/g, ' ').trim().slice(0, 70),
                    width: Math.round(rect.width * 10) / 10,
                    height: Math.round(rect.height * 10) / 10,
                });
            }
        }
        return violations;
    }, { minimum, scope });
}

async function mobileFormFontViolations(page: Page): Promise<Array<{ name: string; fontSize: number }>> {
    return page.evaluate(() => {
        const root = document.querySelector('#studio-typed-root');
        if (!root) return [{ name: 'missing typed root', fontSize: 0 }];
        const violations: Array<{ name: string; fontSize: number }> = [];
        for (const element of Array.from(root.querySelectorAll<HTMLElement>('input, select, textarea'))) {
            const type = (element as HTMLInputElement).type;
            if (['checkbox', 'radio', 'file', 'hidden'].includes(type)) continue;
            const style = getComputedStyle(element);
            if (style.display === 'none' || style.visibility === 'hidden') continue;
            const rect = element.getBoundingClientRect();
            if (rect.width === 0 || rect.height === 0) continue;
            const fontSize = Number.parseFloat(style.fontSize);
            if (fontSize < 16) violations.push({ name: element.getAttribute('aria-label') || element.tagName, fontSize });
        }
        return violations;
    });
}

interface HeadingRow { level: number; text: string }

async function visibleHeadings(page: Page, scope: string): Promise<HeadingRow[]> {
    return page.evaluate((selector) => {
        const root = document.querySelector<HTMLElement>(selector);
        if (!root) return [];
        return Array.from(root.querySelectorAll<HTMLElement>('h1, h2, h3, h4, h5, h6'))
            .filter((element) => {
                const style = getComputedStyle(element);
                const rect = element.getBoundingClientRect();
                return style.display !== 'none' && style.visibility !== 'hidden' && rect.width > 0 && rect.height > 0;
            })
            .map((element) => ({ level: Number(element.tagName.slice(1)), text: element.textContent?.trim() ?? '' }));
    }, scope);
}

function expectWellFormed(rows: HeadingRow[], label: string) {
    expect(rows[0]?.level, `${label} must start at h1`).toBe(1);
    expect(rows.filter((row) => row.level === 1).length, `${label} must keep exactly one h1`).toBe(1);
    for (let index = 1; index < rows.length; index += 1) {
        expect(rows[index].level - rows[index - 1].level, `${label} heading order ${JSON.stringify(rows)}`).toBeLessThanOrEqual(1);
    }
}

function expectRailWellFormed(rows: HeadingRow[]) {
    expect(rows.length).toBeGreaterThan(0);
    expect(rows[0].level).toBe(2);
    for (let index = 1; index < rows.length; index += 1) {
        expect(rows[index].level - rows[index - 1].level, `rail heading order ${JSON.stringify(rows)}`).toBeLessThanOrEqual(1);
    }
}

interface FlatHeadingRow extends HeadingRow { tag: string; role: string | null; ariaLevel: string | null }

async function flatVisibleHeadings(page: Page): Promise<FlatHeadingRow[]> {
    return page.evaluate(() => {
        const root = document.querySelector<HTMLElement>('#studio-typed-root');
        if (!root) return [];
        return Array.from(root.querySelectorAll<HTMLElement>('h1, h2, h3, h4, h5, h6, [role="heading"]'))
            .filter((element) => {
                const style = getComputedStyle(element);
                const rect = element.getBoundingClientRect();
                return style.display !== 'none' && style.visibility !== 'hidden' && rect.width > 0 && rect.height > 0;
            })
            .map((element) => {
                const native = /^H([1-6])$/.exec(element.tagName);
                const ariaLevel = element.getAttribute('aria-level');
                return {
                    level: native ? Number(native[1]) : Number(ariaLevel ?? 0),
                    text: element.textContent?.trim() ?? '',
                    tag: element.tagName,
                    role: element.getAttribute('role'),
                    ariaLevel,
                };
            });
    });
}

async function selectFixtureProvider(page: Page) {
    const root = page.locator('#studio-typed-root');
    await root.locator('nav[aria-label="Studio"]').getByRole('button', { name: 'Models', exact: true }).click();
    await page.getByRole('button', { name: 'Add API provider' }).click();
    await page.getByLabel('Provider name').fill('Fixture provider');
    await page.getByLabel('Provider base URL').fill('http://127.0.0.1:8787/v1');
    await page.getByLabel('Provider model').fill('fixture');
    await page.getByRole('button', { name: 'Save provider' }).click();
    await page.getByRole('button', { name: 'Use Fixture provider' }).click();
    await root.locator('nav[aria-label="Studio"]').getByRole('button', { name: 'Agent', exact: true }).click();
}

function promptLocator(page: Page) {
    return page.locator('#studio-typed-root textarea[aria-label="Prompt"]');
}

test('typed_view_exposes_landmarks_and_one_skip_link', async ({ page }) => {
    await openTypedAt(page, 1440);
    const root = page.locator('#studio-typed-root');
    await expect(root.locator('header.session-toolbar')).toBeVisible();
    await expect(root.locator('nav[aria-label="Studio"]')).toBeVisible();
    await expect(root.locator('nav[aria-label="Tasks and local sessions"]')).toBeVisible();
    await expect(root.locator('main#harness-main')).toBeVisible();
    await expect(root.locator('aside.project-sidebar')).toBeVisible();
    await expect(page.locator('a.skip-link')).toHaveCount(1);
    const skip = page.locator('a.skip-link');
    await expect(skip).toHaveAttribute('href', '#harness-main');
    await skip.focus();
    await page.keyboard.press('Enter');
    await expect(page.locator('#harness-main')).toBeFocused();

    await page.goto('/?legacy=1');
    await expect(page.locator('#app')).toBeVisible();
    await expect(page.locator('a.skip-link')).toHaveCount(1);
    await expect(page.locator('a.skip-link')).toHaveAttribute('href', '#mainColumn');
});

test('visible_heading_hierarchy_is_well_formed_in_every_view', async ({ page }) => {
    await openTypedAt(page, 1440);
    const root = page.locator('#studio-typed-root');
    const navigation = root.locator('nav[aria-label="Studio"]');
    for (const label of ['Agent', 'Workspace', 'Changes', 'Terminal', 'Models', 'Guide']) {
        await navigation.getByRole('button', { name: label, exact: true }).click();
        await expect(root.locator('main h1:visible'), `${label} must show one h1`).toHaveCount(1);
        expectWellFormed(await visibleHeadings(page, '#studio-typed-root main'), label);
    }
    expectRailWellFormed(await visibleHeadings(page, '#studio-typed-root aside.project-sidebar'));
});

test('split_panes_each_have_one_distinguished_h1', async ({ page }) => {
    await openTypedAt(page, 1440);
    const root = page.locator('#studio-typed-root');
    await root.locator('[data-action="split-pane"]').click();
    const panes = root.locator('.pane-columns > [data-pane]');
    await expect(panes).toHaveCount(2);
    const names = await panes.evaluateAll((elements) => elements.map((element) => element.getAttribute('data-pane')));
    expect(new Set(names).size).toBe(2);
    const headings: string[] = [];
    for (const name of names) {
        const rows = await visibleHeadings(page, `#studio-typed-root [data-pane="${name}"]`);
        expectWellFormed(rows, `split pane ${name}`);
        headings.push(rows.find((row) => row.level === 1)?.text ?? '');
    }
    expect(new Set(headings).size).toBe(2);
    await expect(root.locator('[data-region="work-surface"]')).toBeVisible();
});

test('flat_visible_heading_order_starts_with_the_main_h1', async ({ page }) => {
    await openTypedAt(page, 1440);
    const root = page.locator('#studio-typed-root');
    await expect(root.locator('[data-region="task-rail"]')).toBeVisible();
    const rows = await flatVisibleHeadings(page);
    expect(rows.length).toBeGreaterThan(1);
    expect(rows[0]?.text).toBe('Agent');
    expect(rows[0]?.level).toBe(1);
    for (let index = 1; index < rows.length; index += 1) {
        expect(rows[index].level - rows[index - 1].level, `flat heading order ${JSON.stringify(rows)}`).toBeLessThanOrEqual(1);
    }
    const roleHeadings = rows.filter((row) => row.role === 'heading');
    expect(roleHeadings.every((row) => Number.isInteger(row.level) && row.level >= 1 && row.level <= 6)).toBe(true);
});

test('no_aria_hidden_content_is_focusable', async ({ page }) => {
    await openTypedAt(page, 1440);
    const violations = await page.evaluate(() => {
        const root = document.querySelector('#studio-typed-root');
        if (!root) return ['missing typed root'];
        return Array.from(root.querySelectorAll('[aria-hidden="true"]'))
            .filter((element) => element.querySelector('button, a[href], input, select, textarea, summary, [tabindex]:not([tabindex="-1"])'))
            .map((element) => element.outerHTML.slice(0, 120));
    });
    expect(violations).toEqual([]);
});

test('transcript_is_a_polite_live_region_with_real_busy_state', async ({ page }) => {
    await page.goto('/?typed=1&studioTest=1');
    await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
    const seeded = await page.evaluate(async () => {
        const api = (window as unknown as {
            __studioTest?: {
                open(): Promise<unknown>;
                createTaskAndEvents(options: { mode: string; title: string }): Promise<{ taskId: string }>;
            };
        }).__studioTest;
        if (!api) throw new Error('window.__studioTest is missing');
        await api.open();
        return api.createTaskAndEvents({ mode: 'message', title: 'Live region task' });
    });
    await page.reload();
    await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
    const row = page.locator(`#studio-typed-root [data-task-id="${seeded.taskId}"] .task-select`);
    await expect(row).toBeVisible();
    await row.click();
    const transcript = page.locator('#studio-typed-root .task-detail-transcript');
    await expect(transcript).toHaveAttribute('aria-live', 'polite');
    await expect(transcript).toHaveAttribute('aria-busy', 'true');
    const stop = page.locator('#studio-typed-root .run-status').getByRole('button', { name: 'Stop', exact: true });
    await expect(stop).toBeEnabled();
    await stop.click();
    await expect(transcript).toHaveAttribute('aria-busy', 'false');
    await expect(transcript).toHaveAttribute('aria-live', 'polite');
});

test('errors_use_alert_and_session_status_is_polite', async ({ context, page: tabB }) => {
    await tabB.goto('/?typed=1&studioTest=1');
    await expect(tabB.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
    await callTest(tabB, 'open');
    const seeded = await callTest<{ taskId: string }>(tabB, 'createTaskAndEvents', {
        mode: 'finished',
        title: 'Error task',
    });
    await tabB.reload();
    await expect(tabB.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
    const tabA = await waitForSecondWriter(context);
    try {
        const rowA = tabA.locator(`#studio-typed-root [data-task-id="${seeded.taskId}"] .task-select`);
        await expect(rowA).toBeVisible();
        await rowA.click();
        await tabA.getByRole('button', { name: 'New run', exact: true }).click();
        const promptA = tabA.locator('#studio-typed-root textarea[aria-label="Prompt"]');
        await promptA.fill('tab A holds the error-test lease');
        await promptA.press('Enter');
        await expect(
            tabA.locator(`#studio-typed-root [data-task-id="${seeded.taskId}"] .task-state`),
        ).toHaveText('Running');

        const rowB = tabB.locator(`#studio-typed-root [data-task-id="${seeded.taskId}"] .task-select`);
        await expect(rowB).toBeVisible();
        await rowB.click();
        await tabB.getByRole('button', { name: 'New run', exact: true }).click();
        const prompt = promptLocator(tabB);
        await prompt.fill('will fail');
        await prompt.focus();
        await tabB.keyboard.press('Enter');
        const alert = tabB.locator('#studio-typed-root [role="alert"]');
        await expect(alert).toBeVisible();
        await expect(alert).toContainText('Run could not start');
        await expect(prompt).toHaveValue('will fail');
        const status = tabB.locator('#studio-typed-root [role="status"][aria-live="polite"]');
        await expect(status).toBeVisible();
    } finally {
        await tabA.close();
    }
});

test('menus_expose_expanded_listbox_and_option_semantics', async ({ page }) => {
    await openTypedAt(page, 1440);
    const root = page.locator('#studio-typed-root');
    const rail = root.locator('[data-region="task-rail"]');
    const toggle = root.locator('[data-action="toggle-projects"]');
    await expect(rail).toBeVisible();
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');
    const prompt = promptLocator(page);
    await prompt.fill('/');
    const commands = page.locator('#studio-typed-root [role="listbox"][aria-label="Build task commands"]');
    await expect(commands).toBeVisible();
    await expect(prompt).toHaveAttribute('aria-expanded', 'true');
    await expect(prompt).toHaveAttribute('aria-controls', 'agent-composer-commands');
    const firstOption = commands.getByRole('option').first();
    await expect(firstOption).toHaveAttribute('aria-selected', 'true');
    await expect(prompt).toHaveAttribute('aria-activedescendant', await firstOption.getAttribute('id') ?? '');
    await page.keyboard.press('Escape');
    await expect(commands).toHaveCount(0);
    await expect(prompt).toHaveAttribute('aria-expanded', 'false');
    await expect(prompt).toBeFocused();
    await expect(rail).toBeVisible();
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');

    await page.locator('#studio-typed-root .composer-attachments input[type="file"]').setInputFiles({ name: 'notes.txt', mimeType: 'text/plain', buffer: Buffer.from('note') });
    await expect(page.locator('#studio-typed-root .composer-attachment-chip')).toBeVisible();
    await prompt.fill('hello @');
    const mentions = page.locator('#studio-typed-root [role="listbox"][aria-label="Attachment mentions"]');
    await expect(mentions).toBeVisible();
    await expect(mentions.getByRole('option')).toHaveCount(1);
    await expect(prompt).toHaveAttribute('aria-expanded', 'true');
    await prompt.press('Escape');
    await expect(mentions).toHaveCount(0);
    await expect(prompt).toBeFocused();
    await expect(prompt).toHaveValue('hello @');
    await expect(rail).toBeVisible();
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');
});

test('dialogs_return_focus_to_their_invokers_without_closing_the_rail', async ({ page }) => {
    await openTypedAt(page, 1440);
    const root = page.locator('#studio-typed-root');
    const rail = root.locator('[data-region="task-rail"]');
    const toggle = root.locator('[data-action="toggle-projects"]');
    await expect(rail).toBeVisible();
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');
    const paletteButton = root.locator('.toolbar-command');
    await paletteButton.click();
    const palette = page.getByRole('dialog', { name: 'Command palette' });
    await expect(palette).toBeVisible();
    expect(await page.evaluate(() => Boolean(document.activeElement?.closest('dialog')))).toBe(true);
    await page.keyboard.press('Escape');
    await expect(palette).toBeHidden();
    await expect(paletteButton).toBeFocused();
    await expect(rail).toBeVisible();
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');

    const settingsButton = root.locator('.session-toolbar-actions button[aria-label="Settings"]');
    await settingsButton.click();
    const settings = page.getByRole('dialog', { name: 'Studio settings', exact: true });
    await expect(settings).toBeVisible();
    expect(await page.evaluate(() => Boolean(document.activeElement?.closest('dialog')))).toBe(true);
    await page.keyboard.press('Escape');
    await expect(settings).toBeHidden();
    await expect(settingsButton).toBeFocused();
    await expect(rail).toBeVisible();
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');
});

test('tab_order_moves_from_the_prompt_to_send_and_back', async ({ page }) => {
    await openTypedAt(page, 1440);
    await selectFixtureProvider(page);
    const prompt = promptLocator(page);
    await prompt.fill('tab order');
    await prompt.focus();
    await page.keyboard.press('Tab');
    await expect(page.locator('#studio-typed-root .send-button')).toBeFocused();
    await page.keyboard.press('Shift+Tab');
    await expect(prompt).toBeFocused();
});

test('every_visible_interactive_target_is_at_least_24_css_pixels', async ({ page }) => {
    const violations: TargetViolation[] = [];
    const audit = async (label: string, scope = '#studio-typed-root') => {
        violations.push(...(await targetViolations(page, 24, scope)).map((row) => ({ ...row, label })));
    };
    await openTypedAt(page, 1440);
    const root = page.locator('#studio-typed-root');
    const navigation = root.locator('nav[aria-label="Studio"]');
    await audit('1440 agent');
    await root.locator('[data-pane-target="editor"]').click();
    await root.getByRole('button', { name: 'Open demo', exact: true }).click();
    await expect(root.locator('.code-input textarea[aria-label="File contents"]')).toBeEnabled();
    await audit('1440 workspace');
    for (const [pane, label] of [['diff', '1440 changes'], ['terminal', '1440 terminal']] as const) {
        await root.locator(`[data-pane-target="${pane}"]`).click();
        await audit(label);
    }
    await navigation.getByRole('button', { name: 'Models', exact: true }).click();
    await audit('1440 models');
    await navigation.getByRole('button', { name: 'Guide', exact: true }).click();
    await audit('1440 guide');
    await navigation.getByRole('button', { name: 'Agent', exact: true }).click();
    await root.locator('[data-action="split-pane"]').click();
    await audit('1440 split');
    await root.locator('[data-action="close-split"]').click();
    await root.locator('.toolbar-command').click();
    await expect(page.getByRole('dialog', { name: 'Command palette' })).toBeVisible();
    await audit('1440 palette', 'dialog[open]');
    await page.keyboard.press('Escape');
    const prompt = promptLocator(page);
    await prompt.fill('/');
    await expect(page.locator('#studio-typed-root [role="listbox"][aria-label="Build task commands"]')).toBeVisible();
    await audit('1440 composer menu');
    await page.keyboard.press('Escape');

    await openTypedAt(page, 390, 844);
    await audit('390 agent');
    await root.locator('[data-action="toggle-projects"]').click();
    await audit('390 drawer');
    await root.locator('[data-action="toggle-projects"]').click();
    await root.locator('[data-action="split-pane"]').click();
    await audit('390 split');
    await root.locator('[data-action="close-split"]').click();
    await root.locator('.session-toolbar-actions button[aria-label="Settings"]').click();
    await expect(page.getByRole('dialog', { name: 'Studio settings', exact: true })).toBeVisible();
    await audit('390 settings', 'dialog[open]');
    await page.keyboard.press('Escape');
    await prompt.fill('/');
    await expect(page.locator('#studio-typed-root [role="listbox"][aria-label="Build task commands"]')).toBeVisible();
    await audit('390 composer menu');
    await page.keyboard.press('Escape');
    await root.locator('[data-action="toggle-projects"]').click();
    await root.locator('nav[aria-label="Studio"]').getByRole('button', { name: 'Models', exact: true }).click();
    await root.locator('[data-action="toggle-projects"]').click();
    await expect(root.locator('.models-panel')).toBeVisible();
    await audit('390 models');

    await openTypedAt(page, 1024);
    await audit('1024 agent');
    await root.locator('[data-action="split-pane"]').click();
    await audit('1024 split');

    expect(violations, JSON.stringify(violations, null, 2)).toEqual([]);
});

test('coarse_pointer_targets_are_at_least_44_css_pixels', async ({ browser }) => {
    const context = await browser.newContext({ viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true });
    const page = await context.newPage();
    await page.goto('http://127.0.0.1:4173/?typed=1');
    await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
    expect(await page.evaluate(() => matchMedia('(pointer: coarse)').matches)).toBe(true);
    expect(await page.evaluate(() => navigator.maxTouchPoints)).toBeGreaterThan(0);
    const violations: TargetViolation[] = [];
    const root = page.locator('#studio-typed-root');
    violations.push(...(await targetViolations(page, 44)).map((row) => ({ ...row, label: '390 coarse agent' })));
    await root.locator('[data-action="toggle-projects"]').click();
    violations.push(...(await targetViolations(page, 44)).map((row) => ({ ...row, label: '390 coarse drawer' })));
    await root.locator('[data-action="toggle-projects"]').click();
    await root.locator('.session-toolbar-actions button[aria-label="Settings"]').click();
    await expect(page.getByRole('dialog', { name: 'Studio settings', exact: true })).toBeVisible();
    violations.push(...(await targetViolations(page, 44, 'dialog[open]')).map((row) => ({ ...row, label: '390 coarse settings' })));
    await page.keyboard.press('Escape');
    await root.locator('.toolbar-command').click();
    await expect(page.getByRole('dialog', { name: 'Command palette' })).toBeVisible();
    violations.push(...(await targetViolations(page, 44, 'dialog[open]')).map((row) => ({ ...row, label: '390 coarse palette' })));
    await page.keyboard.press('Escape');
    const prompt = promptLocator(page);
    await prompt.fill('/');
    await expect(page.locator('#studio-typed-root [role="listbox"][aria-label="Build task commands"]')).toBeVisible();
    violations.push(...(await targetViolations(page, 44)).map((row) => ({ ...row, label: '390 coarse menu' })));
    await page.keyboard.press('Escape');
    await root.locator('[data-action="toggle-projects"]').click();
    await root.locator('nav[aria-label="Studio"]').getByRole('button', { name: 'Models', exact: true }).click();
    await root.locator('[data-action="toggle-projects"]').click();
    await expect(root.locator('.models-panel')).toBeVisible();
    violations.push(...(await targetViolations(page, 44)).map((row) => ({ ...row, label: '390 coarse models' })));
    expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBe(0);
    await context.close();

    const wide = await browser.newContext({ viewport: { width: 1024, height: 900 }, hasTouch: true, isMobile: true });
    const widePage = await wide.newPage();
    await widePage.goto('http://127.0.0.1:4173/?typed=1');
    await expect(widePage.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
    expect(await widePage.evaluate(() => matchMedia('(pointer: coarse)').matches)).toBe(true);
    violations.push(...(await targetViolations(widePage, 44)).map((row) => ({ ...row, label: '1024 coarse agent' })));
    await widePage.locator('#studio-typed-root [data-action="split-pane"]').click();
    violations.push(...(await targetViolations(widePage, 44)).map((row) => ({ ...row, label: '1024 coarse split' })));
    expect(await widePage.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBe(0);
    await wide.close();
    expect(violations, JSON.stringify(violations, null, 2)).toEqual([]);
});

test('mobile_form_text_is_at_least_16px', async ({ page }) => {
    await openTypedAt(page, 390, 844);
    const violations: Array<{ label: string; name: string; fontSize: number }> = [];
    violations.push(...(await mobileFormFontViolations(page)).map((row) => ({ ...row, label: '390 agent' })));
    const root = page.locator('#studio-typed-root');
    await root.locator('[data-action="toggle-projects"]').click();
    violations.push(...(await mobileFormFontViolations(page)).map((row) => ({ ...row, label: '390 drawer' })));
    await root.locator('[data-action="toggle-projects"]').click();
    await root.locator('.session-toolbar-actions button[aria-label="Settings"]').click();
    await expect(page.getByRole('dialog', { name: 'Studio settings', exact: true })).toBeVisible();
    violations.push(...(await mobileFormFontViolations(page)).map((row) => ({ ...row, label: '390 settings' })));
    await page.keyboard.press('Escape');
    await root.locator('[data-pane-target="editor"]').click();
    await root.getByRole('button', { name: 'Open demo', exact: true }).first().click();
    const file = root.locator('.code-input textarea[aria-label="File contents"]');
    await expect(file).toBeEnabled();
    await expect(file).toHaveCSS('font-size', '16px');
    violations.push(...(await mobileFormFontViolations(page)).map((row) => ({ ...row, label: '390 workspace' })));
    await root.locator('.session-toolbar-actions button[aria-label="Settings"]').click();
    const editorFontSize = page.getByRole('dialog', { name: 'Studio settings', exact: true }).getByLabel('Editor font size');
    await editorFontSize.fill('20');
    await editorFontSize.press('Tab');
    await page.keyboard.press('Escape');
    await expect(file).toHaveCSS('font-size', '20px');
    expect(violations, JSON.stringify(violations, null, 2)).toEqual([]);
});

test('mobile_editor_gutter_and_textarea_share_line_metrics', async ({ page }) => {
    await openTypedAt(page, 390, 844);
    const root = page.locator('#studio-typed-root');
    await root.locator('[data-pane-target="editor"]').click();
    await root.getByRole('button', { name: 'Open demo', exact: true }).first().click();
    const file = root.locator('.code-input textarea[aria-label="File contents"]');
    await expect(file).toBeEnabled();
    interface EditorMetrics {
        gutterFont: number;
        editorFont: number;
        gutterLineHeight: number;
        editorLineHeight: number;
        firstLineHeight: number;
        lastLineHeight: number;
        firstLineTop: number;
        lastLineBottom: number;
        editorContentTop: number;
        lineCount: number;
    }
    const metrics = async (): Promise<EditorMetrics | null> => page.evaluate(() => {
        const gutter = document.querySelector<HTMLElement>('#studio-typed-root .code-gutter');
        const editor = document.querySelector<HTMLTextAreaElement>('#studio-typed-root .code-input textarea');
        if (!gutter || !editor) return null;
        const gs = getComputedStyle(gutter);
        const es = getComputedStyle(editor);
        const lines = Array.from(gutter.querySelectorAll<HTMLElement>('div'));
        const first = lines[0]?.getBoundingClientRect();
        const last = lines[lines.length - 1]?.getBoundingClientRect();
        const contentTop = (element: Element, style: CSSStyleDeclaration) => element.getBoundingClientRect().top + Number.parseFloat(style.borderTopWidth) + Number.parseFloat(style.paddingTop);
        return {
            gutterFont: Number.parseFloat(gs.fontSize),
            editorFont: Number.parseFloat(es.fontSize),
            gutterLineHeight: Number.parseFloat(gs.lineHeight),
            editorLineHeight: Number.parseFloat(es.lineHeight),
            firstLineHeight: first?.height ?? 0,
            lastLineHeight: last?.height ?? 0,
            firstLineTop: first?.top ?? 0,
            lastLineBottom: last?.bottom ?? 0,
            editorContentTop: contentTop(editor, es),
            lineCount: lines.length,
        };
    });
    const assertAligned = (value: EditorMetrics | null, font: number) => {
        expect(value).not.toBeNull();
        const measured = value!;
        expect(measured.gutterFont).toBe(font);
        expect(measured.editorFont).toBe(font);
        expect(measured.gutterLineHeight).toBe(measured.editorLineHeight);
        expect(Math.abs(measured.firstLineHeight - measured.editorLineHeight)).toBeLessThanOrEqual(0.02);
        expect(Math.abs(measured.lastLineHeight - measured.editorLineHeight)).toBeLessThanOrEqual(0.02);
        expect(Math.abs(measured.firstLineTop - measured.editorContentTop)).toBeLessThanOrEqual(0.5);
        expect(Math.abs(measured.lastLineBottom - (measured.editorContentTop + measured.lineCount * measured.editorLineHeight))).toBeLessThanOrEqual(1);
    };
    assertAligned(await metrics(), 16);

    await root.locator('.session-toolbar-actions button[aria-label="Settings"]').click();
    const editorFontSize = page.getByRole('dialog', { name: 'Studio settings', exact: true }).getByLabel('Editor font size');
    await editorFontSize.fill('20');
    await editorFontSize.press('Tab');
    await page.keyboard.press('Escape');
    await expect(file).toHaveCSS('font-size', '20px');
    assertAligned(await metrics(), 20);
});

test('390_900_1024_1440_have_no_horizontal_overflow', async ({ page }) => {
    const overflow = () => page.evaluate(() => ({
        document: document.documentElement.scrollWidth - document.documentElement.clientWidth,
        body: document.body.scrollWidth - document.body.clientWidth,
    }));
    await openTypedAt(page, 390, 844);
    expect(await overflow()).toEqual({ document: 0, body: 0 });
    await page.locator('#studio-typed-root [data-action="toggle-projects"]').click();
    expect(await overflow()).toEqual({ document: 0, body: 0 });
    await page.locator('#studio-typed-root [data-action="toggle-projects"]').click();
    await page.locator('#studio-typed-root [data-action="split-pane"]').click();
    expect(await overflow()).toEqual({ document: 0, body: 0 });
    await page.locator('#studio-typed-root [data-action="close-split"]').click();

    await openTypedAt(page, 900);
    await page.locator('#studio-typed-root [data-action="toggle-projects"]').click();
    expect(await overflow()).toEqual({ document: 0, body: 0 });

    await openTypedAt(page, 1024);
    expect(await overflow()).toEqual({ document: 0, body: 0 });
    await page.locator('#studio-typed-root [data-action="split-pane"]').click();
    expect(await overflow()).toEqual({ document: 0, body: 0 });

    await openTypedAt(page, 1440);
    expect(await overflow()).toEqual({ document: 0, body: 0 });
    await page.locator('#studio-typed-root [data-action="split-pane"]').click();
    expect(await overflow()).toEqual({ document: 0, body: 0 });
});

test('900px_rail_is_a_fixed_overlay', async ({ page }) => {
    await openTypedAt(page, 900);
    const root = page.locator('#studio-typed-root');
    await expect(root.locator('[data-region="task-rail"]')).toBeHidden();
    await root.locator('[data-action="toggle-projects"]').click();
    const rail = root.locator('[data-region="task-rail"]');
    await expect(rail).toBeVisible();
    const geometry = await rail.evaluate((element) => {
        const style = getComputedStyle(element);
        const rect = element.getBoundingClientRect();
        return { position: style.position, top: rect.top, bottom: rect.bottom, width: rect.width, shadow: style.boxShadow, zIndex: style.zIndex };
    });
    expect(geometry.position).toBe('fixed');
    expect(geometry.top).toBe(50);
    expect(geometry.bottom).toBe(900);
    expect(geometry.width).toBe(268);
    expect(geometry.shadow).not.toBe('none');
    expect(geometry.zIndex).toBe('40');
    const detailX = await root.locator('main[data-region="task-detail"]').evaluate((element) => element.getBoundingClientRect().x);
    expect(detailX).toBe(0);
    expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBe(0);
});

test('1024_boundary_is_exact', async ({ page }) => {
    await openTypedAt(page, 1024);
    const root = page.locator('#studio-typed-root');
    const rail = root.locator('[data-region="task-rail"]');
    const toggle = root.locator('[data-action="toggle-projects"]');
    await expect(rail).toBeVisible();
    await expect(rail).toHaveCSS('position', 'static');
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');
    await toggle.click();
    await expect(rail).toBeHidden();
    await page.waitForTimeout(200);
    await expect(rail).toBeHidden();
    await toggle.click();
    await expect(rail).toBeVisible();

    await openTypedAt(page, 1023);
    await expect(rail).toBeHidden();
    await expect(rail).toHaveCSS('position', 'fixed');
    await expect(toggle).toHaveAttribute('aria-expanded', 'false');
    await toggle.click();
    await expect(rail).toBeVisible();
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');
    expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBe(0);
});

test('classic_scrollbar_gutter_keeps_boundary_geometry', async ({ page }) => {
    for (const width of [1024, 1023]) {
        await openTypedAt(page, width);
        await page.addStyleTag({ content: 'html { padding-inline-end: 15px; }' });
        await expect.poll(async () => Math.round(await page.evaluate(() => document.body.getBoundingClientRect().width))).toBe(width - 15);
        expect(await page.evaluate(() => window.innerWidth)).toBe(width);
        expect(await page.evaluate(() => matchMedia('(max-width: 1023px)').matches)).toBe(width === 1023);
        expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBe(0);
        expect(await page.evaluate(() => document.body.scrollWidth - document.body.clientWidth)).toBe(0);
        const rail = page.locator('#studio-typed-root [data-region="task-rail"]');
        if (width === 1024) {
            await expect(rail).toBeVisible();
            await expect(rail).toHaveCSS('position', 'static');
        } else {
            await expect(rail).toBeHidden();
            await expect(rail).toHaveCSS('position', 'fixed');
        }
    }
});

test('narrow_view_is_single_column_with_a_sticky_composer', async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 500 });
    await page.goto('/?typed=1');
    await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
    expect(await page.evaluate(() => matchMedia('(max-width: 520px)').matches)).toBe(true);
    await expect(page.locator('#studio-typed-root .pane-columns')).toHaveCSS('display', 'block');
    await expect(page.locator('#studio-typed-root .agent-composer')).toHaveCSS('position', 'sticky');
    const pane = page.locator('#studio-typed-root .pane-primary');
    expect(await pane.evaluate((element) => element.scrollHeight)).toBeGreaterThan(await pane.evaluate((element) => element.clientHeight));
    for (const position of [0, 150, 10000]) {
        await pane.evaluate((element, value) => element.scrollTo(0, value), position);
        await page.waitForTimeout(50);
        const geometry = await page.locator('#studio-typed-root .agent-composer').evaluate((element) => {
            const rect = element.getBoundingClientRect();
            return { top: rect.top, bottom: rect.bottom, viewport: window.innerHeight };
        });
        expect(geometry.top).toBeGreaterThanOrEqual(0);
        expect(geometry.bottom).toBeLessThanOrEqual(geometry.viewport);
        expect(geometry.viewport - geometry.bottom).toBeLessThanOrEqual(13);
    }
    expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBe(0);
});

test('narrow_split_secondary_replaces_detail_and_returns', async ({ page }) => {
    await openTypedAt(page, 390, 844);
    const root = page.locator('#studio-typed-root');
    await root.locator('[data-action="split-pane"]').click();
    await expect(root.locator('.pane-primary')).toBeHidden();
    await expect(root.locator('.pane-secondary')).toBeVisible();
    await expect(root.locator('[data-region="work-surface"]')).toBeVisible();
    const close = root.locator('[data-action="close-split"]');
    await expect(close).toHaveAttribute('aria-label', 'Close split pane');
    await expect(close).toBeVisible();
    await close.click();
    await expect(root.locator('.pane-secondary')).toHaveCount(0);
    await expect(root.locator('.pane-primary')).toBeVisible();
    await expect(root.locator('.agent-composer')).toBeVisible();
    expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBe(0);
});

test('reduced_motion_removes_nonessential_animation', async ({ page }) => {
    await openTypedAt(page, 1440);
    await page.addStyleTag({ content: '.task-workspace .agent-composer { transition: border-color 120ms linear; }' });
    const activeBefore = await page.locator('#studio-typed-root .agent-composer').evaluate((element) => getComputedStyle(element).transitionDuration);
    expect(Number.parseFloat(activeBefore)).toBeGreaterThan(0);
    await page.emulateMedia({ reducedMotion: 'reduce' });
    expect(await page.evaluate(() => matchMedia('(prefers-reduced-motion: reduce)').matches)).toBe(true);
    const activeAfter = await page.locator('#studio-typed-root .agent-composer').evaluate((element) => getComputedStyle(element).transitionDuration);
    expect(Number.parseFloat(activeAfter)).toBeLessThanOrEqual(0.0001);
    const durations = await page.evaluate(() => {
        const root = document.querySelector('#studio-typed-root');
        if (!root) return [];
        return Array.from(root.querySelectorAll<HTMLElement>('*')).map((element) => {
            const style = getComputedStyle(element);
            return {
                name: typeof element.className === 'string' ? element.className.slice(0, 40) : element.tagName,
                animationDuration: style.animationDuration,
                transitionDuration: style.transitionDuration,
                scrollBehavior: style.scrollBehavior,
            };
        });
    });
    expect(durations.length).toBeGreaterThan(10);
    const active = durations.filter((row) => row.animationDuration !== '0s' || row.transitionDuration !== '0s');
    for (const row of active) {
        expect(Number.parseFloat(row.animationDuration) || 0, `${row.name} animation`).toBeLessThanOrEqual(0.0001);
        expect(Number.parseFloat(row.transitionDuration) || 0, `${row.name} transition`).toBeLessThanOrEqual(0.0001);
        expect(row.scrollBehavior).toBe('auto');
    }
});

test('focus_indication_is_visible_in_dark_and_light', async ({ page }) => {
    const readFocus = async (selector: string) => page.locator(selector).evaluate((element) => {
        const style = getComputedStyle(element);
        return { width: style.outlineWidth, line: style.outlineStyle, color: style.outlineColor, offset: style.outlineOffset };
    });
    for (const theme of ['dark', 'light'] as const) {
        await openTypedAt(page, 1440, 900, theme);
        await page.locator('#studio-typed-root .toolbar-command').focus();
        await page.keyboard.press('Tab');
        const settings = page.locator('#studio-typed-root .session-toolbar-actions button[aria-label="Settings"]');
        await expect(settings).toBeFocused();
        expect(await readFocus('#studio-typed-root .session-toolbar-actions button[aria-label="Settings"]')).toEqual({
            width: '2px',
            line: 'solid',
            color: theme === 'dark' ? 'rgb(245, 243, 238)' : 'rgb(31, 30, 26)',
            offset: '2px',
        });

        const prompt = promptLocator(page);
        await prompt.focus();
        expect(await readFocus('#studio-typed-root .agent-composer')).toEqual({ width: '2px', line: 'solid', color: 'rgb(217, 119, 87)', offset: '1px' });
        expect((await readFocus('#studio-typed-root textarea[aria-label="Prompt"]')).line).toBe('none');

        await page.locator('#studio-typed-root [data-pane-target="editor"]').click();
        await page.getByRole('button', { name: 'Open demo', exact: true }).click();
        const file = page.locator('#studio-typed-root .code-input textarea[aria-label="File contents"]');
        await expect(file).toBeEnabled();
        await file.focus();
        expect(await readFocus('#studio-typed-root .code-frame')).toEqual({
            width: '2px',
            line: 'solid',
            color: theme === 'dark' ? 'rgb(217, 119, 87)' : 'rgb(184, 90, 60)',
            offset: '2px',
        });
        expect((await readFocus('#studio-typed-root .code-input textarea[aria-label="File contents"]')).line).toBe('none');
    }
});

test('forced_colors_keeps_composer_and_code_editor_focus_indicators', async ({ page }) => {
    await openTypedAt(page, 1440);
    await page.emulateMedia({ forcedColors: 'active' });
    expect(await page.evaluate(() => matchMedia('(forced-colors: active)').matches)).toBe(true);
    const readFocus = async (selector: string) => page.locator(selector).evaluate((element) => {
        const style = getComputedStyle(element);
        return { width: style.outlineWidth, line: style.outlineStyle, color: style.outlineColor, offset: style.outlineOffset };
    });
    const prompt = promptLocator(page);
    await prompt.focus();
    expect(await readFocus('#studio-typed-root .agent-composer')).toEqual({ width: '2px', line: 'solid', color: 'rgb(0, 0, 0)', offset: '1px' });
    expect(await readFocus('#studio-typed-root textarea[aria-label="Prompt"]')).toEqual({ width: '2px', line: 'solid', color: 'rgb(0, 0, 0)', offset: '2px' });

    await page.locator('#studio-typed-root [data-pane-target="editor"]').click();
    await page.getByRole('button', { name: 'Open demo', exact: true }).click();
    const file = page.locator('#studio-typed-root .code-input textarea[aria-label="File contents"]');
    await expect(file).toBeEnabled();
    await file.focus();
    expect(await readFocus('#studio-typed-root .code-frame')).toEqual({ width: '2px', line: 'solid', color: 'rgb(0, 0, 0)', offset: '2px' });
    expect(await readFocus('#studio-typed-root .code-input textarea[aria-label="File contents"]')).toEqual({ width: '2px', line: 'solid', color: 'rgb(0, 0, 0)', offset: '2px' });
});

test('final_a11y_targets_headings_focus', async ({ page }) => {
    for (const [width, height] of [[390, 844], [1024, 900], [1440, 900]] as const) {
        await openTypedAt(page, width, height);
        const root = page.locator('#studio-typed-root');
        await expect(page.locator('a.skip-link'), `${width}: exactly one mode-correct skip link`).toHaveCount(1);
        await expect(page.locator('a.skip-link')).toHaveAttribute('href', '#harness-main');
        await expect(root.locator('a.skip-link')).toHaveCount(0);
        const rail = root.locator('[data-region="task-rail"]');
        if (await rail.isHidden()) {
            await root.locator('[data-action="toggle-projects"]').click();
            await expect(rail).toBeVisible();
        }
        const navigation = root.locator('nav[aria-label="Studio"]');
        for (const label of ['Agent', 'Workspace', 'Changes', 'Terminal', 'Models', 'Guide']) {
            await navigation.getByRole('button', { name: label, exact: true }).click();
            await expect(root.locator('main h1:visible'), `${width} ${label}: one h1`).toHaveCount(1);
            expectWellFormed(await visibleHeadings(page, '#studio-typed-root main'), `${width} ${label}`);
            expect(await targetViolations(page, 24), `${width} ${label}: 24px targets`).toEqual([]);
        }
        expectRailWellFormed(await visibleHeadings(page, '#studio-typed-root aside.project-sidebar'));
        await navigation.getByRole('button', { name: 'Agent', exact: true }).click();

        const prompt = promptLocator(page);
        await prompt.focus();
        const composerFocus = await root.locator('.agent-composer').evaluate((element) => {
            const style = getComputedStyle(element);
            return { width: Number.parseFloat(style.outlineWidth), line: style.outlineStyle, offset: style.outlineOffset };
        });
        expect(composerFocus.line, `${width}: composer focus must be visible`).toBe('solid');
        expect(composerFocus.width, `${width}: composer focus width`).toBeGreaterThanOrEqual(2);
        expect(composerFocus.offset, `${width}: composer focus offset`).not.toBe('0px');

        const settingsButton = root.locator('.session-toolbar-actions button[aria-label="Settings"]');
        await settingsButton.click();
        const settings = page.getByRole('dialog', { name: 'Studio settings', exact: true });
        await expect(settings).toBeVisible();
        expect(await page.evaluate(() => Boolean(document.activeElement?.closest('dialog'))), `${width}: focus enters the dialog`).toBe(true);
        expect(await targetViolations(page, 24, 'dialog[open]'), `${width}: dialog targets`).toEqual([]);
        await page.keyboard.press('Escape');
        await expect(settings).toBeHidden();
        await expect(settingsButton, `${width}: focus returns to the invoker`).toBeFocused();
        expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBe(0);
    }
});
