import type { ComponentChildren } from 'preact';
import type { AgentToolCallView, MessageDeliveryErrorView } from '../state/agent.js';

export interface AssistantTimelineEntry {
    kind: 'assistant';
    id: string;
    sequence: number;
    text: string;
    streaming: boolean;
    deliveryError: MessageDeliveryErrorView | null;
}

export interface ToolTimelineEntry {
    kind: 'tool';
    id: string;
    sequence: number;
    toolCall: AgentToolCallView;
}

export interface UnknownTimelineEntry {
    kind: 'unknown';
    id: string;
    sequence: number;
    type: string;
}

export interface UsageTimelineEntry {
    kind: 'usage';
    id: string;
    sequence: number;
    inputTokens: number;
    outputTokens: number;
    modelCalls: number;
}

export interface DeliveryTimelineEntry {
    kind: 'delivery';
    id: string;
    sequence: number;
    messageId: string | null;
    message: string;
}

export type TimelineEntry =
    | AssistantTimelineEntry
    | ToolTimelineEntry
    | UnknownTimelineEntry
    | UsageTimelineEntry
    | DeliveryTimelineEntry;

function toolStatusLabel(toolCall: AgentToolCallView): string {
    if (toolCall.status === 'requested') return 'Requested';
    if (toolCall.status === 'started') return 'Running';
    if (toolCall.status === 'failed') return 'Failed';
    switch (toolCall.resultStatus) {
        case 'ok': return 'Completed';
        case 'error': return 'Completed with error';
        case 'cancelled': return 'Cancelled';
        default: return 'Completed';
    }
}

export interface TimelineEventProps {
    event: TimelineEntry;
    children?: ComponentChildren;
}

export function TimelineEvent({ event, children }: TimelineEventProps) {
    if (event.kind === 'usage') {
        return <p class="timeline-event timeline-usage" data-timeline-kind="usage">
            Usage: {event.inputTokens} input tokens, {event.outputTokens} output tokens, {event.modelCalls} model calls
        </p>;
    }
    if (event.kind === 'assistant') {
        return <article class="timeline-event timeline-assistant" data-timeline-kind="assistant" data-message-id={event.id} aria-busy={event.streaming ? 'true' : 'false'}>
            <header class="timeline-event-header"><span class="message-role">Assistant</span>{event.streaming && <span class="timeline-streaming">Streaming</span>}</header>
            <div class="message-content">{event.text}</div>
            {event.deliveryError && <DeliveryRow error={event.deliveryError} />}
        </article>;
    }
    if (event.kind === 'tool') {
        const { toolCall } = event;
        return <article class="timeline-event timeline-tool" data-timeline-kind="tool" data-tool-call-id={toolCall.toolCallId} data-tool-status={toolCall.status}>
            <header class="timeline-event-header"><span class="message-tool">{toolCall.tool}</span><span class="timeline-tool-status">{toolStatusLabel(toolCall)}</span></header>
            {toolCall.argumentsPreview !== null && <p class="timeline-tool-request"><span>Request</span><code>{toolCall.argumentsPreview}</code></p>}
            {toolCall.outputPreview !== null && <p class="timeline-tool-result"><span>Result</span><code>{toolCall.outputPreview}</code></p>}
            {toolCall.status === 'failed' && <p class="timeline-tool-failure">{toolCall.code}: {toolCall.message}</p>}
            {children}
        </article>;
    }
    if (event.kind === 'unknown') {
        return <p class="timeline-event timeline-unknown" data-timeline-kind="unknown">
            Unrecognized event {event.type} (sequence {event.sequence}) was preserved and is not rendered as assistant or tool output.
        </p>;
    }
    return <DeliveryRow error={{ code: 'message_not_delivered', messageId: event.messageId, message: event.message }} />;
}

function DeliveryRow({ error }: { error: MessageDeliveryErrorView }) {
    return <aside class="timeline-delivery" data-timeline-kind="delivery" data-delivery-message-id={error.messageId ?? 'unknown'} role="note">
        <span>Delivery</span> {error.message}
    </aside>;
}
