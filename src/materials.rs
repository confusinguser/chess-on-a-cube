use crate::cell::CellColor;
use bevy::prelude::*;

pub(crate) fn select_cell_material(material: &mut StandardMaterial, color: CellColor) {
    material.base_color = blend_colors(cell_base_color(color), Color::YELLOW, 0.3);
}

pub(crate) fn normal_cell_material(material: &mut StandardMaterial, color: CellColor) {
    material.base_color = cell_base_color(color);
}

pub(crate) fn can_go_cell_material(material: &mut StandardMaterial, color: CellColor) {
    material.base_color = blend_colors(cell_base_color(color), Color::LIME_GREEN, 0.3);
}

fn cell_base_color(color: CellColor) -> Color {
    match color {
        CellColor::White => Color::ANTIQUE_WHITE,
        CellColor::Black => Color::BLACK,
        CellColor::Gray => Color::GRAY,
    }
}

fn blend_colors(c1: Color, c2: Color, fac: f32) -> Color {
    c1 * fac + c2 * (1. - fac)
}
