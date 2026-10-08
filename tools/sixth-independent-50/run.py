#!/usr/bin/env python3
"""Execute only the independently frozen sixth forecast on its bound executable."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
ROOT=Path(__file__).resolve().parents[2]
FORECAST_SHA='45e554283e8dd27e9483d45fe9cd33dd66a2d4fa208a8e47b7d3b2fb99ea3a0d'
BINARY_SHA='fcf424a65384c5ef5949960f228fda4e1c3e893ec93c65d58fe5b12064352967'
def digest(path):
    h=hashlib.sha256()
    with Path(path).open('rb') as f:
        for block in iter(lambda:f.read(1024*1024),b''): h.update(block)
    return h.hexdigest()
def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--binary',type=Path,required=True)
    parser.add_argument('--forecast',type=Path,required=True)
    parser.add_argument('--out',type=Path,required=True)
    args=parser.parse_args()
    if digest(args.forecast)!=FORECAST_SHA: raise SystemExit('Frozen forecast hash mismatch')
    if digest(args.binary)!=BINARY_SHA: raise SystemExit('Bound executable hash mismatch')
    forecast=json.loads(args.forecast.read_text())
    if len(forecast['cases'])!=50 or [c['id'] for c in forecast['cases']] != [f'S{i:02}' for i in range(1,51)]:
        raise SystemExit('Fifty unique ordered cases required')
    path=ROOT/'tools/fifth-independent-50/run.py'
    spec=importlib.util.spec_from_file_location('cli_evidence',path)
    shared=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(shared)
    args.binary=args.binary.resolve()
    args.out.mkdir(parents=True,exist_ok=False)
    # S-prefixed cases never inherit the earlier F-prefixed refusal exceptions.
    return shared.experiments(args,forecast,args.out)
if __name__=='__main__': sys.exit(main())
