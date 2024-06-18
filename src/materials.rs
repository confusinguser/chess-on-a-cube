use bevy::prelude::*;

use crate::cell::CellColor;
use crate::gamemanager::Palette;

pub(crate) fn select_cell_material(
    material: &mut StandardMaterial,
    palette: Palette,
    color: CellColor,
) {
    material.base_color = blend_colors(color.base_color(palette), Color::YELLOW, 0.3);
}

pub(crate) fn normal_cell_material(
    material: &mut StandardMaterial,
    palette: Palette,
    color: CellColor,
) {
    material.base_color = color.base_color(palette);
}

pub(crate) fn can_go_cell_material(
    material: &mut StandardMaterial,
    palette: Palette,
    color: CellColor,
) {
    material.base_color = blend_colors(color.base_color(palette), Color::LIME_GREEN, 0.3);
}

fn blend_colors(c1: Color, c2: Color, fac: f32) -> Color {
    c1 * fac + c2 * (1. - fac)
}
