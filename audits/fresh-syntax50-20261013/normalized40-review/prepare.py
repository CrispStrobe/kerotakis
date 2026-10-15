#!/usr/bin/env python3
"""Verify preserved forty-case evidence and prepare a review queue; never run a CLI."""
import argparse
import hashlib
import json
import math
from pathlib import Path


class EvidenceError(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise EvidenceError(message)


def strict(raw):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, 'duplicate JSON key: ' + key)
            result[key] = value
        return result

    def number(raw):
        value = float(raw)
        require(math.isfinite(value), 'nonfinite JSON number')
        return value

    def constant(raw):
        raise EvidenceError('nonfinite JSON constant: ' + raw)

    return json.loads(raw, object_pairs_hook=pairs, parse_float=number, parse_constant=constant)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def read(root, name, limit=8 * 1024 * 1024):
    relative = Path(name)
    require(not relative.is_absolute() and '..' not in relative.parts, 'unsafe evidence path')
    path = root / relative
    require(path.resolve().is_relative_to(root.resolve()), 'evidence path escapes root')
    require(not any(p.is_symlink() for p in [path, *path.parents] if p != root.parent),
            'symlinked evidence path')
    require(path.is_file() and path.stat().st_size <= limit, 'missing/oversized evidence: ' + name)
    with path.open('rb') as stream:
        raw = stream.read(limit + 1)
    require(len(raw) <= limit, 'oversized evidence: ' + name)
    return raw


def prepare(design, evidence):
    """Return deterministic review material; physical verdicts require subsequent review."""
    binding_raw = read(design.parent, 'execution-binding.json')
    binding = strict(binding_raw)
    freeze_raw = read(design, 'freeze.json')
    freeze = strict(freeze_raw)
    require(digest(freeze_raw) == binding['design_freeze_sha256'], 'design freeze changed')
    require(freeze['files'] == binding['design_files'], 'design members changed')
    require(freeze['experiment_count'] == binding['experiment_count'] == 40, 'wrong experiment count')
    require(digest(read(design.parent, 'run-frozen.py')) == binding['runner_sha256'], 'runner changed')
    for name, expected in binding['design_files'].items():
        require(digest(read(design, name)) == expected, 'frozen member changed: ' + name)
    experiments = strict(read(design, 'expectations.json'))['experiments']
    scripts = sorted(name for name in binding['design_files'] if name.endswith('.lab'))
    require(len(scripts) == len(experiments) == 40, 'wrong script/expectation count')
    by_script = {x['script']: x for x in experiments}
    require(len(by_script) == 40 and set(by_script) == set(scripts), 'expectation/script mismatch')
    require(len({x['id'] for x in experiments}) == 40, 'duplicate experiment id')
    require(strict(read(evidence, 'binding.json')) == binding, 'execution binding mismatch')
    expected_members = set()
    cases = []
    for ordinal, name in enumerate(scripts, 1):
        stem = f'{ordinal:02d}'
        record_name = stem + '.result.json'
        member_names = [record_name, stem + '.stdout.jsonl', stem + '.stderr.txt']
        expected_members.update(member_names)
        present = [(evidence / member).exists() or (evidence / member).is_symlink() for member in member_names]
        require(all(present) or not any(present), 'orphan/incomplete case evidence: ' + stem)
        forecast = by_script[name]
        case = {'experiment_id': forecast['id'], 'ordinal': ordinal, 'script': name,
                'forecast': forecast, 'physical_verdict': 'pending_review'}
        if not any(present):
            case['execution_status'] = 'missing'
            cases.append(case)
            continue
        raw_record = read(evidence, record_name)
        record = strict(raw_record)
        require(record['case'] == name and type(record['ordinal']) is int and record['ordinal'] == ordinal,
                'case/ordinal mismatch: ' + stem)
        require(record['script_sha256'] == binding['design_files'][name], 'wrong script hash: ' + stem)
        require(type(record['exit_code']) is int and type(record['timeout']) is bool, 'invalid process outcome')
        elapsed = record['elapsed_seconds']
        require(type(elapsed) in (int, float) and math.isfinite(elapsed) and elapsed >= 0, 'invalid elapsed time')
        stdout = read(evidence, member_names[1])
        stderr = read(evidence, member_names[2])
        for key, raw in [('stdout', stdout), ('stderr', stderr)]:
            require(type(record[key + '_bytes']) is int and record[key + '_bytes'] == len(raw), 'stream size mismatch')
            require(record[key + '_sha256'] == digest(raw), 'stream hash mismatch')
        rows = []
        row_lines = []
        parse_errors = []
        for line_number, line in enumerate(stdout.splitlines(), 1):
            try:
                row = strict(line.decode('utf-8'))
                require(isinstance(row, dict), 'non-object CLI row')
            except (ValueError, UnicodeError) as error:
                # Keep valid observations on either side of a malformed raw line.
                parse_errors.append({'line': line_number, 'error': str(error)})
                continue
            rows.append(row)
            row_lines.append(line_number)
        sequences = [row.get('output_sequence') for row in rows]
        sequence_ok = bool(rows) and not parse_errors and all(
            type(x) is int and x == i for i, x in enumerate(sequences))
        case.update(execution_status='recorded', process=record, record_sha256=digest(raw_record),
                    stderr=stderr.decode('utf-8', errors='replace'), stdout_rows=rows,
                    stdout_row_line_numbers=row_lines, stdout_parse_errors=parse_errors,
                    stdout_parse_error=parse_errors[0]['error'] if parse_errors else None,
                    contiguous_output_sequence=sequence_ok)
        cases.append(case)
    actual_members = {p.name for p in evidence.iterdir() if p.name.endswith(('.result.json', '.stdout.jsonl', '.stderr.txt'))}
    require(actual_members <= expected_members, 'unexpected case evidence')
    recorded = sum(c['execution_status'] == 'recorded' for c in cases)
    complete = (evidence / 'completed.json').exists()
    if complete:
        marker = strict(read(evidence, 'completed.json'))
        require(type(marker['cases']) is int and marker['cases'] == recorded == 40, 'false completion marker')
        require(not any(c['process']['timeout'] for c in cases), 'completion contradicts timeout')
        require(marker['binding_sha256'] == digest(read(evidence, 'binding.json')), 'completion binding mismatch')
        require(marker['original_outcomes_preserved'] is True, 'invalid preservation marker')
    return {'schema_version': 1, 'kind': 'verified saved-evidence review queue; not physical acceptance',
            'source_binding': binding, 'recorded_cases': recorded, 'missing_cases': 40 - recorded,
            'completion_marker_verified': complete, 'physical_verdicts_assigned': 0,
            'row_mapping_policy': 'output_sequence is stream position; step is a bench-log reference. Neither is an experiment id. Check operators and script commands before assigning observations.',
            'cases': cases}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--design', type=Path, required=True)
    parser.add_argument('--evidence', type=Path, required=True, help='directory containing the runner case records and binding')
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    report = prepare(args.design, args.evidence)
    # Never overwrite an earlier review or any producer output.
    with args.out.open('x') as output:
        output.write(json.dumps(report, indent=2, allow_nan=False) + '\n')


if __name__ == '__main__':
    main()
