use std::collections::{BTreeMap, VecDeque};

use bevy::prelude::*;

use crate::utils;

#[derive(Clone, Debug)]
pub(crate) struct Cell {
    pub(crate) cell_type: CellType,
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
            cell_type: default(),
            color: cell_color,
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub(crate) enum CellColor {
    White,
    Black,
    Gray,
}

#[derive(Default, Clone, Debug)]
pub(crate) enum CellType {
    #[default]
    Empty,
    Mountain,
}

impl CellType {
    fn walkable(&self) -> bool {
        match self {
            Self::Empty => true,
            Self::Mountain => false,
        }
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
        for direction in utils::CartesianDirection::directions() {
            let adjacent = self.get_cell_in_direction(direction, cube_side_length);

            if adjacent.is_none() {
                continue;
            }

            if i >= 4 {
                warn!("More than 4 directions in get_adjacent => No zero-field in CellCoordinate");
                break;
            }

            output[i] = adjacent.unwrap();
            i += 1;
        }
        output
    }

    fn get_cell_in_direction(
        &self,
        direction: utils::CartesianDirection,
        cube_side_length: u32,
    ) -> Option<CellCoordinates> {
        let normal = self.normal_direction();
        let direction = direction.as_vec3();
        if normal == direction || normal == direction * -1. {
            return None; // We ignore directions which would go out of and into the cube
        }

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
            if normal.x != 0. {
                adjacent.x = old_normal_axis_new_val;
            };
            if normal.y != 0. {
                adjacent.y = old_normal_axis_new_val;
            };
            if normal.z != 0. {
                adjacent.z = old_normal_axis_new_val;
            };
        }

        if direction.x != 0. {
            adjacent.x = relevant_coordinate as u32;
        } else if direction.y != 0. {
            adjacent.y = relevant_coordinate as u32;
        } else if direction.z != 0. {
            adjacent.z = relevant_coordinate as u32;
        }

        Some(adjacent)
    }

    fn get_cell_in_radial_direction(
        &self,
        direction: utils::RadialDirection,
    ) -> Option<CellCoordinates> {
        let direction_vec = direction.as_vec3();
        if utils::vectors_shared_component_sign(self.normal_direction(), direction_vec) != 0 {
            // The direction is not possible to go in on this side
            return None;
        }
        todo!();
    }

    fn get_diagonal_adjacent(&self) {}

    pub(crate) fn normal_direction(&self) -> Vec3 {
        let sign = if self.normal_is_positive { 1. } else { -1. };

        if self.z == 0 {
            Vec3::new(0., 0., sign)
        } else if self.y == 0 {
            Vec3::new(0., sign, 0.)
        } else if self.x == 0 {
            Vec3::new(sign, 0., 0.)
        } else {
            Vec3::ZERO
        }
    }

    /// Returns a list of tuples where the first element is the coordinate and the second is the
    /// distance to the cell
    pub(crate) fn get_cells_max_dist(
        self,
        dist: u32,
        only_walkable: bool,
        board: &Board,
    ) -> Vec<CellCoordinates> {
        let mut output = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back((self, 0));
        while !queue.is_empty() {
            let entry = queue.pop_front().unwrap();
            if entry.1 > dist {
                break;
            }
            output.push(entry.0);

            for adjacent in entry.0.get_adjacent(board.cube_side_length) {
                if only_walkable && !board.get_cell(self).unwrap().cell_type.walkable() {
                    continue;
                }
                if !output.iter().any(|cell| *cell == entry.0) {
                    continue;
                }
                queue.push_back((adjacent, entry.1 + 1));
            }
        }
        output
    }
}

#[derive(Debug)]
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
