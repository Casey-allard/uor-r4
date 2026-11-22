import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { AgentApprovalView, AgentToolCallView } from '../state/agent.js';
import {
    TaskTimeline,
    type TimelineApproval,
    type TimelineEntry,
} from './TaskTimeline.js';

let host: HTMLDivElement;
beforeEach(() => { host = document.createElement('div'); document.body.appendChild(host); });
afterEach(() => { render(null, host); host.remove(); vi.restoreAllMocks(); });

function toolCall(overrides: Partial<AgentToolCallView> = {}): AgentToolCallView {
    return {
        toolCallId: 'tool_1',
        tool: 'write_file',
        status: 'completed',
        argumentsDigest: `sha256:${'a'.repeat(64)}`,
        argumentsPreview: '{"path":"notes.md"}',
        outputDigest: `sha256:${'b'.repeat(64)}`,
        outputPreview: 'wrote 12 bytes',
        resultStatus: 'ok',
        code: null,
        message: null,
        ...overrides,
    };
}

function approval(overrides: Partial<AgentApprovalView> = {}): AgentApprovalView {
    return {
        approvalId: 'apr_1111111111111111',
        toolCallId: 'tool_1',
        tool: 'write_file',
        scope: 'tool:write_file',
        requestDigest: `sha256:${'c'.repeat(64)}`,
        expiresAtMs: Date.UTC(2026, 10, 9, 12, 10, 0),
        decision: null,
        ...overrides,
    };
}

const unavailable: TimelineApproval = { approval: approval(), hostIdentity: null };

function renderTimeline(events: TimelineEntry[], approvals: TimelineApproval[] = [], onResolve = vi.fn()) {
    act(() => { render(<TaskTimeline events={events} approvals={approvals} onResolve={onResolve} />, host); });
    return onResolve;
}

describe('TaskTimeline', () => {
    it('renders events in sequence order rather than array order', () => {
        renderTimeline([
            { kind: 'assistant', id: 'msg_2', sequence: 4, text: 'Second message', streaming: false, deliveryError: null },
            { kind: 'tool', id: 'tool_1', sequence: 2, toolCall: toolCall() },
            { kind: 'assistant', id: 'msg_1', sequence: 1, text: 'First message', streaming: false, deliveryError: null },
        ]);
        const kinds = Array.from(host.querySelectorAll<HTMLElement>('[data-timeline-kind]'))
            .map((element) => element.dataset.timelineKind);
        expect(kinds).toEqual(['assistant', 'tool', 'assistant']);
        expect(host.textContent?.indexOf('First message')).toBeLessThan(host.textContent!.indexOf('Second message'));
    });

    it('keeps an unknown event payload out of the transcript and out of tool output', () => {
        renderTimeline([
            { kind: 'unknown', id: 'evt_2', sequence: 2, type: 'host.instruction' },
        ]);
        const unknown = host.querySelector('[data-timeline-kind="unknown"]')!;
        expect(unknown.textContent).toContain('host.instruction');
        expect(unknown.textContent).toContain('not rendered');
        expect(host.querySelector('[data-timeline-kind="assistant"]')).toBeNull();
        expect(host.querySelector('[data-timeline-kind="tool"]')).toBeNull();
        expect(host.querySelector('script')).toBeNull();
    });

    it('pairs an approval with its tool request and states tool, scope, expiry and host', () => {
        renderTimeline(
            [{ kind: 'tool', id: 'tool_1', sequence: 2, toolCall: toolCall({ status: 'requested', resultStatus: null, outputPreview: null, outputDigest: null }) }],
            [unavailable],
        );
        const tool = host.querySelector('[data-timeline-kind="tool"]')!;
        const card = tool.querySelector('[data-approval-id="apr_1111111111111111"]')!;
        expect(card).not.toBeNull();
        expect(card.textContent).toContain('write_file');
        expect(card.textContent).toContain('tool:write_file');
        expect(card.textContent).toContain('2026-11-09T12:10:00.000Z');
        expect(card.textContent).toContain('Host: not connected');
        expect(card.textContent).toContain('This action needs host approval. Approve it in the local companion.');
        for (const name of ['Approve', 'Deny']) {
            const button = Array.from(card.querySelectorAll('button')).find((item) => item.textContent === name)!;
            expect(button.disabled, `${name} must stay disabled without an approval host`).toBe(true);
        }
    });

    it('resolves through the host path only when the host advertises approval authority', () => {
        const onResolve = renderTimeline(
            [{ kind: 'tool', id: 'tool_1', sequence: 2, toolCall: toolCall() }],
            [{ approval: approval(), hostIdentity: { id: 'Local companion', canApprove: true } }],
        );
        const card = host.querySelector('[data-approval-id]')!;
        expect(card.textContent).toContain('Host: Local companion');
        const approve = Array.from(card.querySelectorAll('button')).find((item) => item.textContent === 'Approve')!;
        expect(approve.disabled).toBe(false);
        act(() => { approve.click(); });
        expect(onResolve).toHaveBeenCalledWith('apr_1111111111111111', 'approved');
    });

    it('renders a message delivery error without offering a retry control', () => {
        renderTimeline([
            {
                kind: 'delivery',
                id: 'delivery_msg_1',
                sequence: 3,
                messageId: 'msg_1f2e3d4c5b6a7988',
                message: 'message msg_1f2e3d4c5b6a7988 was not delivered (cancelled)',
            },
        ]);
        const row = host.querySelector('[data-timeline-kind="delivery"]')!;
        expect(row.textContent).toContain('msg_1f2e3d4c5b6a7988');
        expect(row.querySelector('button')).toBeNull();
    });

    it('renders usage only when a usage entry was emitted', () => {
        renderTimeline([{ kind: 'assistant', id: 'msg_1', sequence: 1, text: 'Hello', streaming: false, deliveryError: null }]);
        expect(host.querySelector('[data-timeline-kind="usage"]')).toBeNull();
        renderTimeline([
            { kind: 'assistant', id: 'msg_1', sequence: 1, text: 'Hello', streaming: false, deliveryError: null },
            { kind: 'usage', id: 'usage_2', sequence: 2, inputTokens: 12, outputTokens: 34, modelCalls: 2 },
        ]);
        const usage = host.querySelector('[data-timeline-kind="usage"]')!;
        expect(usage.textContent).toContain('12 input tokens');
        expect(usage.textContent).toContain('34 output tokens');
        expect(usage.textContent).toContain('2 model calls');
    });

    it('states an honest empty transcript', () => {
        renderTimeline([]);
        expect(host.textContent).toContain('No runtime events for this task yet.');
    });
});

describe('restored approval availability', () => {
    it('keeps_a_restored_approval_read_only_when_no_active_owner_can_resolve_it', () => {
        renderTimeline([], [{
            approval: approval(),
            hostIdentity: { id: 'Local companion', canApprove: true },
            resolvable: false,
        }]);
        const card = host.querySelector('[data-approval-id="apr_1111111111111111"]')!;
        const approve = Array.from(card.querySelectorAll('button')).find((button) => button.textContent === 'Approve')!;
        const deny = Array.from(card.querySelectorAll('button')).find((button) => button.textContent === 'Deny')!;
        expect(approve.disabled).toBe(true);
        expect(deny.disabled).toBe(true);
        expect(card.textContent).toContain('no active run owner');
        expect(card.textContent).toContain('Stop the restored run before starting a new run');
    });
});
