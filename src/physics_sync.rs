use bevy::ecs::schedule::ScheduleLabel;
use bevy_xpbd_3d::PhysicsSchedule;

use crate::ext::*;

pub struct PhysicsSync {
    schedule: Box<dyn ScheduleLabel>
}

impl Default for PhysicsSync {
    fn default() -> Self {
        Self {
            schedule: Box::new(PostUpdate)
        }
    }
}

impl Plugin for PhysicsSync {
    fn build(&self, app: &mut App) {
            app.add_systems(
                PostUpdate,
                (
                    position_to_transform,
                ).chain()
                 .in_set(PhysicsSet::Sync)
            );
    }
}

#[derive(Component, Deref, DerefMut)]
struct PreviousGlobalTransform(DGlobalTransform);


type PhysicsObjectAddedFilter = Or<(Added<RigidBody>, Added<Collider>)>;

fn init_previous_global_transform(
    mut commands: Commands,
    query: Query<(Entity, &DGlobalTransform), PhysicsObjectAddedFilter>,
) {
    for (entity, transform) in &query {
        commands
            .entity(entity)
            .insert(PreviousGlobalTransform(*transform));
    }
}

/// Copies `GlobalTransform` changes to [`Position`] and [`Rotation`].
/// This allows users to use transforms for moving and positioning bodies and colliders.
///
/// To account for hierarchies, transform propagation should be run before this system.
fn transform_to_position(
    mut query: Query<(
        &DGlobalTransform,
        &PreviousGlobalTransform,
        &mut Position,
        &mut Rotation,
    )>,
) {
    for (
        global_transform,
        previous_transform,
        mut position,
        mut rotation,
    ) in &mut query
    {
        // Skip entity if the global transform value hasn't changed
        if *global_transform == previous_transform.0 {
            continue;
        }

        let transform = global_transform.compute_transform();
        let previous_transform = previous_transform.compute_transform();
        let pos = position.0;

        {
            position.0 = (previous_transform.translation
                + (transform.translation - previous_transform.translation))
                + (pos - previous_transform.translation);
        }
        {
            rotation.0 = (previous_transform.rotation
                + (transform.rotation - previous_transform.rotation)
                + (rotation.0 - previous_transform.rotation))
                .normalize();
        }
    }
}

type PosToTransformComponents = (
    &'static mut DTransform,
    &'static Position,
    &'static bevy_xpbd_3d::prelude::Rotation,
    Option<&'static Parent>,
);

type PosToTransformFilter = Or<(Changed<Position>, Changed<bevy_xpbd_3d::prelude::Rotation>)>;

type ParentComponents = (
    &'static DGlobalTransform,
    Option<&'static Position>,
    Option<&'static bevy_xpbd_3d::prelude::Rotation>,
);

// fn transform_to_position()

fn position_to_transform(
    mut query: Query<PosToTransformComponents, PosToTransformFilter>,
    parents: Query<ParentComponents, With<Children>>,
) {
    for (mut transform, pos, rot, parent) in &mut query {
        if let Some(parent) = parent {
            if let Ok((parent_transform, parent_pos, parent_rot)) = parents.get(**parent) {
                // Compute the global transform of the parent using its Position and Rotation
                let parent_transform = parent_transform.compute_transform();
                let parent_pos =
                    parent_pos.map_or(parent_transform.translation, |pos| pos.0);
                let parent_rot = parent_rot.map_or(parent_transform.rotation, |rot| rot.0);
                let parent_scale = parent_transform.scale;
                let parent_transform = DTransform::from_translation(parent_pos)
                    .with_rotation(parent_rot)
                    .with_scale(parent_scale);

                // The new local transform of the child body,
                // computed from the its global transform and its parents global transform
                let new_transform = DGlobalTransform::from(
                    DTransform::from_translation(pos.0).with_rotation(rot.0),
                )
                .reparented_to(&DGlobalTransform::from(parent_transform));

                transform.translation = new_transform.translation;
                transform.rotation = new_transform.rotation;
            }
        } else {
            transform.translation = pos.0;
            transform.rotation = rot.0;
        }
    }
}

fn update_previous_global_transforms(
    mut bodies: Query<(&DGlobalTransform, &mut PreviousGlobalTransform)>,
) {
    for (transform, mut previous_transform) in &mut bodies {
        previous_transform.0 = *transform;
    }
}