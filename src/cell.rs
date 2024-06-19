use std::collections::BTreeMap;
use std::ops::{Index, IndexMut};

use bevy::prelude::*;

use crate::gamemanager::Palette;
use crate::utils::{self, CartesianDirection, RadialDiagonal, RadialDirection};

#[derive(Clone, Debug)]
pub(crate) struct Cell {
    pub(crate) plane: Entity,
    pub(crate) selected_unit_can_move_to: bool,
    pub(crate) coords: CellCoordinates,
    pub(crate) color: CellColor,
}

impl Cell {
    pub(crate) fn new(plane: Entity, coords: CellCoordinates, cell_color: CellColor) -> Self {
        Self {
            plane,
            coords,
            selected_unit_can_move_to: false,
            color: cell_color,
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub(crate) enum CellColor {
    Bright,
    Mid,
    Dark,
}

impl CellColor {
    pub(crate) fn base_color(&self, palette: Palette) -> Color {
        palette.get_colors()[match self {
            Self::Bright => 0,
            Self::Mid => 1,
            Self::Dark => 2,
        }]
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub(crate) struct CellCoordinates {
    x: u32,
    y: u32,
    z: u32,
    normal_is_positive: bool,
}

impl CellCoordinates {
    pub(crate) fn new(x: u32, y: u32, z: u32, normal_is_positive: bool) -> Self {
        CellCoordinates {
            x,
            y,
            z,
            normal_is_positive,
        }
    }

    pub(crate) fn get_adjacent(&self, cube_side_length: u32) -> [CellCoordinates; 4] {
        let mut output: [CellCoordinates; 4] = Default::default();
        let mut i = 0;
        for direction in CartesianDirection::directions() {
            let adjacent = self.get_cell_in_direction(direction, cube_side_length);

            if adjacent.is_none() {
                continue;
            }

            if i >= 4 {
                warn!("More than 4 directions in get_adjacent => No zero-field in CellCoordinate");
                break;
            }

            output[i] = adjacent.unwrap().0;
            i += 1;
        }
        output
    }

    /// Returns a tuple where the second element denotes if the new cell is on a different side
    /// than the first
    pub(crate) fn get_cell_in_direction(
        &self,
        direction: CartesianDirection,
        cube_side_length: u32,
    ) -> Option<(CellCoordinates, bool)> {
        let normal = self.normal_direction();
        if normal.is_parallel_to(direction) {
            return None; // We ignore directions which would go out of and into the cube
        }

        let direction = direction.as_vec3();

        let mut adjacent = *self;
        let mut relevant_coordinate;
        if direction.x != 0. {
            relevant_coordinate = adjacent.x as i32 + direction.x as i32;
        } else if direction.y != 0. {
            relevant_coordinate = adjacent.y as i32 + direction.y as i32;
        } else if direction.z != 0. {
            relevant_coordinate = adjacent.z as i32 + direction.z as i32;
        } else {
            unreachable!();
        };
        let mut folded_to_other_face = false;
        // We start counting coordinates at 1 since 0 represents on the plane
        if relevant_coordinate <= 0 {
            adjacent.normal_is_positive = false;
            relevant_coordinate = 0;
            folded_to_other_face = true;
        } else if relevant_coordinate > cube_side_length as i32 {
            adjacent.normal_is_positive = true;
            relevant_coordinate = 0;
            folded_to_other_face = true;
        }

        if folded_to_other_face {
            let old_normal_axis_new_val = if self.normal_is_positive {
                cube_side_length
            } else {
                1
            };

            // Set the correct coordinate along the old normal vector
            adjacent[normal.axis_num() as usize] = old_normal_axis_new_val
        }

        if direction.x != 0. {
            adjacent.x = relevant_coordinate as u32;
        } else if direction.y != 0. {
            adjacent.y = relevant_coordinate as u32;
        } else if direction.z != 0. {
            adjacent.z = relevant_coordinate as u32;
        }

        Some((adjacent, folded_to_other_face))
    }

    /// Returns a tuple where the second element denotes if the new cell is on a different side
    /// than the first
    pub(crate) fn get_cell_in_radial_direction(
        &self,
        radial_direction: RadialDirection,
        cube_side_length: u32,
    ) -> Option<(CellCoordinates, bool)> {
        if radial_direction
            .rotation_axis()
            .is_parallel_to(self.normal_direction())
        {
            // The direction is not possible to go in on this side
            return None;
        }

        let cartesian_direction = radial_direction.to_cartesian_direction(self.normal_direction());

        self.get_cell_in_direction(cartesian_direction.unwrap(), cube_side_length)
    }

    /// Gets the diagonal that can be reached by walking in the cartesian directions consecutively,
    /// does not return true neighbors. The second element of the returned tuple denotes if the move crosses an edge
    pub(crate) fn get_diagonal(
        &self,
        diagonal: (CartesianDirection, CartesianDirection),
        cube_side_length: u32,
    ) -> Option<(CellCoordinates, bool)> {
        let cell1 = self.get_cell_in_direction(diagonal.0, cube_side_length)?;
        let cell2 = cell1
            .0
            .get_cell_in_direction(diagonal.1, cube_side_length)?;
        if cell1.1 && cell2.1 {
            // The second element tells us if the transformation went over a cube edge, in this
            // case we are in a corner, which means we have a true neighbor in cell2
            return None;
        }

        Some((cell2.0, cell1.1 || cell2.1))
    }

    /// Gets the diagonal that can be reached by walking in the radial directions consecutively,
    /// does not return true neighbors. The second element of the returned tuple denotes if the move crosses an edge
    pub(crate) fn get_diagonal_radial(
        &self,
        diagonal: RadialDiagonal,
        cube_side_length: u32,
    ) -> Option<(CellCoordinates, bool)> {
        let mut directions = Vec::new();
        if self.x != 0 {
            directions.push(CartesianDirection::from_axis_num(0, diagonal.0));
        }
        if self.y != 0 {
            directions.push(CartesianDirection::from_axis_num(1, diagonal.1));
        }
        if self.z != 0 {
            directions.push(CartesianDirection::from_axis_num(2, diagonal.2));
        }

        let Some(dir1) = directions.get(0) else {
            error!("Directions vector is not large enough in get_diagonal_radial. This should not happen.");
            return None;
        };
        let Some(dir2) = directions.get(1) else {
            error!("Directions vector is not large enough in get_diagonal_radial. This should not happen.");
            return None;
        };
        let directions = (*dir1, *dir2);
        
        self.get_diagonal(directions, cube_side_length)
    }
    pub(crate) fn normal_direction(&self) -> CartesianDirection {
        if self.z == 0 {
            if self.normal_is_positive {
                CartesianDirection::Z
            } else {
                CartesianDirection::NegZ
            }
        } else if self.y == 0 {
            if self.normal_is_positive {
                CartesianDirection::Y
            } else {
                CartesianDirection::NegY
            }
        } else if self.x == 0 {
            if self.normal_is_positive {
                CartesianDirection::X
            } else {
                CartesianDirection::NegX
            }
        } else {
            panic!("No zero field on CellCoordinates: {:?}", self);
        }
    }

    pub(crate) fn opposite(&self, cube_side_length: u32) -> CellCoordinates {
        let mut out = *self;
        out.normal_is_positive = !out.normal_is_positive;
        if out.x != 0 {
            out.x = cube_side_length + 1 - out.x;
        }
        if out.y != 0 {
            out.y = cube_side_length + 1 - out.y;
        }
        if out.z != 0 {
            out.z = cube_side_length + 1 - out.z;
        }
        out
    }

    #[allow(unused)]
    pub(crate) fn display(&self) -> String {
        let mut output = match self.normal_direction().abs() {
            CartesianDirection::X => "x",
            CartesianDirection::Y => "y",
            CartesianDirection::Z => "z",
            _ => unreachable!(),
        }
        .to_string();
        if self.normal_is_positive {
            output = output.to_uppercase();
        }

        let mut second_axis = false;
        const LETTERS: [char; 4] = ['a', 'b', 'c', 'd'];
        for i in 0..3 {
            if self[i] == 0 {
                continue;
            }
            if second_axis {
                output.push_str(&self[i].to_string());
            } else {
                output.push(LETTERS[self[i] as usize - 1]);
            }
            second_axis = true;
        }
        output
    }
}

impl Index<usize> for CellCoordinates {
    type Output = u32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("index out of bounds"),
        }
    }
}

impl IndexMut<usize> for CellCoordinates {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("index out of bounds"),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Board {
    board: BTreeMap<CellCoordinates, Cell>,
    pub(crate) cube_side_length: u32,
}

impl Board {
    pub(crate) fn get_cell(&self, coords: CellCoordinates) -> Option<&Cell> {
        self.board.get(&coords)
    }
    pub(crate) fn get_cell_mut(&mut self, coords: CellCoordinates) -> Option<&mut Cell> {
        self.board.get_mut(&coords)
    }
    pub(crate) fn new(cube_side_length: u32) -> Self {
        Board {
            board: BTreeMap::new(),
            cube_side_length,
        }
    }

    pub(crate) fn new_cell(&mut self, coords: CellCoordinates, cell: Cell) {
        self.board.insert(coords, cell);
    }

    pub(crate) fn get_all_cells(&self) -> Vec<&Cell> {
        self.board.values().collect()
    }

    #[must_use]
    pub(crate) fn get_all_cells_mut(&mut self) -> Vec<&mut Cell> {
        self.board.values_mut().collect()
    }
}
