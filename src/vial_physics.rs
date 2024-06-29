use std::collections::{HashMap, HashSet};
use super::{Vial, Object};

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

const G_TO_KG: f32 = 1_000.0;
const M_TO_MM: f32 = 1_000.0;
const MM_TO_M: f32 = 0.001;

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
                .restitution(0.7)
                .build();
            collider.set_density(2.4 * G_TO_KG); // glass
            collider.user_data = obj.id as u128;
            let handle = self.rigid_body_set.insert(rigid_body);
            self.objects.insert(obj.id, handle);
            self.collider_set.insert_with_parent(collider, handle, &mut self.rigid_body_set);
            true
        } else {
            false
        }
    }

    pub fn step(&mut self, dt: Real) {
        // let gravity = vector![0.5, -9.81];
        let gravity = vector![0.0, -9.81];
        // let gravity = vector![0.0, 0.0];
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
