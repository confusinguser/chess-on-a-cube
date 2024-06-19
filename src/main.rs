use bevy::log::*;
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

mod ai;
mod cell;
mod cube_rotation;
mod gamemanager;
mod materials;
mod movement;
mod scene;
mod units;
mod utils;

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
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                cube_rotation::iterate,
                scene::update_cell_colors,
                scene::move_unit_entities,
                scene::spawn_missing_unit_entities,
                gamemanager::ai_play,
            ),
        )
        .add_systems(
            Update,
            scene::prepare_unit_entity.run_if(any_with_component::<scene::PrepareUnit>),
        )
        .run();
}

#[derive(Component)]
struct MainCamera {}

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
                intensity: 5000.0,
                range: 100.,
                shadows_enabled: false,
                ..default()
            },
            transform: Transform::from_xyz(8., 8., 8.),
            ..default()
        },
        MainCamera {},
    ));

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(2., 2., 2.).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
            ..default()
        },
        MainCamera {},
    ));
}
