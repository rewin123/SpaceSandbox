pub mod ui_reflect;
pub mod registration;

use std::any::TypeId;

use bevy::{prelude::*, ecs::{component::ComponentId, change_detection::MutUntyped}, reflect::{ReflectFromPtr, TypeInfo, DynamicEnum, DynamicVariant, DynamicTuple, DynamicStruct}, ptr::PtrMut, app::AppLabel};
use bevy_egui::*;


use crate::selected::{SelectedPlugin, SelectedEntities};
use ui_reflect::*;
use registration::*;

#[derive(Component)]
pub struct SkipInspector;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<SelectedPlugin>() {
            app.add_plugins(SelectedPlugin);
        }

        app.init_resource::<InspectState>();
        app.init_resource::<EditorRegistry>();

        app.editor_registry::<Transform>();
        app.editor_registry::<Name>();

        app.add_systems(Update, (inspect, execute_inspect_command).chain());
    }
}

pub fn mut_untyped_split<'a>(mut mut_untyped: MutUntyped<'a>) -> (PtrMut<'a>, impl FnMut() + 'a) {
    // bypass_change_detection returns a `&mut PtrMut` which is basically useless, because all its methods take `self`
    let ptr = mut_untyped.bypass_change_detection();
    // SAFETY: this is exactly the same PtrMut, just not in a `&mut`. The old one is no longer accessible
    let ptr = unsafe { PtrMut::new(std::ptr::NonNull::new_unchecked(ptr.as_ptr())) };

    (ptr, move || mut_untyped.set_changed())
}

#[derive(Default, Resource)]
struct InspectState {
    create_component_type : Option<ComponentId>,
    commands : Vec<InspectCommand>
}


enum InspectCommand {
    AddComponent(Entity, TypeId)
}

fn execute_inspect_command(
    mut commands : Commands,
    mut state : ResMut<InspectState>,
    registration : Res<EditorRegistry>
) {
    for c in &state.commands {
        match c {
            InspectCommand::AddComponent(e, id) => {
                info!("inspector adding component {:?} to entity {:?}", id, e);
                commands.entity(*e).add(registration.get_spawn_command(id));
            },
        }
    }
    state.commands.clear();
}

fn inspect(
    world : &mut World
) {

    let selected = world.resource::<SelectedEntities>().clone();

    let editor_registry = world.resource::<EditorRegistry>().clone();
    let all_registry = editor_registry.registry.clone();
    let registry = all_registry.read();
    let ctx_e;
    {
        let mut ctx_query = world.query_filtered::<Entity, (With<EguiContext>, With<Window>)>();
        ctx_e = ctx_query.get_single(&world).unwrap();
    }

    let mut components_id = vec![];
    let mut types_id = vec![];


    for reg in registry.iter() {
        if let Some(c_id) = world.components().get_id(reg.type_id()) {
            components_id.push(c_id);
            types_id.push(reg.type_id());
        }
    }

    unsafe {
        let cell = world.as_unsafe_world_cell();
        let mut state = cell.get_resource_mut::<InspectState>().unwrap();

        let mut ctx = cell.get_entity(ctx_e).unwrap().get_mut::<EguiContext>().unwrap();
        let mut commands : Vec<InspectCommand> = vec![];
        egui::SidePanel::right("Inspector").show(ctx.get_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for e in selected.list.iter() {
                    if let Some(e) = cell.get_entity(*e) {
                        let name;
                        if let Some(name_struct) = e.get::<Name>() {
                            name = name_struct.as_str().to_string()
                        } else {
                            name = format!("{:?}", e.id());
                        }
                        ui.heading(&name);
                        for idx in 0..components_id.len() {
                            let c_id = components_id[idx];
                            let t_id = types_id[idx];
                            if let Some(data) = e.get_mut_by_id(c_id) {
                                let registration = registry
                                    .get(t_id).unwrap();
                                if let Some(reflect_from_ptr) = registration.data::<ReflectFromPtr>() {
                                    let (ptr, mut set_changed) = mut_untyped_split(data);
        
                                    let value = reflect_from_ptr.as_reflect_ptr_mut(ptr);
        
                                    ui_for_reflect(ui, value, &name, registration.short_name(),&mut set_changed);
                                }
                            }
                        }

                        //add component
                        let selected_name;
                        if let Some(selected_id) = state.create_component_type {
                            let selected_info = cell.components().get_info(selected_id).unwrap();
                            selected_name = registry.get(selected_info.type_id().unwrap()).unwrap().short_name().to_string();
                        } else {
                            selected_name = "Press to select".to_string();
                        }
                        let combo = egui::ComboBox::new(format!("inspect_select"), "New")
                            .selected_text(&selected_name).show_ui(ui, |ui| {
                                for idx in 0..components_id.len() {
                                    let c_id = components_id[idx];
                                    let t_id = types_id[idx];
                                    
                                    let name = registry.get(t_id).unwrap().short_name();
                                    ui.selectable_value(
                                        &mut state.create_component_type, 
                                        Some(c_id),
                                        name);
                                }
                            });
                        if ui.button("Add component").clicked() {
                            info!("adding component button clicked");
                            if let Some(c_id) = state.create_component_type {
                                info!("adding component {:?}", c_id);
                                let id = cell.components().get_info(c_id).unwrap().type_id().unwrap();
                                commands.push(InspectCommand::AddComponent(e.id(), id));
                            }
                        }
                        
                    }
                }
            });
        });

        state.commands = commands;
    }
    
}