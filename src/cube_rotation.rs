use std::collections::VecDeque;
use std::time::{Duration, Instant};

use bevy::prelude::*;
use derivative::Derivative;

use crate::MainCamera;
use crate::utils::{CartesianDirection, SeeDirection};

#[derive(Debug, Default, Clone)]
pub(crate) struct RotationData {
    rotation_state: RotationState,
    /// The rotation state that is currently targeted by the ongoing animation(s). An animation around the side face only sets the top to `Some`, and vice versa. This is to allow simultaneous animations of the side and the top, without interference once one of them reaches the goal.
    future_rotation_state: RotationState,
    /// List of ongoing animations, oldest animation is first
    animations: VecDeque<RotationAnimationData>,
}

#[derive(Debug, Derivative, Clone, Copy, PartialEq, Eq)]
#[derivative(Default)]
pub(crate) struct RotationState {
    #[derivative(Default(value = "CartesianDirection::Y"))]
    top: CartesianDirection,
    #[derivative(Default(value = "CartesianDirection::Z"))]
    side: CartesianDirection,
}

impl RotationState {
    /// Gives the camera location given the rotation state
    fn camera_location(&self) -> Vec3 {
        let mut output = Vec3::ZERO;
        let top_vec = self.top.as_vec3();
        output += top_vec;
        let side_vec = self.side.as_vec3();
        output += side_vec;
        output += top_vec.cross(side_vec);
        output
    }

    /// Gives the rotation state after a rotation has been made. Rotation argument is defined as the axis in the direction where looking in line with it gives a clockwise rotation.
    fn after_rotation(&self, rotation: CartesianDirection) -> RotationState {
        let mut rotation_state = *self;
        if let Some(new_top) = rotation_state.top.cross(rotation) {
            rotation_state.top = new_top;
        }
        if let Some(new_side) = rotation_state.side.cross(rotation) {
            rotation_state.side = new_side;
        }
        rotation_state
    }

    /// Set the rotation state field from the see direction
    fn set_see_direction(&mut self, see_direction: SeeDirection, value: CartesianDirection) {
        match see_direction {
            SeeDirection::Top => self.top = value,
            SeeDirection::Left => self.side = value,
            _ => {}
        }
    }

    #[allow(unused)]
    /// Panics in some cases if the rotation state is not possible. That is, if the side and top are parallel
    fn get_see_direction(&self, see_direction: SeeDirection) -> CartesianDirection {
        match see_direction {
            SeeDirection::Top => self.top,
            SeeDirection::Left => self.side,
            SeeDirection::Right => self
                .top
                .cross(self.side)
                .expect("Rotation state is not possible"),
            SeeDirection::BackLeft => self
                .top
                .cross(self.side)
                .expect("Rotation state is not possible")
                .opposite(),
            SeeDirection::Bottom => self.top.opposite(),
            SeeDirection::BackRight => self.side.opposite(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RotationAnimationData {
    from: CartesianDirection,
    target: CartesianDirection,
    animation_started: Instant,
    /// What side seen from the camera that from and target are referring to
    side_changing: SeeDirection,
}

impl RotationAnimationData {
    /// Returns the partial camera translation that has currently happened in the animation in this axis
    fn partial_camera_translation(&self, rotation_time: Duration) -> Quat {
        let animation_progress =
            (Instant::now() - self.animation_started).as_secs_f64() / rotation_time.as_secs_f64();
        let rotation_amount = rotation_curve(animation_progress as f32);
        let full_rotation = Quat::from_rotation_arc(self.from.as_vec3(), self.target.as_vec3());
        let mut axis_angle = full_rotation.to_axis_angle();
        axis_angle.1 *= rotation_amount;
        Quat::from_axis_angle(axis_angle.0, axis_angle.1)
    }

    /// Assuming this animation is one that changes the top, what is the intermediate camera up vector?
    fn camera_up_vector(&self, rotation_time: Duration) -> Vec3 {
        let animation_progress =
            (Instant::now() - self.animation_started).as_secs_f64() / rotation_time.as_secs_f64();
        let rotation_amount = rotation_curve(animation_progress as f32);
        self.from.as_vec3() * (1. - rotation_amount) + self.target.as_vec3() * (rotation_amount)
    }
}

pub(crate) fn iterate(
    mut query: Query<(&mut Transform, &MainCamera)>,
    input: Res<Input<KeyCode>>,
    mut rotation_data: Local<RotationData>,
) {
    let rotation_data = &mut *rotation_data;
    let rotation_duration = Duration::from_secs(1);

    conclude_finished_animations(rotation_data, rotation_duration);

    input_handling(input, rotation_data);

    dbg!(&rotation_data);

    // Apply the rotation
    for mut camera in &mut query {
        let mut transform = camera.0;
        transform.translation = rotation_data.rotation_state.camera_location() * 2.;

        transform.translate_around(
            Vec3::ZERO,
            total_animation_rotation(&rotation_data.animations, rotation_duration),
        );

        transform.look_at(
            Vec3::new(0., 0., 0.),
            camera_up_vector(rotation_data, rotation_duration),
        );

        camera.0 = transform;
    }
}

fn conclude_finished_animations(rotation_data: &mut RotationData, rotation_duration: Duration) {
    let mut num_finished_animations = 0;
    for animation in &rotation_data.animations {
        if (Instant::now() - animation.animation_started) > rotation_duration {
            rotation_data
                .rotation_state
                .set_see_direction(animation.side_changing, animation.target);
            num_finished_animations += 1;
        }
    }

    for _ in 0..num_finished_animations {
        rotation_data.animations.pop_front();
    }
}

fn input_handling(input: Res<Input<KeyCode>>, rotation_data: &mut RotationData) {
    let fs = rotation_data.future_rotation_state; // Shorthand
    if input.just_pressed(KeyCode::Right) {
        start_rotation(rotation_data, fs.top.opposite(), SeeDirection::Top);
    } else if input.just_pressed(KeyCode::Left) {
        start_rotation(rotation_data, fs.top, SeeDirection::Top);
    }
    if input.just_pressed(KeyCode::Up) {
        start_rotation(rotation_data, fs.side.opposite(), SeeDirection::Left);
    } else if input.just_pressed(KeyCode::Down) {
        start_rotation(rotation_data, fs.side, SeeDirection::Left);
    }
}

/// @param see_direction: The side (seen from the camera) that this rotation is rotating around
fn start_rotation(
    rotation_data: &mut RotationData,
    rotation: CartesianDirection,
    _see_direction: SeeDirection,
) {
    // If the rotation axis is not parallel to the top axis, then this rotation will modify the side axis
    if !rotation.is_parallel_to(rotation_data.future_rotation_state.side) {
        let target = rotation_data
            .future_rotation_state
            .after_rotation(rotation)
            .side;
        rotation_data.animations.push_back(RotationAnimationData {
            from: rotation_data.future_rotation_state.side,
            target,
            animation_started: Instant::now(),
            side_changing: SeeDirection::Left,
        });
        rotation_data.future_rotation_state.side = target;
    }

    if !rotation.is_parallel_to(rotation_data.future_rotation_state.top) {
        let target = rotation_data
            .future_rotation_state
            .after_rotation(rotation)
            .top;
        rotation_data.animations.push_back(RotationAnimationData {
            from: rotation_data.future_rotation_state.top,
            target,
            animation_started: Instant::now(),
            side_changing: SeeDirection::Top,
        });
        rotation_data.future_rotation_state.top = target;
    }
}

fn total_animation_rotation(
    animations: &VecDeque<RotationAnimationData>,
    rotation_time: Duration,
) -> Quat {
    let mut output = Quat::IDENTITY;
    for animation in animations.iter().rev() {
        output *= animation.partial_camera_translation(rotation_time);
    }
    output
}

fn camera_up_vector(rotation_data: &RotationData, rotation_time: Duration) -> Vec3 {
    let mut output = rotation_data.rotation_state.top.as_vec3();
    for animation in &rotation_data.animations {
        if (animation.side_changing == SeeDirection::Top
            || animation.side_changing == SeeDirection::Bottom)
            && animation.target != animation.from
        {
            output += animation.camera_up_vector(rotation_time) - animation.from.as_vec3();
        }
    }
    output
}

fn rotation_curve(time: f32) -> f32 {
    // if time >= 1. {
    //     return 1.;
    // }
    // if time <= 0. {
    //     return 0.;
    // }
    // time

    let c1 = 1.70158;
    let c3 = c1 + 1.;

    1. + c3 * (time - 1.).powi(3) + c1 * (time - 1.).powi(2)
}

#[allow(unused_imports)]
mod tests {
    use std::time::Duration;

    use bevy::math::Quat;

    use super::*;

    #[test]
    fn rotation_state_after_rotation_test() {
        use super::RotationState;
        use crate::utils::CartesianDirection::*;
        let rotation_state = RotationState { top: Y, side: Z };
        assert_eq!(
            rotation_state.after_rotation(Z),
            RotationState { top: X, side: Z }
        );
    }

    #[test]
    fn total_animation_rotation_with_no_animations() {
        let animations = VecDeque::<RotationAnimationData>::new();
        let rotation_time = Duration::from_secs(1);
        let result = total_animation_rotation(&animations, rotation_time);
        assert_eq!(result, Quat::IDENTITY);
    }
}
