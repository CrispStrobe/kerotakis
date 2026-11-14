# Python encoding preflight: first failure

PR #881's preflight for source `7bd82c8041fc9d67a8e5400bf054dc53b9c0a0cd`
failed the repository's Python encoding lint in GitHub Actions run 43306467715,
job 131491254273. The complete original log has SHA-256
`11954c2dbefc89651cfa6700c37d80bdedaa1125f641571270e308a0c101a8ea`.
The recorded output includes:

```text
text I/O without an explicit encoding breaks on Windows (cp1252). Pass encoding="utf-8".
tools/peroxide_iodide_acid_partition_coverage.py:47: read_text() without encoding=
tools/peroxide_iodide_acid_partition_coverage.py:70: read_text() without encoding=
tools/peroxide_iodide_acid_partition_coverage.py:126: write_text() without encoding=
```

The successor explicitly uses UTF-8 for every text read and write in that
script. Its encoding lint and coverage `--check` passed locally; the complete
preflight remains required on the successor.

A later complete local lint found eleven further implicit encodings in
`tools/redox_respeciation_coverage.py` and `tools/tests/test_sundials_source_cache.py`.
Those calls also use UTF-8 in the successor. The complete encoding lint then
reported zero findings and all eight SUNDIALS source-cache tests passed. The redox coverage check also passed using a database checkout verified against
the pinned PHREEQC submodule commit; the hosted preflight remains required.

The subsequent GitHub preflight on `66d0d9704d5d443684acfefdadf0395f14c791c9`
(run 43316391517, job 131525505503) independently reported those same eleven
calls. Its original complete log has SHA-256
`beb22ec703aea9d28eff65f0ff2f40e5d32b41b3332ff4e8f4f12689c21b7efa`.
