use bevy::prelude::*;
use bevy_eventlistener::{callbacks::ListenerInput, prelude::*};
use bevy_mod_picking::prelude::*;

struct Game {
    board: [Vec<Cell>; 6],
    selected_cell: (u32, u32),
}

struct Cell {
    cell_type: CellType,
    occupant: Option<Unit>,
    plane: Entity,
}

enum CellType {
    EMPTY,
    BLACK,
}

struct Unit {
    unit_type: UnitType,
    cell: (u32, u32),
}

enum UnitType {}

pub fn on_cell_clicked(
    In(click): In<ListenedEvent<Click>>,
    cube: Query<&mut Handle<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) -> Bubble {
    let material = materials.get_mut(cube.get(click.target).unwrap()).unwrap();
    material.base_color = Color::hsl(50., 255., 128.);
    material.base_color_texture = Default::default();
    dbg!(material);

    Bubble::Up
}
