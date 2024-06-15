use std::time::{Duration, Instant};

use bevy::prelude::*;
use derivative::Derivative;

use crate::MainCamera;
use crate::utils::CartesianDirection;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct RotationData {
    rotation_state: RotationState,
    future_rotation_state: RotationState,
    /// The rotation state that is currently targeted by the ongoing animation(s). An animation around the side face only sets the top to `Some`, and vice versa. This is to allow simultaneous animations of the side and the top, without interference once one of them reaches the goal.
    top_rotation_animation: Option<RotationAnimationData>,
    side_rotation_animation: Option<RotationAnimationData>,
}

#[derive(Debug, Derivative, Clone, Copy)]
#[derivative(Default)]
pub(crate) struct RotationState {
    #[derivative(Default(value = "CartesianDirection::Y"))]
    top: CartesianDirection,
    #[derivative(Default(value = "CartesianDirection::X"))]
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
        if let Some(new_top) = rotation.cross(rotation_state.top) {
            rotation_state.top = new_top;
        }
        if let Some(new_side) = rotation.cross(rotation_state.side) {
            rotation_state.side = new_side;
        }
        rotation_state
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RotationAnimationData {
    from: CartesianDirection,
    target: CartesianDirection,
    animation_started: Instant,
}

impl RotationAnimationData {
    /// Returns the partial camera rotation that has currently happened in the animation in this axis
    fn animation_rotation_axis(&self, rotation_time: Duration) -> Quat {
        let animation_progress =
            (Instant::now() - self.animation_started).as_secs_f64() / rotation_time.as_secs_f64();
        let rotation_amount = rotation_curve(animation_progress as f32);
        let full_rotation = Quat::from_rotation_arc(self.from.as_vec3(), self.target.as_vec3());
        let mut axis_angle = full_rotation.to_axis_angle();
        axis_angle.1*=rotation_amount;
        Quat::from_axis_angle(axis_angle.0, axis_angle.1)
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

    start_animations(input, rotation_data);

    dbg!(&rotation_data);
    
    // Apply the rotation
    for mut camera in &mut query {
        let mut transform = camera.0;
        transform.translation = rotation_data.rotation_state.camera_location() * 2.;
        // The camera rotates in the opposite direction from how the cube would have rotated to get
        // to the same place
        transform.translate_around(
            Vec3::ZERO,
            total_animation_rotation(
                &[
                    rotation_data.top_rotation_animation,
                    rotation_data.side_rotation_animation,
                ],
                rotation_duration,
            ),
        );

        transform.look_at(
            Vec3::new(0., 0., 0.),
            rotation_data.rotation_state.top.as_vec3(),
        );

        camera.0 = transform;
    }
}

fn conclude_finished_animations(rotation_data: &mut RotationData, rotation_duration: Duration) {
    if let Some(animation) = &rotation_data.side_rotation_animation {
        if (Instant::now() - animation.animation_started) > rotation_duration {
            rotation_data.rotation_state.top = animation.target;
            rotation_data.side_rotation_animation = None;
        }
    }

    if let Some(animation) = &rotation_data.top_rotation_animation {
        if (Instant::now() - animation.animation_started) > rotation_duration {
            rotation_data.rotation_state.side = animation.target;
            rotation_data.top_rotation_animation = None;
        }
    }
}

fn start_animations(input: Res<Input<KeyCode>>, rotation_data: &mut RotationData) {
    if rotation_data.top_rotation_animation.is_none() {
        let rotation = if input.just_pressed(KeyCode::Right) {
            Some(rotation_data.future_rotation_state.top.opposite())
        } else if input.just_pressed(KeyCode::Left) {
            Some(rotation_data.future_rotation_state.top)
        } else {
            None
        };
        if let Some(rotation) = rotation {
            let target = rotation_data
                .future_rotation_state
                .after_rotation(rotation)
                .side;
            rotation_data.top_rotation_animation = Some(RotationAnimationData {
                from: rotation_data.future_rotation_state.side,
                target,
                animation_started: Instant::now(),
            });
            rotation_data.future_rotation_state.side = target;
        }
    }

    if rotation_data.side_rotation_animation.is_none() {
        let rotation = if input.just_pressed(KeyCode::Up) {
            Some(rotation_data.future_rotation_state.side.opposite())
        } else if input.just_pressed(KeyCode::Down) {
            Some(rotation_data.future_rotation_state.top)
        } else {
            None
        };
        if let Some(rotation) = rotation {
            let target = rotation_data
                .future_rotation_state
                .after_rotation(rotation)
                .top;
            rotation_data.side_rotation_animation = Some(RotationAnimationData {
                from: rotation_data.future_rotation_state.top,
                target,
                animation_started: Instant::now(),
            });
            rotation_data.future_rotation_state.top = target;
        }
    }
}

fn total_animation_rotation(
    animations: &[Option<RotationAnimationData>],
    rotation_time: Duration,
) -> Quat {
    let mut output = Quat::IDENTITY;
    // Iterate without Nones
    for animation in animations.iter().flatten() {
        output *= animation.animation_rotation_axis(rotation_time);
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
    // 
    let c1 = 1.70158;
    let c3 = c1 + 1.;

    1. + c3 * (time - 1.).powi(3) + c1 * (time - 1.).powi(2)
}

mod tests {
    #[test]
    fn camera_location_test() {
    }
}
