mod cube_rotation;
mod gamemanager;
mod materials;
mod scene;
mod utils;

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(
            DefaultPickingPlugins
                .build()
                .disable::<DefaultHighlightingPlugin>(),
        )
        .insert_resource(gamemanager::Game::new(3))
        .add_startup_system(setup)
        .add_system(cube_rotation::rotate)
        .add_system(scene::update_cell_colors)
        .run();
}

#[derive(Component)]
struct MainCamera {
    start_coords: Vec3,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
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
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(8.0, 16.0, 8.0),
            ..default()
        },
        MainCamera {
            start_coords: Vec3::new(8., 16., 8.),
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
