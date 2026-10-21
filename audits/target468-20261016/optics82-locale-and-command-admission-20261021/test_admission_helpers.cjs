// SYNTHETIC WIRE METADATA ONLY: no browser/engine imports or operations.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict'),crypto=require('node:crypto');
const source=fs.readFileSync(path.join(__dirname,'optics18/optics-controls.mjs'),'utf8');
const start=source.indexOf('const POST_SCENE_READ_ONLY = new Set('),end=source.indexOf('async function readDom(page)',start);assert(start>=0&&end>start);
const helperSource=source.slice(start,end);
const funcs=new vm.Script(helperSource+'\n({assertProtocolCommandAdmission,assertPostSceneCommands});',{filename:'ACTUAL_PRODUCER_SOURCE_SLICE'}).runInNewContext({assert});
function fixture(injection) {
 const entries=[{sequence:1,worker:0,direction:'created',url:'SYNTHETIC_METADATA_ONLY'}];
 for(const [id,cmd]of[[10,'set_register'],[11,'run_script'],[12,'run_script']])entries.push({worker:0,direction:'request',origin:'app',value:{id,cmd}},{worker:0,direction:'response',value:{id,type:'done'}});
 entries.push({worker:0,direction:'request',origin:'observer-read-only-scene',value:{id:-1,cmd:'scene'}},{worker:0,direction:'response',value:{id:-1,type:'done'}});
 if(injection){const id=injection.id??100,worker=injection.worker??0;const inserted=[{worker,direction:'request',origin:injection.origin??'app',value:{id,cmd:injection.cmd}}];if(injection.complete!==false)inserted.push({worker,direction:'response',value:{id,type:'done'}});entries.splice(injection.late?entries.length:3,0,...inserted);}
 entries.forEach((e,i)=>e.sequence=i+1);
 const protocol=[10,11,12].map(id=>{const q=entries.find(e=>e.worker===0&&e.direction==='request'&&e.value.id===id),r=entries.find(e=>e.worker===0&&e.direction==='response'&&e.value.id===id);return{worker:0,origin:'app',request_sequence:q.sequence,response_sequence:r.sequence,request:q.value,response:r.value};});
 const r=entries.find(e=>e.direction==='response'&&e.value.id===-1);return{entries,protocol,scene:{worker:0,request:{id:-1,cmd:'scene'},response_sequence:r.sequence}};
}
const outcomes=[];function test(name,fn){fn();outcomes.push({name,passed:true});}
const full=f=>funcs.assertProtocolCommandAdmission(f.entries,f.scene,f.protocol),late=f=>funcs.assertPostSceneCommands(f.entries,f.scene);
test('exact prescribed protocol allowed',()=>{const f=fixture();full(f);late(f);});
for(const cmd of['load_pack','quest_start','quest_stop','init','set_announce_routing','future_unknown_command'])test('in-interval '+cmd+' rejected',()=>assert.throws(()=>full(fixture({cmd})),/unprescribed mutation or unknown protected command/));
for(const cmd of['step','reset','restore','set_locale'])test('completed late '+cmd+' rejected',()=>{const f=fixture({cmd,late:true});assert.throws(()=>full(f),/unprescribed mutation or unknown protected command/);assert.throws(()=>late(f),/late mutating or unknown app command/);});
test('pending late step rejected',()=>assert.throws(()=>late(fixture({cmd:'step',late:true,complete:false})),/late mutating or unknown app command/));
test('wrong-worker mutator impostor rejected',()=>assert.throws(()=>full(fixture({cmd:'run_script',id:11,worker:1})),/unprescribed mutation or unknown protected command/));
test('wrong-ID mutator impostor rejected',()=>assert.throws(()=>full(fixture({cmd:'run_script',id:100})),/unprescribed mutation or unknown protected command/));
test('forged prescribed pair owner rejected',()=>{const f=fixture();f.protocol[1].worker=1;assert.throws(()=>full(f),/prescribed protocol owner\/type differs/);});
test('wrong final scene owner rejected',()=>{const f=fixture();f.scene.worker=1;assert.throws(()=>late(f),/owned final scene terminal missing/);});
test('ordinary in-interval read-only allowed',()=>full(fixture({cmd:'grammar'})));
test('ordinary trailing read-only allowed',()=>{const f=fixture({cmd:'grammar',late:true});full(f);late(f);});
for(const cmd of['scene','snapshot'])test('negative observer '+cmd+' allowed',()=>{const f=fixture({cmd,origin:'observer-read-only-'+cmd,id:-2,late:true});full(f);late(f);});
test('unknown trailing origin rejected',()=>assert.throws(()=>late(fixture({cmd:'scene',origin:'unowned',late:true})),/unknown trailing request owner/));
console.log(JSON.stringify({scope:'SYNTHETIC_WIRE_METADATA_ACTUAL_JS_HELPERS_NO_BROWSER_OR_CHEMISTRY',actual_helper_slice_sha256:crypto.createHash('sha256').update(helperSource).digest('hex'),passed:outcomes.length,failed:0,outcomes},null,2));
