use bevy::prelude::*;

pub(crate) fn select_cell_material(material: &mut StandardMaterial) {
    material.base_color = Color::YELLOW
}

pub(crate) fn normal_cell_material(material: &mut StandardMaterial) {
    material.base_color = Color::ANTIQUE_WHITE;
}

pub(crate) fn can_go_cell_material(material: &mut StandardMaterial) {
    material.base_color = Color::LIME_GREEN;
}

pub(crate) fn can_attack_cell_material(material: &mut StandardMaterial) {
    material.base_color = Color::YELLOW_GREEN;
}
