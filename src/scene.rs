use std::f32::consts::PI;

use bevy::prelude::Vec3;
use bevy::prelude::*;
use bevy::scene::SceneInstance;
use bevy_mod_picking::prelude::*;

use crate::cell::{Cell, CellColor, CellCoordinates};
use crate::gamemanager::{self, spawn_unit_entity, Game};
use crate::materials;

pub(crate) fn construct_cube(
    side_length: u32,
    meshes: &mut ResMut<Assets<Mesh>>,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    material: &StandardMaterial,
    game: &mut ResMut<Game>,
) {
    fn choose_color(
        side_length: u32,
        i: u32,
        mut c1: CellColor,
        mut c2: CellColor,
        switch_colors: bool,
    ) -> CellColor {
        if switch_colors {
            std::mem::swap(&mut c1, &mut c2);
        }
        #[allow(clippy::collapsible_else_if)]
        if side_length % 2 == 0 {
            if (i / side_length + i % 2) % 2 == 0 {
                c1
            } else {
                c2
            }
        } else {
            if i % 2 == 0 {
                c1
            } else {
                c2
            }
        }
    }

    let plane_mesh: Handle<Mesh> = meshes.add(shape::Plane::default().into());
    let spacing = 1. / side_length as f32;
    let offset = 0.5 - spacing / 2.;
    // The total side length of cube is always 1, so we offset
    // by 0.5 to get middle in origo. When cube at origo, half of its side is in negative
    // quadrant, so therefore we subtract the part that is already offset from this phenomenon.
    for side in 0..6 {
        //        lookup_planes.planes[side] = vec![None; side_length.pow(2) as usize];
        for i in 0..side_length.pow(2) {
            let translation;
            let mut rotation;
            let color: CellColor;
            #[allow(clippy::needless_late_init)]
            let coords;
            match side {
                0 | 1 => {
                    translation = Vec3::new(
                        (i % side_length) as f32 * spacing - offset,
                        if side % 2 == 0 { 0.5 } else { -0.5 },
                        (i / side_length % side_length) as f32 * spacing - offset,
                    );
                    rotation = Vec3::new(0., 0., 2.); // Up/down rotate 180 degrees, which is 2 turns
                    color = choose_color(
                        side_length,
                        i,
                        CellColor::Bright,
                        CellColor::Mid,
                        side % 2 == 0,
                    );
                    coords = CellCoordinates::new(
                        i % side_length + 1,
                        0,
                        i / side_length % side_length + 1,
                        side % 2 == 0,
                    )
                }
                2 | 3 => {
                    translation = Vec3::new(
                        (i % side_length) as f32 * spacing - offset,
                        (i / side_length % side_length) as f32 * spacing - offset,
                        if side % 2 == 1 { 0.5 } else { -0.5 },
                    );
                    rotation = Vec3::new(1., 0., 0.);
                    color = choose_color(
                        side_length,
                        i,
                        CellColor::Dark,
                        CellColor::Bright,
                        side % 2 == 1,
                    );
                    coords = CellCoordinates::new(
                        i % side_length + 1,
                        i / side_length % side_length + 1,
                        0,
                        side % 2 == 1,
                    )
                }
                4 | 5 => {
                    translation = Vec3::new(
                        if side % 2 == 0 { 0.5 } else { -0.5 },
                        (i / side_length % side_length) as f32 * spacing - offset,
                        (i % side_length) as f32 * spacing - offset,
                    );
                    rotation = Vec3::new(0., 0., 1.);
                    color = choose_color(
                        side_length,
                        i,
                        CellColor::Mid,
                        CellColor::Dark,
                        side % 2 == 0,
                    );
                    coords = CellCoordinates::new(
                        0,
                        i / side_length % side_length + 1,
                        i % side_length + 1,
                        side % 2 == 0,
                    )
                }
                _ => unreachable!(),
            }

            rotation *= Vec3::splat(PI / 2.);
            if side % 2 == 0 {
                rotation.x -= if rotation.x == 0. { 0. } else { PI };
                rotation.y -= if rotation.y == 0. { 0. } else { PI };
                rotation.z -= if rotation.z == 0. { 0. } else { PI };
            }

            let plane = commands
                .spawn((
                    PbrBundle {
                        mesh: plane_mesh.clone(),
                        material: materials.add(material.clone()),
                        transform: Transform::from_translation(translation)
                            .with_scale(Vec3::splat(spacing))
                            .with_rotation(Quat::from_scaled_axis(rotation)),
                        ..default()
                    },
                    PickableBundle::default(),
                    RaycastPickTarget::default(),
                    MainCube { coords },
                    OnPointer::<Click>::run_callback(gamemanager::on_cell_clicked),
                ))
                .id();

            let cell = Cell::new(plane, coords, color);
            game.board.new_cell(coords, cell);
        }
    }
}

#[derive(Component)]
pub(crate) struct MainCube {
    pub(crate) coords: CellCoordinates,
}

pub(crate) fn update_cell_colors(
    query: Query<(&mut Handle<StandardMaterial>, &MainCube)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game: ResMut<Game>,
) {
    for cell in game.board.get_all_cells() {
        let plane = cell.plane;

        let query_result = query.get(plane).unwrap();
        let material = materials.get_mut(query_result.0).unwrap();
        if game.selected_cell.map_or(false, |x| x == cell.coords) {
            materials::select_cell_material(material, game.palette, cell.color);
        } else if cell.selected_unit_can_move_to {
            materials::can_go_cell_material(material, game.palette, cell.color);
        } else {
            materials::normal_cell_material(material, game.palette, cell.color);
        }
    }
}

/// A "flag" to make a separate system do various things with the created entities
#[derive(Component, Default, Debug)]
pub(crate) struct PrepareUnit;

pub(crate) fn spawn_unit(
    commands: &mut Commands,
    asset_server: &AssetServer,
    model_name: &str,
) -> Entity {
    let entity = commands
        .spawn((
            SceneBundle {
                scene: asset_server.load(format!("models/{}.glb#Scene0", model_name)),
                ..default()
            },
            PrepareUnit,
        ))
        .id();
    entity
}

#[derive(Component)]
pub(crate) struct SceneChild {
    pub(crate) parent_entity: Entity,
}

/// Add pickable and change material color
pub(crate) fn prepare_unit_entity(
    mut commands: Commands,
    mut unloaded_instances: Query<(Entity, &SceneInstance), With<PrepareUnit>>,
    mut material_query: Query<&mut Handle<StandardMaterial>>,
    game: Res<Game>,
    scene_manager: Res<SceneSpawner>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (parent_entity, instance) in unloaded_instances.iter_mut() {
        if !scene_manager.instance_is_ready(**instance) {
            continue;
        }
        commands.entity(parent_entity).remove::<PrepareUnit>();

        let unit = game.units.get_unit_from_entity(parent_entity);
        let color = unit.unwrap().team.color();

        // Iterate over all entities in scene (once it's loaded)
        let handles = scene_manager.iter_instance_entities(**instance);
        for entity in handles {
            commands.entity(entity).insert((
                PickableBundle::default(),
                RaycastPickTarget::default(),
                OnPointer::<Click>::run_callback(gamemanager::on_unit_clicked),
                SceneChild { parent_entity },
            ));

            let material_handle = material_query.get_mut(entity);
            // Every scene, which in our case corresponds to one unit entity, has one material
            // handle, therefore we clone it before changing color
            if let Ok(material_handle) = material_handle {
                let material_handle = material_handle.into_inner();
                let material = materials.get_mut(material_handle).unwrap();
                let mut material_cloned = material.clone();
                material_cloned.base_color = color;
                let material_cloned_handle = materials.add(material_cloned);
                *material_handle = material_cloned_handle;
            }
        }
    }
}

pub(crate) fn spawn_missing_unit_entities(
    mut commands: Commands,
    mut game: ResMut<Game>,
    asset_server: Res<AssetServer>,
) {
    let game = &mut *game;
    let asset_server = &*asset_server;
    for unit in game
        .units
        .all_units_iter_mut()
        .filter(|unit| unit.entity.is_none())
    {
        spawn_unit_entity(
            &mut commands,
            unit,
            &mut game.entities_to_move,
            asset_server,
        )
    }
}

pub(crate) fn kill_unit(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).despawn_recursive();
}

pub(crate) fn move_unit_entities(
    mut query: Query<(Option<&MainCube>, &mut Transform)>,
    mut game: ResMut<Game>,
) {
    let mut success = Vec::with_capacity(game.entities_to_move.len());
    for unit_to_move in &game.entities_to_move {
        let plane = game.board.get_cell(unit_to_move.1).unwrap().plane;
        let target_translation = query.get(plane).unwrap().1.translation;
        let scale = 3. / game.board.cube_side_length as f32;
        let rotation =
            Quat::from_rotation_arc(Vec3::Y, unit_to_move.1.normal_direction().as_vec3());

        let Ok(transform_entity) = query.get_mut(unit_to_move.0) else {
            success.push(false);
            return;
        };
        let mut transform_entity = transform_entity.1;
        transform_entity.translation = target_translation;
        transform_entity.scale = Vec3::new(scale, scale / 2., scale);
        transform_entity.rotation = rotation;
        success.push(true);
    }
    let mut index = 0;
    game.entities_to_move.retain(|_| {
        let out = !success[index];
        index += 1;
        out
    });
}
