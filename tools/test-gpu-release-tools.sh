#!/usr/bin/env bash
# Deterministic GPU-5/GUI-098 release-tool self-tests.
#
# This gate deliberately tests only offline parsers, measurements, evaluators,
# and provenance auditing. Physical WebGPU measurements and the five-host
# release matrix require recorded device evidence and therefore stay out of CI.
set -euo pipefail
cd "$(dirname "$0")/.."

node --test \
  tools/brd080-device-evidence.test.mjs \
  tools/frontend-asset-budget.test.mjs \
  tools/gpu5-probe-lib.test.mjs \
  tools/gpu5-release-evaluate.test.mjs \
  tools/tests/gui098-release-audit.test.mjs
