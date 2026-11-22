import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import {test} from 'node:test';

const host = fs.readFileSync(new URL('../shell/host.js', import.meta.url), 'utf8');
const start = host.indexOf('async function appStateOf('), end = host.indexOf('const approve = async', start);
assert.ok(start >= 0 && end > start, 'stateful approval and proof boundaries must remain present');
const block = host.slice(start, end);
function harness({phase = 'checked', race, mutate = source => source} = {}) {
  const snapshot = (revision) => ({reference: revision, applicationState:{sourceKappa:'identity',phase,approvalId:revision}, files: [['prism/App.lex.tex', 'module'], ['prism/app.json', JSON.stringify({sourceKappa: 'identity', phase})]]});
  let current = snapshot('original');
  const original = current, log = {writes: 0, proofs: 0, audits: 0, refreshes: 0};
  const environment = {
    Map, JSON, TextEncoder, crypto, enc: new TextEncoder(), core: {}, dbReady: Promise.resolve({}),
    captureProjectSource: async () => current,
    applicationShape: () => 'stateful', boundApplicationState: async source => source.applicationState,
    get: async (_store, key) => new Map(current.files).get(key.replace(/^1:/, '')),
    kappa: async () => 'identity', now: () => 'now',
    appShape: {SOURCE_PATH: 'prism/App.lex.tex', STATE_PATH: 'prism/app.json', isAppShape: () => true, theoremsOf: () => [{}], promisesOf: () => [], provingMessage: () => 'proving', proofMessage: () => '', semanticData: () => ({})},
    appModel: async () => ({declarations: []}), kernelReady: Promise.resolve({}),
    prism: {proveModule: async () => {log.proofs++; return {ok: true, record: {id: 'attestation'}};}},
    nextPhase: async (_phase, event) => {if (event === race) current = snapshot('competing-edit'); return event === 'approve' ? 'proving' : 'proved';},
    applyStoredProjectPatch: async (_db, _app, _patch, options) => {
      if (options.source.reference !== current.reference) throw Object.assign(Error('source changed'), {code: 'projectEditConflict'});
      log.writes++; return {};
    },
    refreshApp: async () => log.refreshes++, put: async () => log.audits++,
  };
  return {...vm.runInNewContext(mutate(block) + ';({appStateOf,approveApp,proveApp})', environment), original, log};
}
test('stateful approval reads its module and standing from one captured project snapshot', async () => {
  const h = harness(), app = await h.appStateOf(1);
  assert.equal(app.captured, h.original);
  assert.equal(app.source, 'module'); assert.equal(app.state.phase, 'checked');
  assert.equal(typeof (await h.approveApp(1, app)).prove, 'function');
  assert.equal(h.log.writes, 1); assert.equal(h.log.refreshes, 1);
});
async function approvalRace(mutate) {
  const h = harness({race: 'approve', mutate}), app = await h.appStateOf(1);
  const result = await h.approveApp(1, app);
  assert.match(result.text, /changed before approval/);
  assert.equal(result.prove, undefined); assert.equal(h.log.writes, 0); assert.equal(h.log.refreshes, 0);
}
test('stateful approval refuses a concurrent source change instead of recapturing it', async () => {
  await approvalRace();
  await assert.rejects(approvalRace(source => source.replace('], app.captured,', '], await captureProjectSource(await dbReady, appId),')), {name: 'AssertionError'});
});
async function proofRace(mutate) {
  const h = harness({phase: 'proving', race: 'attest', mutate});
  assert.equal(await h.proveApp(1, 'identity', 'module'), null);
  assert.equal(h.log.proofs, 1); assert.equal(h.log.writes, 0); assert.equal(h.log.audits, 0); assert.equal(h.log.refreshes, 0);
}
test('stateful proof completion refuses a competing commit without writing source or proof', async () => {
  await proofRace();
  await assert.rejects(proofRace(source => { const at = source.indexOf('async function proveApp('); return source.slice(0, at) + source.slice(at).replace('], latest.captured,', '], await captureProjectSource(await dbReady, appId),'); }), {name: 'AssertionError'});
});
test('a stateful proof whose source or phase is already stale sends no request', async () => {
  for (const [phase, key, source] of [['checked', 'identity', 'module'], ['proving', 'other', 'module'], ['proving', 'identity', 'other']]) {
    const h = harness({phase});
    assert.equal(await h.proveApp(1, key, source), null);
    assert.equal(h.log.proofs, 0); assert.equal(h.log.writes, 0);
  }
});

function turnHarness(phase, mutate = source => source) {
  const begin = host.indexOf('async function appAfterTurn('), end = host.indexOf('const afterTurn =', begin);
  assert.ok(begin >= 0 && end > begin);
  const known = {phase, sourceKappa: 'identity', ...(phase === 'drafting' ? {unchecked: 'permission denied'} : {})};
  const source = {reference: 'captured', applicationState:known, files: [['prism/App.lex.tex', 'module\n'], ['prism/app.json', JSON.stringify(known,null,2)+'\n']]};
  const log = {checks: 0, writes: 0};
  const environment = {
    Map, JSON, enc: new TextEncoder(), coreReady: Promise.resolve(), dbReady: Promise.resolve({}), core: {},
    retiredApplicationPaths:()=>[], boundApplicationState:async source=>source.applicationState, lexlean:{canonicalSource(){}}, rulesSnapshotOf:async()=>({expected:undefined}), forgetApplicationFrames(){},
    appShape: {moduleFromTurn: () => ({text: 'module'}), isAppShape: () => true, SOURCE_PATH: 'prism/App.lex.tex', STATE_PATH: 'prism/app.json', card: state => state},
    assertGenerationActive() {}, kappa: async () => 'identity', now: () => 'now',
    nextPhase: async (_phase, event) => event === 'check' ? 'checked' : 'drafting',
    checkAppModule: async () => {log.checks++; return {ok: true, issues: []};},
    applyStoredProjectPatch: async (_db, _app, patch, options) => {assert.equal(options.source, source); log.writes++; return {changed: patch.writes.length, source};},
  };
  return {run: vm.runInNewContext(mutate(host.slice(begin, end)) + ';appAfterTurn', environment), source, log};
}
async function retryDraft(mutate) {
  const h = turnHarness('drafting', mutate), result = await h.run(1, '', {source: h.source});
  assert.equal(h.log.checks, 1); assert.equal(result.card.phase, 'checked');
  assert.equal(result.card.unchecked, undefined); assert.equal(h.log.writes, 1);
}
test('a later approved turn rechecks identical previously denied draft source', async () => {
  await retryDraft();
  await assert.rejects(retryDraft(source => source.replace(' && ["checked", "proving", "proved", "published"].includes(known.phase)', '')), {name: 'AssertionError'});
});
test('identical checked proving proved and published state retains its standing', async () => {
  for (const phase of ['checked', 'proving', 'proved', 'published']) {
    const h = turnHarness(phase), result = await h.run(1, '', {source: h.source});
    assert.equal(result.card.phase, phase); assert.equal(h.log.checks, 0); assert.equal(h.log.writes, 0);
  }
});

function frameBoundary() {
  const begin = host.indexOf('async function settleAppFrames('), end = host.indexOf('function pushLive(', begin);
  assert.ok(begin >= 0 && end > begin, 'settlement must use the production frame boundary');
  const transition = host.slice(host.indexOf('// T6: completed batches'), host.indexOf('const sm = full.match', host.indexOf('// T6: completed batches')));
  assert.ok(transition.includes('await settleAppFrames(appId, abort.signal)'), 'streaming work settles before source writes');
  return {helper: host.slice(begin, end), transition};
}
async function commitAfterFrames({cancel = false, mutate = source => source} = {}) {
  const {helper, transition} = frameBoundary();
  let finish, commits = 0;
  const abort = new AbortController();
  const pending = new Promise(resolve => {finish = resolve;});
  const run = vm.runInNewContext(helper + ';async function run(){' + mutate(transition) + ';commit();}run', {
    appFrames: Promise.resolve({settled: () => pending}), appApps: new Set([1]), appId: 1, execution: {sourceEffects: true}, abort,
    assertGenerationActive: signal => {if (signal?.aborted) throw Object.assign(Error('cancelled'), {code: 'generationCancelled'});},
    commit: () => {commits++;},
  });
  const result = run();
  await new Promise(setImmediate);
  assert.equal(commits, 0, 'CAS cannot advance source while a streaming parser still owns the previous snapshot');
  if (cancel) abort.abort();
  finish();
  if (cancel) {await assert.rejects(result, {code: 'generationCancelled'}); assert.equal(commits, 0);}
  else {await result; assert.equal(commits, 1);}
}
test('streaming frames settle before source commits and late cancellation prevents writes', async () => {
  await commitAfterFrames(); await commitAfterFrames({cancel: true});
  await assert.rejects(commitAfterFrames({mutate: source => source.replace('await settleAppFrames(appId, abort.signal)', 'void 0')}), {name: 'AssertionError'});
});
test('failed preview initialization still reaches chat completion', async () => {
  const {helper} = frameBoundary();
  const begin = host.indexOf('    try { if (execution.sourceEffects && !abort.signal.aborted)', host.indexOf('// T6: completed batches'));
  const end = host.indexOf('\n  m.abort = null;', begin);
  assert.ok(begin >= 0 && end > begin);
  // Exclude the brace belonging to chatTurn's outer finally, not its cleanup.
  const cleanup = host.slice(begin, end).replace(/\n  \}\s*$/, '');
  const run = vm.runInNewContext(helper + ';async function run(){let error=null;' + cleanup + ';return {error,completed:true};}run', {
    appFrames: Promise.resolve().then(() => {throw Error('preview unavailable');}), appApps: new Set([1]), appId: 1,
    execution: {sourceEffects: true}, abort: new AbortController(), assertGenerationActive() {},
  });
  const result = await run();
  assert.equal(result.completed, true); assert.match(result.error, /preview unavailable/);
  const beginFrames = host.indexOf('let appFrames = null;'), endFrames = host.indexOf('// The checker for a stateful app', beginFrames);
  let requests = 0;
  const ready = vm.runInNewContext(host.slice(beginFrames, endFrames) + ';appFramesReady', {
    URL, BASE:'https://foundry.example/', postApp(){}, appShape:{appLive:()=>({ready:true})},
    fetch:async()=>{if(++requests===1)throw Error('temporary failure');return{ok:true,json:async()=>({})};},
  });
  await assert.rejects(ready(),/temporary failure/);assert.equal((await ready()).ready,true);assert.equal(requests,2);
});
