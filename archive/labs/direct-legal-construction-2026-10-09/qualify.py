#!/usr/bin/env python3
"""Immediate positive-only saved-artifact own-feedback check; Rust owns model and oracle."""
from pathlib import Path
import sys
import copy,hashlib,json,os,resource,subprocess,time
W=Path(__file__).resolve().parents[3];R=W/'local/direct-legal'
C=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else R/'config.json';B=W/'local/build/release/examples/geometric-frozen-map-fit'
V=W/'local/restore/codex-protected-discrete-feedback-20261009/runtime/native-reached-prefix-attribution'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(p.read_text())
def write(p,v):
 p.parent.mkdir(parents=True,exist_ok=True)
 with p.open('x') as f:json.dump(v,f,indent=2);f.write('\n')
c=read(C);root=Path(c['out']);subprocess.run([str(V),'verify-report',str(root)],check=True)
r=read(root/'report.json');assert r['status']=='COMPLETED'
assert r['new_backward_calls']==0 and r['new_training_graph_forwards']==0 and r['all_original380_preserved']
assert r['finite_episode_positive']==r['final_gate']['passed']
solver=read(root/'coupled-construction.json')['projection']['legal_solver']
not_run='NOT_RUN_BACKEND_EXECUTION_FAILURE' if solver['status']=='BACKEND_NUMERIC_OR_NO_INCUMBENT' else 'NOT_RUN_CONSTRUCTION_NEGATIVE'
decision=dict(candidate_report_sha256=sha(root/'report.json'),candidate_manifest_sha256=sha(root/'manifest.json'),binary_sha256=sha(B),config_sha256=sha(C),actual9=not_run,full512='NOT_RUN',fresh='NOT_RUN',multi_turn='NOT_RUN')
if r['finite_episode_positive']:
 assert all(r['final_gate'][k] for k in ['strict_combined_descent','strict_episode_descent','all15_conditional_winners','all17_references','all380_original_winners','passed'])
 cp=root/'checkpoint-0001';actual=copy.deepcopy(c);learning=actual.pop('coupled_episode_learning')
 actual['checkpoint']=str(cp);actual['out']=str(R/'qualifications'/root.name);actual['maximum_report_bytes']=256<<20
 actual['prefix_artifact_check']=dict(retained_candidate_root=str(root),retained_intermediate_root=learning['original_inputs']['retained_intermediate_root'],coupled_episode_candidate=dict(donor_credit='full_pool_utility',prefix_transaction='protected_legal_set',expected_report_sha256=sha(root/'report.json'),expected_manifest_sha256=sha(root/'manifest.json'),expected_generate_sha256=sha(cp/'generate.bin'),expected_unary_master_sha256=sha(cp/'generate-source/generate.unary.f32le'),expected_prefix_packed_sha256=sha(cp/'prefix/prefix-q4.bin'),expected_prefix_master_sha256=sha(cp/'prefix/prefix-source-f32.bin'),expected_continuation_sha256=sha(cp/'continuation-field.bin')))
 config=R/'qualifications'/f'{root.name}-config.json';write(config,actual)
 env=dict(os.environ);env.update(RAYON_NUM_THREADS='2',OMP_NUM_THREADS='2',OPENBLAS_NUM_THREADS='2',UOR_REQUIRE_CUDA='0')
 started=time.monotonic()
 with (R/'qualifications'/f'{root.name}.log').open('x') as log:
  process=subprocess.run([str(B),str(config)],env=env,stdout=log,stderr=subprocess.STDOUT)
 decision.update(actual9='EXECUTED' if process.returncode==0 else 'EXECUTION_FAILED',exit_code=process.returncode,seconds=time.monotonic()-started,config_sha256=sha(config),output=actual['out'],max_rss_bytes=resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss)
write(R/f'qualification-decision-{root.name}.json',decision)
print(json.dumps(decision))
if decision.get('exit_code',0):raise SystemExit(decision['exit_code'])
