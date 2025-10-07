use std::f32::consts::PI;

use avian3d::prelude::*;
use bevy::{
    color::palettes::basic,
    mesh::VertexAttributeValues,
    prelude::*,
    window::{WindowResized, WindowResolution},
};

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(1280, 720),
                //XXX cli option to choose fullscreen
                // mode: WindowMode::Fullscreen(
                //     MonitorSelection::Current,
                //     VideoModeSelection::Current,
                // ),
                ..default()
            }),
            ..default()
        }),
        PhysicsPlugins::default(),
    ))
    .add_systems(Startup, setup)
    .add_systems(Last, on_resize);

    app.run();
}

const WORLDBOX_DEPTH: f32 = 2.0;

#[derive(Component)]
struct WorldBox;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) -> Result<()> {
    const WORLDBOX_SIZE: f32 = 1.0;
    let mut mesh = Mesh::from(Cuboid::new(WORLDBOX_SIZE, WORLDBOX_SIZE, WORLDBOX_SIZE));
    invert_normals(&mut mesh)?;
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-1., 2., 3.).with_rotation(Quat::from_axis_angle(Vec3::Y, PI)), //XXX random
        RigidBody::Kinematic,
        children![(
            WorldBox,
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(Color::from(basic::PURPLE))),
            Collider::compound(vec![
                // Ceiling
                (
                    Position::from_xyz(0., WORLDBOX_SIZE / 2., 0.),
                    Quat::IDENTITY,
                    Collider::half_space(Vec3::NEG_Y),
                ),
                // Floor
                (
                    Position::from_xyz(0., -WORLDBOX_SIZE / 2., 0.),
                    Quat::IDENTITY,
                    Collider::half_space(Vec3::Y),
                ),
                // Right wall
                (
                    Position::from_xyz(WORLDBOX_SIZE / 2., 0., 0.),
                    Quat::IDENTITY,
                    Collider::half_space(Vec3::NEG_X),
                ),
                // Left wall
                (
                    Position::from_xyz(-WORLDBOX_SIZE / 2., 0., 0.),
                    Quat::IDENTITY,
                    Collider::half_space(Vec3::X),
                ),
                // Back wall
                (
                    Position::from_xyz(0., 0., WORLDBOX_SIZE / 2.),
                    Quat::IDENTITY,
                    Collider::half_space(Vec3::NEG_Z),
                ),
                // Front wall
                (
                    Position::from_xyz(0., 0., -WORLDBOX_SIZE / 2.),
                    Quat::IDENTITY,
                    Collider::half_space(Vec3::Z),
                ),
            ])
        )],
    ));

    Ok(())
}

fn on_resize(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut resize_reader: MessageReader<WindowResized>,
    camera: Single<(&Projection, &GlobalTransform), With<Camera>>,
    worldbox: Single<&mut Transform, With<WorldBox>>,
) {
    if resize_reader.read().last().is_some() {
        let (projection, camera_global_transform) = camera.into_inner();
        let [_, topright, ..] = projection.get_frustum_corners(WORLDBOX_DEPTH, WORLDBOX_DEPTH);
        let mut worldbox_transform = worldbox.into_inner();
        worldbox_transform.translation = Vec3::new(0., 0., -(topright.z + WORLDBOX_DEPTH / 2.));
        worldbox_transform.scale = Vec3::new(topright.x * 2., topright.y * 2., WORLDBOX_DEPTH);

        // XXX spawn a test cube inside the transformed worldbox
        commands.spawn((
            RigidBody::Dynamic,
            Collider::cuboid(0.1, 0.1, 0.1),
            Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.1))),
            MeshMaterial3d(materials.add(Color::from(basic::YELLOW))),
            Transform::from_translation(
                camera_global_transform
                    .transform_point(worldbox_transform.transform_point(Vec3::ZERO)),
            ),
        ));
    }
}

fn invert_normals(mesh: &mut Mesh) -> Result<()> {
    if let Some(VertexAttributeValues::Float32x3(normals)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL)
    {
        for n in normals.iter_mut() {
            n[0] = -n[0];
            n[1] = -n[1];
            n[2] = -n[2];
        }
    }

    mesh.invert_winding()?;
    Ok(())
}
