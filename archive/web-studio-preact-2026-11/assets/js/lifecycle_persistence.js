(function () {
    'use strict';

    if (globalThis.UORStudioPersistence && globalThis.UORStudioPersistence.LIFECYCLE_SINGLETON === true) return;

    var MIGRATION_KEYS = [
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
    var DESTINATION_KEYS = {
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
    var MARKER_KEY = 'uor.studio.lifecycle.migration.v1';
    var MARKER_VALUE = '{"schema":"uor-r4.studio-lifecycle-migration/1"}';

    function isLiveCompatibilityKey(key) {
        return key === 'uor.studio.modelSources.v1';
    }

    function status(state, error, copied) {
        return { state: state, error: error, copied: copied || [] };
    }

    function cloneStatus(value) {
        return status(value.state, value.error, value.copied.slice());
    }

    function readEntry(storage, key) {
        var raw = storage.getItem(key);
        if (raw === null) {
            return { key: key, sourceKey: key, raw: null, value: null, error: null, destinationKey: null, migratable: !isLiveCompatibilityKey(key) };
        }
        if (key === 'uor.studio.theme') {
            return { key: key, sourceKey: key, raw: raw, value: raw, error: null, destinationKey: null, migratable: true };
        }
        try {
            return { key: key, sourceKey: key, raw: raw, value: JSON.parse(raw), error: null, destinationKey: null, migratable: !isLiveCompatibilityKey(key) };
        } catch (_error) {
            return { key: key, sourceKey: key, raw: raw, value: null, error: 'Invalid JSON', destinationKey: null, migratable: !isLiveCompatibilityKey(key) };
        }
    }

    function readMigrationSnapshot(storage) {
        return MIGRATION_KEYS.map(function (key) { return readEntry(storage, key); });
    }

    function createCopyForwardPlan(snapshot) {
        return snapshot.map(function (entry) {
            if (isLiveCompatibilityKey(entry.sourceKey)) {
                return Object.assign({}, entry, { raw: null, value: null, destinationKey: null, migratable: false });
            }
            if (entry.error !== null) {
                return Object.assign({}, entry, { value: null, destinationKey: null, migratable: false });
            }
            return Object.assign({}, entry, {
                value: entry.value === undefined ? null : structuredClone(entry.value),
                destinationKey: DESTINATION_KEYS[entry.sourceKey],
            });
        });
    }

    function quotaFailure(error) {
        return Boolean(error && (error.name === 'QuotaExceededError' || error.name === 'NS_ERROR_DOM_QUOTA_REACHED'));
    }

    function rollback(storage, created, markerAttempted, code) {
        var failed = false;
        if (markerAttempted) {
            try {
                storage.removeItem(MARKER_KEY);
                if (storage.getItem(MARKER_KEY) !== null) failed = true;
            } catch (_error) {
                failed = true;
            }
        }
        for (var i = created.length - 1; i >= 0; i -= 1) {
            try {
                storage.removeItem(created[i]);
                if (storage.getItem(created[i]) !== null) failed = true;
            } catch (_error) {
                failed = true;
            }
        }
        return status('error', failed ? 'rollback-failed' : code, []);
    }

    function migrateStorage(storage) {
        var marker;
        try {
            marker = storage.getItem(MARKER_KEY);
        } catch (_error) {
            return status('error', 'marker-read-failed', []);
        }
        if (marker === MARKER_VALUE) return status('current', null, []);
        if (marker !== null) return status('error', 'invalid-marker', []);

        var entries = [];
        var pending = [];
        var ensured = [];
        try {
            for (var i = 0; i < MIGRATION_KEYS.length; i += 1) {
                var key = MIGRATION_KEYS[i];
                if (isLiveCompatibilityKey(key)) continue;
                var entry = readEntry(storage, key);
                if (entry.error !== null) return status('error', 'malformed-source', []);
                var destination = DESTINATION_KEYS[key];
                var destinationRaw = storage.getItem(destination);
                if (entry.raw === null) {
                    if (destinationRaw !== null) return status('error', 'destination-conflict', []);
                    continue;
                }
                if (destinationRaw !== null && destinationRaw !== entry.raw) return status('error', 'destination-conflict', []);
                entries.push({ sourceKey: key, raw: entry.raw, destinationKey: destination });
                ensured.push(destination);
                if (destinationRaw === null) pending.push(destination);
            }
        } catch (_error) {
            return status('error', 'preflight-read-failed', []);
        }

        var created = [];
        var markerAttempted = false;
        var stage = 'destination-write-failed';
        try {
            for (var p = 0; p < pending.length; p += 1) {
                var pendingEntry = entries.find(function (entry) { return entry.destinationKey === pending[p]; });
                created.push(pending[p]);
                stage = 'destination-write-failed';
                storage.setItem(pending[p], pendingEntry.raw);
                stage = 'destination-verification-failed';
                if (storage.getItem(pending[p]) !== pendingEntry.raw) return rollback(storage, created, false, stage);
            }
            stage = 'source-verification-failed';
            for (var e = 0; e < entries.length; e += 1) {
                if (storage.getItem(entries[e].sourceKey) !== entries[e].raw) return rollback(storage, created, false, stage);
            }
            stage = 'marker-write-failed';
            markerAttempted = true;
            storage.setItem(MARKER_KEY, MARKER_VALUE);
            stage = 'marker-verification-failed';
            if (storage.getItem(MARKER_KEY) !== MARKER_VALUE) return rollback(storage, created, markerAttempted, stage);
            return status('migrated', null, ensured);
        } catch (error) {
            return rollback(storage, created, markerAttempted, quotaFailure(error) ? 'quota-exceeded' : stage);
        }
    }

    function createLifecyclePersistence(storage) {
        var routed = false;
        var current = status('inactive', null, []);
        function destination(key) {
            return routed && Object.prototype.hasOwnProperty.call(DESTINATION_KEYS, key) ? DESTINATION_KEYS[key] : key;
        }
        return {
            activate: function (typed) {
                if (!typed) {
                    routed = false;
                    current = status('inactive', null, []);
                    return cloneStatus(current);
                }
                current = migrateStorage(storage);
                routed = current.state === 'migrated' || current.state === 'current';
                return cloneStatus(current);
            },
            migrate: function () { return migrateStorage(storage); },
            status: function () { return cloneStatus(current); },
            getItem: function (key) { return storage.getItem(destination(key)); },
            setItem: function (key, value) {
                if (current.state === 'error' && Object.prototype.hasOwnProperty.call(DESTINATION_KEYS, key)) return;
                storage.setItem(destination(key), value);
            },
            removeItem: function (key) {
                if (current.state === 'error' && Object.prototype.hasOwnProperty.call(DESTINATION_KEYS, key)) return;
                storage.removeItem(destination(key));
            },
        };
    }

    var storage;
    try {
        storage = globalThis.localStorage;
    } catch (_error) {
        storage = {
            getItem: function () { throw new Error('Storage unavailable'); },
            setItem: function () { throw new Error('Storage unavailable'); },
            removeItem: function () { throw new Error('Storage unavailable'); },
        };
    }
    var defaultPersistence = createLifecyclePersistence(storage);
    var typed = new URLSearchParams(globalThis.location && globalThis.location.search ? globalThis.location.search : '').get('typed') === '1';
    defaultPersistence.activate(typed);

    globalThis.UORStudioPersistence = {
        LIFECYCLE_SINGLETON: true,
        MIGRATION_KEYS: MIGRATION_KEYS,
        DESTINATION_KEYS: DESTINATION_KEYS,
        LIFECYCLE_MIGRATION_MARKER_KEY: MARKER_KEY,
        LIFECYCLE_MIGRATION_MARKER_VALUE: MARKER_VALUE,
        readMigrationSnapshot: readMigrationSnapshot,
        createCopyForwardPlan: createCopyForwardPlan,
        migrateStorage: migrateStorage,
        createLifecyclePersistence: createLifecyclePersistence,
        activate: function (requested) { return defaultPersistence.activate(requested); },
        status: function () { return defaultPersistence.status(); },
        getItem: function (key) { return defaultPersistence.getItem(key); },
        setItem: function (key, value) { defaultPersistence.setItem(key, value); },
        removeItem: function (key) { defaultPersistence.removeItem(key); },
    };
}());
