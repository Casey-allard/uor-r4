#!/usr/bin/env node
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { cdpConnect, launchChrome, trapPageLoad } from './verify_studio_page_load.mjs';
import { assertMigrationReport } from './verify_studio_migration.mjs';

const origin = process.env.UOR_NATIVE_ORIGIN || 'http://127.0.0.1:8000';
const profileDir = await mkdtemp(join(tmpdir(), 'uor-native-runner-'));
let chrome;
try {
    chrome = await launchChrome(profileDir);
    const cdp = await cdpConnect(chrome.wsUrl);
    const typed = await trapPageLoad(cdp, origin, '/?typed=1', { typed: true });
    if (!typed.loaded) throw new Error('native runner never fired loadEventFired');
    if (typed.external.length) throw new Error('native runner made external requests: ' + JSON.stringify(typed.external));
    if (typed.rawTypeScript.length) throw new Error('native runner requested raw TypeScript: ' + JSON.stringify(typed.rawTypeScript));
    if (typed.runtimeErrors.length) throw new Error('native runner emitted runtime errors: ' + JSON.stringify(typed.runtimeErrors));
    const checks = typed.typed;
    if (!checks?.shell || checks.shellCount !== 1 || !checks.active || !checks.legacyHidden || checks.legacyMcp !== 0 || checks.typedMcp !== 1 || checks.typedMcpAfterWorkflow !== 0 || !checks.artifactReady || !checks.visible || !checks.typedText) {
        throw new Error('native runner did not mount the typed application: ' + JSON.stringify(checks));
    }
    const flow = checks.sessionFlow;
    if (!flow?.created || !flow.beforeSend?.idMatches || !flow.beforeSend.markerMatches || !flow.answerReady || !flow.firstTranscript.includes('42') || flow.otherHasAnswer || !flow.restoredResume?.idMatches || !flow.restoredResume.markerMatches || !flow.restoredAnswer || !flow.generationComplete || !flow.restoredMatchesOriginal) {
        throw new Error('native runner session/UOR workflow failed: ' + JSON.stringify(flow));
    }
    const migrations = [];
    for (const mode of ['success', 'malformed', 'conflict', 'quota']) {
        const result = await trapPageLoad(cdp, origin, '/?typed=1', { migration: mode });
        if (!result.loaded || result.external.length || result.rawTypeScript.length || result.runtimeErrors.length) {
            throw new Error('native migration ' + mode + ' violated runtime/offline boundaries: ' + JSON.stringify(result));
        }
        assertMigrationReport(result, mode);
        migrations.push({ mode, result });
    }
    console.log(JSON.stringify({
        verdict: 'NATIVE TYPED RUNTIME PASSED',
        typed: { requests: typed.requested, external: typed.external, rawTypeScript: typed.rawTypeScript, runtimeErrors: typed.runtimeErrors.length, checks },
        migrations: migrations.map(({ mode, result }) => ({ mode, requests: result.requested, external: result.external, runtimeErrors: result.runtimeErrors.length, checks: result.migration })),
    }, null, 2));
    cdp.ws.close();
} finally {
    if (chrome?.child) chrome.child.kill('SIGTERM');
    await new Promise((resolve) => setTimeout(resolve, 500));
    await rm(profileDir, { recursive: true, force: true }).catch(() => {});
}
