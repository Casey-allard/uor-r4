import { expect, type Page } from '@playwright/test';

export async function openTyped(page: Page, theme: 'dark' | 'light' = 'dark') {
    await page.goto(`/?typed=1&theme=${theme}`);
    await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
}
