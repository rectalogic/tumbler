use std::f32::consts::PI;

use avian3d::prelude::*;
use bevy::{
    audio::{PlaybackMode, Volume},
    color::palettes::basic,
    mesh::VertexAttributeValues,
    prelude::*,
    window::{WindowResized, WindowResolution},
};

#[cfg(feature = "web")]
mod web;

pub fn start(width: u32, height: u32) {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(width, height),
                resizable: false,
                ..default()
            }),
            ..default()
        }),
        PhysicsPlugins::default(),
        // PhysicsDebugPlugin,
        #[cfg(feature = "web")]
        web::plugin,
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, handle_collisions)
    .add_systems(Last, on_resize);

    app.run();
}

const WORLDBOX_DEPTH: f32 = 2.0;

#[derive(Component, Copy, Clone)]
struct WorldBox;

#[derive(Component, Copy, Clone)]
struct Dice;

#[derive(Resource, Deref)]
struct Dice2DiceCollisionSound(Handle<AudioSource>);

#[derive(Resource, Deref)]
struct Dice2WallCollisionSound(Handle<AudioSource>);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) -> Result<()> {
    commands.insert_resource(Dice2DiceCollisionSound(
        asset_server.load("sounds/dice-dice-collision.mp3"),
    ));
    commands.insert_resource(Dice2WallCollisionSound(
        asset_server.load("sounds/dice-wall-collision.mp3"),
    ));

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
        Mass(0.2), // 200 grams
        LinearDamping(0.8),
        LockedAxes::ROTATION_LOCKED,
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
            ]),
            children![(
                PointLight {
                    shadows_enabled: true,
                    ..default()
                },
                Transform::from_xyz(0., WORLDBOX_SIZE / 2.2, WORLDBOX_SIZE),
            )],
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

        let dice_bundle = (
            Dice,
            RigidBody::Dynamic,
            Mass(0.0012), // 1.2 grams
            SceneRoot(assets.load("dice.glb#Scene0")),
            CollisionEventsEnabled,
            Collider::round_cuboid(1.6, 1.6, 1.6, PI / 128.),
            Transform::from_translation(
                camera_global_transform
                    .transform_point(worldbox_transform.transform_point(Vec3::ZERO)),
            )
            .with_scale(Vec3::splat(0.3)),
        );
        commands.spawn_batch([dice_bundle.clone(), dice_bundle]);
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

fn handle_collisions(
    mut commands: Commands,
    mut collisions_started: MessageReader<CollisionStart>,
    collisions: Collisions,
    dice: Query<Entity, With<Dice>>,
    dice2dice_sound: Res<Dice2DiceCollisionSound>,
    dice2wall_sound: Res<Dice2WallCollisionSound>,
) {
    for event in collisions_started.read() {
        if let Some(contact_pair) = collisions.get(event.collider1, event.collider2) {
            let magnitude = contact_pair.total_normal_impulse_magnitude() * 1000.;
            info!(magnitude); //XXX
            let settings = PlaybackSettings {
                mode: PlaybackMode::Despawn,
                volume: Volume::Linear(magnitude),
                ..PlaybackSettings::ONCE
            };
            if dice.contains(event.collider1) && dice.contains(event.collider2) {
                // dice/dice collision
                commands.spawn((AudioPlayer(dice2dice_sound.clone()), settings));
            } else {
                // dice/worldbox collision
                commands.spawn((AudioPlayer(dice2wall_sound.clone()), settings));
            }
        }
    }
}
