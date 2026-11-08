import { afterEach, describe, expect, it, vi } from 'vitest';
import {
    createCopyForwardPlan,
    createLifecyclePersistence,
    LIFECYCLE_MIGRATION_MARKER_KEY,
    LIFECYCLE_MIGRATION_MARKER_VALUE,
    MIGRATION_KEYS,
    migrateLifecycleStorage,
    readMigrationSnapshot,
} from './migration.js';

const ALL_KEYS = [
    'uor.studio.conversations.v1',
    'uor.studio.activeConversation.v1',
    'uor.studio.editor.v1',
    'uor.studio.modelSources.v1',
    'uor.studio.settings.v1',
    'uor.studio.ui.v2',
    'uor.studio.ui.v1',
    'uor.studio.attachments.v1',
    'uor.studio.tabs.v2',
    'uor.studio.recent.v2',
    'uor.studio.theme',
];

const DESTINATIONS: Record<string, string> = {
    'uor.studio.conversations.v1': 'uor.studio.lifecycle.conversations.v2',
    'uor.studio.activeConversation.v1': 'uor.studio.lifecycle.activeConversation.v2',
    'uor.studio.editor.v1': 'uor.studio.lifecycle.editor.v2',
    'uor.studio.settings.v1': 'uor.studio.lifecycle.settings.v2',
    'uor.studio.ui.v2': 'uor.studio.lifecycle.ui.v3',
    'uor.studio.ui.v1': 'uor.studio.lifecycle.ui.v2',
    'uor.studio.attachments.v1': 'uor.studio.lifecycle.attachments.v2',
    'uor.studio.tabs.v2': 'uor.studio.lifecycle.tabs.v3',
    'uor.studio.recent.v2': 'uor.studio.lifecycle.recent.v3',
    'uor.studio.theme': 'uor.studio.lifecycle.theme.v1',
};

function storageWith(entries: Record<string, string>) {
    const values = new Map(Object.entries(entries));
    return {
        values,
        getItem: vi.fn((key: string) => values.has(key) ? values.get(key)! : null),
        setItem: vi.fn((key: string, value: string) => { values.set(key, String(value)); }),
        removeItem: vi.fn((key: string) => { values.delete(key); }),
    };
}

function completeFixture(): Record<string, string> {
    return {
        'uor.studio.conversations.v1': JSON.stringify([{ id: 'session-a', title: 'Migrated', messages: [{ role: 'user', content: '17+25' }], future: { retained: true } }]),
        'uor.studio.activeConversation.v1': JSON.stringify('session-a'),
        'uor.studio.editor.v1': JSON.stringify({ content: 'draft', futureEditor: [1, 2] }),
        'uor.studio.modelSources.v1': JSON.stringify({ active: { kind: 'api', id: 'remote' }, api: [{ id: 'remote', apiKey: 'sentinel-not-a-secret' }] }),
        'uor.studio.settings.v1': JSON.stringify({ theme: 'dark', fontSize: 16, futureSetting: true }),
        'uor.studio.ui.v2': JSON.stringify({ nav: 'collapsed', futureFlag: 42 }),
        'uor.studio.ui.v1': JSON.stringify({ legacyNav: 'archival-only' }),
        'uor.studio.attachments.v1': JSON.stringify([{ name: 'context.txt', content: 'context', futureAttachment: 'retained' }]),
        'uor.studio.tabs.v2': JSON.stringify({ activePath: 'draft.md', tabs: [{ path: 'draft.md', content: 'draft', dirty: true }] }),
        'uor.studio.recent.v2': JSON.stringify([{ path: 'draft.md', rootPath: '/fixture' }]),
        'uor.studio.theme': 'dark',
    };
}

afterEach(() => { vi.restoreAllMocks(); });

describe('migration snapshot', () => {
    it('inventories every current and legacy key', () => {
        expect(MIGRATION_KEYS).toEqual(ALL_KEYS);
        expect(new Set(MIGRATION_KEYS).size).toBe(ALL_KEYS.length);
    });

    it('reads each key once and tolerates unknown fields', () => {
        const storage = storageWith({
            'uor.studio.settings.v1': JSON.stringify({ theme: 'dark', futureSetting: { nested: true } }),
            'uor.studio.ui.v2': JSON.stringify({ nav: 'collapsed', futureFlag: 42 }),
        });
        const snapshot = readMigrationSnapshot(storage);
        expect(storage.getItem).toHaveBeenCalledTimes(ALL_KEYS.length);
        for (const key of ALL_KEYS) expect(storage.getItem).toHaveBeenCalledWith(key);
        expect(snapshot.find((entry) => entry.key === 'uor.studio.settings.v1')?.value).toEqual({ theme: 'dark', futureSetting: { nested: true } });
        expect(snapshot.find((entry) => entry.key === 'uor.studio.ui.v2')?.value).toEqual({ nav: 'collapsed', futureFlag: 42 });
        expect(storage.setItem).not.toHaveBeenCalled();
    });

    it('preserves malformed raw input without throwing or logging its contents', () => {
        const sentinel = 'sentinel-not-a-secret';
        const storage = storageWith({ 'uor.studio.settings.v1': `{not-json-${sentinel}` });
        const snapshot = readMigrationSnapshot(storage);
        const entry = snapshot.find((item) => item.key === 'uor.studio.settings.v1');
        expect(entry?.raw).toBe(`{not-json-${sentinel}`);
        expect(entry?.value).toBeNull();
        expect(entry?.error).toBe('Invalid JSON');
        expect(entry?.error).not.toContain(sentinel);
    });

    it('represents missing values without an error', () => {
        const snapshot = readMigrationSnapshot(storageWith({}));
        expect(snapshot).toHaveLength(ALL_KEYS.length);
        for (const entry of snapshot) {
            expect(entry.raw).toBeNull();
            expect(entry.value).toBeNull();
            expect(entry.error).toBeNull();
        }
    });
});

describe('copy-forward plan', () => {
    it('is deterministic and idempotent', () => {
        const storage = storageWith({ 'uor.studio.theme': 'dark' });
        const snapshot = readMigrationSnapshot(storage);
        expect(createCopyForwardPlan(snapshot)).toEqual(createCopyForwardPlan(snapshot));
        expect(storage.getItem).toHaveBeenCalledTimes(ALL_KEYS.length);
        expect(storage.setItem).not.toHaveBeenCalled();
    });

    it('preserves raw rollback data for migratable keys', () => {
        const raw = JSON.stringify({ title: 'Draft', futureField: [1, 2, 3] });
        const plan = createCopyForwardPlan(readMigrationSnapshot(storageWith({ 'uor.studio.conversations.v1': raw })));
        const entry = plan.find((item) => item.sourceKey === 'uor.studio.conversations.v1');
        expect(entry?.raw).toBe(raw);
        expect(entry?.value).toEqual({ title: 'Draft', futureField: [1, 2, 3] });
        expect(entry?.destinationKey).not.toBe('uor.studio.conversations.v1');
    });

    it('never creates a second provider-secret store or migration marker', () => {
        const raw = JSON.stringify({ active: { kind: 'api', id: 'remote' }, api: [{ id: 'remote', apiKey: 'sentinel-not-a-secret' }] });
        const plan = createCopyForwardPlan(readMigrationSnapshot(storageWith({ 'uor.studio.modelSources.v1': raw })));
        const live = plan.find((entry) => entry.sourceKey === 'uor.studio.modelSources.v1');
        expect(live?.destinationKey).toBeNull();
        expect(live?.migratable).toBe(false);
        expect(plan.filter((entry) => entry.destinationKey !== null).map((entry) => entry.destinationKey)).not.toContain('uor.studio.modelSources.v1');
        expect(plan.every((entry) => !('migrationMarker' in entry))).toBe(true);
        expect(plan.every((entry) => !('payload' in entry))).toBe(true);
        expect(JSON.stringify(plan)).not.toContain('sentinel-not-a-secret');
    });

    it('preserves the raw theme as a valid value and plan', () => {
        const snapshot = readMigrationSnapshot(storageWith({ 'uor.studio.theme': 'dark' }));
        const entry = snapshot.find((item) => item.key === 'uor.studio.theme');
        expect(entry?.value).toBe('dark');
        expect(entry?.error).toBeNull();
        const plan = createCopyForwardPlan(snapshot);
        expect(plan.find((item) => item.sourceKey === 'uor.studio.theme')?.value).toBe('dark');
    });

    it('marks malformed entries non-copyable instead of erasing them', () => {
        const snapshot = readMigrationSnapshot(storageWith({ 'uor.studio.settings.v1': '{bad-json' }));
        const plan = createCopyForwardPlan(snapshot);
        const entry = plan.find((item) => item.sourceKey === 'uor.studio.settings.v1');
        expect(entry?.migratable).toBe(false);
        expect(entry?.destinationKey).toBeNull();
        expect(entry?.raw).toBe('{bad-json');
    });
});


describe('mounted migration transaction and routing', () => {
    it('copies every migratable key raw-exactly, writes the marker last, and preserves all sources', () => {
        const fixture = completeFixture();
        const before = new Map(Object.entries(fixture));
        const storage = storageWith(fixture);
        const writes: string[] = [];
        const originalSet = storage.setItem.getMockImplementation()!;
        storage.setItem.mockImplementation((key, value) => { writes.push(key); originalSet(key, value); });
        vi.spyOn(console, 'log').mockImplementation(() => undefined);
        vi.spyOn(console, 'warn').mockImplementation(() => undefined);
        vi.spyOn(console, 'error').mockImplementation(() => undefined);

        const result = migrateLifecycleStorage(storage);

        expect(result).toMatchObject({ state: 'migrated', error: null });
        for (const [source, destination] of Object.entries(DESTINATIONS)) {
            expect(storage.getItem(destination)).toBe(before.get(source));
            expect(storage.getItem(source)).toBe(before.get(source));
        }
        expect(storage.getItem(LIFECYCLE_MIGRATION_MARKER_KEY)).toBe(LIFECYCLE_MIGRATION_MARKER_VALUE);
        expect(writes.at(-1)).toBe(LIFECYCLE_MIGRATION_MARKER_KEY);
        expect(writes).not.toContain('uor.studio.modelSources.v1');
        expect(writes).not.toContain('uor.studio.lifecycle.modelSources.v1');
        expect(result.copied).toHaveLength(Object.keys(DESTINATIONS).length);
        expect(JSON.stringify(result)).not.toContain('sentinel-not-a-secret');
        for (const destination of Object.values(DESTINATIONS)) {
            expect(storage.getItem(destination) || '').not.toContain('sentinel-not-a-secret');
        }
        expect(console.log).not.toHaveBeenCalled();
        expect(console.warn).not.toHaveBeenCalled();
        expect(console.error).not.toHaveBeenCalled();

        const writeCount = storage.setItem.mock.calls.length;
        expect(migrateLifecycleStorage(storage)).toMatchObject({ state: 'current', error: null });
        expect(storage.setItem).toHaveBeenCalledTimes(writeCount);
    });

    it('routes only migratable writes after activation and keeps provider state in its live key', () => {
        const fixture = completeFixture();
        const originalConversations = fixture['uor.studio.conversations.v1'];
        const storage = storageWith(fixture);
        const persistence = createLifecyclePersistence(storage);

        expect(persistence.activate(true)).toMatchObject({ state: 'migrated', error: null });
        persistence.setItem('uor.studio.conversations.v1', '[{"changed":true}]');
        persistence.removeItem('uor.studio.activeConversation.v1');
        persistence.setItem('uor.studio.modelSources.v1', '{"api":[{"apiKey":"replacement-sentinel"}]}');

        expect(storage.getItem('uor.studio.conversations.v1')).toBe(originalConversations);
        expect(storage.getItem(DESTINATIONS['uor.studio.conversations.v1'])).toBe('[{"changed":true}]');
        expect(storage.getItem('uor.studio.activeConversation.v1')).toBe(fixture['uor.studio.activeConversation.v1']);
        expect(storage.getItem(DESTINATIONS['uor.studio.activeConversation.v1'])).toBeNull();
        expect(storage.getItem('uor.studio.modelSources.v1')).toBe('{"api":[{"apiKey":"replacement-sentinel"}]}');
        expect(storage.getItem('uor.studio.lifecycle.modelSources.v1')).toBeNull();

        expect(persistence.activate(false)).toMatchObject({ state: 'inactive', error: null });
        persistence.setItem('uor.studio.settings.v1', '{"defaultLegacy":true}');
        expect(storage.getItem('uor.studio.settings.v1')).toBe('{"defaultLegacy":true}');
    });

    it('short-circuits source preflight on marked reloads and never resurrects deleted destinations', () => {
        const storage = storageWith({
            'uor.studio.conversations.v1': '[{"old":true}]',
            'uor.studio.lifecycle.conversations.v2': '[{"edited":true}]',
            [LIFECYCLE_MIGRATION_MARKER_KEY]: LIFECYCLE_MIGRATION_MARKER_VALUE,
        });
        storage.getItem.mockClear();
        expect(migrateLifecycleStorage(storage)).toMatchObject({ state: 'current', error: null });
        expect(storage.getItem).toHaveBeenCalledOnce();
        expect(storage.getItem).toHaveBeenCalledWith(LIFECYCLE_MIGRATION_MARKER_KEY);
        expect(storage.setItem).not.toHaveBeenCalled();

        storage.removeItem('uor.studio.lifecycle.conversations.v2');
        expect(migrateLifecycleStorage(storage)).toMatchObject({ state: 'current', error: null });
        expect(storage.getItem('uor.studio.lifecycle.conversations.v2')).toBeNull();
        expect(storage.getItem('uor.studio.conversations.v1')).toBe('[{"old":true}]');
        expect(storage.setItem).not.toHaveBeenCalled();
    });

    it('fails before writes on malformed input, invalid markers, and destination conflicts', () => {
        const fixtures: Record<string, string>[] = [
            { 'uor.studio.settings.v1': '{bad-json-sentinel' },
            { [LIFECYCLE_MIGRATION_MARKER_KEY]: '{"schema":"wrong"}' },
            {
                'uor.studio.conversations.v1': '[{"source":true}]',
                'uor.studio.lifecycle.conversations.v2': '[{"conflict":true}]',
            },
        ];
        for (const fixture of fixtures) {
            const before = new Map(Object.entries(fixture));
            const storage = storageWith(fixture);
            expect(migrateLifecycleStorage(storage).state).toBe('error');
            expect(storage.setItem).not.toHaveBeenCalled();
            expect(storage.getItem(LIFECYCLE_MIGRATION_MARKER_KEY)).toBe(before.get(LIFECYCLE_MIGRATION_MARKER_KEY) ?? null);
            for (const [key, raw] of before) expect(storage.getItem(key)).toBe(raw);
        }
    });

    it('rolls back only new destinations on quota failure and preserves identical existing destinations', () => {
        const fixture = completeFixture();
        fixture['uor.studio.lifecycle.editor.v2'] = fixture['uor.studio.editor.v1'];
        const storage = storageWith(fixture);
        const originalSet = storage.setItem.getMockImplementation()!;
        storage.setItem.mockImplementation((key, value) => {
            if (key === DESTINATIONS['uor.studio.settings.v1']) throw Object.assign(new Error('fixture quota'), { name: 'QuotaExceededError' });
            originalSet(key, value);
        });

        const result = migrateLifecycleStorage(storage);

        expect(result).toMatchObject({ state: 'error' });
        expect(result.error).toContain('quota');
        expect(storage.getItem('uor.studio.lifecycle.editor.v2')).toBe(fixture['uor.studio.editor.v1']);
        for (const destination of Object.values(DESTINATIONS)) {
            if (destination !== 'uor.studio.lifecycle.editor.v2') expect(storage.getItem(destination)).toBeNull();
        }
        expect(storage.getItem(LIFECYCLE_MIGRATION_MARKER_KEY)).toBeNull();
        for (const [source, raw] of Object.entries(fixture)) {
            if (!source.startsWith('uor.studio.lifecycle.')) expect(storage.getItem(source)).toBe(raw);
        }
    });

    it('rolls back every new destination when the marker cannot be written', () => {
        const fixture = completeFixture();
        const storage = storageWith(fixture);
        const originalSet = storage.setItem.getMockImplementation()!;
        storage.setItem.mockImplementation((key, value) => {
            if (key === LIFECYCLE_MIGRATION_MARKER_KEY) throw new Error('marker write failed');
            originalSet(key, value);
        });
        expect(migrateLifecycleStorage(storage).state).toBe('error');
        for (const destination of Object.values(DESTINATIONS)) expect(storage.getItem(destination)).toBeNull();
        expect(storage.getItem(LIFECYCLE_MIGRATION_MARKER_KEY)).toBeNull();
        for (const [source, raw] of Object.entries(fixture)) expect(storage.getItem(source)).toBe(raw);
    });

    it('rejects destination read-back failures and rolls back new writes', () => {
        const storage = storageWith(completeFixture());
        const originalGet = storage.getItem.getMockImplementation()!;
        let mismatch = true;
        storage.getItem.mockImplementation((key) => {
            if (mismatch && key === DESTINATIONS['uor.studio.conversations.v1'] && storage.values.has(key)) return null;
            return originalGet(key);
        });
        expect(migrateLifecycleStorage(storage).state).toBe('error');
        mismatch = false;
        for (const destination of Object.values(DESTINATIONS)) expect(storage.getItem(destination)).toBeNull();
        expect(storage.getItem(LIFECYCLE_MIGRATION_MARKER_KEY)).toBeNull();
    });

    it('does not overwrite an intervening source edit and rolls back before the marker', () => {
        const fixture = completeFixture();
        const storage = storageWith(fixture);
        const originalSet = storage.setItem.getMockImplementation()!;
        storage.setItem.mockImplementation((key, value) => {
            originalSet(key, value);
            if (key === DESTINATIONS['uor.studio.conversations.v1']) {
                storage.values.set('uor.studio.settings.v1', '{"intervening":true}');
            }
        });
        expect(migrateLifecycleStorage(storage).state).toBe('error');
        expect(storage.getItem('uor.studio.settings.v1')).toBe('{"intervening":true}');
        for (const destination of Object.values(DESTINATIONS)) expect(storage.getItem(destination)).toBeNull();
        expect(storage.getItem(LIFECYCLE_MIGRATION_MARKER_KEY)).toBeNull();
    });

    it('keeps ui.v1 archival without inventing a current-ui fallback', () => {
        const storage = storageWith({ 'uor.studio.ui.v1': '{"legacyNav":"archived"}' });
        const persistence = createLifecyclePersistence(storage);
        expect(persistence.activate(true).state).toBe('migrated');
        expect(storage.getItem('uor.studio.lifecycle.ui.v2')).toBe('{"legacyNav":"archived"}');
        expect(persistence.getItem('uor.studio.ui.v2')).toBeNull();
    });
});


describe('migration failure preservation', () => {
    it('makes migratable keys read-only after activation failure without blocking the live provider key', () => {
        const storage = storageWith({ 'uor.studio.settings.v1': '{malformed-source' });
        const persistence = createLifecyclePersistence(storage);
        expect(persistence.activate(true).state).toBe('error');

        persistence.setItem('uor.studio.settings.v1', '{"replacement":true}');
        persistence.removeItem('uor.studio.settings.v1');
        persistence.setItem('uor.studio.conversations.v1', '[{"replacement":true}]');
        persistence.setItem('uor.studio.modelSources.v1', '{"api":[{"apiKey":"sentinel-not-a-secret"}]}');

        expect(storage.getItem('uor.studio.settings.v1')).toBe('{malformed-source');
        expect(storage.getItem('uor.studio.conversations.v1')).toBeNull();
        expect(storage.getItem('uor.studio.modelSources.v1')).toBe('{"api":[{"apiKey":"sentinel-not-a-secret"}]}');
    });
});


describe('marker verification and rollback failures', () => {
    it('rolls back new writes and preserves sources after marker mismatch or read failure', () => {
        for (const mode of ['mismatch', 'read-failure']) {
            const fixture = completeFixture();
            const storage = storageWith(fixture);
            const originalGet = storage.getItem.getMockImplementation()!;
            let intercept = true;
            storage.getItem.mockImplementation((key) => {
                if (intercept && key === LIFECYCLE_MIGRATION_MARKER_KEY && storage.values.has(key)) {
                    intercept = false;
                    if (mode === 'mismatch') return '{"schema":"wrong"}';
                    throw new Error('marker read failed');
                }
                return originalGet(key);
            });

            expect(migrateLifecycleStorage(storage).state).toBe('error');
            for (const destination of Object.values(DESTINATIONS)) expect(storage.getItem(destination)).toBeNull();
            expect(storage.getItem(LIFECYCLE_MIGRATION_MARKER_KEY)).toBeNull();
            for (const [source, raw] of Object.entries(fixture)) expect(storage.getItem(source)).toBe(raw);
        }
    });

    it('reports rollback-failed, preserves sources, and retains only destinations that cannot be removed', () => {
        const fixture = completeFixture();
        fixture['uor.studio.lifecycle.editor.v2'] = fixture['uor.studio.editor.v1'];
        const storage = storageWith(fixture);
        const originalSet = storage.setItem.getMockImplementation()!;
        storage.setItem.mockImplementation((key, value) => {
            if (key === DESTINATIONS['uor.studio.settings.v1']) throw Object.assign(new Error('fixture quota'), { name: 'QuotaExceededError' });
            originalSet(key, value);
        });
        storage.removeItem.mockImplementation((key) => {
            if (key === DESTINATIONS['uor.studio.conversations.v1']) throw new Error('rollback remove failed');
            storage.values.delete(key);
        });

        const result = migrateLifecycleStorage(storage);

        expect(result).toMatchObject({ state: 'error', error: 'rollback-failed' });
        expect(storage.getItem(DESTINATIONS['uor.studio.conversations.v1'])).not.toBeNull();
        expect(storage.getItem('uor.studio.lifecycle.editor.v2')).toBe(fixture['uor.studio.editor.v1']);
        for (const destination of Object.values(DESTINATIONS)) {
            if (destination !== DESTINATIONS['uor.studio.conversations.v1'] && destination !== 'uor.studio.lifecycle.editor.v2') {
                expect(storage.getItem(destination)).toBeNull();
            }
        }
        expect(storage.getItem(LIFECYCLE_MIGRATION_MARKER_KEY)).toBeNull();
        for (const [source, raw] of Object.entries(fixture)) {
            if (!source.startsWith('uor.studio.lifecycle.')) expect(storage.getItem(source)).toBe(raw);
        }
    });
});


describe('initial marker read failure', () => {
    it('fails before any write when the existing marker cannot be read', () => {
        const fixture = completeFixture();
        const storage = storageWith(fixture);
        storage.getItem.mockImplementation((key) => {
            if (key === LIFECYCLE_MIGRATION_MARKER_KEY) throw new Error('initial marker read failed');
            return fixture[key] ?? null;
        });

        expect(migrateLifecycleStorage(storage)).toMatchObject({ state: 'error', error: 'marker-read-failed' });
        expect(storage.setItem).not.toHaveBeenCalled();
        expect(storage.removeItem).not.toHaveBeenCalled();
        for (const [source, raw] of Object.entries(fixture)) expect(fixture[source]).toBe(raw);
    });
});
