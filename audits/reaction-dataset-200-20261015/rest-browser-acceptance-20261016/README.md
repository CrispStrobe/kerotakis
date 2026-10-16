# Actual REST browser acceptance — 2026-10-16

[The independently reviewed receipt](review.json) accepts fifteen UI, privacy and
network controls using actual Chromium 153 in a disposable remote browser VM.
The production `rest.html` bytes match `web/rest.html` at reviewed PR795 source
`efd79bf88eb1514bb87506171c39e808c3b544e1` (SHA-256
`f8ccf3dfb62ceafb104db6daad69d67886823f4a20a2bd9d3989ee7b0dcdff80`).
Browser web security and CORS remained enabled. Production configuration and
untrusted execution-worker security were unchanged.

Actual form submission returned HTTP200, completed execution and two native
observations. Invalid credentials returned 401; unsupported commands returned
422. Endpoint validation, text-only hostile-response rendering, empty browser
storage/cookies and reload cleanup passed. The labelled clear button clears key
and results while retaining the editable script; submit retains both inputs in
page memory. Neither input persisted in the tested browser stores.

The actual production client executed successfully from its Vercel origin.
GitHub Pages returned **404** for `/kerotakis/rest.html`. A controlled copy of the
exact client at the approved Pages origin executed the real cross-origin API;
an unapproved origin was blocked by actual browser CORS. That controlled origin
test proves browser/API compatibility, **not Pages publication**. Publish through
the normal main Pages pipeline after source integration, then verify the actual
served client.

Production remained API revision `9bee95d9` with engine `d791d306`. These controls
do not accept a newer preview engine or assign scientific verdicts. Setup
failures remain preserved. The first browser report also preserved one harness
assertion failure: it queried `status` instead of the API's `execution_status`.
The unchanged saved response was independently reviewed offline; no API execution
was repeated to correct that assertion.
