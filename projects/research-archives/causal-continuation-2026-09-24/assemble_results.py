from pathlib import Path
import json,hashlib,statistics,subprocess,datetime
r=Path('/Users/casey.allard/uor-r4-investigations/causal-20260924T000007');w=Path('/Users/casey.allard/uor-r4-worktrees/observer-transport-20260923')
def read(name):return json.loads((r/name).read_text())
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
roots=['geometry-1','relative-control-1','cost-1','ordinary-1','intervened-1','intervened-replay-1','final-parent-1','final-ordinary-1','final-intervened-1','cost-idle-parent','cost-idle-intervened']
reports={name:read(name+'/receipt.json') for name in roots}
source={name:read(name+'/causal-source.json') for name in roots if (r/name/'causal-source.json').exists()}
ver=read('native-verification.json');assert len(ver['verified_reports'])==len(roots)
assert sha(r/'intervened-1/selected.tlx')=='1ac700658a476e356ff64db95003277f30c27b3d181ca183444dd7621c5d955f'
cost={}
for name in ['cost-1','cost-idle-parent','cost-idle-intervened']:
 a=reports[name]['result'];rounds=a['interleaved_rounds'];med=[statistics.median(x['microseconds_per_token']for x in rr)for rr in rounds]
 cost[name]={'median_microseconds_per_token':med,'original_over_compiled':med[0]/med[1],'score_comparisons':a['all_row_comparisons'],'generated_sequences_equal':a['all_generated_sequences_equal'],'logical_compiled_plan_bytes':a['compiled_plan_bytes'],'nonzeros':a['compiled_nonzeros']}
final={}
for name in ['parent','ordinary','intervened']:
 a=reports['final-'+name+'-1']['result'];final[name]={'score':a['score'],'source_identity':a['final_sources'],'repository_bits':a['repository']['bits']}
summary={'schema':'uor-r4.causal-continuation-summary/1','recorded_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),
 'base_commit':'7fb232ad55d925c5442bb75a9d73a13967aa1f9a','experiment_commit':'4af04cf56f213df40b78c73a7c51ed5274517857','experiment_executable_sha256':sha(r/'causal-lexical'),
 'history_review':{'main_commit_count':88,'since':'2026-09-19T00:00:07Z','scope':'Every main commit message read; relevant current source and handoffs traced; not an exhaustive line-by-line review of every historical patch.'},
 'reports':reports,'source_bindings':source,'cost_summary':cost,'final_source_summary':final,
 'resume_verification':read('resume-verification.json'),'generated_rust':read('generated-rust-results.json'),
 'native_verification':ver,'instruction_scope':read('instruction-audit.json'),'cleanup':read('cleanup-receipt.json'),
 'prospective_plans':{name:read(name)for name in ['projection.json','data-plan.json','fitting-protocol.json','final-evaluation-freeze.json']},
 'receipt_sha256':{name:sha(r/name/'receipt.json')for name in roots},
 'limits':['No global serving default changed','Crossed withheld combinations are gradient-withheld but used as an exposed selection gate','Historical grounding cases are regression data','No working general dialogue/code generation','Strong ordinary geometric control matches Q8','Pre-tokenized prompt/generation timing is not full application or energy measurement','Successful read-plan call is allocation-free; recurrence and preparation still allocate']}
(r/'study-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps({'reports':len(roots),'bytes':(r/'study-summary.json').stat().st_size,'cost':cost,'final':{k:v['score']['bits']for k,v in final.items()}},indent=2))
