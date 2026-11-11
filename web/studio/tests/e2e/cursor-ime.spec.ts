import { expect, test, type Page } from '@playwright/test';
import { openTypedAt } from './support';

interface CompositionEventRecord {
    type: string;
    data?: string;
    trusted?: boolean;
    key?: string;
    keyCode?: number;
    isComposing?: boolean;
    prevented?: boolean;
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

async function openAgentWithFixtureProvider(page: Page) {
    await openTypedAt(page, 1440);
    await selectFixtureProvider(page);
}

async function installSendSpy(page: Page) {
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
}

async function sentPrompts(page: Page): Promise<string[]> {
    return page.evaluate(() => (window as unknown as { __studioSendCalls: string[] }).__studioSendCalls);
}

async function installCompositionRecorder(page: Page) {
    await page.evaluate(() => {
        const target = window as unknown as { __compositionEvents: CompositionEventRecord[] };
        target.__compositionEvents = [];
        for (const type of ['compositionstart', 'compositionupdate', 'compositionend']) {
            window.addEventListener(type, (event) => {
                target.__compositionEvents.push({ type, data: (event as CompositionEvent).data, trusted: event.isTrusted });
            }, true);
        }
        const prompt = document.querySelector<HTMLTextAreaElement>('#studio-typed-root textarea[aria-label="Prompt"]');
        prompt?.addEventListener('keydown', (event) => {
            target.__compositionEvents.push({
                type: 'keydown',
                key: event.key,
                keyCode: event.keyCode,
                isComposing: event.isComposing,
                prevented: event.defaultPrevented,
            });
        }, true);
    });
}

async function compositionRecords(page: Page): Promise<CompositionEventRecord[]> {
    return page.evaluate(() => (window as unknown as { __compositionEvents: CompositionEventRecord[] }).__compositionEvents);
}

async function writeClipboard(page: Page, text: string) {
    await page.evaluate(async (value) => { await navigator.clipboard.writeText(value); }, text);
}

function promptLocator(page: Page) {
    return page.locator('#studio-typed-root textarea[aria-label="Prompt"]');
}

test('existing_ime_composition_regression', async ({ page }) => {
    await openAgentWithFixtureProvider(page);
    await installSendSpy(page);
    await installCompositionRecorder(page);
    const prompt = promptLocator(page);
    await prompt.fill('existing ');
    await prompt.focus();
    await prompt.evaluate((element) => element.setSelectionRange(9, 9));
    const cdp = await page.context().newCDPSession(page);
    await cdp.send('Input.imeSetComposition', { text: 'にほん', selectionStart: 3, selectionEnd: 3 });
    await expect.poll(async () => (await compositionRecords(page)).filter((record) => record.type === 'compositionstart').length).toBe(1);
    let records = await compositionRecords(page);
    expect(records.find((record) => record.type === 'compositionstart')?.trusted).toBe(true);
    expect(records.find((record) => record.type === 'compositionupdate')?.data).toBe('にほん');
    expect(await prompt.inputValue()).toBe('existing にほん');

    await cdp.send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13, nativeVirtualKeyCode: 13 });
    await cdp.send('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13, nativeVirtualKeyCode: 13 });
    await expect.poll(async () => (await compositionRecords(page)).filter((record) => record.type === 'keydown').length).toBeGreaterThan(0);
    records = await compositionRecords(page);
    const enter = records.find((record) => record.type === 'keydown' && record.key === 'Enter');
    expect(enter?.isComposing).toBe(true);
    expect(await sentPrompts(page)).toEqual([]);
    expect(await prompt.inputValue()).toBe('existing にほん');

    await cdp.send('Input.insertText', { text: '日本' });
    await expect(prompt).toHaveValue('existing 日本');
    expect(await prompt.evaluate((element) => element.selectionStart)).toBe('existing 日本'.length);
    await expect.poll(async () => (await compositionRecords(page)).filter((record) => record.type === 'compositionend').length).toBe(1);
    expect(await sentPrompts(page)).toEqual([]);
});

test('dead_key_composition_preserves_text', async ({ page }) => {
    await openAgentWithFixtureProvider(page);
    await installSendSpy(page);
    await installCompositionRecorder(page);
    const prompt = promptLocator(page);
    await prompt.fill('caf');
    await prompt.focus();
    await prompt.evaluate((element) => element.setSelectionRange(3, 3));
    const cdp = await page.context().newCDPSession(page);
    await cdp.send('Input.imeSetComposition', { text: '´', selectionStart: 1, selectionEnd: 1 });
    await cdp.send('Input.imeSetComposition', { text: 'é', selectionStart: 1, selectionEnd: 1 });
    await cdp.send('Input.insertText', { text: 'é' });
    await expect(prompt).toHaveValue('café');
    expect(await prompt.evaluate((element) => element.selectionStart)).toBe(4);
    expect(await sentPrompts(page)).toEqual([]);
    const records = await compositionRecords(page);
    expect(records.filter((record) => record.type === 'compositionstart' && record.trusted === true)).toHaveLength(1);
    const updates = records.filter((record) => record.type === 'compositionupdate' && record.trusted === true).map((record) => record.data);
    expect(updates[0]).toBe('´');
    expect(updates[1]).toBe('é');
    expect(updates.every((data, index) => index < 2 || data === 'é')).toBe(true);
});

test('rtl_paste_preserves_caret', async ({ page }) => {
    await openAgentWithFixtureProvider(page);
    await installSendSpy(page);
    await page.context().grantPermissions(['clipboard-read', 'clipboard-write'], { origin: 'http://127.0.0.1:4173' });
    const rtl = 'مرحبا بالعالم';
    await writeClipboard(page, rtl);
    const prompt = promptLocator(page);
    await prompt.fill('alpha omega');
    await prompt.focus();
    await prompt.evaluate((element) => element.setSelectionRange(6, 6));
    await page.keyboard.press('Meta+V');
    await expect(prompt).toHaveValue(`alpha ${rtl}omega`);
    expect(await prompt.evaluate((element) => element.selectionStart)).toBe(6 + rtl.length);
    expect(await sentPrompts(page)).toEqual([]);
});

test('long_paste_preserves_caret_and_scroll', async ({ page }) => {
    await openAgentWithFixtureProvider(page);
    await installSendSpy(page);
    await page.context().grantPermissions(['clipboard-read', 'clipboard-write'], { origin: 'http://127.0.0.1:4173' });
    const longText = Array.from({ length: 320 }, (_, index) => `line ${index} ${'x'.repeat(40)}`).join('\n');
    await writeClipboard(page, longText);
    const prompt = promptLocator(page);
    await prompt.fill('start ');
    await prompt.focus();
    await prompt.evaluate((element) => element.setSelectionRange(6, 6));
    await page.keyboard.press('Meta+V');
    await expect.poll(async () => (await prompt.inputValue()).length).toBe(`start ${longText}`.length);
    expect(await prompt.inputValue()).toBe(`start ${longText}`);
    const state = await prompt.evaluate((element) => ({
        caret: element.selectionStart,
        scrollTop: element.scrollTop,
        scrollHeight: element.scrollHeight,
        clientHeight: element.clientHeight,
    }));
    expect(state.caret).toBe(`start ${longText}`.length);
    expect(state.scrollHeight).toBeGreaterThan(state.clientHeight);
    expect(state.scrollTop).toBeGreaterThan(0);
    expect(state.scrollTop + state.clientHeight).toBeGreaterThanOrEqual(state.scrollHeight - 24);
    expect(await sentPrompts(page)).toEqual([]);
    await expect(page.locator('#studio-typed-root .agent-composer')).toBeVisible();
    expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBeLessThanOrEqual(0);
});

test('shift_enter_inserts_newline_at_the_real_caret', async ({ page }) => {
    await openAgentWithFixtureProvider(page);
    await installSendSpy(page);
    const prompt = promptLocator(page);
    await prompt.fill('abcd');
    await prompt.focus();
    await prompt.evaluate((element) => element.setSelectionRange(2, 2));
    await page.keyboard.press('Shift+Enter');
    await page.keyboard.type('X');
    await expect(prompt).toHaveValue('ab\nXcd');
    expect(await prompt.evaluate((element) => element.selectionStart)).toBe(4);
    expect(await sentPrompts(page)).toEqual([]);
});

test('plain_enter_submits_through_the_runtime_run_path', async ({ page }) => {
    await openTypedAt(page, 1440);
    await installSendSpy(page);
    const prompt = promptLocator(page);
    await prompt.fill('send control');
    await expect(page.locator('#studio-typed-root .send-button')).toBeEnabled();
    await prompt.focus();
    await page.keyboard.press('Enter');
    await expect(page.locator('#studio-typed-root [data-task-id]')).toHaveCount(1);
    await expect(page.locator('#studio-typed-root [data-task-id] .task-state')).toHaveText(/Running|Finished/);
    await expect(prompt).toHaveValue('');
    expect(await sentPrompts(page)).toEqual([]);
});
