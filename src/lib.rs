use std::f32::consts::PI;

use avian3d::{
    math::{Scalar, Vector},
    prelude::*,
};
use bevy::{
    color::palettes::basic,
    mesh::VertexAttributeValues,
    prelude::*,
    window::{WindowResized, WindowResolution},
};

#[cfg(feature = "web")]
mod web;

pub fn start() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(720, 1280),
                resizable: false,
                canvas: Some("canvas".into()),
                ..default()
            }),
            ..default()
        }),
        PhysicsPlugins::default(),
        #[cfg(feature = "web")]
        web::plugin,
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, (apply_forces, log_worldbox))
    .add_systems(Last, on_resize);

    app.run();
}

const WORLDBOX_DEPTH: f32 = 2.0;

#[derive(Component, Copy, Clone)]
struct WorldBox;

#[derive(Component, Copy, Clone)]
struct Dice;

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
        Projection::Perspective(PerspectiveProjection {
            fov: PI / 2.0,
            ..default()
        }),
        RigidBody::Dynamic,
        GravityScale(0.0),
        Mass(3.0),
        LinearDamping(0.8),
        LockedAxes::ROTATION_LOCKED,
        AngularInertia::new(Vec3::new(2.0, 2.0, 2.0)),
        NoAutoAngularInertia,
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
    assets: ResMut<AssetServer>,
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
            Dice,
            RigidBody::Dynamic,
            SceneRoot(assets.load("dice.glb#Scene0")),
            //XXX use Collider::round_cuboid
            ColliderConstructorHierarchy::new(ColliderConstructor::ConvexDecompositionFromMesh),
            Transform::from_translation(
                camera_global_transform
                    .transform_point(worldbox_transform.transform_point(Vec3::ZERO)),
            )
            .with_scale(Vec3::splat(0.1)),
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

fn log_worldbox(query: Single<&Transform, With<Camera3d>>) {
    // info!("{:?}", query.translation);
}

fn apply_forces(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<Forces, With<Camera3d>>,
) {
    for mut forces in &mut query {
        let up = keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
        let down = keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
        let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
        let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

        let horizontal = right as i8 - left as i8;
        let vertical = down as i8 - up as i8;
        let direction =
            Vector::new(horizontal as Scalar, vertical as Scalar, 0.0).normalize_or_zero();
        forces.apply_force(direction * 50.);
        // forces.apply_linear_impulse(direction * time.delta_secs());
    }
}
