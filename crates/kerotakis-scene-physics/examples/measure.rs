use std::time::Instant;

use kerotakis_core::VesselId;
use kerotakis_scene_physics::{
    replay, PhysicsAction, PhysicsReplayV1, QuantizedPose, Spawn, TimedInput,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Serialize)]
struct Probe {
    trace_sha256: String,
    step_ms: Vec<f64>,
}

fn main() {
    let initial = (0..20)
        .map(|object| Spawn {
            object,
            item: if object % 2 == 0 {
                "beaker"
            } else {
                "test_tube"
            }
            .into(),
            vessel: Some(VesselId(object as usize)),
            pose: QuantizedPose::from_metres(
                (object % 5) as f32 * 0.12 - 0.24,
                0.3 + (object / 5) as f32 * 0.18,
                0.0,
            )
            .unwrap(),
        })
        .collect();
    let spec = PhysicsReplayV1 {
        version: 1,
        catalog_version: 1,
        replay_seed: 71,
        ticks: 360,
        initial,
        inputs: vec![TimedInput {
            tick: 0,
            sequence: 0,
            action: PhysicsAction::Nudge {
                object: 0,
                impulse_micronewton_seconds: [80_000, 20_000],
            },
        }],
    };
    let mut step_ms = Vec::with_capacity(8);
    let mut outcome = None;
    for _ in 0..8 {
        let start = Instant::now();
        let current = replay(&spec).unwrap();
        step_ms.push(start.elapsed().as_secs_f64() * 1000.0 / spec.ticks as f64);
        outcome = Some(current);
    }
    let trace = serde_json::to_vec(&outcome.unwrap()).unwrap();
    let probe = Probe {
        trace_sha256: format!("{:x}", Sha256::digest(trace)),
        step_ms,
    };
    println!("{}", serde_json::to_string(&probe).unwrap());
}
