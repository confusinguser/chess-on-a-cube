//! This example demonstrates the built-in 3d shapes in Bevy.
//! The scene includes a patterned texture and a rotation for visualizing the normals and UVs.

use std::{f32::consts::PI, time::Duration};

use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat}, asset::HandleId,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_startup_system(setup)
        .add_system(rotate)
        .run();
}

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
struct MainCube;

#[derive(Component)]
struct MainCamera;


fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

        let cube_mesh: Handle<Mesh> = meshes.add(shape::Cube::default().into());


    let parent = commands.spawn(
            PbrBundle {
                mesh: meshes.add(shape::Torus::default().into()),
                material: debug_material.clone(),
                transform: Transform::from_xyz(0., 0., 0.),
                visibility: Visibility::Hidden,
                ..default()
            }).id();

    commands.spawn((
        PbrBundle {
            mesh: cube_mesh,
            material: debug_material.clone(),
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        },
        MainCube,
    ));

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


    let mut child = commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(2., 2., 2.).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ..default()
    },
    MainCamera));
    child.set_parent(
        parent
    );
}


#[derive(Default,Debug)]
struct RotationData {
    target_rotation: Vec3,
    current_rotation: Vec3,
    time_started_rotation_y: Duration,
    time_started_rotation_x: Duration,
}

fn rotate(mut query: Query<&mut Transform, With<MainCamera>>, time: Res<Time>, input: Res<Input<KeyCode>>, mut rotation_data: Local<RotationData>) {
    let rotation_duration = 1.;
// Start at zero if we get to 4, since that is equal to a full turn
    if rotation_data.current_rotation.x >= 4. {
        rotation_data.target_rotation.x -= 4.*(rotation_data.current_rotation.x/4.).floor();
        rotation_data.current_rotation.x %= 4.;
    }
    if rotation_data.current_rotation.y >= 4. {
        rotation_data.target_rotation.y -= 4.*(rotation_data.current_rotation.y/4.).floor();
        rotation_data.current_rotation.y %= 4.;
    }

    if input.just_pressed(KeyCode::Left) && rotation_data.time_started_rotation_y.is_zero() {
        rotation_data.time_started_rotation_y = time.elapsed();
        rotation_data.target_rotation.y = rotation_data.current_rotation.y -1.
    }
    if input.just_pressed(KeyCode::Right) && rotation_data.time_started_rotation_y.is_zero() {
        rotation_data.time_started_rotation_y = time.elapsed();
        rotation_data.target_rotation.y = rotation_data.current_rotation.y +1.
    }
    if input.just_pressed(KeyCode::Up) && rotation_data.time_started_rotation_x.is_zero() {
        rotation_data.time_started_rotation_x = time.elapsed();
        rotation_data.target_rotation.x = rotation_data.current_rotation.x -1.
    }
    if input.just_pressed(KeyCode::Down) && rotation_data.time_started_rotation_x.is_zero() {
        rotation_data.time_started_rotation_x = time.elapsed();
        rotation_data.target_rotation.x = rotation_data.current_rotation.x +1.
    }

    let mut rotation_needed = rotation_data.current_rotation;
    if !rotation_data.time_started_rotation_y.is_zero() {
        let time_elapsed = time.elapsed()- rotation_data.time_started_rotation_y;
        rotation_needed.y += (rotation_data.target_rotation.y-rotation_data.current_rotation.y).signum()*
            rotation_curve(time_elapsed.as_secs_f32()/rotation_duration);
        if time_elapsed.as_secs_f32() > rotation_duration {
            rotation_data.time_started_rotation_y = Duration::default();
            rotation_data.current_rotation.y = rotation_data.target_rotation.y;
        }
    }

    if !rotation_data.time_started_rotation_x.is_zero() {
        let time_elapsed = time.elapsed()- rotation_data.time_started_rotation_x;
        rotation_needed.x += (rotation_data.target_rotation.x-rotation_data.current_rotation.x).signum()*
            rotation_curve(time_elapsed.as_secs_f32()/rotation_duration);
        if time_elapsed.as_secs_f32() > rotation_duration {
            rotation_data.time_started_rotation_x = Duration::default();
            rotation_data.current_rotation.x = rotation_data.target_rotation.x;
        }
    }
    dbg!(&rotation_data);


    for mut camera in &mut query {
        let rot = Quat::from_euler(EulerRot::XYZ, 0.,rotation_needed.y*PI/2., 0. );
        camera.translation = Vec3::new(2., 2., 2.);
        camera.translate_around(Vec3::new(0., 0., 0.), rot);
        let rot = Quat::from_euler(EulerRot::XYZ, rotation_needed.x*PI/2., 0. ,0.);
        camera.translate_around(Vec3::new(0., 0., 0.), rot);
        camera.look_at(Vec3::new(0., 0., 0.), vector_from_heading_x(rotation_needed.x*PI/2.));
    }
}

fn vector_from_heading_x(angle_x: f32) -> Vec3 {
    let angle = -angle_x+PI/2.;
    Vec3::new(0., angle.sin(), angle.cos())
}

fn rotation_curve(time:f32) -> f32 {
    if time >= 1. {
        return 1.;
    }
    if time <= 0. {
        return 0.;
    }

    let c1 = 1.70158;
    let c3 = c1 + 1.;

    return 1. + c3 * (time - 1.).powi(3) + c1 * (time - 1.).powi(2);
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
