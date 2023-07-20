use crate::cell::CellCoordinates;
use crate::gamemanager::Team;
use crate::utils::RadialDirection;
use bevy::prelude::*;

#[derive(Clone, Debug)]
pub(crate) struct Unit {
    pub(crate) unit_type: UnitType,
    pub(crate) coords: CellCoordinates,
    /// The entity that represents this unit on the board
    pub(crate) entity: Option<Entity>,
    pub(crate) team: Team,
    pub(crate) dead: bool,
}

impl Unit {
    pub(crate) fn new(unit_type: UnitType, team: Team, coords: CellCoordinates) -> Self {
        Unit {
            unit_type,
            coords,
            entity: None,
            team,
            dead: false,
        }
    }

    pub(crate) fn set_entity(&mut self, entity: Entity) {
        self.entity = Some(entity);
    }

    pub(crate) fn move_unit_to(&mut self, coords: CellCoordinates) {
        self.coords = coords
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[allow(unused)]
pub(crate) enum UnitType {
    Rook,
    Bishop,
    King,
    /// (The direction that the pawn moves in, if the pawn has moved before)
    Pawn(RadialDirection, bool),
    Knight,
    Queen,
}

impl UnitType {
    pub(crate) fn model_name(&self) -> &str {
        match self {
            UnitType::Rook => "rook",
            UnitType::Bishop => "bishop",
            UnitType::King => "king",
            UnitType::Pawn(_, _) => "pawn",
            UnitType::Knight => "knight",
            UnitType::Queen => "queen",
        }
    }

    pub(crate) fn can_capture_over_edge(&self) -> bool {
        matches!(self, Self::Knight)
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
