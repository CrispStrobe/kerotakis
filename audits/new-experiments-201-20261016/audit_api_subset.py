#!/usr/bin/env python3
"""Run a pinned gateway's actual admission function offline; never send requests."""
import argparse
import ast
from collections import Counter
import datetime
import hashlib
import json
from pathlib import Path
import re
import subprocess

HERE = Path(__file__).resolve().parent
API_PATH = 'services/rest_api/app.py'


def digest(value):
    return hashlib.sha256(value).hexdigest()


class AdmissionRefusal(Exception):
    def __init__(self, status_code, detail):
        self.status_code, self.detail = status_code, detail
        super().__init__(detail)


def validator_at(commit):
    if not re.fullmatch('[0-9a-f]{40}', commit):
        raise ValueError('Use a full immutable Git commit')
    source = subprocess.check_output(['git', 'show', f'{commit}:{API_PATH}'])
    tree = ast.parse(source)
    nodes = []
    for node in tree.body:
        if isinstance(node, ast.Assign) and any(
            isinstance(t, ast.Name) and t.id in {'SCRIPT_LIMIT', 'VERBS'}
            for t in node.targets
        ):
            nodes.append(node)
        elif isinstance(node, ast.FunctionDef) and node.name == 'validate_script':
            nodes.append(node)
    if len(nodes) != 3:
        raise ValueError('Unexpected validator dependency layout; review source manually')
    scope = {'re': re, 'HTTPException': AdmissionRefusal}
    # Execute only the two constants and pure validator, never app/settings/store setup.
    exec(compile(ast.Module(body=nodes, type_ignores=[]), API_PATH, 'exec'), scope)
    return scope['validate_script'], scope['VERBS'], source


def outcome(validate, script):
    try:
        validate(script)
        return {'admitted': True}
    except AdmissionRefusal as exc:
        return {'admitted': False, 'status_code': exc.status_code, 'detail': exc.detail}


def adapt_register(script):
    """Remove only the exact presentation directive; retain every other byte/line."""
    return ''.join(line for line in script.splitlines(keepends=True)
                   if line.strip() != 'register lv3')


def adapt_eyes(script):
    """Canonicalize only exact parser-proven look [vN] aliases; do not drop observations."""
    result = []
    for line in script.splitlines(keepends=True):
        match = re.fullmatch(r'\s*look(?:\s+(v[1-9][0-9]*))?\s*', line)
        if match:
            ending = '\r\n' if line.endswith('\r\n') else '\n' if line.endswith('\n') else ''
            result.append(f"measure {match[1] or 'v1'} eyes{ending}")
        else:
            result.append(line)
    return ''.join(result)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', required=True)
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--canonical-eyes', action='store_true')
    args = parser.parse_args()
    validate, verbs, source = validator_at(args.source)
    controls = [
        ('admit_basic_addition', 'add v1 water 1mL\n', True),
        ('reject_session_directive', 'register lv3\n', False),
        ('reject_aggregate_duration', 'wait 3600s\nwait 1s\n', False),
        ('reject_excess_vessels', 'new\n' * 17, False),
        ('reject_control_characters', 'inspect\x00\n', False),
    ]
    control_results = []
    for name, script, expected in controls:
        result = outcome(validate, script)
        if result['admitted'] != expected:
            raise ValueError(f'Pinned admission control changed: {name}')
        control_results.append({'name': name, **result})
    rows = []
    blocked_counts = Counter()
    for record in json.loads((HERE / 'intake.json').read_text()):
        script = (HERE / record['original_script']).read_text()
        if digest(script.encode()) != record['script_sha256']:
            raise ValueError('Original script hash mismatch')
        adapted = adapt_register(script)
        blocked = sorted({line.split()[0] for line in adapted.splitlines()
                          if line.strip() and not line.lstrip().startswith('#')
                          and line.split()[0] not in verbs})
        blocked_counts.update(blocked)
        row = {'id': record['original_id'],
                     'original_script': record['original_script'],
                     'original_script_sha256': record['script_sha256'],
                     'original_admission': outcome(validate, script),
                     'register_only_adapter_sha256': digest(adapted.encode()),
                     'register_only_admission': outcome(validate, adapted),
                     'remaining_unsupported_verbs': blocked}
        if args.canonical_eyes:
            canonical = adapt_eyes(adapted)
            row['canonical_eyes_script'] = canonical
            row['canonical_eyes_sha256'] = digest(canonical.encode())
            row['canonical_eyes_admission'] = outcome(validate, canonical)
        rows.append(row)
    report = {'schema': 1,
              'recorded_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
              'kind': 'Offline execution of pinned gateway admission; no Kero parser/engine/API request',
              'source_head': args.source, 'gateway_path': API_PATH,
              'gateway_sha256': digest(source), 'allowed_verbs': sorted(verbs),
              'adapter': 'Remove only exact register lv3 presentation lines; no physical/measurement changes',
              'adapter_source_proof': 'crates/kerotakis-cli/src/main.rs::Session::exec_line assigns only self.register',
              'controls': control_results,
              'summary': {'records': len(rows),
                          'original_admitted': sum(r['original_admission']['admitted'] for r in rows),
                          'register_only_admitted': sum(r['register_only_admission']['admitted'] for r in rows),
                          'remaining_unsupported_verb_case_counts': dict(sorted(blocked_counts.items()))},
              'cases': rows, 'requests_sent': 0, 'physical_verdict': 'not_assigned'}
    if args.canonical_eyes:
        parser_source = subprocess.check_output(['git', 'show', f'{args.source}:crates/kerotakis-core/src/script.rs'])
        report['eyes_alias_source'] = {'path': 'crates/kerotakis-core/src/script.rs', 'sha256': digest(parser_source), 'proof': 'look/observe map to Operator::Measure Instrument::Eyes; measure eyes maps to the same instrument'}
        report['summary']['canonical_eyes_admitted'] = sum(r['canonical_eyes_admission']['admitted'] for r in rows)
    with args.report.open('x') as stream:
        json.dump(report, stream, indent=2)
        stream.write('\n')
    print(json.dumps(report['summary'], indent=2))


if __name__ == '__main__':
    main()
