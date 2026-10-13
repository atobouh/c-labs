// Dependency-free native WebView2 integration test. Node 22+; never uses learner data.
import {spawn} from 'node:child_process';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {resolve,join} from 'node:path';
import assert from 'node:assert/strict';

const executable=process.argv[2];
const output=process.argv[3];
if(!executable||!output)throw Error('Usage: node tools/test_desktop_r03.mjs <exe> <new evidence directory>');
const root=resolve(output);
await mkdir(root,{recursive:false}); // Refuse to overwrite a prior evidence/data directory.
const data=join(root,'learner');await mkdir(data);
const port=9227;
const evidence={executable:resolve(executable),data,checks:[],errors:[]};
let child,client;
const delay=ms=>new Promise(r=>setTimeout(r,ms));
async function until(fn,label,limit=20000){const start=Date.now();while(Date.now()-start<limit){try{const result=await fn();if(result)return result;}catch{}await delay(80);}throw Error('Timed out: '+label);}
async function connect(){
  const target=await until(async()=>{const targets=await(await fetch(`http://127.0.0.1:${port}/json/list`)).json();return targets.find(t=>t.url.includes('studio.html'));},'native page');
  const socket=new WebSocket(target.webSocketDebuggerUrl);await new Promise((r,j)=>{socket.addEventListener('open',r,{once:true});socket.addEventListener('error',j,{once:true});});
  let next=0;const pending=new Map();
  socket.addEventListener('message',event=>{const msg=JSON.parse(event.data);if(msg.id){const p=pending.get(msg.id);pending.delete(msg.id);if(msg.error)p?.reject(Error(JSON.stringify(msg.error)));else p?.resolve(msg.result);}else if(msg.method==='Runtime.exceptionThrown')evidence.errors.push(msg.params.exceptionDetails.text+': '+JSON.stringify(msg.params.exceptionDetails.exception));});
  client={send(method,params={}){return new Promise((resolve,reject)=>{const id=++next;pending.set(id,{resolve,reject});socket.send(JSON.stringify({id,method,params}));});},close(){socket.close();}};
  await client.send('Runtime.enable');await until(()=>evaluate('Boolean(window.MachineRoomBridge && document.querySelector(\'[data-route="chapter/learn"]\'))'),'course catalog');
  await client.send('Network.enable');
  await client.send('Network.emulateNetworkConditions',{offline:true,latency:0,downloadThroughput:0,uploadThroughput:0});
}
async function launch(){child=spawn(resolve(executable),[],{windowsHide:true,env:{...process.env,CLAB_DATA_DIR:data,WEBVIEW2_USER_DATA_FOLDER:join(root,'webview'),WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:`--remote-debugging-port=${port}`},stdio:'ignore'});await connect();}
async function stop(){client?.close();const processToStop=child;if(!processToStop||processToStop.exitCode!==null)return;await new Promise(r=>{processToStop.once('exit',r);processToStop.kill();});await delay(350);}
async function evaluate(expression){const r=await client.send('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});if(r.exceptionDetails)throw Error(r.exceptionDetails.exception?.description||r.exceptionDetails.text);return r.result.value;}
const invoke=(command,args={})=>evaluate(`window.__TAURI__.core.invoke(${JSON.stringify(command)},${JSON.stringify(args)})`);
const click=selector=>evaluate(`document.querySelector(${JSON.stringify(selector)}).click()`);
const text=selector=>evaluate(`document.querySelector(${JSON.stringify(selector)})?.textContent`);
const route=phase=>evaluate(`window.MachineRoomBridge.navigate(${JSON.stringify('chapter/'+phase)},false)`);
async function fill(selector,value){await evaluate(`(()=>{const e=document.querySelector(${JSON.stringify(selector)});e.value=${JSON.stringify(value)};e.dispatchEvent(new Event('input',{bubbles:true}));})()`);}
async function snapshot(name){await client.send('Page.enable');const image=await client.send('Page.captureScreenshot',{format:'png',captureBeyondViewport:false});await writeFile(join(root,name+'.png'),Buffer.from(image.data,'base64'));}
function pass(name,detail){evidence.checks.push({name,detail});console.log('PASS '+name+(detail?' · '+detail:''));}
async function submitFiles(files){for(let i=0;i<files.length;i++){await click(`[data-pilot-file="${i}"]`);await fill('#pilotCode',files[i].text);}await until(()=>text('#pilotDraftState').then(t=>t==='Saved locally'),'saved package');}
async function grade(){await click('[data-pilot-action="grade"]');await until(()=>evaluate('Boolean(document.querySelector("[data-pilot-action=grade]")?.disabled)'),'grade start');await until(()=>evaluate('Boolean(!document.querySelector("[data-pilot-action=grade]")?.disabled && !document.querySelector("[data-pilot-action=cancel]")?.hidden===false)'),'grade end',30000);return(await invoke('get_learning_state')).attempts[0];}
async function reference(item){const manifest=JSON.parse(await readFile(item.artifact,'utf8'));const folder=resolve(item.artifact,'..');return Promise.all(manifest.learner_source_paths.map(async(path,i)=>({path,text:await readFile(join(folder,manifest.solution_files[i]),'utf8')})));}
try{
  await launch();
  const pilot=await invoke('get_pilot');assert.equal(pilot.items.length,13);pass('native bundled pilot','13 verified learning records');
  await route('learn');assert.match(await text('.reader'),/Objects side by side/);
  for(let i=0;i<4;i++){
    await click(`[data-pilot-lesson="${i}"]`);await click('.pilot-example summary');await click('[data-pilot-action="example-run"]');await until(()=>text('#exampleOutput').then(t=>t?.startsWith('passed')),'real lesson example');
  }
  await click('[data-pilot-lesson="0"]');await snapshot('lesson');pass('all four lesson examples execute offline','including the multi-file pointer/count example');
  await click('[data-pilot-action="read-next"]');await until(()=>text('.reader').then(t=>t?.includes('An index is an offset')),'next lesson after saving read flag');pass('lesson navigation and read persistence');
  await route('exercises');await fill('[name=index]','2');await fill('[name=value]','9');await evaluate('document.querySelector("#pilotAnswer").requestSubmit()');await until(()=>text('#exerciseFeedback').then(t=>t?.includes('Indexing starts')),'wrong feedback');await click('[data-pilot-action="hint"]');await until(()=>text('.pilot-hints').then(t=>t?.includes('Hint 1')),'hint');
  await fill('[name=index]','3');await fill('[name=value]','21');await evaluate('document.querySelector("#pilotAnswer").requestSubmit()');await until(()=>text('#exerciseFeedback').then(t=>t?.includes('Correct reasoning')),'correct feedback');assert.equal((await invoke('get_learning_state')).values['review-exercise-offset'].active,true);await snapshot('exercise');pass('wrong/correct typed answer, hint, and review queue');
  await click('[data-pilot-exercise="1"]');await fill('[name=index]','5');await evaluate('document.querySelector("#pilotAnswer").requestSubmit()');await until(()=>text('#exerciseFeedback').then(t=>t?.includes('Correct reasoning')),'last element');
  await click('[data-pilot-exercise="2"]');await fill('[name=index]','6');await evaluate('document.querySelector("[name=can_form]").value="true";document.querySelector("[name=can_dereference]").value="false";document.querySelector("#pilotAnswer").requestSubmit()');await until(()=>text('#exerciseFeedback').then(t=>t?.includes('Correct reasoning')),'one-past');pass('all three exercise records');
  await click('[data-pilot-animation="0"]');for(let i=0;i<7;i++)await click('[data-pilot-action="model-next"]');assert.match(await text('#modelCaption'),/one-past position is not read/);await click('[data-pilot-action="model-back"]');assert.match(await text('#modelCaption'),/99/);await click('[data-pilot-action="model-reset"]');assert.match(await text('#modelCaption'),/No element/);await click('[data-pilot-action="model-pause"]');assert.equal(await text('[data-pilot-action="model-pause"]'),'Paused');
  await click('[data-pilot-action="models-close"]');await client.send('Emulation.setEmulatedMedia',{features:[{name:'prefers-reduced-motion',value:'reduce'}]});await click('[data-pilot-animation="1"]');for(let i=0;i<3;i++)await click('[data-pilot-action="model-next"]');assert.match(await text('.pilot-variables'),/not evaluated/);assert.equal(await evaluate('document.querySelector(".pilot-static").open'),true);await snapshot('boundary');pass('reversible models, boundary stop, and reduced-motion static view');
  await route('labs');
  for(let i=0;i<3;i++){
    if(i)await evaluate(`(()=>{const select=document.querySelector('#pilotLab');select.value='${i}';select.dispatchEvent(new Event('change',{bubbles:true}));})()`);
    await until(()=>evaluate('Boolean(document.querySelector("#pilotCode"))'),'lab package');const item=pilot.items.filter(i=>i.kind==='lab')[i];
    assert.equal((await grade()).status,'failed');await submitFiles(await reference(item));const result=await grade();assert.equal(result.status,'passed');pass('native '+item.id+' grading',`${result.checks.filter(c=>c.status==='passed').length} real passing checks; starter rejected`);
  }
  await evaluate('document.querySelector(".pilot-workbench").scrollIntoView({block:"start"})');await snapshot('lab');await route('project');await until(()=>evaluate('Boolean(document.querySelector("#pilotCode"))'),'project package');
  assert.equal((await grade()).status,'failed');assert.equal((await invoke('get_learning_state')).gates.length,0);const project=pilot.items.find(i=>i.kind==='project');const correct=await reference(project);await submitFiles(correct);
  await click('#pilotSelfReview');
  // Change the saved source during compilation: a real pass for the earlier bytes must not unlock.
  await click('[data-pilot-action="grade"]');await until(()=>evaluate('Boolean(document.querySelector("[data-pilot-action=grade]")?.disabled)'),'stale run begins');await fill('#pilotCode',correct[1].text+'\n/* revised while checks ran */\n');await until(()=>evaluate('Boolean(!document.querySelector("[data-pilot-action=grade]")?.disabled)'),'stale run ends',30000);
  let learner=await invoke('get_learning_state');assert.equal(learner.attempts[0].status,'passed');assert.equal(learner.gates.length,0);pass('stale passing submission cannot unlock');
  await submitFiles(correct);const accepted=await grade();assert.equal(accepted.status,'passed');learner=await invoke('get_learning_state');assert.equal(learner.gates[0].chapter_id,'foundations-strings-input');assert.equal(learner.gates[0].attempt_id,accepted.job_id);await evaluate('document.querySelector(".pilot-workbench").scrollIntoView({block:"start"})');await snapshot('project');await evaluate('document.querySelector(".pilot-rubric").scrollIntoView({block:"start"})');await snapshot('project-gate');pass('real project acceptance and version-bound unlock',`${accepted.checks.length} passing checks`);
  await click('[data-pilot-file="0"]');await fill('#pilotCode',correct[0].text+'\n/* draft retained after accepted attempt */\n');await until(()=>text('#pilotDraftState').then(t=>t==='Saved locally'),'edited project saved');const selection=await evaluate('(()=>{const e=document.querySelector("#pilotCode");e.focus();e.setSelectionRange(12,24);e.scrollTop=100;const css=getComputedStyle(e);return {outline:css.outlineStyle,shadow:css.boxShadow,selection:e.selectionStart,scroll:e.scrollTop};})()');assert.equal(selection.outline,'none');assert.equal(selection.shadow,'none');pass('editor focus, selection, scrolling',JSON.stringify(selection));
  await stop();await launch();learner=await invoke('get_learning_state');assert.equal(learner.values['lesson-objects-side-by-side'].read,true);assert.equal(learner.values['hint-exercise-offset'],1);assert.equal(learner.values['review-exercise-offset'].active,true);assert.equal(learner.gates[0].attempt_id,accepted.job_id);await route('project');await until(()=>evaluate('Boolean(document.querySelector("#pilotCode"))'),'restored project');assert.match(await evaluate('document.querySelector("#pilotCode").value'),/draft retained/);pass('relaunch restores lesson, hint, review, changed draft, attempt history, and gate');
  // A hard interruption preserves the draft and pending attempt, never a fabricated pass.
  const endless=correct.map(f=>({...f}));endless[0].text=endless[0].text.replace(/\{/, '{\n    for (;;) { }');await submitFiles(endless);await click('[data-pilot-action="grade"]');await until(()=>text('#pilotRunState').then(t=>t==='running checks'),'cancellable native execution');await click('[data-pilot-action="cancel"]');await until(()=>text('#pilotRunState').then(t=>t==='cancelled'),'cancelled grading');learner=await invoke('get_learning_state');assert.equal(learner.attempts[0].status,'cancelled');assert.equal(learner.gates[0].attempt_id,accepted.job_id);pass('cancelled grading cannot replace the accepted gate');
  await click('[data-pilot-action="grade"]');await until(()=>text('#pilotRunState').then(t=>t==='running checks'),'interruptible native execution');await stop();await launch();learner=await invoke('get_learning_state');assert.equal(learner.attempts[0].status,'interrupted');assert.equal(learner.gates[0].attempt_id,accepted.job_id);pass('hard-interrupted grading remains interrupted; earlier accepted gate unchanged');
  await route('project');await until(()=>evaluate('Boolean(document.querySelector("#pilotCode"))'),'restored interrupted draft');await submitFiles(correct);assert.equal((await grade()).status,'passed');pass('successful grading after interrupted process');
  await evaluate('window.MachineRoomBridge.navigate("review",false)');assert.match(await text('#screen'),/Read the offset/);await snapshot('review');
  await evaluate('window.MachineRoomBridge.navigate("projects",false)');assert.match(await text('#screen'),/Accepted attempt retained/);await snapshot('projects');
  await evaluate('window.MachineRoomBridge.navigate("course/foundations",false)');assert.match(await text('#screen'),/Unlocked \u00b7 outline/);await snapshot('course');
  await evaluate('window.MachineRoomBridge.navigate("lexicon",false)');assert.match(await text('#screen'),/The C lexicon/);await snapshot('lexicon');
  await evaluate('window.MachineRoomBridge.navigate("openlab",false)');await until(()=>text('#openConsoleLabel').then(t=>t?.includes('Ready')),'open lab compiler state');await snapshot('openlab');pass('course, project collection, review, lexicon, and Open Lab routes remain live');
  await route('project');await until(()=>evaluate('Boolean(document.querySelector("#pilotCode"))'),'project layout');
  for(const width of [1024,390]){await client.send('Emulation.setDeviceMetricsOverride',{width,height:844,deviceScaleFactor:1,mobile:false});assert.equal(await evaluate('document.documentElement.scrollWidth>window.innerWidth'),false);await snapshot('project-'+width);}await client.send('Emulation.clearDeviceMetricsOverride');pass('compiled UI at 1024px and 390px','no page horizontal overflow');
  assert.deepEqual(evidence.errors,[]);pass('no native JavaScript exceptions');
}catch(e){evidence.failure=e.stack;console.error(e.stack);process.exitCode=1;}finally{await writeFile(join(root,'evidence.json'),JSON.stringify(evidence,null,2));await stop();}
