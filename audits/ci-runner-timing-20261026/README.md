# Accepted-run CI timing observations

[CI40280911837](https://github.com/CrispStrobe/kerotakis/actions/runs/40280911837) and [fleet40280911865](https://github.com/CrispStrobe/kerotakis/actions/runs/40280911865) are the accepted historical `0d4dee67` runs. [TIMINGS.json](TIMINGS.json) records actual API job/step durations and raw metadata hashes. These are sums of overlapping wall/queue intervals, not CPU time, billing or critical-path sums.

Full preflight takes1456s (1405s in its script); native macOS1377s and Ubuntu1010s. Their workspace tests take974s and736s respectively. The fleet has33 successful jobs:570s summed job wall and883s summed queue intervals. Its single CLI build takes218s; one thermochemical family queues153s and executes9s, including its1s experiment-recording step.

These measurements justify the [bounded acceleration scopes](../../docs/ci-runner-acceleration-20261026.md). They do not predict an exact speedup or authorize skipping checks. The current diagnostic candidate remains unchanged while its own required runs wait for runners.
