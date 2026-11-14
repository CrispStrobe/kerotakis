# PR882: completed build and preflight, partial acceptance

The existing [audited CLI build](https://github.com/CrispStrobe/kerotakis/actions/runs/43603851962/job/132474143587) and [API preflight job](https://github.com/CrispStrobe/kerotakis/actions/runs/43603851911/job/132474142501) completed successfully. No duplicate build, dispatch or replay was started to collect these results.

Both checked out GitHub's synthetic PR merge `0a7391f145ab6d7ca2c4812974570768df589bb7`, with parents the reviewed base `00784e5f` and head `439bbe56`. Independent Git-object review shows its tree is exactly the reviewed head's tree, `264bed2b00d67ca27baa84b46aea106503947405`, with an empty tree diff. This is a PR validation merge object, not integration into main.

The actual release binary SHA256 is `e7293591f625c1b32df6ed2c55fb8430d89a5c060af61d739a9b7b3805dc87b2`. Its included checksum matches the downloaded artifact bytes. The actual producer Cargo.lock SHA256 is `c2d28fbc40c3e864abc011d7c3731cf449a2a13737a3c7e4ecd89d8b5c238fd7`; the lock and submodule inventory are retained here. The build command was `cargo build --release -p kerotakis-cli`, without `--locked`. No source-committed lock or equality with older producer locks is claimed.

The preflight log reports 37 passing auth, shared-admission and native-sandbox policy tests, plus standalone client and service-worker controls. These results are useful source evidence but do not establish complete platform/native/fleet or product acceptance. PR882 remains draft until its actual remaining acceptance is earned. Existing downstream fleet jobs remain authoritative; do not duplicate or bypass them.

This artifact earns zero original catalog entries. Its new checksum must not replace missing executable checksums in historical source-bound qualifications. Those original script, source, lock, output and identity-limit bindings remain unchanged.
