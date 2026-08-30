//! BRD-071 bounded decision spike: deterministic two-dimensional scene physics.
//!
//! This crate is deliberately outside `kerotakis-core`. It produces poses and
//! BRD-070 collision proposals; it cannot mutate chemistry or invent events.

use std::collections::{BTreeMap, BTreeSet};

use kerotakis_core::authority::{CollisionProposal, ReplaySeed, SpillDestination};
use kerotakis_core::VesselId;
use rapier2d::prelude::*;
use serde::{Deserialize, Serialize};

pub const REPLAY_VERSION: u32 = 1;
pub const CATALOG_VERSION: u32 = 1;
pub const TICKS_PER_SECOND: u32 = 120;
pub const POSITION_QUANTA_PER_METRE: f32 = 1_000_000.0;
pub const MAX_REPLAY_TICKS: u64 = 36_000;
pub const MAX_OBJECTS: usize = 64;
pub const MAX_INPUTS: usize = 4_096;

pub type ObjectId = u32;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "shape", rename_all = "snake_case")]
pub enum ColliderSpec {
    Box {
        half_width_m: f32,
        half_height_m: f32,
    },
    CapsuleY {
        half_segment_m: f32,
        radius_m: f32,
    },
    /// Three rectangles: base, left wall, right wall. This keeps a vessel
    /// hollow so ports and future insertions are not blocked by a fake hull.
    OpenVessel {
        half_width_m: f32,
        height_m: f32,
        wall_m: f32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortSpec {
    pub id: String,
    pub x_m: f32,
    pub y_m: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicsItemSpec {
    pub id: String,
    pub model_version: u32,
    pub collider: ColliderSpec,
    pub ports: Vec<PortSpec>,
    pub mass_kg: f32,
    pub friction: f32,
    pub restitution: f32,
    pub break_impulse_ns: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PhysicsError {
    #[error("catalog item `{0}` is unknown")]
    UnknownItem(String),
    #[error("object id {0} occurs more than once")]
    DuplicateObject(ObjectId),
    #[error("invalid catalog item `{0}`")]
    InvalidCatalog(String),
    #[error("invalid or unsorted replay input")]
    InvalidInput,
}

impl PhysicsItemSpec {
    pub fn validate(&self) -> Result<(), PhysicsError> {
        let finite_positive = |x: f32| x.is_finite() && x > 0.0;
        let (shape_ok, half_width, vertical_extent) = match self.collider {
            ColliderSpec::Box {
                half_width_m,
                half_height_m,
            } => (
                finite_positive(half_width_m) && finite_positive(half_height_m),
                half_width_m,
                half_height_m,
            ),
            ColliderSpec::CapsuleY {
                half_segment_m,
                radius_m,
            } => (
                finite_positive(half_segment_m) && finite_positive(radius_m),
                radius_m,
                half_segment_m + radius_m,
            ),
            ColliderSpec::OpenVessel {
                half_width_m,
                height_m,
                wall_m,
            } => (
                finite_positive(half_width_m)
                    && finite_positive(height_m)
                    && finite_positive(wall_m)
                    && wall_m < half_width_m,
                half_width_m,
                height_m,
            ),
        };
        let scalars_ok = finite_positive(self.mass_kg)
            && self.friction.is_finite()
            && self.friction >= 0.0
            && self.restitution.is_finite()
            && (0.0..=1.0).contains(&self.restitution)
            && self.break_impulse_ns.is_none_or(finite_positive);
        let unique_ports = self
            .ports
            .iter()
            .map(|p| p.id.as_str())
            .collect::<BTreeSet<_>>()
            .len()
            == self.ports.len();
        let ports_ok = self.ports.iter().all(|p| {
            !p.id.is_empty()
                && p.x_m.is_finite()
                && p.y_m.is_finite()
                && p.x_m.abs() <= half_width
                && p.y_m.abs() <= vertical_extent
        });
        if self.id.is_empty()
            || self.model_version == 0
            || !shape_ok
            || !scalars_ok
            || !unique_ports
            || !ports_ok
        {
            return Err(PhysicsError::InvalidCatalog(self.id.clone()));
        }
        Ok(())
    }
}

/// Executable prototype subset of the apparatus catalog. Dimensions are
/// explicit decision-spike assumptions, not scientific glassware claims.
pub fn prototype_catalog() -> BTreeMap<String, PhysicsItemSpec> {
    let mut catalog = BTreeMap::new();
    let entries = [
        (
            "beaker",
            ColliderSpec::OpenVessel {
                half_width_m: 0.035,
                height_m: 0.09,
                wall_m: 0.004,
            },
            0.10,
        ),
        (
            "test_tube",
            ColliderSpec::CapsuleY {
                half_segment_m: 0.07,
                radius_m: 0.012,
            },
            0.025,
        ),
        (
            "conical_flask",
            ColliderSpec::OpenVessel {
                half_width_m: 0.045,
                height_m: 0.12,
                wall_m: 0.004,
            },
            0.14,
        ),
        (
            "round_bottom_flask",
            ColliderSpec::CapsuleY {
                half_segment_m: 0.025,
                radius_m: 0.045,
            },
            0.16,
        ),
        (
            "rack",
            ColliderSpec::Box {
                half_width_m: 0.12,
                half_height_m: 0.02,
            },
            0.40,
        ),
        (
            "tray",
            ColliderSpec::Box {
                half_width_m: 0.20,
                half_height_m: 0.012,
            },
            0.30,
        ),
    ];
    for (id, collider, mass_kg) in entries {
        let port_y = match collider {
            ColliderSpec::Box { half_height_m, .. } => half_height_m,
            ColliderSpec::CapsuleY {
                half_segment_m,
                radius_m,
            } => half_segment_m + radius_m,
            ColliderSpec::OpenVessel { height_m, .. } => height_m,
        };
        catalog.insert(
            id.to_owned(),
            PhysicsItemSpec {
                id: id.to_owned(),
                model_version: 1,
                collider,
                ports: vec![PortSpec {
                    id: "top".into(),
                    x_m: 0.0,
                    y_m: port_y,
                }],
                mass_kg,
                friction: 0.45,
                restitution: 0.08,
                break_impulse_ns: Some(0.8),
            },
        );
    }
    catalog
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct QuantizedPose {
    pub x_um: i32,
    pub y_um: i32,
    pub angle_urad: i32,
}

impl QuantizedPose {
    pub fn from_metres(x: f32, y: f32, angle: f32) -> Result<Self, PhysicsError> {
        if !x.is_finite() || !y.is_finite() || !angle.is_finite() {
            return Err(PhysicsError::InvalidInput);
        }
        Ok(Self {
            x_um: (x * POSITION_QUANTA_PER_METRE).round() as i32,
            y_um: (y * POSITION_QUANTA_PER_METRE).round() as i32,
            angle_urad: (angle * POSITION_QUANTA_PER_METRE).round() as i32,
        })
    }
    fn floats(self) -> (f32, f32, f32) {
        (
            self.x_um as f32 / POSITION_QUANTA_PER_METRE,
            self.y_um as f32 / POSITION_QUANTA_PER_METRE,
            self.angle_urad as f32 / POSITION_QUANTA_PER_METRE,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spawn {
    pub object: ObjectId,
    pub item: String,
    pub vessel: Option<VesselId>,
    pub pose: QuantizedPose,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum PhysicsAction {
    Drop {
        object: ObjectId,
        linear_velocity_um_s: [i32; 2],
        angular_velocity_urad_s: i32,
    },
    Nudge {
        object: ObjectId,
        impulse_micronewton_seconds: [i32; 2],
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimedInput {
    pub tick: u64,
    pub sequence: u32,
    pub action: PhysicsAction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicsReplayV1 {
    pub version: u32,
    pub catalog_version: u32,
    pub replay_seed: ReplaySeed,
    pub ticks: u64,
    pub initial: Vec<Spawn>,
    pub inputs: Vec<TimedInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectPose {
    pub object: ObjectId,
    pub pose: QuantizedPose,
    pub sleeping: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayOutcome {
    pub tick: u64,
    pub poses: Vec<ObjectPose>,
    pub collisions: Vec<CollisionProposal>,
}

struct BodyMeta {
    handle: RigidBodyHandle,
    collider: ColliderHandle,
    vessel: Option<VesselId>,
}

fn collider(spec: &ColliderSpec) -> ColliderBuilder {
    match *spec {
        ColliderSpec::Box {
            half_width_m,
            half_height_m,
        } => ColliderBuilder::cuboid(half_width_m, half_height_m),
        ColliderSpec::CapsuleY {
            half_segment_m,
            radius_m,
        } => ColliderBuilder::capsule_y(half_segment_m, radius_m),
        ColliderSpec::OpenVessel {
            half_width_m,
            height_m,
            wall_m,
        } => {
            let side_half_h = height_m / 2.0;
            ColliderBuilder::compound(vec![
                (
                    Pose::from_translation(Vector::new(0.0, wall_m / 2.0)),
                    SharedShape::cuboid(half_width_m, wall_m / 2.0),
                ),
                (
                    Pose::from_translation(Vector::new(-half_width_m + wall_m / 2.0, side_half_h)),
                    SharedShape::cuboid(wall_m / 2.0, side_half_h),
                ),
                (
                    Pose::from_translation(Vector::new(half_width_m - wall_m / 2.0, side_half_h)),
                    SharedShape::cuboid(wall_m / 2.0, side_half_h),
                ),
            ])
        }
    }
}

pub fn replay(spec: &PhysicsReplayV1) -> Result<ReplayOutcome, PhysicsError> {
    if spec.version != REPLAY_VERSION
        || spec.catalog_version != CATALOG_VERSION
        || spec.ticks == 0
        || spec.ticks > MAX_REPLAY_TICKS
        || spec.initial.len() > MAX_OBJECTS
        || spec.inputs.len() > MAX_INPUTS
    {
        return Err(PhysicsError::InvalidInput);
    }
    if spec
        .inputs
        .windows(2)
        .any(|w| (w[0].tick, w[0].sequence) >= (w[1].tick, w[1].sequence))
        || spec.inputs.iter().any(|i| i.tick >= spec.ticks)
    {
        return Err(PhysicsError::InvalidInput);
    }
    let catalog = prototype_catalog();
    for item in catalog.values() {
        item.validate()?;
    }
    let mut world = PhysicsWorld::new();
    world.integration_parameters.dt = 1.0 / TICKS_PER_SECOND as f32;
    world.integration_parameters.max_ccd_substeps = 4;
    world.integration_parameters.num_solver_iterations = 8;
    world.gravity = Vector::new(0.0, -9.81);
    world.insert(
        RigidBodyBuilder::fixed().translation(Vector::new(0.0, -0.015)),
        // The spike's support is intentionally wide: edge/floor routing is a
        // separate drop-corpus case, while this world proves support CCD.
        ColliderBuilder::cuboid(50.0, 0.015),
    );
    let mut bodies = BTreeMap::<ObjectId, BodyMeta>::new();
    let mut initial = spec.initial.clone();
    initial.sort_by_key(|s| s.object);
    for spawn in initial {
        if bodies.contains_key(&spawn.object) {
            return Err(PhysicsError::DuplicateObject(spawn.object));
        }
        let item = catalog
            .get(&spawn.item)
            .ok_or_else(|| PhysicsError::UnknownItem(spawn.item.clone()))?;
        let (x, y, angle) = spawn.pose.floats();
        let rb = RigidBodyBuilder::dynamic()
            .translation(Vector::new(x, y))
            .rotation(angle)
            .additional_mass(item.mass_kg)
            .ccd_enabled(true)
            .can_sleep(true);
        let cb = collider(&item.collider)
            .friction(item.friction)
            .restitution(item.restitution);
        let (handle, collider) = world.insert(rb, cb);
        bodies.insert(
            spawn.object,
            BodyMeta {
                handle,
                collider,
                vessel: spawn.vessel,
            },
        );
    }
    let mut next_input = 0;
    let mut maximum_impulses = BTreeMap::<usize, f32>::new();
    for tick in 0..spec.ticks {
        while next_input < spec.inputs.len() && spec.inputs[next_input].tick == tick {
            match &spec.inputs[next_input].action {
                PhysicsAction::Drop {
                    object,
                    linear_velocity_um_s,
                    angular_velocity_urad_s,
                } => {
                    let meta = bodies.get(object).ok_or(PhysicsError::InvalidInput)?;
                    let body = world
                        .bodies
                        .get_mut(meta.handle)
                        .ok_or(PhysicsError::InvalidInput)?;
                    body.set_linvel(
                        Vector::new(
                            linear_velocity_um_s[0] as f32 / 1e6,
                            linear_velocity_um_s[1] as f32 / 1e6,
                        ),
                        true,
                    );
                    body.set_angvel(*angular_velocity_urad_s as f32 / 1e6, true);
                }
                PhysicsAction::Nudge {
                    object,
                    impulse_micronewton_seconds,
                } => {
                    let meta = bodies.get(object).ok_or(PhysicsError::InvalidInput)?;
                    world
                        .bodies
                        .get_mut(meta.handle)
                        .ok_or(PhysicsError::InvalidInput)?
                        .apply_impulse(
                            Vector::new(
                                impulse_micronewton_seconds[0] as f32 / 1e6,
                                impulse_micronewton_seconds[1] as f32 / 1e6,
                            ),
                            true,
                        );
                }
            }
            next_input += 1;
        }
        world.step();
        for meta in bodies.values() {
            let Some(vessel) = meta.vessel else { continue };
            for pair in world.narrow_phase.contact_pairs_with(meta.collider) {
                let impulse = pair.total_impulse_magnitude();
                if impulse > 0.0 {
                    maximum_impulses
                        .entry(vessel.0)
                        .and_modify(|maximum| *maximum = maximum.max(impulse))
                        .or_insert(impulse);
                }
            }
        }
    }
    let poses = bodies
        .into_iter()
        .map(|(object, meta)| {
            let body = &world.bodies[meta.handle];
            let p = body.translation();
            Ok(ObjectPose {
                object,
                pose: QuantizedPose::from_metres(p.x, p.y, body.rotation().angle())?,
                sleeping: body.is_sleeping(),
            })
        })
        .collect::<Result<Vec<_>, PhysicsError>>()?;
    let collisions = maximum_impulses
        .into_iter()
        .map(|(vessel, impulse_ns)| CollisionProposal {
            vessel: VesselId(vessel),
            impulse_ns: f64::from(impulse_ns),
            destination_if_broken: SpillDestination::Bench {
                zone: "react".into(),
            },
            replay_seed: spec.replay_seed,
        })
        .collect();
    Ok(ReplayOutcome {
        tick: spec.ticks,
        poses,
        collisions,
    })
}

/// A retained wasm export for honest candidate-payload measurement. The
/// return value depends on running Rapier, preventing dead-code elimination.
#[no_mangle]
pub extern "C" fn kerotakis_brd071_wasm_probe() -> i32 {
    let spec = PhysicsReplayV1 {
        version: REPLAY_VERSION,
        catalog_version: CATALOG_VERSION,
        replay_seed: 71,
        ticks: 120,
        initial: vec![Spawn {
            object: 1,
            item: "beaker".into(),
            vessel: Some(VesselId(0)),
            pose: QuantizedPose {
                x_um: 0,
                y_um: 500_000,
                angle_urad: 0,
            },
        }],
        inputs: Vec::new(),
    };
    replay(&spec).map_or(-1, |outcome| outcome.poses[0].pose.y_um)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kerotakis_core::authority::SceneProposal;
    use sha2::{Digest, Sha256};

    fn drop_replay(item: &str, x: f32, y: f32, angle: f32, vy: i32) -> PhysicsReplayV1 {
        PhysicsReplayV1 {
            version: 1,
            catalog_version: 1,
            replay_seed: 71,
            ticks: 360,
            initial: vec![Spawn {
                object: 7,
                item: item.into(),
                vessel: Some(VesselId(0)),
                pose: QuantizedPose::from_metres(x, y, angle).unwrap(),
            }],
            inputs: vec![TimedInput {
                tick: 0,
                sequence: 0,
                action: PhysicsAction::Drop {
                    object: 7,
                    linear_velocity_um_s: [0, vy],
                    angular_velocity_urad_s: 500_000,
                },
            }],
        }
    }

    #[test]
    fn catalog_is_valid_and_hollow_shapes_are_explicit() {
        let catalog = prototype_catalog();
        assert!(catalog.values().all(|item| item.validate().is_ok()));
        assert!(matches!(
            catalog["beaker"].collider,
            ColliderSpec::OpenVessel { .. }
        ));
    }

    #[test]
    fn exact_replay_and_serialized_input_are_identical() {
        let script = drop_replay("beaker", 0.0, 1.2, 0.2, -2_000_000);
        let first = replay(&script).unwrap();
        let decoded: PhysicsReplayV1 =
            serde_json::from_str(&serde_json::to_string(&script).unwrap()).unwrap();
        assert_eq!(first, replay(&decoded).unwrap());
        assert!(first.poses[0].pose.y_um >= 0);
    }

    #[test]
    fn decision_probe_matches_cross_host_golden() {
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
            version: REPLAY_VERSION,
            catalog_version: CATALOG_VERSION,
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
        let trace = serde_json::to_vec(&replay(&spec).unwrap()).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(trace)),
            "efb244defa19e5fcf9dacc8f0ca26801458dbe85b5e31f223dc09e91330ce0ea"
        );
    }

    #[test]
    fn keyboard_and_touch_quantization_compile_to_the_same_input() {
        let touch = QuantizedPose::from_metres(0.123_456_4, 0.8, 0.0).unwrap();
        let keyboard = QuantizedPose {
            x_um: 123_456,
            y_um: 800_000,
            angle_urad: 0,
        };
        assert_eq!(touch, keyboard);
    }

    #[test]
    fn collision_is_only_an_authority_proposal() {
        let outcome = replay(&drop_replay("test_tube", 0.0, 0.8, 1.2, -4_000_000)).unwrap();
        let proposal = outcome
            .collisions
            .first()
            .cloned()
            .expect("drop contacts bench");
        assert!(proposal.impulse_ns > 0.0);
        assert!(matches!(
            SceneProposal::Collision(proposal),
            SceneProposal::Collision(_)
        ));
    }

    #[test]
    fn ccd_drop_corpus_does_not_tunnel_through_bench() {
        for item in [
            "beaker",
            "test_tube",
            "conical_flask",
            "round_bottom_flask",
            "rack",
            "tray",
        ] {
            for (height, angle, speed) in [
                (0.2, 0.0, -2_000_000),
                (0.8, 0.7, -8_000_000),
                (1.5, 1.4, -20_000_000),
            ] {
                let out = replay(&drop_replay(item, 0.0, height, angle, speed)).unwrap();
                assert!(
                    out.poses[0].pose.y_um > -100_000,
                    "{item} tunneled: {:?}",
                    out.poses[0]
                );
                assert!(!out.collisions.is_empty(), "{item} never contacted bench");
            }
        }
    }

    #[test]
    fn stack_and_tip_are_stable_and_repeatable() {
        let mut script = drop_replay("tray", 0.0, 0.4, 0.0, 0);
        script.initial.push(Spawn {
            object: 2,
            item: "beaker".into(),
            vessel: None,
            pose: QuantizedPose::from_metres(0.01, 0.65, 0.35).unwrap(),
        });
        assert_eq!(replay(&script).unwrap(), replay(&script).unwrap());
    }

    #[test]
    fn malformed_inputs_refuse() {
        let mut script = drop_replay("beaker", 0.0, 1.0, 0.0, 0);
        script.inputs.push(script.inputs[0].clone());
        assert_eq!(replay(&script), Err(PhysicsError::InvalidInput));
    }

    #[test]
    fn untrusted_replay_size_is_bounded() {
        let mut script = drop_replay("beaker", 0.0, 1.0, 0.0, 0);
        script.ticks = MAX_REPLAY_TICKS + 1;
        assert_eq!(replay(&script), Err(PhysicsError::InvalidInput));
        script.ticks = 0;
        assert_eq!(replay(&script), Err(PhysicsError::InvalidInput));
        script.ticks = 10;
        script.initial = vec![script.initial[0].clone(); MAX_OBJECTS + 1];
        assert_eq!(replay(&script), Err(PhysicsError::InvalidInput));
    }
}
