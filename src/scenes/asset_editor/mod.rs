use std::path::PathBuf;

use bevy::{prelude::*, math::DVec3, reflect::TypeUuid};
use bevy_egui::{egui::{Ui, self}, EguiContext};
use bevy_proto::{prelude::{PrototypesMut, ProtoAssetEvent, ProtoCommands, prototype_ready, Prototypical, ReflectSchematic, Schematic}, backend::schematics::FromSchematicInput};
use bevy_transform64::{DTransformBundle, prelude::DTransform};
use serde::{Serialize, Deserialize};
use space_physics::prelude::{ColliderBuilder, nalgebra, SpaceCollider};

use crate::{SceneType, pawn_system::Pawn, ship::{instance_rotate::InstanceRotate, prelude::VoxelInstance, VOXEL_SIZE}};
use bevy_common_assets::ron::RonAssetPlugin;

mod explorer;

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "576c943a-477a-4885-add8-28d774f44beb"]
struct ProtoPaths {
    paths: Vec<String>
}

#[derive(Resource)]
struct AssetEditorHandleState {
    paths : Handle<ProtoPaths>
}
struct AssetEditorState<'a> {
    paths : &'a ProtoPaths
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
#[system_set()]
enum AsssetEditorSet {
    Base
}

pub struct AssetEditorPlugin;

impl Plugin for AssetEditorPlugin {
    fn build(&self, app: &mut App) {

        app.configure_set(AsssetEditorSet::Base.run_if(in_state(SceneType::AssetEditor)));

        app.register_type::<BlockConfig>();
        app.add_systems((load,show_proto_editor, setup_block).in_set(AsssetEditorSet::Base));
        app.add_system(listen_load_event.after(load).in_set(AsssetEditorSet::Base));
        app.add_system(setup.in_schedule(OnEnter(SceneType::AssetEditor)));
        app.insert_resource(ProtoEditor::default());
        app.add_startup_systems((load_state,));
        app.add_plugin(RonAssetPlugin::<ProtoPaths>::new(&["proto_list.ron"]));
        app.insert_resource(AssetEditorHandleState {
            paths : Handle::<ProtoPaths>::default()
        });
        app.add_event::<LabelClickedEvent>();
        app.insert_resource(CurrentProto {
            path : None,
            name : String::new(),
            proto_handle : Handle::default(),
            entity : None
        });
    }
}

#[derive(Resource)]
struct CurrentProto {
    path : Option<String>,
    name : String,
    proto_handle : Handle<bevy_proto::prelude::Prototype>,
    entity : Option<Entity>
}

pub struct LabelClickedEvent {
    pub path: String,
}

#[derive(Default, Resource)]
pub struct ProtoEditor {
    selected_path: Option<String>,  // Store the selected path
}

impl ProtoEditor {
    pub fn new(path: String) -> Self {
        Self {
            selected_path: None,
        }
    }

    fn show(&mut self, ui: &mut Ui, state : &mut AssetEditorState, mut events: &mut EventWriter<LabelClickedEvent>) {
        for path in state.paths.paths.iter() {
            let is_selected = match &self.selected_path {
                Some(selected_path) => path == selected_path,
                None => false,
            };
            if ui.selectable_label(is_selected, path).clicked() {
                self.selected_path = Some(path.clone());
                events.send(LabelClickedEvent {
                    path: path.clone(),
                });
            }
        }
    }
}

fn listen_load_event(
    mut commands: ProtoCommands,
    mut events : EventReader<AssetEvent<bevy_proto::prelude::Prototype>>,
    mut cur_state : ResMut<CurrentProto>,
    assets : ResMut<Assets<bevy_proto::prelude::Prototype>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut proto_asset_events: EventReader<ProtoAssetEvent>,
    mut event_reader : EventReader<LabelClickedEvent>
    
) {
    for ev in events.iter() {
        match ev {
            AssetEvent::Created { handle } => {
                spawn_proto(&mut cur_state, handle, &assets, &mut commands, &keyboard_input, &mut proto_asset_events);
            },
            AssetEvent::Modified { handle } => {
                spawn_proto(&mut cur_state, handle, &assets, &mut commands, &keyboard_input, &mut proto_asset_events);
            },
            AssetEvent::Removed { handle } => {},
        }
    }

    for ev in event_reader.iter() {
        let handle = cur_state.proto_handle.clone();
        spawn_proto(&mut cur_state, &handle, &assets, &mut commands, &keyboard_input, &mut proto_asset_events);
    }
}

fn spawn_proto(cur_state: &mut ResMut<CurrentProto>, handle: &Handle<bevy_proto::proto::Prototype>, assets: &ResMut<Assets<bevy_proto::proto::Prototype>>, commands: &mut bevy_proto::backend::proto::ProtoCommands<bevy_proto::proto::Prototype, bevy_proto::prelude::ProtoConfig>, keyboard_input: &Res<Input<KeyCode>>, proto_asset_events: &mut EventReader<bevy_proto::backend::proto::ProtoAssetEvent<bevy_proto::proto::Prototype>>) {
    if cur_state.proto_handle.id() == handle.id() {
        if let Some(proto) = assets.get(handle) {
            let prev_id = cur_state.name.clone();
            cur_state.name = proto.id().clone();
            let cur_id = cur_state.name.clone();
            spawn(
                commands, 
                keyboard_input, 
                &mut cur_state.entity, 
                proto_asset_events, 
                &prev_id,
                &cur_id);
        }
    }
}

fn show_proto_editor(
    mut proto : ResMut<ProtoEditor>, 
    mut ctx : Query<&mut bevy_egui::EguiContext>,
    mut all_states : ResMut<Assets<ProtoPaths>>,
    mut handle_state : Res<AssetEditorHandleState>,
    mut event_writer : EventWriter<LabelClickedEvent>
    ) {
    let Some(mut ctx) = ctx.iter_mut().next() else {
        return;
    };


    egui::Window::new("Proto Editor").show(ctx.get_mut(), |ui| {

        if let Some(paths) = all_states.get(&handle_state.paths) {
            let mut state = AssetEditorState {
                paths
            };
            proto.show(ui, &mut state, &mut event_writer); 
        } else {
            ui.label("Loading...");
        }
    });
}

fn load(mut prototypes: PrototypesMut, mut current_proto: ResMut<CurrentProto>, mut event_reader : EventReader<LabelClickedEvent>) {
    for event in event_reader.iter() {
        current_proto.path = Some(event.path.clone());
        let handle = prototypes.load(&event.path);
        current_proto.proto_handle = handle;
    }
}

fn load_state(mut state : ResMut<AssetEditorHandleState>, mut asset_server : ResMut<AssetServer>) {
    state.paths = asset_server.load("all_proto.proto_list.ron");
}

fn spawn(
    mut commands: &mut ProtoCommands,
    keyboard_input: &Res<Input<KeyCode>>,
    mut previous: &mut Option<Entity>,
    mut proto_asset_events: &mut EventReader<ProtoAssetEvent>,
    prev_id : &String,
    cur_id : &String
) {
    if cur_id != prev_id {
        if let Some(e) = previous {
            commands.entity(*e).entity_commands().despawn_recursive();
            *previous = None
        }
    }

    if previous.is_none() || keyboard_input.just_pressed(KeyCode::Space) {
        *previous = Some(commands.spawn(cur_id).id());
    }

    // Listen for changes:
    for proto_asset_event in proto_asset_events.iter() {
        match proto_asset_event {
            // Only trigger a re-insert of the prototype when modified and if IDs match
            ProtoAssetEvent::Modified { id, .. } if id == cur_id => {
                commands
                    .entity(previous.unwrap())
                    .insert(cur_id);
            }
            _ => {}
        }
    }
}

fn inspect(query: Query<DebugName, Added<Name>>) {
    for name in &query {
        info!("Spawned: {:?}", name);
    }
}

fn setup(mut cmds : Commands) {
    let pawn = cmds.spawn(Camera3dBundle {
        camera_3d : Camera3d {
            clear_color : bevy::core_pipeline::clear_color::ClearColorConfig::Custom(Color::Rgba { red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0 }),
            ..default()
        },
        ..default()
    }).insert(
        DTransformBundle::from_transform(
            DTransform::from_xyz(10.0, 10.0, 10.0).looking_at(DVec3::new(0.0, 0.0, 0.0), DVec3::Y))).id();

    cmds.entity(pawn).insert(Pawn { camera_id: pawn });

    // ambient light
    cmds.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });

    const HALF_SIZE: f32 = 100.0;
    cmds.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-2.5),
            ..default()
        },
        ..default()
    });
}


#[derive(Component, Reflect, FromReflect, Schematic)]
#[reflect(Schematic)]
pub struct BlockConfig {
    pub bbox : IVec3,
    pub origin : DVec3
}

fn setup_block(mut cmds : Commands, mut query : Query<(Entity, &BlockConfig), Changed<BlockConfig>>) {
    for (entity, config) in query.iter() {
        cmds.entity(entity).insert(InstanceRotate::default());

        let instance = VoxelInstance {
            bbox : config.bbox.clone(),
            common_id : 0,
            origin : config.origin.clone()
        };
        let bbox = instance.bbox.clone();

        let collider_pos = -instance.origin.clone() * bbox.as_dvec3() / 2.0 * VOXEL_SIZE;

        let collider = ColliderBuilder::cuboid(
            bbox.x as f64 * VOXEL_SIZE / 2.0, 
            bbox.y as f64 * VOXEL_SIZE / 2.0, 
            bbox.z as f64 * VOXEL_SIZE / 2.0
        ).translation(nalgebra::Vector3::new(collider_pos.x, collider_pos.y, collider_pos.y)).build();

        cmds.entity(entity)
                .insert(SpaceCollider(collider))
                .insert(instance);

    }
}