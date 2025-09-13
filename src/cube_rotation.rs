use std::collections::VecDeque;

use bevy::prelude::*;
use derivative::Derivative;

use crate::utils::{CartesianDirection, SeeDirection};
use crate::MainCamera;

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
    animation_started: f64,
    /// What side seen from the camera that from and target are referring to
    side_changing: SeeDirection,
}

impl RotationAnimationData {
    /// Returns the partial camera translation that has currently happened in the animation in this axis
    fn partial_camera_translation(&self, current_time: f64, rotation_duration: f32) -> Quat {
        let animation_progress = ((current_time - self.animation_started) as f32) / rotation_duration;
        let rotation_amount = rotation_curve(animation_progress.clamp(0.0, 1.0));
        let full_rotation = Quat::from_rotation_arc(self.from.as_vec3(), self.target.as_vec3());
        let mut axis_angle = full_rotation.to_axis_angle();
        axis_angle.1 *= rotation_amount;
        Quat::from_axis_angle(axis_angle.0, axis_angle.1)
    }

    /// Assuming this animation is one that changes the top, what is the intermediate camera up vector?
    fn camera_up_vector(&self, current_time: f64, rotation_duration: f32) -> Vec3 {
        let animation_progress = ((current_time - self.animation_started) as f32) / rotation_duration;
        let rotation_amount = rotation_curve(animation_progress.clamp(0.0, 1.0));
        self.from.as_vec3() * (1. - rotation_amount) + self.target.as_vec3() * (rotation_amount)
    }
}

pub(crate) fn iterate(
    mut query: Query<(&mut Transform, &MainCamera)>,
    input: Res<ButtonInput<KeyCode>>,
    mut rotation_data: Local<RotationData>,
    time: Res<Time>,
) {
    let rotation_data = &mut *rotation_data;
    let rotation_duration = 1.0; // Duration in seconds as f32
    let current_time = time.elapsed_seconds_f64();

    conclude_finished_animations(rotation_data, current_time, rotation_duration);

    input_handling(input, rotation_data, current_time);

    // Apply the rotation
    for mut camera in &mut query {
        let mut transform = camera.0;
        transform.translation = rotation_data.rotation_state.camera_location() * 2.;

        transform.translate_around(
            Vec3::ZERO,
            total_animation_rotation(&rotation_data.animations, current_time, rotation_duration),
        );

        transform.look_at(
            Vec3::new(0., 0., 0.),
            camera_up_vector(rotation_data, current_time, rotation_duration),
        );

        camera.0 = transform;
    }
}

fn conclude_finished_animations(rotation_data: &mut RotationData, current_time: f64, rotation_duration: f32) {
    let mut num_finished_animations = 0;
    for animation in &rotation_data.animations {
        if (current_time - animation.animation_started) > rotation_duration as f64 {
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

fn input_handling(input: Res<ButtonInput<KeyCode>>, rotation_data: &mut RotationData, current_time: f64) {
    let fs = rotation_data.future_rotation_state; // Shorthand
    if input.just_pressed(KeyCode::ArrowRight) {
        start_rotation(rotation_data, fs.top.opposite(), SeeDirection::Top, current_time);
    } else if input.just_pressed(KeyCode::ArrowLeft) {
        start_rotation(rotation_data, fs.top, SeeDirection::Top, current_time);
    }
    if input.just_pressed(KeyCode::ArrowUp) {
        start_rotation(rotation_data, fs.side.opposite(), SeeDirection::Left, current_time);
    } else if input.just_pressed(KeyCode::ArrowDown) {
        start_rotation(rotation_data, fs.side, SeeDirection::Left, current_time);
    }
}

/// @param see_direction: The side (seen from the camera) that this rotation is rotating around
fn start_rotation(
    rotation_data: &mut RotationData,
    rotation: CartesianDirection,
    _see_direction: SeeDirection,
    current_time: f64,
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
            animation_started: current_time,
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
            animation_started: current_time,
            side_changing: SeeDirection::Top,
        });
        rotation_data.future_rotation_state.top = target;
    }
}

fn total_animation_rotation(
    animations: &VecDeque<RotationAnimationData>,
    current_time: f64,
    rotation_duration: f32,
) -> Quat {
    let mut output = Quat::IDENTITY;
    for animation in animations.iter().rev() {
        output *= animation.partial_camera_translation(current_time, rotation_duration);
    }
    output
}

fn camera_up_vector(rotation_data: &RotationData, current_time: f64, rotation_duration: f32) -> Vec3 {
    let mut output = rotation_data.rotation_state.top.as_vec3();
    for animation in &rotation_data.animations {
        if (animation.side_changing == SeeDirection::Top
            || animation.side_changing == SeeDirection::Bottom)
            && animation.target != animation.from
        {
            output += animation.camera_up_vector(current_time, rotation_duration) - animation.from.as_vec3();
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
        let current_time = 0.0f64;
        let rotation_duration = 1.0f32;
        let result = total_animation_rotation(&animations, current_time, rotation_duration);
        assert_eq!(result, Quat::IDENTITY);
    }
}
