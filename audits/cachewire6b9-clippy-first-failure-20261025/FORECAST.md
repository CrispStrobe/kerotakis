# Strict lint stop on the cache transport successor

Full CI40213984686/source6b928abb stops both native platforms and cache-only Wasm linting at unused `Deserialize` and `Serialize` trait imports in `aqueous.rs:24`. Runtime compilation already succeeds in the audited CLI producer, but the failed jobs reach no native tests. Preserve raw log hashes and do not count compiler setup as physical acceptance.

Remove only the unused import line. Keep the actual custom serializers/deserializers, every transport/bridge/lesson assertion, all thresholds and strict lint commands unchanged. The remaining source bytes must compare exactly with6b928abb; current full platform/preflight/fleet/Wasm acceptance must still run on the successor. Active work is not cancelled for optional changes; this is the demonstrated required compiler repair.
