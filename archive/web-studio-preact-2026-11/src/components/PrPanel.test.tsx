import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest';

import type { PrSummary } from './PrPanel.js';

let host: HTMLDivElement;

beforeEach(() => {
    host = document.createElement('div');
    document.body.appendChild(host);
    localStorage.clear();
});

afterEach(() => {
    render(null, host);
    host.remove();
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    localStorage.clear();
});

const summary: PrSummary = {
    title: 'Add digest-bound artifact previews',
    number: 2042,
    state: 'open',
    author: 'casey',
    repository: 'casey/uor-r4',
    sourceBranch: 'kimi/cursor-issue-sync',
    targetBranch: 'main',
    changedFiles: 7,
    url: 'https://github.com/casey/uor-r4/pull/2042',
    source: 'local',
};

async function renderPr(summaryValue: PrSummary | null, unavailableReason?: string): Promise<void> {
    const { PrPanel } = await import('./PrPanel.js');
    act(() => {
        render(
            <PrPanel
                summary={summaryValue}
                unavailableReason={unavailableReason}
            />,
            host,
        );
    });
}

describe('PrPanel', () => {
    test('pr_tab_is_read_only_link_without_integration', async () => {
        const fetchSpy = vi.fn();
        vi.stubGlobal('fetch', fetchSpy);

        await renderPr(summary);

        const panel = host.querySelector('[data-pr-panel]');
        expect(panel).not.toBeNull();
        expect(panel?.getAttribute('data-pr-read-only')).not.toBeNull();
        expect(host.querySelectorAll('h1')).toHaveLength(1);
        expect(host.textContent).toContain('Pull request');
        expect(host.textContent).toContain('Add digest-bound artifact previews');
        expect(host.textContent).toContain('#2042');
        expect(host.textContent).toContain('Read-only');
        expect(host.textContent).toContain('casey/uor-r4');
        expect(host.textContent).toContain('kimi/cursor-issue-sync');
        expect(host.textContent).toContain('main');

        const link = host.querySelector<HTMLAnchorElement>('a[data-pr-link]');
        expect(link).not.toBeNull();
        expect(link?.getAttribute('href')).toBe('https://github.com/casey/uor-r4/pull/2042');
        expect(link?.getAttribute('target')).toBe('_blank');
        expect(link?.getAttribute('rel')).toBe('noopener noreferrer');
        expect(link?.textContent).toContain('Open pull request');

        expect(fetchSpy).not.toHaveBeenCalled();
        expect(host.querySelector('form')).toBeNull();
        expect(host.querySelector('iframe')).toBeNull();
        expect(host.querySelector('script')).toBeNull();
        expect(localStorage.length).toBe(0);
    });

    test('no_merge_control_without_service_identity', async () => {
        await renderPr(summary);

        const controls = [...host.querySelectorAll('button, a, input, select, textarea')]
            .map((node) => `${node.tagName} ${node.textContent ?? ''} ${node.getAttribute('aria-label') ?? ''}`)
            .join(' ');
        expect(controls).not.toMatch(/\bmerge\b/i);
        expect(controls).not.toMatch(/\bapprove\b/i);
        expect(controls).not.toMatch(/\bclose pull request\b/i);
        expect(controls).not.toMatch(/\bsign in\b/i);
        expect(host.textContent).not.toMatch(/github connected/i);
        expect(host.textContent).not.toMatch(/signed in as/i);
        expect(host.textContent).not.toMatch(/service identity/i);
        expect(host.querySelector('[data-pr-integration]')).toBeNull();
        expect(host.querySelector('[data-action*="merge"]')).toBeNull();
    });

    test('missing_pr_summary_is_an_explicit_unavailable_state', async () => {
        await renderPr(null, 'No local or imported pull request summary is available.');

        expect(host.querySelector('[data-pr-unavailable]')?.textContent).toContain(
            'No local or imported pull request summary is available.',
        );
        expect(host.querySelector('a')).toBeNull();
        expect(host.textContent).toContain('does not connect');
        expect(host.querySelectorAll('h1')).toHaveLength(1);
    });

    test('unsafe_pr_links_are_withheld_not_rendered', async () => {
        const unsafe: PrSummary[] = [
            { ...summary, url: 'javascript:alert(1)' },
            { ...summary, url: 'http://github.com/casey/uor-r4/pull/2042' },
            { ...summary, url: 'https://user:pass@github.com/casey/uor-r4/pull/2042' },
            { ...summary, url: 'https://github.com/casey/uor-r4/pull/2042?token=secret' },
            { ...summary, url: 'https://github.com/casey/uor-r4/issues/2042' },
            { ...summary, url: 'file:///tmp/pr.html' },
        ];

        for (const value of unsafe) {
            await renderPr(value);
            expect(host.querySelector('a[data-pr-link]'), value.url).toBeNull();
            expect(host.querySelector('[data-pr-link-withheld]'), value.url).not.toBeNull();
            expect(host.textContent, value.url).not.toContain('javascript:');
            expect(host.textContent, value.url).not.toContain('token=secret');
        }
    });

    test('summary_text_is_rendered_as_text_not_markup', async () => {
        await renderPr({
            ...summary,
            title: '<img src=x onerror=alert(1)>Review',
            author: '<script>alert(1)</script>',
        });

        expect(host.querySelector('img')).toBeNull();
        expect(host.querySelector('script')).toBeNull();
        expect(host.textContent).toContain('<img src=x onerror=alert(1)>Review');
        expect(host.textContent).toContain('<script>alert(1)</script>');
    });
});

describe('PrPanel state truthfulness', () => {
    test('unknown_pr_state_is_not_reported_as_merged', async () => {
        await renderPr({
            ...summary,
            state: 'queued-for-review' as PrSummary['state'],
        });
        expect(host.textContent).toContain('State unavailable');
        expect(host.textContent).not.toContain('Merged');
    });
});
