#!/usr/bin/env bash
WT=/mnt/volume1/kerotakis-e9c
: > "$WT/.gate/gate.log"; echo "QUEUED $(date -Is)" >> "$WT/.gate/gate.log"
exec flock -o -w 3000 /mnt/volume1/.kero-build-lock "$WT/.gate/run.sh" "$@"
