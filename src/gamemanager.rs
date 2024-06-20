use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::{ai, movement, units::*};
use crate::ai::AICache;
use crate::cell::*;
use crate::movement::GameMove;
use crate::scene::{self, MainCube, SceneChild};

#[derive(Resource, Debug)]
pub(crate) struct Game {
    pub(crate) board: Board,
    pub(crate) units: Units,
    pub(crate) selected_cell: Option<CellCoordinates>,
    pub(crate) turn: Team,
    pub(crate) entities_to_move: Vec<(Entity, CellCoordinates)>,
    pub(crate) palette: Palette,
    pub(crate) ai_playing: Option<Team>,
}

impl Game {
    pub(crate) fn new(cube_side_length: u32) -> Self {
        Game {
            board: Board::new(cube_side_length),
            units: Units::game_starting_configuration(cube_side_length),
            selected_cell: None,
            turn: Team::White,
            entities_to_move: Vec::new(),
            palette: Palette::Pinkish,
            ai_playing: Some(Team::Black),
        }
    }

    fn next_player_turn(&mut self) {
        self.turn = self.turn.opposite()
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub(crate) enum Palette {
    Filippa,
    Pinkish,
}

impl Palette {
    fn get_colors_str(&self) -> [&str; 3] {
        match self {
            Self::Filippa => ["473A2A", "A7805E", "ECC998"],
            Self::Pinkish => ["B23A48", "FB9489", "FCB8B0"],
        }
    }

    pub(crate) fn get_colors(&self) -> [Color; 3] {
        let mut output: [Color; 3] = Default::default();
        for (i, str) in self.get_colors_str().iter().enumerate() {
            output[i] = Color::hex(str).unwrap();
        }
        output
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

    pub(crate) fn opposite(&self) -> Self {
        match self {
            Team::Black => Team::White,
            Team::White => Team::Black,
        }
    }

    pub(crate) fn sign(&self) -> i32 {
        match self {
            Team::Black => -1,
            Team::White => 1,
        }
    }
}

pub(crate) fn on_cell_clicked(
    mut click_events: EventReader<Pointer<Click>>,
    query: Query<(Option<&MainCube>, &mut Transform)>,
    mut game: ResMut<Game>,
    mut commands: Commands,
) {
    let game = &mut *game;
    let Some(click_event) = click_events.read().next() else {
        return;
    };
    let mut target = click_event.target;
    for click_event in click_events.read() {
        target = click_event.target;
    }
    on_cell_clicked_internal(target, &query, game, &mut commands)
}

fn on_cell_clicked_internal(
    target: Entity,
    query: &Query<(Option<&MainCube>, &mut Transform)>,
    game: &mut Game,
    commands: &mut Commands,
) {
    let Ok(cell_clicked) = query.get(target) else {
        return;
    };
    if cell_clicked.0.is_none() {
        // Didn't click a part of the cube
        game.selected_cell = None;
        reset_cells_new_selection(game);
        return;
    }
    let clicked_coords = cell_clicked.0.unwrap().coords;

    let old_selected_cell = game.selected_cell;
    game.selected_cell = Some(clicked_coords);

    let clicked_cell = game.board.get_cell_mut(clicked_coords).unwrap();

    if clicked_cell.selected_unit_can_move_to {
        // Move selected unit
        if let Some(from) = old_selected_cell {
            let game_move = GameMove {
                from,
                to: clicked_coords,
            };
            if make_move(game_move, game, commands) {
                assert!(game.units.get_unit(clicked_coords).is_some());
                game.next_player_turn();
            }
        }
    }

    // Mark cells
    reset_cells_new_selection(game);
    let Some(unit) = game.units.get_unit(clicked_coords) else {
        return;
    };
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

pub(crate) fn make_move(game_move: GameMove, game: &mut Game, commands: &mut Commands) -> bool {
    let captured_unit = game.units.get_unit_mut(game_move.to);
    if let Some(captured_unit) = captured_unit {
        if captured_unit.team == game.turn {
            return false;
        }
        if let Some(entity) = captured_unit.entity {
            info!("Killing unit");
            scene::kill_unit(commands, entity);
        };
        captured_unit.dead = true;
        game.units.remove_dead_units();
    }

    let Some(unit) = game.units.get_unit_mut(game_move.from) else {
        return false;
    };
    if unit.team != game.turn {
        return false;
    }

    unit.move_unit_to(game_move.to);
    let Some(entity) = unit.entity else {
        warn!("Unit entity was None");
        return false;
    };
    game.entities_to_move.push((entity, game_move.to));
    if let UnitType::Pawn(_, ref mut has_moved) = unit.unit_type {
        *has_moved = true;
    }
    true
}

fn reset_cells_new_selection(game: &mut Game) {
    for cell in game.board.get_all_cells_mut() {
        cell.selected_unit_can_move_to = false;
    }
}

pub(crate) fn spawn_unit_entity(
    commands: &mut Commands,
    unit: &mut Unit,
    entities_to_move: &mut Vec<(Entity, CellCoordinates)>,
    asset_server: &AssetServer,
) {
    let model_name = unit.unit_type.model_name();
    let entity = scene::spawn_unit(commands, asset_server, model_name);
    entities_to_move.push((entity, unit.coords));
    unit.set_entity(entity);
}

pub(crate) fn on_unit_clicked(
    mut click_events: EventReader<Pointer<Click>>,
    query: Query<(Option<&MainCube>, &mut Transform)>,
    scene_child_query: Query<&SceneChild>,
    mut game: ResMut<Game>,
    mut commands: Commands,
) {
    let game = &mut *game;
    for click in click_events.read() {
        let result = scene_child_query.get(click.target);
        let Ok(scene_child) = result else {
            warn!("Unit that was clicked on has disappeared");
            return;
        };
        if let Some(unit) = game.units.get_unit_from_entity(scene_child.parent_entity) {
            if let Some(cell) = game.board.get_cell(unit.coords) {
                on_cell_clicked_internal(cell.plane, &query, game, &mut commands);
            } else {
                warn!("Cell is None");
            }
        } else {
            warn!("Unit is None");
        }
    }
}

pub(crate) fn ai_play(
    mut game: ResMut<Game>,
    mut commands: Commands,
    mut ai_cache: Local<AICache>,
) {
    if game
        .ai_playing
        .map_or(false, |ai_playing| ai_playing == game.turn)
    {
        // It is AI's turn
        let next_move = ai::next_move(&game.board, &game.units, game.turn, 3, &mut ai_cache);
        make_move(next_move, &mut game, &mut commands);
        game.next_player_turn();
    }
}
