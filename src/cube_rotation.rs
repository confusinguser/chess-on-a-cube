use crate::utils::{self, CartesianDirection};
use crate::MainCamera;
use bevy::prelude::*;
use std::f32::consts::PI;
use std::time::Duration;

#[derive(Debug)]
pub(crate) struct RotationData {
    current_rotation: Quat,
    current_camera_up: CartesianDirection,
    time_started_rotations: [Duration; 4],
    reversed_axes: [bool; 4],
    camera_rotated_times: i32,
}

impl Default for RotationData {
    fn default() -> Self {
        Self {
            current_rotation: Default::default(),
            current_camera_up: CartesianDirection::Y,
            time_started_rotations: Default::default(),
            reversed_axes: Default::default(),
            camera_rotated_times: Default::default(),
        }
    }
}

pub(crate) fn rotate(
    mut query: Query<(&mut Transform, &MainCamera)>,
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut rotation_data: Local<RotationData>,
) {
    let time = &*time;
    let rotation_data = &mut *rotation_data;
    let rotation_duration = 1.;
    macro_rules! input_handling {
        ($keycode:tt, $axis:tt, $camera_rotation: expr) => {
            if input.just_pressed(KeyCode::$keycode) {
                // When spinning around the y-axis we are also spinning the location of the x-axis. We
                // always want the "x-axis" to be the left face of the cube seen from the camera
                let axis =
                    direction_after_spatial_and_camera_rotation(CartesianDirection::$axis, rotation_data.current_rotation, rotation_data.camera_rotated_times)
                    .expect("Current rotation does not have anything other than quarter turns").as_vec3() * PI / 2.;
                let axis_num = utils::first_nonzero_component(axis).unwrap() as usize;
                if rotation_data.time_started_rotations[axis_num].is_zero() {
                    rotation_data.reversed_axes[axis_num] = axis[axis_num] < 0.;
                    rotation_data.time_started_rotations[axis_num] = time.elapsed();
                    if $camera_rotation != 0 {
                    rotation_data.time_started_rotations[3] = time.elapsed();
                        rotation_data.reversed_axes[3] = $camera_rotation == -1;
                        rotation_data.camera_rotated_times += $camera_rotation;
                    }
                }

            }
        }
    }

    // Input
    input_handling!(Left, NegY, 0);
    input_handling!(Right, Y, 0);
    input_handling!(Down, Z, 1);
    input_handling!(Up, NegZ, -1);

    let mut rotation_needed = rotation_data.current_rotation;
    dbg!(rotation_data.current_rotation.to_euler(EulerRot::XYZ));
    let mut camera_rotation_up_needed = rotation_data.current_camera_up.as_vec3();
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

    animate_camera_rotation(
        time,
        &mut rotation_data.time_started_rotations[3],
        &mut rotation_data.current_camera_up,
        rotation_duration,
        &mut camera_rotation_up_needed,
        rotation_data.reversed_axes[3],
        rotation_data.current_rotation,
    );

    // Apply the rotation
    for mut camera in &mut query {
        let mut transform = camera.0;
        transform.translation = camera.1.start_coords;
        transform.translate_around(Vec3::new(0., 0., 0.), rotation_needed);

        transform.look_at(Vec3::new(0., 0., 0.), camera_rotation_up_needed);

        camera.0 = transform;
    }
}

fn animate_camera_rotation(
    time: &Time,
    time_started_rotation: &mut Duration,
    current_camera_rotation: &mut CartesianDirection,
    rotation_duration: f32,
    rotation_needed: &mut Vec3,
    reversed: bool,
    rotation: Quat,
) {
    if time_started_rotation.is_zero() {
        return; // No rotation happening on axis
    }
    let time_elapsed = time.elapsed() - time_started_rotation.to_owned();
    let rotation_amount = if reversed { -1. } else { 1. }
        * rotation_curve(time_elapsed.as_secs_f32() / rotation_duration)
        * 2.
        * PI
        / 3.;

    let target =
        direction_after_spatial_and_camera_turn(*current_camera_rotation, rotation, reversed);
    let quat_path = Quat::from_rotation_arc(*rotation_needed, target.as_vec3());
    *rotation_needed =
        Quat::from_axis_angle(quat_path.to_axis_angle().0, rotation_amount).mul_vec3(Vec3::Y);
    if time_elapsed.as_secs_f32() > rotation_duration {
        *time_started_rotation = Duration::default();
        *current_camera_rotation =
            direction_after_spatial_and_camera_turn(*current_camera_rotation, rotation, reversed);
    }
}

/// Takes the normal and gives back a new normal where the cube is both rotated and the camera
/// turned a third of a turn either clockwise or counterclockwise
fn direction_after_spatial_and_camera_turn(
    normal: CartesianDirection,
    rot: Quat,
    counterclockwise: bool,
) -> CartesianDirection {
    use CartesianDirection::*;
    let mut only_camera = match normal.abs() {
        X => Y,
        Y => Z,
        Z => X,
        _ => unreachable!(),
    };
    if only_camera.is_negative() {
        only_camera = only_camera.opposite();
    }
    // Later, make a reset func to recover from errors, TODO
    let output = new_axis_on_side_after_rotation(only_camera, rot).unwrap();
    if counterclockwise {
        // Two clockwise => one counterclockwise
        return direction_after_spatial_and_camera_turn(output, rot, false);
    }
    output
}

/// TODO: Come up with better names dude
fn direction_after_spatial_and_camera_rotation(
    normal: CartesianDirection,
    rot: Quat,
    camera_rotated_times: i32,
) -> Option<CartesianDirection> {
    let axis_after_camera = match camera_rotated_times % 3 {
        0 => normal,
        1 | -2 => direction_after_spatial_and_camera_turn(normal, rot, false),
        2 | -1 => direction_after_spatial_and_camera_turn(normal, rot, true),
        _ => unreachable!(),
    };
    new_axis_on_side_after_rotation(axis_after_camera, rot)
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

fn direction_after_rotation(
    direction: CartesianDirection,
    rot: Quat,
) -> Option<CartesianDirection> {
    CartesianDirection::from_vec3_round(rot.mul_vec3(direction.as_vec3()))
}

fn new_axis_on_side_after_rotation(
    normal: CartesianDirection,
    rot: Quat,
) -> Option<CartesianDirection> {
    for direction in CartesianDirection::directions() {
        let Some(rotated_dir) = direction_after_rotation(direction, rot) else {
            return None;
        };
        if rotated_dir == normal {
            return Some(direction);
        }
    }
    unreachable!()
}

/// # Arguments
/// axis: The first axis in the EulerRot should correspond to the axis animated
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
    }
}

mod tests {
    #[test]
    fn new_axis_on_side_after_rotation_test() {
        for direction in crate::utils::CartesianDirection::directions() {
            for direction2 in crate::utils::CartesianDirection::directions() {
                let o = crate::cube_rotation::new_axis_on_side_after_rotation(
                    direction,
                    bevy::prelude::Quat::from_rotation_arc(
                        direction2.as_vec3(),
                        direction.as_vec3(),
                    ),
                )
                .unwrap();
                assert_eq!(o, direction2, "From: {:?}, to: {:?}", direction2, direction)
            }
        }
    }
}
