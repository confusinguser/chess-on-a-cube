use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::scene::{self, MainCube};

#[derive(Resource)]
pub(crate) struct Game {
    pub(crate) board: Board,
    selected_cell: Option<CellCoordinates>,
    phase: GamePhase,
}
impl Game {
    pub(crate) fn new(cube_side_length: u32) -> Self {
        Game {
            board: Board::new(cube_side_length),
            selected_cell: None,
            phase: GamePhase::Play,
        }
    }
}

#[derive(PartialEq)]
enum GamePhase {
    PlaceUnits,
    Play,
}

pub(crate) struct Board {
    board: [Vec<Cell>; 6],
    pub(crate) cube_side_length: u32,
}

impl Board {
    fn get_cell(&self, coords: CellCoordinates) -> Option<&Cell> {
        self.board[coords.side as usize].get((coords.y * self.cube_side_length + coords.x) as usize)
    }
    pub(crate) fn get_cell_mut(&mut self, coords: CellCoordinates) -> Option<&mut Cell> {
        self.board[coords.side as usize]
            .get_mut((coords.y * self.cube_side_length + coords.x) as usize)
    }
    fn new(cube_side_length: u32) -> Self {
        let mut board: [Vec<Cell>; 6] = Default::default();
        for side in &mut board {
            *side = vec![Cell::default(); cube_side_length.pow(2) as usize];
        }
        Board {
            board,
            cube_side_length,
        }
    }
}

#[derive(Default, Clone)]
pub(crate) struct Cell {
    cell_type: CellType,
    occupant: Option<Unit>,
    plane: Option<Entity>,
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
enum CellType {
    #[default]
    Empty,
    Black,
}

#[derive(Clone)]
pub(crate) struct Unit {
    unit_type: UnitType,
    cell: CellCoordinates,
}

impl Unit {
    fn new(unit_type: UnitType, cell: CellCoordinates) -> Self {
        Unit { unit_type, cell }
    }

    fn where_can_go(&self) -> Vec<CellCoordinates> {
        todo!();
    }
}

#[derive(Clone)]
enum UnitType {
    Normal,
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct CellCoordinates {
    side: u32,
    x: u32,
    y: u32,
}

impl CellCoordinates {
    fn from_side_index(side: u32, index: u32, cube_size: u32) -> Self {
        CellCoordinates {
            side,
            x: index % cube_size,
            y: index / cube_size,
        }
    }

    pub(crate) fn new(side: u32, x: u32, y: u32) -> Self {
        CellCoordinates { side, x, y }
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
    let cell = game.board.get_cell_mut(cell_clicked.1.coords).unwrap();
    cell.set_occupant(Unit::new(UnitType::Normal, cell_clicked.1.coords));
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
            scene::unselect_cell_material(old_selected_material);
        }
    }

    game.selected_cell = Some(cell_clicked.1.coords);

    if let Some(selected_cell) = game.selected_cell {
        if let Some(selected_cell) = game.board.get_cell_mut(selected_cell) {
            if let Some(occupant) = &selected_cell.occupant {
                let unit_can_go = occupant.where_can_go();
            }
        }
    }
}
