#!/usr/bin/env python3
"""Read live GitHub issues and local planning docs into an explicit knowledge import.

Uses authenticated gh for reads only. --ingest explicitly writes the local index;
this command never changes GitHub, runs model code, or schedules future work.
"""

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import subprocess


def run_json(args, payload=None):
    result = subprocess.run(args, input=json.dumps(payload) if payload else None,
                            text=True, capture_output=True, check=True)
    return json.loads(result.stdout)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', default='UOR-Foundation/uor-r4')
    parser.add_argument('--repo-root', type=Path, default=Path.cwd())
    parser.add_argument('--output-root', type=Path, required=True)
    parser.add_argument('--open-issues', action='store_true')
    parser.add_argument('--issue', type=int, action='append', default=[])
    parser.add_argument('--ingest', action='store_true')
    args = parser.parse_args()
    if not args.open_issues and not args.issue:
        parser.error('select --open-issues and/or explicit --issue numbers')
    owner, repo = args.repo.split('/', 1)
    root, output = args.repo_root.resolve(), args.output_root.resolve()
    output.mkdir(parents=True, exist_ok=True)
    os.chmod(output, 0o700)
    numbers = set(args.issue)
    if args.open_issues:
        found = run_json(['gh','issue','list','--repo',args.repo,'--state','open',
                          '--limit','1000','--json','number'])
        if len(found) >= 1000:
            raise RuntimeError('Open issue listing reached its cap; paginate explicitly')
        numbers.update(row['number'] for row in found)
    if any(n <= 0 for n in numbers):
        raise ValueError('Issue numbers must be positive')
    fields = '''number title state url updatedAt closedAt body
      assignees(first:100){nodes{login} pageInfo{hasNextPage}}
      milestone{number title state} parent{number title state url}
      subIssues(first:100){nodes{number title state url} pageInfo{hasNextPage}}
      blockedBy(first:100){nodes{number title state url} pageInfo{hasNextPage}}
      comments(last:2){nodes{url createdAt body}}'''
    issues = []
    # Batches avoid an unbounded GraphQL query. Connections fail closed at caps.
    ordered = sorted(numbers)
    for start in range(0, len(ordered), 20):
        query = 'query { repository(owner:' + json.dumps(owner) + ',name:' + json.dumps(repo) + ') {'
        query += 'isPrivate nameWithOwner '
        query += ''.join(f'i{n}:issue(number:{n}){{{fields}}}' for n in ordered[start:start+20])
        query += '}}'
        response = run_json(['gh','api','graphql','--input','-'], {'query':query})
        if response.get('errors'):
            raise RuntimeError('GitHub returned GraphQL errors')
        repository=response['data']['repository']
        if repository is None or repository['isPrivate']:
            raise RuntimeError('This public knowledge sync rejects private or unavailable repositories')
        if repository['nameWithOwner'].casefold()!=args.repo.casefold():
            raise RuntimeError('Repository identity differs from the requested source')
        for n in ordered[start:start+20]:
            row=repository[f'i{n}']
            if row is None:
                raise RuntimeError('Requested issue unavailable')
            if any(row[k]['pageInfo']['hasNextPage'] for k in ('assignees','subIssues','blockedBy')):
                raise RuntimeError(f"Native relationship cap reached for #{row['number']}")
            issues.append(row)
    now = datetime.now(timezone.utc).isoformat()
    head = subprocess.check_output(['git','-C',str(root),'rev-parse','HEAD'], text=True).strip()
    state = {'schema':'uor-native-snapshot-v1','collected_at':now,'repository':args.repo,
             'local_head':head,'selection':{'open_issues':args.open_issues,'explicit':args.issue},
             'issues':issues,'notes':['Latest two comments per selected issue; not complete comment history.',
                                     'Local documents are content-addressed checkout snapshots, not necessarily merged files.']}
    (output/'native-snapshot.json').write_text(json.dumps(state,indent=2)+'\n')
    os.chmod(output/'native-snapshot.json',0o600)
    records, edges = {}, []

    def item(kind,title,body,origin,revision,status):
        value=dict(kind=kind,title=title,body=body,origin=origin,revision=revision,
                   evidence_status=status,visibility='public')
        value['id']='kb:'+hashlib.sha256(json.dumps(value,sort_keys=True,ensure_ascii=False).encode()).hexdigest()
        value.update(collected_at=now,content_sha256=hashlib.sha256(body.encode()).hexdigest())
        records[value['id']]=value
        return value['id']

    def edge(source,relation,target,basis):
        edges.append(dict(source=source,relation=relation,target=target,basis=basis,visibility='public'))

    def document(kind,title,text,origin,revision,status):
        ids=[]
        lines=text.splitlines(keepends=True)
        first=0
        while first<len(lines):
            last=first; length=0
            while last<len(lines) and (last==first or length+len(lines[last])<=18000):
                length+=len(lines[last]);last+=1
            # Preserve an issue-comment anchor; line ranges are local extraction locators.
            url=origin if '#' in origin else origin+f'#L{first+1}'
            ids.append(item(kind,f'{title} [lines {first+1}-{last}]',''.join(lines[first:last]),url,revision,status))
            first=last
        return ids

    ids={}
    for row in issues:
        meta={k:v for k,v in row.items() if k not in ('body','comments')}
        ids[row['number']]=item('issue',f"#{row['number']} {row['title']}",json.dumps(meta,indent=2),
                                 row['url'],row['updatedAt'],'NATIVE_SNAPSHOT')
        for chunk in document('issue_body',f"#{row['number']} {row['title']}",row['body'],row['url'],row['updatedAt'],'NATIVE_SNAPSHOT'):
            edge(chunk,'part_of',ids[row['number']],'Body from this live native issue snapshot')
        for comment in row['comments']['nodes']:
            for chunk in document('issue_comment',f"#{row['number']} latest comment",comment['body'],comment['url'],comment['createdAt'],'NATIVE_SNAPSHOT'):
                edge(chunk,'part_of',ids[row['number']],'Latest native comment; not the complete history')
    for row in issues:
        for blocker in row['blockedBy']['nodes']:
            if blocker['number'] in ids:
                edge(ids[row['number']],'blocked_by',ids[blocker['number']], 'Native GitHub blockedBy relationship at this snapshot')
        parent=row['parent']
        if parent and parent['number'] in ids:
            edge(ids[row['number']],'child_of',ids[parent['number']], 'Native GitHub parent relationship at this snapshot')
    fixed_paths=['AGENTS.md','README.md','ROADMAP.md',
                 'docs/r4_intelligence_completion_plan.md',
                 'docs/geometric_intelligence_programme.md',
                 'docs/uor_productization_integration_plan.md',
                 'docs/integration/claim-ledger.json','docs/integration/adopted-issues.json',
                 'docs/integration/tooling-status.json','docs/integration/source-catalog.json']
    paths=[root/p for p in fixed_paths]+sorted((root/'docs/integration').glob('*.md'))
    for p in paths:
        if not p.exists():
            continue
        if not p.resolve().is_relative_to(root):
            raise RuntimeError(f'Public document symlink escapes the selected checkout: {p}')
        text=p.read_text();rev='sha256:'+hashlib.sha256(text.encode()).hexdigest()
        document('current_plan_document',p.relative_to(root).as_posix(),text,str(p),rev,'LOCAL_CHECKOUT_SNAPSHOT')
    ledger_path=root/'docs/integration/claim-ledger.json'
    if ledger_path.exists():
        text=ledger_path.read_text();ledger=json.loads(text)
        refs={}
        for key,source in ledger['sources'].items():
            refs[key]=item('evidence_reference',key,json.dumps(source,indent=2),source['url'],
                           ledger['source_revision'],'PINNED_EVIDENCE_REFERENCE')
        for claim in ledger['claims']:
            cid=item('claim',claim['id'],json.dumps(claim,indent=2,ensure_ascii=False),str(ledger_path),
                     'sha256:'+hashlib.sha256(text.encode()).hexdigest(),claim['evidence_status'])
            for reference in claim.get('evidence_refs',[]):
                edge(cid,'cites',refs[reference],'Explicit claim-ledger evidence reference; support is limited to the claim scope/status')
    batch=output/'public.jsonl'
    batch.write_text(''.join(json.dumps(r,ensure_ascii=False)+'\n' for r in list(records.values())+edges))
    os.chmod(batch,0o600)
    receipt={'schema':'uor-knowledge-sync-v1','collected_at':now,'selected_issues':len(issues),
             'records':len(records),'edges':len(edges),'import_sha256':hashlib.sha256(batch.read_bytes()).hexdigest(),
             'github_mutations':0,'local_ingest_requested':args.ingest}
    if args.ingest:
        receipt['ingest']=run_json(['uor-knowledge','ingest',str(batch)])
    (output/'sync-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    os.chmod(output/'sync-receipt.json',0o600)
    print(json.dumps(receipt,indent=2))


if __name__=='__main__':
    main()
