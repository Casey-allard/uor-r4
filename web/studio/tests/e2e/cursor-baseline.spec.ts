import { test, expect } from '@playwright/test';
import { openTyped, openTypedAt } from './support';

const protectedSelectors = [
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

test('protected_selectors_are_preserved', async ({ page }) => {
    await openTyped(page);
    const root = page.locator('#studio-typed-root');
    await expect(page.locator('#app')).toBeHidden();
    for (const selector of protectedSelectors) {
        await expect(root.locator(selector).first(), `${selector} must be visible in the typed root`).toBeVisible();
    }
    await expect(root.getByRole('button', { name: 'Send', exact: true })).toBeVisible();
});

test('only_one_skip_link_and_it_targets_active_main', async ({ page }) => {
    await openTyped(page);
    const skip = page.locator('a.skip-link');
    await expect(skip).toHaveCount(1);
    await expect(page.locator('#studio-typed-root a.skip-link')).toHaveCount(0);
    await expect(skip).toHaveAttribute('href', '#harness-main');
    await expect(page.locator('#harness-main')).toBeVisible();
    await skip.focus();
    await expect(skip).toBeVisible();
});

test('legacy_selection_keeps_main_column_skip_target', async ({ page }) => {
    for (const selection of ['?legacy=1', '?typed=0']) {
        await page.goto(`/${selection}`);
        await expect(page.locator('#app')).toBeVisible();
        await expect(page.locator('#studio-typed-root')).toBeHidden();
        await expect(page.locator('a.skip-link')).toHaveCount(1);
        await expect(page.locator('a.skip-link')).toHaveAttribute('href', '#mainColumn');
    }
});

test('1023_defaults_closed', async ({ page }) => {
    await openTypedAt(page, 1023);
    const rail = page.locator('#studio-typed-root [data-region="task-rail"]');
    const toggle = page.locator('#studio-typed-root [data-action="toggle-projects"]');
    await expect(rail).toBeHidden();
    await expect(toggle).toHaveAttribute('aria-expanded', 'false');
    await toggle.click();
    await expect(rail).toBeVisible();
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');
    await toggle.click();
    await expect(rail).toBeHidden();
    await toggle.click();
    await expect(rail).toBeVisible();
});

test('1024_defaults_open', async ({ page }) => {
    await openTypedAt(page, 1024);
    const rail = page.locator('#studio-typed-root [data-region="task-rail"]');
    const toggle = page.locator('#studio-typed-root [data-action="toggle-projects"]');
    await expect(rail).toBeVisible();
    await toggle.click();
    await expect(rail).toBeHidden();
    await expect(toggle).toHaveAttribute('aria-expanded', 'false');
    await page.waitForTimeout(200);
    await expect(rail).toBeHidden();
    await toggle.click();
    await expect(rail).toBeVisible();
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');
});

test('typed_mount_closes_existing_legacy_palette', async ({ page }) => {
    await page.addInitScript(() => {
        let value: unknown;
        (window as unknown as { __seededPaletteOpened?: boolean }).__seededPaletteOpened = false;
        Object.defineProperty(window, '__studioUi', {
            configurable: true,
            get: () => value,
            set: (next: unknown) => {
                value = next;
                const ui = next as { openPalette?: () => void } | null;
                if (!ui || typeof ui.openPalette !== 'function') return;
                ui.openPalette();
                (window as unknown as { __seededPaletteOpened?: boolean }).__seededPaletteOpened = document.getElementById('commandPalette')?.hidden === false;
            },
        });
    });
    await openTyped(page);
    await page.waitForTimeout(300);
    expect(await page.evaluate(() => (window as unknown as { __seededPaletteOpened?: boolean }).__seededPaletteOpened)).toBe(true);
    await expect(page.locator('#commandPalette')).toBeHidden();
});

test('late_legacy_ui_publication_resets_palette_state', async ({ page }) => {
    await openTyped(page);
    const state = await page.evaluate(async () => {
        const ui = (window as unknown as {
            __studioUi: { openPalette(): void; closePalette(restoreFocus?: boolean): void };
        }).__studioUi;
        const palette = document.getElementById('commandPalette') as HTMLElement;
        ui.openPalette();
        const visibleBeforeDispatch = !palette.hidden;
        window.dispatchEvent(new CustomEvent('uor-studio-v2-ready'));
        const closedByReadiness = palette.hidden;
        await new Promise<void>((resolve) => setTimeout(resolve, 0));
        ui.openPalette();
        const paletteOpenWasReset = !palette.hidden;
        ui.closePalette(false);
        return { visibleBeforeDispatch, closedByReadiness, paletteOpenWasReset };
    });
    expect(state.visibleBeforeDispatch).toBe(true);
    expect(state.closedByReadiness).toBe(true);
    expect(state.paletteOpenWasReset).toBe(true);
    await expect(page.locator('#commandPalette')).toBeHidden();
});

test('typed_mount_clears_palette_hashes', async ({ page }) => {
    for (const hash of ['#command-palette', '#palette']) {
        await page.goto(`/?typed=1${hash}`);
        await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
        await page.waitForTimeout(250);
        expect(new URL(page.url()).hash).toBe('');
        await expect(page.locator('#commandPalette')).toBeHidden();
    }
});

test('legacy_palette_delayed_open_cannot_beat_guard', async ({ page }) => {
    await page.goto('/?typed=1#palette');
    await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
    await page.waitForTimeout(400);
    await expect(page.locator('#commandPalette')).toBeHidden();
    expect(new URL(page.url()).hash).toBe('');
    await page.keyboard.press('ControlOrMeta+k');
    await page.waitForTimeout(100);
    await expect(page.locator('#commandPalette')).toBeHidden();
});

test('legacy_query_keeps_app_visible_and_typed_root_hidden', async ({ page }) => {
    for (const selection of ['?legacy=1', '?typed=0']) {
        await page.goto(`/${selection}#command-palette`);
        await expect(page.locator('#app')).toBeVisible();
        await expect(page.locator('#studio-typed-root')).toBeHidden();
        await expect(page.locator('#commandPalette')).toBeVisible({ timeout: 2000 });
        await expect(page.locator('a.skip-link')).toHaveCount(1);
        await expect(page.locator('a.skip-link')).toHaveAttribute('href', '#mainColumn');
    }
});
