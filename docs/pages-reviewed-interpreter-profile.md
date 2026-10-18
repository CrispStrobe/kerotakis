# Match the Pages engine to the reviewed BASIC profile

The actual published Pages source `76fe144` serves an IPhreeqc module but
refuses USER_PUNCH with BASIC disabled. The separate Vercel source `ed7c452d`
passes actual interpreter arithmetic and uncached native/browser solution
parity. See the
[retained browser observation](https://github.com/CrispStrobe/kerotakis/blob/a1dbcedddca6bd6e8eae86048f045f9cc1dcd126/audits/target468-20261016/production-release-ed7c452d-20261018/browser-report.json).

The `demo` workflow builds and publishes Pages. It previously omitted
`IPHREEQC_BASIC_MODE`, whose standalone builder default is disabled; the
required `iphreeqc-wasm`/`wasm-bridge` gates explicitly select MY-BASIC.
Select that same reviewed profile for the demo and run the existing actual
Wasm solver/BASIC probe against the final packaged site. This checks real
precipitation, native thermochemistry and USER_PUNCH output, rather than
accepting a present file or metadata declaration. Existing browser, PWA,
localization, DOM, fluid and other acceptance steps remain unchanged.

The standalone explicit disabled mode remains available for its separate
contract. No chemistry parameters, cache answers, assertions, snapshots or
branch protection change. Validation remains pending: build the exact changed
source, run all existing required gates and applicable own fleet, integrate
normally, then observe actual published Pages assets and interpreter arithmetic
and an uncached solution against a source-bound native reference. A workflow
edit or successful Vercel test cannot certify an unpublished Pages payload.
