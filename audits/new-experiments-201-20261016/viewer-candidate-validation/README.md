# Viewer candidate: targeted ownership validation

Ten deterministic checks pass against the exact actual-source TypeScript modules
identified in [stdout](stdout.json). This is an uncommitted candidate snapshot,
not an integrated release or browser result. [Execution hashes](execution.json)
and [review scope](review.json) bind the saved output and portable reader.

The earlier candidate passed four controller cases but leaked a canvas for three
late adapter setup failures; its [structured first outcome](first-candidate-structured-outcome.json)
remains separate. The fixed candidate passes stale loading/resize/disposal,
setup-failure ordering, stale renderer loading and all three late setup failures.

Run `node --max-old-space-size=128 check.cjs CHECKOUT [TYPESCRIPT_MODULE]` with
pinned TypeScript5.9.3 dependencies available. It transforms the actual source
in memory and uses a synthetic DOM/renderer; no Vite, browser, WebGL, native
chemistry engine or application build starts. Source hashes can differ on a new
candidate and must receive their own receipt. The original frozen source-failure
proof remains unchanged. Maintained test discovery and actual browser acceptance
still gate the eventual product PR.
