#!/usr/bin/env python3
"""Read-only integrity, binary parameter and saved-summary audit. No model execution."""
import collections, hashlib, json, pathlib, struct, subprocess
import blake3
P=pathlib.Path
BASE=P('/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20')
REPO=P('/Users/casey.allard/.codex/worktrees/takeover-research-reconciliation/uor-r4')
HEAD='51b275bf'
sha=lambda b:hashlib.sha256(b).hexdigest()
load=lambda p:json.loads(p.read_text())
o={'schema':'uor-r4.contextual-emission-principal-evidence-audit/1','scope':'Read-only sealed report integrity, binary parameter inspection, saved aggregate/sample crosschecks and source-derived fixture reconstruction. No builds, model inference, training or sealed-root mutation. Full per-position hit/loss/source-correctness aggregates are not independently numerically reproducible from the retained sufficient statistics because none were saved.','review_revision':subprocess.check_output(['git','rev-parse',HEAD],cwd=REPO,text=True).strip(),'integrity':{},'attempt_comparison':{}}
rs={i:load(BASE/f'contextual-emission-{i}/result.json')for i in range(1,5)}
exe=P('/Users/casey.allard/uor-r4/target/release/competitive-reader'); exesha=sha(exe.read_bytes())if exe.exists()else None
for i,r in rs.items():
 root=BASE/f'contextual-emission-{i}';m=load(root/'manifest.json');actual={str(p.relative_to(root))for p in root.rglob('*')if p.is_file()}-{'manifest.json'};listed={x['path']for x in m['files']}
 o['integrity'][i]={'root':str(root),'listed_files':len(listed),'missing':sorted(listed-actual),'unlisted':sorted(actual-listed),'seal_errors':[x['path']for x in m['files']if len((root/x['path']).read_bytes())!=x['bytes']or blake3.blake3((root/x['path']).read_bytes()).hexdigest()!=x['blake3']],'result_sha256':sha((root/'result.json').read_bytes()),'artifact_hashes_match':{a['arm']:{'params':sha((root/f"artifacts/{a['arm']}.rlrc").read_bytes())==a['params_sha256'],'residual':sha((root/f"artifacts/{a['arm']}.rlce").read_bytes())==a['residual_sha256']}for a in r['artifacts']},'submitted_source_matches':{x['path']:sha(subprocess.check_output(['git','show',HEAD+':'+x['path']],cwd=REPO))==x['sha256']for x in r['running_source']['source_files']},'declared_executable_sha256':r['running_source']['executable_sha256'],'retained_current_executable_sha256':exesha,'current_executable_matches':exesha==r['running_source']['executable_sha256']}
 o['attempt_comparison'][i]={'same_as_attempt4':{k:r.get(k)==rs[4].get(k)for k in ['instrument','validity','learning','comparisons','changed_source_pairs_fresh','generated','artifacts']},'elapsed_s':r['elapsed_s'],'claimed_at':load(root/'attempt.json')['claimed_at'],'sealed_at':m['sealed_at'],'fresh_h4_hits':next(a['fresh_hits']for a in r['comparisons']if a['arm']=='h4_read_conditioned')}
r=rs[4]
inputs=[('E',BASE/'head-projection-3/corrected/empirical.cpl2',r['inputs']['E_sha256']),('S',BASE/'s-attribution-3/corrected/separable_older_query_read.cpx3',r['inputs']['S_sha256']),('raw_tokenizer',BASE.parent/'sources/smollm2-135m-instruct/tokenizer.json','9ca9acddb6525a194ec8ac7a87f24fbba7232a9a15ffa1af0c1224fcd888e47c'),('reader_parent',BASE/'relational-learning-4/artifacts/relational_ctx.rlr2','d4ab183d3975be563139ba42f3a5fb9ec61fa1691603bfcc3d603787d7ab5b55')]
o['inputs']={label:{'path':str(p),'sha256':sha(p.read_bytes()),'matches_source_or_report_declared':sha(p.read_bytes())==s}for label,p,s in inputs}
o['inputs']['reader_binding_scope']='Reader-parent hash is source constant and audited here, but the contextual-emission runner does not check or record that constant at load; it checks S/tokenizer linkage inside the reader artifact. Derived-tokenizer and group-table hashes are retained reports, not re-derived in this no-model audit.'
def parse_rc(path):
 b=path.read_bytes();c=4;ver,nt=struct.unpack_from('<II',b,c);c+=8;t=list(b[c:c+nt]);c+=nt;nd,=struct.unpack_from('<I',b,c);c+=4;domain=list(struct.unpack_from('<'+'I'*nd,b,c));c+=4*nd;nc,=struct.unpack_from('<I',b,c);c+=4;v=list(b[c:c+nc]);c+=nc
 return {'sha256':sha(b),'bytes':len(b),'consumed':c,'version':ver,'transport_changes_from_index_initialization':[{'relation':i,'initial':i,'final':x}for i,x in enumerate(t)if x!=i],'value_domain':domain,'value_codes':v,'value_changes_from_index_initialization':[{'value':domain[i],'initial':i,'final':x}for i,x in enumerate(v)if x!=i]}
def parse_res(path):
 b=path.read_bytes();ver,width,shift,nr=struct.unpack_from('<IIII',b,4);c=20;emb=list(struct.unpack_from('<'+'b'*(nr*width),b,c));c+=nr*width;nw,=struct.unpack_from('<I',b,c);c+=4;w=list(struct.unpack_from('<'+'b'*(nw*width),b,c));c+=nw*width;rows=[i for i in range(nw)if any(w[i*width:(i+1)*width])]
 return {'sha256':sha(b),'bytes':len(b),'consumed':c,'version':ver,'width':width,'shift':shift,'r_rows':nr,'output_rows_stored':nw,'active_output_rows':rows,'nonzero_coefficients':sum(x!=0 for x in w),'embedding_sha256':sha(bytes((x%256 for x in emb))),'embedding_zero_columns':[j for j in range(width)if not any(emb[k*width+j]for k in range(nr))],'max_abs_w':max(map(abs,w)),'all_coefficients_ternary':all(-1<=x<=1 for x in w+emb),'initial_output_was_zero':True}
o['artifact_parameters']={arm:{'params':parse_rc(BASE/f'contextual-emission-4/artifacts/{arm}.rlrc'),'residual':parse_res(BASE/f'contextual-emission-4/artifacts/{arm}.rlce')}for arm in ['h4_emission','cyclic_c120_emission']}
o['artifact_parameters']['scope']='Parameters differ from index-map/zero-output initializations. Same state count/capacity and byte-identical embeddings: seeded() ORs the nominal seeds with1, collapsing their apparent difference. RLRC does not serialize algebra; RLCE does not bind reader, tokenizer, base, seed, data or algebra. Artifacts are not numerically evaluated by this script.'
# Reconstruct only authored data, using a previously saved bank list for the same derived tokenizer.
# This is data/instrument analysis, not model execution.
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
   bs=blocks.copy();bs.insert(pos,(srole,key,vals[vi]));prefix=[t for b in bs for t in b]+[qrole,key];items.append({'pair':pid,'member':member,'prefix':prefix,'target':outs[vi],'source_abs':pos*3+2,'relevant_value':vals[vi],'source_role':srole,'query_role':qrole,'key':key})
 return items
pops={name:specs(n,seed)for name,n,seed in [('dev',90,0xC0F00001),('tune',60,0xC0F00002),('fresh',60,0xC0F00003)]}
rec={'scope':'Source-derived fixture reconstruction, not saved original per-position model evidence. Same-tokenizer role/key/value banks recovered from separately sealed competitive-reader-1/population.json. No new BPE, source selection, logits or predictions executed.','bank_path':str(bankpath),'bank_sha256':sha(bankpath.read_bytes()),'panels':{},'split_overlap':{}}
for name,items in pops.items():
 checks=[]
 for a,b in zip(items[::2],items[1::2]):
  diff=[i for i,(x,y)in enumerate(zip(a['prefix'],b['prefix']))if x!=y];checks.append(diff==[a['source_abs']]and a['target']!=b['target']and a['prefix'][-5:]==b['prefix'][-5:])
 counts=collections.Counter(x['target']for x in items)
 rec['panels'][name]={'positions':len(items),'pairs':len(items)//2,'all_pairs_change_only_relevant_payload_and_keep_last5':all(checks),'all_targets_absent_from_prefix':all(x['target']not in x['prefix']for x in items),'target_counts':dict(sorted(counts.items())),'majority_answer_hits':max(counts.values()),'majority_answer_pair_both':0,'uniform_eight_class_expected_hits':len(items)/8,'uniform_independent_expected_pair_both':len(items)/128,'exact_prefix_duplicates':len(items)-len({tuple(x['prefix'])for x in items}),'canonical_items_sha256':sha(json.dumps(items,sort_keys=True,separators=(',',':')).encode())}
for a,b in [('dev','tune'),('dev','fresh'),('tune','fresh')]:
 A,B=pops[a],pops[b];fns={'complete_prefix':lambda x:tuple(x['prefix']),'query_role_key':lambda x:(x['query_role'],x['key']),'query_role_relevant_value':lambda x:(x['query_role'],x['relevant_value']),'role_key_value':lambda x:(x['query_role'],x['key'],x['relevant_value']),'relevant_value_target':lambda x:(x['relevant_value'],x['target'])}
 rec['split_overlap'][a+'_vs_'+b]={key:{'left_unique':len({fn(x)for x in A}),'right_unique':len({fn(x)for x in B}),'intersection':len({fn(x)for x in A}&{fn(x)for x in B})}for key,fn in fns.items()}
o['fixture_reconstruction']=rec
for arm in ['h4_emission','cyclic_c120_emission']:
 p=o['artifact_parameters'][arm]['params'];code=dict(zip(p['value_domain'],p['value_codes']));groups=collections.defaultdict(list)
 for v,c in code.items():groups[c].append(v)
 p['alias_groups']={k:v for k,v in groups.items()if len(v)>1}
 p['same_code_changed_source_pairs_assuming_correct_payload_selection']={name:sum(code[a['relevant_value']]==code[b['relevant_value']]for a,b in zip(items[::2],items[1::2]))for name,items in pops.items()}
 p['alias_scope']='For any pair with both correct source payloads selected and unchanged query/relation, equal value codes imply equal q1 and identical logits, so both distinct targets cannot be emitted. A wider readout alone cannot recover those distinctions. Counts here are reconstructed task pairs, not measured selected-source rows.'
sample=[]
for row in r['changed_source_pairs_fresh']:
 a,b=pops['fresh'][2*row['pair']:2*row['pair']+2];sample.append({'pair':row['pair'],'reconstructed_values_answers_match_saved':(a['relevant_value'],a['target'],b['relevant_value'],b['target'])==(row['value_a'],row['answer_a'],row['value_b'],row['answer_b']),'only_relevant_payload_changes':sum(x!=y for x,y in zip(a['prefix'],b['prefix']))==1,'correct_a_recomputed':row['emitted_a']==row['answer_a'],'correct_b_recomputed':row['emitted_b']==row['answer_b'],'both_correct_recomputed':row['emitted_a']==row['answer_a']and row['emitted_b']==row['answer_b'],'emission_changes':row['emitted_a']!=row['emitted_b'],'reported':row})
o['retained_samples']={'paired_sample_rows':sample,'paired_first_four_correct_count':sum(x['correct_a_recomputed']+x['correct_b_recomputed']for x in sample),'paired_first_four_both_correct':sum(x['both_correct_recomputed']for x in sample),'generation': [{'sample':i,'first_correct':row['emitted'][0]==row['answer'],'emissions_match_steps':row['emitted']==[s['emitted']for s in row['steps']],'answer_matches_reconstructed_fixture':row['answer']==pops['fresh'][i]['target'],'reported':row}for i,row in enumerate(r['generated'])]}
o['reported_counts_not_independently_recomputed']=r['comparisons']
o['reported_learning_not_calibrated_CE']=r['learning']
o['source_corrections']=[
 {'finding':'Learned value codes erase required output distinctions','evidence':'H4 value codes [22,22,11,22,4,89,89,27] collapse 8 values into 5 codes; C120 [23,1,23,98,45,41,2,2] collapses into 6.','consequence':'Conditional on correct reads and fixed query/relation, aliased values give identical q1/local/residual and cannot both produce correct different answers. An arbitrary wider head cannot repair these input aliases.'},
 {'finding':'No correct-source measurement','evidence':'validity.source_selected counts CePos.read, which is true whenever choose_scored returns any action; CeItem.seq.answer_source_abs is None and selected source references are not retained.','consequence':'180/180 reads does not clear selection as a bottleneck. Since targets are a one-to-one function of relevant value, ambiguous selected-payload signatures imply at least one wrong selected payload in each ambiguous signature; they are not clean evidence of insufficient emission capacity.'},
 {'finding':'Raw fixed-point logits treated as nats','evidence':'train_output_map and served_nll_bits exponentiate z_local directly; normal runner bits multiplies logits by 2^-f_bits first.','consequence':'Reported 15561 -> 7987 values measure unit-temperature-on-integer-logits objectives, not calibrated model CE. They cannot be divided by a constant afterward without full logits.'},
 {'finding':'Fresh population exposed before repairs','evidence':'All attempts use seeds C0F00001/2/3; attempt1 already saves fresh hits before optimizer and output-row-budget repairs in2. Attempts2-4 metrics and artifacts repeat exactly.','consequence':'One reused development/regression draw; no untouched post-selection acceptance panel. Same output classes/values appear in all splits, and overlapping relation/content combinations are not explicitly held out.'},
 {'finding':'Missing row evidence','evidence':'Seals contain only attempt.json, result.json and four artifacts. No complete prefixes, selected occurrence/payload, per-position logits/margins, full pair predictions or oracle margins.','consequence':'Saved8pair-member samples and3generated trajectories can be checked, but120fresh hits,60pair hits, selection correctness and180row ceiling cannot independently be rederived without new model evaluation.'},
 {'finding':'No independently loaded behavioral replay','evidence':'eval_arm and generation run before export/reload; from_bytes equality is checked only after prediction.','consequence':'Parameter serialization round-trip is established, not actual independently loaded predictor parity.'},
 {'finding':'Convergence/capacity diagnosis unestablished','evidence':'One-coordinate ternary flip code can revert an accepted first candidate to its original saved coefficient on a later rejected candidate, while leaving cached best score stale. W refinement also precedes map changes, with no W refit under final features. Repeated extra passes cannot distinguish local search behavior from global attainable capacity.','consequence':'Do not widen only because 2 -> 5 passes left metrics unchanged. Correct/refit the learner and report source-conditional ceilings first.'},
 {'finding':'Upper range bound is not reachability','evidence':'32768 is max absolute residual bound; target/winner movement is state-,row- andsign-dependent.','consequence':'Bound exceeding 16640 does not prove adequacy of attainable margins.'},
 {'finding':'Matched embeddings confirmed; geometric advantage unestablished','evidence':'Nominal H4/C120 seeds 0x51E50000/1 both become 0x51E50001 after OR 1. Artifact embedding hashes equal. Algebra differs while capacity/common features match.','consequence':'13 versus 12 positions and 1 versus 1 pairs on a repeated population do not establish H4 advantage; preserve both operators.'},
 {'finding':'Limited positive scope','evidence':'Actual parameters differ from initialization; first 4 saved changed-source pairs change emissions, 2/8 members correct; all 3 saved generation traces step-match and 1/3 first token is correct. Aggregates report 13/120 and 1/60 H4.','consequence':'Learned residual can influence uncopied emission under source changes, but reliability, geometric advantage, general prose, natural-text preservation and whole-path efficiency remain unqualified.'}
]
o['cost_scope']={'retained_model_attempt_elapsed_s':sum(r['elapsed_s']for r in rs.values()),'per_attempt_elapsed_s':{i:r['elapsed_s']for i,r in rs.items()},'inference_timing':'NOT_RUN','energy':'UNAVAILABLE','ordinary_text_preservation':'NOT_RUN for these operators','exact_disabled_logits_parity':'Source follows identical local path, but saved reports contain only zero-hit counts, not measured logit equality.'}
o['script_sha256']=sha(P(__file__).read_bytes());out=P('/tmp/uor-pr1335-evidence-review.json');out.write_text(json.dumps(o,indent=2)+'\n');print(json.dumps({'out':str(out),'integrity':o['integrity'],'fixture_panels':rec['panels'],'split_overlap':rec['split_overlap'],'artifact_parameters':o['artifact_parameters'],'samples':{k:v for k,v in o['retained_samples'].items()if k.startswith('paired_first')}},indent=2))
