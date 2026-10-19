// Fresh deployed partition and retained peroxide regressions; implementation references are not empirical calibration.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import path from 'node:path';
import {chromium} from '@playwright/test';
const spec=JSON.parse(await readFile(path.join(process.env.GITHUB_WORKSPACE,'audits/live-rest-browser-20261018/release-binding.json'),'utf8'));
const origin='https://kerotakis-web-payload.vercel.app';
const output=process.env.BROWSER_REPORT_DIR;await mkdir(output,{recursive:true});
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const report={source_binding:spec,owner_credentials_supplied:false,new_corpus_executions:0,checks:[],assets:{},raw:{}};
function check(name,condition,observed){report.checks.push({name,passed:Boolean(condition),observed});assert(condition,name);}
const boundary=result=>result.steps.flatMap(step=>step.events).some(event=>event.event==='not_yet_modeled'&&event.reason?.key==='not-modeled.peroxide-rich-medium');
const dry='add v1 H2O2 0.326375mol\nwait 1s\nadd v1 HClO 0.019062mol\nwait 1s\n';
const vessel=result=>result.bench.vessels[0];
const withdrawn=result=>vessel(result).solution==null&&vessel(result).free_proton===0&&vessel(result).free_hydroxide===0;
const amount=(result,id)=>vessel(result).contents.filter(p=>p.species===id).reduce((s,p)=>s+p.moles,0);
let browser;
try{
 for(const [name,digest] of Object.entries(spec.frontend_assets)){
  const response=await fetch(origin+'/'+name,{redirect:'error',signal:AbortSignal.timeout(15000)});const bytes=Buffer.from(await response.arrayBuffer());
  report.assets[name]={status:response.status,sha256:sha(bytes),bytes:bytes.length};assert(response.ok&&sha(bytes)===digest,'Actual bound asset '+name);
 }
 browser=await chromium.launch({headless:true});report.browser={version:browser.version(),web_security_disabled:false};
 const page=await browser.newPage();await page.goto(origin+'/rest.html',{waitUntil:'networkidle'});
 async function run(locale,script,continuation=''){
  return page.evaluate(async({locale,script,continuation})=>{
   const module=await import('/kerotakis_wasm.js');await module.default();const {PhreeqcPool}=await import('/kerotakis.mjs');const factory=(await import('/iphreeqc.mjs')).default;
   const lab=new module.Lab();try{
    lab.setLocale(locale);const meta=JSON.parse(lab.meta());const pool=await PhreeqcPool.create(factory,async file=>{const response=await fetch('/db/'+file,{redirect:'error'});if(!response.ok)throw Error('Database '+response.status);return response.text();},tag=>lab.aqueousDatabase(tag));
    let calls=0;lab.setSolver((tag,input)=>{calls++;return pool.solve(tag,input);});
    const before=JSON.parse(lab.runScript(script));const before_calls=calls;const after=continuation?JSON.parse(lab.runScript(continuation)):null;
    return {meta,before,before_calls,after,after_calls:calls};
   }finally{lab.free();}
  },{locale,script,continuation});
 }
 function bound(raw){assert(typeof raw.meta.git_rev==='string'&&raw.meta.git_rev.length>=7&&spec.expected_engine_source.startsWith(raw.meta.git_rev),'Actual Wasm revision');}
 let raw=await run('en',dry);bound(raw);report.raw.dry=raw;
 check('dry peroxide and reaction-water trace refuse aqueous prediction',raw.before_calls===0&&boundary(raw.before)&&withdrawn(raw.before)&&amount(raw.before,'water')>0&&amount(raw.before,'H2O2')>0&&amount(raw.before,'HClO')>0,{calls:raw.before_calls,water_mol:amount(raw.before,'water'),solution:vessel(raw.before).solution});
 raw=await run('en','add v1 water 100mL\nadd v1 H2O2 0.001mol\nadd v1 HClO 0.0001mol\n');bound(raw);report.raw.aqueous=raw;
 check('water-rich positive control uses actual aqueous engine',raw.before_calls>0&&!boundary(raw.before)&&Number.isFinite(vessel(raw.before).solution?.ph),{calls:raw.before_calls,solution:vessel(raw.before).solution});
 raw=await run('en','add v1 water 10mL\nadd v1 NaCl 0.0001mol\n','add v1 H2O2 1mol\n');bound(raw);report.raw.transition=raw;
 check('crossing the medium boundary withdraws a real prior reading',raw.before_calls>0&&Number.isFinite(vessel(raw.before).solution?.ph)&&raw.after_calls===raw.before_calls&&boundary(raw.after)&&withdrawn(raw.after)&&amount(raw.after,'H2O2')>0,{before_calls:raw.before_calls,after_calls:raw.after_calls,before_solution:vessel(raw.before).solution,after_solution:vessel(raw.after).solution});
 for(const [locale,label,notice] of [['de','Peroxidzersetzung','Diese peroxidhaltige Flüssigkeit'],['fr','Décomposition du peroxyde','Ce liquide']]){
  raw=await run(locale,dry);bound(raw);report.raw[locale]=raw;const rendered=raw.before.steps.flatMap(s=>Array.isArray(s.rendered)?s.rendered:[s.rendered]).join('\n');
  check(locale+' actual medium notice and kinetic label are localized',raw.before_calls===0&&boundary(raw.before)&&withdrawn(raw.before)&&rendered.includes(label)&&rendered.includes(notice)&&!rendered.includes('peroxide-decomposition')&&!rendered.includes('this peroxide-containing liquid'),{rendered,calls:raw.before_calls});
 }
 raw=await run('en','add v1 water 2mol\nadd v1 methanol 0.01mol\nchromatograph v1\n');bound(raw);report.raw.methanol=raw;
 const peak=raw.before.steps.flatMap(step=>step.events).filter(e=>e.event==='chromatographed').flatMap(e=>e.peaks).find(p=>p.species==='methanol');
 const expectedMethanolK=0.10700346067019094;
 check('dedicated methanol subgroup reaches actual browser column',Math.abs(vessel(raw.before).temperature-298.15)<1e-9&&Number.isFinite(peak?.partition_k)&&Math.abs((peak.partition_k-expectedMethanolK)/expectedMethanolK)<1e-4,{actual_k:peak?.partition_k,expected_same_model_reference_k:expectedMethanolK,temperature:vessel(raw.before).temperature,qualification:'Independent implementation reference, not measured retention or empirical calibration.'});
 raw=await run('en','add v1 water 2mol\nadd v1 hexane 0.5mol\nadd v1 ethanol 0.01mol\n','drain v1 v2\n');bound(raw);report.raw.drain=raw;
 const shares=raw.after.bench.vessels.map(v=>v.contents.filter(p=>p.species==='ethanol').reduce((sum,p)=>sum+p.moles,0));
 const total=id=>raw.after.bench.vessels.flatMap(v=>v.contents).filter(p=>p.species===id).reduce((sum,p)=>sum+p.moles,0);
 check('checked layer calculation and drain preserve actual browser inventories',raw.before.bench.vessels.length===1&&raw.after.bench.vessels.length===2&&shares.every(n=>Number.isFinite(n)&&n>0&&n<0.01)&&Math.abs(total('ethanol')-0.01)<1e-12&&Math.abs(total('water')-2)<1e-10&&Math.abs(total('hexane')-0.5)<1e-10,{ethanol_shares:shares,total_water:total('water'),total_hexane:total('hexane'),total_ethanol:total('ethanol')});
 report.all_checks_passed=report.checks.length===7&&report.checks.every(c=>c.passed);assert(report.all_checks_passed);
}catch(error){report.error=String(error);report.all_checks_passed=false;throw error;}
finally{await writeFile(path.join(output,'report.json'),JSON.stringify(report,null,2)+'\n');if(browser)await browser.close();}
