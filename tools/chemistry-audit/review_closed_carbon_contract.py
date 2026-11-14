#!/usr/bin/env python3
"""Check the exact versioned delta without changing any frozen manifest."""
import argparse
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent


def check():
    original = HERE / 'source-fleets-2'
    successor = HERE / 'source-fleets-2-closed-carbon-v1'
    names = {p.name for p in original.glob('*.json')}
    if names != {p.name for p in successor.glob('*.json')} or len(names) != 12:
        raise ValueError('successor must retain every original family')
    for name in sorted(names):
        original_bytes = (original / name).read_bytes()
        actual = (successor / name).read_bytes()
        if name != 'polyprotic-carbonate.json':
            if actual != original_bytes:
                raise ValueError(f'unrelated frozen family changed: {name}')
            continue
        expected = json.loads(original_bytes)
        matches = [r for r in expected['relations'] if r['id'] == 'pc-bicarbonate-dose']
        if len(matches) != 1:
            raise ValueError('original forecast is missing or ambiguous')
        matches[0].update(
            id='pc-bicarbonate-closed-carbon-v1',
            assertion='carbonate-closed-equilibrium-v1',
            parameters={'expected_doses': [.0005, .001, .002, .004]},
        )
        if json.loads(actual) != expected:
            raise ValueError('successor changes more than the reviewed relation')
    print('12 families / 288 unchanged protocols / one independently reviewed law revision')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true', required=True)
    parser.parse_args()
    check()
