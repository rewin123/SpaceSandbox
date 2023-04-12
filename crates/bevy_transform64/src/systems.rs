use crate::{components::{DGlobalTransform, DTransform}, WorldOrigin, SimpleWorldOrigin, DTransformBundle};
use bevy::{ecs::{
    change_detection::Ref,
    prelude::{Changed, DetectChanges, Entity, Query, With, Without},
}, prelude::{Added, Commands, GlobalTransform, RemovedComponents, Transform, Res, ResMut}, math::{Affine3A, Vec3A, DAffine3, DVec3}};
use bevy::hierarchy::{Children, Parent};

fn daffine_to_f32(
    affine: &DAffine3,
) -> Affine3A {
    Affine3A {
        matrix3 : bevy::math::Mat3A {
            x_axis : Vec3A::new(affine.matrix3.x_axis.x as f32, affine.matrix3.x_axis.y as f32, affine.matrix3.x_axis.z as f32),
            y_axis : Vec3A::new(affine.matrix3.y_axis.x as f32, affine.matrix3.y_axis.y as f32, affine.matrix3.y_axis.z as f32),
            z_axis : Vec3A::new(affine.matrix3.z_axis.x as f32, affine.matrix3.z_axis.y as f32, affine.matrix3.z_axis.z as f32),
        },
        translation : Vec3A::new(affine.translation.x as f32, affine.translation.y as f32, affine.translation.z as f32),
    }
}

pub fn sync_f64_f32(
    mut commands : Commands,
    mut query: Query<(Entity, &DGlobalTransform), (Added<DGlobalTransform>, Without<GlobalTransform>)>,
    mut query_changed: Query<(Entity, &DGlobalTransform, &mut GlobalTransform)>,
    mut query_changed_tranform: Query<(Entity, &DTransform, &mut Transform, Option<&Parent>)>,
    mut query_deleted: Query<Entity, (With<GlobalTransform>, Without<DGlobalTransform>)>,
    mut query_cmd_add : Query<(Entity, &Transform), (Without<DTransform>)>,
    world_origin : Res<WorldOrigin>,
) {

    let world_origin_pos = match world_origin.clone() {
        WorldOrigin::Entity(e) => {
            if let Ok((_, transform, _)) = query_changed.get_mut(e) {
                transform.translation().clone()
            } else {
                println!("Entity not found in sync_f64_f32");
                DVec3::ZERO
            }
        },
        WorldOrigin::Position(pos) => pos,
    };

    for (entity, global_transform) in query.iter_mut() {
        let mut affine = global_transform.affine();
        affine.translation -= world_origin_pos;
        let affine_f32 = daffine_to_f32(&affine);
    
        commands.entity(entity).insert(
            bevy::prelude::GlobalTransform::from(affine_f32)
        );
    }

    for (entity, d_global_transform, mut global_transforms) in query_changed.iter_mut() {
        let mut affine = d_global_transform.affine();
        affine.translation -= world_origin_pos;
        let affine_f32 = daffine_to_f32(&affine);
        // println!("{:?}", affine_f32);
        *global_transforms = GlobalTransform::from(affine_f32);
    }

    for (entity, d_transform, mut transform, parent) in query_changed_tranform.iter_mut() {
        if let Some(parent) = parent {
            transform.translation = d_transform.translation.as_vec3();
        } else {
            transform.translation = (d_transform.translation - world_origin_pos).as_vec3();
        }
        transform.scale = d_transform.scale.as_vec3();
        transform.rotation = d_transform.rotation.as_f32();
        // println!("{:?}", d_transform);
    }

    for entity in query_deleted.iter() {
        commands.entity(entity).remove::<GlobalTransform>();
    }

    for (entity, f32_transform) in query_cmd_add.iter() {
        // let mut rot = f32_transform.rotation.as_f64();
        // if f32_transform.rotation.xyz().is_nan() {
        //     rot = bevy::math::DQuat::default();
        // }
        println!("Spawn dtransform with {:?}", f32_transform.rotation.as_f64());
        commands.entity(entity).insert(
            DTransformBundle::from_transform(DTransform {
                translation : f32_transform.translation.as_dvec3(),
                rotation: f32_transform.rotation.as_f64(),
                scale: f32_transform.scale.as_dvec3(),
            })
        ).remove::<Transform>();
    }
}

/// Update [`GlobalTransform`] component of entities that aren't in the hierarchy
///
/// Third party plugins should ensure that this is used in concert with [`propagate_transforms`].
pub fn sync_simple_transforms(
    mut query: Query<
        (&DTransform, &mut DGlobalTransform),
        (Changed<DTransform>, Without<Parent>, Without<Children>),
    >,
) {
    query
        .par_iter_mut()
        .for_each_mut(|(transform, mut global_transform)| {
            *global_transform = DGlobalTransform::from(*transform);
        });
}

/// Update [`GlobalTransform`] component of entities based on entity hierarchy and
/// [`Transform`] component.
///
/// Third party plugins should ensure that this is used in concert with [`sync_simple_transforms`].
pub fn propagate_transforms(
    mut root_query: Query<
        (Entity, &Children, Ref<DTransform>, &mut DGlobalTransform),
        Without<Parent>,
    >,
    transform_query: Query<(Ref<DTransform>, &mut DGlobalTransform, Option<&Children>), With<Parent>>,
    parent_query: Query<(Entity, Ref<Parent>)>,
) {
    root_query.par_iter_mut().for_each_mut(
        |(entity, children, transform, mut global_transform)| {
            let changed = transform.is_changed();
            if changed {
                *global_transform = DGlobalTransform::from(*transform);
            }

            for (child, actual_parent) in parent_query.iter_many(children) {
                assert_eq!(
                    actual_parent.get(), entity,
                    "Malformed hierarchy. This probably means that your hierarchy has been improperly maintained, or contains a cycle"
                );
                // SAFETY:
                // - `child` must have consistent parentage, or the above assertion would panic.
                // Since `child` is parented to a root entity, the entire hierarchy leading to it is consistent.
                // - We may operate as if all descendants are consistent, since `propagate_recursive` will panic before 
                //   continuing to propagate if it encounters an entity with inconsistent parentage.
                // - Since each root entity is unique and the hierarchy is consistent and forest-like,
                //   other root entities' `propagate_recursive` calls will not conflict with this one.
                // - Since this is the only place where `transform_query` gets used, there will be no conflicting fetches elsewhere.
                unsafe {
                    propagate_recursive(
                        &global_transform,
                        &transform_query,
                        &parent_query,
                        child,
                        changed || actual_parent.is_changed(),
                    );
                }
            }
        },
    );
}

/// Recursively propagates the transforms for `entity` and all of its descendants.
///
/// # Panics
///
/// If `entity`'s descendants have a malformed hierarchy, this function will panic occur before propagating
/// the transforms of any malformed entities and their descendants.
///
/// # Safety
///
/// - While this function is running, `transform_query` must not have any fetches for `entity`,
/// nor any of its descendants.
/// - The caller must ensure that the hierarchy leading to `entity`
/// is well-formed and must remain as a tree or a forest. Each entity must have at most one parent.
unsafe fn propagate_recursive(
    parent: &DGlobalTransform,
    transform_query: &Query<
        (Ref<DTransform>, &mut DGlobalTransform, Option<&Children>),
        With<Parent>,
    >,
    parent_query: &Query<(Entity, Ref<Parent>)>,
    entity: Entity,
    mut changed: bool,
) {
    let (global_matrix, children) = {
        let Ok((transform, mut global_transform, children)) =
            // SAFETY: This call cannot create aliased mutable references.
            //   - The top level iteration parallelizes on the roots of the hierarchy.
            //   - The caller ensures that each child has one and only one unique parent throughout the entire
            //     hierarchy.
            //
            // For example, consider the following malformed hierarchy:
            //
            //     A
            //   /   \
            //  B     C
            //   \   /
            //     D
            //
            // D has two parents, B and C. If the propagation passes through C, but the Parent component on D points to B,
            // the above check will panic as the origin parent does match the recorded parent.
            //
            // Also consider the following case, where A and B are roots:
            //
            //  A       B
            //   \     /
            //    C   D
            //     \ /
            //      E
            //
            // Even if these A and B start two separate tasks running in parallel, one of them will panic before attempting
            // to mutably access E.
            (unsafe { transform_query.get_unchecked(entity) }) else {
                return;
            };

        changed |= transform.is_changed();
        if changed {
            *global_transform = parent.mul_transform(*transform);
        }
        (*global_transform, children)
    };

    let Some(children) = children else { return };
    for (child, actual_parent) in parent_query.iter_many(children) {
        assert_eq!(
            actual_parent.get(), entity,
            "Malformed hierarchy. This probably means that your hierarchy has been improperly maintained, or contains a cycle"
        );
        // SAFETY: The caller guarantees that `transform_query` will not be fetched
        // for any descendants of `entity`, so it is safe to call `propagate_recursive` for each child.
        //
        // The above assertion ensures that each child has one and only one unique parent throughout the
        // entire hierarchy.
        unsafe {
            propagate_recursive(
                &global_matrix,
                transform_query,
                parent_query,
                child,
                changed || actual_parent.is_changed(),
            );
        }
    }
}

pub fn convert_world_origin(
    world_origin : Res<WorldOrigin>,
    query: Query<&DGlobalTransform>,
    mut simple_world_origin : ResMut<SimpleWorldOrigin>
) {
    match *world_origin {
        WorldOrigin::Entity(e) => {
            if let Ok(transform) = query.get(e) {
                simple_world_origin.origin = transform.translation();
            } else {
                simple_world_origin.origin = DVec3::new(0.0, 0.0, 0.0);
            }
        },
        WorldOrigin::Position(pos) => {
            simple_world_origin.origin = DVec3::new(pos.x, pos.y, pos.z);
        },
    };
}

pub fn replace_transforms(
    mut commands : Commands,
    mut query: Query<(&mut Transform, &DTransform), Without<Parent>>,
    simple_world_origin : Res<SimpleWorldOrigin>
) {
    // for (mut transform, dtransform) in query.iter_mut() {
    //     dtransform.set_f32_transform(&mut transform, simple_world_origin.origin);
    // }
    // for (entity, transform) in query.iter() {
    //     let dtransform = DTransform {
    //         translation: DVec3::new(transform.translation.x as f64, transform.translation.y as f64, transform.translation.z as f64),
    //         scale: DVec3::new(transform.scale.x as f64, transform.scale.y as f64, transform.scale.z as f64),
    //         rotation: bevy::math::DQuat { x: transform.rotation.x as f64, y: transform.rotation.y as f64, z: transform.rotation.z as f64, w: transform.rotation.w as f64 },
    //     };

    //     commands.entity(entity).insert(DTransformBundle::from_transform(dtransform)).remove::<Transform>().insert(GlobalTransform::default());

    //     println!("Remove with creation transform of {:?}", transform);
    // }

    // for (entity, transform) in del_query.iter() {
    //     commands.entity(entity).remove::<Transform>().insert(GlobalTransform::default());
    //     println!("Remove transform of {:?}", entity);
    // }
}

#[cfg(test)]
mod test {
    use bevy::app::prelude::*;
    use bevy::ecs::prelude::*;
    use bevy::ecs::system::CommandQueue;
    use bevy::math::{vec3, dvec3};
    use bevy::tasks::{ComputeTaskPool, TaskPool};

    use crate::components::{DGlobalTransform, DTransform};
    use crate::{systems::*, DTransformBundle};
    use crate::TransformBundle;
    use bevy::hierarchy::{BuildChildren, BuildWorldChildren, Children, Parent};

    #[test]
    fn did_propagate() {
        ComputeTaskPool::init(TaskPool::default);
        let mut world = World::default();

        let mut schedule = Schedule::new();
        schedule.add_systems((sync_simple_transforms, propagate_transforms));

        // Root entity
        world.spawn(DTransformBundle::from(DTransform::from_xyz(1.0, 0.0, 0.0)));

        let mut children = Vec::new();
        world
            .spawn(DTransformBundle::from(DTransform::from_xyz(1.0, 0.0, 0.0)))
            .with_children(|parent| {
                children.push(
                    parent
                        .spawn(DTransformBundle::from(DTransform::from_xyz(0.0, 2.0, 0.)))
                        .id(),
                );
                children.push(
                    parent
                        .spawn(DTransformBundle::from(DTransform::from_xyz(0.0, 0.0, 3.)))
                        .id(),
                );
            });
        schedule.run(&mut world);

        assert_eq!(
            *world.get::<DGlobalTransform>(children[0]).unwrap(),
            DGlobalTransform::from_xyz(1.0, 0.0, 0.0) * DTransform::from_xyz(0.0, 2.0, 0.0)
        );

        assert_eq!(
            *world.get::<DGlobalTransform>(children[1]).unwrap(),
            DGlobalTransform::from_xyz(1.0, 0.0, 0.0) * DTransform::from_xyz(0.0, 0.0, 3.0)
        );
    }

    #[test]
    fn did_propagate_command_buffer() {
        let mut world = World::default();

        let mut schedule = Schedule::new();
        schedule.add_systems((sync_simple_transforms, propagate_transforms));

        // Root entity
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);
        let mut children = Vec::new();
        commands
            .spawn(DTransformBundle::from(DTransform::from_xyz(1.0, 0.0, 0.0)))
            .with_children(|parent| {
                children.push(
                    parent
                        .spawn(DTransformBundle::from(DTransform::from_xyz(0.0, 2.0, 0.0)))
                        .id(),
                );
                children.push(
                    parent
                        .spawn(DTransformBundle::from(DTransform::from_xyz(0.0, 0.0, 3.0)))
                        .id(),
                );
            });
        queue.apply(&mut world);
        schedule.run(&mut world);

        assert_eq!(
            *world.get::<DGlobalTransform>(children[0]).unwrap(),
            DGlobalTransform::from_xyz(1.0, 0.0, 0.0) * DTransform::from_xyz(0.0, 2.0, 0.0)
        );

        assert_eq!(
            *world.get::<DGlobalTransform>(children[1]).unwrap(),
            DGlobalTransform::from_xyz(1.0, 0.0, 0.0) * DTransform::from_xyz(0.0, 0.0, 3.0)
        );
    }

    #[test]
    fn correct_children() {
        ComputeTaskPool::init(TaskPool::default);
        let mut world = World::default();

        let mut schedule = Schedule::new();
        schedule.add_systems((sync_simple_transforms, propagate_transforms));

        // Add parent entities
        let mut children = Vec::new();
        let parent = {
            let mut command_queue = CommandQueue::default();
            let mut commands = Commands::new(&mut command_queue, &world);
            let parent = commands.spawn(DTransform::from_xyz(1.0, 0.0, 0.0)).id();
            commands.entity(parent).with_children(|parent| {
                children.push(parent.spawn(DTransform::from_xyz(0.0, 2.0, 0.0)).id());
                children.push(parent.spawn(DTransform::from_xyz(0.0, 3.0, 0.0)).id());
            });
            command_queue.apply(&mut world);
            schedule.run(&mut world);
            parent
        };

        assert_eq!(
            world
                .get::<Children>(parent)
                .unwrap()
                .iter()
                .cloned()
                .collect::<Vec<_>>(),
            children,
        );

        // Parent `e1` to `e2`.
        {
            let mut command_queue = CommandQueue::default();
            let mut commands = Commands::new(&mut command_queue, &world);
            commands.entity(children[1]).add_child(children[0]);
            command_queue.apply(&mut world);
            schedule.run(&mut world);
        }

        assert_eq!(
            world
                .get::<Children>(parent)
                .unwrap()
                .iter()
                .cloned()
                .collect::<Vec<_>>(),
            vec![children[1]]
        );

        assert_eq!(
            world
                .get::<Children>(children[1])
                .unwrap()
                .iter()
                .cloned()
                .collect::<Vec<_>>(),
            vec![children[0]]
        );

        assert!(world.despawn(children[0]));

        schedule.run(&mut world);

        assert_eq!(
            world
                .get::<Children>(parent)
                .unwrap()
                .iter()
                .cloned()
                .collect::<Vec<_>>(),
            vec![children[1]]
        );
    }

    #[test]
    fn correct_transforms_when_no_children() {
        let mut app = App::new();
        ComputeTaskPool::init(TaskPool::default);

        app.add_systems((sync_simple_transforms, propagate_transforms));

        let translation = dvec3(1.0, 0.0, 0.0);

        // These will be overwritten.
        let mut child = Entity::from_raw(0);
        let mut grandchild = Entity::from_raw(1);
        let parent = app
            .world
            .spawn((
                DTransform::from_translation(translation),
                DGlobalTransform::IDENTITY,
            ))
            .with_children(|builder| {
                child = builder
                    .spawn(TransformBundle::IDENTITY)
                    .with_children(|builder| {
                        grandchild = builder.spawn(TransformBundle::IDENTITY).id();
                    })
                    .id();
            })
            .id();

        app.update();

        // check the `Children` structure is spawned
        assert_eq!(&**app.world.get::<Children>(parent).unwrap(), &[child]);
        assert_eq!(&**app.world.get::<Children>(child).unwrap(), &[grandchild]);
        // Note that at this point, the `GlobalTransform`s will not have updated yet, due to `Commands` delay
        app.update();

        let mut state = app.world.query::<&DGlobalTransform>();
        for global in state.iter(&app.world) {
            assert_eq!(global, &DGlobalTransform::from_translation(translation));
        }
    }

    #[test]
    #[should_panic]
    fn panic_when_hierarchy_cycle() {
        ComputeTaskPool::init(TaskPool::default);
        // We cannot directly edit Parent and Children, so we use a temp world to break
        // the hierarchy's invariants.
        let mut temp = World::new();
        let mut app = App::new();

        app.add_systems((propagate_transforms, sync_simple_transforms));

        fn setup_world(world: &mut World) -> (Entity, Entity) {
            let mut grandchild = Entity::from_raw(0);
            let child = world
                .spawn(DTransformBundle::IDENTITY)
                .with_children(|builder| {
                    grandchild = builder.spawn(DTransformBundle::IDENTITY).id();
                })
                .id();
            (child, grandchild)
        }

        let (temp_child, temp_grandchild) = setup_world(&mut temp);
        let (child, grandchild) = setup_world(&mut app.world);

        assert_eq!(temp_child, child);
        assert_eq!(temp_grandchild, grandchild);

        app.world
            .spawn(DTransformBundle::IDENTITY)
            .push_children(&[child]);
        std::mem::swap(
            &mut *app.world.get_mut::<Parent>(child).unwrap(),
            &mut *temp.get_mut::<Parent>(grandchild).unwrap(),
        );

        app.update();
    }
}