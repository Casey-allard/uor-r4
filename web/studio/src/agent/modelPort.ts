export type ModelFinishReason = 'stop' | 'length' | 'tool' | 'cancelled' | 'error';

export interface ModelHistoryMessage {
    role: 'user' | 'assistant';
    text: string;
}

export interface ModelRequest {
    taskId: string;
    runId: string;
    generation: number;
    mode: 'plan' | 'agent';
    prompt: string;
    history: readonly ModelHistoryMessage[];
}

export type ModelStreamEvent =
    | { type: 'text-delta'; text: string }
    | {
          type: 'tool-call';
          toolCallId: string;
          tool: string;
          arguments: unknown;
      }
    | {
          type: 'usage';
          inputTokens: number;
          outputTokens: number;
          modelCalls: number;
      }
    | { type: 'finished'; reason: ModelFinishReason }
    | {
          type: 'failed';
          code: string;
          message: string;
          retryable: boolean;
      };

export interface ModelPort {
    readonly id: string;
    readonly kind: string;
    readonly synthetic: boolean;
    open(request: ModelRequest): AsyncIterable<ModelStreamEvent>;
}

export const REPLAY_MODEL_ID = 'synthetic-replay';
export const REPLAY_MODEL_KIND = 'synthetic-replay';
export const MAX_REPLAY_SCRIPT_POSITIONS = 256;
export const REPLAY_MODEL_LABEL =
    'Synthetic deterministic replay fixture; not an R4 model or a task brain.';

export type ReplayModelTurn = readonly ModelStreamEvent[];

export interface ReplayModelPortOptions {
    chunks?: readonly string[];
    usage?: {
        inputTokens: number;
        outputTokens: number;
        modelCalls: number;
    };
    script?: readonly ReplayModelTurn[];
    /** Deterministic pacing between default replay events; 0 keeps tests immediate. */
    paceMs?: number;
}

// Labelled synthetic port. It deterministically produces text, tool, usage,
// finished and failed events. It does not call R4, a provider or a host
// process.
export class ReplayModelPort implements ModelPort {
    readonly id = REPLAY_MODEL_ID;
    readonly kind = REPLAY_MODEL_KIND;
    readonly synthetic = true;
    readonly label = REPLAY_MODEL_LABEL;

    private readonly chunks: readonly string[];
    private readonly usage: {
        inputTokens: number;
        outputTokens: number;
        modelCalls: number;
    } | null;
    private readonly turns: readonly ReplayModelTurn[] | null;
    private readonly paceMs: number;
    private readonly turnIndexByRun = new Map<string, number>();

    constructor(options: ReplayModelPortOptions = {}) {
        this.chunks = options.chunks ?? ['Synthetic replay acknowledges: '];
        this.usage = options.usage ?? null;
        this.turns =
            options.script === undefined
                ? null
                : options.script.length === 0
                  ? [[]]
                  : options.script;
        this.paceMs = Math.max(0, options.paceMs ?? 0);
    }

    get scriptPositionCount(): number {
        return this.turnIndexByRun.size;
    }

    async *open(request: ModelRequest): AsyncIterable<ModelStreamEvent> {
        if (this.turns !== null) {
            const index = this.turnIndexByRun.get(request.runId) ?? 0;
            if (index >= this.turns.length) return;
            this.turnIndexByRun.set(request.runId, index + 1);
            this.pruneTurnIndexes();
            for (const event of this.turns[index] ?? []) yield event;
            return;
        }
        const text = `${this.chunks.join('')}${request.prompt}`;
        await this.pause();
        yield { type: 'text-delta', text };
        const usage =
            this.usage === null
                ? {
                      inputTokens: request.prompt.length,
                      outputTokens: text.length,
                      modelCalls: 1,
                  }
                : this.usage;
        await this.pause();
        yield { type: 'usage', ...usage };
        await this.pause();
        yield { type: 'finished', reason: 'stop' };
    }

    private pause(): Promise<void> {
        if (this.paceMs === 0) return Promise.resolve();
        return new Promise<void>((resolve) => {
            globalThis.setTimeout(resolve, this.paceMs);
        });
    }

    // Completed script positions are evicted oldest-first once the map exceeds
    // the cap. Active (not-yet-exhausted) positions are never evicted, so two
    // live runs cannot consume each other's turns.
    private pruneTurnIndexes(): void {
        if (this.turnIndexByRun.size <= MAX_REPLAY_SCRIPT_POSITIONS) return;
        const length = this.turns?.length ?? 0;
        for (const [runId, index] of this.turnIndexByRun) {
            if (index < length) continue;
            this.turnIndexByRun.delete(runId);
            if (this.turnIndexByRun.size <= MAX_REPLAY_SCRIPT_POSITIONS) {
                return;
            }
        }
    }
}
