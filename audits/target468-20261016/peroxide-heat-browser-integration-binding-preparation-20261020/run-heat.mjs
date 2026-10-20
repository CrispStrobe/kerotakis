// Preparation only: do not run until a separately reviewed actual integrated release binding exists.
import assert from 'node:assert/strict';
import {readFile,writeFile,mkdir}from'node:fs/promises';
import {createHash}from'node:crypto';
import {chromium}from'@playwright/test';
import {runHeatChecks}from'./heat-controls.mjs';
const binding=JSON.parse(await readFile(process.env.HEAT_RELEASE_BINDING,'utf8'));assert.equal(binding.reviewed_candidate_source,'18f2bebd6f9aafe85d3ba1d76e2c2f9cd5d11e9a');assert.match(binding.expected_engine_source,/^[a-f0-9]{40}$/);
const origin='https://kerotakis-web-payload.vercel.app';const output=process.env.BROWSER_REPORT_DIR;await mkdir(output,{recursive:true});let browser,report,client;
try{
 const response=await fetch(origin+'/rest.html',{redirect:'error',cache:'no-store',signal:AbortSignal.timeout(15000)});const bytes=Buffer.from(await response.arrayBuffer());client={status:response.status,sha256:createHash('sha256').update(bytes).digest('hex'),bytes:bytes.length};assert(response.ok&&client.sha256===binding.client_sha256);
 browser=await chromium.launch({headless:true});const page=await browser.newPage();await page.goto(origin+'/rest.html',{waitUntil:'networkidle'});report=await runHeatChecks(page,binding);report.browser={version:browser.version(),web_security_disabled:false};
}catch(error){report=error.heatReport??{binding,checks:[],all_checks_passed:false,error:String(error)};throw error;}
finally{if(client&&report)report.client=client;await writeFile(output+'/report.json',JSON.stringify(report,null,2)+'\n');if(browser)await browser.close();}
