from pathlib import Path
import subprocess,json,hashlib,datetime
ROOT=Path('/Users/casey.allard/.local/state/uor-r4/recovery/20260929-lab-system/continuation-20260930')
WT='/Users/casey.allard/uor-r4/.worktrees/durable-lab-system-20260929'
REPO='UOR-Foundation/uor-r4'
start=datetime.datetime.now(datetime.timezone.utc).isoformat()
records=[]
def gh(endpoint,paged=False):
 args=['gh','api',endpoint]
 if paged:args+=['--paginate','--slurp']
 value=json.loads(subprocess.check_output(args))
 return [x for page in value for x in page] if paged else value
def add(kind,identity,url,value):
 raw=json.dumps(value,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()
 records.append({'schema':'uor-r4.startup-source/1','kind':kind,'identity':identity,'url':url,'content_sha256':hashlib.sha256(raw).hexdigest(),'source':value})
sha=subprocess.check_output(['git','rev-parse','origin/main'],cwd=WT,text=True).strip()
paths=['AGENTS.md','README.md','STATUS.md','docs/labs/README.md','docs/labs/plan-2026-09-29.md','docs/labs/protocol.md','docs/labs/operations.md','docs/labs/adapters.md','docs/labs/integration-queue.md','docs/integration/project-track.md','docs/integration/current-state.md','docs/integration/DECISIONS.md','docs/integration/agent-execution-policy.md','docs/integration/agent-execution-policy.json','docs/PROJECT_MAP.md']+[f'docs/labs/prompts/{name}.md' for name in ['join','codex','claude','antigravity','opencode-deepseek']]
for path in paths:
 value=subprocess.check_output(['git','show',sha+':'+path],cwd=WT,text=True)
 add('protected_document',sha+':'+path,f'https://github.com/{REPO}/blob/{sha}/{path}',{'main_sha':sha,'path':path,'text':value,'utf8_sha256':hashlib.sha256(value.encode()).hexdigest()})
issues=[820,1508,1509,1510,1511,1512,1513,1515,1520,1525]
for n in issues:
 x=gh(f'repos/{REPO}/issues/{n}');add('issue',str(n),x['html_url'],x)
 for c in gh(f'repos/{REPO}/issues/{n}/comments?per_page=100',True):add('issue_comment',str(c['id']),c['html_url'],c)
prs=gh(f'repos/{REPO}/pulls?state=open&per_page=100',True)
for pr in prs:
 n=pr['number'];add('pull_request',str(n),pr['html_url'],pr)
 for endpoint,kind in [('issues/'+str(n)+'/comments','pr_conversation_comment'),('pulls/'+str(n)+'/reviews','submitted_review'),('pulls/'+str(n)+'/comments','inline_review_comment')]:
  for c in gh(f'repos/{REPO}/{endpoint}?per_page=100',True):
   if kind=='submitted_review' and c.get('state')=='PENDING':continue
   add(kind,str(c['id']),c.get('html_url',pr['html_url']),c)
coord=json.loads(subprocess.check_output(['/Users/casey.allard/.local/share/uor-r4/bin/lab-runner-a345e766','coord','status','/Users/casey.allard/.local/share/uor-r4/coord.git']))
add('operational_state',str(coord['sequence']),f'https://github.com/{REPO}/tree/codex/lab-state',coord)
payload=''.join(json.dumps(x,ensure_ascii=False,separators=(',',':'))+'\n' for x in records).encode()
output=ROOT/'startup-sources.jsonl';output.write_bytes(payload)
finish=datetime.datetime.now(datetime.timezone.utc).isoformat()
receipt={'schema':'uor-r4.startup-freshness/1','collected_start_utc':start,'collected_finish_utc':finish,'main_sha':sha,'policy_sha':coord['policy_sha'],'operational_sequence':coord['sequence'],'records':len(records),'documents':paths,'issue_numbers':issues,'open_pr_numbers':[p['number'] for p in prs],'sha256':hashlib.sha256(payload).hexdigest(),'bytes':len(payload),'coverage_complete':False,'coverage':['Protected authority documents and five complete goals pinned to exact main source.','Selected programme/epic/board/recovery issue bodies and all paginated ordinary comments.','Every open PR returned by paginated REST: body, conversation, submitted reviews and inline review comments.','Operational state at collected sequence.'],'gaps':['Not a complete repository/history/all-closed-issue index.','No binary/model/artifact payloads or private session material.','No GraphQL resolved-thread state, pending reviews, deleted revisions, check logs or client startup acknowledgments.','REST collection is a time-bounded nontransactional snapshot; refresh live identities before claims, experiments and delivery.'],'authority':'Rebuildable navigation/index only; GitHub originals, live claims, protected source and artifact-bound receipts remain authoritative.'}
(ROOT/'startup-freshness.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({k:receipt[k] for k in ['records','bytes','main_sha','policy_sha','open_pr_numbers','coverage_complete']}))
