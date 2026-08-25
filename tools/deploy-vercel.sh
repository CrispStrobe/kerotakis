#!/usr/bin/env bash
# Deploy a built web payload (tools/build-web.sh output) to Vercel.
#
# The same artifact GitHub Pages serves; Vercel is the second, faster
# mirror. Static deploy — no build on Vercel's side, vercel.json rides in
# the payload. Auth per the machine's vercel-deploys.md: token from
# ~/.env, team scope fixed. Preview by default; pass --prod for the
# production alias once the register-dial UX is demo-ready.
#
# Usage: tools/deploy-vercel.sh /path/to/payload [--prod]
set -euo pipefail

PAYLOAD="${1:?usage: deploy-vercel.sh /path/to/payload [--prod]}"
SCOPE="crispstrobes-projects"
PROD=""
[ "${2:-}" = "--prod" ] && PROD="--prod"

[ -f "$PAYLOAD/index.html" ] || { echo "no index.html in $PAYLOAD"; exit 1; }
[ -f "$PAYLOAD/app/index.html" ] || echo "note: payload has no app/ — console only"

VERCEL_TOKEN="$(grep '^VERCEL_TOKEN=' ~/.env | cut -d= -f2-)"
[ -n "$VERCEL_TOKEN" ] || { echo "no VERCEL_TOKEN in ~/.env"; exit 1; }

# --name is deprecated; project linkage comes from `vercel link` state or
# is created on first deploy under the scope. cwd IS the payload so the
# static files deploy as-is.
cd "$PAYLOAD"
# Headless hardening: the CLI crashes on uv_tty_init when it tries to
# open a TTY from a background shell — starve it of one (stdin from
# /dev/null, TERM=dumb, CI=1) and take ITS exit code directly instead
# of trusting log greps (which missed exactly that crash once).
LOG="$(mktemp /tmp/kero-vercel-deploy.XXXXXX.log)"
set +e
TERM=dumb CI=1 npx --yes vercel deploy --token "$VERCEL_TOKEN" --scope "$SCOPE" --yes $PROD \
    </dev/null >"$LOG" 2>&1
rc=$?
set -e
tail -5 "$LOG"
# Belt and braces: the CLI has exited 0 while printing "Error:" before
# (the rate-limit cap) — refuse both failure shapes.
if [ "$rc" -ne 0 ] || grep -q '^Error:' "$LOG"; then
    echo "deploy FAILED (exit $rc; log: $LOG)"
    exit 1
fi
echo "deploy OK (log: $LOG)"
grep -Eo 'https://[a-z0-9.-]*vercel\.app[^ ]*' "$LOG" | tail -1
