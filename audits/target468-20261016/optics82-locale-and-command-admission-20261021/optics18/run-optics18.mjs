// Prospective thin wrapper. No default execution, launcher or provider operations.
import assert from 'node:assert/strict';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {chromium} from '@playwright/test';
import {observeOptics18} from './optics-controls.mjs';
const sha = b => createHash('sha256').update(b).digest('hex');
export async function main(argv) {
 assert(argv.includes('--execute'), 'PREPARED ONLY: explicit parent-reviewed execution required');
 const remaining=argv.filter(a=>a!=='--execute');assert.equal(argv.filter(a=>a==='--execute').length,1);assert.equal(remaining.length,14);const args={};for(let i=0;i<remaining.length;i+=2){assert(!Object.hasOwn(args,remaining[i]),'duplicate argument');args[remaining[i]]=remaining[i+1];}
 assert.deepEqual(Object.keys(args).sort(), ['--binding','--binding-sha256','--closure','--closure-sha256','--out','--projection','--projection-sha256'].sort());
 const root = path.dirname(fileURLToPath(import.meta.url));
 async function pinned(file, digest) {assert.match(digest,/^[a-f0-9]{64}$/);const bytes=await readFile(file);assert.equal(sha(bytes),digest,'external pin differs');return JSON.parse(bytes);}
 const closure = await pinned(args['--closure'],args['--closure-sha256']);
 for (const [name,digest] of Object.entries(closure.files_sha256)) {assert(/^[A-Za-z0-9_.-]+$/.test(name));assert.equal(sha(await readFile(path.join(root,name))),digest,'frozen program/input '+name);}
 for (const name of ['run-optics18.mjs','optics-controls.mjs','optics_saved_predicates_v4.py','PROTOCOLS.json','EXPECTED-WORDS.json','OPTICAL-ORACLE.json','review_saved18.py','project_app_assets.py']) assert(Object.hasOwn(closure.files_sha256,name));
 const binding = await pinned(args['--binding'],args['--binding-sha256']);
 const projection = await pinned(args['--projection'],args['--projection-sha256']);
 assert.match(binding.expected_engine_source,/^[a-f0-9]{40}$/);
 assert.equal(binding.expected_engine_source,projection.source);assert.equal(binding.native_binary_sha256,projection.native_binary_sha256);assert.equal(binding.actual_producer_lock_sha256,projection.actual_producer_lock_sha256);
 assert.equal(binding.artifact_sha256,projection.artifact_sha256);assert.equal(binding.verified_release_sha256,projection.verified_release_sha256);
 const expectedApp=Object.fromEntries(Object.entries(projection.app_assets).map(([k,v])=>[k,v.sha256]));assert.deepEqual(binding.app_assets,expectedApp);
 const expectedEngine=Object.fromEntries(Object.entries(projection.engine_and_rest_assets).filter(([k])=>k!=='rest.html').map(([k,v])=>[k,v.sha256]));assert.deepEqual(binding.frontend_assets,expectedEngine);assert.equal(binding.client_sha256,projection.engine_and_rest_assets['rest.html'].sha256);
 const protocols=JSON.parse(await readFile(path.join(root,'PROTOCOLS.json'),'utf8'));
 await mkdir(args['--out'],{recursive:false});let browser,report,primaryError;
 const retainPartial=async value=>{await writeFile(path.join(args['--out'],'report.partial.json'),JSON.stringify(value,null,2)+'\n');};
 await mkdir(path.join(args['--out'],'screenshots'),{recursive:false});
 const retainScreenshot=async (name,bytes)=>{assert(/^screenshots\/[a-z0-9-]+-(en|de|fr)\.png$/.test(name));await writeFile(path.join(args['--out'],name),bytes,{flag:'wx'});};
 try {
  // Match accepted64: ordinary secure Chromium, no bypassCSP/ignoreHTTPS/websecurity flags.
  browser=await chromium.launch({headless:true});
  report=await observeOptics18(browser,binding,protocols,retainPartial,retainScreenshot);
  report.browser={version:browser.version(),web_security_disabled:false,no_owner_credentials_supplied:true};
  report.program_closure_sha256=args['--closure-sha256'];report.binding_sha256=args['--binding-sha256'];report.projection_sha256=args['--projection-sha256'];
  report.actual_observation_rows_complete=report.rows.length===18;
  report.independent_saved_predicates_executed=false;report.product_acceptance_claimed=false;
 } catch(error) {
  primaryError=error;
  // Actual partial record was persisted by observer; never manufacture substitute rows.
  if (!report) {
   try {report=JSON.parse(await readFile(path.join(args['--out'],'report.partial.json'),'utf8'));}
   catch(partialError) {
    // Failure metadata is not a substitute observation row; retain primary cause.
    report={state:'WRAPPER_FAILURE_METADATA_ONLY',original_wrapper_error:String(error),partial_report_error:String(partialError),product_acceptance_claimed:false};
   }
  }
  report.wrapper_error=String(error);report.product_acceptance_claimed=false;throw error;
 } finally {
  let cleanupError;
  try {if(report) await writeFile(path.join(args['--out'],'report.json'),JSON.stringify(report,null,2)+'\n',{flag:'wx'});} catch(error) {cleanupError=error;}
  try {if(browser) await browser.close();} catch(error) {cleanupError??=error;}
  if(cleanupError) {if(primaryError) console.error('Cleanup failed after preserved original observer error:',cleanupError);else throw cleanupError;}
 }
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url))await main(process.argv.slice(2));
