use crate::movement::GameMove;
use crate::{movement, units::*};

use crate::cell::*;
use crate::scene::{self, MainCube, SceneChild};
use bevy::prelude::*;
use bevy::scene::SceneInstance;
use bevy_mod_picking::prelude::*;

#[derive(Resource, Debug)]
pub(crate) struct Game {
    pub(crate) board: Board,
    pub(crate) units: Units,
    pub(crate) selected_cell: Option<CellCoordinates>,
    pub(crate) phase: GamePhase,
    pub(crate) stored_units: Vec<Unit>,
    pub(crate) turn: Team,
    pub(crate) units_to_move: Vec<(Entity, CellCoordinates)>,
}
impl Game {
    pub(crate) fn new(cube_side_length: u32) -> Self {
        Game {
            board: Board::new(cube_side_length),
            units: Default::default(),
            selected_cell: None,
            phase: GamePhase::PlaceUnits,
            stored_units: vec![
                Unit::new(UnitType::Rook, Team::Black, CellCoordinates::default()),
                Unit::new(UnitType::Bishop, Team::White, CellCoordinates::default()),
            ],
            turn: Team::White,
            units_to_move: Vec::new(),
        }
    }

    fn next_player_turn(&mut self) {
        self.turn = match self.turn {
            Team::Black => Team::White,
            Team::White => Team::Black,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Team {
    Black,
    White,
}
impl Team {
    pub(crate) fn color(&self) -> Color {
        match self {
            Self::Black => Color::DARK_GRAY,
            Self::White => Color::BISQUE,
        }
    }
}

#[derive(PartialEq, Debug)]
pub(crate) enum GamePhase {
    PlaceUnits,
    Play,
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
        GamePhase::Play => on_cell_clicked_play_phase(click.target, &mut query, game, commands),
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
                spawn_unit_entity(
                    &mut commands,
                    &mut unit,
                    coords,
                    &mut game.units_to_move,
                    asset_server,
                );
                unit.coords = coords;
            }
            game.units.add_unit(unit);
        }
    }
    if game.stored_units.is_empty() {
        game.phase = GamePhase::Play;
    }
}

fn on_cell_clicked_play_phase(
    target: Entity,
    query: &mut Query<(Option<&MainCube>, &mut Transform)>,
    mut game: &mut Game,
    mut commands: Commands,
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
    ) -> bool {
        let captured_unit = units.get_unit_mut(captured_unit_coords);
        if let Some(captured_unit) = captured_unit {
            if captured_unit.team == turn {
                return false;
            }
            if let Some(entity) = captured_unit.entity {
                scene::kill_unit(commands, entity);
            };
            captured_unit.dead = true;
            units.remove_dead_units();
        }
        true
    }

    if clicked_cell.selected_unit_can_move_to {
        let mut should_move = true;
        if game.units.is_unit_at(clicked_coords) {
            should_move = capture_unit(
                &mut commands,
                clicked_coords, // captured_unit_coords
                &mut game.units,
                game.turn,
            );
        }

        // Move selected unit
        if should_move {
            if let Some(from) = old_selected_cell {
                let game_move = GameMove {
                    from,
                    to: clicked_coords,
                };
                if movement::make_move(game_move, game) {
                    game.next_player_turn();
                }
            }
        }
    }

    // Mark cells
    reset_cells_new_selection(game);
    let Some(unit) = game.units.get_unit(clicked_coords) else { return;};
    if unit.team != game.turn {
        return;
    }
    // Mark which cells the selected unit can go to
    let unit_moves = movement::get_unit_moves(unit, &game.board, &game.units);
    for unit_move in unit_moves {
        let cell = game.board.get_cell_mut(unit_move);
        match cell {
            None => {
                warn!("Cell {:?} doesn't exist", unit_move);
            }
            Some(cell) => {
                let unit_at_destination = game.units.get_unit(unit_move);
                // Check so normal pieces can't capture over edge
                if (unit.unit_type.can_capture_over_edge()
                    || unit_at_destination.is_none()
                    || unit.coords.normal_direction() == unit_move.normal_direction())
                // Prevent taking units on same team
                    && unit_at_destination.map_or(true, |unit_at_d| unit.team != unit_at_d.team)
                {
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
    units_to_move: &mut Vec<(Entity, CellCoordinates)>,
    asset_server: Res<AssetServer>,
) {
    // let plane = game.board.get_cell(coords).unwrap().plane;
    // let mut translation = query.get(plane).unwrap().1.translation;
    // let scale = 1. / game.board.cube_side_length as f32 / 3.;
    // translation += coords.normal_direction().as_vec3() * scale;
    // let transform = Transform::from_translation(translation).with_scale(Vec3::splat(scale));

    let model_name = unit.unit_type.model_name();
    let entity = scene::spawn_unit(commands, asset_server, model_name);
    units_to_move.push((entity, coords));
    unit.set_entity(entity);
}

pub(crate) fn on_unit_clicked(
    In(click): In<ListenedEvent<Click>>,
    mut query: Query<(Option<&MainCube>, &mut Transform)>,
    scene_child_query: Query<&SceneChild>,
    mut game: ResMut<Game>,
    commands: Commands,
) -> Bubble {
    let game = &mut *game;
    if game.phase == GamePhase::Play {
        let Ok(scene_child) = scene_child_query.get(click.target) else {
            warn!("Err when getting scene_child");
            return Bubble::Up;
        };
        if let Some(unit) = game.units.get_unit_from_entity(scene_child.parent_entity) {
            if let Some(cell) = game.board.get_cell(unit.coords) {
                on_cell_clicked_play_phase(cell.plane, &mut query, game, commands);
            } else {
                warn!("Cell is None");
            }
        } else {
            warn!("Unit is None");
        }
    }
    Bubble::Burst
}
