import { test, expect } from '@playwright/test';
import { openTyped } from './support';

test('static preview refuses an interactive PTY without a native session service', async ({ page }) => {
    await openTyped(page);
    await page.locator('nav[aria-label="Studio"]').getByRole('button', { name: 'Terminal', exact: true }).click();
    await expect(page.locator('#studio-typed-root #terminalConsentState')).toContainText('Interactive PTY is unavailable in this browser build.');
    await expect(page.locator('#studio-typed-root').getByRole('button', { name: 'Open terminal', exact: true })).toBeDisabled();
    await expect(page.locator('#studio-typed-root').getByRole('button', { name: 'Run command', exact: true })).toHaveCount(0);
    await expect(page.locator('#studio-typed-root').getByLabel('Terminal command', { exact: true })).toHaveCount(0);
});

test('static mobile terminal keeps the same explicit unsupported state without overflow', async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 852 });
    await openTyped(page, 'light');
    await page.locator('[data-action="toggle-projects"]').click();
    await page.locator('nav[aria-label="Studio"]').getByRole('button', { name: 'Terminal', exact: true }).click();
    await expect(page.locator('#studio-typed-root #terminalConsentState')).toContainText('Interactive PTY is unavailable in this browser build.');
    await expect(page.locator('#studio-typed-root').getByRole('button', { name: 'Open terminal', exact: true })).toBeDisabled();
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
});
