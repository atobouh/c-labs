(() => {
  'use strict';
  const invoke = (command, args) => window.__TAURI__.core.invoke(command, args);
  const $ = selector => document.querySelector(selector);
  const native = () => typeof window.__TAURI__?.core?.invoke === 'function';
  let compiler = null, detecting = null, context = {}, activeJob = null, last = null, lastSource = null, lastStandard = null, lastLibraries = '', lastCompiler = '';
  const labels = {'passed':'Exited successfully','failed':'Program failed','compile-error':'Compilation error','timed-out':'Deadline exceeded','cancelled':'Cancelled','output-limit':'Output limit reached','toolchain-unavailable':'Compiler unavailable','internal-error':'Run unavailable'};

  async function detect() {
    if (!native()) return null;
    if (detecting) return detecting;
    detecting = invoke('toolchain_status').then(info => { compiler = info; refresh(); return info; }).catch(error => { compiler = {available:false,message:String(error)}; refresh(); return compiler; });
    return detecting;
  }

  function refresh() {
    if (!$('#openCode')) return;
    const run = $('#nativeRun'), cancel = $('#nativeCancel'), status = $('#toolchainStatus');
    const unsupported = $('#standardMode')?.value === 'C23' && !compiler?.c23;
    if (run) { run.disabled = !!activeJob || !compiler?.available || unsupported; run.textContent = activeJob ? 'Running…' : 'Run code'; }
    if (cancel) cancel.hidden = !activeJob;
    if (status) { status.textContent = !compiler ? 'Checking compiler…' : compiler.available ? compiler.identity : 'Compiler unavailable'; status.title = compiler?.message || ''; }
    const option = $('#standardMode option[value="C23"]') || [...($('#standardMode')?.options || [])].find(o => o.value === 'C23');
    if (option && compiler) { option.value='C23'; option.disabled = !compiler.c23; option.textContent = compiler.c23 ? 'C23' : 'C23 (compiler unavailable)'; }
    if (unsupported && !activeJob) $('#openConsoleLabel').textContent = 'Choose C17 or a newer compiler';
    else if (last && !activeJob) showResult(last);
    else if (!activeJob && compiler && !compiler.available) { $('#openOutput').textContent = compiler.message; $('#openConsoleLabel').textContent = 'Compiler unavailable'; }
  }

  function mount(options) {
    if (!native() || !$('#openCode')) return;
    context = options || {};
    const flagsLabel=document.querySelector('label[for="libraryFlags"]');
    if(flagsLabel) flagsLabel.textContent='Link libraries';
    if($('#libraryFlags')) $('#libraryFlags').placeholder='-lm or -lSDL2';
    const libraryNote=$('#libraryForm .form-note');
    if(libraryNote) libraryNote.textContent='The folder is used for include and library search. Only -lNAME link options are accepted; libraries must already be installed locally.';
    const button = $('.open-editor [data-action="open-preview"]');
    if (button) { button.id = 'nativeRun'; button.dataset.action = 'native-run'; button.textContent = 'Run code'; button.insertAdjacentHTML('beforebegin','<button type="button" class="button runtime-cancel" id="nativeCancel" hidden>Cancel</button>'); }
    if (!$('#toolchainStatus')) $('.openlab-top').insertAdjacentHTML('beforeend','<div class="toolchain-strip"><span class="status-dot" aria-hidden="true"></span><span id="toolchainStatus" role="status">Checking compiler…</span><button type="button" class="quiet-button" id="configureCompiler">Configure</button></div>');
    $('.openlab-note').textContent = 'Code runs locally with your installed compiler. Drafts stay on this computer. Only run code you trust.';
    if (!last && !activeJob) { $('#openOutput').textContent = 'Run your source to see its actual output and compiler diagnostics.'; $('#openConsoleLabel').textContent = 'Ready'; }
    refresh(); detect();
  }

  function setOutput(text) { const el = $('#openOutput'); if (el && el.textContent !== text) el.textContent = text; }
  function showResult(result) {
    if (!$('#openCode')) return;
    const stale = lastSource !== null && (lastSource !== $('#openCode').value || lastStandard!==$('#standardMode').value || lastLibraries!==JSON.stringify(context.libraries || []) || lastCompiler!==compiler?.path+'|'+compiler?.identity);
    $('#openConsoleLabel').textContent = (stale ? 'Previous run · ' : '') + (labels[result.status] || result.status) + ' · ' + result.elapsed_ms + ' ms';
    setOutput([result.stdout, result.stderr, result.message].filter(Boolean).join('\n'));
    const profile = $('#openProfile'); if (profile) profile.textContent = lastStandard + ' · ' + (result.exit_code === null ? result.status : 'exit ' + result.exit_code) + (stale ? ' · earlier submission' : '');
  }

  async function run() {
    if (activeJob || !$('#openCode')) return;
    const source = $('#openCode').value;
    lastSource = source; lastStandard=$('#standardMode').value; lastLibraries=JSON.stringify(context.libraries || []); lastCompiler=compiler?.path+'|'+compiler?.identity; last = null;
    setOutput('Compiling your source…');
    $('#openConsoleLabel').textContent = 'Starting';
    $('#nativeRun').disabled = true;
    try {
      const libraries=context.libraries || [], links=[];
      for(const library of libraries) for(const flag of (library.flags || '').split(/\s+/).filter(Boolean)) {
        if(!/^-l[A-Za-z0-9_-]+$/.test(flag)) throw new Error('Use link-library names such as -lm or -lSDL2. Add include/library directories through the Local folder field.');
        links.push(flag.slice(2));
      }
      const id = await invoke('start_run',{request:{files:[{path:'scratch.c',text:source}],standard:$('#standardMode').value,include_dirs:libraries.map(l=>l.path),link_libraries:links}});
      activeJob = id; refresh();
      for (;;) {
        const snapshot = await invoke('get_run',{jobId:id});
        if (activeJob !== id) return;
        if (snapshot.result) { last = snapshot.result; activeJob = null; refresh(); showResult(last); return; }
        if ($('#openConsoleLabel')) $('#openConsoleLabel').textContent = snapshot.phase === 'compiling' ? 'Compiling…' : snapshot.phase === 'running' ? 'Running locally…' : 'Queued';
        if (snapshot.stdout || snapshot.stderr) setOutput([snapshot.stdout,snapshot.stderr].filter(Boolean).join('\n'));
        await new Promise(resolve=>setTimeout(resolve,100));
      }
    } catch(error) { activeJob = null; setOutput(String(error)); if ($('#openConsoleLabel')) $('#openConsoleLabel').textContent = 'Could not run'; refresh(); }
  }

  function compilerDialog() {
    let dialog = $('#compilerDialog');
    if (!dialog) {
      dialog = document.createElement('dialog'); dialog.id = 'compilerDialog'; dialog.className = 'library-dialog'; dialog.setAttribute('aria-labelledby','compilerDialogTitle');
      dialog.innerHTML = '<form id="compilerForm"><div class="dialog-top"><h2 id="compilerDialogTitle">Local C compiler</h2><button class="button" type="button" id="closeCompiler">Close</button></div><p>Choose an installed GCC or Clang executable. The app uses C17 and does not download a toolchain.</p><label for="compilerPath">Compiler executable</label><input id="compilerPath" autocomplete="off" placeholder="C:\\…\\bin\\gcc.exe" required><p id="compilerIdentity"></p><p id="compilerError" role="alert"></p><button class="button primary" type="submit">Use compiler</button></form>';
      document.body.append(dialog);
      $('#closeCompiler').addEventListener('click',()=>dialog.close());
      $('#compilerForm').addEventListener('submit',async event=>{ event.preventDefault(); const button=event.submitter; button.disabled=true; $('#compilerError').textContent='Checking compiler…'; try { compiler=await invoke('set_compiler_path',{path:$('#compilerPath').value.trim()}); detecting=Promise.resolve(compiler); dialog.close(); if(!last){setOutput('Run your source to see its actual output and compiler diagnostics.');if($('#openConsoleLabel'))$('#openConsoleLabel').textContent='Ready';} refresh(); } catch(error) { $('#compilerError').textContent=String(error); } finally { button.disabled=false; } });
    }
    $('#compilerPath').value=compiler?.path || ''; $('#compilerIdentity').textContent=compiler?.available ? compiler.identity+' · '+compiler.target : compiler?.message || ''; $('#compilerError').textContent=''; dialog.showModal();
  }

  document.addEventListener('click',async event=>{
    const button=event.target.closest('button'); if(!button || !native()) return;
    if(button.id==='nativeRun') run();
    if(button.id==='nativeCancel' && activeJob) { button.disabled=true; try { await invoke('cancel_run',{jobId:activeJob}); } catch(error) { setOutput(String(error)); } finally { button.disabled=false; } }
    if(button.id==='configureCompiler') compilerDialog();
  });
  document.addEventListener('input',event=>{ if(event.target.id==='openCode' && last && !activeJob) showResult(last); });
  document.addEventListener('change',event=>{ if(event.target.id==='standardMode') { context.standard=event.target.value; refresh(); } });
  window.MachineRoomRuntime={mount,detect,compiler:()=>compiler};
})();
