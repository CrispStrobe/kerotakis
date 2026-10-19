# Checked partition: full small-record publication

This separately frozen publication complements the [compact acceptance checkpoint](../partition-factor-acceptance-20261019/README.md).
It publishes853 original small raw producer files under [raw](raw), the independent
[acceptance receipt](acceptance.json), and all2193 authenticated
[artifact-member hashes](artifact-member-hashes.json). [Publication binding](publication.json)
retains exact byte counts and hashes for the two larger authenticated binary/archive
blobs kept outside Git. No engine, scientific experiment or native test was rerun.

Exact candidate `d6720509`, binary `41ca6122`, actual producer lock `66886813`:
2153 core,184 thermo,620 native chemistry and two external-hook controls pass;
all500 coverage checks have zero unmet requirements, failures or drift; all33
original logical jobs/142 steps/29 law reports pass. Original source/fixture/data
and first-failure bindings remain unchanged. This is software/source acceptance,
not empirical calibration or a new scientific-catalog selection.

The corrected portable reviewer in the compact checkpoint takes the original
execution-directory layout with the retained actual archives, source plan,
worker and first macOS failure log. The public raw text/hash publication does not
replace those binary inputs or assert that a text-only copy can reauthenticate
binary bytes. Integration and production delivery have separate receipts.
