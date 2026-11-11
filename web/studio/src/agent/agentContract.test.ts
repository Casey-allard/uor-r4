import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

import {
    AGENT_CONTRACT_DIGEST,
    AGENT_CONTRACT_VERSION,
    AGENT_ERROR_CODES,
    AGENT_ERROR_SCHEMA,
    AGENT_EVENT_SCHEMA,
    AGENT_EVENT_TYPES,
    AGENT_RUN_STATES,
    APPROVAL_TTL_MS,
    CANONICAL_JSON_SCHEMA,
    CURSOR_PREFIX,
    MAX_EVENT_BYTES,
    MAX_EVENTS_PER_RUN,
    MAX_SAFE_INTEGER,
    TERMINAL_AGENT_EVENT_TYPES,
    type AgentCursor,
    type AgentError,
    type AgentErrorCode,
    type AgentEvent,
    type AgentEventDataMap,
    type ApprovalRequest,
    type KnownAgentEventType,
} from './generated/agentContract.js';

interface SchemaProperty {
    type?: string | string[];
    enum?: string[];
    const?: unknown;
    minimum?: number;
    maximum?: number;
    pattern?: string;
    minLength?: number;
    properties?: Record<string, SchemaProperty>;
    required?: string[];
}

interface ContractSchema {
    $id: string;
    $schema: string;
    canonical_json_schema: string;
    contract_version: number;
    cursor_prefix: string;
    envelope: { required: string[]; properties: Record<string, SchemaProperty> };
    error_codes: string[];
    error_schema: string;
    event_types: Record<
        string,
        { data: { required: string[]; properties: Record<string, SchemaProperty> }; terminal: boolean }
    >;
    limits: { approval_ttl_ms: number; max_event_bytes: number; max_events_per_run: number };
    run_states: string[];
    schema_id: string;
    title: string;
}

const schemaPath = resolve(
    moduleDirectory(import.meta.url),
    '../../../../crates/uor-r4-studio-agent/contract.schema.json',
);

function moduleDirectory(url: string): string {
    const path = moduleFilePath(url);
    return path.slice(0, path.lastIndexOf('/'));
}

function moduleFilePath(url: string): string {
    const withoutQuery = url.split('?')[0];
    if (withoutQuery.startsWith('/@fs/')) {
        return decodeURIComponent(withoutQuery.slice('/@fs'.length));
    }
    if (withoutQuery.startsWith('file://')) {
        return decodeURIComponent(new URL(withoutQuery).pathname);
    }
    return decodeURIComponent(withoutQuery);
}

async function readSchemaText(): Promise<string> {
    return readFile(schemaPath, 'utf8');
}

async function readSchema(): Promise<ContractSchema> {
    return JSON.parse(await readSchemaText()) as ContractSchema;
}

describe('generated agent contract', () => {
    it('pins the schema identity constants', async () => {
        const schema = await readSchema();
        expect(AGENT_EVENT_SCHEMA).toBe('uor-r4.studio.agent-event/1');
        expect(AGENT_ERROR_SCHEMA).toBe('uor-r4.studio.agent-error/1');
        expect(CANONICAL_JSON_SCHEMA).toBe('uor-r4.canonical-json/1');
        expect(AGENT_CONTRACT_VERSION).toBe(1);
        expect(CURSOR_PREFIX).toBe('uorse1');
        expect(schema.$id).toBe(AGENT_EVENT_SCHEMA);
        expect(schema.schema_id).toBe(AGENT_EVENT_SCHEMA);
        expect(schema.error_schema).toBe(AGENT_ERROR_SCHEMA);
        expect(schema.canonical_json_schema).toBe(CANONICAL_JSON_SCHEMA);
        expect(schema.contract_version).toBe(AGENT_CONTRACT_VERSION);
        expect(schema.$schema).toBe('https://json-schema.org/draft/2020-12/schema');
    });

    it('hashes the exact checked-in schema bytes including the trailing newline', async () => {
        const bytes = await readFile(schemaPath);
        const text = await readSchemaText();
        const digest = createHash('sha256').update(bytes).digest('hex');
        expect(text.endsWith('\n')).toBe(true);
        expect(text.startsWith('{"$id":"uor-r4.studio.agent-event/1"')).toBe(true);
        expect(AGENT_CONTRACT_DIGEST).toMatch(/^[0-9a-f]{64}$/);
        expect(AGENT_CONTRACT_DIGEST).toBe(
            '20079c51893c30faa9789158ca3aa3d3c7ac486bbb3c3555ce51316bda8b7ebf',
        );
        expect(AGENT_CONTRACT_DIGEST).toBe(digest);
    });

    it('derives every event type and the terminal set from the schema', async () => {
        const schema = await readSchema();
        expect([...AGENT_EVENT_TYPES]).toEqual(Object.keys(schema.event_types).sort());
        expect([...TERMINAL_AGENT_EVENT_TYPES]).toEqual(
            Object.keys(schema.event_types)
                .filter((eventType) => schema.event_types[eventType].terminal)
                .sort(),
        );
        expect(AGENT_EVENT_TYPES).toContain('run.queued');
        expect(AGENT_EVENT_TYPES).toContain('run.started');
        expect(AGENT_EVENT_TYPES).toContain('run.resumed');
        expect(TERMINAL_AGENT_EVENT_TYPES).toEqual([
            'run.cancelled',
            'run.expired',
            'run.failed',
            'run.finished',
        ]);
        const known: KnownAgentEventType = 'run.queued';
        expect(known).toBe(AGENT_EVENT_TYPES[AGENT_EVENT_TYPES.indexOf('run.queued')]);
    });

    it('derives the error-code taxonomy and run states from the schema', async () => {
        const schema = await readSchema();
        expect([...AGENT_ERROR_CODES]).toEqual(schema.error_codes);
        expect(new Set(AGENT_ERROR_CODES).size).toBe(AGENT_ERROR_CODES.length);
        expect(AGENT_ERROR_CODES).toContain('runtime_unavailable');
        expect(AGENT_ERROR_CODES).toContain('host_unavailable');
        expect(AGENT_ERROR_CODES.indexOf('runtime_unavailable')).not.toBe(
            AGENT_ERROR_CODES.indexOf('host_unavailable'),
        );
        expect([...AGENT_RUN_STATES]).toEqual(schema.run_states);
    });

    it('derives limits from the schema', async () => {
        const schema = await readSchema();
        expect(MAX_SAFE_INTEGER).toBe(9_007_199_254_740_991);
        expect(MAX_EVENT_BYTES).toBe(schema.limits.max_event_bytes);
        expect(MAX_EVENTS_PER_RUN).toBe(schema.limits.max_events_per_run);
        expect(APPROVAL_TTL_MS).toBe(schema.limits.approval_ttl_ms);
    });

    it('generates run.queued and run.started with a nullable retry id', () => {
        const queued: AgentEventDataMap['run.queued'] = { queued_at_ms: 1 };
        const started: AgentEventDataMap['run.started'] = {
            started_at_ms: 1,
            model: { id: 'replay', kind: 'fixture' },
            mode: 'agent',
            environment: { kind: 'local-browser' },
            retry_of_run_id: null,
        };
        const retried: AgentEventDataMap['run.started'] = {
            ...started,
            retry_of_run_id: 'run_7f3a2c1b9d4e6a80',
        };
        expect(queued.queued_at_ms).toBe(1);
        expect(started.retry_of_run_id).toBeNull();
        expect(retried.retry_of_run_id).toBe('run_7f3a2c1b9d4e6a80');
    });

    it('generates run.resumed with every required resume field', () => {
        const resumed: AgentEventDataMap['run.resumed'] = {
            resumed_at_ms: 1,
            previous_generation: 1,
            snapshot_digest: 'sha256:' + 'a'.repeat(64),
        };
        const fields: Array<keyof AgentEventDataMap['run.resumed']> = [
            'resumed_at_ms',
            'previous_generation',
            'snapshot_digest',
        ];
        expect(fields.every((field) => field in resumed)).toBe(true);
        expect(resumed.previous_generation + 1).toBe(2);
    });

    it('keeps the known event union discriminable by type', () => {
        const event: AgentEvent = {
            schema: 'uor-r4.studio.agent-event/1',
            event_id: 'evt_run_7f3a2c1b9d4e6a80_1_000000000001',
            project_id: 'prj_4b1d9e0c7a2f6e51',
            task_id: 'tsk_9a3c5e7b1d2f4a60',
            run_id: 'run_7f3a2c1b9d4e6a80',
            generation: 1,
            sequence: 1,
            recorded_at_ms: 1_794_283_200_123,
            type: 'run.started',
            data: {
                started_at_ms: 1,
                model: { id: 'replay', kind: 'fixture' },
                mode: 'agent',
                environment: { kind: 'local-browser' },
                retry_of_run_id: null,
            },
        };
        if (event.type === 'run.started') {
            expect(event.data.retry_of_run_id).toBeNull();
            expect(event.data.mode).toBe('agent');
        } else {
            throw new Error('the known run.started event did not narrow');
        }
    });

    it('keeps templated AgentError, AgentCursor and ApprovalRequest consistent with the contract', async () => {
        const schema = await readSchema();

        // The schema has no full object definition for AgentError; this type is
        // templated from crates/uor-r4-studio-agent/src/error.rs.
        const error: AgentError = {
            schema: AGENT_ERROR_SCHEMA,
            code: 'runtime_unavailable',
            message: 'runtime is down',
            retryable: true,
            task_id: null,
            run_id: null,
        };
        const code: AgentErrorCode = error.code;
        expect(schema.error_codes).toContain(code);
        expect(error.schema).toBe(schema.error_schema);
        expect(Object.keys(error).sort()).toEqual([
            'code',
            'message',
            'retryable',
            'run_id',
            'schema',
            'task_id',
        ]);

        // The schema has no full object definition for AgentCursor; this type is
        // templated from crates/uor-r4-studio-agent/src/contract.rs.
        const cursor: AgentCursor = {
            project_id: 'prj_4b1d9e0c7a2f6e51',
            task_id: 'tsk_9a3c5e7b1d2f4a60',
            run_id: 'run_7f3a2c1b9d4e6a80',
            generation: 1,
            sequence: 6,
        };
        expect(Object.keys(cursor).sort()).toEqual([
            'generation',
            'project_id',
            'run_id',
            'sequence',
            'task_id',
        ]);
        const envelopeKeys = new Set(schema.envelope.required);
        expect(envelopeKeys.has('project_id')).toBe(true);
        expect(envelopeKeys.has('task_id')).toBe(true);
        expect(envelopeKeys.has('run_id')).toBe(true);
        expect(envelopeKeys.has('generation')).toBe(true);
        expect(envelopeKeys.has('sequence')).toBe(true);

        // The schema has no full object definition for ApprovalRequest; the
        // tool.approval_required payload supplies its event fields and the Rust
        // struct adds the generation, task, run and requested_at_ms identity.
        const approval: ApprovalRequest = {
            approval_id: 'apr_0123456789abcdef',
            task_id: 'tsk_9a3c5e7b1d2f4a60',
            run_id: 'run_7f3a2c1b9d4e6a80',
            generation: 1,
            tool_call_id: 'tc_0123456789abcdef',
            tool: 'shell',
            scope: 'workspace',
            request_digest: 'sha256:' + 'b'.repeat(64),
            requested_at_ms: 1,
            expires_at_ms: 2,
        };
        const schemaApprovalKeys = Object.keys(
            schema.event_types['tool.approval_required'].data.properties,
        ).sort();
        expect(schemaApprovalKeys).toEqual([
            'approval_id',
            'expires_at_ms',
            'request_digest',
            'scope',
            'tool',
            'tool_call_id',
        ]);
        for (const key of schemaApprovalKeys) {
            expect(Object.prototype.hasOwnProperty.call(approval, key)).toBe(true);
        }
        expect(Object.keys(approval)).toHaveLength(schemaApprovalKeys.length + 4);
        expect(approval.request_digest).toMatch(/^sha256:[0-9a-f]{64}$/);
    });
});
