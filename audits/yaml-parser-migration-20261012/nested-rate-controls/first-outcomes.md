# Nested-rate controls: first outcomes

## First execution

After freeze commit `c1fe0ef8` was pushed, the unchanged candidate completed
**4 pass / 8 fail**. The [independent receipt](../nested-rate-first-independent-review-1acdda46.json)
verifies all twelve raw captures, source, actual lock and binary. Every unknown
key was accepted with no diagnostic and the positive counterpart's complete
summary. `B: 1` visibly becomes default `b=0`. Acceptance of Troe `t2` is observed;
its omission is source-inferred because inspect omits Troe parameters. Three
valid cases and missing-required-`Ea` pass. The original YAML parser was not
executed on these twelve controls, so this is no migration-regression claim.

The manifest and collector remain unchanged. A separate narrow admission
repair and new source binding are required; no repaired-source or integration
acceptance is claimed.
