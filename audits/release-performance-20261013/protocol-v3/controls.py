#!/usr/bin/env python3
"""Versioned entry point; shared implementation and definitions are freeze-bound."""
import argparse
import json
import importlib.util
from pathlib import Path


def main():
    here = Path(__file__).resolve().parent
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument('--bindings', type=Path, required=True)
    args, _ = parser.parse_known_args()
    plan = json.loads((here / 'candidate-plan.json').read_text())
    binding = json.loads(args.bindings.read_text())
    if binding['candidate']['binary_sha256'] != plan['controlled_candidate_binary_sha256']:
        raise ValueError('Version3 requires the frozen controlled candidate binary')
    spec = importlib.util.spec_from_file_location('versioned_consumer', here.parent / 'controls.py')
    consumer = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(consumer)
    consumer.HERE = here
    consumer.__file__ = __file__
    consumer.paired.HERE = here
    consumer.main()


if __name__ == '__main__':
    main()
