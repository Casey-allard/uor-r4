import { test, expect } from '@playwright/test';
import { openTyped } from './support';

test('default URL enters the typed workbench', async ({ page }) => {
    await page.goto('/?publish=cutover#theme=light');
    await expect(page.locator('.harness-shell')).toBeVisible();
    await expect(page.locator('.agent-workspace')).toBeVisible();
    await expect(page).toHaveURL('/?publish=cutover#theme=light');
    await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
    await expect(page.locator('#app')).toBeHidden();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
});

test('explicit legacy selections retain the legacy interface', async ({ page }) => {
    for (const selection of ['?legacy=1', '?typed=0']) {
        await page.goto(`/${selection}`);
        await expect(page).toHaveURL(`/${selection}`);
        await expect(page.locator('#app')).toBeVisible();
        await expect(page.locator('#studio-typed-root')).toBeHidden();
    }
});

test('uses the approved neutral workbench palette instead of the green placeholder', async ({ page }) => {
    await openTyped(page);
    const shell = page.locator('.harness-shell');
    await expect(shell).toBeVisible();
    const tokens = await shell.evaluate((element) => { const style = getComputedStyle(element); return { background: style.getPropertyValue('--harness-bg').trim(), accent: style.getPropertyValue('--harness-accent').trim(), panel: style.getPropertyValue('--harness-panel').trim() }; });
    expect(tokens.background).toBe('#141413'); expect(tokens.accent).toBe('#d97757');
    expect(Object.values(tokens)).not.toContain('#111512'); expect(Object.values(tokens)).not.toContain('#1a211b'); expect(Object.values(tokens)).not.toContain('#8fd3a7');
});

test('renders the project, session, transcript, composer, and pane landmarks', async ({ page }) => {
    await openTyped(page);
    for (const selector of ['.project-sidebar', '#studio-typed-root .session-toolbar', '.session-list', '.workspace-grid', '.pane-dock', '.agent-transcript', '.agent-composer']) await expect(page.locator(selector)).toBeVisible();
});

test('exposes environment, project, permission, and model selectors', async ({ page }) => {
    await openTyped(page);
    for (const control of ['environment', 'project', 'permission', 'model']) await expect(page.locator(`[data-control="${control}"]`)).toBeVisible();
});

test('switches between real editor, diff, terminal, and explicit browser-unavailable panes', async ({ page }) => {
    await openTyped(page);
    for (const pane of ['editor', 'diff', 'terminal']) { await page.locator(`[data-pane-target="${pane}"]`).click(); await expect(page.locator(`[data-pane="${pane}"]`)).toBeVisible(); }
    await page.locator('[data-pane-target="browser"]').click();
    await expect(page.locator('#studio-typed-root [data-pane="browser"]')).toBeVisible();
    await expect(page.getByText('Browser preview requires a harness service')).toBeVisible();
});

test('opens and dismisses the mobile project drawer with Escape', async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 852 }); await openTyped(page);
    await page.locator('[data-action="toggle-projects"]').click(); await expect(page.locator('.project-sidebar')).toBeVisible();
    await page.keyboard.press('Escape'); await expect(page.locator('.project-sidebar')).toBeHidden();
});

test('splits chat and editor with a keyboard-operable pane splitter', async ({ page }) => {
    await openTyped(page); await page.locator('[data-action="split-pane"]').click();
    await expect(page.locator('[data-pane="chat"]')).toBeVisible(); await expect(page.locator('[data-pane="editor"]')).toBeVisible();
    const splitter = page.getByRole('separator', { name: 'Resize panes' }); await expect(splitter).toBeVisible();
    const before = await splitter.getAttribute('aria-valuenow'); await splitter.focus(); await page.keyboard.press('ArrowRight');
    expect(await splitter.getAttribute('aria-valuenow')).not.toBe(before);
});

test('shows the prompt editor inside the composer', async ({ page }) => {
    await openTyped(page);
    const prompt = page.locator('.agent-composer textarea[aria-label="Prompt"]'); await expect(prompt).toBeVisible();
    await prompt.fill('Inspect the workbench'); await expect(prompt).toHaveValue('Inspect the workbench');
    await expect(page.getByRole('button', { name: 'Send', exact: true })).toBeVisible();
});

test('applies the light theme to the shell instead of retaining dark tokens', async ({ page }) => {
    await openTyped(page, 'light');
    const theme = await page.locator('.harness-shell').evaluate((element) => ({ root: document.documentElement.dataset.theme, background: getComputedStyle(element).getPropertyValue('--harness-bg').trim(), text: getComputedStyle(element).getPropertyValue('--harness-text').trim() }));
    expect(theme.root).toBe('light'); expect(theme.background).toBe('#faf9f5'); expect(theme.text).toBe('#141413');
});

test('mobile split shows the secondary surface and close restores the primary surface', async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 852 }); await openTyped(page);
    await page.locator('[data-action="split-pane"]').click(); await expect(page.locator('.pane-secondary')).toBeVisible(); await expect(page.locator('.pane-primary')).toBeHidden();
    await page.locator('[data-action="close-split"]').click(); await expect(page.locator('.pane-primary')).toBeVisible(); await expect(page.locator('.pane-secondary')).toHaveCount(0);
});
