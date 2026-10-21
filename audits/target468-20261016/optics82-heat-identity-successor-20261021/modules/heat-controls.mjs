// Prepared only. Importing launches no browser and makes no request.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
export const protocol='register lv3\nadd v1 water 10mol\nadd v1 H2O2 0.1mol\nadd v1 KI 0.001mol\n';
export const continuation='wait 1s\n';
const REVIEWED_CANDIDATE_SOURCE='62f647275fd800d4992f5516651dfc9e61f47a05';
const amount=(v,id)=>v.contents.filter(p=>p.species===id).reduce((s,p)=>s+p.moles,0);
export async function runHeatChecks(page,binding){
 assert.equal(binding.reviewed_candidate_source,REVIEWED_CANDIDATE_SOURCE);assert.match(binding.expected_engine_source,/^[a-f0-9]{40}$/);const source=binding.expected_engine_source;assert.equal(new URL(page.url()).protocol,'https:');
 const origin=new URL(page.url()).origin;const report={status:'ACTUAL_BROWSER_RESULT_PENDING',binding,origin,protocol,continuation,raw:{},assets:{},checks:[],scientific_selections:0,no_owner_credentials_supplied:true};
 try{
  const required=['iphreeqc.mjs','iphreeqc.wasm','kerotakis.mjs','kerotakis_wasm.js','kerotakis_wasm_bg.wasm'];assert.deepEqual(Object.keys(binding.frontend_assets).sort(),required.sort());
  for(const [name,digest]of Object.entries(binding.frontend_assets)){
   const response=await fetch(new URL('/'+name,origin),{redirect:'error',cache:'no-store',signal:AbortSignal.timeout(15000)});const bytes=Buffer.from(await response.arrayBuffer());const actual=createHash('sha256').update(bytes).digest('hex');report.assets[name]={status:response.status,sha256:actual,bytes:bytes.length};assert(response.ok&&actual===digest,name);
  }
  for(const locale of ['de','fr']){
   const raw=await page.evaluate(async({locale,protocol,continuation})=>{
    const wasm=await import('/kerotakis_wasm.js');await wasm.default();const {PhreeqcPool}=await import('/kerotakis.mjs');const factory=(await import('/iphreeqc.mjs')).default;const lab=new wasm.Lab();
    try{
     lab.setLocale(locale);const meta=JSON.parse(lab.meta());const pool=await PhreeqcPool.create(factory,async file=>{const r=await fetch('/db/'+file,{redirect:'error'});if(!r.ok)throw Error('database status '+r.status);return r.text();},tag=>lab.aqueousDatabase(tag));let calls=0;lab.setSolver((tag,input)=>{calls++;return pool.solve(tag,input);});
     const preparation=JSON.parse(lab.runScript(protocol));const before=JSON.parse(lab.snapshot());const before_calls=calls;const result=JSON.parse(lab.runScript(continuation));const after=JSON.parse(lab.snapshot());return{meta,preparation,before,before_calls,result,after,after_calls:calls};
    }finally{lab.free();}
   },{locale,protocol,continuation});report.raw[locale]=raw;
   assert(typeof raw.meta.git_rev==='string'&&raw.meta.git_rev.length>=7&&source.startsWith(raw.meta.git_rev));
   assert.equal(raw.before.vessels.length,1);assert.equal(raw.after.vessels.length,1);const a=raw.before.vessels[0],b=raw.after.vessels[0];assert.equal(a.id,0);assert.equal(b.id,0);assert(Math.abs(b.elapsed_seconds-a.elapsed_seconds-1)<1e-12);
   const consumed=amount(a,'H2O2')-amount(b,'H2O2');assert(consumed>1e-8&&consumed<amount(a,'H2O2'));
   const events=raw.result.steps.flatMap(s=>s.events);const heat=events.filter(e=>e.event==='reaction_heat_released');assert(heat.length>0&&heat.every(e=>e.vessel===0&&e.reaction==='peroxide-decomposition'&&typeof e.energy_j==='number'&&Number.isFinite(e.energy_j)&&e.energy_j>0));const q=heat.reduce((s,e)=>s+e.energy_j,0);assert(Math.abs(q/consumed-98480)<=790);
   const rendered=raw.result.steps.flatMap(s=>s.rendered??[]).join('\n');const words=locale==='de'?['Peroxidzersetzung','setzte','frei']:['Décomposition du peroxyde','a libéré'];assert(words.every(x=>rendered.includes(x)));assert(!rendered.includes('peroxide-decomposition')&&!rendered.includes(' released '));
   report.checks.push({name:locale+' actual peroxide heat per consumed molecule and localized joules',passed:true,observed:{consumed_mol:consumed,heat_j:q,j_per_consumed_mol:q/consumed}});
  }
  assert.equal(report.checks.length,2);report.all_checks_passed=true;return report;
 }catch(error){report.all_checks_passed=false;report.error=String(error);error.heatReport=report;throw error;}
}
