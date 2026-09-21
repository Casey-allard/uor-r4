#!/usr/bin/env python3
"""Read-only seal, artifact and saved-row audit. No model execution."""
import collections, hashlib, json, pathlib, struct, subprocess
import blake3
P=pathlib.Path
BASE=P('/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20')
REPO=P('/Users/casey.allard/.codex/worktrees/takeover-research-reconciliation/uor-r4')
HEAD='ce8d9340c0f5a22de97319decf8bc18dcba4d46d'
sha=lambda b:hashlib.sha256(b).hexdigest()
load=lambda p:json.loads(p.read_text())
o={'schema':'uor-r4.consistent-emission-principal-evidence-audit/1','scope':'Read-only sealed-report integrity, binary parameter inspection, saved-row reconstruction, source-derived fixture reconstruction. No builds, model inference, fitting or sealed-root mutation. Full H4/C120 hits and per-source conditional accuracy cannot be recomputed because rows omit emitted tokens and logits.','review_revision':HEAD,'integrity':{},'attempt_comparison':{}}
rs={i:load(BASE/f'consistent-emission-{i}/result.json')for i in [1,2]}
rr={i:[json.loads(x)for x in (BASE/f'consistent-emission-{i}/rows.jsonl').read_text().splitlines()]for i in [1,2]}
exe=P('/Users/casey.allard/uor-r4/target/release/competitive-reader');exesha=sha(exe.read_bytes())if exe.exists()else None
for i,r in rs.items():
 root=BASE/f'consistent-emission-{i}';m=load(root/'manifest.json');actual={str(p.relative_to(root))for p in root.rglob('*')if p.is_file()}-{'manifest.json'};listed={x['path']for x in m['files']}
 o['integrity'][i]={'root':str(root),'listed_files':len(listed),'missing':sorted(listed-actual),'unlisted':sorted(actual-listed),'seal_errors':[x['path']for x in m['files']if len((root/x['path']).read_bytes())!=x['bytes']or blake3.blake3((root/x['path']).read_bytes()).hexdigest()!=x['blake3']],'result_sha256':sha((root/'result.json').read_bytes()),'rows_sha256':sha((root/'rows.jsonl').read_bytes()),'artifact_hashes_match':{a['arm']:{'params':sha((root/f"artifacts/{a['arm']}.rlrc").read_bytes())==a['params_sha256'],'residual':sha((root/f"artifacts/{a['arm']}.rlce").read_bytes())==a['residual_sha256']}for a in r['artifacts']},'submitted_source_matches':{x['path']:sha(subprocess.check_output(['git','show',HEAD+':'+x['path']],cwd=REPO))==x['sha256']for x in r['running_source']['source_files']},'declared_executable_sha256':r['running_source']['executable_sha256'],'retained_current_executable_sha256':exesha,'current_executable_matches':exesha==r['running_source']['executable_sha256']}
 o['attempt_comparison'][i]={'same_as_attempt2':{k:r.get(k)==rs[2].get(k)for k in ['instrument','validity','learning','comparisons','changed_source_pairs_fresh','generated','artifacts','screen','correct_source_stratum']},'elapsed_s':r['elapsed_s'],'claimed_at':load(root/'attempt.json')['claimed_at'],'sealed_at':m['sealed_at'],'rows_identical_to_attempt2':rr[i]==rr[2]}
r=rs[2]
inputs=[('E',BASE/'head-projection-3/corrected/empirical.cpl2',r['inputs']['E_sha256']),('S',BASE/'s-attribution-3/corrected/separable_older_query_read.cpx3',r['inputs']['S_sha256']),('raw_tokenizer',BASE.parent/'sources/smollm2-135m-instruct/tokenizer.json','9ca9acddb6525a194ec8ac7a87f24fbba7232a9a15ffa1af0c1224fcd888e47c'),('reader_parent',BASE/'relational-learning-4/artifacts/relational_ctx.rlr2',r['inputs']['selector_sha256'])]
o['inputs']={label:{'path':str(p),'sha256':sha(p.read_bytes()),'matches_source_or_report_declared':sha(p.read_bytes())==s}for label,p,s in inputs}
def parse_rc(path):
 b=path.read_bytes();c=4;ver,nt=struct.unpack_from('<II',b,c);c+=8;t=list(b[c:c+nt]);c+=nt;nd,=struct.unpack_from('<I',b,c);c+=4;domain=list(struct.unpack_from('<'+'I'*nd,b,c));c+=4*nd;nc,=struct.unpack_from('<I',b,c);c+=4;v=list(b[c:c+nc]);c+=nc
 groups=collections.defaultdict(list)
 for val,code in zip(domain,v):groups[code].append(val)
 return {'sha256':sha(b),'bytes':len(b),'consumed':c,'version':ver,'transport_changes_from_index_initialization':[{'relation':i,'initial':i,'final':x}for i,x in enumerate(t)if x!=i],'value_domain':domain,'value_codes':v,'injective':len(set(v))==len(v),'alias_groups':{k:v for k,v in groups.items()if len(v)>1}}
def parse_res(path):
 b=path.read_bytes();ver,width,shift,nr=struct.unpack_from('<IIII',b,4);c=20;emb=list(struct.unpack_from('<'+'b'*(nr*width),b,c));c+=nr*width;nw,=struct.unpack_from('<I',b,c);c+=4;w=list(struct.unpack_from('<'+'b'*(nw*width),b,c));c+=nw*width;rows=[i for i in range(nw)if any(w[i*width:(i+1)*width])]
 return {'sha256':sha(b),'bytes':len(b),'consumed':c,'version':ver,'width':width,'shift':shift,'r_rows':nr,'output_rows_stored':nw,'active_output_rows':rows,'nonzero_coefficients':sum(x!=0 for x in w),'embedding_sha256':sha(bytes((x%256 for x in emb))),'all_coefficients_ternary':all(-1<=x<=1 for x in w+emb)}
o['artifact_parameters']={arm:{'params':parse_rc(BASE/f'consistent-emission-2/artifacts/{arm}.rlrc'),'residual':parse_res(BASE/f'consistent-emission-2/artifacts/{arm}.rlce')}for arm in ['h4_emission','cyclic_c120_emission']}
# Source-derived fixture reconstruction only, no tokenizer/model execution.
bankpath=BASE/'competitive-reader-1/population.json';banks=load(bankpath)['banks'];vals=r['instrument']['values'];outs=r['instrument']['output_bank'];mask=(1<<64)-1
assert vals==banks['values_fit'][:8]
class Rng:
 def __init__(self,s):self.s=s
 def next(self):
  s=self.s;s^=(s<<13)&mask;s^=s>>7;s^=(s<<17)&mask;self.s=s&mask;return self.s
 def pick(self,x):return x[self.next()%len(x)]
def specs(n,seed):
 rng=Rng(seed);items=[]
 for pid in range(n):
  key=rng.pick(banks['keys']);fa,fb=rng.pick(banks['role_pairs']);qrole,srole=(fa,fb)if rng.next()&1==0 else(fb,fa);i0=rng.next()%8;i1=rng.next()%8
  if i1==i0:i1=(i0+1)%8
  pool=[p for p in banks['role_pairs']if p!=[fa,fb]and p!=[fb,fa]];blocks=[]
  for _ in range(3):
   a,b=rng.pick(pool);role=a if rng.next()&1==0 else b;v=rng.pick(vals);blocks.append((role,key,v))
  pos=rng.next()%3
  for member,vi in enumerate([i0,i1]):
   bs=blocks.copy();bs.insert(pos,(srole,key,vals[vi]));prefix=[t for b in bs for t in b]+[qrole,key];items.append({'pair':pid,'member':member,'prefix':prefix,'target':outs[vi],'source_abs':pos*3+2,'value':vals[vi],'source_role':srole,'query_role':qrole,'key':key})
 return items
pops={name:specs(n,seed)for name,n,seed in [('dev',90,0xC0F00011),('tune',60,0xC0F00012),('final',60,0xC0F00021)]}
rows={name:[x for x in rr[2]if x['split']==name]for name in pops}
rec={'scope':'Source-derived fixture reconstruction, not independently observed model outputs. Same-tokenizer role/key/value banks from sealed competitive-reader-1/population.json. No BPE, model logits or selection executed.','bank_path':str(bankpath),'bank_sha256':sha(bankpath.read_bytes()),'panels':{},'split_overlap':{}}
for name,items in pops.items():
 checks=[]
 for a,b in zip(items[::2],items[1::2]):
  diff=[i for i,(x,y)in enumerate(zip(a['prefix'],b['prefix']))if x!=y];checks.append(diff==[a['source_abs']]and a['target']!=b['target']and a['prefix'][-5:]==b['prefix'][-5:])
 counts=collections.Counter(x['target']for x in items)
 rec['panels'][name]={'positions':len(items),'pairs':len(items)//2,'rows_match_reconstructed_fixture':all(all(a[k]==b[k]for k in ['pair','value','target'])for a,b in zip(items,rows[name])),'all_pairs_change_only_relevant_payload_and_keep_last5':all(checks),'all_targets_absent_from_prefix':all(x['target']not in x['prefix']for x in items),'target_counts':dict(sorted(counts.items())),'majority_answer_hits':max(counts.values()),'exact_prefix_duplicates':len(items)-len({tuple(x['prefix'])for x in items}),'canonical_items_sha256':sha(json.dumps(items,sort_keys=True,separators=(',',':')).encode())}
fns={'complete_prefix':lambda x:tuple(x['prefix']),'query_role_key':lambda x:(x['query_role'],x['key']),'query_role_relevant_value':lambda x:(x['query_role'],x['value']),'role_key_value':lambda x:(x['query_role'],x['key'],x['value']),'relevant_value_target':lambda x:(x['value'],x['target'])}
for a,b in [('dev','tune'),('dev','final'),('tune','final')]:
 A,B=pops[a],pops[b]
 rec['split_overlap'][a+'_vs_'+b]={key:{'left_unique':len({fn(x)for x in A}),'right_unique':len({fn(x)for x in B}),'intersection':len({fn(x)for x in A}&{fn(x)for x in B})}for key,fn in fns.items()}
old={name:specs(n,seed)for name,n,seed in [('dev',90,0xC0F00001),('tune',60,0xC0F00002),('final',60,0xC0F00003)]}
rec['old_panel_exact_prefix_overlap']={a:{b:len({tuple(x['prefix'])for x in aa}&{tuple(x['prefix'])for x in bb})for b,bb in old.items()}for a,aa in pops.items()}
o['fixture_reconstruction']=rec
# Reproduce categorical fitting exactly: BTreeMap max_by_key keeps the largest token on a tie.
counts=collections.Counter(x['target']for x in rows['dev']);constant=max(counts,key=lambda t:(counts[t],t));cm=collections.defaultdict(collections.Counter)
for x in rows['dev']:cm[x['selected_payload']][x['target']]+=1
cat={p:max(cs,key=lambda t:(cs[t],t))for p,cs in cm.items()}
o['row_recomputation']={'constant':constant,'categorical_table':cat,'categorical_fit_counts':{k:dict(v)for k,v in cm.items()},'panels':{},'fields':sorted(rr[2][0])}
for name,xs in rows.items():
 def stats(fn):
  hs=[fn(x)==x['target']for x in xs];pairs=collections.defaultdict(list)
  for x,h in zip(xs,hs):pairs[x['pair']].append(h)
  strata={s:{'positions':sum(bool(x[s])for x in xs),'hits':sum(h and x[s]for x,h in zip(xs,hs))}for s in ['correct_exact_occurrence','correct_payload_value']}
  return {'hits':sum(hs),'pairs_both':sum(len(v)==2 and all(v)for v in pairs.values()),'strata':strata}
 catstats=stats(lambda x:cat.get(x['selected_payload'],constant));conststats=stats(lambda x:constant)
 correctval=sum(x['selected_payload']==x['value']for x in xs);reported= r['correct_source_stratum'][name]
 sigs=collections.defaultdict(set)
 for x in xs:sigs[(x['q0'],x['relation'],x['selected_payload'])].add(x['target'])
 code=dict(zip(o['artifact_parameters']['h4_emission']['params']['value_domain'],o['artifact_parameters']['h4_emission']['params']['value_codes']))
 col=0;colwrong=0;groups=collections.defaultdict(list)
 for x in xs:groups[(x['q0'],x['relation'])].append(x)
 for g in groups.values():
  for i,a in enumerate(g):
   for b in g[i+1:]:
    if a['target']!=b['target']and code[a['selected_payload']]==code[b['selected_payload']]:
     col+=1;colwrong+=not(a['correct_payload_value']and b['correct_payload_value'])
 o['row_recomputation']['panels'][name]={'rows':len(xs),'pairs':len({x['pair']for x in xs}),'any_read':sum(x['read']for x in xs),'correct_exact_occurrence_booleans':sum(x['correct_exact_occurrence']for x in xs),'correct_value_recomputed':correctval,'correct_value_boolean_matches':all(x['correct_payload_value']==(x['selected_payload']==x['value'])for x in xs),'source_counts_match_report':reported['positions']==len(xs)and reported['correct_exact_occurrence']==sum(x['correct_exact_occurrence']for x in xs)and reported['correct_payload_value']==correctval,'categorical':catstats,'constant':conststats,'initial_local_wrong_from_negative_margin':sum(x['target_margin']<0 for x in xs),'signature_count':len(sigs),'ambiguous_signatures':sum(len(v)>1 for v in sigs.values()),'targets_in_ambiguous_signatures':sum(len(v)for v in sigs.values()if len(v)>1),'incompatible_output_collision_pairs':col,'collision_pairs_with_at_least_one_wrong_selected_value':colwrong}
o['row_recomputation']['limitation']='Correct exact occurrence can only be recounted from saved booleans: selected seq/abs/payload_abs and intended abs are not saved. H4/C120 predictions, q1, selected refs, logits and final margins are absent, so model hits, pairs and conditional source accuracy remain reported-only. target_margin is the frozen local margin, not the learned emitter margin.'
o['retained_samples']={'generation':[{'sample':i,'first_correct':g['emitted'][0]==g['answer'],'emissions_match_steps':g['emitted']==[s['emitted']for s in g['steps']],'answer_matches_fixture':g['answer']==pops['final'][i]['target'],'reported':g}for i,g in enumerate(r['generated'])],'pair_samples':[{'pair':g['pair'],'both_correct_recomputed':g['emitted_a']==g['answer_a']and g['emitted_b']==g['answer_b'],'emissions_change':g['emitted_a']!=g['emitted_b'],'reported':g}for g in r['changed_source_pairs_fresh']]}
o['reported_model_counts_not_independently_recomputed']=r['comparisons']
o['findings']=[
 'Both sealed roots valid, seven listed files each. Delivered-source hashes match submitted revision; attempt1 runner differs as expected. Attempt2 rows, artifacts, predictions and populations repeat attempt1 identically, with collision reporting added.',
 'The final population was first evaluated in attempt1, then replayed unchanged in attempt2. No artifact/model selection change between attempts is evident. It is one new seed draw, not independent confirmation; docs should describe first measurement and replay rather than a second untouched final.',
 'All eight value codes in each delivered artifact are distinct: prior payload aliasing is actually repaired in these artifacts. Penalty code alone is not a universal injectivity constraint, since it checks only observed conflicting examples. No proof of representational impossibility follows from current optimizer outcome.',
 'Saved rows reproduce categorical 164/180,112/120,108/120 and 81/90,56/60,54/60 pairs. Categorical achieves all correct-payload positions, including occasional wrong occurrence with same value. This is a useful finite-label association control, not a cost/architecture-matched geometric baseline.',
 'Categorical emits the table answer directly, without competing against fixed local logits. H4/C120 must overcome a local target deficit with a bounded residual. The result diagnoses a large task-level integration gap but does not isolate geometry or expressivity from optimization and output-interface differences.',
 'All value-to-target associations recur across all splits; no unseen value or output-class transfer is tested. Exact prefixes differ. The task validates contextual retrieval followed by an eight-entry learned association, not unseen compositional generalization.',
 'A count-only file is not sufficient to independently audit geometric per-position hits or paired wins: rows omit predictions and exact occurrence references. Preserve reported counts as such, and add required outputs/refs to the next runnable boundary.',
 'Three loaded-artifact autoregressive traces of three tokens were generated and saved. One of three first answers is correct; continuations are edia-f, edia-f and ](.. . Generated continuations are therefore executed, not NOT_RUN; useful prose/text integration remains unqualified.',
 'The design makes target a function of payload only. Beating the categorical control is neither necessary for a useful learned emission mechanism nor a clean attribution test. Testing relations later is sensible, but first fix acknowledged remaining learning mismatches and keep this negative panel as a regression.'
]
o['cost_scope']={'per_attempt_elapsed_s':{i:r['elapsed_s']for i,r in rs.items()},'total_retained_attempt_elapsed_s':sum(r['elapsed_s']for r in rs.values()),'model_replay_in_this_audit':False,'inference_timing':'NOT_RUN','physical_energy':'UNAVAILABLE','ordinary_text_preservation':'NOT_RUN'}
o['conditional_accuracy_bounds']={}
for name, report_key in [('dev','dev_hits'),('tune','tune_hits'),('final','fresh_hits')]:
 xs=rows[name]; n=len(xs); c=sum(x['correct_exact_occurrence']for x in xs)
 bounds={}
 for arm in ['h4_read_conditioned','cyclic_c120_read_conditioned']:
  h=next(a[report_key]for a in r['comparisons']if a['arm']==arm)
  bounds[arm]={'correct_source_positions':c,'reported_total_hits':h,'minimum_possible_correct_source_hits':max(0,h-(n-c)),'maximum_possible_correct_source_hits':min(h,c),'scope':'Arithmetic bound from reported total hits and recounted source correctness flags; not actual per-source model evaluation.'}
 o['conditional_accuracy_bounds'][name]=bounds
o['script_sha256']=sha(P(__file__).read_bytes());out=P('/tmp/uor-pr1336-evidence-review.json');out.write_text(json.dumps(o,indent=2)+'\n');print(json.dumps({'output':str(out),'integrity':o['integrity'],'attempt_comparison':o['attempt_comparison'],'artifact_parameters':o['artifact_parameters'],'row_recomputation':o['row_recomputation'],'split_overlap':rec['split_overlap']},indent=2))
