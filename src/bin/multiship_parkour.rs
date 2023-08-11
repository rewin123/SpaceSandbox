use SpaceSandbox::ext::*;
use SpaceSandbox::{prelude::*, ship::save_load::DiskShipBase64, scenes::{NotificationPlugin, fps_mode::{FPSPlugin, self}, settings::SettingsPlugin}, pawn_system::{PawnPlugin, ChangePawn}, control::SpaceControlPlugin, objects::prelude::{GravityGenerator, GravityGeneratorPlugin}};



#[derive(Component)]
struct RotateMe {
    pub ang_vel : DVec3
}

fn main() {
    App::default()
        .add_plugins(SpaceExamplePlguin)
        .register_type::<DiskShipBase64>()
        .add_plugins(FPSPlugin)
        .add_plugins(SpaceSandbox::ship::common::VoxelInstancePlugin)
        .add_plugins(NotificationPlugin)
        .add_plugins(PawnPlugin)
        .add_plugins(SpaceControlPlugin)
        .add_plugins(SettingsPlugin)
        .add_plugins(GravityGeneratorPlugin)

        .add_systems(Startup, startup)
        .add_systems(Startup, startup_player)

        .add_systems(Update, fps_mode::show_controller_settings)
        .add_systems(Update, rotate_my_system)

        .run();
}



fn startup_player(
    mut commands : Commands,
    mut pawn_event : EventWriter<ChangePawn>,
) {
    fps_mode::startup_player(&mut commands, &mut pawn_event);
}

fn startup(
    mut commands : Commands,
    mut meshes : ResMut<Assets<Mesh>>,
    mut materials : ResMut<Assets<StandardMaterial>>,
) {
    prepare_enviroment(
        &mut meshes,
        &mut materials, 
        &mut commands);
    spawn_ship(
        DVec3::new(11.0, -0.5, 0.0),
        &mut meshes,
        &mut materials,
        &mut commands);
    
    spawn_ship(
        DVec3::new(11.0, 11.0, 0.0),
        &mut meshes,
        &mut materials,
        &mut commands);
}

fn spawn_ship(
    pos : DVec3,
    mut meshes: &mut Assets<Mesh>, 
    mut materials: &mut Assets<StandardMaterial>, 
    mut commands: &mut Commands) {

    let plane_mesh = meshes.add(
        Mesh::from(bevy::prelude::shape::Box::new(10.0, 0.5, 10.0))
    );

    let mat = materials.add(
        StandardMaterial {
            base_color : Color::DARK_GREEN,
            ..default()
        }
    );

    commands.spawn(
        PbrBundle {
            mesh: plane_mesh.clone(),
            material : mat.clone(),
            ..default()
        }
    ).insert(DTransformBundle::from_transform(
        DTransform::from_xyz(pos.x, pos.y, pos.z)
    ))
    .insert(Position(pos))
    .insert(
        Collider::cuboid(10.0, 0.25, 10.0)
    )
    .insert(RigidBody::Kinematic)
    .insert(GravityScale(0.0))
    .insert(
        RotateMe {
            ang_vel : DVec3::new(0.3, 0.0, 0.0)
        }
    )
    .insert(GravityGenerator {
        gravity_force: DVec3::new(0.0, -9.81, 0.0),
        radius: 5.0,
    });
}

fn rotate_my_system(
    mut query : Query<(&RotateMe, &mut AngularVelocity)>,

) {
    for (me, mut vel) in query.iter_mut() {
        vel.0 = me.ang_vel;
    }
}

fn prepare_enviroment(mut meshes: &mut Assets<Mesh>, mut materials: &mut Assets<StandardMaterial>, commands: &mut Commands) {
    let plane_mesh = meshes.add(
        Mesh::from(bevy::prelude::shape::Box::new(10.0, 0.5, 10.0))
    );
    let cube_mesh = meshes.add(
        Mesh::from(bevy::prelude::shape::Cube::new(1.0))
    );
    let mat = materials.add(
        StandardMaterial {
            base_color : Color::GRAY,
            ..default()
        }
    );

    commands.spawn(
        PbrBundle {
            mesh: plane_mesh.clone(),
            material : mat.clone(),
            ..default()
        }
    ).insert(DTransformBundle::from_transform(
        DTransform::from_xyz(0.0, -0.5, 0.0)
    ))
    .insert(Position(DVec3::new(0.0, -0.5, 0.0)))
    .insert(RigidBody::Static)
    .insert(
        Collider::cuboid(10.0, 0.5, 10.0)
    );

    let cube_poses = vec![
        DVec3::new(5.0, 0.0, 0.0),
        DVec3::new(-5.0, 0.0, 0.0),
        DVec3::new(0.0, 0.0, 5.0),
        DVec3::new(0.0, 0.0, -5.0),
    ];
    let cube_transforms = cube_poses.iter().map(|pos| {
        DTransformBundle::from_transform(
            DTransform::from_xyz(pos.x, pos.y, pos.z)
        )
    }).collect::<Vec<_>>();

    for cube_transform in cube_transforms {
        commands.spawn(PbrBundle {
            mesh: cube_mesh.clone(),
            material : mat.clone(),
            ..default()
        }).insert(cube_transform)
        .insert(RigidBody::Static)
        .insert(Position(cube_transform.local.translation))
        .insert(Collider::cuboid(1.0, 1.0, 1.0));
    }
    
    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadows_enabled: true,
            ..default()
        },
        ..default()
    }).insert(DTransformBundle::from_transform(
        DTransform::from_xyz(10.0, 10.0, 10.0).looking_at(DVec3::new(0.0, 0.0, 0.0), DVec3::Y)
    ));
}