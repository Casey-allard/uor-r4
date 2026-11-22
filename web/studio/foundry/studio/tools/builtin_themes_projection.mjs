// Reproduce the imported renderer schemas; this is not a Prism compiler.
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {fileURLToPath} from 'node:url';
const root=fileURLToPath(new URL('../',import.meta.url)),vendor=path.join(root,'vendor/prism'),require=createRequire(path.join(vendor,'package.json')),esbuild=require('esbuild');
const result=await esbuild.build({absWorkingDir:vendor,stdin:{contents:'export {themesData} from "./src/shared/themes";',resolveDir:vendor,sourcefile:'foundry-builtin-themes.ts',loader:'ts'},bundle:true,platform:'browser',target:'es2022',format:'esm',minify:true,legalComments:'inline',write:false,metafile:true,logLevel:'silent',});
const bytes=result.outputFiles[0].contents,hash=b=>createHash('sha256').update(b).digest('hex');
const inputs=Object.keys(result.metafile.inputs).filter(p=>p!=='foundry-builtin-themes.ts').sort().map(p=>({path:p,sha256:hash(fs.readFileSync(path.resolve(vendor,p)))}));
const record={profile:'foundry/imported-builtin-themes/1',compiler:esbuild.version,generatorSha256:hash(fs.readFileSync(fileURLToPath(import.meta.url))),packageLockSha256:hash(fs.readFileSync(path.join(vendor,'package-lock.json'))),licenseSha256:hash(fs.readFileSync(path.join(vendor,'LICENSE'))),inputs,output:{path:'builtin-themes.mjs',sha256:hash(bytes),bytes:bytes.length}};
const target=path.join(root,'shell/builtin-themes.mjs'),lock=path.join(root,'BUILTIN_THEMES_LOCK.json'),expected=JSON.stringify(record,null,2)+'\n';
if(process.argv.includes('--check')){if(!fs.existsSync(target)||hash(fs.readFileSync(path.join(root,'shell/builtin-themes.LICENSE.txt')))!==record.licenseSha256||!Buffer.from(bytes).equals(fs.readFileSync(target))||fs.readFileSync(lock,'utf8')!==expected)throw Error('Imported theme catalog projection changed');console.log('Imported theme catalogs reproduce exactly');}
else{fs.writeFileSync(target,bytes);fs.writeFileSync(lock,expected);fs.copyFileSync(path.join(vendor,'LICENSE'),path.join(root,'shell/builtin-themes.LICENSE.txt'));console.log(JSON.stringify({inputs:inputs.length,bytes:bytes.length,sha256:record.output.sha256}));}
