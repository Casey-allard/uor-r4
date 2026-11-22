const DESTINATIONS = {
    'uor.studio.conversations.v1': 'uor.studio.lifecycle.conversations.v2',
    'uor.studio.activeConversation.v1': 'uor.studio.lifecycle.activeConversation.v2',
    'uor.studio.editor.v1': 'uor.studio.lifecycle.editor.v2',
    'uor.studio.settings.v1': 'uor.studio.lifecycle.settings.v2',
    'uor.studio.ui.v2': 'uor.studio.lifecycle.ui.v3',
    'uor.studio.ui.v1': 'uor.studio.lifecycle.ui.v2',
    'uor.studio.attachments.v1': 'uor.studio.lifecycle.attachments.v2',
    'uor.studio.tabs.v2': 'uor.studio.lifecycle.tabs.v3',
    'uor.studio.recent.v2': 'uor.studio.lifecycle.recent.v3',
    'uor.studio.theme': 'uor.studio.lifecycle.theme.v1',
};
const MARKER_KEY = 'uor.studio.lifecycle.migration.v1';
const MARKER_VALUE = '{"schema":"uor-r4.studio-lifecycle-migration/1"}';

function fixture(mode) {
    const entries = {
        'uor.studio.conversations.v1': JSON.stringify([{ id: 'session-a', title: 'Migrated', createdAt: 1, updatedAt: 2, messages: [{ role: 'user', content: '17+25' }], future: { retained: true } }]),
        'uor.studio.activeConversation.v1': JSON.stringify('session-a'),
        'uor.studio.editor.v1': JSON.stringify({ content: 'seed draft', futureEditor: [1, 2] }),
        'uor.studio.modelSources.v1': JSON.stringify({ active: { kind: 'api', id: 'remote' }, api: [{ id: 'remote', apiKey: 'sentinel-not-a-secret' }] }),
        'uor.studio.settings.v1': JSON.stringify({ theme: 'light', fontSize: 16, futureSetting: true }),
        'uor.studio.ui.v2': JSON.stringify({ nav: 'expanded', futureFlag: 42 }),
        'uor.studio.ui.v1': JSON.stringify({ legacyNav: 'archival-only' }),
        'uor.studio.attachments.v1': JSON.stringify([{ name: 'seed.txt', content: 'seed context', futureAttachment: 'retained' }]),
        'uor.studio.tabs.v2': JSON.stringify({ activePath: 'draft.md', activeRootPath: '', tabs: [{ path: 'draft.md', content: 'seed draft', dirty: true, mode: 'draft', rootPath: '' }] }),
        'uor.studio.recent.v2': JSON.stringify([{ path: 'draft.md', rootPath: '' }]),
        'uor.studio.theme': 'light',
    };
    if (mode === 'malformed') entries['uor.studio.settings.v1'] = '{malformed-source';
    if (mode === 'conflict') entries['uor.studio.lifecycle.conversations.v2'] = '[{"conflicting":true}]';
    return entries;
}

export function migrationPreload(mode) {
    const entries = fixture(mode);
    return `(()=>{
        const entries=${JSON.stringify(entries)};
        if (!sessionStorage.getItem('uor.studio.fixture.migration.initialized')) {
            localStorage.clear();
            for (const [key, raw] of Object.entries(entries)) localStorage.setItem(key, raw);
            sessionStorage.setItem('uor.studio.fixture.migration.initialized', '1');
        }
        if (${JSON.stringify(mode)}==='quota') {
            const originalSet=Storage.prototype.setItem;
            let thrown=false;
            Storage.prototype.setItem=function(key,value){
                if(!thrown && key==='uor.studio.lifecycle.settings.v2'){
                    thrown=true;
                    throw new DOMException('one-shot browser quota','QuotaExceededError');
                }
                return originalSet.call(this,key,value);
            };
        }
        window.__migrationOriginal=entries;
    })();`;
}

export function migrationExpression(mode, phase) {
    const constants = { destinations: DESTINATIONS, markerKey: MARKER_KEY, markerValue: MARKER_VALUE };
    if (mode !== 'success') {
        return `(async()=>{
            const ${'constants'}=${JSON.stringify(constants)};
            const root=document.getElementById('studio-typed-root');
            const pause=(ms)=>new Promise((resolve)=>setTimeout(resolve,ms));
            let errorVisible=false;
            for(let attempt=0;attempt<75;attempt+=1){
                if(root?.textContent.includes('Browser-state migration failed:')){errorVisible=true;break;}
                await pause(100);
            }
            const original=window.__migrationOriginal||{};
            const oldPreserved=Object.keys(constants.destinations).every((key)=>localStorage.getItem(key)===original[key]);
            const destinationValues=Object.values(constants.destinations).map((key)=>localStorage.getItem(key));
            const conflictPreserved=${mode === 'conflict' ? "localStorage.getItem('uor.studio.lifecycle.conversations.v2')==='[{\"conflicting\":true}]'" : 'true'};
            const noUnexpectedDestinations=${mode === 'conflict' ? "destinationValues.filter((value)=>value!==null).length===1" : 'destinationValues.every((value)=>value===null)'};
            return {
                errorVisible,
                shellCount:document.querySelectorAll('#studio-typed-root .studio-shell').length,
                status:window.UORStudioPersistence?.status() || null,
                markerAbsent:localStorage.getItem(constants.markerKey)===null,
                oldPreserved,
                conflictPreserved,
                noUnexpectedDestinations,
                errorText:root?.textContent.slice(0,300) || '',
            };
        })()`;
    }
    if (phase === 'reload') {
        return `(()=>{
            const ${'constants'}=${JSON.stringify(constants)};
            const original=window.__migrationOriginal||{};
            const oldPreserved=Object.keys(constants.destinations).every((key)=>localStorage.getItem(key)===original[key]);
            const lifecycleSecrets=Object.keys(localStorage).filter((key)=>key.startsWith('uor.studio.lifecycle.')).map((key)=>localStorage.getItem(key)||'');
            return {
                shellCount:document.querySelectorAll('#studio-typed-root .studio-shell').length,
                status:window.UORStudioPersistence?.status() || null,
                marker:localStorage.getItem(constants.markerKey)===constants.markerValue,
                renamed:window.__studioUi?.listSessions().some((item)=>item.id==='session-a' && item.title==='Renamed migration') || false,
                settings:window.__studioUi?.getSettings().fontSize===22,
                workspace:window.__studioFiles?.activeFile()?.path==='draft.md' && window.__studioFiles?.activeFile()?.content==='edited draft',
                theme:document.documentElement.dataset.theme==='dark',
                currentUi:document.getElementById('app')?.dataset.nav==='collapsed',
                attachments:window.__studioUi?.getAttachments().some((item)=>item.name==='pasted-context.txt' && item.content==='reload context') || false,
                recentEmpty:(()=>{const list=document.getElementById('recentFiles');return Boolean(list)&&list.textContent.includes('No files opened yet.')&&!list.querySelector('.recent-file');})(),
                oldPreserved,
                deletedDestinationNotResurrected:localStorage.getItem('uor.studio.lifecycle.recent.v3')===null && localStorage.getItem('uor.studio.recent.v2')===original['uor.studio.recent.v2'],
                providerIsolated:!lifecycleSecrets.some((value)=>value.includes('sentinel-not-a-secret')) && localStorage.getItem('uor.studio.lifecycle.modelSources.v1')===null,
            };
        })()`;
    }
    return `(async()=>{
        const ${'constants'}=${JSON.stringify(constants)};
        const original=window.__migrationOriginal||{};
        const root=document.getElementById('studio-typed-root');
        const pause=(ms)=>new Promise((resolve)=>setTimeout(resolve,ms));
        const button=(label)=>Array.from(root.querySelectorAll('button')).find((candidate)=>candidate.textContent===label);
        const oldPreserved=()=>Object.keys(constants.destinations).every((key)=>localStorage.getItem(key)===original[key]);
        const providerIsolated=()=>{
            const lifecycleSecrets=Object.keys(localStorage).filter((key)=>key.startsWith('uor.studio.lifecycle.')).map((key)=>localStorage.getItem(key)||'');
            return !lifecycleSecrets.some((value)=>value.includes('sentinel-not-a-secret')) && localStorage.getItem('uor.studio.lifecycle.modelSources.v1')===null;
        };
        let shell=false;
        for(let attempt=0;attempt<75;attempt+=1){
            if(root?.querySelector('.studio-shell')){shell=true;break;}
            await pause(100);
        }
        const restored={
            session:window.__studioUi?.listSessions().some((item)=>item.id==='session-a' && item.title==='Migrated') || false,
            active:window.__studioUi?.activeSessionId()==='session-a',
            messages:window.__studioUi?.getConversationMessages('session-a').some((item)=>item.content==='17+25') || false,
            settings:window.__studioUi?.getSettings().fontSize===16,
            attachments:window.__studioUi?.getAttachments().some((item)=>item.name==='seed.txt' && item.content==='seed context') || false,
            workspace:window.__studioFiles?.activeFile()?.path==='draft.md' && window.__studioFiles?.activeFile()?.content==='seed draft',
            tabs:window.__studioFiles?.tabs().some((item)=>item.path==='draft.md') || false,
            theme:document.documentElement.dataset.theme==='light',
            currentUi:document.getElementById('app')?.dataset.nav==='expanded',
        };
        const destinationsCopied=Object.entries(constants.destinations).every(([source,destination])=>localStorage.getItem(destination)!==null);
        const archivalUiV1=localStorage.getItem('uor.studio.lifecycle.ui.v2')===original['uor.studio.ui.v1'];
        const oldPreservedBefore=oldPreserved();
        const rename=button('Rename Migrated');
        if(rename)rename.click();
        await pause(50);
        const title=root.querySelector('input[aria-label="Session title"]');
        if(title){
            title.focus();
            title.value='Renamed migration';
            title.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertText',data:'Renamed migration'}));
            await pause(0);
        }
        const save=button('Save title');
        if(save)save.click();
        await pause(100);
        const settings=button('Settings');
        if(settings)settings.click();
        await pause(50);
        const font=root.querySelector('input[aria-label="Editor font size"]');
        if(font){
            font.focus();
            font.value='22';
            font.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertText',data:'22'}));
            await pause(0);
            font.blur();
        }
        const themeSelect=root.querySelector('select[aria-label="Theme"]');
        if(themeSelect){
            themeSelect.value='dark';
            themeSelect.dispatchEvent(new Event('change',{bubbles:true}));
            await pause(0);
        }
        const navCollapse=document.getElementById('btnNavCollapse');
        if(navCollapse)navCollapse.click();
        const closeSettings=button('Close');
        if(closeSettings)closeSettings.click();
        await pause(50);
        const agent=button('Agent');
        if(agent)agent.click();
        await pause(50);
        const pasted=root.querySelector('textarea[aria-label="Pasted context"]');
        if(pasted){
            pasted.value='reload context';
            pasted.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertText',data:'reload context'}));
            await pause(0);
        }
        const attach=button('Attach pasted text');
        if(attach)attach.click();
        await pause(50);
        const workspace=button('Workspace');
        if(workspace)workspace.click();
        await pause(50);
        const editor=root.querySelector('textarea[aria-label="File contents"]');
        if(editor){
            editor.value='edited draft';
            editor.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertText',data:'edited draft'}));
            await pause(0);
        }
        await pause(100);
        const edits={
            renamed:window.__studioUi?.listSessions().some((item)=>item.id==='session-a' && item.title==='Renamed migration') || false,
            conversationDestination: (localStorage.getItem('uor.studio.lifecycle.conversations.v2')||'').includes('Renamed migration'),
            settings:window.__studioUi?.getSettings().fontSize===22,
            settingsDestination:JSON.parse(localStorage.getItem('uor.studio.lifecycle.settings.v2')||'{}').fontSize===22,
            theme:document.documentElement.dataset.theme==='dark' && localStorage.getItem('uor.studio.lifecycle.theme.v1')==='dark',
            currentUi:document.getElementById('app')?.dataset.nav==='collapsed' && JSON.parse(localStorage.getItem('uor.studio.lifecycle.ui.v3')||'{}').nav==='collapsed',
            attachments:window.__studioUi?.getAttachments().some((item)=>item.name==='pasted-context.txt' && item.content==='reload context') && (localStorage.getItem('uor.studio.lifecycle.attachments.v2')||'').includes('pasted-context.txt'),
            workspace:window.__studioFiles?.activeFile()?.content==='edited draft' && JSON.parse(localStorage.getItem('uor.studio.lifecycle.tabs.v3')||'{}').tabs[0].content==='edited draft',
        };
        const oldPreservedAfterEdits=oldPreserved();
        const providerIsolatedAfterEdits=providerIsolated();
        const diagnostics={
            renameFound:Boolean(rename),
            titleFound:Boolean(title),
            saveFound:Boolean(save),
            settingsFound:Boolean(settings),
            fontFound:Boolean(font),
            titles:window.__studioUi?.listSessions().map((item)=>({id:item.id,title:item.title})) || [],
            alerts:Array.from(root.querySelectorAll('[role="alert"]')).map((node)=>node.textContent),
            dialogs:Array.from(root.querySelectorAll('dialog')).map((node)=>({open:node.open,text:node.textContent.slice(0,200)})),
        };
        localStorage.removeItem('uor.studio.lifecycle.recent.v3');
        return {
            shell,
            status:window.UORStudioPersistence?.status() || null,
            marker:localStorage.getItem(constants.markerKey)===constants.markerValue,
            restored,
            destinationsCopied,
            archivalUiV1,
            oldPreservedBefore,
            edits,
            oldPreservedAfterEdits,
            providerIsolatedAfterEdits,
            diagnostics,
        };
    })()`;
}

export function assertMigrationReport(report, mode) {
    const checks = report?.migration;
    if (!checks) throw new Error('missing migration checks');
    const values = mode === 'success' ? checks : checks;
    if (mode !== 'success') {
        if (!values.errorVisible || values.shellCount !== 0 || values.status?.state !== 'error' || !values.markerAbsent || !values.oldPreserved || !values.conflictPreserved || !values.noUnexpectedDestinations) {
            throw new Error('migration failure fixture did not fail closed: ' + JSON.stringify(values));
        }
        if (mode === 'quota' && values.status?.error !== 'quota-exceeded') throw new Error('one-shot quota was retried or hidden: ' + JSON.stringify(values));
        return;
    }
    if (!values.shell || values.status?.state !== 'migrated' || !values.marker || !values.destinationsCopied || !values.archivalUiV1 || !values.oldPreservedBefore || !values.oldPreservedAfterEdits || !values.providerIsolatedAfterEdits) {
        throw new Error('migration success fixture did not preserve its contract: ' + JSON.stringify(values));
    }
    for (const [name, passed] of Object.entries(values.restored || {})) {
        if (!passed) throw new Error('migration did not restore ' + name + ': ' + JSON.stringify(values));
    }
    for (const [name, passed] of Object.entries(values.edits || {})) {
        if (!passed) throw new Error('migration edit failed ' + name + ': ' + JSON.stringify(values));
    }
    const reload = values.reload;
    if (!values.pageReloaded || !reload || reload.shellCount !== 1 || reload.status?.state !== 'current' || !reload.marker || !reload.renamed || !reload.settings || !reload.workspace || !reload.theme || !reload.currentUi || !reload.attachments || !reload.recentEmpty || !reload.oldPreserved || !reload.deletedDestinationNotResurrected || !reload.providerIsolated) {
        throw new Error('marked reload did not preserve the migrated state: ' + JSON.stringify(values));
    }
}
