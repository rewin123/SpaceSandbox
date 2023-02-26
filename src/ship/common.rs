use bevy::prelude::*;
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
    pub common_id : u32
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

    let mut indexer = 0;

    {
        let bbox : IVec3 = [8, 1, 8].into();
        let cfg = VoxelInstanceConfig {
            name : "Metal grids".to_string(),
            instance : VoxelInstance { bbox: bbox.clone(), common_id : indexer },
            create : ClosureInstance::new(move |cmds : &mut Commands, asset_server : &AssetServer| {
                cmds.spawn(SceneBundle {
                    scene: asset_server.load("ss13/wall_models/metal_grid/metal_grid.gltf#Scene0"),
                    
                    ..default()
                }).insert(Collider::cuboid(
                    bbox.x as f32 * VOXEL_SIZE / 2.0, 
                    bbox.y as f32 * VOXEL_SIZE / 2.0, 
                    bbox.z as f32 * VOXEL_SIZE / 2.0))
                .insert(VoxelInstance { bbox: bbox.clone(), common_id : indexer }).id()
            }).to_box()
        };

        configs.push(cfg);
        indexer += 1;
    }

    {
        let bbox : IVec3 = [2, 1, 2].into();
        let cfg = VoxelInstanceConfig {
            name : TELEPORN_NAME.to_string(),
            instance : VoxelInstance { bbox: bbox.clone(), common_id : indexer },
            create : ClosureInstance::new(move |cmds : &mut Commands, asset_server : &AssetServer| {
                cmds.spawn(SceneBundle {
                    scene: asset_server.load("furniture/teleport_spot.glb#Scene0"),
                    
                    ..default()
                }).insert(Collider::cuboid(
                    bbox.x as f32 * VOXEL_SIZE / 2.0, 
                    bbox.y as f32 * VOXEL_SIZE / 2.0 / 2.0, 
                    bbox.z as f32 * VOXEL_SIZE / 2.0 ))
                .insert(VoxelInstance { bbox: bbox.clone(), common_id : indexer }).id()
            }).to_box()
        };

        configs.push(cfg);
        indexer += 1;
    }

    {
        let bbox : IVec3 = [8, 1, 8].into();
        let cfg = VoxelInstanceConfig {
            name : "White plate".to_string(),
            instance : VoxelInstance { bbox: bbox.clone(), common_id : indexer },
            create : ClosureInstance::new(move |cmds : &mut Commands, asset_server : &AssetServer| {
                cmds.spawn(SceneBundle {
                    scene: asset_server.load("ship/tiles/base_plate.glb#Scene0"),
                    
                    ..default()
                }).insert(Collider::cuboid(
                    bbox.x as f32 * VOXEL_SIZE / 2.0, 
                    bbox.y as f32 * VOXEL_SIZE / 2.0, 
                    bbox.z as f32 * VOXEL_SIZE / 2.0))
                .insert(VoxelInstance { bbox: bbox.clone(), common_id : indexer }).id()
            }).to_box()
        };

        configs.push(cfg);
        indexer += 1;
    }

    {
        let bbox : IVec3 = [8, 1, 8].into();
        let cfg = VoxelInstanceConfig {
            name : "White triangle plate".to_string(),
            instance : VoxelInstance { bbox: bbox.clone(), common_id : indexer },
            create : ClosureInstance::new(move |cmds : &mut Commands, asset_server : &AssetServer| {
                cmds.spawn(SceneBundle {
                    scene: asset_server.load("ship/tiles/white_triangle_plate.glb#Scene0"),
                    
                    ..default()
                }).insert(Collider::cuboid(
                    bbox.x as f32 * VOXEL_SIZE / 2.0, 
                    bbox.y as f32 * VOXEL_SIZE / 2.0, 
                    bbox.z as f32 * VOXEL_SIZE / 2.0))
                .insert(VoxelInstance { bbox: bbox.clone(), common_id : indexer }).id()
            }).to_box()
        };

        configs.push(cfg);
        indexer += 1;
    }

    {
        let bbox : IVec3 = [8, 8, 8].into();
        let cfg = VoxelInstanceConfig {
            name : "White door".to_string(),
            instance : VoxelInstance { bbox: bbox.clone(), common_id : indexer },
            create : ClosureInstance::new(move |cmds : &mut Commands, asset_server : &AssetServer| {
                cmds.spawn(SceneBundle {
                    scene: asset_server.load("ship/tiles/door.glb#Scene0"),
                    
                    ..default()
                }).insert(Collider::cuboid(
                    bbox.x as f32 * VOXEL_SIZE / 2.0, 
                    bbox.y as f32 * VOXEL_SIZE / 2.0, 
                    bbox.z as f32 * VOXEL_SIZE / 2.0))
                .insert(VoxelInstance { bbox: bbox.clone(), common_id : indexer }).id()
            }).to_box()
        };

        configs.push(cfg);
        indexer += 1;
    }

    {
        let bbox : IVec3 = [8, 1, 8].into();
        let cfg = VoxelInstanceConfig {
            name : "Window".to_string(),
            instance : VoxelInstance { bbox: bbox.clone(), common_id : indexer },
            create : ClosureInstance::new(move |cmds : &mut Commands, asset_server : &AssetServer| {
                cmds.spawn(SceneBundle {
                    scene: asset_server.load("ship/tiles/window.glb#Scene0"),
                    
                    ..default()
                }).insert(Collider::cuboid(
                    bbox.x as f32 * VOXEL_SIZE / 2.0, 
                    bbox.y as f32 * VOXEL_SIZE / 2.0, 
                    bbox.z as f32 * VOXEL_SIZE / 2.0))
                .insert(VoxelInstance { bbox: bbox.clone(), common_id : indexer }).id()
            }).to_box()
        };

        configs.push(cfg);
        indexer += 1;
    }

    {
        let bbox : IVec3 = [8, 1, 8].into();
        let cfg = VoxelInstanceConfig {
            name : "Corner window".to_string(),
            instance : VoxelInstance { bbox: bbox.clone(), common_id : indexer },
            create : ClosureInstance::new(move |cmds : &mut Commands, asset_server : &AssetServer| {
                cmds.spawn(SceneBundle {
                    scene: asset_server.load("ship/tiles/corner_window.glb#Scene0"),
                    
                    ..default()
                }).insert(Collider::cuboid(
                    bbox.x as f32 * VOXEL_SIZE / 2.0, 
                    bbox.y as f32 * VOXEL_SIZE / 2.0, 
                    bbox.z as f32 * VOXEL_SIZE / 2.0))
                .insert(VoxelInstance { bbox: bbox.clone(), common_id : indexer }).id()
            }).to_box()
        };

        configs.push(cfg);
        indexer += 1;
    }

    {
        let bbox : IVec3 = [8, 1, 8].into();
        let cfg = VoxelInstanceConfig {
            name : "Engine".to_string(),
            instance : VoxelInstance { bbox: bbox.clone(), common_id : indexer },
            create : ClosureInstance::new(move |cmds : &mut Commands, asset_server : &AssetServer| {
                cmds.spawn(SceneBundle {
                    scene: asset_server.load("ship/tiles/engine.glb#Scene0"),
                    
                    ..default()
                }).insert(Collider::cuboid(
                    bbox.x as f32 * VOXEL_SIZE / 2.0, 
                    bbox.y as f32 * VOXEL_SIZE / 2.0, 
                    bbox.z as f32 * VOXEL_SIZE / 2.0))
                .insert(VoxelInstance { bbox: bbox.clone(), common_id : indexer }).id()
            }).to_box()
        };

        configs.push(cfg);
        indexer += 1;
    }

    {
        let bbox : IVec3 = [8, 1, 8].into();
        let cfg = VoxelInstanceConfig {
            name : "Pilot seat".to_string(),
            instance : VoxelInstance { bbox: bbox.clone(), common_id : indexer },
            create : ClosureInstance::new(move |cmds : &mut Commands, asset_server : &AssetServer| {
                cmds.spawn(SceneBundle {
                    scene: asset_server.load("ship/tiles/pilot_seat.glb#Scene0"),
                    
                    ..default()
                }).insert(Collider::cuboid(
                    bbox.x as f32 * VOXEL_SIZE / 2.0, 
                    bbox.y as f32 * VOXEL_SIZE / 2.0, 
                    bbox.z as f32 * VOXEL_SIZE / 2.0))
                .insert(VoxelInstance { bbox: bbox.clone(), common_id : indexer }).id()
            }).to_box()
        };

        configs.push(cfg);
        indexer += 1;
    }

    {
        let bbox : IVec3 = [8, 1, 8].into();
        let cfg = VoxelInstanceConfig {
            name : "Pilot top window".to_string(),
            instance : VoxelInstance { bbox: bbox.clone(), common_id : indexer },
            create : ClosureInstance::new(move |cmds : &mut Commands, asset_server : &AssetServer| {
                cmds.spawn(SceneBundle {
                    scene: asset_server.load("ship/tiles/pilot_top_window.glb#Scene0"),
                    
                    ..default()
                }).insert(Collider::cuboid(
                    bbox.x as f32 * VOXEL_SIZE / 2.0, 
                    bbox.y as f32 * VOXEL_SIZE / 2.0, 
                    bbox.z as f32 * VOXEL_SIZE / 2.0))
                .insert(VoxelInstance { bbox: bbox.clone(), common_id : indexer }).id()
            }).to_box()
        };

        configs.push(cfg);
        indexer += 1;
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