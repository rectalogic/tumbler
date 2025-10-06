use avian3d::prelude::*;
use bevy::{
    camera::CameraProjection,
    color::palettes::basic,
    mesh::VertexAttributeValues,
    prelude::*,
    window::{WindowMode, WindowResized, WindowResolution},
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
    .add_systems(Update, on_resize);

    app.run();
}

const BOX_DEPTH: f32 = 4.0;

#[derive(Component)]
struct WorldBox;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) -> Result<()> {
    commands.spawn((Camera3d::default(), Transform::from_xyz(0., 0., BOX_DEPTH)));
    let mut mesh = Mesh::from(Cuboid::new(0.5, 0.5, 0.5));
    invert_normals(&mut mesh)?;
    commands.spawn((
        WorldBox,
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(Color::from(basic::PURPLE))),
    ));
    Ok(())
}

fn on_resize(
    mut resize_reader: MessageReader<WindowResized>,
    projection: Single<&Projection, With<Camera>>,
    worldbox: Single<&mut Transform, With<WorldBox>>,
) {
    if resize_reader.read().last().is_some() {
        let [_, topright, ..] = projection.get_frustum_corners(BOX_DEPTH, BOX_DEPTH * 2.);
        let mut transform = worldbox.into_inner();
        transform.translation = Vec3::new(0., 0., topright.z - BOX_DEPTH / 2.0);
        transform.scale = Vec3::new(topright.x, topright.y, BOX_DEPTH);
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
