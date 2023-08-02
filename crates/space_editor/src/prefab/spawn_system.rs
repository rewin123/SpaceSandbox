use bevy::prelude::*;
use bevy_scene_hook::{HookedSceneBundle, SceneHook};

use super::component::*;

pub fn spawn_scene(
    mut commands : Commands,
    prefabs : Query<(Entity, &ScenePrefab, Option<&Children>), Changed<ScenePrefab>>,
    auto_childs : Query<&PrefabAutoChild>,
    asset_server : Res<AssetServer>
) {
    for (e, prefab, children) in prefabs.iter() {
        let id = commands.spawn(
             SceneBundle { 
                scene: asset_server.load(format!("{}#{}", &prefab.path, &prefab.scene)), 
                ..default() })
            .id();
        commands.entity(e).add_child(id);
        commands.entity(e).
                insert(VisibilityBundle::default());
        commands.entity(e).insert(GlobalTransform::default());
    }
}