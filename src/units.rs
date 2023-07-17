use crate::cell::{Board, CellCoordinates};
use crate::gamemanager::Team;
use bevy::prelude::*;

#[derive(Clone, Debug)]
pub(crate) struct Unit {
    pub(crate) unit_type: UnitType,
    pub(crate) coords: CellCoordinates,
    /// The entity that represents this unit on the board
    pub(crate) entity: Option<Entity>,
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
            team,
            dead: false,
            range: unit_type.range(),
        }
    }

    /// Returns a list of tuples where the first element is the coordinate of the cell the unit
    /// can move to, and the second element is how far away the cell is (how much it depletes the
    /// range of the unit)
    pub(crate) fn cells_can_move_to(&self, board: &Board) -> Vec<CellCoordinates> {
        match self.unit_type {
            UnitType::Melee => melee_unit_movement(self, board),
            UnitType::Laser => laser_unit_movement(self, board),
        }
    }

    pub(crate) fn set_entity(&mut self, entity: Entity) {
        self.entity = Some(entity);
    }

    pub(crate) fn move_unit_to(&mut self, coords: CellCoordinates) {
        self.coords = coords
    }
}

fn melee_unit_movement(unit: &Unit, board: &Board) -> Vec<CellCoordinates> {
    unit.coords.get_cells_max_dist(unit.range, true, board)
}

fn laser_unit_movement(unit: &Unit, board: &Board) -> Vec<CellCoordinates> {
    unit.coords.get_cells_max_dist(unit.range, true, board)
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum UnitType {
    Melee,
    Laser,
}

impl UnitType {
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

    pub(crate) fn remove_dead_units(&mut self) {
        self.units.retain(|unit| !unit.dead)
    }

    pub(crate) fn add_unit(&mut self, unit: Unit) {
        self.units.push(unit)
    }
}
