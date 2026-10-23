"""Small saved-log checks only. No native execution or archive bodies read."""
from pathlib import Path
import argparse,json,re
from libtest_rows import parse_unfiltered,InvalidReport
p=argparse.ArgumentParser();p.add_argument('--evidence',type=Path,required=True);args=p.parse_args()
completed=json.loads((args.evidence/'COMPLETED.json').read_text());total=0;failures=[];panic_rows=[]
for row in completed['actual_outcomes']:
 key=row['key'];text=(args.evidence/(key+'-tests.stdout')).read_text()
 names=re.findall(r'^(\S+): test$',(args.evidence/(key+'-list.stdout')).read_text(),re.M)
 actual=parse_unfiltered(text,names);assert actual['ignored']==0;total+=len(actual['cases'])
 failures.extend((key,n) for n,status in actual['cases'] if status=='FAILED')
 if ' - should panic ... ' in text:panic_rows.append((key,text,names))
expected_failures={
 ('kerotakis-core--scene_golden','five_release_lessons_match_numeric_scene_golden'),
 ('kerotakis-phreeqc--colligative_numbers','a_saturated_brine_boils_hotter_than_the_dilute_law_allows'),
 ('kerotakis-phreeqc--colligative_numbers','a_tenth_molal_brine_is_answered_by_a_model_and_not_by_raoult'),
}
assert total==3047 and set(failures)==expected_failures and len(failures)==3 and len(panic_rows)==2
# Mutate one REAL saved passing expected-panic result into a failed result,
# with coherent totals: the parser must preserve failure, not erase it.
key,text,names=panic_rows[0];name=re.search(r'^test (\S+) - should panic \.\.\. ok$',text,re.M)[1]
changed=text.replace('test '+name+' - should panic ... ok','test '+name+' - should panic ... FAILED',1)
matched=re.search(r'test result: ok\. (\d+) passed; 0 failed;',changed);assert matched
count=int(matched[1]);changed=changed.replace(matched[0],f'test result: FAILED. {count-1} passed; 1 failed;',1)
result=parse_unfiltered(changed,names);assert result['failed']==1 and not result['all_assertions_pass']
# Omitting or duplicating an actual result must fail accounting.
line='test '+name+' - should panic ... ok'
for malformed in [text.replace(line,'',1),text+'\n'+line+'\n']:
 try:parse_unfiltered(malformed,names)
 except InvalidReport:pass
 else:raise AssertionError('Malformed actual raw report was accepted')
print(json.dumps({'actual_discovered_cases':total,'actual_old_source_failures':failures,'expected_panic_targets':len(panic_rows),'failed_expected_panic_remains_failure':True,'missing_and_duplicate_actual_result_rejected':True,'no_native_execution_or_archive_body_reads':True}))
