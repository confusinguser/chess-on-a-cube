use bevy::math::vec4;
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use std::f32::consts::PI;

use bevy::prelude::Vec3;

use crate::gamemanager::{self, CellCoordinates, Game};
use crate::materials;

pub(crate) fn construct_cube(
    side_length: u32,
    meshes: &mut ResMut<Assets<Mesh>>,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    material: &StandardMaterial,
    game: &mut ResMut<Game>,
) {
    let plane_mesh: Handle<Mesh> = meshes.add(shape::Plane::default().into());
    let spacing = 1. / (side_length) as f32;
    let offset = 0.5 - spacing / 2.;
    // The total side length of cube is always 1, so we offset
    // by 0.5 to get middle in origo. When cube at origo, half of its side is in negative
    // quadrant, so therefore we subtract the part that is already offset from this phenomenon.
    for side in 0..6 {
        //        lookup_planes.planes[side] = vec![None; side_length.pow(2) as usize];
        for i in 0..side_length.pow(2) {
            let translation;
            let mut rotation;
            let coords;
            match side {
                0 | 1 => {
                    translation = Vec3::new(
                        (i % side_length) as f32 * spacing - offset,
                        if side % 2 == 0 { 0.5 } else { -0.5 },
                        (i / side_length % side_length) as f32 * spacing - offset,
                    );
                    rotation = Vec3::new(0., 0., 2.); // Up/down rotate 180 degrees, which is 2 turns
                    coords = CellCoordinates::new(
                        i % side_length,
                        0,
                        i / side_length % side_length,
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
                    coords = CellCoordinates::new(
                        i % side_length,
                        i / side_length % side_length,
                        0,
                        side % 2 == 0,
                    )
                }
                4 | 5 => {
                    translation = Vec3::new(
                        if side % 2 == 0 { 0.5 } else { -0.5 },
                        (i / side_length % side_length) as f32 * spacing - offset,
                        (i % side_length) as f32 * spacing - offset,
                    );
                    rotation = Vec3::new(0., 0., 1.);
                    coords = CellCoordinates::new(
                        0,
                        i / side_length % side_length,
                        i % side_length,
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

            game.board.new_cell(coords);
            game.board.get_cell_mut(coords).unwrap().set_plane(plane);
            //lookup_planes.planes[side][i as usize] = Some(plane);
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
        let plane = cell.plane.unwrap();

        let query_result = query.get(plane).unwrap();
        let material = materials.get_mut(query_result.0).unwrap();
        if game.selected_cell.map_or(false, |x| x == cell.coords) {
            materials::select_cell_material(material);
        } else if cell.selected_unit_can_go {
            materials::can_go_cell_material(material);
        } else {
            materials::normal_cell_material(material);
        }
    }
}

pub(crate) fn spawn_unit(
    commands: &mut Commands,
    translation: Vec3,
    asset_server: Res<AssetServer>,
) -> Entity {
    commands
        .spawn(SceneBundle {
            transform: Transform::from_translation(translation),
            scene: asset_server.load("models/AlienCake/alien.glb#Scene0"),
            ..default()
        })
        .id()
}
