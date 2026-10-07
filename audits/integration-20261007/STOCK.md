# Stock ownership integration

This slice depends on PR #756 and ports accepted `stock.rs` ownership plus
the minimal Bench serialization/refusal adapters. It carries conservative
precision controls, opt-in compensated controls and finite-precision boundary
controls. Original test bytes are retained in adjacent `.rs.txt` snapshots;
compiled copies have only rustfmt changes. [Hashes and mapping](stock-port.json).

It changes default stock draws from main's permissive absolute slack to the
already-audited conservative relative debit certificate. Invalid draws and
inaccurate or swallowed debits refuse; opt-in compensated draws retain a
bounded low component. Refusals are atomic inside the ledger. This does not
prove every existing Bench operation rolls back after later solver failure;
transaction integration remains AUD-00/TXN-01 work.

[The ownership decision](../../docs/chemistry-amount-ownership-decision-20261007.md)
states the public API and save boundary. Hosted integration validation is
pending; original branch evidence does not establish this port's acceptance.
