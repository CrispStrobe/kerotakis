# Oracle Jobs (LIC-010)

Oracle jobs run build-time-only validation tools against external
references (stock PHREEQC, Cantera, Reaktoro, etc.). Their output
never ships in a release binary.

## Directory structure

```
tools/oracle/
├── README.md          ← this file
├── cache/             ← oracle-specific build cache (gitignored)
├── output/            ← raw oracle results (gitignored)
└── approved/          ← reviewed oracle facts promoted to test fixtures
```

## Rules

1. Oracle binaries and their outputs live under `tools/oracle/`,
   never in `crates/` or `data/`.
2. `cache/` and `output/` are in `.gitignore`. Nothing from these
   directories enters version control without explicit review.
3. Approved oracle facts (numerical values, tolerances) are copied
   to `crates/*/tests/oracle/expected/` as reviewed test fixtures.
4. Oracle jobs do not run in CI by default. They run on demand in a
   licensed oracle environment.
5. The oracle itself (e.g. stock PHREEQC binary) is never committed,
   vendored, or fetched by CI.
