# Replacing PHREEQC BASIC

Status: migration in progress, 2026-08-21.

## Implementation progress

- Stage 1 is complete. Kerotakis obtains species reaction enthalpy through the
  checked native `GetSpeciesDeltaH` bridge; neutralisation no longer creates a
  `USER_PUNCH` program. Native results agree with the opt-in legacy BASIC
  oracle for all four embedded databases within the oracle's printed
  precision.
- Stage 2 is complete for native builds. Default Kerotakis builds compile the
  project-owned rejecting backend and omit `PBasic.cpp` and `PBasic.h` from
  compiler inputs. The complete engine suite passes in this configuration,
  capability failures are covered directly, and the cache-only suite passes.
  The Emscripten module also builds in this configuration and its live AgCl
  precipitation check passes without filesystem access.
- Stage 3 is in progress. MY-BASIC is pinned at
  `38baab02ece70b650f5e687e485d879f80843256`; its two core files and full MIT
  notice are vendored byte-for-byte with a provenance manifest and hash audit.
  The disabled-file-loading C99 build passes smoke checks for values, arrays,
  control flow, callbacks, errors, and suspension. Toolchain integration and
  platform builds remain before the stage is complete.
- Stages 4 through 7 have not started. The legacy implementation remains in the
  source tree solely for the explicit `legacy-basic-oracle` development
  feature; it is not part of default native or WebAssembly build inputs.

## Goal

Remove the Gillespie-derived `PBasic` implementation from every Kerotakis
source and release artifact, first without losing chemistry Kerotakis currently
uses and then, where justified, restoring PHREEQC BASIC compatibility on top of
the MIT-licensed MY-BASIC interpreter.

The final runtime must satisfy the project's direct-inclusion policy: only
Kerotakis-owned code with the required grant, MIT, Apache-2.0, BSD-2/3-Clause,
CC0, CC BY, verified public-domain material, or equivalently permissive terms
may enter the official payload. Historical GPL distributions of Gillespie
BASIC are useful provenance evidence but do not satisfy that policy.

This is an engineering plan, not a legal opinion. Ambiguous code does not enter
the replacement.

## Invariants

- Do not copy, translate, mechanically rewrite, or adapt any implementation
  from `PBasic.cpp`, `PBasic.h`, `basic.p`, Chipmunk BASIC, Gillespie-BASIC, or
  another uncleared descendant.
- Preserve PHREEQC's ordinary equilibrium work: solution speciation, pH/pe,
  ionic strength, mineral equilibrium, gas equilibrium, surface/exchange
  equilibrium, redox, and normal selected output.
- Unsupported BASIC-backed keywords must fail explicitly. They must never be
  silently ignored or return plausible-looking zeroes.
- Stock PHREEQC may be used as an external differential oracle while developing
  the replacement. It is not a runtime, release, fixture-generation, or hidden
  build dependency.
- Every stage lands with tests and leaves the tree in a usable state.
- Stage 7 is the release gate: no official binary containing the legacy
  interpreter may be published after the migration is declared complete.

## Sequence and gates

The stages are ordered. Stages 1 and 2 remove the immediate and structural
dependency. Stages 3 through 6 restore only the compatibility Kerotakis can
justify and test. Stage 7 removes the old implementation and proves its
absence.

### Stage 1 — remove Kerotakis' immediate BASIC dependency

Kerotakis currently invokes PHREEQC BASIC directly only to evaluate
`DELTA_H_SPECIES("OH-")` in
`crates/kerotakis-phreeqc/src/aqueous.rs`. PHREEQC already implements this
calculation natively as `Phreeqc::calc_deltah_s(const char*)` in
`vendor/iphreeqc/src/phreeqcpp/basicsubs.cpp` and declares it in
`vendor/iphreeqc/src/phreeqcpp/Phreeqc.h`.

Tasks:

- Add a narrow C++ method to `IPhreeqc` that calls
  `PhreeqcPtr->calc_deltah_s(species)` for the loaded database and current
  temperature/pressure state.
- Expose that method through the flat C API in `IPhreeqc.h` and
  `IPhreeqcLib.cpp`. Return an explicit status separately from the numeric
  result so an invalid instance or unknown species cannot be confused with a
  valid zero enthalpy.
- Declare the new C function in `crates/kerotakis-phreeqc/src/lib.rs` and wrap
  it in a safe `Phreeqc` method.
- Replace the `USER_PUNCH` probe in `neutralisation_enthalpy` with that safe
  native call.
- Preserve the sign convention: neutralisation is the reverse of the database
  reaction defining `OH-`, so the returned dissociation enthalpy is negated.
- Document whether the method requires a minimal `SOLUTION` run to establish
  temperature. Do not depend on implicit mutable state without a test.

Checks:

- Unit-test invalid instance, unknown species, and finite-value handling at the
  C/Rust boundary.
- Test the value for `OH-` against the old `USER_PUNCH` route for each embedded
  database while the oracle route is still present.
- Retain the existing acid/base and equilibrator integration tests.
- Run the native crate test suite with `--features engine` and the cache-only
  suite without the engine.

Exit gate:

- No Kerotakis-owned runtime input contains `USER_PUNCH`, `USER_PRINT`,
  `RATES`, or `CALCULATE_VALUES` merely to retrieve a value already available
  through the native API.
- Neutralisation results remain within the existing database-specific
  tolerances.

### Stage 2 — add a no-BASIC PHREEQC build

Create a build mode in which the equilibrium engine compiles and runs without
the legacy interpreter. This stage deliberately disables BASIC-backed features;
it does not emulate them.

Tasks:

- Add an explicit CMake option such as `IPHREEQC_WITH_BASIC`, defaulting off in
  Kerotakis builds during the migration.
- Remove `src/phreeqcpp/PBasic.cpp` and `PBasic.h` from the no-BASIC source
  list in `vendor/iphreeqc/CMakeLists.txt`.
- Replace unconditional construction in
  `src/phreeqcpp/mainsubs.cpp` with a project-owned rejecting implementation or
  a nullable backend with checked dispatch.
- Keep the small `basic_compile`, `basic_run`, and `basic_free` boundary stable
  where that reduces churn, but implement the no-BASIC branch entirely in new
  Kerotakis-owned code.
- Reject at least `RATES`, `KINETICS` that require a rate program,
  `USER_PUNCH`, `USER_PRINT`, `CALCULATE_VALUES`, and `USER_GRAPH` before
  execution. Include the keyword and the disabled capability in the error.
- Check databases at load time and inputs at run time separately. A database
  may contain dormant rate definitions that ordinary equilibrium does not use;
  decide and test whether those definitions are retained inertly or rejected.
- Add the option to native and Emscripten build scripts so both targets select
  the same capability deliberately.

Checks:

- Add a focused test for every rejected keyword and assert the exact error
  category.
- Add negative tests showing that unsupported programs never execute partly.
- Run representative equilibrium tests covering aqueous speciation, minerals,
  gases, surfaces/exchange, redox, brines, and normal `SELECTED_OUTPUT`.
- Build static native libraries and the Emscripten module without either legacy
  source file in the compiler inputs.

Exit gate:

- The complete current Kerotakis test suite passes in no-BASIC mode, except for
  tests intentionally classified as future BASIC compatibility tests.
- Running a BASIC-backed keyword produces a stable capability error rather
  than a crash, hang, silent omission, or fabricated result.

### Stage 3 — vendor a pinned MY-BASIC revision

MY-BASIC is a standard-C embeddable interpreter under the MIT license. It may
provide the generic language engine, not the PHREEQC compatibility behavior.

Tasks:

- Review and pin one full upstream commit SHA; never vendor a mutable branch.
  The candidate observed during this investigation was
  `38baab02ece70b650f5e687e485d879f80843256`, but it must pass the dependency
  review at the time it is adopted.
- Vendor only the required core files (`core/my_basic.c` and
  `core/my_basic.h`) plus the complete upstream `LICENSE`. Do not include the
  standalone shell, examples, binaries, or unrelated assets.
- Record upstream URL, commit, retrieval date, file hashes, license, copyright
  notice, local build options, and modifications in the provenance manifest.
- Preserve the MIT header in both core source files and surface the dependency
  in notices and the SBOM.
- Audit configurable features. Disable file import, interactive input, native
  OS access, and other facilities not required by PHREEQC programs.
- Compile with the project's native, Android, Apple, and Emscripten toolchains.

Checks:

- Verify the vendored hashes against the pinned upstream commit in a
  non-release audit script.
- Compile the two core files as C and link them into the C++ IPhreeqc build.
- Run MY-BASIC's relevant upstream tests without importing test code into the
  shipping payload.
- Add smoke tests for numeric and string values, arrays, control flow, native
  callbacks, errors, and cancellation.

Exit gate:

- The dependency has a complete, reproducible MIT provenance record and no
  unreviewed files enter the build or release artifact.

### Stage 4 — implement a clean PHREEQC adapter

Implement the existing PHREEQC-facing interpreter contract on MY-BASIC without
reusing the legacy implementation. Preserve the boundary, not the old code.
Using a new name such as `KeroBasicAdapter` is preferable for provenance even
if `Phreeqc::basic_compile`, `basic_run`, and `basic_free` remain unchanged.

Tasks:

- Define a project-owned opaque compiled-program object to replace the legacy
  `linebase`, `varbase`, and `loopbase` internals.
- Translate PHREEQC's numbered-line form into MY-BASIC labels and translate
  numeric jump/restore targets deterministically.
- Implement `SAVE` and `PUNCH` as adapter operations with explicit sinks, not
  console output.
- Populate PHREEQC runtime variables such as `M`, `M0`, `TIME`, `TC`, `TK`,
  `PARM()`, and solution volume before each run.
- Register chemistry functions as native callbacks using interpreter userdata.
  Start with the functions required by the accepted corpus, such as `ACT`,
  `MOL`, `TOT`, `SI`, `SR`, `LM`, and `DELTA_H_SPECIES`.
- Preserve numeric precision, string ownership, array bounds, case behavior,
  boolean semantics, and error locations deliberately. Do not assume MY-BASIC
  and PHREEQC BASIC agree by accident.
- Add deterministic instruction, wall-clock, recursion, allocation, array, and
  output budgets. Cancellation must unwind without corrupting the owning
  `Phreeqc` instance.
- Keep file, network, process, and host-environment access unavailable.

Checks:

- Unit-test each source transformation independently, including strings and
  comments containing keyword-like text.
- Unit-test every registered callback for argument count/type, missing species,
  finite results, and ownership.
- Test compile/run/free cycles, repeated runs, multiple independent IPhreeqc
  instances, cancellation, and cleanup after errors.
- Fuzz the compatibility preprocessor and adapter boundary with bounded input.

Exit gate:

- A minimal Kerotakis-owned `RATES`, `CALCULATE_VALUES`, and `USER_PUNCH`
  program compiles and runs without any legacy interpreter source in its
  compilation unit.

### Stage 5 — define and implement the supported dialect

Compatibility is a versioned feature set, not a claim that every historical
PHREEQC BASIC program works.

Tasks:

- Inventory the BASIC programs actually present in the bundled PHREEQC
  databases, Kerotakis inputs, and accepted tests.
- Produce a machine-readable capability manifest covering statements,
  operators, built-in functions, variables, arrays, strings, comments, and
  limits.
- Initially support only syntax exercised by bundled `RATES`,
  `CALCULATE_VALUES`, and Kerotakis-owned inputs.
- Prioritize arithmetic, assignment, `IF/THEN/ELSE`, `FOR/NEXT`,
  `WHILE/WEND`, `GOTO`, `GOSUB/RETURN`, `DATA/READ/RESTORE`, arrays, `SAVE`,
  and `PUNCH` where the corpus requires them.
- Add chemistry callbacks one by one, with a direct test and documented return
  convention for each.
- Reject every unsupported token or semantic form at compile time where
  possible. Include the program name, source line, and unsupported feature.
- Version the advertised compatibility level so caches and bug reports can
  identify the exact dialect implementation.

Checks:

- Add one positive and one negative test per manifest feature.
- Compile every accepted bundled program as a corpus test.
- Maintain an explicit list of bundled programs that remain unsupported and
  ensure they fail for the documented reason.

Exit gate:

- All programs labeled supported compile successfully; all programs labeled
  unsupported fail deterministically before changing chemical state.

### Stage 6 — differential and live validation

Compare the replacement with an unmodified stock PHREEQC executable kept
outside the official source and release payload.

Tasks:

- Build a Kerotakis-owned differential corpus that exercises compilation,
  `SAVE`, `PUNCH`, `CALCULATE_VALUES`, kinetic rate evaluation, time stepping,
  branching, arrays, strings, and expected failures.
- Run the same database, PHREEQC input, temperatures, tolerances, and time
  steps through stock PHREEQC and the replacement.
- Compare observable chemistry rather than allocator or formatting details:
  saved moles, punched values, pH, totals, saturation indices, phase amounts,
  kinetic trajectories, warnings, and failure category.
- Define absolute/relative tolerances per observable. Do not hide systematic
  differences behind a single broad tolerance.
- Add execution-limit cases: infinite loops, excessive output, recursion,
  oversized arrays, malformed programs, cancellation, and retry after failure.
- Keep oracle jobs optional and reproducible. Release builds and ordinary CI
  must not download or execute stock PHREEQC.
- Persist only independently distributable test inputs and minimal numerical
  results approved by the provenance policy; record oracle version and command
  separately.

Checks:

- Unit checks run on every change.
- Differential checks run in the licensed oracle environment on demand and
  before increasing the advertised compatibility level.
- Native and Emscripten live checks use the same corpus and tolerances.
- Any mismatch is classified as adapter bug, expected semantic difference,
  stock-PHREEQC issue, or unsupported feature before it can be waived.

Exit gate:

- Every supported corpus program agrees with the stock oracle within its
  declared tolerances and exhibits equivalent failure behavior under the
  declared resource limits.

### Stage 7 — delete the legacy implementation and prove absence

Tasks:

- Delete `vendor/iphreeqc/src/phreeqcpp/PBasic.cpp` and `PBasic.h` from the
  Kerotakis source tree and remove every build-system reference to them.
- Remove any copied `basic.p`, generated translation, patch, fixture, token
  table, or excerpt derived from the legacy interpreter.
- Retain provenance notes as factual documentation; do not retain source
  excerpts as evidence.
- Search source, generated build directories, archives, static libraries,
  native binaries, Android packages, Apple bundles, and WebAssembly output for
  legacy filenames, class/function symbols, distinctive notices, and known
  interpreter strings.
- Generate a linker map or symbol listing for each release target and archive
  the absence check with release evidence.
- Update notices, SBOM, source manifests, documentation, and capability output
  to identify MY-BASIC and the Kerotakis adapter accurately.
- Run the full unit, integration, differential, fuzz-smoke, native, mobile, and
  Emscripten validation matrix.

Checks:

- Source check: no tracked legacy interpreter implementation remains.
- Build-input check: compiler traces contain no legacy source path.
- Symbol check: no `PBasic` implementation symbol remains unless a deliberately
  retained compatibility-facing name is documented as new Kerotakis code.
- String check: no legacy permission notice or distinctive implementation text
  appears in shipping artifacts.
- Reproducibility check: a clean checkout builds without retrieving the old
  interpreter.

Exit gate:

- The source manifest, build inputs, SBOM, linker/symbol evidence, and release
  artifacts all demonstrate that the old implementation is absent.
- Ordinary equilibrium and every advertised replacement-BASIC feature pass on
  all supported targets.

## Definition of done

The migration is complete only when all of the following are true:

- Kerotakis obtains neutralisation enthalpy through a native API, not
  `USER_PUNCH`.
- A no-BASIC configuration remains available and fully tested.
- MY-BASIC is pinned, minimally vendored, and attributed under its MIT terms.
- The adapter is demonstrably new Kerotakis code and supports a documented,
  bounded dialect.
- Supported programs agree with the external stock-PHREEQC oracle.
- Unsupported programs fail explicitly without mutating chemical state.
- No legacy interpreter source, object code, symbols, or text ships in any
  official artifact.
