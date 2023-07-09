use std::{f32::consts::PI, time::Duration};
mod gamemanager;
mod scene;

use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_mod_picking::prelude::*;

use self::gamemanager::Game;
use self::scene::construct_cube;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(
            DefaultPickingPlugins
                .build()
                .disable::<DefaultHighlightingPlugin>(),
        )
        .insert_resource(LookupPlanes {
            planes: Default::default(),
        })
        .insert_resource(Game::new(3))
        .add_startup_system(setup)
        .add_system(rotate)
        .add_system(scene::update_cell_colors)
        .run();
}

#[derive(Component)]
struct MainCamera;

#[derive(Resource)]
struct LookupPlanes {
    planes: [Vec<Option<Entity>>; 6],
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut lookup_planes: ResMut<LookupPlanes>,
    mut game: ResMut<Game>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    let material = StandardMaterial {
        base_color: Color::ANTIQUE_WHITE,
        ..default()
    };

    construct_cube(
        game.board.cube_side_length,
        &mut meshes,
        &mut commands,
        &mut materials,
        &material,
        &mut game,
    );

    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Torus::default().into()),
        material: debug_material.clone(),
        transform: Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(0.03)),
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(2., 2., 2.).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
            ..default()
        },
        RaycastPickCamera::default(), // Enable picking with this camera
        MainCamera,
    ));
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Vec3i {
    x: i32,
    y: i32,
    z: i32,
}

impl From<Vec3i> for Vec3 {
    fn from(val: Vec3i) -> Self {
        Vec3::new(val.x as f32, val.y as f32, val.z as f32)
    }
}

#[derive(Default, Debug)]
struct RotationData {
    target_rotation: Vec3i,
    current_rotation: Vec3i,
    time_started_rotation_y: Duration,
    time_started_rotation_x: Duration,
}

fn rotate(
    mut query: Query<&mut Transform, With<MainCamera>>,
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut rotation_data: Local<RotationData>,
) {
    let rotation_duration = 1.0;
    // Start at zero if we get to 4, since that is equal to a full turn
    if rotation_data.current_rotation.x >= 4 {
        // RHS rounds down to nearest integer divisible by 4
        rotation_data.target_rotation.x -= rotation_data.current_rotation.x / 4 * 4;
        rotation_data.current_rotation.x %= 4;
    }
    if rotation_data.current_rotation.y >= 4 {
        // RHS rounds down to nearest integer divisible by 4
        rotation_data.target_rotation.y -= rotation_data.current_rotation.y / 4 * 4;
        rotation_data.current_rotation.y %= 4;
    }

    if input.just_pressed(KeyCode::Left) && rotation_data.time_started_rotation_y.is_zero() {
        rotation_data.time_started_rotation_y = time.elapsed();
        rotation_data.target_rotation.y = rotation_data.current_rotation.y - 1
    }
    if input.just_pressed(KeyCode::Right) && rotation_data.time_started_rotation_y.is_zero() {
        rotation_data.time_started_rotation_y = time.elapsed();
        rotation_data.target_rotation.y = rotation_data.current_rotation.y + 1
    }
    if input.just_pressed(KeyCode::Up) && rotation_data.time_started_rotation_x.is_zero() {
        rotation_data.time_started_rotation_x = time.elapsed();
        rotation_data.target_rotation.x = rotation_data.current_rotation.x - 1
    }
    if input.just_pressed(KeyCode::Down) && rotation_data.time_started_rotation_x.is_zero() {
        rotation_data.time_started_rotation_x = time.elapsed();
        rotation_data.target_rotation.x = rotation_data.current_rotation.x + 1
    }

    let mut rotation_needed: Vec3 = rotation_data.current_rotation.clone().into();
    if !rotation_data.time_started_rotation_y.is_zero() {
        let time_elapsed = time.elapsed() - rotation_data.time_started_rotation_y;
        rotation_needed.y += (rotation_data.target_rotation.y - rotation_data.current_rotation.y)
            .signum() as f32
            * rotation_curve(time_elapsed.as_secs_f32() / rotation_duration);
        if time_elapsed.as_secs_f32() > rotation_duration {
            rotation_data.time_started_rotation_y = Duration::default();
            rotation_data.current_rotation.y = rotation_data.target_rotation.y;
        }
    }

    if !rotation_data.time_started_rotation_x.is_zero() {
        let time_elapsed = time.elapsed() - rotation_data.time_started_rotation_x;
        rotation_needed.x += (rotation_data.target_rotation.x - rotation_data.current_rotation.x)
            .signum() as f32
            * rotation_curve(time_elapsed.as_secs_f32() / rotation_duration);
        if time_elapsed.as_secs_f32() > rotation_duration {
            rotation_data.time_started_rotation_x = Duration::default();
            rotation_data.current_rotation.x = rotation_data.target_rotation.x;
        }
    }

    for mut camera in &mut query {
        let mut rot = Quat::from_euler(EulerRot::XYZ, 0., rotation_needed.y * PI / 2., 0.);
        camera.translation = Vec3::new(2., 2., 2.);
        camera.translate_around(Vec3::new(0., 0., 0.), rot);

        let up: Vec3;
        let angle = rotation_needed.x * PI / 2.;
        // When spinning around the y-axis we are also spinning the location of the x-axis. We
        // always want the "x-axis" to be the left face of the cube seen from the camera
        let mut rotation_parity = rotation_data.current_rotation.y % 4;
        if rotation_parity.is_negative() {
            rotation_parity += 4;
        }
        match rotation_parity {
            0 => {
                rot = Quat::from_euler(EulerRot::XYZ, 0., 0., angle);
                up = Vec3::new((angle + PI / 2.).cos(), (angle + PI / 2.).sin(), 0.);
            }
            1 => {
                rot = Quat::from_euler(EulerRot::XYZ, angle, 0., 0.);
                up = Vec3::new(0., (-angle + PI / 2.).sin(), (-angle + PI / 2.).cos());
            }
            2 => {
                rot = Quat::from_euler(EulerRot::XYZ, 0., 0., -angle);
                up = Vec3::new((-angle + PI / 2.).cos(), (-angle + PI / 2.).sin(), 0.);
            }
            3 => {
                rot = Quat::from_euler(EulerRot::XYZ, -angle, 0., 0.);
                up = Vec3::new(0., (angle + PI / 2.).sin(), (angle + PI / 2.).cos());
            }
            _ => {
                unreachable!()
            }
        }
        camera.translate_around(Vec3::new(0., 0., 0.), rot);
        camera.look_at(Vec3::new(0., 0., 0.), up);
    }
}

fn rotation_curve(time: f32) -> f32 {
    if time >= 1. {
        return 1.;
    }
    if time <= 0. {
        return 0.;
    }

    let c1 = 1.70158;
    let c3 = c1 + 1.;

    1. + c3 * (time - 1.).powi(3) + c1 * (time - 1.).powi(2)
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    )
}
