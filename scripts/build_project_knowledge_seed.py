#!/usr/bin/env python3
"""Prepare provenance-preserving local knowledge imports from an explicit audit.

This reads files and writes JSONL; it does not fetch, publish, execute upstream
code, mutate GitHub, or import data into the live database. Private sources are
written separately. Pass --antigravity-brain only for an authorized history import.
"""

import argparse
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re


def digest(text):
    return hashlib.sha256(text.encode()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--audit-root', type=Path, required=True)
    parser.add_argument('--project-root', type=Path, required=True)
    parser.add_argument('--output-root', type=Path, required=True)
    parser.add_argument('--antigravity-brain', type=Path)
    args = parser.parse_args()
    audit, project, output = args.audit_root, args.project_root, args.output_root
    output.mkdir(parents=True, exist_ok=True)
    os.chmod(output, 0o700)
    now = datetime.now(timezone.utc).isoformat()
    records, edges = {}, []

    def item(kind, title, body, origin, revision, status, visibility='public'):
        row = dict(kind=kind, title=title, body=body, origin=origin,
                   revision=revision, visibility=visibility, evidence_status=status)
        identity = digest(json.dumps(row, sort_keys=True, ensure_ascii=False))
        row.update(id='kb:' + identity, content_sha256=digest(body), collected_at=now)
        records.setdefault(row['id'], row)
        return row['id']

    def edge(source, relation, target, basis, visibility='public'):
        edges.append(dict(source=source, relation=relation, target=target,
                          basis=basis, visibility=visibility))

    def document(kind, title, body, origin, revision, status, visibility='public'):
        # Line-preserving bounded chunks, each with the full file's revision.
        def locator(line):
            return origin if '#' in origin else origin + f'#L{line}'

        ids, lines, start, size = [], [], 1, 0
        for lineno, line in enumerate(body.splitlines(keepends=True), 1):
            if lines and size + len(line) > 18000:
                ids.append(item(kind, f'{title} [lines {start}-{lineno-1}]', ''.join(lines),
                                locator(start), revision, status, visibility))
                lines, start, size = [], lineno, 0
            lines.append(line)
            size += len(line)
        if lines:
            ids.append(item(kind, f'{title} [lines {start}-{start+len(lines)-1}]', ''.join(lines),
                            locator(start), revision, status, visibility))
        return ids

    catalog = json.loads((project/'docs/integration/source-catalog.json').read_text())
    repo_ids = {}
    for row in catalog['repositories']:
        repo_ids[row['full_name']] = item('repository', row['full_name'],
            json.dumps(row, ensure_ascii=False, indent=2), row['html_url'],
            row.get('default_branch_sha') or 'UNRESOLVED_HEAD', row['coverage_label'])

    issues = json.loads((audit/'product/roadmap-issues-raw.json').read_text())
    issue_ids = {}
    for row in issues['issues']:
        number = row['number']
        meta = {k:v for k,v in row.items() if k not in ('body', 'comments')}
        root = item('issue', f"#{number} {row['title']}",
                    json.dumps(meta, ensure_ascii=False, indent=2), row['url'],
                    row['updatedAt'], 'NATIVE_SNAPSHOT')
        issue_ids[number] = root
        for chunk in document('issue_body', f"#{number} {row['title']}", row['body'],
                              row['url'], row['updatedAt'], 'NATIVE_SNAPSHOT'):
            edge(chunk, 'part_of', root, 'Body from this native issue snapshot')
        for comment in row.get('comments', {}).get('nodes', []):
            for chunk in document('issue_comment', f"#{number} completion/context comment",
                                  comment['body'], comment['url'], comment['createdAt'], 'NATIVE_SNAPSHOT'):
                edge(chunk, 'part_of', root, 'Native issue comment; date and source URL retained')
    # Native relationships are read from the compact graph, not inferred from prose.
    state = json.loads((audit/'product/roadmap-state.json').read_text())
    for row in state['issues']:
        for blocker in row.get('blocked_by', []):
            n = blocker['number'] if isinstance(blocker, dict) else blocker
            if n in issue_ids:
                edge(issue_ids[row['number']], 'blocked_by', issue_ids[n], 'Native GitHub blockedBy relationship at audit')
        parent = row.get('parent')
        if isinstance(parent, dict) and parent.get('number') in issue_ids:
            edge(issue_ids[row['number']], 'child_of', issue_ids[parent['number']], 'Native GitHub parent relationship at audit')

    for source in state['planning_documents']:
        path = audit/'product/planning-docs'/source['path']
        text = path.read_text()
        if digest(text) != source['sha256']:
            raise ValueError(f"Planning source changed: {source['path']}")
        document('planning_snapshot', source['path'], text,
                 f"https://github.com/UOR-Foundation/uor-r4/blob/{state['source_main_sha']}/{source['path']}",
                 state['source_main_sha'], 'PINNED_REPOSITORY_DOCUMENT')

    manifest = json.loads((audit/'uor/source-manifest.json').read_text())
    for source in manifest:
        path = audit/'uor/sources'/source['repo'].replace('/', '__')/'files'/source['path']
        raw = path.read_bytes()
        if len(raw) != source['bytes'] or hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest() != source['blob_sha']:
            raise ValueError(f"Pinned source mismatch: {source['repo']} {source['path']}")
        for chunk in document('upstream_source', source['repo'] + '/' + source['path'], raw.decode(),
                              source['source_url'], source['commit'], 'PINNED_SOURCE_NOT_EXECUTED'):
            edge(chunk, 'part_of', repo_ids[source['repo']], 'Pinned Git source file in selected source manifest')

    research_manifest = json.loads((audit/'research/snapshot-manifest.json').read_text())
    verified_research_paths = set()
    for source in research_manifest:
        path = audit/'research'/source['path']
        raw = path.read_bytes()
        if len(raw) != source['bytes'] or hashlib.sha256(raw).hexdigest() != source['sha256']:
            raise ValueError(f"Research source changed: {source['path']}")
        verified_research_paths.add(path.resolve())
    research = json.loads((audit/'research/research-triage.json').read_text())
    for row in research['repositories']:
        rid = item('source_review', row['repo'] + ' integration assessment', json.dumps(row, ensure_ascii=False, indent=2),
                   row['url'], row['head_sha'], 'SELECTED_SOURCE_REVIEW')
        edge(rid, 'reviews', repo_ids[row['repo']], 'Explicit source review; no new upstream execution')
        source_root = audit/'research/snapshots'/row['repo'].replace('/', '__')
        for path in sorted(source_root.rglob('*')):
            if not path.is_file() or path.suffix.lower() not in ('.md', '.rs', '.py', '.lean', '.g', '.toml'):
                continue
            if path.resolve() not in verified_research_paths:
                raise ValueError(f'Unmanifested research source: {path}')
            rel = path.relative_to(source_root).as_posix()
            for chunk in document('upstream_source', row['repo'] + '/' + rel, path.read_text(),
                         f"https://github.com/{row['repo']}/blob/{row['head_sha']}/{rel}", row['head_sha'], 'PINNED_SOURCE_NOT_EXECUTED'):
                edge(chunk, 'part_of', repo_ids[row['repo']], 'Selected local source snapshot; execution not claimed')

    paths = [project/'docs/uor_productization_integration_plan.md'] + sorted((project/'docs/integration').glob('*.md'))
    paths += sorted((project/'docs/integration').glob('*ledger.json'))
    for path in paths:
        text = path.read_text()
        document('project_plan' if path.name == 'uor_productization_integration_plan.md' else 'project_review',
                 path.name, text, str(path.resolve()), 'sha256:' + digest(text), 'LOCAL_PROPOSAL_AND_AUDIT')

    ledger_path = project/'docs/integration/claim-ledger.json'
    if ledger_path.exists():
        ledger = json.loads(ledger_path.read_text())
        source_ids = {}
        for name, source in ledger['sources'].items():
            source_ids[name] = item('evidence_reference', name, json.dumps(source, indent=2),
                                    source['url'], ledger['source_revision'], 'PINNED_EVIDENCE_REFERENCE')
        for claim in ledger['claims']:
            cid = item('claim', claim['id'], json.dumps(claim, ensure_ascii=False, indent=2),
                       str(ledger_path.resolve()), 'sha256:' + digest(ledger_path.read_text()), claim['evidence_status'])
            for reference in claim.get('evidence_refs', []):
                edge(cid, 'cites', source_ids[reference], 'Explicit claim-ledger evidence reference; support is limited to the claim scope/status')

    restricted = audit/'uor/restricted/inventory.json'
    if restricted.exists():
        text = restricted.read_text()
        document('restricted_inventory', 'Restricted UOR ecosystem inventory', text,
                 str(restricted.resolve()), 'sha256:' + digest(text), 'DISCOVERY_ONLY', 'private')
    imported_history = []
    if args.antigravity_brain:
        for folder in ('81b7d007-788b-413b-b488-b10c2b026d57', 'd7fb41f0-238c-4760-903d-1b02ed6b58ca'):
            for path in sorted((args.antigravity_brain/folder).glob('*.md')):
                text = path.read_text()
                if not re.search(r'uor|riemannian|r4', text, re.I):
                    continue
                document('provider_history', 'Antigravity ' + folder + '/' + path.name,
                         text, str(path.resolve()), 'sha256:' + digest(text), 'HISTORICAL_UNVERIFIED', 'private')
                imported_history.append({'path':str(path.resolve()), 'sha256':digest(text)})

    for visibility in ('public', 'private'):
        selected = [r for r in records.values() if r['visibility'] == visibility]
        selected += [e for e in edges if e['visibility'] == visibility]
        path = output/(visibility + '.jsonl')
        with path.open('w') as stream:
            for row in selected:
                stream.write(json.dumps(row, ensure_ascii=False) + '\n')
        os.chmod(path, 0o600)
    receipt = {'schema':'uor-knowledge-seed-v1', 'created_at':now,
               'records':len(records), 'edges':len(edges),
               'coverage':dict(Counter(r['visibility'] + ':' + r['kind'] for r in records.values())),
               'history_documents':imported_history,
               'inputs':{'audit_root':str(audit.resolve()), 'project_root':str(project.resolve())},
               'outputs':{name:hashlib.sha256((output/name).read_bytes()).hexdigest() for name in ('public.jsonl','private.jsonl')}}
    (output/'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    os.chmod(output/'receipt.json', 0o600)
    print(json.dumps({k:v for k,v in receipt.items() if k not in ('history_documents','inputs')}, indent=2))


if __name__ == '__main__':
    main()
