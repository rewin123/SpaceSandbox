use rapier3d_f64::prelude::*;
use bevy::{prelude::*, math::DVec3, utils::HashMap};

#[derive(Resource)]
pub struct RapierContext {
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub impulse_joint_set: ImpulseJointSet,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solvers: CCDSolver,   
    pub query_pipeline : QueryPipeline,

    pub entity2collider : HashMap<Entity, ColliderHandle>,
    pub entity2rigidbody : HashMap<Entity, RigidBodyHandle>,
}

impl Default for RapierContext {
    fn default() -> Self {
        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::default(),
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::default(),
            island_manager: IslandManager::default(),
            broad_phase: BroadPhase::default(),
            narrow_phase: NarrowPhase::default(),
            multibody_joint_set: MultibodyJointSet::default(),
            ccd_solvers: CCDSolver::default(),
            query_pipeline : QueryPipeline::default(),
            entity2collider: HashMap::new(),
            entity2rigidbody: HashMap::new(),
        }
    }
}

impl RapierContext {
    pub fn step(&mut self, dt: f64, gravity : &DVec3) {

        self.integration_parameters.dt = dt.min(1.0 / 30.0);


        self.physics_pipeline.step(
            // &rapier3d_f64::math::Vector::new(gravity.x, gravity.y, gravity.z),
            &rapier3d_f64::math::Vector::new(0.0, -9.8, 0.0),
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solvers,
            Some(&mut self.query_pipeline),
            &(),
            &()
        );
    }

    pub fn propagate_modified_body_positions_to_colliders(&mut self) {
        self.rigid_body_set.propagate_modified_body_positions_to_colliders(&mut self.collider_set);
    }
}