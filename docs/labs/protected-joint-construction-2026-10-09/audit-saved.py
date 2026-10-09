#!/usr/bin/env python3
"""Inspect saved gradients/receipts only. No model, backward, proposal scoring or data draw."""
from pathlib import Path
import hashlib,json,math,struct,sys,time
import numpy as np
R=Path(sys.argv[1]); OUT=Path(sys.argv[2]);start=time.monotonic()
def read(n):return json.loads((R/n).read_text())
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def need(v,m):
 if not v:raise ValueError(m)
def floats(n):
 b=(R/n).read_bytes();return list(struct.unpack('<'+'f'*(len(b)//4),b))
def vector(v):
 need(len(v)==1920 and all(math.isfinite(x) for x in v),'finite joint1920 vector')
 return v
def dot(a,b):
 need(len(a)==len(b),'dot dimensions differ')
 return sum(x*y for x,y in zip(a,b))
report=read('report.json');g=read('coupled-gradient-receipt.json');m=read('protected-margin-receipt.json');j=read('coupled-construction.json')
need(report['status']=='COMPLETED' and report['new_backward_calls']==411 and report['new_training_graph_forwards']==822,'completed scope')
need(len(m['terms'])==380 and len(g['perterm'])==31,'gradient population')
need(h(R/'protected-margin-receipt.json')==g['protected_margin_receipt_sha256']==j['protected_margin_receipt_sha256'],'margin hash chain')
raw=[];zeros=0
for i,t in enumerate(m['terms']):
 need(t['guard_index']==i and t['shape']==[1920] and t['status']=='PRESENT','guard identity')
 need(h(R/t['file'])==t['sha256'],'Jacobian hash')
 row=floats(t['file']);need(len(row)==1920 and all(math.isfinite(x) for x in row),'finite Jacobian')
 raw.append(row);zeros+=int(not any(row))
 b=(R/t['original_masses_file']).read_bytes();need(h(R/t['original_masses_file'])==t['original_masses_sha256'],'native mass identity')
 mass=struct.unpack('<4096Q',b);winner=max(range(4096),key=lambda k:(mass[k],-k));rival=max((k for k in range(4096) if k!=winner),key=lambda k:(mass[k],-k))
 need((winner,rival,mass[winner],mass[rival])==(t['winner'],t['rival'],t['winner_mass'],t['rival_mass']),'mechanical native pair')
 need(h(R/t['utility_file'])==t['utility_sha256'],'margin utility hash')
 u=read(t['utility_file']);need(u['row']==31+i and u['unit_margin_weight']==1,'unit margin donor namespace')
totals={name:[0.0]*960 for name in ['prefix.coefficients','generate.unary']}
for term in g['perterm']:
 for t in term['families']:
  need(h(R/t['file'])==t['sha256'] and t['status']=='PRESENT' and t['shape']==[960],'objective raw hash/shape/status')
  values=floats(t['file']);need(len(values)==960 and all(math.isfinite(x) for x in values),'objective finite shape')
  totals[t['family']]=[struct.unpack('<f',struct.pack('<f',x+y))[0] for x,y in zip(totals[t['family']],values)]
files={(x['family'],x['kind']):x for x in g['files']}
pg=[];master=[]
for family in ['prefix.coefficients','generate.unary']:
 for kind in ['gradient','initial-master']:need(h(R/files[(family,kind)]['file'])==files[(family,kind)]['sha256'],'aggregate/master hash')
 need(struct.pack('<960f',*totals[family])==(R/files[(family,'gradient')]['file']).read_bytes(),'ordered f32 gradient sum')
 pg+=floats(files[(family,'gradient')]['file']);master+=floats(files[(family,'initial-master')]['file'])
vector(pg);vector(master)
unit=[]
for row in raw:
 n=math.sqrt(dot(row,row));unit.append([x/n for x in row] if n else [float(x) for x in row])
d=vector(j['projection']['direction']);norm=math.sqrt(dot(d,d));tol=1e-10*norm;res=[dot(row,d) for row in unit]
need(tol==j['projection']['tolerance'] and res==j['projection']['final_residuals'],'saved final residual replay')
need(all(x>=-tol for x in res)==j['projection']['linear_constraints_passed'],'final residual decision')
# Independent array arithmetic replay is approximate. Its direction closeness
# does not establish identical feasibility near zero and never overrides Rust.
a=np.asarray(unit,dtype=np.float64);d2=-np.asarray(pg,dtype=np.float64);counts=np.zeros(380,dtype=np.int64)
for _ in range(256):
 for i,row in enumerate(a):
  v=float(row@d2)
  if v<0:d2-=v*row;counts[i]+=1
err=float(np.max(np.abs(d2-np.asarray(d))))
ind_tol=1e-10*math.sqrt(dot(d2,d2));ind_res=[dot(row,d2) for row in unit]
ind_pass=all(x>=-ind_tol for x in ind_res)
need(np.allclose(d2,d,rtol=1e-9,atol=1e-12),'independent projected direction differs')
trials=[]
for t in j['joint_vectors']:
 delta=vector(t['actual_delta']);qres=[dot(row,delta) for row in unit];qtol=1e-10*math.sqrt(dot(delta,delta));linear=dot(pg,delta)
 need(qres==t['quantized_margin_residuals'] and qtol==t['quantized_tolerance'] and linear==t['actual_CE_linear_delta'],'actual displacement replay')
 need(all(x>=-qtol for x in qres)==t['quantized_constraints_passed'],'quantized decision')
 trials.append({'radius':t['radius'],'eligible':t['eligible'],'CE_linear_delta':linear,'violated_guards':sum(x< -qtol for x in qres),'minimum_residual':min(qres),'tolerance':qtol,'native':t['native']})
if j['selected']['status']=='unchanged':
 for family,path in [('prefix.coefficients','checkpoint-0001/prefix/prefix-source-f32.bin'),('generate.unary','checkpoint-0001/generate-source/generate.unary.f32le')]:
  need((R/path).read_bytes()==(R/files[(family,'initial-master')]['file']).read_bytes(),'unchanged fractional master bytes')
r={'status':'PASS_SAVED_ARITHMETIC','scope':'raw gradient identities and ordered objective f32 sums; mechanical native margins; approximate independent256-pass direction replay; exact hash-bound saved-displacement residual arithmetic, not full proposal or native-scoring authentication; no model rerun','report_sha256':h(R/'report.json'),'manifest_sha256':h(R/'manifest.json'),'source_commit':report['source_commit'],'protected_rows':380,'zero_Jacobians':zeros,'objective_raw_files':62,'independent_direction_max_abs_error':err,'independent_projection_passed':ind_pass,'independent_projection_tolerance':ind_tol,'independent_projection_minimum_residual':min(ind_res),'projection_norm':norm,'projection_tolerance':tol,'projection_minimum_residual':min(res),'projection_violated_guards':sum(x< -tol for x in res),'projection_passed':j['projection']['linear_constraints_passed'],'original_gradient_dot_projected_direction':dot(pg,d),'trials':trials,'selected':j['selected'],'final_gate':report['final_gate'],'baseline_objective':report['baseline_objective'],'candidate_objective':report['candidate_objective'],'seconds':time.monotonic()-start}
with OUT.open('x') as f:json.dump(r,f,indent=2);f.write('\n')
print(json.dumps({k:v for k,v in r.items() if k not in ['trials','baseline_objective','candidate_objective']}))
