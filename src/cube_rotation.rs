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
}

impl Default for RotationData {
    fn default() -> Self {
        Self {
            current_rotation: Default::default(),
            current_camera_up: CartesianDirection::Y,
            time_started_rotations: Default::default(),
            reversed_axes: Default::default(),
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

    dbg!(
        &rotation_data,
        rotation_data.current_rotation.mul_vec3(Vec3::splat(1.))
    );

    let mut input_handling =
        |keycode: KeyCode, axis: CartesianDirection, camera_rotation: i32, reversed: bool| {
            if input.just_pressed(keycode) {
                let axis_rotated = direction_after_camera_turn(
                    axis.abs(),
                    rotation_data.current_rotation,
                    rotation_data.current_camera_up,
                    0,
                )
                .expect("Current rotation does not have anything other than quarter turns");
                let axis_num = axis_rotated.axis_num() as usize;
                if rotation_data.time_started_rotations[axis_num].is_zero()
                    && (rotation_data.time_started_rotations[3].is_zero() || camera_rotation == 0)
                {
                    rotation_data.reversed_axes[axis_num] = reversed;
                    rotation_data.time_started_rotations[axis_num] = time.elapsed();
                    if camera_rotation != 0 {
                        rotation_data.time_started_rotations[3] = time.elapsed();
                        rotation_data.reversed_axes[3] = camera_rotation == -1;
                    }
                }
            };
        };

    // Input
    input_handling(KeyCode::Left, CartesianDirection::Y, 0, true);
    input_handling(KeyCode::Right, CartesianDirection::Y, 0, false);
    input_handling(KeyCode::Down, CartesianDirection::Z, 1, false);
    input_handling(KeyCode::Up, CartesianDirection::Z, -1, true);
    if input.just_pressed(KeyCode::Space) {
        rotation_data.time_started_rotations[3] = time.elapsed();
        rotation_data.reversed_axes[3] = input.pressed(KeyCode::A);
    }

    let mut rotation_needed = rotation_data.current_rotation;
    let mut camera_rotation_up_needed = rotation_data.current_camera_up.as_vec3();
    // Has to happen before world axes so that the rotation is the same on even the last one
    animate_camera_rotation(
        time,
        &mut rotation_data.time_started_rotations[3],
        &mut rotation_data.current_camera_up,
        rotation_duration,
        &mut camera_rotation_up_needed,
        rotation_data.reversed_axes[3],
        rotation_data.current_rotation,
    );

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

    dbg!(rotation_needed.mul_vec3(Vec3::splat(1.)));
    // Apply the rotation
    for mut camera in &mut query {
        let mut transform = camera.0;
        transform.translation = camera.1.start_coords;
        // The camera rotates in the opposite direction from how the cube would have rotated to get
        // to the same place
        transform.translate_around(Vec3::new(0., 0., 0.), rotation_needed);

        transform.look_at(Vec3::new(0., 0., 0.), camera_rotation_up_needed);

        camera.0 = transform;
    }
}

fn animate_camera_rotation(
    time: &Time,
    time_started_rotation: &mut Duration,
    current_camera_up: &mut CartesianDirection,
    rotation_duration: f32,
    rotation_needed: &mut Vec3,
    reversed: bool,
    rotation: Quat,
) {
    if time_started_rotation.is_zero() {
        return; // No rotation happening on axis
    }
    let time_elapsed = time.elapsed() - time_started_rotation.to_owned();
    let cancelled_out_camera_up = CartesianDirection::from_vec3_round(
        rotation.inverse().mul_vec3(current_camera_up.as_vec3()),
    );

    // Two clockwise => counterclockwise
    let target = direction_after_camera_turn(
        cancelled_out_camera_up.unwrap(),
        rotation,
        *current_camera_up,
        if reversed { 2 } else { 1 },
    )
    .unwrap();
    dbg!(target);
    // let target = direction_after_rotation(target, rotation).unwrap();

    let quat_path = Quat::from_rotation_arc(current_camera_up.as_vec3(), target.as_vec3());
    let rotation_amount = rotation_curve(time_elapsed.as_secs_f32() / rotation_duration)
        * quat_path.to_axis_angle().1;

    *rotation_needed = Quat::from_axis_angle(quat_path.to_axis_angle().0, rotation_amount)
        .mul_vec3(current_camera_up.as_vec3());
    if time_elapsed.as_secs_f32() > rotation_duration {
        *time_started_rotation = Duration::default();
        *current_camera_up = target;
    }
}

/// Note: The normal inputted has to be UNROTATED
fn direction_after_camera_turn(
    mut normal: CartesianDirection,
    rot: Quat,
    current_camera_up: CartesianDirection,
    clockwise_turns: u32,
) -> Option<CartesianDirection> {
    use CartesianDirection::*;
    for _ in 0..clockwise_turns {
        normal = match normal.abs() {
            X => Y,
            Y => Z,
            Z => X,
            _ => unreachable!(),
        }
    }
    match normal {
        CartesianDirection::Y => Some(current_camera_up),
        CartesianDirection::Z => to_the_side_from_camera_perspective(
            rot.mul_vec3(Vec3::splat(1.)),
            current_camera_up,
            false,
        ),
        CartesianDirection::X => to_the_side_from_camera_perspective(
            rot.mul_vec3(Vec3::splat(1.)),
            current_camera_up,
            true,
        ),
        _ => {
            // TODO: Reset everything instead
            error!("The normal is not visible from current position");
            dbg!(current_camera_up, normal);
            None
        }
    }
}

/// From the camera perspective, gets the leftmost or rightmost side normal.
/// current_camera_up has to be visible from the camera loc
fn to_the_side_from_camera_perspective(
    camera_loc: Vec3,
    current_camera_up: CartesianDirection,
    to_the_right: bool,
) -> Option<CartesianDirection> {
    let sides = order_of_sides(current_camera_up);
    let mut side_indicies = Vec::with_capacity(2);
    for i in 0..3 {
        let c = camera_loc[i];
        let mut side = Vec3::ZERO;
        side[i] = c;
        let axis = CartesianDirection::from_vec3_round(side).unwrap();
        if current_camera_up.axis_num() == i as u32 {
            if current_camera_up != axis {
                error!("The side camera up is not visible");
                dbg!(camera_loc, axis);
                return None;
            }
            continue;
        }
        side_indicies.push((axis, sides.iter().position(|&side| side == axis).unwrap()));
    }
    assert_eq!(side_indicies.len(), 2);
    side_indicies.sort_by_key(|side_index| side_index.1);

    if side_indicies[0].1 == 0 && side_indicies[1].1 == 3 {
        return Some(side_indicies[if to_the_right { 0 } else { 1 }].0);
    }
    Some(side_indicies[if to_the_right { 1 } else { 0 }].0)
}

fn rotation_curve(time: f32) -> f32 {
    if time >= 1. {
        return 1.;
    }
    if time <= 0. {
        return 0.;
    }
    time

    // let c1 = 1.70158;
    // let c3 = c1 + 1.;

    // 1. + c3 * (time - 1.).powi(3) + c1 * (time - 1.).powi(2)
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

/// Outputs the order that the sides on the side come in when the inputted side is on the top. In
/// clockwise order when looking at it from the bottom up.
fn order_of_sides(up: CartesianDirection) -> [CartesianDirection; 4] {
    use CartesianDirection::*;
    let mut output = match up.abs() {
        X => [Y, Z, NegY, NegZ],
        Y => [Z, X, NegZ, NegX],
        Z => [X, Y, NegX, NegY],
        _ => unreachable!(),
    };

    if up.is_negative() {
        output.reverse();
    }
    output
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
