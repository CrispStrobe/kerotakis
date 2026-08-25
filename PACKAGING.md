# Packaging: the PWA, the Mac app, the iPhone app

One UI (`web/app`), three shipped products. The bench is the same Svelte
app in all three; only the transport under `EngineHost` differs, which is
the whole point of PROTOCOL.md:

| Product | Engine | Where it comes from |
|---|---|---|
| **PWA** | `kerotakis-wasm` + IPhreeqc in a module worker | `tools/build-web.sh`, served statically |
| **macOS app** | IPhreeqc linked natively, in-process | `tools/build-macos-appstore.sh` |
| **iOS app** | the same native engine | `tools/build-ios-appstore.sh` |

The UI cannot tell them apart. Neither should a reader of this file: a
behaviour that differs between two of them is a bug in one of them.

---

## The PWA

`tools/build-web.sh` produces the whole payload. Its layout matters, because
every URL in the manifest and the service worker is relative to where it
lands:

```
/                      the console page (index.html) + the engine payload
/app/                  the bench (web/app, content-hashed by Vite)
/manifest.webmanifest  ONE manifest, shared, scoped to the payload root
/sw.js                 ONE worker, scope "/", precaching both documents
/privacy.html          required by the App Store and by external TestFlight
```

Deployed at `crispstrobe.github.io/kerotakis/` — a **subpath**, which is why
`tools/test-pwa.mjs` serves it from one too. A payload served at the root
hides every relative-URL defect in the manifest.

Three things about it are easy to get wrong and were:

- **The bench must register the worker itself.** Only the console page ever
  did, so opening `/app/` — the URL the README advertises — installed
  nothing at all unless you happened to visit the console first.
  `web/app/src/lib/pwa.svelte.ts` is the bench's half; it resolves the
  worker through `resolvePayloadBase()`, the same function that finds the
  engine, so the two can never disagree about where the payload root is.
- **Vite will content-hash a private copy of anything the HTML links.** A
  `<link rel="manifest" href="../manifest.webmanifest">` left in
  `web/app/index.html` becomes `app/assets/manifest-<hash>.webmanifest`, and
  a manifest served from `app/assets/` scopes the installed app to
  `app/assets/`. `vite.config.ts` injects those tags after the asset pass
  instead.
- **A directory URL is not a cached URL.** `app/index.html` is precached;
  `app/` is not. Offline navigation to `/app/` only works because the worker
  falls back to the document that owns the path.

`node tools/test-pwa.mjs <payload>` asserts all of this against a real
headless Chrome over the DevTools protocol — scope, resolved `start_url`,
every icon's HTTP status, the precache contents, and three offline
navigations. It runs in CI's `demo` job. No dependencies: Node's global
`WebSocket` speaks CDP.

---

## Icons

One vector mark, four masters, because each destination masks it
differently. `tools/gen-icons.py` draws the masters; `tools/gen-app-icons.sh`
runs that, expands with `tauri icon`, and then fixes the two outputs Tauri
gets wrong for Apple:

| Master | For | Why it is its own file |
|---|---|---|
| `master-full-bleed.svg` | iOS, Android, the App Store marketing icon | the OS masks it; a rounded or inset source survives as a dark ring |
| `master-macos.svg` | the `.icns` | macOS applies **no** mask, so the squircle has to *be* the artwork, on a transparent margin |
| `master-maskable.svg` | the PWA `purpose: maskable` icon | Android may crop to a circle, so the mark fits the inner-80% safe zone |
| `master-rounded.svg` | favicon, PWA `purpose: any` | nothing masks these and a bare square would be the odd one out |

The squircle is a sampled superellipse, not an `rx` rounded rect: its corner
profile matches a real macOS system icon to within 1–2% at every depth,
where a circular-arc corner visibly does not.

Two channel rules pull in opposite directions and both are enforced in
`gen-icons.py`: `tauri::generate_context!` panics with *"icon … is not
RGBA"* on a channel-less PNG, and Apple **rejects** an alpha channel in the
1024 marketing icon. Hence `icons/icon.png` (RGBA, for Tauri) and
`icons/appstore-1024.png` (RGB, for upload) from the same master, and the
iOS asset set flattened after `tauri icon` writes it.

`python3 tools/gen-icons.py --check` proves the committed PNGs still match
their source.

---

## The desktop and mobile shell

`web/app/src-tauri` is deliberately outside the cargo workspace so a
kerotakis build never needs the platform webview toolchains. It carries its
own empty `[workspace]` table, which is what lets it build from a git
worktree under `.claude/worktrees/` — the root manifest's `exclude` is
relative to the root and does not match that path.

**The shell is a library with a thin binary on top**, and that is not
stylistic. iOS never links a Rust binary: cargo emits `libapp.a`, Xcode
does the final link, and the generated `main.mm` calls `ffi::start_app()`
— the extern `"C"` symbol `tauri::mobile_entry_point` writes from the
annotated `run()`. A crate with only a `main.rs` fails with *"no library
targets found"* at the "Build Rust Code" phase, long after everything else
has already succeeded. The library target must be named `app`, because the
generated project links `libapp.a` by that exact name.

**The shell bundles its own payload.** The bench fetches its lessons and the
codex from `resolvePayloadBase()`; on the web that is one directory up, and
in a Tauri shell there is no directory up, so `tools/build-shell-payload.sh`
assembles `web/app/public/engine/` and Vite copies it into `dist/`. It runs
from `beforeBuildCommand`, not as a step to remember, because forgetting it
ships an app with an empty lesson picker and no experiment catalog — which
looks like a design choice rather than a missing directory.

### macOS

```console
$ tools/build-macos-appstore.sh --no-upload     # build, sign, wrap, stop
$ ASC_APP_ID=<numeric app id> tools/build-macos-appstore.sh
```

Mac App Store submission has two artifacts and two identities, which is why
this is a script and not one `tauri build`:

- the **`.app`**, signed `Apple Distribution: …`, sandboxed, with the App
  Store provisioning profile embedded (`bundle.macOS.files`) so the identity
  entitlements it claims are ones it may claim;
- the **`.pkg`**, a `productbuild` wrapper signed with the separate
  `3rd Party Mac Developer Installer: …` identity. `altool --type macos`
  uploads the `.pkg`, never the `.app`.

Entitlements are split for the same reason: `entitlements.plist` is the
sandbox, the open/save panel and printing; `entitlements.appstore.plist`
adds `com.apple.application-identifier` and `com.apple.developer.team-identifier`,
which Xcode injects on its own and Tauri does not. A build signed with
anything but the matching profile is rejected for naming an identifier it
cannot claim, so those two keys must not be in the base file.

### 🚨 A sandboxed WKWebView renders nothing without `network.client`

The app makes no network requests, so the first cut of `entitlements.plist`
declared no network entitlement — and shipped **a black window**. It
launched, opened a window titled Kerotakis, kept a live
`com.apple.WebKit.WebContent` process, and answered Tauri's own
`asset_resolver` correctly for `/index.html`, the hashed JS and CSS, and
every lesson. It just never ran a line of the frontend: an IPC tripwire on
`engine_request` saw zero calls where an unsandboxed build saw `hello`,
`scene`, `species`, `grammar`.

macOS routes everything a WKWebView loads through WebKit's networking
process, **including a custom scheme served out of the binary**, and inside
the App Sandbox that process needs `com.apple.security.network.client` to
load anything at all. The entitlement is not about what the app talks to.
Nothing is logged when it is missing.

Two measurement mistakes made this take far longer than it should have, and
both are worth avoiding:

- Counting live `WebContent` processes to decide whether the webview works.
  There is one in every variant, including the broken ones — a live
  WebContent process is not a rendered page.
- Comparing a *bright-pixel* percentage against a guessed threshold. This
  is a dark UI: a fully rendered bench is only 2.5% above mid-brightness,
  so "2.49% painted" looked blank next to a threshold of 3%. Calibrate
  against a capture known to be good — the fixed build matches it exactly.

`tools/run-macos-local.sh` exists so this is caught before an upload rather
than after one.

The build is universal. An Apple-Silicon-only macOS build simply cannot be
installed on an Intel Mac and the store gives no warning about it.

> Comments in an entitlements file must not contain a literal `--` anywhere,
> not even mid-sentence. `codesign`'s AMFI parser enforces the XML rule that
> most tools ignore, and fails with a line number in a file it does not name.
> `plutil -lint` will not catch it.

### iOS

```console
$ tools/build-ios-appstore.sh --no-upload
$ ASC_APP_ID=<numeric app id> tools/build-ios-appstore.sh
```

Signing is **manual**, against the account's canonical Distribution
certificate and the `Kerotakis AppStore CI` profile. Automatic signing works
and is Tauri's documented recommendation, but on this account it is the
wrong trade twice: it manages iOS capability changes, and a capability change
invalidates *every* provisioning profile for the App ID; and it mints
certificates against a hard account-wide cap whose overflow Apple resolves by
revoking one that other apps depend on. Manual signing contacts Apple at
build time not at all.

Nothing in `tauri.conf.json` can express this. `bundle.iOS.developmentTeam`
does not reach the pbxproj, and the `IOS_CERTIFICATE` / `IOS_MOBILE_PROVISION`
environment variables only import a certificate into a temporary keychain.
A freshly generated project contains exactly one signing key
(`CODE_SIGN_IDENTITY = "iPhone Developer"`), so three of the four settings
have to be **inserted** — a sed-style replace matches nothing and reports
success. `tools/ios/patch-signing.py` does the insertion.

Order is enforced by the script and is not arbitrary:

```
tauri ios init
  -> patch-privacy.py + xcodegen generate     (regenerates the pbxproj)
  -> patch-signing.py                          (LAST; a regenerate discards it)
  -> tauri ios build --export-method app-store-connect
```

`PrivacyInfo.xcprivacy` goes in `gen/apple/` **root**, not in
`gen/apple/<app>_iOS/`: that directory is scanned wholesale by the target's
`sources`, so a file placed there is emitted twice ("Multiple commands
produce …"). It must end up at the built bundle's root, and the script
asserts that it did.

---

## App Store Connect

`tools/asc/` drives everything Apple exposes. The `.p8` key is never in this
repository — `client.py` looks in `.appstoreconnect/`, then
`~/.appstoreconnect/private_keys/`, then `ASC_API_KEY_P8_BASE64`.

```console
$ python3 tools/asc/client.py GET '/v1/apps?limit=200'      # raw API
$ python3 tools/asc/fetch-profile.py "Kerotakis AppStore CI" out.mobileprovision
$ python3 tools/asc/testflight.py ios --internal-only       # internal, no review
$ python3 tools/asc/testflight.py ios                       # external + submit
```

All the store and beta copy lives in `tools/asc/metadata.json`, so it is
reviewable in a diff rather than typed into a web form once and forgotten.

`testflight.py` is idempotent and its order is forced by Apple:

1. **Export compliance.** A `VALID` build is *"not in an internally testable
   state"* until `usesNonExemptEncryption` is set. This is **not** an
   external-only requirement — adding a build to any group fails without it.
   `ITSAppUsesNonExemptEncryption=false` in `Info.plist` answers it at build
   time; the PATCH is the belt to that braces.
2. **Beta review contact.** PATCH-only, exists as soon as the app does, and
   rejects the entire request without `contactPhone`.
3. **Beta app localisation** — description, feedback email, and the privacy
   policy URL. Beta App Review reads these.
4. **What to test**, per build, per locale.
5. **Groups.** Internal needs no Apple review at all and is live within
   minutes. External needs Beta App Review (same-day-ish, unlike full App
   Store review) and gets a public join link for free, minted on creation.
6. **Submit** — and then **re-read the submission**, because a 201 is not
   proof: a build that fails Apple's re-validation has its submission
   silently rolled back, and `processingState: INVALID` has no reason
   exposed anywhere in the API. The cause is only in the email Apple sends
   the account holder and on the build row in TestFlight's web UI.

### The two steps no API key can do

Both need a browser, and an Admin-role key is refused for both. They are the
only manual steps in the whole pipeline.

**1. Create the app record.** `POST /v1/apps` answers, verbatim:

```
403 FORBIDDEN_ERROR
The resource 'apps' does not allow 'CREATE'.
Allowed operations are: GET_COLLECTION, GET_INSTANCE, UPDATE
```

At <https://appstoreconnect.apple.com/apps> → **+** → **New App**:

| Field | Value |
|---|---|
| Platforms | **macOS and iOS** (tick both; one record serves both) |
| Name | Kerotakis |
| Primary language | English (U.S.) |
| Bundle ID | `com.crispstrobe.kerotakis` — already registered |
| SKU | `KEROTAKIS-001` |
| User access | Full Access |

Everything after that is API-driven again.

**2. The App Privacy label.** App Store Connect → the app → **App Privacy**
→ *Data Not Collected*. One click, and it is the truthful answer: no
account, no analytics, no identifiers, no network.

---

## Continuous delivery

Four workflows, and the split between them is deliberate: `ci.yml` gates
every push, the other three only run on a `v*` tag or an explicit dispatch,
because they produce artifacts that cost something to get wrong.

| Workflow | Trigger | Produces |
|---|---|---|
| `ci.yml` | every push and PR | the gate, and the Pages deploy of the PWA |
| `release.yml` | `v*` tag, or dispatch | `.dmg` / `.deb` / `.AppImage` / `.msi`, attached to a **draft** release |
| `appstore.yml` | `v*` tag, or dispatch | signed `.pkg` and `.ipa`, uploaded to App Store Connect |
| `android.yml` | `v*` tag, or dispatch | `.apk`, plus a `.aab` when a keystore is configured |

**The build logic is not in the workflows.** `appstore.yml` calls
`tools/build-macos-appstore.sh` and `tools/build-ios-appstore.sh` — the
same commands that run on a laptop — so a CI failure can be reproduced
without GitHub. The YAML's job is secrets, triggers, and artifacts.

**Every dispatch defaults to not shipping.** `release.yml` and
`appstore.yml` build, sign and verify but do not publish or upload unless
the ref is a tag; `android.yml` builds unsigned unless a keystore exists.
An upload cannot be undone and a build number cannot be reused, so the safe
thing has to be the default thing.

**No job is `continue-on-error`.** A mobile job carrying it reported green
for a month over a two-line link error (appstore.md). If a platform breaks,
these go red and name it.

`appstore.yml` also launches the built Mac app and checks it is not a blank
window, because a `.pkg` that renders nothing signs, validates and uploads
perfectly.

### Secrets

Nothing works from a fork: Actions secrets are not exposed to fork PRs,
which is the intended behaviour for a public repository.

| Secret | Used by | Where it comes from |
|---|---|---|
| `ASC_API_KEY_P8_BASE64` | appstore | `base64 -i AuthKey_<id>.p8` |
| `ASC_KEY_ID`, `ASC_ISSUER_ID`, `ASC_TEAM_ID` | appstore | account constants |
| `DIST_CERT_P12_BASE64`, `DIST_CERT_PASSWORD` | appstore | the canonical Distribution `.p12` — **never mint a new certificate**; Apple's cap is enforced by revoking one another app depends on |
| `IOS_PROFILE_BASE64` | appstore (iOS) | `profileContent` of `Kerotakis AppStore CI` |
| `MAC_PROFILE_BASE64` | appstore (macOS) | `profileContent` of `Kerotakis Mac App Store` |
| `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD` | android | `keytool -genkey -v -keystore kerotakis.jks -keyalg RSA -keysize 2048 -validity 10000 -alias kerotakis` |

The profiles are fetched from the API at build time when no secret is set,
so a machine with the `.p8` needs neither `*_PROFILE_BASE64`.

### What Android still needs

The workflow generates the project, cross-compiles IPhreeqc for all four
ABIs, asserts the manifest stays permissionless, and produces artifacts.
**Play upload is not automated**, and that is a decision rather than an
omission: the first release of an app has to be created in the Play Console
by hand, and automating later uploads needs a service-account JSON that
does not exist yet. Wiring it now would be a step that looks done.

## What a human still decides

Submitting for **full App Store review** is a product decision, not a build
step, and full review additionally wants screenshots (iPhone 6.9" and iPad
13"; the Simulator can produce real ones, no device needed). External
TestFlight needs neither.
