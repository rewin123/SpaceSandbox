use bevy::{prelude::*, math::{DQuat, DVec3}};
use bevy_transform64::{prelude::{DGlobalTransform, DTransform}, WorldOrigin, SimpleWorldOrigin};
use crate::prelude::{SpaceCollider, RapierContext};

use super::components::*;
use bevy_prototype_debug_lines::*;

fn debug_draw_circle(
    lines : &mut DebugLines,
    origin : Vec3,
    radius : f32,
    color : Color,
    rotation : Quat,
    segments : u32,
) {
    
    let mut points = Vec::new();
    for i in 0..segments {
        let angle = i as f32 * 2.0 * std::f32::consts::PI / segments as f32;
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        points.push(Vec2::new(x, y));
    }

    //rotate circle
    let mut points_rotated = Vec::new();
    for point in points {
        let point_rotated = rotation * Vec3::new(point.x, point.y, 0.0) + origin;
        points_rotated.push(point_rotated);
    }

    for idx in 0..(points_rotated.len() - 1) {
        lines.line_colored(
            points_rotated[idx],
            points_rotated[idx + 1],
            0.0,
            color
        );
    }
}

fn debug_draw_cuboid(
    lines : &mut DebugLines,
    global_transform : &GlobalTransform,
    color : Color,
    shape : &rapier3d_f64::prelude::Cuboid,
) {

    let half_extents = Vec3::new(shape.half_extents.x as f32, shape.half_extents.y as f32, shape.half_extents.z as f32);

    let vertices = [
        Vec3::new(half_extents.x, half_extents.y, half_extents.z),
        Vec3::new(-half_extents.x, half_extents.y, half_extents.z),
        Vec3::new(half_extents.x, -half_extents.y, half_extents.z),
        Vec3::new(-half_extents.x, -half_extents.y, half_extents.z),
        Vec3::new(half_extents.x, half_extents.y, -half_extents.z),
        Vec3::new(-half_extents.x, half_extents.y, -half_extents.z),
        Vec3::new(half_extents.x, -half_extents.y, -half_extents.z),
        Vec3::new(-half_extents.x, -half_extents.y, -half_extents.z),
    ];

    // Transform the vertices by the global transform of the cuboid
    let transformed_vertices = vertices
        .iter()
        .map(|v| global_transform.transform_point(*v))
        .collect::<Vec<_>>();

    // Draw the edges of the cuboid using the transformed vertices
    lines.line_colored(
        transformed_vertices[0],
        transformed_vertices[1],
        0.0,
        color,
    );
    lines.line_colored(
        transformed_vertices[1],
        transformed_vertices[3],
        0.0,
        color,
    );
    lines.line_colored(
        transformed_vertices[3],
        transformed_vertices[2],
        0.0,
        color,
    );
    lines.line_colored(
        transformed_vertices[2],
        transformed_vertices[0],
        0.0,
        color,
    );
    lines.line_colored(
        transformed_vertices[0],
        transformed_vertices[4],
        0.0,
        color,
    );
    lines.line_colored(
        transformed_vertices[1],
        transformed_vertices[5],
        0.0,
        color,
    );
    lines.line_colored(
        transformed_vertices[3],
        transformed_vertices[7],
        0.0,
        color,
    );
    lines.line_colored(
        transformed_vertices[2],
        transformed_vertices[6],
        0.0,
        color,
    );
    lines.line_colored(
        transformed_vertices[4],
        transformed_vertices[5],
        0.0,
        color,
    );
    lines.line_colored(
        transformed_vertices[5],
        transformed_vertices[7],
        0.0,
        color,
    );
    lines.line_colored(
        transformed_vertices[7],
        transformed_vertices[6],
        0.0,
        color,
    );
    lines.line_colored(
        transformed_vertices[6],
        transformed_vertices[4],
        0.0,
        color,
    );
}

pub fn draw_colliders(
    context : Res<RapierContext>,
    world_origin : Res<SimpleWorldOrigin>,
    mut colliders : Query<(&SpaceCollider, &GlobalTransform)>,
    mut lines: ResMut<DebugLines>,
) {

    for (handle, col) in context.collider_set.iter() {
        match col.shape().shape_type() {
            rapier3d_f64::prelude::ShapeType::Ball => {
                if let Some(ball) = col.shape().as_ball() {
                    let pos = col.position();
                    let mut d_translation = DVec3::new(pos.translation.x, pos.translation.y, pos.translation.z) - world_origin.origin;

                    debug_draw_circle(
                        &mut lines,
                        Vec3::new(d_translation.x as f32, d_translation.y as f32, d_translation.z as f32),
                        ball.radius as f32,
                        Color::YELLOW,
                        Quat::IDENTITY,
                        10
                    );

                    debug_draw_circle(
                        &mut lines,
                        Vec3::new(d_translation.x as f32, d_translation.y as f32, d_translation.z as f32),
                        ball.radius as f32,
                        Color::YELLOW,
                        Quat::from_euler(EulerRot::XYZ, std::f32::consts::PI / 2.0, 0.0, 0.0),
                        10
                    );
                }
            },
            rapier3d_f64::prelude::ShapeType::Cuboid => {
                if let Some(cuboid) = col.shape().as_cuboid() {
                    let pos = col.position();
                    let mut d_translation = DVec3::new(pos.translation.x, pos.translation.y, pos.translation.z) - world_origin.origin;
                    let rot = col.rotation();
                    let rot = bevy::math::Quat::from_xyzw(
                        rot.i as f32,
                        rot.j as f32,
                        rot.k as f32,
                        rot.w as f32,
                    );

                    let mut transform = Transform::from_translation(Vec3::new(d_translation.x as f32, d_translation.y as f32, d_translation.z as f32));
                    transform = transform.with_rotation(rot);

                    debug_draw_cuboid(&mut lines, &GlobalTransform::from(transform), Color::YELLOW, cuboid);

                }
            },
            rapier3d_f64::prelude::ShapeType::Capsule => {
                
            },
            rapier3d_f64::prelude::ShapeType::Segment => todo!(),
            rapier3d_f64::prelude::ShapeType::Triangle => todo!(),
            rapier3d_f64::prelude::ShapeType::TriMesh => todo!(),
            rapier3d_f64::prelude::ShapeType::Polyline => todo!(),
            rapier3d_f64::prelude::ShapeType::HalfSpace => todo!(),
            rapier3d_f64::prelude::ShapeType::HeightField => todo!(),
            rapier3d_f64::prelude::ShapeType::Compound => todo!(),
            rapier3d_f64::prelude::ShapeType::ConvexPolyhedron => todo!(),
            rapier3d_f64::prelude::ShapeType::Cylinder => todo!(),
            rapier3d_f64::prelude::ShapeType::Cone => todo!(),
            rapier3d_f64::prelude::ShapeType::RoundCuboid => todo!(),
            rapier3d_f64::prelude::ShapeType::RoundTriangle => todo!(),
            rapier3d_f64::prelude::ShapeType::RoundCylinder => todo!(),
            rapier3d_f64::prelude::ShapeType::RoundCone => todo!(),
            rapier3d_f64::prelude::ShapeType::RoundConvexPolyhedron => todo!(),
            rapier3d_f64::prelude::ShapeType::Custom => todo!(),
        }
    }

    // println!("Draw colliders");
    // for (collider, transform) in colliders.iter() {
    //     match collider.collider.shape().shape_type() {
    //         rapier3d_f64::prelude::ShapeType::Ball => {
    //             if let Some(ball) = collider.collider.shape().as_ball() {
    //                 debug_draw_circle(
    //                     &mut lines,
    //                     transform.translation(),
    //                     ball.radius as f32,
    //                     Color::YELLOW,
    //                     Quat::IDENTITY,
    //                     20
    //                 );
    //                 //second circle
    //                 debug_draw_circle(
    //                     &mut lines,
    //                     transform.translation(),
    //                     ball.radius as f32,
    //                     Color::YELLOW,
    //                     Quat::from_euler(EulerRot::XYZ, std::f32::consts::PI / 2.0, 0.0, 0.0),
    //                     10
    //                 );
    //             }
                
    //         },
    //         rapier3d_f64::prelude::ShapeType::Cuboid => {
    //             if let Some(cuboid) = collider.collider.shape().as_cuboid() {
    //                 debug_draw_cuboid(
    //                     &mut lines,
    //                     transform,
    //                     Color::YELLOW,
    //                     cuboid
    //                 );
    //             }
    //         },
    //         rapier3d_f64::prelude::ShapeType::Capsule => todo!(),
    //         rapier3d_f64::prelude::ShapeType::Segment => todo!(),
    //         rapier3d_f64::prelude::ShapeType::Triangle => todo!(),
    //         rapier3d_f64::prelude::ShapeType::TriMesh => todo!(),
    //         rapier3d_f64::prelude::ShapeType::Polyline => todo!(),
    //         rapier3d_f64::prelude::ShapeType::HalfSpace => todo!(),
    //         rapier3d_f64::prelude::ShapeType::HeightField => todo!(),
    //         rapier3d_f64::prelude::ShapeType::Compound => todo!(),
    //         rapier3d_f64::prelude::ShapeType::ConvexPolyhedron => todo!(),
    //         rapier3d_f64::prelude::ShapeType::Cylinder => todo!(),
    //         rapier3d_f64::prelude::ShapeType::Cone => todo!(),
    //         rapier3d_f64::prelude::ShapeType::RoundCuboid => todo!(),
    //         rapier3d_f64::prelude::ShapeType::RoundTriangle => todo!(),
    //         rapier3d_f64::prelude::ShapeType::RoundCylinder => todo!(),
    //         rapier3d_f64::prelude::ShapeType::RoundCone => todo!(),
    //         rapier3d_f64::prelude::ShapeType::RoundConvexPolyhedron => todo!(),
    //         rapier3d_f64::prelude::ShapeType::Custom => {},
    //     }
    // }
}