use bevy::{prelude::*, math::DVec3};
use bevy_transform64::{prelude::*, WorldOrigin};
use bevy_egui::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build().disable::<TransformPlugin>())
        .add_plugins(DTransformPlugin)
        .add_plugins(bevy_egui::EguiPlugin)

        .add_systems(Startup,(setup, apply_deferred).chain())
        .add_systems(Update, debug_gui)
        .add_systems(Update, fly)
        .add_systems(Update, camera_orbit)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    let mesh = meshes.add(Mesh::from(shape::Cube { size: 0.5 }));
    let material = materials.add(Color::rgb(0.8, 0.7, 0.6).into());

    let base_cube = commands.spawn(PbrBundle {
        mesh: mesh.clone(),
        material: material.clone(),
        visibility : Visibility::Visible,
        ..Default::default()
    }).insert(DTransformBundle::from_transform(DTransform::from_xyz(0.0, 0.0, 0.0))).id();

    let base_cube_2 = commands.spawn(PbrBundle {
        mesh: mesh.clone(),
        material: material.clone(),
        visibility : Visibility::Visible,
        transform: Transform::from_xyz(0.0, 1.0, 0.0),
        ..Default::default()
    }).insert(DTransformBundle::from_transform(DTransform::from_xyz(0.0, 1.0, 0.0))).id();

    
    let base_cube_3 = commands.spawn(PbrBundle {
        mesh: mesh.clone(),
        material: material.clone(),
        visibility : Visibility::Visible,
        transform: Transform::from_xyz(0.0, 1.0, 0.0),
        ..Default::default()
    }).insert(DTransformBundle::from_transform(DTransform::from_xyz(0.0, 1.0, 0.0))).id();

    commands.entity(base_cube).add_child(base_cube_2);
    commands.entity(base_cube_2).add_child(base_cube_3);

    let camera = commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(5.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    }).insert(DTransformBundle::from_transform(DTransform::from_xyz(5.0, 0.0, 5.0).looking_at(DVec3::ZERO, DVec3::Y))).id();

    commands.insert_resource(WorldOrigin::Entity(camera));

}

fn fly(
    _transform : Query<&mut Transform, Without<Parent>>,
    mut dtransforms : Query<&mut DTransform, Without<Parent>>,
    time : Res<Time>
) {
    let speed = 10000.0;
    // for mut transform in transform.iter_mut() {
    //     transform.translation.y += speed * time.delta_seconds();
    // }
    for mut dtransform in dtransforms.iter_mut() {
        dtransform.translation.y += (speed * time.delta_seconds()) as f64;
    }
}

fn camera_orbit(
    _transform : Query<&mut Transform, With<Camera>>,
    mut dtransforms : Query<&mut DTransform, With<Camera>>,
    time : Res<Time>
) {
    let speed = 1.0;
    // for mut transform in transform.iter_mut() {
    //     let angle = speed * time.elapsed_seconds();
    //     let x = angle.cos() * 5.0;
    //     let z = angle.sin() * 5.0;
    //     transform.translation.x = x;
    //     transform.translation.z = z;
    //     let target = Vec3::new(0.0, transform.translation.y, 0.0);
    //     transform.look_at(target, Vec3::Y);
    // }
    for mut dtransform in dtransforms.iter_mut() {
        let angle = speed * time.elapsed_seconds();
        let x = angle.cos() * 5.0;
        let z = angle.sin() * 5.0;
        dtransform.translation.x = x as f64;
        dtransform.translation.z = z as f64;
        let target = DVec3::new(0.0, dtransform.translation.y, 0.0);
        dtransform.look_at(target, DVec3::Y);
    }
}

fn debug_gui(
    mut egui_ctxs : Query<&mut EguiContext>,
    world_origin : ResMut<WorldOrigin>,
    dtransforms : Query<&DTransform>,
    _transforms : Query<&Transform>,
) {
    egui::SidePanel::left("debug").show(egui_ctxs.single_mut().get_mut(), |ui| {
        ui.label("World Origin");
        ui.label(format!("{:?}", world_origin));

        if let WorldOrigin::Entity(entity) = *world_origin {
            // let pos = transforms.get(entity).unwrap().translation;
            let dpos = dtransforms.get(entity).unwrap().translation;

            // let dist = pos.length();
            let ddist = dpos.length();

            // ui.label(format!("f32 dist: {:?}", dist));
            ui.label(format!("f64 dist: {:?}", ddist));
        }        
    });
}
