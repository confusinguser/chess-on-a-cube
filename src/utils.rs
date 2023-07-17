use bevy::prelude::Vec3;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Vec3i {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) z: i32,
}

impl From<Vec3i> for Vec3 {
    fn from(val: Vec3i) -> Self {
        Vec3::new(val.x as f32, val.y as f32, val.z as f32)
    }
}

/// Returns 0 if the vectors do not share a non-zero component,
/// 1 if the vectors share a non-zero component with the same sign,
/// -1 if the vectors do not share a non-zero component with the same sign
/// but share a non-zero component with differing signs
pub(crate) fn vectors_shared_component_sign(v1: Vec3, v2: Vec3) -> i32 {
    let pairs = [(v1.x, v2.x), (v1.y, v2.y), (v1.z, v2.z)];
    for pair in pairs {
        if pair.0 * pair.1 > 0. {
            return 1;
        }
    }
    for pair in pairs {
        if pair.0 * pair.1 < 0. {
            return -1;
        }
    }
    0
}

/// Returns first non-zero component of vector in the order XYZ where x is 0. Returns None if all
/// components are zero
pub(crate) fn first_nonzero_component(v: Vec3) -> Option<u32> {
    if v.x != 0. {
        return Some(0);
    }
    if v.y != 0. {
        return Some(1);
    }
    if v.z != 0. {
        return Some(2);
    }
    None
}

pub(crate) enum RadialDirection {
    ClockwiseX,
    CounterX,
    ClockwiseY,
    CounterY,
    ClockwiseZ,
    CounterZ,
}

impl RadialDirection {
    pub(crate) fn as_vec3(&self) -> Vec3 {
        match self {
            RadialDirection::ClockwiseX => Vec3::new(1., 0., 0.),
            RadialDirection::CounterX => Vec3::new(-1., 0., 0.),
            RadialDirection::ClockwiseY => Vec3::new(0., 1., 0.),
            RadialDirection::CounterY => Vec3::new(0., -1., 0.),
            RadialDirection::ClockwiseZ => Vec3::new(0., 0., 1.),
            RadialDirection::CounterZ => Vec3::new(0., 0., -1.),
        }
    }
}

pub(crate) enum CartesianDirection {
    X,
    NegX,
    Y,
    NegY,
    Z,
    NegZ,
}

impl CartesianDirection {
    pub(crate) fn as_vec3(&self) -> Vec3 {
        match self {
            CartesianDirection::X => Vec3::new(1., 0., 0.),
            CartesianDirection::NegX => Vec3::new(-1., 0., 0.),
            CartesianDirection::Y => Vec3::new(0., 1., 0.),
            CartesianDirection::NegY => Vec3::new(0., -1., 0.),
            CartesianDirection::Z => Vec3::new(0., 0., 1.),
            CartesianDirection::NegZ => Vec3::new(0., 0., -1.),
        }
    }

    pub(crate) fn directions() -> [CartesianDirection; 6] {
        [
            CartesianDirection::X,
            CartesianDirection::NegX,
            CartesianDirection::Y,
            CartesianDirection::NegY,
            CartesianDirection::Z,
            CartesianDirection::NegZ,
        ]
    }
}

pub(crate) fn radial_direction_to_cartesian_direction(
    radial_direction: RadialDirection,
    normal: CartesianDirection,
) -> CartesianDirection {
    todo!();
}
