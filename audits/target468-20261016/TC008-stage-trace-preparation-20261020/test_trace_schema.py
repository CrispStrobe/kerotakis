"""Synthetic causal-schema probes ONLY. Never native/chemical success evidence."""
import copy,json
from review_trace import validate_call_graph

def enter(call,method,boundary):return {'kind':'adapter_enter','call':call,'method':method,'boundary':boundary,'command_index':7}
def leave(call,method,boundary,answer=None):
 r={'kind':'adapter_exit','call':call,'method':method,'boundary':boundary,'command_index':7,'result':'ok'}
 if method!='equilibrate':r['opaque_current_answer_present']=answer
 return r
rows=[enter(0,'equilibrate','phase_coupled'),enter(1,'equilibrate','displacement'),enter(2,'characterize_optional_current','aqueous'),leave(2,'characterize_optional_current','aqueous',False),leave(1,'equilibrate','displacement'),leave(0,'equilibrate','phase_coupled')]
assert [p[1].get('opaque_current_answer_present') for p in validate_call_graph(rows)]==[False,None,None]
strict=[enter(0,'characterize_current','phase_coupled'),enter(1,'characterize_current','displacement'),enter(2,'characterize_current','aqueous'),leave(2,'characterize_current','aqueous',True),leave(1,'characterize_current','displacement',True),leave(0,'characterize_current','phase_coupled',True)]
assert len(validate_call_graph(strict))==3
bad=[]
a=copy.deepcopy(rows);a[3]['method']='equilibrate';bad.append(('trusted exit relabeled ordinary solve',a))
a=copy.deepcopy(rows);del a[3];bad.append(('trusted terminal record omitted',a))
a=copy.deepcopy(rows);a[4],a[3]=a[3],a[4];bad.append(('trusted result placed after parent exit',a))
a=copy.deepcopy(strict);a[3]['opaque_current_answer_present']=False;bad.append(('strict adapter falsely accepts missing receipt',a))
a=copy.deepcopy(rows);a[4]['opaque_current_answer_present']=True;bad.append(('ordinary solve falsely claims opaque receipt',a))
a=copy.deepcopy(rows);a[3]['command_index']=8;bad.append(('trusted result assigned to another original command',a))
a=copy.deepcopy(rows);a[2]['call']=0;bad.append(('trusted call reused parent identity',a))
a=copy.deepcopy(rows);a[3]['boundary']='phase_coupled';bad.append(('trusted result assigned to another boundary',a))
for name,value in bad:
 try:validate_call_graph(value)
 except AssertionError:continue
 raise AssertionError(('adversarial trace accepted',name))
print(json.dumps({'kind':'SYNTHETIC_CAUSAL_SCHEMA_ONLY_NOT_CHEMISTRY','positive_controls':2,'rejected_adversarial_controls':[x[0] for x in bad],'engine_or_compiler_calls':0}))
