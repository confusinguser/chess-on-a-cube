use bevy::prelude::*;

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

#[derive(Debug, Clone, Copy)]
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
            Self::ClockwiseX => Vec3::new(1., 0., 0.),
            Self::CounterX => Vec3::new(-1., 0., 0.),
            Self::ClockwiseY => Vec3::new(0., 1., 0.),
            Self::CounterY => Vec3::new(0., -1., 0.),
            Self::ClockwiseZ => Vec3::new(0., 0., 1.),
            Self::CounterZ => Vec3::new(0., 0., -1.),
        }
    }

    /// When on a side that has normal vector on the same axis as one of the elements, start
    /// walking toward negative coordinates to continue walking in the same radial direction
    fn negate_movement_on(&self) -> [CartesianDirection; 2] {
        match self {
            Self::ClockwiseX | Self::CounterX => [CartesianDirection::NegY, CartesianDirection::X],
            Self::ClockwiseY | Self::CounterY => [CartesianDirection::X, CartesianDirection::Z],
            Self::ClockwiseZ | Self::CounterZ => [CartesianDirection::NegX, CartesianDirection::Z],
        }
    }
    fn is_counterclockwise(&self) -> bool {
        match self {
            Self::ClockwiseX | Self::ClockwiseY | Self::ClockwiseZ => false,
            Self::CounterX | Self::CounterY | Self::CounterZ => true,
        }
    }

    pub(crate) fn rotation_axis(&self) -> CartesianDirection {
        match self {
            Self::ClockwiseX => CartesianDirection::X,
            Self::CounterX => CartesianDirection::NegX,
            Self::ClockwiseY => CartesianDirection::Y,
            Self::CounterY => CartesianDirection::NegY,
            Self::ClockwiseZ => CartesianDirection::Z,
            Self::CounterZ => CartesianDirection::NegZ,
        }
    }
    pub(crate) fn directions() -> [RadialDirection; 6] {
        [
            Self::ClockwiseX,
            Self::CounterX,
            Self::ClockwiseY,
            Self::CounterY,
            Self::ClockwiseZ,
            Self::CounterZ,
        ]
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
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
            Self::X => Vec3::new(1., 0., 0.),
            Self::NegX => Vec3::new(-1., 0., 0.),
            Self::Y => Vec3::new(0., 1., 0.),
            Self::NegY => Vec3::new(0., -1., 0.),
            Self::Z => Vec3::new(0., 0., 1.),
            Self::NegZ => Vec3::new(0., 0., -1.),
        }
    }

    fn is_negative(&self) -> bool {
        match self {
            Self::X | Self::Y | Self::Z => false,
            Self::NegX | Self::NegY | Self::NegZ => true,
        }
    }

    pub(crate) fn abs(&self) -> CartesianDirection {
        match self {
            Self::X | Self::NegX => Self::X,
            Self::Y | Self::NegY => Self::Y,
            Self::Z | Self::NegZ => Self::Z,
        }
    }

    pub(crate) fn directions() -> [CartesianDirection; 6] {
        [
            Self::X,
            Self::NegX,
            Self::Y,
            Self::NegY,
            Self::Z,
            Self::NegZ,
        ]
    }

    pub(crate) fn axis_num(&self) -> u32 {
        match self {
            Self::X | Self::NegX => 0,
            Self::Y | Self::NegY => 1,
            Self::Z | Self::NegZ => 2,
        }
    }
}

pub(crate) fn radial_direction_to_cartesian_direction(
    radial_direction: RadialDirection,
    normal: CartesianDirection,
) -> Option<CartesianDirection> {
    if normal.abs() == radial_direction.rotation_axis().abs() {
        warn!("utils::radial_direction_to_cartesian_direction called with radial_direction on same axis as normal");
        return None;
    }

    let mut negate = false;
    let negate_movement_on_axes = radial_direction.negate_movement_on();
    for axis in negate_movement_on_axes {
        if normal == axis {
            negate = true;
            break;
        }
    }

    if radial_direction.is_counterclockwise() {
        negate = !negate
    }

    return Some(
        *CartesianDirection::directions()
            .iter()
            .find(|dir| {
                dir.abs() != normal.abs()
                    && dir.abs() != radial_direction.rotation_axis().abs()
                    && dir.is_negative() ^ !negate
            })
            .unwrap(),
    );
}
