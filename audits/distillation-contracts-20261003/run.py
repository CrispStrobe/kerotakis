"""Replay frozen focused controls and check material and energy contracts."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import subprocess
import sys

SUITE = Path(__file__).resolve().parent
REPO = SUITE.parents[1]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def strict(raw):
    def number(value):
        result = float(value)
        if not math.isfinite(result):
            raise ValueError('nonfinite JSON')
        return result
    return json.loads(raw, parse_float=number,
                      parse_constant=lambda value: (_ for _ in ()).throw(ValueError(value)))


def amount(row, vessel, species):
    v = next(v for v in row['bench']['vessels'] if v['id'] == vessel)
    return sum(p['moles'] for p in v['contents'] if p['species'] == species)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=REPO, text=True).strip()
    subprocess.run([sys.executable, str(REPO/'audits/independent-next-50-20261002/run.py'),
                    '--suite', str(SUITE), '--binary', str(args.binary),
                    '--binary-source-commit', commit, '--out', str(args.out)], check=True)
    receipt = strict((args.out/'execution.json').read_text())
    cases = strict((SUITE/'predictions.json').read_text())
    checks = []

    def check(name, passed):
        checks.append(dict(name=name, passed=bool(passed)))

    check('frozen predictions', receipt['predictions_sha256'] == digest(SUITE/'predictions.json'))
    check('executable identity', receipt['binary_source_commit'] == commit
          and receipt['binary_sha256'] == digest(args.binary))
    check('all controls in both modes', sorted((r['id'], r['mode']) for r in receipt['runs'])
          == sorted((c['id'], mode) for c in cases for mode in ['text', 'json']))
    for record in receipt['runs']:
        ident = record['id']
        label = ident+' '+record['mode']
        check(label+' completed', record['exit_code'] == 0)
        for stream in ['stdout', 'stderr']:
            check(label+' '+stream+' integrity', digest(args.out/record[stream]) == record[stream+'_sha256'])
        if record['mode'] != 'json':
            continue
        rows = [strict(line) for line in (args.out/record['stdout']).read_text().splitlines()]
        views = [r for r in rows if r['operator']['op'] == 'inspect']
        check(ident+' exactly two inspections', len(views) == 2)
        events = [e for r in rows for e in r['events']]
        check(ident+' no failure or capability refusal', not any(
            e['event'] in ['solver_failed', 'not_yet_modeled', 'refused'] for e in events))
        cuts = [e for e in events if e['event'] == 'distilled']
        check(ident+' one completed cut', len(cuts) == 1)
        if len(views) != 2 or len(cuts) != 1:
            continue
        before, after = views
        cut = cuts[0]
        check(ident+' positive finite heat', math.isfinite(cut['energy_kj']) and cut['energy_kj'] > 0)
        species = ['water', 'ethanol'] if ident == 'D03' else ['water' if ident == 'D01' else 'methanol']
        for name in species:
            initial = amount(before, 0, name)
            final = amount(after, 0, name) + amount(after, 1, name)
            check(ident+' '+name+' conserved', abs(final/initial-1) < 1e-8)
        if ident in ['D01', 'D02']:
            name = species[0]
            target = amount(before, 0, name)*1e-14
            received = amount(after, 1, name)
            check(ident+' requested positive microscopic cut', received > 0 and abs(received/target-1) < 1e-10)
        elif ident == 'D03':
            water, ethanol = amount(after, 1, 'water'), amount(after, 1, 'ethanol')
            check(ident+' spends requested heat', abs(cut['energy_kj']/8-1) < 1e-10)
            check(ident+' latent meter matches material', abs((water*40.657+ethanol*38.58)/8-1) < 1e-8)
            initial_fraction = amount(before, 0, 'ethanol')/(amount(before, 0, 'water')+amount(before, 0, 'ethanol'))
            check(ident+' positive enriched overhead', water > 0 and ethanol > 0 and ethanol/(water+ethanol) > initial_fraction)
            check(ident+' endpoint boiling drift', cut['ended'] > cut['at'])
        else:
            initial = amount(before, 0, 'methanol')
            check(ident+' whole inventory lifted', amount(after, 0, 'methanol') == 0
                  and abs(amount(after, 1, 'methanol')/initial-1) < 1e-12)
            check(ident+' surplus heat not claimed', cut['energy_kj'] < 100)
    result = dict(checks=len(checks), passed=all(c['passed'] for c in checks), details=checks)
    (args.out/'verification.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(dict(checks=result['checks'], passed=result['passed'],
                          failures=[c for c in checks if not c['passed']]), indent=2))
    return 0 if result['passed'] else 1


if __name__ == '__main__':
    sys.exit(main())
