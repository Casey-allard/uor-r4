// FoundrySpecification.promptLibraryBoundary; IndexedDB completion owns acknowledgement.
const KEY='foundry:prompts:v1', MAX_BYTES=4*1024*1024;
const codes=['invalidPromptCommand','promptLibraryInvalid','promptMissing','promptSlugConflict','promptLibraryLimit','promptLibraryStorageFailed'];
export const promptFailure=code=>Object.assign(new Error(codes.includes(code)?code:'promptLibraryStorageFailed'),{code:codes.includes(code)?code:'promptLibraryStorageFailed'});
const fail=code=>{throw promptFailure(code);};
const copy=(value,code)=>{try{return structuredClone(value);}catch{fail(code);}};
const bytes=value=>new TextEncoder().encode(value).length;
const object=value=>value&&typeof value==='object'&&!Array.isArray(value);
const exact=(value,keys)=>object(value)&&Object.keys(value).sort().join()===keys.split(',').sort().join();
const id=value=>Number.isSafeInteger(value)&&value>0;
const date=value=>value instanceof Date&&Number.isFinite(value.getTime());
function fields(value,code){
 for(const [key,max] of [['title',256],['content',65536],['description',4096]])if(value[key]!==undefined&&(typeof value[key]!=='string'||bytes(value[key])>max||(key!=='description'&&!value[key].length)))fail(code);
 if(value.slug!==undefined&&value.slug!==null&&(typeof value.slug!=='string'||!/^[-a-zA-Z0-9]{1,128}$/.test(value.slug)))fail(code);
}
export function capturePromptCommand(kind,input){
 const value=copy(input,'invalidPromptCommand');
 if(kind==='list'){if(value!==undefined)fail('invalidPromptCommand');return value;}
 if(kind==='delete'){if(!id(value))fail('invalidPromptCommand');return value;}
 if(!['create','update'].includes(kind)||!object(value)||Object.keys(value).some(key=>!(kind==='create'?['title','description','content','slug']:['id','title','description','content','slug']).includes(key)))fail('invalidPromptCommand');
 if(kind==='create'&&(value.title===undefined||value.content===undefined)||kind==='update'&&!id(value.id))fail('invalidPromptCommand');
 fields(value,'invalidPromptCommand');return value;
}
export function readPromptLibrary(raw){
 if(raw===undefined)return {profile:'foundry/prompts/1',nextId:1,records:[]};
 const value=copy(raw,'promptLibraryInvalid');
 if(!exact(value,'profile,nextId,records')||value.profile!=='foundry/prompts/1'||!id(value.nextId)||!Array.isArray(value.records)||value.records.length>1000)fail('promptLibraryInvalid');
 const slugs=new Set();let previous=0;
 for(const row of value.records){
  if(!exact(row,'id,title,description,content,slug,createdAt,updatedAt')||!id(row.id)||row.id<=previous||row.id>=value.nextId||typeof row.title!=='string'||typeof row.content!=='string'||row.description!==null&&typeof row.description!=='string'||row.slug!==null&&typeof row.slug!=='string'||!date(row.createdAt)||!date(row.updatedAt)||row.updatedAt<row.createdAt)fail('promptLibraryInvalid');
  fields({...row,description:row.description??undefined},'promptLibraryInvalid');
  if(row.slug!==null){if(slugs.has(row.slug))fail('promptLibraryInvalid');slugs.add(row.slug);}previous=row.id;
 }
 if(bytes(JSON.stringify(value))>MAX_BYTES)fail('promptLibraryInvalid');return value;
}
export function operatePromptLibrary(raw,kind,input,now=new Date()){
 const value=capturePromptCommand(kind,input),journal=readPromptLibrary(raw);
 if(kind==='list')return {journal,result:journal.records};
 if(!date(now))fail('invalidPromptCommand');let result;
 if(kind==='create'){
  if(journal.records.length===1000||journal.nextId===Number.MAX_SAFE_INTEGER)fail('promptLibraryLimit');
  result={id:journal.nextId++,title:value.title,description:value.description??null,content:value.content,slug:value.slug??null,createdAt:new Date(now),updatedAt:new Date(now)};journal.records.push(result);
 }else{
  const index=journal.records.findIndex(row=>row.id===(kind==='delete'?value:value.id));if(index<0)fail('promptMissing');
  if(kind==='delete')journal.records.splice(index,1);
  else{const row=journal.records[index];for(const key of ['title','description','content','slug'])if(value[key]!==undefined)row[key]=value[key];row.updatedAt=new Date(Math.max(now.getTime(),row.updatedAt.getTime()));}
 }
 const slugs=journal.records.filter(row=>row.slug!==null).map(row=>row.slug);if(new Set(slugs).size!==slugs.length)fail('promptSlugConflict');
 if(bytes(JSON.stringify(journal))>MAX_BYTES)fail('promptLibraryLimit');
 return {journal:readPromptLibrary(journal),result:copy(result,'promptLibraryInvalid')};
}
export function createPromptLibrary(database){
 return async(kind,raw)=>{
  const input=capturePromptCommand(kind,raw);let db;try{db=await database;}catch{fail('promptLibraryStorageFailed');}
  return new Promise((resolve,reject)=>{
   let tx,result,problem;
   try{
    tx=db.transaction('settings',kind==='list'?'readonly':'readwrite');
    tx.oncomplete=()=>resolve(result);
    tx.onabort=()=>reject(problem||promptFailure('promptLibraryStorageFailed'));
    tx.onerror=()=>{problem??=promptFailure('promptLibraryStorageFailed');};
    const store=tx.objectStore('settings'),request=store.get(KEY);
    request.onsuccess=()=>{try{const answer=operatePromptLibrary(request.result,kind,input);result=answer.result;if(kind!=='list')store.put(answer.journal,KEY);}catch(error){problem=codes.includes(error?.code)?error:promptFailure('promptLibraryStorageFailed');tx.abort();}};
   }catch{if(tx)try{tx.abort();}catch{}reject(promptFailure('promptLibraryStorageFailed'));}
  });
 };
}
