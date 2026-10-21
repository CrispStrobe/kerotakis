"""Exact Rust test identities; only two source-qualified fixture aliases exist."""
import re
QUALIFIED_NAMES={
 'missing_forward_interaction_is_a_named_refusal':'unifac::tests::missing_forward_interaction_is_a_named_refusal',
 'reverse_interaction_does_not_stand_in_for_missing_direction':'unifac::tests::reverse_interaction_does_not_stand_in_for_missing_direction',
}
def native_test_rows(raw):
 assert type(raw)is bytes
 # Log prefixes can contain the GitHub timestamp/job/step; identity and result are exact row tokens.
 pattern=rb'(?:^|\n)[^\n]*?\btest ([A-Za-z_][A-Za-z_0-9:]*)' + rb'(?: - should panic)? \.\.\. (ok|FAILED|ignored)\r?(?=\n|$)'
 return [(name.decode('ascii'),outcome.decode('ascii'))for name,outcome in re.findall(pattern,raw)]
def require_exact_native_test(rows,declared):
 assert type(declared)is str and re.fullmatch('[A-Za-z_][A-Za-z_0-9]*(?:::[A-Za-z_][A-Za-z_0-9]*)*',declared),'Expected declared exact Rust test identity'
 canonical=QUALIFIED_NAMES.get(declared,declared)
 # Suffix comparison detects ambiguity only; it never grants a passing alias.
 same_leaf=[(name,outcome)for name,outcome in rows if name.rsplit('::',1)[-1]==declared.rsplit('::',1)[-1]]
 assert same_leaf==[(canonical,'ok')],(declared,canonical,same_leaf)
 return {'declared_fixture':declared,'canonical_test':canonical,'actual_outcome':'ok','matching_rows':1}
