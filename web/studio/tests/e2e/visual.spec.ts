import { test, expect } from '@playwright/test';
import { openTyped } from './support';

const surfaces = [
    { name: 'agent', target: null, ready: '.agent-workspace' }, { name: 'workspace', target: 'editor', ready: '.workspace-editor' },
    { name: 'changes', target: 'diff', ready: '.changes-panel' }, { name: 'terminal', target: 'terminal', ready: '.terminal-panel' },
    { name: 'models', target: null, ready: '.models-panel' }, { name: 'guide', target: null, ready: '#harness-main h1' },
    { name: 'split', target: null, ready: '.pane-secondary' },
] as const;

for (const theme of ['dark', 'light'] as const) for (const viewport of [{ name: 'desktop', width: 1440, height: 900 }, { name: 'mobile', width: 390, height: 852 }]) {
    for (const surface of surfaces) test(`capture ${surface.name} ${theme} ${viewport.name}`, async ({ page }, testInfo) => {
        await page.setViewportSize({ width: viewport.width, height: viewport.height }); await openTyped(page, theme);
        if (surface.target) await page.locator(`[data-pane-target="${surface.target}"]`).click();
        if ((surface.name === 'models' || surface.name === 'guide') && viewport.name === 'mobile') await page.locator('[data-action="toggle-projects"]').click();
        if (surface.name === 'models') await page.locator('#studio-navigation').getByRole('button', { name: 'Models', exact: true }).click();
        if (surface.name === 'guide') await page.locator('#studio-navigation').getByRole('button', { name: 'Guide', exact: true }).click();
        if (surface.name === 'split') await page.locator('[data-action="split-pane"]').click();
        await expect(page.locator(surface.ready)).toBeVisible(); await expect(page.locator('.toast')).toHaveCount(0, { timeout: 6000 });
        await page.screenshot({ path: testInfo.outputPath(`${surface.name}-${theme}-${viewport.name}.png`), fullPage: true });
    });
    test(`capture legacy ${theme} ${viewport.name}`, async ({ page }, testInfo) => {
        await page.setViewportSize({ width: viewport.width, height: viewport.height }); await page.goto(`/?theme=${theme}`);
        await expect(page.locator('#app')).toBeVisible(); await expect(page.locator('.toast')).toHaveCount(0, { timeout: 6000 });
        await page.screenshot({ path: testInfo.outputPath(`legacy-${theme}-${viewport.name}.png`), fullPage: true });
    });
}
