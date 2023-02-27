use bevy::{prelude::*, ecs::system::EntityCommands};
use bevy_rapier3d::prelude::Collider;

use super::VOXEL_SIZE;

pub const TELEPORN_NAME : &'static str = "Teleport spot";

pub trait BuildInstance {
    fn build(&self, cmds : &mut Commands, asset_server : &AssetServer) -> Entity;
}

pub struct ClosureInstance<F> {
    pub f : F
}

impl<F> ClosureInstance<F> {
    fn new(f : F) -> ClosureInstance<F> {
        ClosureInstance {
            f
        }
    }

    fn to_box(self) -> Box<ClosureInstance<F>> {
        Box::new(self)
    }
}

impl<F> BuildInstance for ClosureInstance<F>
        where F : Fn(&mut Commands, &AssetServer) -> Entity {
    fn build(&self, cmds : &mut Commands, asset_server : &AssetServer) -> Entity {
        (self.f)(cmds, asset_server)
    }
}

#[derive(Component, Clone)]
pub struct VoxelInstance {
    pub bbox : IVec3,
    pub common_id : u32,
    pub origin : Vec3
}


pub struct VoxelInstanceConfig
 {
    pub name : String,
    pub instance : VoxelInstance,
    pub create : Box<dyn BuildInstance + Send + Sync>
}

#[derive(Resource)]
pub struct AllVoxelInstances {
    pub configs : Vec<VoxelInstanceConfig>
}

fn spawn_static_instance<F>(
        indexer : &mut u32,
        bbox : IVec3,
        origin : Vec3,
        name : &str,
        path : &str,
        after_build : F) -> VoxelInstanceConfig
    where F : Fn(&mut EntityCommands) + Send  + Sync + 'static {
    
    let owned_path = path.to_string();

    let instance = VoxelInstance {
        bbox : bbox.clone(),
        common_id : *indexer,
        origin : origin.clone()
    };
    let cfg = VoxelInstanceConfig {
        name : name.to_string(),
        instance : instance.clone(),
        create : ClosureInstance::new(move |cmds : &mut Commands, asset_server : &AssetServer| {
            let id = cmds.spawn(SceneBundle {
                scene: asset_server.load(&owned_path),
                
                ..default()
            })
            .insert(instance.clone()).id();

            let collider_pos = -instance.origin.clone() * bbox.as_vec3() / 2.0 * VOXEL_SIZE;
            
            let shifted_collider = cmds.spawn((Collider::cuboid(
                bbox.x as f32 * VOXEL_SIZE / 2.0, 
                bbox.y as f32 * VOXEL_SIZE / 2.0, 
                bbox.z as f32 * VOXEL_SIZE / 2.0)))
            .insert(SpatialBundle::from(Transform::from_xyz(collider_pos.x, collider_pos.y, collider_pos.z))).id();

            cmds.entity(id).add_child(shifted_collider);
            after_build(&mut cmds.entity(id));

            id
        }).to_box()
    };
    *indexer += 1;

    cfg
}

pub fn init_all_voxel_instances(
    mut cmds : Commands
) {
    let mut configs = vec![];

    let mut indexer = 0;

    {
        let cfg = spawn_static_instance(
            &mut indexer, 
            [8, 1, 8].into(), 
            [0.0, 0.0, 0.0].into(), 
            "Metal grids", 
            "ss13/wall_models/metal_grid/metal_grid.gltf#Scene0",
            |_|{});
        configs.push(cfg);
    }

    {   
        let cfg = spawn_static_instance(
            &mut indexer,
            [2, 1, 2].into(),
            [0.0, 0.0, 0.0].into(),
            TELEPORN_NAME,
            "furniture/teleport_spot.glb#Scene0",
            |_|{}
        );
        configs.push(cfg);
    }

    {
        let cfg = spawn_static_instance(
            &mut indexer,
            [8, 1, 8].into(),
            [0.0, 0.0, 0.0].into(),
            "White plate",
            "ship/tiles/base_plate.glb#Scene0",
            |_|{}
        );
        configs.push(cfg);
    }

    {
        let cfg = spawn_static_instance(
            &mut indexer,
            [8, 1, 8].into(),
            [0.0, 0.0, 0.0].into(),
            "White triangle plate",
            "ship/tiles/white_triangle_plate.glb#Scene0",
            |_|{}
        );
        configs.push(cfg);
    }

    {
        let cfg = spawn_static_instance(
            &mut indexer,
            [8, 8, 8].into(),
            [0.0, 0.0, 0.0].into(),
            "White door",
            "ship/tiles/door.glb#Scene0",
            |_|{}
        );
        configs.push(cfg);
    }

    {
        let cfg = spawn_static_instance(
            &mut indexer,
            [8, 1, 8].into(),
            [0.0, 0.0, 0.0].into(),
            "Window",
            "ship/tiles/window.glb#Scene0",
            |_|{}
        );
        configs.push(cfg);
    }

    {
        let cfg = spawn_static_instance(
            &mut indexer,
            [8, 1, 8].into(),
            [0.0, 0.0, 0.0].into(),
            "Corner window",
            "ship/tiles/corner_window.glb#Scene0",
            |_|{}
        );
        configs.push(cfg);
    }

    {
        let cfg = spawn_static_instance(
            &mut indexer,
            [8, 1, 8].into(),
            [0.0, 0.0, 0.0].into(),
            "Engine",
            "ship/tiles/engine.glb#Scene0",
            |_|{}
        );
        configs.push(cfg);
    }

    {
        let cfg = spawn_static_instance(
            &mut indexer,
            [8, 4, 8].into(),
            [0.0, -1.0, 0.0].into(),
            "Pilot seat",
            "ship/tiles/pilot_seat.glb#Scene0",
            |_|{}
        );
        configs.push(cfg);
    }

    {
        let cfg = spawn_static_instance(
            &mut indexer,
            [8, 1, 8].into(),
            [0.0, 0.0, 0.0].into(),
            "Pilot top window",
            "ship/tiles/pilot_top_window.glb#Scene0",
            |_|{}
        );
        configs.push(cfg);
    }

    let all_instances = AllVoxelInstances {
        configs
    };

    cmds.insert_resource(all_instances);
}


pub struct VoxelInstancePlugin;

impl Plugin for VoxelInstancePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_all_voxel_instances);
    }
}