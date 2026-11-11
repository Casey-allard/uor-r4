import { expect, type Page } from '@playwright/test';

export async function openTyped(page: Page, theme: 'dark' | 'light' = 'dark') {
    await page.goto(`/?typed=1&theme=${theme}`);
    await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
}

export async function openTypedAt(page: Page, width: number, height = 900, theme: 'dark' | 'light' = 'dark') {
    await page.setViewportSize({ width, height });
    await openTyped(page, theme);
}
