use bevy::{prelude::*, math::{DQuat, DVec3}};
use bevy_transform64::{prelude::{DGlobalTransform, DTransform}, WorldOrigin, SimpleWorldOrigin};
use rapier3d_f64::prelude::ColliderBuilder;
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

fn debug_draw_capsule(
    lines: &mut DebugLines,
    global_transform: &GlobalTransform,
    color: Color,
    shape: &rapier3d_f64::prelude::Capsule,
) {
    let radius = shape.radius as f32;
    let half_height = shape.half_height() as f32;

    let segments = 16;
    let angle_step = 2.0 * std::f32::consts::PI / segments as f32;

    let mut top_circle_points = Vec::new();
    let mut bottom_circle_points = Vec::new();

    for i in 0..segments {
        let angle = i as f32 * angle_step;
        let x = radius * angle.cos();
        let z = radius * angle.sin();

        top_circle_points.push(Vec3::new(x, half_height, z));
        bottom_circle_points.push(Vec3::new(x, -half_height, z));
    }

    // Transform the vertices by the global transform of the capsule
    let transformed_top_circle_points = top_circle_points
        .iter()
        .map(|v| global_transform.transform_point(*v))
        .collect::<Vec<_>>();
    let transformed_bottom_circle_points = bottom_circle_points
        .iter()
        .map(|v| global_transform.transform_point(*v))
        .collect::<Vec<_>>();

    // Draw the top and bottom circles
    for i in 0..segments {
        let next_i = (i + 1) % segments;
        lines.line_colored(
            transformed_top_circle_points[i],
            transformed_top_circle_points[next_i],
            0.0,
            color,
        );
        lines.line_colored(
            transformed_bottom_circle_points[i],
            transformed_bottom_circle_points[next_i],
            0.0,
            color,
        );
    }

    // Draw the lines connecting top and bottom circles
    for i in 0..segments {
        lines.line_colored(
            transformed_top_circle_points[i],
            transformed_bottom_circle_points[i],
            0.0,
            color,
        );
    }
}

pub fn draw_colliders(
    context : Res<RapierContext>,
    world_origin : Res<SimpleWorldOrigin>,
    mut colliders : Query<(&SpaceCollider, &GlobalTransform)>,
    mut lines: ResMut<DebugLines>,
) {
    for (handle, col) in context.collider_set.iter() {
        iter_draw_colliders(col, &world_origin, &mut lines);
    }
}

fn iter_draw_colliders(col: &rapier3d_f64::prelude::Collider, world_origin: &Res<SimpleWorldOrigin>, lines: &mut ResMut<DebugLines>) {
    match col.shape().shape_type() {
        rapier3d_f64::prelude::ShapeType::Ball => {
            if let Some(ball) = col.shape().as_ball() {
                draw_ball(col, world_origin, lines, ball);
            }
        },
        rapier3d_f64::prelude::ShapeType::Cuboid => {
            if let Some(cuboid) = col.shape().as_cuboid() {
                draw_cuboid(col, world_origin, lines, cuboid);
            }
        },
        rapier3d_f64::prelude::ShapeType::Capsule => {
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

            debug_draw_capsule(lines, &GlobalTransform::from(transform), Color::RED, col.shape().as_capsule().unwrap());
        },
        rapier3d_f64::prelude::ShapeType::Segment => todo!(),
        rapier3d_f64::prelude::ShapeType::Triangle => todo!(),
        rapier3d_f64::prelude::ShapeType::TriMesh => todo!(),
        rapier3d_f64::prelude::ShapeType::Polyline => todo!(),
        rapier3d_f64::prelude::ShapeType::HalfSpace => todo!(),
        rapier3d_f64::prelude::ShapeType::HeightField => todo!(),
        rapier3d_f64::prelude::ShapeType::Compound => {
            if let Some(compound) = col.shape().as_compound() {
                for (pos, shape) in compound.shapes() {
                    let collider = ColliderBuilder::new(shape.clone()).position(pos.clone()).build();
                    iter_draw_colliders(&collider, world_origin, lines);
                }
            }
        },
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

fn draw_cuboid(col: &rapier3d_f64::prelude::Collider, world_origin: &Res<SimpleWorldOrigin>, lines: &mut ResMut<DebugLines>, cuboid: &rapier3d_f64::parry::shape::Cuboid) {
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

    let color = if col.parent().is_some() {
        Color::RED 
    } else {
        Color::YELLOW
    };

    debug_draw_cuboid(lines, &GlobalTransform::from(transform), color, cuboid);
}

fn draw_ball(col: &rapier3d_f64::prelude::Collider, world_origin: &Res<SimpleWorldOrigin>, lines: &mut ResMut<DebugLines>, ball: &rapier3d_f64::parry::shape::Ball) {
    let pos = col.position();
    let mut d_translation = DVec3::new(pos.translation.x, pos.translation.y, pos.translation.z) - world_origin.origin;
                    
    let color = if col.parent().is_some() {
        Color::RED 
    } else {
        Color::YELLOW
    };

    debug_draw_circle(
        lines,
        Vec3::new(d_translation.x as f32, d_translation.y as f32, d_translation.z as f32),
        ball.radius as f32,
        color,
        Quat::IDENTITY,
        10
    );

    debug_draw_circle(
        lines,
        Vec3::new(d_translation.x as f32, d_translation.y as f32, d_translation.z as f32),
        ball.radius as f32,
        color,
        Quat::from_euler(EulerRot::XYZ, std::f32::consts::PI / 2.0, 0.0, 0.0),
        10
    );
}