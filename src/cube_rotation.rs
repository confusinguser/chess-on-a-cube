use crate::utils::Vec3i;
use crate::MainCamera;
use bevy::prelude::*;
use std::f32::consts::PI;
use std::time::{self, Duration};

#[derive(Default, Debug)]
pub(crate) struct RotationData {
    target_rotation: Vec3i,
    current_rotation: Vec3i,
    time_started_rotation_y: time::Duration,
    time_started_rotation_x: time::Duration,
}

pub(crate) fn rotate(
    mut query: Query<(&mut Transform, &MainCamera)>,
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
        let mut transform = camera.0;
        transform.translation = camera.1.start_coords;
        transform.translate_around(Vec3::new(0., 0., 0.), rot);

        let up: Vec3;
        let angle = rotation_needed.x * PI / 2.;
        // When spinning around the y-axis we are also spinning the location of the x-axis. We
        // always want the "x-axis" to be the left face of the cube seen from the transform
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
        transform.translate_around(Vec3::new(0., 0., 0.), rot);
        transform.look_at(Vec3::new(0., 0., 0.), up);
        camera.0 = transform;
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
