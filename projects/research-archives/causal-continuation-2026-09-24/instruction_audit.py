from pathlib import Path
import subprocess,re,json,hashlib
r=Path('/Users/casey.allard/uor-r4-investigations/causal-20260924T000007');binary=r/'causal-lexical'
nm=subprocess.check_output(['nm',str(binary)],text=True)
symbols=[]
for line in nm.splitlines():
 p=line.split()
 if len(p)==3 and re.fullmatch('[0-9a-fA-F]+',p[0]):symbols.append((int(p[0],16),p[2]))
symbols.sort();chosen=[(i,a,n)for i,(a,n)in enumerate(symbols)if 'TlReadPlan10score_into' in n];assert len(chosen)==1
idx,start,name=chosen[0];end=next(a for a,n in symbols[idx+1:]if a>start)
p=subprocess.Popen(['otool','-tvV',str(binary)],stdout=subprocess.PIPE,text=True);lines=[]
for line in p.stdout:
 match=re.match(r'^([0-9a-fA-F]{16})\s+(.+)$',line)
 if match:
  address=int(match.group(1),16)
  if start<=address<end:lines.append(line)
  elif address>=end and lines:break
p.stdout.close();p.terminate();p.wait()
assert lines,'symbol instructions not found'
(r/'read-plan-assembly.txt').write_text(name+'\n'+''.join(lines))
forbidden={'mul','madd','msub','smull','umull','smaddl','umaddl','fadd','fsub','fmul','fdiv','fmla','fmls','scvtf','ucvtf','fcvt','fcmp','sdiv','udiv'}
found=[line.strip()for line in lines if line.split()[1]in forbidden]
record={'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'symbol':name,'start':hex(start),'end':hex(end),'instructions':len(lines),'forbidden_mnemonics':sorted(forbidden),'matches':found,'scope':'Inspection of the named AArch64 successful/error-path read-plan symbol in the retained release executable. Does not inspect every callee or the complete application, and does not establish whole-path energy or matrix-free mathematical structure.'}
(r/'instruction-audit.json').write_text(json.dumps(record,indent=2));print(json.dumps(record,indent=2))
