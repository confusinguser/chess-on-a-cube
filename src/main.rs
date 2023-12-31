mod ai;
mod cell;
mod cube_rotation;
mod gamemanager;
mod materials;
mod movement;
mod scene;
mod units;
mod utils;

use bevy::log::*;
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(LogPlugin {
                    level: Level::WARN,
                    ..default()
                }),
        )
        .add_plugins(
            DefaultPickingPlugins
                .build()
                .disable::<DefaultHighlightingPlugin>(),
        )
        .insert_resource(gamemanager::Game::new(4))
        .add_startup_system(setup)
        .add_system(cube_rotation::rotate)
        .add_system(scene::update_cell_colors)
        .add_system(scene::prepare_unit_entity.run_if(any_with_component::<scene::PrepareUnit>()))
        .add_system(scene::move_unit_entities)
        .add_system(scene::spawn_missing_unit_entities)
        .add_system(gamemanager::ai_play)
        .run();
}

#[derive(Component)]
struct MainCamera {
    start_coords: Vec3,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut game: ResMut<gamemanager::Game>,
) {
    let material = StandardMaterial {
        base_color: Color::ANTIQUE_WHITE,
        ..default()
    };

    scene::construct_cube(
        game.board.cube_side_length,
        &mut meshes,
        &mut commands,
        &mut materials,
        &material,
        &mut game,
    );

    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                intensity: 9000.0,
                range: 100.,
                shadows_enabled: false,
                ..default()
            },
            transform: Transform::from_xyz(8., 8., 8.),
            ..default()
        },
        MainCamera {
            start_coords: Vec3::new(8., 8., 8.),
        },
    ));

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(2., 2., 2.).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
            ..default()
        },
        RaycastPickCamera::default(), // Enable picking with this camera
        MainCamera {
            start_coords: Vec3::new(2., 2., 2.),
        },
    ));
}
