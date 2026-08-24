# Kerotakis app (GUI-010)

The bench UI: one Svelte + TypeScript application, shipped as the web PWA
and inside the Tauri shells (ROADMAP-GUI.md). It talks to the engine only
through the EngineHost protocol (PROTOCOL.md); the web transport is a
module worker running `kerotakis-wasm`.

```
npm install
npm test          # host protocol unit tests (vitest)
npm run licences  # every installed package against the licence allowlist
npm run dev       # dev server — needs the engine, see below
npm run build     # static files in dist/
```

## Getting the engine

The worker loads the wasm-bindgen output from `public/engine/`:

```
tools/build-web.sh                       # from the repo root
cp "$(cargo metadata --format-version 1 --no-deps | \
      python3 -c 'import sys,json;print(json.load(sys.stdin)["target_directory"])')"/web/* \
   web/app/public/engine/
```

Without it the app starts, says the engine is not loaded, and refuses
chemistry rather than faking any — the honest degradation.

The legacy console page (`web/index.html`) is unchanged and remains the
deployed PWA until this app reaches parity (GUI-011…015); it then survives
as the "console" view.
