#!/usr/bin/env bash
WT=/mnt/volume1/kerotakis-e9c
LOG="$WT/.gate/gate.log"
: > "$LOG"
cd "$WT" || { echo "RESULT: FAIL (cwd guard)" >>"$LOG"; exit 0; }
FREE=$(df -BG --output=avail /mnt/volume1 | tail -1 | tr -dc '0-9')
echo "free=${FREE}G" >>"$LOG"
[ "${FREE:-0}" -lt 8 ] && { echo "RESULT: FAIL (free-space guard)" >>"$LOG"; exit 0; }
export CARGO_TARGET_DIR="$WT/target"
RC=0
for step in "$@"; do
  echo "=== STEP: $step ===" >>"$LOG"
  bash -lc "cd '$WT' && $step" >>"$LOG" 2>&1
  c=$?; echo "=== CODE $c ===" >>"$LOG"; [ "$c" -ne 0 ] && RC=1
done
[ "$RC" -eq 0 ] && echo "RESULT: PASS" >>"$LOG" || echo "RESULT: FAIL" >>"$LOG"
exit 0
