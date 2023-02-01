use bevy::prelude::*;
use bevy_rapier3d::prelude::Collider;

use super::VOXEL_SIZE;

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
    pub bbox : IVec3
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

pub fn init_all_voxel_instances(
    mut cmds : Commands
) {
    let mut configs = vec![];

    {
        let bbox : IVec3 = [4, 1, 4].into();
        let cfg = VoxelInstanceConfig {
            name : "Metal grids".to_string(),
            instance : VoxelInstance { bbox: bbox.clone() },
            create : ClosureInstance::new(move |cmds : &mut Commands, asset_server : &AssetServer| {
                cmds.spawn(SceneBundle {
                    scene: asset_server.load("ss13/wall_models/metal_grid/metal_grid.gltf#Scene0"),
                    
                    ..default()
                }).insert(Collider::cuboid(
                    bbox.x as f32 * VOXEL_SIZE / 2.0, 
                    bbox.y as f32 * VOXEL_SIZE / 2.0, 
                    bbox.z as f32 * VOXEL_SIZE / 2.0)).id()
            }).to_box()
        };

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