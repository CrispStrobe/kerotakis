import { afterEach, describe, expect, it } from "vitest";
import { render } from "svelte/server";
import type { SceneVessel } from "../host/EngineHost";
import { i18n } from "../i18n.svelte";
import Vessel from "./Vessel.svelte";
const base: SceneVessel = {id:2,label:"beaker",liquid:{volume_l:0.1,srgb:[120,150,210],colour_word:"blue",cloudiness:0,path_length_cm:4},solids:[],bubbling:false,boundary:"open",temperature_k:298.15,pressure_pa:101325,elapsed_s:0,mass_g:100,words:"a beaker of blue solution",badges:[]};
function draw(sample: SceneVessel): string {const before=JSON.stringify(sample);const html=render(Vessel,{props:{vessel:sample,register:"lv3",effects:[]}}).body;expect(JSON.stringify(sample)).toBe(before);return html;}
function shown(html:string,kind:string):string {const readout=html.match(new RegExp(`<span[^>]*class="[^"]*${kind}[^"]*"[^>]*>[\\s\\S]*?<\\/span>`))?.[0]??"";return (readout.match(/<strong\b[^>]*>([\s\S]*?)<\/strong>/)?.[1]??"").replace(/<!--[\s\S]*?-->/g,"").trim();}
afterEach(()=>{i18n.locale="en";});
describe("engine-authoritative optical species labels",()=>{
  it.each([
    ["en","methyl_orange","methyl orange","methyl orange"],
    ["de","methyl_orange","methyl orange","Methylorange"],
    ["fr","methyl_orange","methyl orange","hélianthine"],
    ["en","bromothymol_blue","bromothymol blue","bromothymol blue"],
    ["de","bromothymol_blue","bromothymol blue","Bromthymolblau"],
    ["fr","bromothymol_blue","bromothymol blue","bleu de bromothymol"],
  ])("shows authoritative pH labels for %s / %s",(locale,id,english,expected)=>{
    i18n.locale=locale;const sample={...base,liquid:{...base.liquid!,optical_ph_gaps:[id],optical_species_names:{[id]:english}}};const html=draw(sample);
    expect(shown(html,"optical-ph-gap-readout")).toBe(expected);expect(html).toContain(`data-ph-gaps="${id}"`);expect(html).toContain(expected);expect(html).not.toContain(`>${id}</strong>`);
  });
  it.each([["en","copper sulfate"],["de","Kupfersulfat"],["fr","sulfate de cuivre"]])("shows authoritative missing-spectrum labels in %s",(locale,expected)=>{
    i18n.locale=locale;const sample={...base,liquid:{...base.liquid!,spectral_gaps:["CuSO4"],optical_species_names:{CuSO4:"copper sulfate"}}};const html=draw(sample);expect(shown(html,"spectral-gap-readout")).toBe(expected);expect(html).toContain('data-gaps="CuSO4"');
  });
  it.each([["en","methyl orange"],["de","Methylorange"],["fr","hélianthine"]])("preserves direct English legacy labels in %s",(locale,expected)=>{
    i18n.locale=locale;const sample={...base,liquid:{...base.liquid!,optical_ph_gaps:["methyl orange"]}};expect(shown(draw(sample),"optical-ph-gap-readout")).toBe(expected);
  });
  it("keeps unknown native-complex labels and canonical attributes",()=>{i18n.locale="de";const sample={...base,liquid:{...base.liquid!,spectral_gaps:["Cu(NH3)4+2"],optical_species_names:{}}};const html=draw(sample);expect(shown(html,"spectral-gap-readout")).toBe("Cu(NH3)4+2");expect(html).toContain('data-gaps="Cu(NH3)4+2"');});
  it("preserves older canonical payload fallback without guessing underscores",()=>{i18n.locale="de";const sample={...base,liquid:{...base.liquid!,optical_ph_gaps:["unknown_indicator_key"]}};expect(shown(draw(sample),"optical-ph-gap-readout")).toBe("unknown_indicator_key");});
  it("leaves a complete blank without coverage labels",()=>{const html=draw(base);expect(html).not.toContain("optical-ph-gap-readout");expect(html).not.toContain("spectral-gap-readout");expect(html).not.toContain('class="spectral-gap-hatch');});
  it("shares one hatch while translating both distinct gap causes",()=>{i18n.locale="de";const sample={...base,liquid:{...base.liquid!,optical_ph_gaps:["methyl_orange"],spectral_gaps:["CuSO4"],optical_species_names:{methyl_orange:"methyl orange",CuSO4:"copper sulfate"}}};const html=draw(sample);expect(shown(html,"optical-ph-gap-readout")).toBe("Methylorange");expect(shown(html,"spectral-gap-readout")).toBe("Kupfersulfat");expect(html.match(/class="spectral-gap-hatch/g)).toHaveLength(1);});
});
