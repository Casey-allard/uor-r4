'use strict';
// Source adapter only: original Showcase fold and serialized records are unchanged.
const fs = require('node:fs'), path = require('node:path'), crypto = require('node:crypto');
const base = path.resolve(process.argv[2] || path.join(__dirname, '../../source-inputs/SpiralCore_Showcase'));
const output = path.resolve(process.argv[3] || __dirname);
const sha = b => crypto.createHash('sha256').update(b).digest('hex');
if (sha(fs.readFileSync(path.join(base, 'core.js'))) !== '7ca3dcfb552ca6ff3fc8696734265bcd9b993aa8b8e6242984060ace8b35bc22') throw Error('Source pin mismatch');
if (sha(fs.readFileSync(path.join(base, 'transport.js'))) !== '1d254a75dfe497f91b302ada0d581eab3407c7ea8f3994920a58d9a2652dc0fe') throw Error('Transport source pin mismatch');
const C = require(path.join(base, 'core.js'));
const T = require(path.join(base, 'transport.js'));
fs.mkdirSync(output, {recursive:true});
const inputs = {
  unique: Buffer.concat(Array.from({length: 1024}, (_,i) => crypto.createHash('sha256').update('CS_RANGE_ATOM_'+i).digest())),
  repeated: Buffer.concat(Array.from({length: 243}, (_,i) => Buffer.alloc(32, [65,66,67][i%3]))),
  utf8: Buffer.from('a'.repeat(31)+'🌀'+'b'.repeat(28)+'音楽'+'é\0tail\0'.repeat(67)),
  empty: Buffer.alloc(0)
};
(async () => {
  const result = {scope:'BYTE_DAG_FIXTURE_CONSTRUCTION_ONLY', node:process.version, sourceSha256:sha(fs.readFileSync(path.join(base,'core.js'))), transportSha256:sha(fs.readFileSync(path.join(base,'transport.js'))), cases:[]};
  for(const [name, bytes] of Object.entries(inputs)) {
    const start = process.hrtime.bigint();
    const f = await C.fold(new Uint8Array(bytes));
    const nodes = new Map(f.nodes.map(n=>[n.id,n]));
    const records = T.recordsFor(nodes, f.roots, true);
    const elapsed = process.hrtime.bigint()-start;
    const scales = C.scalesFor(f.n);
    // This small manifest is adapter framing, not a claimed Showcase wire format.
    const manifest = [bytes.length, f.n, f.roots.length, ...f.roots.flatMap((id,i)=>[id,scales[i]])].join(' ')+'\n';
    fs.writeFileSync(path.join(output,name+'.raw'),bytes);
    fs.writeFileSync(path.join(output,name+'.records'),records);
    fs.writeFileSync(path.join(output,name+'.manifest'),manifest);
    result.cases.push({name,inputBytes:bytes.length,uniqueNodes:nodes.size,recordBytes:records.length,rootOccurrences:f.roots.length,constructionNanoseconds:elapsed.toString(),inputSha256:sha(bytes),recordSha256:sha(records)});
  }
  fs.writeFileSync(path.join(output,'construction.json'),JSON.stringify(result,null,2)+'\n');
  process.stdout.write(JSON.stringify(result,null,2)+'\n');
})().catch(e=>{console.error(e);process.exitCode=1;});
