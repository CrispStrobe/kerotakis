# Actual corrected-source cancellation control

Source `f3517e089018d661d52e6c6afd456eb254e54c3c` changed only the three
job-level aggregate conditions. The separately frozen scheduler control
[run39120053704](https://github.com/CrispStrobe/kerotakis/actions/runs/39120053704)
received one normal cancellation. It became terminal with all three aggregates
cancelled and no queued/running job left, without a force-cancel request.
No physics recording step ran; this is not another corpus execution.

Read [design](frozen-design.json), [actual receipt](ACCEPTED-scheduler-control.json),
[public scheduler observations](scheduler-observations.json) and
[byte manifest](sha256.json). Original complete API records are preserved with
their hashes; the public projection retains source, job IDs/states and recording
step evidence without private operating data.

This validates the cancellation behavior only. The run conclusion is cancelled,
not a successful chemistry workflow. PR830 still requires its separate ordinary
CI/full-fleet acceptance and normal protected integration. No scientific selection,
assertion waiver or branch-protection change is claimed.
