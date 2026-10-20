#!/usr/bin/env python3
"""Additional solved-species, phase, export and absent-reservoir evidence; saved bytes only."""
import hashlib,json,re,subprocess,zipfile
from decimal import Decimal,localcontext
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];BASE=ROOT/'audits/new-experiments-201-20261016/native-screen-results';D=Decimal;Z=D(0);sha=lambda b:hashlib.sha256(b).hexdigest();load=lambda p:json.loads(p.read_bytes(),parse_float=D)
def amount(v,sp,phase=None):return sum((p['moles'] for p in v['contents'] if p['species']==sp and (phase is None or p['phase']==phase)),Z)
def snap(r):return {v['id']:v for v in r['bench']['vessels']}
def main():
 original=load(HERE/'assessment.json');source=original['cases'][0]['source']['source_head'];vessel_raw=subprocess.check_output(['git','show',source+':crates/kerotakis-core/src/vessel.rs'],cwd=ROOT);vessel=vessel_raw.decode();reg_raw=subprocess.check_output(['git','show',source+':data/registry/registry-source-v1.json'],cwd=ROOT);reg=json.loads(reg_raw,parse_float=D);comps={r['species_id']:r for r in reg['compositions']}
 optional_vectors=['unresolved_materials','material_objects','electrodes','exchanges','adsorbed','solid_solutions','surface_colours','charred_materials'];optional_options=['soap_scum','lemon_paper_mark','surface_particles','emulsion']
 for field in optional_vectors:assert re.search(r'#\[serde\(default, skip_serializing_if = "Vec::is_empty"\)\]\s+pub '+field+r':',vessel),field
 for field in optional_options:assert re.search(r'#\[serde\(default, skip_serializing_if = "Option::is_none"\)\]\s+pub '+field+r':',vessel),field
 def atoms(v,element,phase=None):
  total=Z
  for p in v['contents']:
   if phase is not None and p['phase']!=phase:continue
   if p['species']=='base_equivalents':continue
   coeff=sum((e['count']['value'] for e in comps[p['species']]['elements'] if e['element']==element),Z);total+=coeff*p['moles']
  return total
 ar=BASE/'raw-artifact.zip';assert hashlib.file_digest(ar.open('rb'),'sha256').hexdigest()==original['raw_archive_sha256'];rows_by_id={};digests={};reservoirs=[];keys=set()
 with localcontext() as ctx,zipfile.ZipFile(ar) as z:
  ctx.prec=45
  for c in original['cases']:
   stem=c['id'].replace(':','_');raw=z.read(stem+'/stdout.jsonl');assert sha(raw)==c['stdout_sha256'];script=z.read(stem+'/experiment.lab');assert sha(script)==c['script_sha256'];rows=[json.loads(x,parse_float=D) for x in raw.splitlines() if x.strip()];rows_by_id[c['id']]=rows;digests[c['id']]={'stdout_sha256':sha(raw),'script_sha256':sha(script)};count=0
   for i,row in enumerate(rows):
    assert set(row['bench'])=={'vessels'},(c['id'],i,'extra bench reservoir')
    for v in row['bench']['vessels']:
     keys.update(v);assert v.get('surfaces')==[]
     for f in optional_vectors:assert not v.get(f), (c['id'],i,f)
     for f in optional_options:assert v.get(f) is None,(c['id'],i,f)
     assert v['nuclides']=={'inventory':{}};assert all(x==0 for x in v['foam'].values());assert v['pending_co2_transfer_mol']==0;assert v['resolved']=={'solution':None,'valid':False};count+=1
   reservoirs.append({'id':c['id'],'checked_vessel_snapshots':count,'surfaces_empty':True,'optional_owned_vectors_empty_by_authenticated_serde_omission':optional_vectors,'optional_owned_objects_none_by_authenticated_serde_omission':optional_options,'nuclides_empty':True,'foam_all_zero':True,'pending_CO2_zero':True,'resolved_mirror_empty':True,'bench_other_material_reservoirs_absent':True})
  known={'co2_partial_pressure_atm','contents','elapsed_seconds','excess_enthalpy_j','foam','free_hydroxide','free_proton','headspace','id','label','lots','nuclides','pending_co2_transfer_mol','pressure','resolved','solute_charge','solution','surfaces','temperature','thermal_mode'};assert keys==known
  phos=[];rows=rows_by_id['batch2:ab-034'];final=snap(rows[-1])
  for vid,v in final.items():
   solved=[]
   for x in v['solution']['species']:
    if 'PO4' not in x['name']:continue
    solved.append({'name':x['name'],'molality_mol_kgw':x['molality'],'reported_activity':x['activity'],'solved_mol_from_reported_molality':x['molality']*v['solution']['solvent_kg']})
   h2=next(x for x in solved if x['name']=='H2PO4-');hpo=next(x for x in solved if x['name']=='HPO4-2');po=next((x for x in solved if x['name']=='PO4-3'),None)
   if vid==0:assert h2['solved_mol_from_reported_molality']>D('.0009')
   if vid==1:assert hpo['solved_mol_from_reported_molality']>D('.0008')
   phos.append({'vessel':vid,'actual_pH':v['solution']['ph'],'actual_T_K':v['temperature'],'actual_solvent_kg':v['solution']['solvent_kg'],'canonical_total_P_mol':atoms(v,'P'),'solved_species':solved,'canonical_and_solved_views_must_not_be_added':True})
  pofree=[next((x['solved_mol_from_reported_molality'] for x in v['solved_species'] if x['name']=='PO4-3'),Z) for v in phos];assert pofree[2]>100*pofree[1]
  rows=rows_by_id['batch2:pq-006'];fractions=[]
  for vid,v in snap(rows[-1]).items():
   solids=[p for p in v['contents'] if p['phase']=='solid'];assert {p['species'] for p in solids}<={'Mg(OH)2','CaCO3'}
   for p in v['contents']:
    if p['phase']=='solid':continue
    if any(e['element'] in ('Ca','Mg') for e in comps.get(p['species'],{}).get('elements',[])):assert p['species'] in ('Ca+2','Mg+2')
   fractions.append({'vessel':vid,'exact_solid_portions':solids,'Ca_aqueous_mol':atoms(v,'Ca','aqueous'),'Mg_aqueous_mol':atoms(v,'Mg','aqueous'),'total_Ca_mol':atoms(v,'Ca'),'total_Mg_mol':atoms(v,'Mg')})
  assert {p['species'] for p in fractions[0]['exact_solid_portions']}=={'Mg(OH)2'};assert {p['species'] for p in fractions[1]['exact_solid_portions']}=={'Mg(OH)2','CaCO3'}
  rows=rows_by_id['batch2:qa-006'];exports=[{'row':i,**e} for i,r in enumerate(rows) for e in r['events'] if e['event'] in ('gas_evolved','gas_absorbed')];assert len(exports)==100 and {e['event'] for e in exports}=={'gas_evolved'} and {e['species'] for e in exports}=={'CO2'};exported=sum(e['moles'] for e in exports);assert abs(exported-D('.002'))<D('1e-12')
  rows=rows_by_id['batch2:ec-007'];i,row=next((i,r) for i,r in enumerate(rows) if r['operator']['op']=='add' and r['operator']['species']=='Zn');before=snap(rows[i-1])[0];after=snap(row)[0];ag=amount(after,'Ag','solid')-amount(before,'Ag','solid');solidloss=amount(before,'AgCl','solid')-amount(after,'AgCl','solid');aqdraw=amount(before,'Ag+')-amount(after,'Ag+');znloss=D('.0003')-amount(after,'Zn','solid');assert solidloss>0 and aqdraw>0 and abs(solidloss+aqdraw-ag)<D('1e-12') and abs(ag-2*znloss)<D('1e-12');assert amount(after,'Ag+')<D('1e-6') and amount(after,'AgCl','solid')>D('.0003') and amount(after,'Zn','solid')>D('.0002')
  ec={'row':i,'AgCl_solid_before_mol':amount(before,'AgCl','solid'),'AgCl_solid_after_mol':amount(after,'AgCl','solid'),'AgCl_solid_loss_mol':solidloss,'Ag_aqueous_before_mol':amount(before,'Ag+'),'Ag_aqueous_after_mol':amount(after,'Ag+'),'Ag_aqueous_draw_mol':aqdraw,'formed_Ag_mol':ag,'solid_loss_plus_aqueous_draw_minus_metal_mol':solidloss+aqdraw-ag,'Zn_oxidation_mol':znloss,'remaining_solid_Zn_mol':amount(after,'Zn','solid'),'observable_gate_mol':D('1e-6'),'partial_response_only':True,'equilibrium_completion_not_established':True,'coupling_proof':'NetAgClsolidlossanddissolvedAgdrawaccountforallnewAgmetal, with2AgperZn. AppreciablesolidAgClandZnremainwhilefreeAgisbelowthehistoricaldisplay-scalegate; nofullthermodynamicequilibriumclaim.'}
 out={'schema':'kerotakis.batch2.fourth-parent-questions-addendum.v1','source':source,'vessel_source_sha256':sha(vessel_raw),'registry_sha256':sha(reg_raw),'original_assessment_sha256':sha((HERE/'assessment.json').read_bytes()),'raw_archive_sha256':original['raw_archive_sha256'],'raw_cases':digests,'all12_noncontents_reservoir_reviews':reservoirs,'serialized_vessel_keys':sorted(keys),'AB034_solved_phosphate':phos,'PQ006_exact_fraction_identities':fractions,'QA006_all_boundary_exports':{'events':exports,'only_CO2_exported':True,'total_CO2_mol':exported},'EC007_partial_coupling_and_threshold':ec,'ownership_convention':'Contentsarelivecanonicalmaterial.Solvedsolution/activitiesandfreeH/OHdiagnosticsaremirroredresolvedproperties,notadditionalmolecules.MaterialLotsareadditionprovenance/metadata,notseparatelyownedamounts.Headspacegaslivesincontents.Nonuclide/surface/electrode/exchange/adsorbed/solid-solution/material-object/emulsion/foamreservoirisactiveinthese12cases.','new_engine_or_API_requests':0,'catalog_selection_assigned':False};print(json.dumps(out,indent=2,default=str))
if __name__=='__main__':main()
