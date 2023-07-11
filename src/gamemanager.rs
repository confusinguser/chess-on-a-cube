use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::scene::{self, MainCube};

#[derive(Resource, Debug)]
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
            stored_units: vec![Unit::new(UnitType::Normal, CellCoordinates::default())],
        }
    }
}

#[derive(PartialEq, Debug)]
pub(crate) enum GamePhase {
    PlaceUnits,
    Play,
}
#[derive(Debug)]
pub(crate) struct Board {
    board: BTreeMap<CellCoordinates, Cell>,
    units: Vec<Unit>,
    pub(crate) cube_side_length: u32,
}

impl Board {
    pub(crate) fn get_cell(&self, coords: CellCoordinates) -> Option<&Cell> {
        self.board.get(&coords)
    }
    pub(crate) fn get_cell_mut(&mut self, coords: CellCoordinates) -> Option<&mut Cell> {
        self.board.get_mut(&coords)
    }
    fn new(cube_side_length: u32) -> Self {
        Board {
            board: BTreeMap::new(),
            units: Vec::new(),
            cube_side_length,
        }
    }
    pub(crate) fn get_unit_at(&self, coords: CellCoordinates) -> Option<&Unit> {
        self.units.iter().find(|unit| unit.coords == coords)
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

#[derive(Default, Clone, Debug)]
pub(crate) struct Cell {
    pub(crate) cell_type: CellType,
    pub(crate) plane: Option<Entity>,
    pub(crate) selected_unit_can_go: bool,
    pub(crate) coords: CellCoordinates,
}

impl Cell {
    pub(crate) fn set_plane(&mut self, plane: Entity) {
        self.plane = Some(plane);
    }
}

#[derive(Default, Clone, Debug)]
pub(crate) enum CellType {
    #[default]
    Empty,
    Black,
}

#[derive(Clone, Debug)]
pub(crate) struct Unit {
    unit_type: UnitType,
    pub(crate) coords: CellCoordinates,
    /// The physical object that represents this unit
    entity: Option<Entity>,
}

impl Unit {
    fn new(unit_type: UnitType, coords: CellCoordinates) -> Self {
        Unit {
            unit_type,
            coords,
            entity: None,
        }
    }

    fn where_can_go(&self, cube_side_length: u32) -> Vec<CellCoordinates> {
        self.coords.get_adjacent(cube_side_length).into()
    }

    fn set_entity(&mut self, entity: Entity) {
        self.entity = Some(entity);
    }
}

#[derive(Clone, Debug)]
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
        let mut i = 0;
        for direction in directions {
            if normal == direction || normal == direction * -1. {
                continue; // We ignore directions which would go out of and into the cube
            }
            if i >= 4 {
                warn!("More than 4 directions in get_adjacent => No zero-field in CellCoordinate");
                break;
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
                let set_old_normal_to = if self.normal_is_positive {
                    cube_side_length
                } else {
                    1
                };

                // Set the correct coordinate along the old normal vector
                if normal.x != 0. {
                    adjacent.x = set_old_normal_to;
                };
                if normal.y != 0. {
                    adjacent.y = set_old_normal_to;
                };
                if normal.z != 0. {
                    adjacent.z = set_old_normal_to;
                };
            }

            if direction.x != 0. {
                adjacent.x = relevant_coordinate as u32;
            } else if direction.y != 0. {
                adjacent.y = relevant_coordinate as u32;
            } else if direction.z != 0. {
                adjacent.z = relevant_coordinate as u32;
            }

            output[i] = adjacent;
            i += 1;
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
}

pub(crate) fn on_cell_clicked(
    In(click): In<ListenedEvent<Click>>,
    query: Query<(&mut Handle<StandardMaterial>, &MainCube, &Transform)>,
    game: ResMut<Game>,
    commands: Commands,
    asset_server: Res<AssetServer>,
) -> Bubble {
    let cell_clicked = query.get(click.target).unwrap();
    match game.phase {
        GamePhase::Play => on_cell_clicked_play_phase(cell_clicked, &query, game),
        GamePhase::PlaceUnits => {
            on_cell_clicked_place_units_phase(cell_clicked, &query, game, commands, asset_server)
        }
    }
    Bubble::Up
}

fn on_cell_clicked_place_units_phase(
    cell_clicked: (&Handle<StandardMaterial>, &MainCube, &Transform),
    query: &Query<(&mut Handle<StandardMaterial>, &MainCube, &Transform)>,
    mut game: ResMut<Game>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let game = &mut *game; // Convert game to normal rust reference for partial borrow
    let coords = cell_clicked.1.coords;
    if game.board.get_unit_at(coords).is_none() {
        if let Some(mut unit) = game.stored_units.pop() {
            if unit.entity.is_none() {
                spawn_unit_mesh(&mut commands, &mut unit, coords, game, query, asset_server);
                unit.coords = coords;
            }
            game.board.units.push(unit);
        }
    }
    if game.stored_units.is_empty() {
        game.phase = GamePhase::Play;
    }
}

fn on_cell_clicked_play_phase(
    cell_clicked: (&Handle<StandardMaterial>, &MainCube, &Transform),
    query: &Query<(&mut Handle<StandardMaterial>, &MainCube, &Transform)>,
    mut game: ResMut<Game>,
) {
    let coords = cell_clicked.1.coords;
    dbg!(coords);
    game.selected_cell = Some(coords);
    reset_cells_new_selection(&mut game);
    let cube_side_length = game.board.cube_side_length;
    if let Some(occupant) = game.board.get_unit_at(coords) {
        // Mark which cells the selected unit can go to
        let cells_can_go = occupant.where_can_go(cube_side_length);
        for cell_coords in cells_can_go {
            let cell = game.board.get_cell_mut(cell_coords);
            match cell {
                None => {
                    warn!("Cell {:?} doesn't exist", cell_coords);
                }
                Some(cell) => cell.selected_unit_can_go = true,
            }
        }
    }
}

fn reset_cells_new_selection(game: &mut Game) {
    for cell in game.board.get_all_cells_mut() {
        cell.selected_unit_can_go = false;
    }
}

pub(crate) fn spawn_unit_mesh(
    commands: &mut Commands,
    unit: &mut Unit,
    coords: CellCoordinates,
    game: &Game,
    query: &Query<(&mut Handle<StandardMaterial>, &MainCube, &Transform)>,
    asset_server: Res<AssetServer>,
) {
    let plane = game
        .board
        .get_cell(coords)
        .unwrap()
        .plane
        .expect("No plane entity for Cell");
    let mut translation = query.get(plane).unwrap().2.translation;
    let scale = 1. / game.board.cube_side_length as f32 / 3.;
    translation += coords.normal_direction() * scale;
    let transform = Transform::from_translation(translation).with_scale(Vec3::splat(scale));
    let entity = scene::spawn_unit(commands, transform, asset_server);
    unit.set_entity(entity);
}
