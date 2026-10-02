"""Repeat the baseline's five documented input adaptations with the final CLI."""
import hashlib, json, subprocess, time
from pathlib import Path
root=Path(__file__).resolve().parent
repo=root.parents[1]
binary=repo/'target/debug/kero'
results=root/'after'
records=[]
inputs=[('07','07-name-corrected'),('15','15-name-corrected'),('45','45-oil-substitute'),('46','46-oil-substitute'),('47','47-oil-substitute'),('co2-consistency','co2-consistency'),('heat-disclosure','heat-disclosure')]
for ident, name in inputs:
    script=root/'followups'/f'{name}.lab'
    for mode in ['json','text']:
        args=[str(binary),'run',str(script)] + (['--json'] if mode=='json' else [])
        start=time.perf_counter()
        p=subprocess.run(args,cwd=repo,capture_output=True,text=True,timeout=90)
        prefix=ident+'.corrected' if ident.isdigit() else ident
        (results/f'{prefix}.{mode}.stdout').write_text(p.stdout)
        (results/f'{prefix}.{mode}.stderr').write_text(p.stderr)
        records.append(dict(id=ident,script=name,script_sha256=hashlib.sha256(script.read_bytes()).hexdigest(),mode=mode,exit_code=p.returncode,seconds=time.perf_counter()-start))
        if p.returncode: raise RuntimeError(f'{name} {mode}: {p.stderr}')
metadata=dict(binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),runs=records)
(results/'followups-execution.json').write_text(json.dumps(metadata,indent=2)+'\n')
print('Followups:',len(records),'CLI processes, all successful')
