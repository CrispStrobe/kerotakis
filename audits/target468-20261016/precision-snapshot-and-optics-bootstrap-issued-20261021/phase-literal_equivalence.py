"""Source equivalence only; never claims an execution on the successor."""
def literal_equivalence(repo, old, new):
 import hashlib,re,struct,subprocess
 assert old=='ceff11dc9d9f86efdbe33bb33af4fe45807ddb22'
 assert isinstance(new,str)and re.fullmatch('[a-f0-9]{40}',new)and new!=old
 def git(*args):return subprocess.check_output(['git',*args],cwd=repo,timeout=55)
 def entries(ref):
  result={}
  for item in git('ls-tree','-r','--full-tree','-z',ref).split(b'\0'):
   if not item:continue
   header,name=item.split(b'\t',1);key=name.decode('utf-8');assert key not in result;result[key]=header.decode('ascii')
  return result
 a,b=entries(old),entries(new);assert a.keys()==b.keys(),'Added/deleted tracked paths outside one-literal successor'
 docs={'docs/STATUS.md','docs/improvement-agenda-20261014.md','docs/next-agent-handoff.md','docs/peroxide-medium-and-i18n-20261019.md','docs/post-hydrolysis-gap-lanes-20261019.md','docs/scientific-coverage-expansion-20261020.md','docs/unifac-input-admission-20261019.md','docs/wrap-up-and-open-lanes-20261019.md'}
 snapshot='crates/kerotakis-phreeqc/tests/colligative_numbers.rs'
 target='crates/kerotakis-phreeqc/src/aqueous.rs';changed=sorted(name for name in a if a[name]!=b[name]);assert target in changed and set(changed)<={target,snapshot}|docs and snapshot in changed,changed
 for name in changed:assert a[name].split()[:2]==b[name].split()[:2],'Tracked mode/type changed'
 before=git('show',old+':'+target);after=git('show',new+':'+target);previous=b'1.2345678912345678';replacement=b'1.234_567_891_234_568'
 assert before.count(previous)==1 and after==before.replace(previous,replacement,1),'Other source/test/assertion change'
 old_snapshot=git('show',old+':'+snapshot);new_snapshot=git('show',new+':'+snapshot)
 assert hashlib.sha256(old_snapshot).hexdigest()=='e1d855277d95e05abf4528517ef0c91d08b7fd9f071812887d8d0c4e22b58cad' and hashlib.sha256(new_snapshot).hexdigest()=='d7b4f5d9a7d201ebc271006b9f6f9d26f868fa7f6c2334937d9e0ea38f42e7ac','Snapshot patch bytes differ from reviewed successor'
 executable=lambda raw:[line for line in raw.splitlines()if line.strip()and not line.lstrip().startswith(b'//')]
 old_pin=b'        (freezing_c + 0.3516).abs() < 0.004,';new_pin=b'        (freezing_c + 0.3468).abs() < 0.0004,'
 assert old_snapshot.count(old_pin)==1 and new_snapshot.count(new_pin)==1
 assert [line.replace(old_pin,new_pin)for line in executable(old_snapshot)]==executable(new_snapshot),'Other executable assertion changed'
 bits=lambda raw:struct.pack('>d',float(raw.decode('ascii').replace('_',''))).hex()
 assert bits(previous)==bits(replacement)=='3ff3c0ca42d8b4db'
 return {'prior_focus_source':old,'new_full_source':new,'prior_tree':git('rev-parse',old+'^{tree}').decode().strip(),'new_tree':git('rev-parse',new+'^{tree}').decode().strip(),'all_tracked_entries_checked':len(a),'changed_paths':changed,'permitted_documentation_paths':sorted(docs),'literal_source':target,'before_source_sha256':hashlib.sha256(before).hexdigest(),'after_source_sha256':hashlib.sha256(after).hexdigest(),'old_literal':previous.decode(),'new_literal':replacement.decode(),'identical_binary64_hex':bits(previous),'snapshot_test':snapshot,'old_snapshot_sha256':hashlib.sha256(old_snapshot).hexdigest(),'new_snapshot_sha256':hashlib.sha256(new_snapshot).hexdigest(),'snapshot_only_executable_change':'implementation pin tightened tenfold; all physical assertions unchanged','prior_focus_reused_as_prior_only':True,'successor_focus_execution_claimed':False}
