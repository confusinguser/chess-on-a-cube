use crate::units::*;
use std::collections::{BTreeMap, VecDeque};

use bevy::prelude::*;
use bevy::scene::SceneInstance;
use bevy_mod_picking::prelude::*;

use crate::scene::{self, MainCube};

#[derive(Resource, Debug)]
pub(crate) struct Game {
    pub(crate) board: Board,
    pub(crate) units: Units,
    pub(crate) selected_cell: Option<CellCoordinates>,
    pub(crate) phase: GamePhase,
    pub(crate) stored_units: Vec<Unit>,
}
impl Game {
    pub(crate) fn new(cube_side_length: u32) -> Self {
        Game {
            board: Board::new(cube_side_length),
            units: Default::default(),
            selected_cell: None,
            phase: GamePhase::PlaceUnits,
            stored_units: vec![
                Unit::new(UnitType::Melee, Team::Black, CellCoordinates::default()),
                Unit::new(UnitType::Laser, Team::White, CellCoordinates::default()),
            ],
        }
    }

    fn next_player_turn(&mut self) {
        if let GamePhase::Play(turn) = self.phase {
            self.phase = GamePhase::Play(match turn {
                Team::Black => Team::White,
                Team::White => Team::Black,
            })
        };
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Team {
    Black,
    White,
}

#[derive(Debug, Default)]
pub(crate) struct Units {
    units: Vec<Unit>,
}

impl Units {
    pub(crate) fn get_unit(&self, coords: CellCoordinates) -> Option<&Unit> {
        self.units.iter().find(|unit| unit.coords == coords)
    }

    pub(crate) fn get_unit_mut(&mut self, coords: CellCoordinates) -> Option<&mut Unit> {
        self.units.iter_mut().find(|unit| unit.coords == coords)
    }

    /// TODO: Make unit tests for this one
    pub(crate) fn get_units_mut(
        &mut self,
        coords_list: Vec<CellCoordinates>,
    ) -> Vec<Option<&mut Unit>> {
        let mut found_units = self
            .units
            .iter_mut()
            .filter(|unit| coords_list.iter().any(|coords| *coords == unit.coords))
            .collect::<Vec<&mut Unit>>();

        let mut output = Vec::with_capacity(coords_list.len());
        output.resize_with(coords_list.len(), || None);
        for coords in coords_list {
            let position = found_units.iter().position(|unit| unit.coords == coords);
            if let Some(position) = position {
                output[position] = Some(found_units.remove(position));
            }
        }
        output
    }

    pub(crate) fn get_unit_from_entity(&self, entity: Entity) -> Option<&Unit> {
        self.units.iter().find(|unit| {
            if let Some(unit_entity) = unit.entity {
                unit_entity == entity
            } else {
                false
            }
        })
    }

    pub(crate) fn is_unit_at(&self, coords: CellCoordinates) -> bool {
        self.units.iter().any(|unit| unit.coords == coords)
    }

    fn remove_dead_units(&mut self) {
        self.units.retain(|unit| !unit.dead)
    }
}

#[derive(PartialEq, Debug)]
pub(crate) enum GamePhase {
    PlaceUnits,
    Play(Team),
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
    fn new(cube_side_length: u32) -> Self {
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

pub(crate) fn on_cell_clicked(
    In(click): In<ListenedEvent<Click>>,
    mut query: Query<(Option<&MainCube>, &mut Transform)>,
    mut game: ResMut<Game>,
    commands: Commands,
    asset_server: Res<AssetServer>,
) -> Bubble {
    let game = &mut *game;
    match game.phase {
        GamePhase::Play(turn) => {
            on_cell_clicked_play_phase(click.target, &mut query, game, commands, turn)
        }
        GamePhase::PlaceUnits => on_cell_clicked_place_units_phase(
            click.target,
            &mut query,
            game,
            commands,
            asset_server,
        ),
    }
    Bubble::Up
}

fn on_cell_clicked_place_units_phase(
    target: Entity,
    query: &mut Query<(Option<&MainCube>, &mut Transform)>,
    game: &mut Game,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let game = &mut *game; // Convert game to normal rust reference for partial borrow
    let cell_clicked = query.get(target);
    let coords;
    if let Ok(cell_clicked) = cell_clicked {
        if cell_clicked.0.is_none() {
            // Didn't click a part of the cube
            return;
        }
        coords = cell_clicked.0.unwrap().coords;
    } else {
        return;
    }

    if game.units.get_unit(coords).is_none() {
        if let Some(mut unit) = game.stored_units.pop() {
            if unit.entity.is_none() {
                spawn_unit_entity(&mut commands, &mut unit, coords, game, query, asset_server);
                unit.coords = coords;
            }
            game.units.units.push(unit);
        }
    }
    if game.stored_units.is_empty() {
        game.phase = GamePhase::Play(Team::Black);
    }
}

fn on_cell_clicked_play_phase(
    target: Entity,
    query: &mut Query<(Option<&MainCube>, &mut Transform)>,
    mut game: &mut Game,
    mut commands: Commands,
    turn: Team,
) {
    let cell_clicked = query.get(target);
    let clicked_coords;
    if let Ok(cell_clicked) = cell_clicked {
        if cell_clicked.0.is_none() {
            // Didn't click a part of the cube
            return;
        }
        clicked_coords = cell_clicked.0.unwrap().coords;
    } else {
        return;
    }

    let old_selected_cell = game.selected_cell;
    game.selected_cell = Some(clicked_coords);

    let clicked_cell = game.board.get_cell_mut(clicked_coords).unwrap();

    fn capture_unit(
        commands: &mut Commands,
        captured_unit_coords: CellCoordinates,
        units: &mut Units,
        turn: Team,
    ) {
        let captured_unit = units.get_unit_mut(captured_unit_coords);
        if let Some(captured_unit) = captured_unit {
            if captured_unit.team == turn {
                return;
            }
            kill_unit_entity(commands, captured_unit);
            captured_unit.dead = true;
            units.remove_dead_units();
        }
    }

    if clicked_cell.selected_unit_can_move_to {
        if game.units.is_unit_at(clicked_coords) {
            capture_unit(
                &mut commands,
                clicked_coords, // captured_unit_coords
                &mut game.units,
                turn,
            );
        }

        // Move selected unit
        if let Some(capturing_unit_coords) = old_selected_cell {
            if let Some(capturing_unit) = game.units.get_unit_mut(capturing_unit_coords) {
                capturing_unit.move_unit_to(clicked_coords);
                move_unit_entity(clicked_coords, &mut game.board, query, capturing_unit);
                game.next_player_turn();
            }
        }
    }

    // Mark cells
    reset_cells_new_selection(game);
    if let Some(occupant) = game.units.get_unit(clicked_coords) {
        if occupant.team != turn {
            return;
        }
        // Mark which cells the selected unit can go to
        let cells_can_go = occupant.cells_can_move_to(&game.board);
        for cell_coords in cells_can_go {
            let cell = game.board.get_cell_mut(cell_coords);
            match cell {
                None => {
                    warn!("Cell {:?} doesn't exist", cell_coords);
                }
                Some(cell) => {
                    cell.selected_unit_can_move_to = true;
                }
            }
        }
    }
}

fn reset_cells_new_selection(game: &mut Game) {
    for cell in game.board.get_all_cells_mut() {
        cell.selected_unit_can_move_to = false;
    }
}

pub(crate) fn spawn_unit_entity(
    commands: &mut Commands,
    unit: &mut Unit,
    coords: CellCoordinates,
    game: &Game,
    query: &mut Query<(Option<&MainCube>, &mut Transform)>,
    asset_server: Res<AssetServer>,
) {
    let plane = game.board.get_cell(coords).unwrap().plane;
    let mut translation = query.get(plane).unwrap().1.translation;
    let scale = 1. / game.board.cube_side_length as f32 / 3.;
    translation += coords.normal_direction() * scale;
    let transform = Transform::from_translation(translation).with_scale(Vec3::splat(scale));
    let model_name = unit.unit_type.model_name();
    let entity = scene::spawn_unit(commands, transform, asset_server, model_name);
    unit.set_entity(entity);
}

pub(crate) fn move_unit_entity(
    coords: CellCoordinates,
    board: &mut Board,
    query: &mut Query<(Option<&MainCube>, &mut Transform)>,
    unit: &Unit,
) {
    if unit.entity.is_none() {
        return;
    }

    let plane = board.get_cell(coords).unwrap().plane;
    let mut target_translation = query.get(plane).unwrap().1.translation;
    let scale = 1. / board.cube_side_length as f32 / 3.;
    target_translation += coords.normal_direction() * scale;

    query.get_mut(unit.entity.unwrap()).unwrap().1.translation = target_translation;
}

//TODO fix
pub(crate) fn on_unit_clicked(
    In(click): In<ListenedEvent<Click>>,
    scene_query: Query<(Entity, &SceneInstance)>,
    mut query: Query<(Option<&MainCube>, &mut Transform)>,
    mut game: ResMut<Game>,
    scene_manager: Res<SceneSpawner>,
    commands: Commands,
) -> Bubble {
    let game = &mut *game;
    if let GamePhase::Play(turn) = game.phase {
        if let Some(unit) = game.units.get_unit_from_entity(click.target) {
            if let Some(cell) = game.board.get_cell(unit.coords) {
                on_cell_clicked_play_phase(cell.plane, &mut query, game, commands, turn);
            } else {
                warn!("Cell is None");
            }
        } else {
            warn!("Unit is None");
        }
    }
    Bubble::Burst
}

fn kill_unit_entity(commands: &mut Commands, unit: &mut Unit) {
    if let Some(entity) = unit.entity {
        scene::kill_unit(commands, entity);
    }
}
