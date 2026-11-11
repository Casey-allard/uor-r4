import { expect, test } from '@playwright/test';
import { openTyped } from './support';

const FOUR_MIB = 4 * 1024 * 1024;

test('the actual attachment store accepts two valid documents totaling four MiB plus one byte', async ({ page }) => {
    await openTyped(page);
    const root = page.locator('#studio-typed-root');
    const picker = root.locator('input[type="file"][aria-label="Attach files"]');

    await picker.setInputFiles({ name: 'four-mib.md', mimeType: 'text/markdown', buffer: Buffer.alloc(FOUR_MIB, 'a') });
    await expect(root.locator('.composer-attachment-chip').filter({ hasText: 'four-mib.md' })).toHaveCount(1);

    await picker.setInputFiles({ name: 'one-byte.md', mimeType: 'text/markdown', buffer: Buffer.from('b') });
    const handle = await page.waitForFunction(() => {
        const studio = (window as unknown as { __studioUi: { getAttachments(): Array<{ name: string }> } }).__studioUi;
        const names = studio.getAttachments().map((attachment) => attachment.name);
        const toast = document.getElementById('toastRegion')?.textContent || '';
        if (/Attachments exceed the (4|25) MiB total limit/.test(toast)) return { state: 'limit', names, toast };
        if (names.includes('four-mib.md') && names.includes('one-byte.md')) return { state: 'accepted', names, toast };
        return null;
    }, null, { timeout: 5000 });
    const observed = await handle.jsonValue() as { state: string; names: string[]; toast: string };

    expect(observed.state, JSON.stringify(observed)).toBe('accepted');
    expect(observed.names, JSON.stringify(observed)).toEqual(['four-mib.md', 'one-byte.md']);
    expect(observed.toast).not.toContain('Attachments exceed the 4 MiB total limit; remove a file first.');
    expect(observed.toast).not.toContain('Attachments exceed the 25 MiB total limit');
    await expect(root.locator('.composer-attachment-chip').filter({ hasText: 'one-byte.md' })).toHaveCount(1);
});
