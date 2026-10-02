import subprocess,pathlib,os
base=pathlib.Path('/mnt/storage/kerotakis-maintenance-20261002');repo=pathlib.Path('/mnt/volume1/kerotakis');root=repo/'audits/independent-50-20261002/phase-ownership'
def run(stage, log, args):
 subprocess.run(['python3',str(base/'component-guarded-command.py'),stage,log]+args,cwd=repo,check=True)
core=['transport','transport_verb','prepared_kitchen_objects','material_object_state','kitchen_biology','transfer_trace','interface_conservation','solver_transactions','conservation','phase_heat_capacity','pressure_boiling','states','mix','selected_phase_transfer','phase_routes','distil','atomic_distil_refusal','layers','liquid_extraction','vegetable_oil_layers']
args=['cargo','test','-p','kerotakis-core','--lib']
for name in core:args+=['--test',name]
run('phase-ownership-core','phase-ownership-core.log',args+['-j1','--','--test-threads=1'])
run('phase-ownership-phreeqc','phase-ownership-phreeqc.log',['cargo','test','-p','kerotakis-phreeqc','--test','exchange_transport','--test','surface_transport','-j1','--','--test-threads=1'])
run('phase-ownership-cli','phase-ownership-cli.log',['cargo','test','-p','kerotakis-cli','--bin','kero','--test','thermal_contracts','--test','provenance','--test','observable_coverage','--test','headspace_json','--test','json_contract','-j1','--','--test-threads=1'])
run('phase-ownership-wasm','phase-ownership-wasm.log',['cargo','check','-p','kerotakis-wasm','--target','wasm32-unknown-unknown','-j1'])
env=dict(os.environ,KERO_WORKERS='1',KERO_BIN=str(repo/'target/debug/kero'),KERO_RESULTS_DIR=str(root/'replay'))
with (base/'phase-ownership-replay.log').open('w') as out:
 subprocess.run(['nice','-n','5','python3',str(base/'component-replay.py')],cwd=repo,env=env,stdout=out,stderr=subprocess.STDOUT,check=True)
with (base/'phase-ownership-assessment.log').open('w') as out:
 subprocess.run(['python3',str(repo/'audits/independent-50-20261002/verify_extended.py')],cwd=repo,env=env,stdout=out,stderr=subprocess.STDOUT,check=True)
print('Phase ownership validation complete',flush=True)
