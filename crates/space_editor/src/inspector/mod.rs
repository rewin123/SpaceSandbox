use std::fmt::format;
pub mod ui_reflect;

use bevy::{prelude::*, ecs::{component::ComponentId, change_detection::MutUntyped}, reflect::{ReflectFromPtr, TypeInfo, DynamicEnum, DynamicVariant, DynamicTuple, DynamicStruct}, ptr::PtrMut};
use bevy_egui::*;


use crate::selected::{SelectedPlugin, SelectedEntities};
use ui_reflect::*;

#[derive(Component)]
pub struct SkipInspector;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<SelectedPlugin>() {
            app.add_plugins(SelectedPlugin);
        }

        app.register_type::<Transform>();

        app.add_systems(Update, inspect);
    }
}

pub fn mut_untyped_split<'a>(mut mut_untyped: MutUntyped<'a>) -> (PtrMut<'a>, impl FnMut() + 'a) {
    // bypass_change_detection returns a `&mut PtrMut` which is basically useless, because all its methods take `self`
    let ptr = mut_untyped.bypass_change_detection();
    // SAFETY: this is exactly the same PtrMut, just not in a `&mut`. The old one is no longer accessible
    let ptr = unsafe { PtrMut::new(std::ptr::NonNull::new_unchecked(ptr.as_ptr())) };

    (ptr, move || mut_untyped.set_changed())
}


fn inspect(
    world : &mut World
) {

    let selected = world.resource::<SelectedEntities>().clone();
    let all_registry = world.resource::<AppTypeRegistry>().clone();
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
        let mut ctx = cell.get_entity(ctx_e).unwrap().get_mut::<EguiContext>().unwrap();
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
                        
                    }
                }
            });
        });
        
    }
    
}