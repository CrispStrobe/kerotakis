#!/usr/bin/env python3
"""Versioned entry point; shared implementation and definitions are freeze-bound."""
import importlib.util
from pathlib import Path


def main():
    here = Path(__file__).resolve().parent
    spec = importlib.util.spec_from_file_location('versioned_consumer', here.parent / 'controls.py')
    consumer = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(consumer)
    consumer.HERE = here
    consumer.__file__ = __file__
    consumer.paired.HERE = here
    consumer.main()


if __name__ == '__main__':
    main()
