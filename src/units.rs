use std::slice::{Iter, IterMut};

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

    pub(crate) fn material_value(&self) -> f32 {
        match self {
            UnitType::Rook => 5.,
            UnitType::Bishop => 3.5,
            UnitType::King => 1000.,
            UnitType::Pawn(_, _) => 1.,
            UnitType::Knight => 3.,
            UnitType::Queen => 9.,
        }
    }

    pub(crate) fn symbol(&self) -> char {
        match self {
            UnitType::Rook => '♖',
            UnitType::Bishop => '♗',
            UnitType::King => '♔',
            UnitType::Pawn(_, _) => '♙',
            UnitType::Knight => '♘',
            UnitType::Queen => '♕',
        }
    }
}

#[derive(Debug, Default, Clone)]
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

    pub(crate) fn game_starting_configuration(cube_side_length: u32) -> Units {
        let mut output = Units::default();
        macro_rules! unit_mirror {
            ($color:tt $type:tt at ($x:tt, $y:tt, $z:tt, $normal_positive:tt)) => {
                let unit = Unit::new(
                    UnitType::$type,
                    Team::$color,
                    CellCoordinates::new($x, $y, $z, $normal_positive),
                );
                let mut unit2 = unit.clone();
                unit2.coords = unit2.coords.opposite(cube_side_length);
                unit2.team = unit.team.opposite();
                output.add_unit(unit);
                output.add_unit(unit2);
            };
        }

        macro_rules! unit_mirror_pawn {
            ($color:tt walking in $direction:tt at ($x:tt, $y:tt, $z:tt, $normal_positive:tt)) => {
                let unit = Unit::new(
                    UnitType::Pawn(RadialDirection::$direction, false),
                    Team::$color,
                    CellCoordinates::new($x, $y, $z, $normal_positive),
                );
                let unit2 = Unit::new(
                    UnitType::Pawn(RadialDirection::$direction, false),
                    Team::$color.opposite(),
                    CellCoordinates::new($x, $y, $z, $normal_positive).opposite(cube_side_length),
                );
                output.add_unit(unit);
                output.add_unit(unit2);
            };
        }

        unit_mirror!(White King at (4, 0, 4, true));
        unit_mirror!(White Knight at (3, 0, 3, true));
        unit_mirror!(White Queen at (4, 4, 0, true));
        unit_mirror!(White Rook at (0, 4, 4, true));
        unit_mirror_pawn!(White walking in ClockwiseY at (3, 4, 0, true));
        unit_mirror_pawn!(White walking in CounterX at (4, 3, 0, true));
        unit_mirror_pawn!(White walking in ClockwiseZ at (0, 3, 4, true));
        unit_mirror_pawn!(White walking in CounterY at (0, 4, 3, true));
        unit_mirror_pawn!(White walking in ClockwiseX at (4, 0, 3, true));
        unit_mirror_pawn!(White walking in CounterZ at (3, 0, 4, true));

        output
    }

    pub(crate) fn all_units_iter_mut(&mut self) -> IterMut<Unit> {
        self.units.iter_mut()
    }

    pub(crate) fn all_units_iter(&self) -> Iter<Unit> {
        self.units.iter()
    }

    pub(crate) fn remove_unit(&mut self, coords: CellCoordinates) -> Option<Unit> {
        let Some(index) = self.units.iter().position(|unit| unit.coords==coords) else {
            return None;
        };

        Some(self.units.swap_remove(index))
    }
}
