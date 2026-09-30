// Execute only the checked-in trusted validator against in-memory API fixtures.
// This makes no network calls and never executes receipt argv or PR contents.
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const workflow = path.resolve(__dirname, '../../../.github/workflows/lab-delivery.yml');
const source = fs.readFileSync(workflow, 'utf8').split('          script: |\n')[1]
  .split('\n').map(line => line.startsWith('            ') ? line.slice(12) : line).join('\n');
const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor;
const run = new AsyncFunction('github', 'context', 'core', 'require', source);
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const head = 'a'.repeat(40), base = 'b'.repeat(40), group = 'c'.repeat(40);
const ops = 'd'.repeat(40), tree = 'e'.repeat(40);
const log = Buffer.from('synthetic validation fixture output\n');
const evidence = {path: '/local/check.log', sha256: hash(log)};
let count = 0;

function setup(isGroup = false) {
  const receipt = {
    schema: 'uor-r4.delivery-receipt/1', repository: 'owner/repo', pull_request: 1,
    task_issue: 2, author_session: 'author', claim_session: 'author', claim_epoch: 1,
    work_card: 'sha256:1111111111111111111111111111111111111111111111111111111111111111', head_sha: head, base_sha: base, change_kind: 'code', class: 'A',
    checks: ['diff-check', 'format', 'compile', 'focused-tests'].map(name => ({
      name, head_sha: head, base_sha: base, argv: ['fixture'], exit_code: 0, outcome: 'PASS', log: evidence
    })),
    reviews: [{review_id: 'review-1', reviewer_session: 'reviewer', launched_by: 'other',
      head_sha: head, base_sha: base, decision: 'APPROVE', unresolved_required_fixes: 0, evidence}],
    result_evidence: null, result_reader_session: null, council: null
  };
  const envelope = {schema: 'uor-r4.server-delivery/1', receipt,
    evidence: [{local_path: evidence.path, path: 'delivery/evidence/record.log', sha256: evidence.sha256}]};
  if (isGroup) envelope.integration = {
    head_sha: group, base_sha: base, pull_request: 1, pr_head_sha: head,
    checks: [{name: 'integration-check', head_sha: group, base_sha: base,
      argv: ['fixture-integration'], exit_code: 0, outcome: 'PASS', log: evidence}]
  };
  return {
    envelope, files: ['src/helper.rs'], pr: {state: 'open', draft: false, head: {sha: head}, base: {sha: base, ref: 'main'}},
    state: {schema: 'uor-r4.lab-state/1', repository: 'owner/repo',
      labs: {author: {available: true, heartbeat: 100}},
      policy_sha: 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb', tasks: {'2': {policy_sha: 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb', session: 'author', epoch: 1, work_card: 'sha256:1111111111111111111111111111111111111111111111111111111111111111', phase: 'claimed', expires: 1300}}},
    missing: false, log, tree, groupSize: 1, policyChanges: []
  };
}

async function test(name, modify, expected, isGroup = false) {
  const fixture = setup(isGroup); modify(fixture);
  let failure;
  const statuses = [];
  const github = {
    rest: {
      repos: {
        createCommitStatus: async value => statuses.push(value.state),
        getContent: async ({path: file}) => {
          if (fixture.missing) { const error = new Error('missing'); error.status = 404; throw error; }
          const bytes = file === 'state.json' ? Buffer.from(JSON.stringify(fixture.state)) :
            file.endsWith('.json') ? Buffer.from(JSON.stringify(fixture.envelope)) : fixture.log;
          return {data: {type: 'file', encoding: 'base64', size: bytes.length, content: bytes.toString('base64')},
            headers: {date: 'Thu, 01 Jan 1970 00:01:41 GMT'}};
        }
      },
      pulls: {get: async () => ({data: fixture.pr}), listFiles: () => {}},
      git: {
        getRef: async () => ({data: {object: {sha: ops}}}),
        getCommit: async ({commit_sha}) => ({data: {parents: [{sha: base}], tree: {sha: commit_sha === group ? fixture.tree : tree}}})
      }
    },
    paginate: async () => fixture.files.map(filename => ({filename})),
    graphql: async () => ({repository: {issue: {state: 'OPEN', blockedBy: {nodes: [], pageInfo: {hasNextPage: false}}}}}),
    request: async route => route.includes('/compare/') ? {data: {status: fixture.policyChanges.length ? 'ahead' : 'identical', files: fixture.policyChanges.map(filename => ({filename}))}} : ({data: [{type: 'merge_queue', parameters: {max_entries_to_build: fixture.groupSize, max_entries_to_merge: fixture.groupSize}}]})
  };
  const context = {repo: {owner: 'owner', repo: 'repo'}, runId: 1, eventName: 'workflow_dispatch',
    payload: {inputs: {pull_request: '1', merge_group_sha: isGroup ? group : ''}}};
  const summary = {addHeading() {return this;}, addRaw() {return this;}, async write() {}};
  await run(github, context, {setFailed: message => {failure = message;}, summary}, require);
  const passed = !failure && statuses.at(-1) === 'success';
  if (passed !== expected) throw new Error(`${name}: unexpected ${passed ? 'PASS' : 'FAIL'} ${failure || ''}`);
  count += 1;
  process.stdout.write(`PASS ${name}\n`);
}

async function main() {
  await test('valid PR', () => {}, true);
  await test('missing receipt', f => {f.missing = true;}, false);
  await test('stale source head', f => {f.envelope.receipt.head_sha = group;}, false);
  await test('self review', f => {f.envelope.receipt.reviews[0].reviewer_session = 'author';}, false);
  await test('unavailable check', f => {f.envelope.receipt.checks[0].outcome = 'UNAVAILABLE';}, false);
  await test('changed evidence', f => {f.log = Buffer.from('changed');}, false);
  await test('stale task epoch', f => {f.state.tasks['2'].epoch = 2;}, false);
  await test('expired task', f => {f.state.tasks['2'].expires = 100;}, false);
  await test('unavailable steward', f => {f.state.labs.author.available = false;}, false);
  await test('unadopted operations policy', f => {f.policyChanges = ['docs/labs/operations.md'];}, false);
  for (const file of ['AGENTS.md', 'docs/labs/operations.md', 'tools/lab-runner/src/coord.rs',
    'tools/lab-runner/src/admission.rs', 'tools/lab-runner/src/delivery.rs']) {
    await test(`class A rejected: ${file}`, f => {f.files = [file];}, false);
  }
  for (const file of ['src/native_geometric_cli.rs', 'src/service.rs', 'src/native_wasm.rs', 'crates/uor-r4-api/src/serving.rs']) {
    await test(`code-only rejected: ${file}`, f => {f.files = [file];}, false);
  }
  await test('valid group', () => {}, true, true);
  await test('group tree differs', f => {f.tree = head;}, false, true);
  await test('group checks absent', f => {f.envelope.integration.checks = [];}, false, true);
  await test('multiple PR group unsupported', f => {f.groupSize = 2;}, false, true);
  process.stdout.write(`${count}/${count} data-only workflow fixtures; no network or PR code executed.\n`);
}
main().catch(error => {process.stderr.write(`${error.stack}\n`); process.exitCode = 1;});
