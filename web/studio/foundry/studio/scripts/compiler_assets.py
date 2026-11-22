"""Bounded compiler archive adapter for FoundrySpecification.compilerAssetBoundary."""
import pathlib
from editor_assets import EditorAssetError, stage_editor_assets, check_editor_assets
COMPILER = {"name": "esbuild-wasm", "version": "0.25.5",
    "url": "https://registry.npmjs.org/esbuild-wasm/-/esbuild-wasm-0.25.5.tgz",
    "lock": "runtime/package-lock.json", "destination": "compiler",
    "profile": "foundry/compiler-assets/1",
    "required": {"LICENSE.md", "esm/browser.min.js", "esbuild.wasm"},
    "prefix": "", "strip": "package/",
    "selected": {"package/LICENSE.md", "package/esm/browser.min.js", "package/esbuild.wasm"}}
class CompilerAssetError(Exception):
    def __init__(self, code):
        self.code = code
        super().__init__(f"Compiler assets refused: {code}")
def compiler_assets(root, check=False):
    try:
        return (check_editor_assets if check else stage_editor_assets)(root, COMPILER)
    except EditorAssetError as error:
        raise CompilerAssetError(error.code.replace("Editor", "Compiler").replace("editor", "compiler")) from None
if __name__ == "__main__":
    import sys
    receipt = compiler_assets(pathlib.Path(__file__).resolve().parent.parent, "--check" in sys.argv)
    print(f"verified {len(receipt['files'])} locked compiler files")
