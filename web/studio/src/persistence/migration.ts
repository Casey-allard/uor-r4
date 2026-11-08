import '../../assets/js/lifecycle_persistence.js';

const KEYS = [
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
] as const;

export const MIGRATION_KEYS = runtime().MIGRATION_KEYS as unknown as typeof KEYS;
export const LIFECYCLE_MIGRATION_MARKER_KEY = runtime().LIFECYCLE_MIGRATION_MARKER_KEY;
export const LIFECYCLE_MIGRATION_MARKER_VALUE = runtime().LIFECYCLE_MIGRATION_MARKER_VALUE;

export type MigrationKey = typeof KEYS[number];
export type MigrationState = 'inactive' | 'migrated' | 'current' | 'error';

export interface MigrationEntry {
    key: MigrationKey;
    sourceKey: MigrationKey;
    raw: string | null;
    value: unknown;
    error: string | null;
    destinationKey: string | null;
    migratable: boolean;
}

export interface MigrationStorage {
    getItem(key: string): string | null;
}

export interface LifecycleMigrationStorage extends MigrationStorage {
    setItem(key: string, value: string): void;
    removeItem(key: string): void;
}

export interface LifecycleMigrationResult {
    state: MigrationState;
    error: string | null;
    copied: string[];
}

export interface LifecyclePersistence {
    activate(typed: boolean): LifecycleMigrationResult;
    migrate(): LifecycleMigrationResult;
    status(): LifecycleMigrationResult;
    getItem(key: string): string | null;
    setItem(key: string, value: string): void;
    removeItem(key: string): void;
}

interface PersistenceRuntime {
    MIGRATION_KEYS: readonly string[];
    LIFECYCLE_MIGRATION_MARKER_KEY: 'uor.studio.lifecycle.migration.v1';
    LIFECYCLE_MIGRATION_MARKER_VALUE: '{"schema":"uor-r4.studio-lifecycle-migration/1"}';
    readMigrationSnapshot(storage: MigrationStorage): MigrationEntry[];
    createCopyForwardPlan(snapshot: MigrationEntry[]): MigrationEntry[];
    migrateStorage(storage: LifecycleMigrationStorage): LifecycleMigrationResult;
    createLifecyclePersistence(storage: LifecycleMigrationStorage): LifecyclePersistence;
    activate(typed: boolean): LifecycleMigrationResult;
    status(): LifecycleMigrationResult;
}

function runtime(): PersistenceRuntime {
    return (globalThis as unknown as { UORStudioPersistence: PersistenceRuntime }).UORStudioPersistence;
}

export function readMigrationSnapshot(storage: MigrationStorage): MigrationEntry[] {
    return runtime().readMigrationSnapshot(storage);
}

export function createCopyForwardPlan(snapshot: MigrationEntry[]): MigrationEntry[] {
    return runtime().createCopyForwardPlan(snapshot);
}

export function migrateLifecycleStorage(storage: LifecycleMigrationStorage): LifecycleMigrationResult {
    return runtime().migrateStorage(storage);
}

export function createLifecyclePersistence(storage: LifecycleMigrationStorage): LifecyclePersistence {
    return runtime().createLifecyclePersistence(storage);
}

export function activateStudioMigration(typed: boolean): LifecycleMigrationResult {
    return runtime().activate(typed);
}

export function studioMigrationStatus(): LifecycleMigrationResult {
    return runtime().status();
}
