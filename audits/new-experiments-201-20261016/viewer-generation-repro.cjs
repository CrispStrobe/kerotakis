const assert = require('node:assert/strict');
const cp = require('node:child_process');
const crypto = require('node:crypto');
const {join} = require('node:path');
const root = cp.execFileSync('git',['rev-parse','--show-toplevel'],{encoding:'utf8'}).trim();
const ts = require(join(root,'spikes/brd080/node_modules/typescript'));
assert.equal(ts.version,'5.9.3');
const head = 'c44692053a94c59dcf62e7fb67549f84c7d18f83';
const cache = {}, pins = [];
const expectedPins = {"spikes/brd080/src/comparison.ts": "3855843d1915ac906593f0e0c5737b6f3df53f235136cd305f61e11d96cbaaae", "spikes/brd080/src/adapter.ts": "b958c729dbe5207e3566ec48c33d19bac226a10fd115f8da77450332bb57b847", "spikes/brd080/src/3dmolAdapter.ts": "d364d3f37c253adbf2728d9ec81ff37f5c26180468c210a54553898f2fe3c30b"};
function load(name) {
 if(cache[name]) return cache[name];
 const path = `spikes/brd080/src/${name}.ts`;
 const source = cp.execFileSync('git',['show',`${head}:${path}`],{cwd:root,encoding:'utf8'});
 const digest=crypto.createHash('sha256').update(source).digest('hex');
 assert.equal(digest,expectedPins[path],`Unexpected source hash: ${path}`);
 pins.push({path,sha256:digest});
 const js = ts.transpileModule(source,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS}}).outputText;
 const module={exports:{}};cache[name]=module.exports;
 new Function('require','module','exports',js)((id)=>{assert.equal(id,'./adapter');return load('adapter');},module,module.exports);
 return module.exports;
}
const {ComparisonController}=load('comparison');
const {createThreeDMolAdapter}=load('3dmolAdapter');
function deferred(){let resolve;const promise=new Promise(r=>resolve=r);return {promise,resolve};}
function host(){const h={children:[],style:{width:'',height:''},clientWidth:500,clientHeight:300,getBoundingClientRect:()=>({width:500,height:300}),appendChild(c){this.children.push(c);c.host=this;},replaceChildren(){this.children.splice(0);},querySelectorAll(s){return s==='canvas'?this.children:[];}};return h;}
function canvas(id){return {id,style:{},remove(){const i=this.host.children.indexOf(this);if(i>=0)this.host.children.splice(i,1);}};}
function moduleFor(id){return {createViewer(h){h.appendChild(canvas(id));return {addModel(){return {setStyle(){},setClickable(){}};},resize(){},render(){},zoomTo(){},stopAnimate(){},clear(){},removeAllLabels(){}};}};}
const fixture=(id)=>({id,kind:'molecule',format:'sdf',text:'local fixture',description:id,atoms:[{id:0,element:'O',x:0,y:0,z:0}],bonds:[]});
global.window={devicePixelRatio:1};global.document={};global.devicePixelRatio=1;
async function staleLoader(){
 const h=host(),gates=[deferred(),deferred()],started=[deferred(),deferred()];let n=0;
 const adapter=createThreeDMolAdapter(()=>{const i=n++;started[i].resolve();return gates[i].promise;});
 const c=new ComparisonController(h,()=>{});
 const old=c.show(adapter,fixture('old'),true);await started[0].promise;
 const current=c.show(adapter,fixture('current'),true);await started[1].promise;
 gates[1].resolve(moduleFor('current-canvas'));await current;
 assert.equal(c.status.state,'ready');assert.deepEqual(h.children.map(x=>x.id),['current-canvas']);
 gates[0].resolve(moduleFor('old-canvas'));await old;
 assert.equal(c.status.state,'ready');assert.equal(c.snapshot().fixture,'current');assert.equal(h.children.length,0);
 return {scenario:'stale-3dmol-load-disposes-new-canvas',beforeStaleCompletion:{status:'ready',canvases:['current-canvas']},afterStaleCompletion:{status:c.status.state,snapshot:c.snapshot(),canvases:h.children.length},hypothesisReproduced:true};
}
function stubSession(h,id,{disposeGate,resizeGate,resizeStarted}={}){h.appendChild(canvas(id));return {setLabels(){},select(){},resize(){resizeStarted?.resolve();return resizeGate?.promise;},snapshot(){return {candidate:'stub',fixture:id,selectedAtomIds:[],labelsVisible:false,width:500,height:300,dpr:1,status:'ready'};},dispose(){return disposeGate?.promise;}};}
async function afterDispose(){
 const h=host(),gate=deferred(),started=deferred(),mounts=[];const c=new ComparisonController(h,()=>{});
 const initial={id:'initial',label:'initial',supports:()=>true,mount:async()=>({...stubSession(h,'initial'),dispose(){started.resolve();return gate.promise;}})};
 await c.show(initial,fixture('initial'),true);
 const a={id:'a',label:'a',supports:()=>true,mount:async()=>{mounts.push('stale');return {...stubSession(h,'stale'),dispose(){h.children.at(-1)?.remove();}};}};
 const b={id:'b',label:'b',supports:()=>true,mount:async()=>{mounts.push('current');return stubSession(h,'current');}};
 const stale=c.show(a,fixture('stale'),true);await started.promise;await c.show(b,fixture('current'),true);assert.deepEqual(h.children.map(x=>x.id),['current']);
 gate.resolve();await stale;assert.deepEqual(mounts,['current','stale']);assert.equal(c.snapshot().fixture,'current');assert.equal(h.children.length,0);
 return {scenario:'stale-show-resumes-after-dispose-await',mountOrder:mounts,status:c.status.state,snapshot:c.snapshot(),canvases:h.children.length,hypothesisReproduced:true};
}
async function afterResize(){
 const h=host(),gate=deferred(),started=deferred(),updates=[];const c=new ComparisonController(h,s=>updates.push(s.message));
 const a={id:'a',label:'a',supports:()=>true,mount:async()=>stubSession(h,'old',{resizeGate:gate,resizeStarted:started})};
 const b={id:'b',label:'b',supports:()=>true,mount:async()=>stubSession(h,'current')};
 const old=c.show(a,fixture('old'),true);await started.promise;await c.show(b,fixture('current'),true);gate.resolve();await old;
 assert.match(c.status.message,/old is ready in a/);assert.equal(c.snapshot().fixture,'current');
 return {scenario:'stale-resize-publishes-ready',status:c.status,snapshot:c.snapshot(),canvases:h.children.length,hypothesisReproduced:true};
}
(async()=>{const results=[await staleLoader(),await afterDispose(),await afterResize()];console.log(JSON.stringify({source:head,sourcePins:pins,runtime:process.version,typescript:ts.version,kind:'Pure Node deterministic fake-host/loader reproduction using in-memory TypeScript transform of actual source; no renderer library, browser or GPU invoked.',results},null,2));})().catch(e=>{console.error(e.message);process.exitCode=1;});
