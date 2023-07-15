use crate::utils;
use crate::MainCamera;
use bevy::prelude::*;
use std::f32::consts::PI;
use std::time::Duration;

#[derive(Default, Debug)]
pub(crate) struct RotationData {
    current_rotation: Quat,
    time_started_rotations: [Duration; 3],
    reversed_axes: [bool; 3],
}

pub(crate) fn rotate(
    mut query: Query<(&mut Transform, &MainCamera)>,
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut rotation_data: Local<RotationData>,
) {
    let time = &*time;
    let mut rotation_data = &mut *rotation_data;
    let rotation_duration = 1.0;
    macro_rules! input_handling {
        ($keycode:expr, $axis:expr) => {
            if input.just_pressed($keycode) {
                // When spinning around the y-axis we are also spinning the location of the x-axis. We
                // always want the "x-axis" to be the left face of the cube seen from the camera
                let mut axis =
                    new_axis_on_side_after_rotation($axis, rotation_data.current_rotation) * PI / 2.;
                let mut axis_num = utils::first_nonzero_component(axis).unwrap() as usize;
                if axis_num == 2 {
                    axis = Vec3::new(axis[axis_num],0.,0.);
                    axis_num = 0;
                }
                dbg!(axis, axis_num);
                if rotation_data.time_started_rotations[axis_num].is_zero() {
                    rotation_data.reversed_axes[axis_num] = axis[axis_num] < 0.;
                    rotation_data.time_started_rotations[axis_num] = time.elapsed();
                }
            }
        }
    }

    // Input
    input_handling!(KeyCode::Left, -Vec3::Y);
    input_handling!(KeyCode::Right, Vec3::Y);
    input_handling!(KeyCode::Down, Vec3::Z);
    input_handling!(KeyCode::Up, -Vec3::Z);

    let mut rotation_needed = rotation_data.current_rotation;

    // Animate world axes
    // x-axis
    animate_axis(
        time,
        &mut rotation_data.time_started_rotations[0],
        &mut rotation_data.current_rotation,
        rotation_duration,
        EulerRot::XYZ,
        &mut rotation_needed,
        rotation_data.reversed_axes[0],
    );
    // y-axis
    animate_axis(
        time,
        &mut rotation_data.time_started_rotations[1],
        &mut rotation_data.current_rotation,
        rotation_duration,
        EulerRot::YXZ,
        &mut rotation_needed,
        rotation_data.reversed_axes[1],
    );
    // z-axis
    animate_axis(
        time,
        &mut rotation_data.time_started_rotations[2],
        &mut rotation_data.current_rotation,
        rotation_duration,
        EulerRot::ZXY,
        &mut rotation_needed,
        rotation_data.reversed_axes[2],
    );

    // Apply the rotation
    for mut camera in &mut query {
        let mut transform = camera.0;
        transform.translation = camera.1.start_coords;
        transform.translate_around(Vec3::new(0., 0., 0.), rotation_needed);

        /*        let up;
        let mut rotation_parity = rotation_data.current_rotation.y % 4;
        if rotation_parity.is_negative() {
            rotation_parity += 4;
        }

        let angle = (rotation_needed.x + rotation_needed.z) * PI / 2.;
        match rotation_parity {
            0 => {
                up = Vec3::new((angle + PI / 2.).cos(), (angle + PI / 2.).sin(), 0.);
            }
            1 => {
                up = Vec3::new(0., (-angle + PI / 2.).sin(), (-angle + PI / 2.).cos());
            }
            2 => {
                up = Vec3::new((-angle + PI / 2.).cos(), (-angle + PI / 2.).sin(), 0.);
            }
            3 => {
                up = Vec3::new(0., (-angle + PI / 2.).sin(), (-angle + PI / 2.).cos());
            }
            _ => {
                unreachable!()
            }
        }*/

        transform.look_at(Vec3::new(0., 0., 0.), Vec3::new(0., 1., 0.));
        //        transform.look_at(Vec3::new(0., 0., 0.), up);
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

fn direction_after_rotation(direction: Vec3, rot: Quat) -> Vec3 {
    rot.mul_vec3(direction)
}

fn new_axis_on_side_after_rotation(normal_of_side: Vec3, rot: Quat) -> Vec3 {
    let directions = [Vec3::X, Vec3::Y, Vec3::Z];

    for direction in directions {
        let rotated_dir = direction_after_rotation(direction, rot);
        let shared_component_sign =
            utils::vectors_shared_component_sign(rotated_dir, normal_of_side);
        if shared_component_sign != 0 {
            return direction * shared_component_sign as f32;
        }
    }
    Vec3::ZERO
}

/// # Arguments
/// axis: The middle axis in the EulerRot should correspond to the axis
fn animate_axis(
    time: &Time,
    time_started_rotation: &mut Duration,
    current_rotation: &mut Quat,
    rotation_duration: f32,
    axis: EulerRot,
    rotation_needed: &mut Quat,
    reversed: bool,
) {
    if time_started_rotation.is_zero() {
        return; // No rotation happening on axis
    }
    let time_elapsed = time.elapsed() - time_started_rotation.to_owned();
    let rotation_amount = if reversed { -1. } else { 1. }
        * rotation_curve(time_elapsed.as_secs_f32() / rotation_duration)
        * PI
        / 2.;

    *rotation_needed *= Quat::from_euler(axis, rotation_amount, 0., 0.);

    if time_elapsed.as_secs_f32() > rotation_duration {
        *time_started_rotation = Duration::default();
        *current_rotation *=
            Quat::from_euler(axis, if reversed { -1. } else { 1. } * PI / 2., 0., 0.);
        dbg!(current_rotation.to_euler(EulerRot::XYZ),);
        dbg!(current_rotation.to_euler(EulerRot::YXZ),);
        dbg!(current_rotation.to_euler(EulerRot::ZXY),);
    }
}
