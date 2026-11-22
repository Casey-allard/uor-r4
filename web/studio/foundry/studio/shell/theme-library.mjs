// FoundrySpecification.themeLibraryBoundary; IndexedDB completion owns acknowledgement.
const KEY='foundry:themes:v1', MAX_BYTES=4*1024*1024;
const codes=['invalidThemeCommand','themeLibraryInvalid','themeMissing','themeLibraryLimit','themeStorageFailed','themeAppMissing','themeCatalogMismatch'];
export const themeFailure=code=>Object.assign(new Error(codes.includes(code)?code:'themeStorageFailed'),{code:codes.includes(code)?code:'themeStorageFailed'});
const fail=code=>{throw themeFailure(code);};
const copy=(value,code)=>{try{return structuredClone(value);}catch{fail(code);}};
const bytes=value=>new TextEncoder().encode(value).length;
const object=value=>value&&typeof value==='object'&&!Array.isArray(value);
const exact=(value,keys)=>object(value)&&Object.keys(value).sort().join()===keys.split(',').sort().join();
const id=value=>Number.isSafeInteger(value)&&value>0;
const date=value=>value instanceof Date&&Number.isFinite(value.getTime());
function fields(value,code){
 for(const [key,max] of [['name',256],['prompt',65536],['description',4096]])if(value[key]!==undefined&&(typeof value[key]!=='string'||bytes(value[key])>max||(key!=='description'&&!value[key].length)))fail(code);
}
export function captureThemeCommand(kind,input){
 const value=copy(input,'invalidThemeCommand');
 if(kind==='list'){if(value!==undefined)fail('invalidThemeCommand');return value;}
 if(kind==='delete'){if(!exact(value,'id')||!id(value.id))fail('invalidThemeCommand');return value;}
 if(!['create','update'].includes(kind)||!object(value)||Object.keys(value).some(key=>!(kind==='create'?['name','description','prompt']:['id','name','description','prompt']).includes(key)))fail('invalidThemeCommand');
 if(kind==='create'&&(value.name===undefined||value.prompt===undefined)||kind==='update'&&!id(value.id))fail('invalidThemeCommand');
 fields(value,'invalidThemeCommand');return value;
}
export function readThemeLibrary(raw){
 if(raw===undefined)return {profile:'foundry/themes/1',nextId:1,records:[]};
 const value=copy(raw,'themeLibraryInvalid');
 if(!exact(value,'profile,nextId,records')||value.profile!=='foundry/themes/1'||!id(value.nextId)||!Array.isArray(value.records)||value.records.length>1000)fail('themeLibraryInvalid');
 let previous=0;
 for(const row of value.records){
  if(!exact(row,'id,name,description,prompt,createdAt,updatedAt')||!id(row.id)||row.id<=previous||row.id>=value.nextId||typeof row.name!=='string'||typeof row.prompt!=='string'||row.description!==null&&typeof row.description!=='string'||!date(row.createdAt)||!date(row.updatedAt)||row.updatedAt<row.createdAt)fail('themeLibraryInvalid');
  fields({...row,description:row.description??undefined},'themeLibraryInvalid');
  previous=row.id;
 }
 if(bytes(JSON.stringify(value))>MAX_BYTES)fail('themeLibraryInvalid');return value;
}
export function operateThemeLibrary(raw,kind,input,now=new Date()){
 const value=captureThemeCommand(kind,input),journal=readThemeLibrary(raw);
 if(kind==='list')return {journal,result:journal.records};
 if(!date(now))fail('invalidThemeCommand');let result;
 if(kind==='create'){
  if(journal.records.length===1000||journal.nextId===Number.MAX_SAFE_INTEGER)fail('themeLibraryLimit');
  result={id:journal.nextId++,name:value.name,description:value.description??null,prompt:value.prompt,createdAt:new Date(now),updatedAt:new Date(now)};journal.records.push(result);
 }else{
  const index=journal.records.findIndex(row=>row.id===value.id);if(index<0)fail('themeMissing');
  if(kind==='delete')journal.records.splice(index,1);
  else{const row=journal.records[index];for(const key of ['name','description','prompt'])if(value[key]!==undefined)row[key]=value[key];row.updatedAt=new Date(Math.max(now.getTime(),row.updatedAt.getTime()));result=row;}
 }
 if(bytes(JSON.stringify(journal))>MAX_BYTES)fail('themeLibraryLimit');
 return {journal:readThemeLibrary(journal),result:copy(result,'themeLibraryInvalid')};
}
function selection(value,journal,catalog){
 if(value===null)return null;
 if(typeof value!=='string')fail('invalidThemeCommand');
 const builtin=catalog.find(row=>row.id===value);if(builtin)return builtin;
 if(!/^custom:[1-9][0-9]*$/.test(value))fail('invalidThemeCommand');
 const key=Number(value.slice(7));if(!id(key))fail('invalidThemeCommand');
 const row=journal.records.find(row=>row.id===key);if(!row)fail('themeMissing');return row;
}
export function readThemeCatalog(builtins){
 const catalog=copy(builtins,'themeCatalogMismatch');
 if(!Array.isArray(catalog)||!catalog.length||new Set(catalog.map(row=>row.id)).size!==catalog.length||catalog.some(row=>!exact(row,'id,name,description,icon,prompt')||Object.values(row).some(v=>typeof v!=='string')||!row.id||row.id.startsWith('custom:')))fail('themeCatalogMismatch');
 return catalog;
}
// FoundrySpecification.initialProjectThemeBoundary: called inside the creation transaction.
export function initialProjectTheme(settings,raw,catalog){
 const journal=readThemeLibrary(raw),themeId=settings?.selectedThemeId??'';
 if(themeId==='')return null;
 selection(themeId,journal,catalog);return themeId;
}
export function createThemeLibrary(database,builtins){
 const catalog=readThemeCatalog(builtins);
 return async(kind,raw)=>{
  if(kind==='catalog'){if(raw!==undefined)fail('invalidThemeCommand');return copy(catalog,'themeCatalogMismatch');}
  const appCommand=['select','selected','resolve'].includes(kind);
  const input=appCommand?copy(raw,'invalidThemeCommand'):captureThemeCommand(kind,raw);
  if(appCommand&&(!exact(input,kind==='select'?'appId,themeId':'appId')||!id(input.appId)||kind==='select'&&input.themeId!==null&&typeof input.themeId!=='string'))fail('invalidThemeCommand');
  if(kind==='select'&&input.themeId!==null&&!catalog.some(row=>row.id===input.themeId)&&(!/^custom:[1-9][0-9]*$/.test(input.themeId)||!id(Number(input.themeId.slice(7)))))fail('invalidThemeCommand');
  let db;try{db=await database;}catch{fail('themeStorageFailed');}
  return new Promise((resolve,reject)=>{
   let tx,result,problem;
   const refuse=error=>{problem=codes.includes(error?.code)?error:themeFailure('themeStorageFailed');try{tx.abort();}catch{reject(problem);}};
   try{
    const write=['create','update','delete','select'].includes(kind);
    tx=db.transaction(['settings','apps'],write?'readwrite':'readonly');
    tx.oncomplete=()=>resolve(result);tx.onabort=()=>reject(problem||themeFailure('themeStorageFailed'));tx.onerror=()=>{problem??=themeFailure('themeStorageFailed');};
    const settings=tx.objectStore('settings'),apps=tx.objectStore('apps'),request=settings.get(KEY);
    request.onsuccess=()=>{try{
     const journal=readThemeLibrary(request.result);
     if(appCommand){
      const lookup=apps.get(input.appId);lookup.onsuccess=()=>{try{
       const app=lookup.result;if(!app)fail('themeAppMissing');
       const themeId=kind==='select'?input.themeId:app.themeId??null;
       const theme=selection(themeId,journal,catalog);
       if(kind==='select')apps.put({...app,themeId},input.appId);
       else result=kind==='resolve'?theme?.prompt??'':themeId;
      }catch(error){refuse(error);}};
     }else{
      const answer=operateThemeLibrary(journal,kind,input);result=answer.result;
      if(write)settings.put(answer.journal,KEY);
      if(kind==='delete'){
       const home=settings.get('user');home.onsuccess=()=>{try{const value=home.result;if(value?.selectedThemeId==='custom:'+input.id)settings.put({...value,selectedThemeId:''},'user');}catch(error){refuse(error);}};
       const cursor=apps.openCursor();cursor.onsuccess=()=>{try{const entry=cursor.result;if(!entry)return;if(entry.value.themeId==='custom:'+input.id)entry.update({...entry.value,themeId:null});entry.continue();}catch(error){refuse(error);}};
      }
     }
    }catch(error){refuse(error);}};
   }catch(error){if(tx)refuse(error);else reject(themeFailure('themeStorageFailed'));}
  });
 };
}
