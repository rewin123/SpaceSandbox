use std::fmt::format;

use bevy::{prelude::*, ecs::{component::ComponentId, change_detection::MutUntyped}, reflect::{ReflectFromPtr, TypeInfo, DynamicEnum, DynamicVariant, DynamicTuple, DynamicStruct}, ptr::PtrMut};
use bevy_egui::*;


use crate::selected::{SelectedPlugin, SelectedEntities};

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

fn ui_for_reflect(
        ui : &mut egui::Ui,
        value : &mut dyn Reflect,
        hash : &str,
        name : &str,
        set_changed : &mut impl FnMut()) {
    match value.reflect_mut() {
        bevy::reflect::ReflectMut::Struct(value) => {ui_for_struct(ui, value, hash, name, set_changed)},
        bevy::reflect::ReflectMut::TupleStruct(value) => {ui_for_tuple_struct(ui, value, hash, name, set_changed)},
        bevy::reflect::ReflectMut::Tuple(value) => {println!("Tuple")},
        bevy::reflect::ReflectMut::List(value) => {ui_for_list(ui, value, hash, name, set_changed)},
        bevy::reflect::ReflectMut::Array(value) => {println!("Array")},
        bevy::reflect::ReflectMut::Map(value) => {println!("Map")},
        bevy::reflect::ReflectMut::Enum(value) => {ui_for_enum(ui, value, hash, name, set_changed)},
        bevy::reflect::ReflectMut::Value(value) => {ui_for_value(ui, value, hash, name, set_changed)},
    }
}

fn ui_for_list(
    ui : &mut egui::Ui,
    value : &mut dyn bevy::reflect::List,
    hash : &str,
    name : &str,
    set_changed : &mut impl FnMut()
) {
    let hash = format!("{}{}", hash, name);
    ui.label(name);
    ui.indent(&hash, |ui| {
        for idx in 0..value.len() {
            let subname = format!("{}", idx);
            ui_for_reflect(ui, value.get_mut(idx).unwrap(), &format!("{}{}", hash, subname), &subname, set_changed);
        }
    });
}

fn ui_for_enum(
    ui : &mut egui::Ui,
    value : &mut dyn bevy::reflect::Enum,
    hash : &str,
    name : &str,
    set_changed : &mut impl FnMut()
) {
    let hash = format!("{}{}", hash, name);
    let varian_idx = value.variant_index();
    let selected_name = value.variant_name().to_string();
    let TypeInfo::Enum(enum_info) = value.get_represented_type_info().unwrap() else {
        ui.label("Broken enum");
        return;
    };
    let mut next_name = selected_name.clone();
    egui::ComboBox::new(&hash, name)
            .selected_text(&selected_name)
            .show_ui(ui, |ui| {
        for idx in 0..enum_info.variant_len() {
            let name = enum_info.variant_at(idx).unwrap().name();
            ui.selectable_value(&mut next_name, name.to_string(), name);
        }
    });
    if next_name != selected_name {
        let info = enum_info.variant(&next_name).unwrap();
        let variant = match info {
            bevy::reflect::VariantInfo::Struct(s) => DynamicVariant::Struct(DynamicStruct::default()),
            bevy::reflect::VariantInfo::Tuple(s) => DynamicVariant::Tuple(DynamicTuple::default()),
            bevy::reflect::VariantInfo::Unit(s) => DynamicVariant::Unit,
        };
        let new_variant = DynamicEnum::new(next_name, variant);
        value.apply(&new_variant);
    }
    ui.label(&format!("Enum: {}", selected_name));
    if value.field_len() > 0 {
        ui.indent(&format!("{}indent", hash), |ui| {
            for idx in 0..value.field_len() {
                let name = value.name_at(idx).unwrap_or("").to_string();
                let field = value.field_at_mut(idx).unwrap();
                ui_for_reflect(ui, field, &hash, &name, set_changed);
            }
        });
    }
}

fn ui_for_tuple_struct(
    ui : &mut egui::Ui,
    value : &mut dyn TupleStruct,
    hash : &str,
    name : &str,
    set_changed : &mut impl FnMut()
) {
    let hash = format!("{}{}", hash, value.type_name());
    egui::CollapsingHeader::new(format!("{}", name))
            .show(ui, |ui| {
        ui.indent(&hash, |ui| {
            for idx in 0..value.field_len() {
                if let Some(field) = value.field_mut(idx) {
                    ui_for_reflect(ui, field, format!("{}{}", hash, name).as_str(), "", set_changed)
                }
            }
        });
    });
}

fn ui_for_value(
        ui : &mut egui::Ui,
        value : &mut dyn Reflect,
        hash : &str,
        name : &str,
        set_changed : &mut impl FnMut()) {
    let hash = format!("{}{}", hash, value.type_name());
    
    if value.represents::<f32>() {
        let val = value.downcast_mut::<f32>().unwrap();
        ui.horizontal(|ui| {
            ui.label(name);
            if ui.add(egui::DragValue::new(val).min_decimals(2).speed(0.1)).changed() {
                set_changed();
            }
        });
    } else if value.represents::<Entity>() {
        let val = value.downcast_ref::<Entity>().unwrap();
        ui.label(format!("{} : {:?}", name, val));
        
    } else {
        ui.label(format!("{} not reflected", name));
    }
}

fn ui_for_struct(
        ui : &mut egui::Ui,
        value : &mut dyn Struct,
        hash : &str,
        name : &str,
        set_changed : &mut impl FnMut()) {
    let hash = format!("{}{}", hash, value.type_name());
    egui::CollapsingHeader::new(format!("{}", name))
            .show(ui, |ui| {
        ui.indent(&hash, |ui| {
            for idx in 0..value.field_len() {
                let mut name = "".to_string();
                if let Some(name_str) = value.name_at(idx) {
                    name = name_str.to_string();
                }
                if let Some(field) = value.field_at_mut(idx) {
                    ui_for_reflect(ui, field, format!("{}{}", hash, name).as_str(), &name, set_changed)
                }
            }
        });
    });
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