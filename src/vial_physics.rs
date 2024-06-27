use std::collections::HashMap;
use super::{Vial, Object};

use rapier2d::prelude::*;

pub struct VialPhysics {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    objects: Vec<RigidBodyHandle>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
    physics_hooks: (),
    event_handler: (),
}

const g_to_kg: f32 = 0.0001;
const m_to_mm: f32 = 0.0001;
const mm_to_m: f32 = 1_000.0;

impl VialPhysics {
    pub fn new(vial: &Vial) -> Self {

        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();
        // Convert from mm to m.
        let vial_size_m = vial.size * mm_to_m;

        let wall_width = 1.0 * mm_to_m;
        /* Create the ground. */
        let collider = ColliderBuilder::cuboid(vial_size_m.x, wall_width)
            .translation(vector![0.0, - wall_width / 2.0])
            .build();
        collider_set.insert(collider);

        // Walls

        // Left wall
        let collider = ColliderBuilder::cuboid(wall_width, vial_size_m.y)
            .translation(vector![-wall_width / 2.0, vial_size_m.y / 2.0 - wall_width / 2.0])
            .build();
        collider_set.insert(collider);

        // Right wall
        let collider = ColliderBuilder::cuboid(wall_width, vial_size_m.y * 1.5)
            .translation(vector![vial_size_m.x -  wall_width / 2.0,
                                 vial_size_m.y / 2.0 - wall_width / 2.0])
            .build();
        collider_set.insert(collider);

        let mut objects = vec![];
        for obj in &vial.objects {
            /* Create the bouncing ball. */
            let pos_m = obj.pos * mm_to_m;
            let mut rigid_body = RigidBodyBuilder::dynamic()
                .translation(vector![pos_m.x, pos_m.y])
                .build();
            rigid_body.user_data = obj.id as u128;
            let mut collider = ColliderBuilder::ball((obj.size as f32 * mm_to_m).max(0.2)).restitution(0.7)
                                                                     .build();
            collider.set_mass(0.1);
            collider.user_data = obj.id as u128;
            let handle = rigid_body_set.insert(rigid_body);
            objects.push(handle);
            collider_set.insert_with_parent(collider, handle, &mut rigid_body_set);
        }

        /* Create other structures necessary for the simulation. */
        let integration_parameters = IntegrationParameters::default();
        let mut physics_pipeline = PhysicsPipeline::new();
        let mut island_manager = IslandManager::new();
        let mut broad_phase = DefaultBroadPhase::new();
        let mut narrow_phase = NarrowPhase::new();
        let mut impulse_joint_set = ImpulseJointSet::new();
        let mut multibody_joint_set = MultibodyJointSet::new();
        let mut ccd_solver = CCDSolver::new();
        let mut query_pipeline = QueryPipeline::new();
        let physics_hooks = ();
        let event_handler = ();
        VialPhysics {
            rigid_body_set,
            collider_set,
            objects,
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
        }
    }

    pub fn step(&mut self, dt: Real) {
        // let gravity = vector![0.5, -9.81];
        let gravity = vector![0.0, -9.81];
        let mut accum = 0.0;
        /* Run the game loop, stepping the simulation once per frame. */
        while accum < dt {
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
            accum += 1.0 / 60.0;
        }
    }

    pub fn project(&self, vial: &mut Vial) {
        let mut map: HashMap<u128, Object> = vial.objects.drain(..).map(|o| (o.id, o)).collect();
        for handle in &self.objects {
            if let Some(rigid_body) = self.rigid_body_set.get(*handle) {
                if let Some(mut obj) = map.remove(&rigid_body.user_data) {
                    let p = rigid_body.translation();
                    obj.pos.x = p.x * m_to_mm;
                    obj.pos.y = p.y * m_to_mm;
                    vial.objects.push(obj);
                }
            }
        }
    }
}
