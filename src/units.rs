use crate::gamemanager::{Board, CellCoordinates, Team};
use bevy::prelude::*;

#[derive(Clone, Debug)]
pub(crate) struct Unit {
    pub(crate) unit_type: UnitType,
    pub(crate) coords: CellCoordinates,
    /// The entity that represents this unit on the board
    pub(crate) entity: Option<Entity>,
    pub(crate) hp: i32,
    pub(crate) team: Team,
    pub(crate) dead: bool,
    /// How many more cells this unit can move in on this turn
    pub(crate) range: u32,
}

impl Unit {
    pub(crate) fn new(unit_type: UnitType, team: Team, coords: CellCoordinates) -> Self {
        Unit {
            unit_type,
            coords,
            entity: None,
            hp: unit_type.starting_hp(),
            team,
            dead: false,
            range: unit_type.range(),
        }
    }

    /// Returns a list of tuples where the first element is the coordinate of the cell the unit
    /// can move to, and the second element is how far away the cell is (how much it depletes the
    /// range of the unit)
    pub(crate) fn cells_can_move_to(&self, board: &Board) -> Vec<(CellCoordinates, u32)> {
        match self.unit_type {
            UnitType::Melee => melee_unit_movement(self, board),
            UnitType::Laser => laser_unit_movement(self, board),
        }
    }

    pub(crate) fn cells_can_attack(&self, board: &Board) -> Vec<CellCoordinates> {
        match self.unit_type {
            UnitType::Melee => melee_unit_attack(self, board),
            UnitType::Laser => Vec::new(),
        }
    }

    pub(crate) fn set_entity(&mut self, entity: Entity) {
        self.entity = Some(entity);
    }

    pub(crate) fn take_damage(&mut self, damage: i32) {
        self.hp -= damage;
        if self.hp <= 0 {
            self.dead = true;
        }
    }
}

fn melee_unit_movement(unit: &Unit, board: &Board) -> Vec<(CellCoordinates, u32)> {
    unit.coords.get_cells_max_dist(unit.range, true, board)
}

fn laser_unit_movement(unit: &Unit, board: &Board) -> Vec<(CellCoordinates, u32)> {
    unit.coords.get_cells_max_dist(unit.range, true, board)
}

fn melee_unit_attack(unit: &Unit, board: &Board) -> Vec<CellCoordinates> {
    unit.coords.get_adjacent(board.cube_side_length).into()
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum UnitType {
    Melee,
    Laser,
}

impl UnitType {
    fn starting_hp(&self) -> i32 {
        match self {
            Self::Melee => 4,
            Self::Laser => 2,
        }
    }

    pub(crate) fn damage(&self) -> i32 {
        match self {
            Self::Melee => 2,
            Self::Laser => 1,
        }
    }

    pub(crate) fn range(&self) -> u32 {
        match self {
            Self::Melee => 2,
            Self::Laser => 1,
        }
    }

    pub(crate) fn model_name(&self) -> &str {
        match self {
            UnitType::Melee => "melee",
            UnitType::Laser => "laser",
        }
    }
}
