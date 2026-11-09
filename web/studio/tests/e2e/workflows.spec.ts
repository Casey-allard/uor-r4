import { test, expect } from '@playwright/test';
import { openTyped } from './support';

test('opens a real new-project workflow instead of a decorative button', async ({ page }) => {
    await openTyped(page); await page.locator('[data-action="new-project"]').click();
    await expect(page.getByRole('dialog', { name: 'New project' })).toBeVisible();
    await expect(page.getByLabel('Project name')).toBeVisible(); await expect(page.getByLabel('Project folder')).toBeVisible();
});

test('creates and resumes independent sessions through the shared shell', async ({ page }) => {
    await openTyped(page);
    const sessions = page.locator('.session-list [data-session-id]');
    await page.locator('[data-action="new-session"]').click(); await expect(sessions).toHaveCount(1);
    const firstId = await sessions.first().getAttribute('data-session-id');
    await page.locator('[data-action="new-session"]').click(); await expect(sessions).toHaveCount(2);
    expect(firstId).not.toBeNull();
    await sessions.first().locator('[data-action="resume-session"]').click(); await expect(sessions.first()).toHaveAttribute('aria-current', 'true');
    await sessions.nth(1).locator('[data-action="resume-session"]').click(); await expect(sessions.nth(1)).toHaveAttribute('aria-current', 'true');
});
