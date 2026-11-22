import { useState } from 'preact/hooks';

import { Icon } from './ui/Icon.js';

export type PrSummaryState = 'open' | 'draft' | 'closed' | 'merged';
export type PrSummarySource = 'local' | 'imported' | 'companion';

/**
 * A read-only pull-request summary supplied by the page's local state, an
 * imported record or the local companion. It is never fetched by this panel.
 */
export interface PrSummary {
    title: string;
    number?: number;
    state?: PrSummaryState;
    author?: string;
    repository?: string;
    sourceBranch?: string;
    targetBranch?: string;
    changedFiles?: number;
    url?: string;
    source?: PrSummarySource;
}

export interface PrPanelProps {
    summary?: PrSummary | null;
    unavailableReason?: string | null;
}

let nextPrPanelId = 0;

/**
 * Accepts only an absolute HTTPS link to a pull or merge-request page with no
 * credentials, port, query, fragment or non-PR path. The result is rendered as
 * a normal link activated by the user; the panel never requests it itself.
 */
export function safePullRequestLink(value: unknown): string | null {
    if (typeof value !== 'string' || value.length === 0 || value.length > 2048) {
        return null;
    }
    let url: URL;
    try {
        url = new URL(value);
    } catch {
        return null;
    }
    if (url.protocol !== 'https:') return null;
    if (url.username !== '' || url.password !== '') return null;
    if (url.port !== '') return null;
    if (url.search !== '' || url.hash !== '') return null;
    if (url.hostname === 'localhost' || url.hostname.endsWith('.local')) {
        return null;
    }
    const pullRequestPath =
        /^\/[^/]+\/[^/]+\/pull\/\d+\/?$/.test(url.pathname) ||
        /^\/[^/]+\/[^/]+\/-\/merge_requests\/\d+\/?$/.test(url.pathname);
    if (!pullRequestPath) return null;
    return url.href;
}

function sourceLabel(source: PrSummarySource | undefined): string {
    if (source === 'imported') return 'Imported summary';
    if (source === 'companion') return 'Companion summary';
    return 'Local summary';
}

function stateLabel(state: PrSummaryState | undefined): string {
    switch (state) {
        case undefined: return 'State not supplied';
        case 'open': return 'Open';
        case 'draft': return 'Draft';
        case 'closed': return 'Closed';
        case 'merged': return 'Merged';
        default: return 'State unavailable (not reported as merged)';
    }
}

export function PrPanel({ summary = null, unavailableReason = null }: PrPanelProps) {
    const [id] = useState(() => `pr-panel-${++nextPrPanelId}`);
    const headingId = `${id}-heading`;
    const link = summary === null ? null : safePullRequestLink(summary.url);
    const linkWithheld =
        summary !== null && summary.url !== undefined && link === null;

    return <section class="desktop-panel pr-panel" aria-labelledby={headingId} data-pr-panel data-pr-read-only>
        <header class="panel-heading">
            <div>
                <h1 id={headingId}>Pull request</h1>
                <p>Read-only summary. This local session does not connect to a hosting service, authenticate, or change a pull request.</p>
            </div>
            <span class="panel-icon" aria-hidden="true"><Icon name="changes" /></span>
        </header>
        {summary === null
            ? <p class="panel-summary" data-pr-unavailable>
                {unavailableReason ?? 'No pull request summary is available in this local session.'}
            </p>
            : <article class="pr-summary" data-pr-summary aria-labelledby={`${id}-title`}>
                <h2 id={`${id}-title`}>{summary.title}</h2>
                <p class="pr-summary-source">{sourceLabel(summary.source)} · Read-only</p>
                <dl>
                    {summary.repository !== undefined && <><dt>Repository</dt><dd style={{ overflowWrap: 'anywhere' }}>{summary.repository}</dd></>}
                    {summary.number !== undefined && <><dt>Number</dt><dd>#{summary.number}</dd></>}
                    <dt>State</dt><dd>{stateLabel(summary.state)}</dd>
                    {summary.author !== undefined && <><dt>Author</dt><dd style={{ overflowWrap: 'anywhere' }}>{summary.author}</dd></>}
                    {summary.sourceBranch !== undefined && <><dt>Source branch</dt><dd style={{ overflowWrap: 'anywhere' }}>{summary.sourceBranch}</dd></>}
                    {summary.targetBranch !== undefined && <><dt>Target branch</dt><dd style={{ overflowWrap: 'anywhere' }}>{summary.targetBranch}</dd></>}
                    {summary.changedFiles !== undefined && <><dt>Changed files</dt><dd>{summary.changedFiles}</dd></>}
                </dl>
                {linkWithheld && <p class="pr-link-withheld" role="status" data-pr-link-withheld>
                    This summary link was withheld because it is not a safe pull-request URL.
                </p>}
                {link !== null && <p class="pr-link"><a href={link} target="_blank" rel="noopener noreferrer" data-pr-link>Open pull request<span class="visually-hidden"> in a new tab</span></a></p>}
            </article>}
    </section>;
}
