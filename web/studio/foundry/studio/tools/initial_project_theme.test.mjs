import assert from 'node:assert/strict';
import {test} from 'node:test';
import {invokeHost} from '../shell/host-boundary.mjs';
import fs from 'node:fs';
import {initialProjectTheme,readThemeCatalog,operateThemeLibrary} from '../shell/theme-library.mjs';
import {themesData} from '../shell/builtin-themes.mjs';
import {INITIAL_THEME_PATCHES} from '../vendor/prism/hologram.initial-theme.mjs';
const catalog=readThemeCatalog(themesData);
test('initial selection validates builtins custom references and empty preference',()=>{
 const {journal}=operateThemeLibrary(undefined,'create',{name:'Theme',prompt:'Exact'});
 assert.equal(initialProjectTheme(undefined,undefined,catalog),null);
 assert.equal(initialProjectTheme({selectedThemeId:''},undefined,catalog),null);
 assert.equal(initialProjectTheme({selectedThemeId:'default'},undefined,catalog),'default');
 assert.equal(initialProjectTheme({selectedThemeId:'custom:1'},journal,catalog),'custom:1');
 for(const selectedThemeId of ['custom:01','custom:2','unknown',1])assert.throws(()=>initialProjectTheme({selectedThemeId},journal,catalog));
 assert.throws(()=>initialProjectTheme({selectedThemeId:'default'},{},catalog),{code:'themeLibraryInvalid'});
});
test('initial theme catalog is detached and refuses invalid catalog boundaries',()=>{
 const source=structuredClone(themesData),copy=readThemeCatalog(source);source[0].id='changed';assert.equal(copy[0].id,catalog[0].id);
 for(const value of [[],[...themesData,themesData[0]],[{...themesData[0],id:'custom:1'}]])assert.throws(()=>readThemeCatalog(value),{code:'themeCatalogMismatch'});
});
test('first prompt reads committed selection and never reapplies React settings',()=>{
 const file=new URL('../vendor/prism/src/first_prompt/FirstPromptProvider.tsx',import.meta.url),original=fs.readFileSync(file,'utf8'),patch=INITIAL_THEME_PATCHES[0];
 assert.equal(INITIAL_THEME_PATCHES.length,1);assert.equal(patch.file,'src/first_prompt/FirstPromptProvider.tsx');assert.equal(original.split(patch.from).length,2);
 const source=original.replace(patch.from,patch.to),adapter=fs.readFileSync(new URL('../vendor/prism/hologram.screens.mjs',import.meta.url),'utf8');assert.ok(adapter.includes('import {INITIAL_THEME_PATCHES} from "./hologram.initial-theme.mjs";'));assert.ok(adapter.includes('  ...INITIAL_THEME_PATCHES,'));
 const body=source.slice(source.indexOf('async applyTheme(appId)'),source.indexOf('async openPreviewIfSetupRequired'));
 assert.ok(body.includes('await ipc.template.getAppTheme({ appId })'));assert.ok(!body.includes('setAppTheme')&&!body.includes('settings?.selectedThemeId'));
});
test('production creation and mandatory browser qualify initial theme atomically',async()=>{
 for(const code of ['projectScaffoldFailed','projectPreparationFailed','projectStorageFailed']){const reply=await invokeHost('create-app',[],()=>{throw Object.assign(Error('PRIVATE'),{code});},null);assert.equal(reply.error.code,code);assert.ok(!reply.error.message.includes('PRIVATE'));}

 const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
 assert.ok(read('../shell/host.js').includes('createStoredProject(await dbReady, prepared, nulls, themesData)'));
 assert.ok(read('../../.github/workflows/studio.yml').includes('node tools/initial_project_theme_browser.mjs'));
});
