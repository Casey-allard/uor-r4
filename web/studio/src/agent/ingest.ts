import {
    AGENT_EVENT_TYPES,
    MAX_EVENT_BYTES,
    type AgentErrorCode,
} from './generated/agentContract.js';
import {
    AgentProtocolError,
    parseAgentEventWire,
    type AgentAppendResult,
    type AgentIncomingEvent,
} from '../state/agent.js';

const KNOWN_EVENT_TYPES: ReadonlySet<string> = new Set(AGENT_EVENT_TYPES);
const MAX_FRAMES_PER_PUSH = 64;
export const MAX_INGEST_FRAMES_PER_PUSH = MAX_FRAMES_PER_PUSH;
const INITIAL_BUFFER_BYTES = 64;

export type IngestOutcomeKind =
    | 'applied'
    | 'duplicate'
    | 'rejected'
    | 'unknown';

// Four explicit outcomes for one ingested frame. `unknown` still means the
// frame was applied and preserved by the reducer; it only reports that the
// event type is not a known rendering type, so callers must not project it as
// assistant text, a tool result or a completion.
export class IngestOutcome {
    readonly kind: IngestOutcomeKind;
    readonly event: AgentIncomingEvent | null;
    private readonly errorCode: AgentErrorCode | null;
    private readonly resync: boolean;

    private constructor(
        kind: IngestOutcomeKind,
        errorCode: AgentErrorCode | null,
        event: AgentIncomingEvent | null,
        resync: boolean,
    ) {
        this.kind = kind;
        this.errorCode = errorCode;
        this.event = event;
        this.resync = resync;
    }

    static applied(event: AgentIncomingEvent | null = null): IngestOutcome {
        return new IngestOutcome('applied', null, event, false);
    }

    static duplicate(event: AgentIncomingEvent | null = null): IngestOutcome {
        return new IngestOutcome('duplicate', null, event, false);
    }

    static rejected(
        code: AgentErrorCode,
        requiresResync = false,
    ): IngestOutcome {
        return new IngestOutcome('rejected', code, null, requiresResync);
    }

    static unknown(event: AgentIncomingEvent | null = null): IngestOutcome {
        return new IngestOutcome('unknown', null, event, false);
    }

    code(): AgentErrorCode | null {
        return this.errorCode;
    }

    // Carried from the store's own resync decision (or from an explicit cursor
    // expiry), never re-derived from the error-code name. A terminal-run
    // out_of_order and a cross-task rejection cannot be repaired by resyncing
    // this worker and therefore report false.
    get requiresResync(): boolean {
        return this.resync;
    }
}

// Amortized byte accumulator. Appends double the capacity and a completed
// frame compacts the remaining partial once, so a frame split across many
// chunks costs O(n) copied bytes instead of O(n²).
export class NdjsonBuffer {
    private data = new Uint8Array(INITIAL_BUFFER_BYTES);
    private end = 0;
    private scan = 0;

    copyBytes = 0;
    growths = 0;

    get length(): number {
        return this.end;
    }

    get capacity(): number {
        return this.data.length;
    }

    get scanned(): number {
        return this.scan;
    }

    append(bytes: Uint8Array): void {
        if (bytes.length === 0) return;
        const required = this.end + bytes.length;
        if (required > this.data.length) {
            let capacity = this.data.length;
            while (capacity < required) capacity *= 2;
            const next = new Uint8Array(capacity);
            next.set(this.data.subarray(0, this.end));
            this.copyBytes += this.end;
            this.data = next;
            this.growths += 1;
        }
        this.data.set(bytes, this.end);
        this.copyBytes += bytes.length;
        this.end = required;
    }

    view(): Uint8Array {
        return this.data.subarray(0, this.end);
    }

    setScanned(value: number): void {
        this.scan = Math.min(Math.max(0, value), this.end);
    }

    compact(from: number): void {
        if (from <= 0) return;
        const remaining = this.end - from;
        if (remaining > 0) this.data.copyWithin(0, from, this.end);
        this.copyBytes += remaining;
        this.end = remaining;
        this.scan = remaining;
    }

    reset(): void {
        this.end = 0;
        this.scan = 0;
    }

    toUint8Array(): Uint8Array {
        return this.data.slice(0, this.end);
    }
}

export interface DecodedFrameBatch {
    frames: string[];
    buffer: NdjsonBuffer;
    error: AgentErrorCode | null;
}

function exceedsFrameLimit(bytes: Uint8Array): boolean {
    let count = 0;
    for (let index = 0; index < bytes.length; index += 1) {
        if (bytes[index] === 0x0a) {
            count += 1;
            if (count > MAX_FRAMES_PER_PUSH) return true;
        }
    }
    return false;
}

function decodeLine(line: Uint8Array): string | null {
    try {
        const decoded = new TextDecoder('utf-8', { fatal: true }).decode(line);
        return decoded.endsWith('\r') ? decoded.slice(0, -1) : decoded;
    } catch {
        return null;
    }
}

// Splits NDJSON bytes on complete LF frames and keeps the trailing partial
// frame buffered. A chunk may end inside a multi-byte UTF-8 code point; only
// complete lines are decoded, so the split is invisible to the frame text.
// More than 64 complete frames in one push are rejected before any frame text
// is allocated or returned. On a late malformed/oversized frame the complete
// prefix is returned with the error; the caller decides whether to apply it.
// The worker path applies prefix frames before reporting the error.
export function decodeFrame(
    bytes: Uint8Array,
    buffer: NdjsonBuffer = new NdjsonBuffer(),
): DecodedFrameBatch {
    if (exceedsFrameLimit(bytes)) {
        buffer.reset();
        return { frames: [], buffer, error: 'limit_exceeded' };
    }
    buffer.append(bytes);
    const view = buffer.view();
    const frames: string[] = [];
    let start = 0;
    let index = buffer.scanned;
    while (index < view.length) {
        if (view[index] !== 0x0a) {
            index += 1;
            continue;
        }
        if (index - start > MAX_EVENT_BYTES) {
            buffer.reset();
            return { frames, buffer, error: 'limit_exceeded' };
        }
        const decoded = decodeLine(view.subarray(start, index));
        if (decoded === null) {
            buffer.reset();
            return { frames, buffer, error: 'malformed_event' };
        }
        frames.push(decoded);
        start = index + 1;
        index = start;
    }
    if (buffer.length - start > MAX_EVENT_BYTES) {
        buffer.reset();
        return { frames, buffer, error: 'limit_exceeded' };
    }
    if (start > 0) buffer.compact(start);
    buffer.setScanned(buffer.length);
    return { frames, buffer, error: null };
}

// EOF never applies the buffered partial frame. A clean empty buffer has no
// outcome; any bytes without a terminating LF are truncated input.
export function decodeEof(
    buffer: NdjsonBuffer | Uint8Array,
): IngestOutcome | null {
    return buffer.length === 0
        ? null
        : IngestOutcome.rejected('truncated_frame');
}

export type IngestSink = (
    event: AgentIncomingEvent,
) => Promise<AgentAppendResult>;

export interface NdjsonIngestOptions {
    sink: IngestSink;
    taskId?: string;
    runId?: string;
}

function protocolCode(error: unknown): AgentErrorCode {
    if (error instanceof AgentProtocolError) return error.code;
    return 'malformed_event';
}

// One stream worker owns one byte buffer and one bound task/run identity.
// Events go through the caller's existing reducer/durable append sink; this
// class never applies or projects an event itself. Pushes are serialized so a
// concurrent transport cannot mutate the buffer while a sink is awaited.
// A halted worker stays halted (including after a gap or cursor_expired).
// Recovery is a new worker created after the runtime has verified a snapshot
// and resumed the run; do not reuse the halted worker.
export class NdjsonIngest {
    private readonly sink: IngestSink;
    private readonly taskId: string | undefined;
    private readonly runId: string | undefined;
    private readonly buffer = new NdjsonBuffer();
    private haltCode: AgentErrorCode | null = null;
    private haltRequiresResync = false;
    private gate: Promise<void> = Promise.resolve();

    constructor(options: NdjsonIngestOptions) {
        this.sink = options.sink;
        this.taskId = options.taskId;
        this.runId = options.runId;
    }

    get bufferedBytes(): Uint8Array {
        return this.buffer.toUint8Array();
    }

    get bufferedLength(): number {
        return this.buffer.length;
    }

    get copyBytes(): number {
        return this.buffer.copyBytes;
    }

    get bufferCapacity(): number {
        return this.buffer.capacity;
    }

    get halted(): AgentErrorCode | null {
        return this.haltCode;
    }

    push(bytes: Uint8Array): Promise<IngestOutcome[]> {
        const run = this.gate.then(() => this.pushInternal(bytes));
        this.gate = run.then(
            () => undefined,
            () => undefined,
        );
        return run;
    }

    async eof(): Promise<IngestOutcome | null> {
        await this.gate;
        if (this.haltCode !== null) return null;
        if (this.buffer.length === 0) return null;
        this.buffer.reset();
        this.halt('truncated_frame', false);
        return IngestOutcome.rejected('truncated_frame');
    }

    // A 410/cursor_expired response resyncs from a verified snapshot in the
    // runtime. The worker only reports the interruption; it never fabricates
    // a completion event for the expired cursor. A replacement worker is
    // required after the verified resume; this one stays halted.
    async cursorExpired(): Promise<IngestOutcome> {
        await this.gate;
        this.buffer.reset();
        this.halt('cursor_expired', true);
        return IngestOutcome.rejected('cursor_expired', true);
    }

    private async pushInternal(bytes: Uint8Array): Promise<IngestOutcome[]> {
        if (this.haltCode !== null) {
            return [
                IngestOutcome.rejected(
                    this.haltCode,
                    this.haltRequiresResync,
                ),
            ];
        }
        if (exceedsFrameLimit(bytes)) {
            this.halt('limit_exceeded', false);
            return [IngestOutcome.rejected('limit_exceeded')];
        }
        this.buffer.append(bytes);
        const view = this.buffer.view();
        const outcomes: IngestOutcome[] = [];
        let start = 0;
        let index = this.buffer.scanned;
        while (index < view.length) {
            if (view[index] !== 0x0a) {
                index += 1;
                continue;
            }
            if (index - start > MAX_EVENT_BYTES) {
                this.buffer.reset();
                this.halt('limit_exceeded', false);
                outcomes.push(IngestOutcome.rejected('limit_exceeded'));
                return outcomes;
            }
            const decoded = decodeLine(view.subarray(start, index));
            if (decoded === null) {
                this.buffer.reset();
                this.halt('malformed_event', false);
                outcomes.push(IngestOutcome.rejected('malformed_event'));
                return outcomes;
            }
            outcomes.push(await this.ingestFrame(decoded));
            start = index + 1;
            if (this.haltCode !== null) {
                this.buffer.reset();
                return outcomes;
            }
            index = start;
        }
        if (this.buffer.length - start > MAX_EVENT_BYTES) {
            this.buffer.reset();
            this.halt('limit_exceeded', false);
            outcomes.push(IngestOutcome.rejected('limit_exceeded'));
            return outcomes;
        }
        if (start > 0) this.buffer.compact(start);
        this.buffer.setScanned(this.buffer.length);
        return outcomes;
    }

    private async ingestFrame(frame: string): Promise<IngestOutcome> {
        if (frame.trim().length === 0) {
            return this.reject('malformed_event');
        }
        let event: AgentIncomingEvent;
        try {
            event = parseAgentEventWire(frame);
        } catch (error) {
            return this.reject(protocolCode(error));
        }
        if (this.taskId !== undefined && event.task_id !== this.taskId) {
            return this.reject('cross_task_event');
        }
        if (this.runId !== undefined && event.run_id !== this.runId) {
            return this.reject('out_of_order');
        }
        let result: AgentAppendResult;
        try {
            result = await this.sink(event);
        } catch (error) {
            return this.reject(protocolCode(error));
        }
        if (result.status === 'ignored') {
            return IngestOutcome.duplicate(event);
        }
        if (result.status === 'rejected') {
            return this.reject(
                result.code ?? 'invalid_event',
                result.requiresResync === true,
            );
        }
        return KNOWN_EVENT_TYPES.has(event.type)
            ? IngestOutcome.applied(event)
            : IngestOutcome.unknown(event);
    }

    private reject(
        code: AgentErrorCode,
        requiresResync = false,
    ): IngestOutcome {
        this.halt(code, requiresResync);
        return IngestOutcome.rejected(code, requiresResync);
    }

    private halt(code: AgentErrorCode, requiresResync: boolean): void {
        this.haltCode = code;
        this.haltRequiresResync = requiresResync;
        this.buffer.reset();
    }
}
