"""Independent tar fixture for the compiler's locked asset profile."""
import base64, gzip, hashlib, io, json, pathlib, sys, tarfile, tempfile
from unittest.mock import patch
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1] / 'scripts'))
import editor_assets as a
from compiler_assets import COMPILER, CompilerAssetError, compiler_assets

def fixture(omit=(), name='esbuild-wasm'):
    rows = {'package/package.json': json.dumps({'name': name, 'version': '0.25.5'}).encode(),
            'package/LICENSE.md': b'independent license',
            'package/esm/browser.min.js': b'export const version="fixture";',
            'package/esbuild.wasm': b'\0asm\1\0\0\0'}
    stream=io.BytesIO()
    with tarfile.open(fileobj=stream,mode='w') as archive:
        for path,body in rows.items():
            if path in omit: continue
            item=tarfile.TarInfo(path); item.size=len(body); archive.addfile(item,io.BytesIO(body))
    return gzip.compress(stream.getvalue())
def locked(blob):
    return {'packages': {'node_modules/esbuild-wasm': {'version':'0.25.5','resolved':COMPILER['url'],'integrity':'sha512-'+base64.b64encode(hashlib.sha512(blob).digest()).decode()}}}
def refuse(fn,code,kind=a.EditorAssetError):
    try:fn()
    except kind as e: assert e.code==code,(e.code,code)
    else:raise AssertionError('invalid compiler accepted')
blob=fixture();lock=locked(blob)
files,receipt=a.decode_package(blob,lock,COMPILER)
assert set(files)=={'LICENSE.md','esm/browser.min.js','esbuild.wasm'}
assert files['esbuild.wasm']==b'\0asm\1\0\0\0'
assert receipt['source']['name']=='esbuild-wasm' and receipt['profile']=='foundry/compiler-assets/1'
assert receipt['files']==[{'path':p,'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)} for p,b in sorted(files.items())]
refuse(lambda:a.decode_package(blob+b'changed',lock,COMPILER),'editorPackageIntegrityMismatch')
for bad in [fixture(omit=['package/LICENSE.md']),fixture(omit=['package/esbuild.wasm']),fixture(name='imposter')]:
    refuse(lambda:a.decode_package(bad,locked(bad),COMPILER),'invalidEditorArchive')
wrong=locked(blob);wrong['packages']['node_modules/esbuild-wasm']['version']='0.25.6'
refuse(lambda:a.decode_package(blob,wrong,COMPILER),'invalidEditorLock')
with tempfile.TemporaryDirectory() as tmp:
    root=pathlib.Path(tmp);(root/'runtime').mkdir();(root/'shell').mkdir();(root/'runtime/package-lock.json').write_text(json.dumps(lock))
    def transport(command,**options):
        assert '--offline' in command and '--ignore-scripts' in command and options['timeout']==60
        (pathlib.Path(command[command.index('--pack-destination')+1])/'esbuild-wasm-0.25.5.tgz').write_bytes(blob)
    with patch.object(a.subprocess,'run',transport):
        compiler_assets(root);compiler_assets(root,True)
        (root/'shell/compiler/esbuild.wasm').write_bytes(b'altered')
        refuse(lambda:compiler_assets(root,True),'compilerPackageIntegrityMismatch',CompilerAssetError)
print(json.dumps({'passed':True,'files':3}))
