use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy::utils::petgraph::Direction;
use bevy_mod_picking::prelude::*;

use crate::scene::{self, MainCube};
use crate::Vec3i;

#[derive(Resource)]
pub(crate) struct Game {
    pub(crate) board: Board,
    pub(crate) selected_cell: Option<CellCoordinates>,
    pub(crate) phase: GamePhase,
    pub(crate) stored_units: Vec<Unit>,
}
impl Game {
    pub(crate) fn new(cube_side_length: u32) -> Self {
        Game {
            board: Board::new(cube_side_length),
            selected_cell: None,
            phase: GamePhase::PlaceUnits,
            stored_units: Default::default(),
        }
    }
}

#[derive(PartialEq)]
pub(crate) enum GamePhase {
    PlaceUnits,
    Play,
}

pub(crate) struct Board {
    board: BTreeMap<CellCoordinates, Cell>,
    pub(crate) cube_side_length: u32,
}

impl Board {
    fn get_cell(&self, coords: CellCoordinates) -> Option<&Cell> {
        self.board.get(&coords)
    }
    pub(crate) fn get_cell_mut(&mut self, coords: CellCoordinates) -> Option<&mut Cell> {
        self.board.get_mut(&coords)
    }
    fn new(cube_side_length: u32) -> Self {
        Board {
            board: BTreeMap::new(),
            cube_side_length,
        }
    }

    pub(crate) fn new_cell(&mut self, coords: CellCoordinates) {
        self.board.insert(coords, Cell::default());
    }

    pub(crate) fn get_all_cells(&self) -> Vec<&Cell> {
        self.board.values().collect()
    }
}

#[derive(Default, Clone)]
pub(crate) struct Cell {
    pub(crate) cell_type: CellType,
    pub(crate) occupant: Option<Unit>,
    pub(crate) plane: Option<Entity>,
    pub(crate) selected_unit_can_go: bool,
    pub(crate) coords: CellCoordinates,
}

impl Cell {
    pub(crate) fn set_plane(&mut self, plane: Entity) {
        self.plane = Some(plane);
    }

    pub(crate) fn set_occupant(&mut self, occupant: Unit) {
        self.occupant = Some(occupant);
    }
}

#[derive(Default, Clone)]
pub(crate) enum CellType {
    #[default]
    Empty,
    Black,
}

#[derive(Clone)]
pub(crate) struct Unit {
    unit_type: UnitType,
    pub(crate) cell: CellCoordinates,
}

impl Unit {
    fn new(unit_type: UnitType, cell: CellCoordinates) -> Self {
        Unit { unit_type, cell }
    }

    fn where_can_go(&self, cube_side_length: u32) -> Vec<CellCoordinates> {
        self.cell.get_adjacent(cube_side_length).into()
    }
}

#[derive(Clone)]
enum UnitType {
    Normal,
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

    fn get_adjacent(&self, cube_side_length: u32) -> [CellCoordinates; 4] {
        let directions = [
            Vec3::X,
            Vec3::NEG_X,
            Vec3::Y,
            Vec3::NEG_Y,
            Vec3::Z,
            Vec3::NEG_Z,
        ];

        let mut output: [CellCoordinates; 4] = Default::default();
        let normal = self.normal_direction();
        for (i, &direction) in directions.iter().enumerate() {
            if normal == direction || normal == direction * -1. {
                continue; // We ignore directions which would go out of and into the cube
            }

            let mut adjacent = *self;
            let mut x = adjacent.x as i32 + direction.x as i32;
            let mut y = adjacent.y as i32 + direction.y as i32;
            let mut z = adjacent.z as i32 + direction.z as i32;
            for c in [x, y, z].iter_mut() {
                if *c < 0 {
                    adjacent.normal_is_positive = false;
                    *c = 0;
                } else if *c >= cube_side_length as i32 {
                    adjacent.normal_is_positive = true;
                    *c = cube_side_length as i32;
                }
                let set_to = if self.normal_is_positive {
                    (cube_side_length - 1) as i32
                } else {
                    0
                };
                // Set the right coordinate along the old normal vector
                if normal.x != 0. {
                    x = set_to;
                };
                if normal.y != 0. {
                    y = set_to;
                };
                if normal.z != 0. {
                    z = set_to;
                };
            }
            adjacent.x = x as u32;
            adjacent.y = y as u32;
            adjacent.z = z as u32;
            output[i] = adjacent;
        }
        output
    }

    fn normal_direction(&self) -> Vec3 {
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

    fn manhattan_distance(c1: Self, c2: Self) -> f32 {
        todo!();
    }
}

pub(crate) fn on_cell_clicked(
    In(click): In<ListenedEvent<Click>>,
    query: Query<(&mut Handle<StandardMaterial>, &MainCube)>,
    materials: ResMut<Assets<StandardMaterial>>,
    game: ResMut<Game>,
) -> Bubble {
    let cell_clicked = query.get(click.target).unwrap();
    match game.phase {
        GamePhase::Play => on_cell_clicked_play_phase(cell_clicked, &query, materials, game),
        GamePhase::PlaceUnits => {
            on_cell_clicked_place_units_phase(cell_clicked, &query, materials, game)
        }
    }
    Bubble::Up
}

fn on_cell_clicked_place_units_phase(
    cell_clicked: (&Handle<StandardMaterial>, &MainCube),
    query: &Query<'_, '_, (&mut Handle<StandardMaterial>, &MainCube)>,
    materials: ResMut<'_, Assets<StandardMaterial>>,
    mut game: ResMut<'_, Game>,
) {
    let game = &mut *game; // Convert game to normal rust reference for partial borrow
    let cell = game.board.get_cell_mut(cell_clicked.1.coords).unwrap();
    if cell.occupant.is_none() {
        if let Some(unit) = game.stored_units.pop() {
            cell.set_occupant(unit);
        }
    }
    if game.stored_units.is_empty() {
        game.phase = GamePhase::Play;
    }
}

fn on_cell_clicked_play_phase(
    cell_clicked: (&Handle<StandardMaterial>, &MainCube),
    query: &Query<(&mut Handle<StandardMaterial>, &MainCube)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut game: ResMut<Game>,
) {
    let new_selected_material = materials.get_mut(cell_clicked.0).unwrap();
    crate::scene::select_cell_material(new_selected_material);

    if let Some(selected_cell) = game.selected_cell {
        if let Some(plane) = game.board.get_cell_mut(selected_cell).unwrap().plane {
            let old_selected = query.get(plane).unwrap();
            let old_selected_material = materials.get_mut(old_selected.0).unwrap();
            scene::normal_cell_material(old_selected_material);
        }
    }

    game.selected_cell = Some(cell_clicked.1.coords);
    let cube_side_length = game.board.cube_side_length;
    if let Some(selected_cell) = game.selected_cell {
        if let Some(selected_cell) = game.board.get_cell_mut(selected_cell) {
            if let Some(occupant) = &selected_cell.occupant {
                let cells_can_go = occupant.where_can_go(cube_side_length);
                for cell_can_go in cells_can_go {
                    mark_cell_can_go(cell_can_go, query, &mut materials, &mut game)
                }
            }
        }
    }
}

fn mark_cell_can_go(
    cell_coords: CellCoordinates,
    query: &Query<(&mut Handle<StandardMaterial>, &MainCube)>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    game: &mut ResMut<Game>,
) {
    let cell = game.board.get_cell_mut(cell_coords).unwrap();
    cell.selected_unit_can_go = true;
}
