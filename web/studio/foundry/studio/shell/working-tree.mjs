// FoundrySpecification.workingTreeBoundary: captured source, independently verified basis.
import {captureProjectSource} from './project-edit.mjs';
import {prepareSnapshot} from './snapshot-restore.mjs';
export const workingTreeFailure=code=>Object.assign(new Error(({invalidWorkingTreeRequest:'Invalid working-tree request',workingTreeStateInvalid:'Working source is invalid',workingTreeComparisonLimit:'Working-tree comparison exceeds its resource budget'})[code]||'Working-tree query refused'),{code});
const pathValid=p=>typeof p==='string'&&p.length>0&&p.length<=10000&&!/[\\\u0000-\u001f:]/.test(p)&&p.split('/').every(x=>x&&x!=='.'&&x!=='..');
function fileMap(files){if(!Array.isArray(files)||files.length>1000)throw workingTreeFailure('workingTreeStateInvalid');const map=new Map();let bytes=0;for(const entry of files){if(!Array.isArray(entry)||entry.length!==2||!pathValid(entry[0])||typeof entry[1]!=='string'||map.has(entry[0]))throw workingTreeFailure('workingTreeStateInvalid');bytes+=new TextEncoder().encode(entry[1]).length;if(bytes>48*1024*1024)throw workingTreeFailure('workingTreeComparisonLimit');map.set(...entry);}return map;}
export function lineChanges(before,after,budget={remaining:4000000}){
 const lines=s=>{const out=[];for(const match of s.matchAll(/[^\n]*\n|[^\n]+$/g)){if(out.length>=100000)throw workingTreeFailure('workingTreeComparisonLimit');out.push(match[0]);}return out;};const a=lines(before),b=lines(after);let start=0,endA=a.length,endB=b.length;
 while(start<endA&&start<endB&&a[start]===b[start])start++;
 while(endA>start&&endB>start&&a[endA-1]===b[endB-1]){endA--;endB--;}
 const n=endA-start,m=endB-start,cost=n*m;if(cost>budget.remaining)throw workingTreeFailure('workingTreeComparisonLimit');budget.remaining-=cost;
 let previous=new Uint32Array(m+1),current=new Uint32Array(m+1);
 for(let i=0;i<n;i++){for(let j=0;j<m;j++)current[j+1]=a[start+i]===b[start+j]?previous[j]+1:Math.max(previous[j+1],current[j]);[previous,current]=[current,previous];current.fill(0);}
 return {additions:m-previous[m],deletions:n-previous[m]};
}
export function compareWorkingFiles(current,basis){const before=fileMap(basis),after=fileMap(current),budget={remaining:4000000},changes=[];
 for(const path of [...new Set([...before.keys(),...after.keys()])].sort()){const oldContent=before.get(path)??'',newContent=after.get(path)??'';if(before.has(path)===after.has(path)&&oldContent===newContent)continue;changes.push({path,status:!before.has(path)?'added':!after.has(path)?'deleted':'modified',oldContent,newContent,...lineChanges(oldContent,newContent,budget)});}return changes;
}
export async function readWorkingTree(db,raw,boundary,{capture=captureProjectSource,verify=prepareSnapshot}={}){
 let input;try{input=structuredClone(raw);}catch{throw workingTreeFailure('invalidWorkingTreeRequest');}
 if(!input||Object.keys(input).some(k=>!['appId','filePath'].includes(k))||!Number.isSafeInteger(input.appId)||input.appId<1||(input.filePath!==undefined&&!pathValid(input.filePath)))throw workingTreeFailure('invalidWorkingTreeRequest');
 const captured=await capture(db,input.appId),snapshot=await verify(captured.reference,boundary);
 if(input.filePath!==undefined){const before=fileMap(snapshot.files),after=fileMap(captured.files);return {path:input.filePath,oldContent:before.get(input.filePath)??'',newContent:after.get(input.filePath)??''};}
 return compareWorkingFiles(captured.files,snapshot.files).map(({path,status,additions,deletions})=>({path,status,additions,deletions}));
}
