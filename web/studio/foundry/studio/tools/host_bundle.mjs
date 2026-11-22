import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const generator=fileURLToPath(import.meta.url),studio=path.resolve(path.dirname(generator),'..');
const messages={invalidHostBundleInput:'The host bundle inputs are invalid',hostBundleMismatch:'The host bundle or receipt does not match its inputs',hostBundleCompilerMismatch:'The host bundle compiler does not match the renderer lock'};
const failure=code=>Object.assign(new Error(messages[code]),{code});
const digest=bytes=>createHash('sha256').update(bytes).digest('hex');
export function hostBundleInputs(shellRoot){
 shellRoot=path.resolve(shellRoot);
  const captured=new Map();let sourceBytes=0;
  const relative=file=>{try{const rel=path.relative(shellRoot,path.resolve(file)).split(path.sep).join('/');if(!rel||rel==='..'||rel.startsWith('../')||path.isAbsolute(rel)||fs.realpathSync(file)!==path.resolve(file)||!fs.statSync(file).isFile())throw failure('invalidHostBundleInput');return rel;}catch{throw failure('invalidHostBundleInput');}};
  const capture=file=>{
   const rel=relative(file);if(captured.has(rel))return captured.get(rel).bytes;
   const bytes=fs.readFileSync(file);sourceBytes+=bytes.length;
   if(captured.size>=256||sourceBytes>16*1024*1024)throw failure('invalidHostBundleInput');
   captured.set(rel,{bytes,sha256:digest(bytes)});return bytes;
  };
  return {captured,relative,capture};
}
export function checkHostBundleBytes(shellRoot,output,receiptBytes){
 try{if(!fs.readFileSync(path.join(shellRoot,'host.bundle.js')).equals(output)||fs.readFileSync(path.join(shellRoot,'host-bundle.json'),'utf8')!==receiptBytes)throw 0;}catch{throw failure('hostBundleMismatch');}
}
export function checkHostBundleCompiler(locked,version){
 try{if(JSON.parse(locked).packages?.['node_modules/esbuild']?.version!==version)throw 0;}catch{throw failure('hostBundleCompilerMismatch');}
}
export async function projectHostBundle({shellRoot=path.join(studio,'shell'),lockPath=path.join(studio,'vendor/prism/package-lock.json'),modelPath=path.join(studio,'../app/src/FoundrySpecification.lex.tex'),check=false}={}){
 try{
  shellRoot=path.resolve(shellRoot);
  const locked=fs.readFileSync(lockPath),model=fs.readFileSync(modelPath),generatorBytes=fs.readFileSync(generator);
  let esbuild;try{esbuild=await import('../vendor/prism/node_modules/esbuild/lib/main.js');}catch{throw failure('hostBundleCompilerMismatch');}
  checkHostBundleCompiler(locked,esbuild.version);
  const {captured,relative,capture}=hostBundleInputs(shellRoot),dynamic=new Set();
  const result=await esbuild.build({absWorkingDir:shellRoot,entryPoints:['host.js'],outfile:'host.bundle.js',bundle:true,format:'esm',platform:'browser',target:'es2022',minify:true,write:false,metafile:true,logLevel:'silent',plugins:[{name:'preserve-source-module-urls',setup(build){
   build.onResolve({filter:/.*/},args=>{
    if(args.kind!=='dynamic-import')return;
    if(!args.path.startsWith('.'))throw failure('invalidHostBundleInput');
    const target=path.resolve(path.dirname(args.importer),args.path),rel=relative(target);capture(target);dynamic.add(rel);
    return {path:'./'+rel,external:true};
   });
   build.onLoad({filter:/\.(mjs|js)$/},async args=>{
    let source=capture(args.path).toString('utf8');
    // These functions are canonical source data as well as executable code.
    if(relative(args.path)==='prism.js'){
     const marker='Object.freeze([prismAnswer.toString(), prismDecide.toString()])';
     if(source.split(marker).length!==2)throw failure('invalidHostBundleInput');
     const parts=(await import(pathToFileURL(args.path).href+'?host-bundle='+digest(Buffer.from(source)))).INTERPRETER_PARTS;
     if(!Array.isArray(parts)||parts.length!==2||parts.some((part,i)=>typeof part!=='string'||!part.startsWith('function '+['prismAnswer','prismDecide'][i]+'(')||!source.includes(part)))throw failure('invalidHostBundleInput');
     source=source.replace(marker,'Object.freeze('+JSON.stringify(parts)+')');
    }
    if(!source.includes('import.meta.url'))return {contents:source,loader:'js',resolveDir:path.dirname(args.path)};
    if(source.includes('__foundrySourceURL'))throw failure('invalidHostBundleInput');
    const transformed=await esbuild.transform(source,{loader:'js',format:'esm',target:'es2022',define:{'import.meta.url':'__foundrySourceURL'},banner:`const __foundrySourceURL = new URL(${JSON.stringify('./'+relative(args.path))}, import.meta.url).href;`});
    return {contents:transformed.code,loader:'js',resolveDir:path.dirname(args.path)};
   });
  }}]});
  if(result.outputFiles.length!==1||result.outputFiles[0].contents.length>8*1024*1024)throw failure('invalidHostBundleInput');
  for(const input of Object.keys(result.metafile.inputs))if(!captured.has(relative(path.resolve(shellRoot,input))))throw failure('invalidHostBundleInput');
  for(const [rel,input] of captured)if(digest(fs.readFileSync(path.join(shellRoot,rel)))!==input.sha256)throw failure('invalidHostBundleInput');
  const output=Buffer.from(result.outputFiles[0].contents);
  const receipt={schema:'foundry/host-bundle/1',authority:{path:'app/src/FoundrySpecification.lex.tex',sha256:digest(model)},generator:{path:'studio/tools/host_bundle.mjs',sha256:digest(generatorBytes)},compiler:{name:'esbuild',version:esbuild.version,lockPath:'studio/vendor/prism/package-lock.json',lockSha256:digest(locked)},inputs:[...captured].sort(([a],[b])=>a<b?-1:a>b?1:0).map(([name,input])=>({path:name,sha256:input.sha256,bytes:input.bytes.length})),dynamicImports:[...dynamic].sort(),output:{path:'host.bundle.js',sha256:digest(output),bytes:output.length}};
  const receiptBytes=JSON.stringify(receipt,null,2)+'\n',bundlePath=path.join(shellRoot,'host.bundle.js'),receiptPath=path.join(shellRoot,'host-bundle.json');
  if(check){
   checkHostBundleBytes(shellRoot,output,receiptBytes);
  }else{fs.writeFileSync(bundlePath,output);fs.writeFileSync(receiptPath,receiptBytes);}
  return receipt;
 }catch(error){if(Object.hasOwn(messages,error?.code))throw error;throw failure('invalidHostBundleInput');}
}
if(process.argv[1]&&path.resolve(process.argv[1])===generator){
 try{if(process.argv.slice(2).some(arg=>arg!=='--check'))throw failure('invalidHostBundleInput');const receipt=await projectHostBundle({check:process.argv.includes('--check')});console.log(`host bundle: ${receipt.inputs.length} inputs, ${receipt.output.bytes} bytes, sha256 ${receipt.output.sha256}`);}catch(error){console.error(`${error.code}: ${error.message}`);process.exitCode=1;}
}
