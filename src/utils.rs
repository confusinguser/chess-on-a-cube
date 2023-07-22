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

pub(crate) fn nonzero_components(v: Vec3) -> Vec<u32> {
    let mut output = Vec::new();
    for i in 0..3 {
        if v[i] != 0. {
            output.push(i as u32);
        }
    }
    output
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum RadialDirection {
    ClockwiseX,
    CounterX,
    ClockwiseY,
    CounterY,
    ClockwiseZ,
    CounterZ,
}

impl RadialDirection {
    /// When on a side that has normal vector on the same axis as one of the elements, start
    /// walking toward negative coordinates to continue walking in the same radial direction
    fn negate_movement_on(&self) -> [CartesianDirection; 2] {
        match self {
            Self::ClockwiseX | Self::CounterX => [CartesianDirection::Y, CartesianDirection::NegZ],
            Self::ClockwiseY | Self::CounterY => [CartesianDirection::NegX, CartesianDirection::Z],
            Self::ClockwiseZ | Self::CounterZ => [CartesianDirection::X, CartesianDirection::NegY],
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

    pub(crate) fn to_cartesian_direction(
        self,
        normal: CartesianDirection,
    ) -> Option<CartesianDirection> {
        if normal.abs() == self.rotation_axis().abs() {
            warn!("utils::radial_direction_to_cartesian_direction called with radial_direction on same axis as normal");
            return None;
        }

        let mut negate = false;
        let negate_movement_on_axes = self.negate_movement_on();
        for axis in negate_movement_on_axes {
            if normal == axis {
                negate = true;
                break;
            }
        }

        if self.is_counterclockwise() {
            negate = !negate
        }

        let out = CartesianDirection::directions()
            .iter()
            .find(|dir| {
                dir.abs() != normal.abs()
                    && dir.abs() != self.rotation_axis().abs()
                    && dir.is_negative() ^ !negate
            })
            .copied();

        out
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
    pub(crate) fn from_axis_num(axis_num: u32, is_positive: bool) -> Self {
        let mut output = match axis_num {
            0 => Self::X,
            1 => Self::Y,
            2 => Self::Z,
            _ => unreachable!(),
        };

        if !is_positive {
            output = output.opposite();
        }
        output
    }

    /// `vec` is almost a cartesian direction
    pub(crate) fn from_vec3_round(mut vec: Vec3) -> Option<Self> {
        for i in 0..3 {
            vec[i] = vec[i].round()
        }

        if nonzero_components(vec).len() != 1 {
            return None;
        }

        let Some(axis_num) = first_nonzero_component(vec) else {
            return None;
        };

        Some(Self::from_axis_num(
            axis_num,
            vec[axis_num as usize].signum() > 0.,
        ))
    }

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

    pub(crate) fn is_negative(&self) -> bool {
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

    pub(crate) fn axis_num(&self) -> u32 {
        match self {
            Self::X | Self::NegX => 0,
            Self::Y | Self::NegY => 1,
            Self::Z | Self::NegZ => 2,
        }
    }

    #[must_use]
    pub(crate) fn opposite(&self) -> CartesianDirection {
        match self {
            Self::X => Self::NegX,
            Self::NegX => Self::X,
            Self::Y => Self::NegY,
            Self::NegY => Self::Y,
            Self::Z => Self::NegZ,
            Self::NegZ => Self::Z,
        }
    }

    /// Returns the positive direction whose axis that is perpendicular to the two others. Returns
    /// None if the two directions are on the same axis
    pub(crate) fn get_perpendicular_axis(
        &self,
        other: CartesianDirection,
    ) -> Option<CartesianDirection> {
        if self.abs() == other.abs() {
            // Both are on same axis, so there are two perpendiculars
            return None;
        }
        for axis_num in 0..3 {
            if self.axis_num() != axis_num && other.axis_num() != axis_num {
                return Some(CartesianDirection::from_axis_num(axis_num, true));
            }
        }
        None
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

    pub(crate) fn diagonals() -> [(Self, Self); 12] {
        let mut out = [(Self::X, Self::X); 12];
        let mut i = 0;
        for dir in Self::directions() {
            for dir2 in Self::directions() {
                if dir.abs() == dir2.abs()
                    || out
                        .iter()
                        .any(|&diagonal| diagonal == (dir, dir2) || diagonal == (dir2, dir))
                {
                    continue;
                } else {
                    out[i] = (dir, dir2);
                    i += 1;
                }
            }
        }
        out
    }
}
