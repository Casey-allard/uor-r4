import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { activateStudioMigration, studioMigrationStatus } from './migration.js';

interface PersistenceGlobal { LIFECYCLE_SINGLETON?: boolean }
const globalWindow = window as unknown as { UORStudioPersistence: PersistenceGlobal };

beforeEach(() => {
    activateStudioMigration(false);
    window.localStorage.clear();
});

afterEach(() => {
    vi.restoreAllMocks();
    activateStudioMigration(false);
    window.localStorage.clear();
});

describe('global migration runtime singleton', () => {
    it('does not activate or replace the successful global instance on a second evaluation', async () => {
        window.localStorage.setItem('uor.studio.conversations.v1', '[]');
        expect(activateStudioMigration(true).state).toBe('migrated');
        const first = globalWindow.UORStudioPersistence;

        // @ts-expect-error A query suffix creates a second real evaluation of the plain-JavaScript runtime.
        await import('../../assets/js/lifecycle_persistence.js?second-success-evaluation');

        expect(globalWindow.UORStudioPersistence).toBe(first);
        expect(studioMigrationStatus().state).toBe('migrated');
    });

    it('keeps a one-shot first failure visible instead of succeeding on a second evaluation', async () => {
        window.localStorage.setItem('uor.studio.conversations.v1', '[]');
        const originalSet = Storage.prototype.setItem;
        let thrown = false;
        vi.spyOn(Storage.prototype, 'setItem').mockImplementation(function (this: Storage, key, value) {
            if (!thrown && key === 'uor.studio.lifecycle.conversations.v2') {
                thrown = true;
                throw new DOMException('one-shot quota', 'QuotaExceededError');
            }
            originalSet.call(this, key, value);
        });
        expect(activateStudioMigration(true)).toMatchObject({ state: 'error', error: 'quota-exceeded' });
        expect(studioMigrationStatus()).toMatchObject({ state: 'error', error: 'quota-exceeded' });
        const first = globalWindow.UORStudioPersistence;
        expect(first.LIFECYCLE_SINGLETON).toBe(true);

        // @ts-expect-error A query suffix creates a second real evaluation of the plain-JavaScript runtime.
        await import('../../assets/js/lifecycle_persistence.js?second-failure-evaluation');

        expect(globalWindow.UORStudioPersistence).toBe(first);
        expect(studioMigrationStatus()).toMatchObject({ state: 'error', error: 'quota-exceeded' });
        expect(window.localStorage.getItem('uor.studio.lifecycle.migration.v1')).toBeNull();
    });
});
