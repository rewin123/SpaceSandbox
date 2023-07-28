use bevy::prelude::*;
use bevy_egui::*;

use super::selected::*;

pub struct SpaceHierarchyPlugin {

}

impl Default for SpaceHierarchyPlugin {
    fn default() -> Self {
        Self {

        }
    }
}

impl Plugin for SpaceHierarchyPlugin {
    fn build(&self, app: &mut App) {

        if !app.is_plugin_added::<SelectedPlugin>() {
            app.add_plugins(SelectedPlugin);
        }

        app.add_systems(Update, show_hierarchy);
    }
}


fn show_hierarchy(
    mut contexts : EguiContexts,
    query: Query<(Entity, Option<&Name>, Option<&Children>, Option<&Parent>)>,
    mut selected : ResMut<SelectedEntities>
) {
    egui::SidePanel::left("Scene hierarchy")
        .show(contexts.ctx_mut(), |ui| {
        for (entity, name, children, parent) in query.iter() {
            if parent.is_none() {
                draw_entity(ui, &query, 0, entity, name, children, &mut selected);
            }
        }
    });
}

fn draw_entity(
    ui: &mut egui::Ui,
    query: &Query<(Entity, Option<&Name>, Option<&Children>, Option<&Parent>)>,
    indent: usize,
    entity: Entity,
    name: Option<&Name>,
    children: Option<&Children>,
    selected : &mut SelectedEntities
) {
    let entity_name = name.map_or_else(
        || format!("Entity {:?}", entity),
        |name| format!("Entity {:?}: {:?}", entity, name.as_str()),
    );

    ui.indent(entity_name.clone(), |ui| {
        let is_selected = selected.list.contains(&entity);

        if ui.selectable_label(is_selected, entity_name).clicked() {
            if !is_selected {
                selected.list.insert(entity);
            } else {
                selected.list.remove(&entity);
            }
        }
        if let Some(children) = children {
            for child in children.iter() {
                if let Ok((_, child_name, child_children, _)) = query.get(*child) {
                    draw_entity(ui, query, indent + 1, *child, child_name, child_children, selected);
                }
            }
        }
    });
}