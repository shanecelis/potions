use std::collections::{HashMap, HashSet};
use super::{Vial, Object, VialLoc};
use bevy_math::Vec2;
use std::f32::consts::PI;
use crate::constant::*;

use rapier2d::prelude::*;

pub struct VialPhysics {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    // objects: Vec<RigidBodyHandle>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
    objects: HashMap<u128, RigidBodyHandle>,
    physics_hooks: (),
    event_handler: (),
}


impl VialPhysics {
    pub fn new(vial: &Vial) -> Self {

        let rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();
        // Convert from mm to m.
        let vial_size_m = vial.size * MM_TO_M;

        let wall_width_m = 10.0 * MM_TO_M;
        /* Create the ground. */
        let collider = ColliderBuilder::cuboid(vial_size_m.x, wall_width_m)
            .translation(vector![0.0, - wall_width_m])
            .build();
        collider_set.insert(collider);

        // Walls

        // Left wall
        let collider = ColliderBuilder::cuboid(wall_width_m, vial_size_m.y)
            .translation(vector![-wall_width_m,
                                 vial_size_m.y / 2.0 - wall_width_m])
            .build();
        collider_set.insert(collider);

        // Right wall
        let collider = ColliderBuilder::cuboid(wall_width_m, vial_size_m.y)
            .translation(vector![vial_size_m.x + wall_width_m,
                                 vial_size_m.y / 2.0 - wall_width_m])
            .build();
        collider_set.insert(collider);

        /* Create other structures necessary for the simulation. */
        let integration_parameters = IntegrationParameters {
            max_ccd_substeps: 1000,
            ..Default::default()
        };

        let physics_pipeline = PhysicsPipeline::new();
        let island_manager = IslandManager::new();
        let broad_phase = DefaultBroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let impulse_joint_set = ImpulseJointSet::new();
        let multibody_joint_set = MultibodyJointSet::new();
        let ccd_solver = CCDSolver::new();
        let query_pipeline = QueryPipeline::new();
        let physics_hooks = ();
        let event_handler = ();
        let mut vial_physics = VialPhysics {
            rigid_body_set,
            collider_set,
            objects: HashMap::new(),
            integration_parameters,
            physics_pipeline,
            island_manager,
            broad_phase,
            narrow_phase,
            impulse_joint_set,
            multibody_joint_set,
            ccd_solver,
            query_pipeline,
            physics_hooks,
            event_handler,
        };

        for obj in &vial.objects {
            vial_physics.insert(obj);
        }
        vial_physics
    }

    pub fn insert(&mut self, obj: &Object) -> bool {
        if !self.objects.contains_key(&obj.id) {
            // dbg!(obj);
            let pos_m = obj.pos * MM_TO_M;
            // dbg!(pos_m);
            let mut rigid_body = RigidBodyBuilder::dynamic()
                .translation(vector![pos_m.x, pos_m.y])
                .ccd_enabled(true)
                .build();
            rigid_body.user_data = obj.id as u128;
            let mut collider = ColliderBuilder::ball(obj.size as f32 * MM_TO_M)
                // .restitution(0.7)
                .restitution(0.1)
                .build();
            // collider.set_density(GLASS_DENSITY); // glass
            collider.set_density(WATER_DENSITY); // glass
            collider.user_data = obj.id as u128;
            let handle = self.rigid_body_set.insert(rigid_body);
            self.objects.insert(obj.id, handle);
            self.collider_set.insert_with_parent(collider, handle, &mut self.rigid_body_set);
            true
        } else {
            false
        }
    }

    pub fn add_buoyancy_forces(&mut self, vial: &Vial) {

        let mut map: HashMap<u128, &Object> = vial.objects.iter().map(|o| (o.id, o)).collect();
        for (handle, rigid_body) in self.rigid_body_set.iter_mut() {
            rigid_body.reset_forces(true);
            let p: &Vector<Real> = rigid_body.translation();
            // let p = r.translation;
            let v = rigid_body.velocity_at_point(&Point::from(*p));
            let pos_mm = Vec2::new(p.x * M_TO_MM, p.y * M_TO_MM);
            match vial.in_layer(pos_mm) {
                Some(VialLoc::Layer { index: i, height: layer_height }) => 
                    if let Some(obj) = map.remove(&rigid_body.user_data) {
                        let s = obj.size as f32 * MM_TO_M;
                        if let Some(buoyancy_area) = circle_buoyancy_area(s, p, &vector![0.0, 1.0], layer_height * MM_TO_M) {
                            let buoyancy_force = vector![0.0, (buoyancy_area * GRAVITY * WATER_DENSITY) as f32];
                            // rigid_body.add_force(buoyancy_force, false);
                            // rigid_body.set_linear_damping(200.0);
                            // rigid_body.set_linear_damping(dbg!(drag_force(WATER_DENSITY, 1.0, 2.0 * s, CIRCLE_DRAG)));
                            let fudge = 1.0;
                            rigid_body.set_linear_damping(fudge * drag_force(WATER_DENSITY, v.y.abs(), 2.0 * s, CIRCLE_DRAG));
                            // rigid_body.set_linear_damping(fudge * drag_force(WATER_DENSITY, v.y * v.y, 2.0 * s, CIRCLE_DRAG));
                        }
                    },
                Some(VialLoc::Top { height: layer_height }) => {
                    if let Some(obj) = map.remove(&rigid_body.user_data) {
                        let s = obj.size as f32 * MM_TO_M;
                    if let Some(buoyancy_area) = circle_buoyancy_area(s, p, &vector![0.0, 1.0], layer_height * MM_TO_M) {
                        assert!(buoyancy_area <= 0.01, "{}", buoyancy_area);
                    }
                    }
                }
                _ => {
                let m = rigid_body.mass();
                rigid_body.add_force(vector![0.0, -m* GRAVITY], false);
                }
            }
        }
    }

    pub fn step(&mut self, dt: Real) {
        // let gravity = vector![0.0, -GRAVITY];
        let gravity = vector![0.0, 0.0];
        // let mut accum = 0.0;
        /* Run the game loop, stepping the simulation once per frame. */
        // self.integration_parameters.dt = dt;
        // while accum < dt {
            self.physics_pipeline.step(
                &gravity,
                &self.integration_parameters,
                &mut self.island_manager,
                &mut self.broad_phase,
                &mut self.narrow_phase,
                &mut self.rigid_body_set,
                &mut self.collider_set,
                &mut self.impulse_joint_set,
                &mut self.multibody_joint_set,
                &mut self.ccd_solver,
                Some(&mut self.query_pipeline),
                &self.physics_hooks,
                &self.event_handler,
            );
        //     accum += self.integration_parameters.dt;
        // }
    }

    pub fn project(&mut self, vial: &mut Vial) {
        let mut map: HashMap<u128, &mut Object> = vial.objects.iter_mut().map(|o| (o.id, o)).collect();
        let mut remove_handles = vec![];
        for (handle, rigid_body) in self.rigid_body_set.iter() {
            if let Some(obj) = map.remove(&rigid_body.user_data) {
                let p = rigid_body.translation();
                obj.pos.x = p.x * M_TO_MM;
                obj.pos.y = p.y * M_TO_MM;
                // dbg!(obj.pos);
            } else {
                // This vial doesn't have this object anymore. Drop it.
                remove_handles.push((handle, rigid_body.user_data));
            }
        }
        for (handle, id) in remove_handles {
            self.objects.remove(&id);
            self.rigid_body_set.remove(handle, &mut self.island_manager, &mut self.collider_set, &mut self.impulse_joint_set, &mut self.multibody_joint_set, true);
        }
        assert!(map.len() == 0);
    }
}

fn drag_force(fluid_density: f32, relative_velocity: f32, reference_area: f32, drag_coefficient: f32) -> f32 {
    0.5 * fluid_density * relative_velocity * relative_velocity * reference_area * drag_coefficient
}

pub(crate) fn circle_wedge_area(R: f32, h: f32) -> f32 {
    R * R * ((R - h)/ R).acos() - (R - h) * (2.0 * R * h - h * h).sqrt()
}

pub(crate) fn circle_buoyancy_area(R: f32, p: &Vector<Real>, water_normal: &Vector<Real>, w: f32) -> Option<f32> {
    let d = p.dot(&water_normal) - w; // distance to plane
    if d > 0.0 {
        None
    } else {
        let h = R - d;
        let A = PI * R * R;
        if h > R {
            Some(A)
        } else {
            Some(A - circle_wedge_area(R, h))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_circle_wedge_area() {
        assert_eq!(circle_wedge_area(1.0, 0.0), 0.0);
        assert_eq!(circle_wedge_area(1.0, 1.0), PI / 2.0);
        // assert_eq!(circle_wedge_area(1.0, 2.0), PI);
    }

    #[test]
    fn test_circle_buoyancy_area() {
        assert_eq!(circle_buoyancy_area(1.0, &vector![0.0, 0.0], &vector![0.0, 1.0], 0.0), Some(PI / 2.0));
        assert_eq!(circle_buoyancy_area(1.0, &vector![0.0, 0.0], &vector![0.0, 1.0], 2.0), Some(PI));
        assert_eq!(circle_buoyancy_area(1.0, &vector![0.0, 0.0], &vector![0.0, 1.0], -2.0), None);
    }

}
