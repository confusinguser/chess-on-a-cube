use bevy::prelude::*;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

pub(crate) fn button_system(
    mut interaction_query: Query<(&Interaction, &MenuButton), (Changed<Interaction>, With<Button>)>,
    mut ui_query: Query<(&mut Style, &mut UI)>,
) {
    for (interaction, menu_button) in &mut interaction_query {
        dbg!(interaction);
        match *interaction {
            Interaction::Clicked => {
                dbg!(menu_button.button_type);
                match menu_button.button_type {
                    ButtonType::Continue => {
                        hide_all_uis(&mut ui_query);
                    }
                    ButtonType::Settings => {
                        show_ui(&mut ui_query, UiType::Settings);
                    }
                }
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub(crate) enum UiType {
    Main,
    Settings,
}

#[derive(Component)]
pub(crate) struct UI {
    ui_type: UiType,
    currently_shown: bool,
}

impl UI {
    fn new(ui_type: UiType) -> Self {
        UI {
            ui_type,
            currently_shown: false,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
enum ButtonType {
    Continue,
    Settings,
}

#[derive(Component)]
pub(crate) struct MenuButton {
    button_type: ButtonType,
}

impl MenuButton {
    fn new(button_type: ButtonType) -> Self {
        MenuButton { button_type }
    }
}

pub(crate) fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let button_style = Style {
        size: Size {
            width: Val::Px(150.0),
            height: Val::Px(65.0),
        },
        margin: UiRect {
            left: Val::Px(10.0),
            right: Val::Px(10.0),
            top: Val::Px(10.0),
            bottom: Val::Px(10.0),
        },
        // horizontally center child text
        justify_content: JustifyContent::Center,
        // vertically center child text
        align_items: AlignItems::Center,
        ..default()
    };

    // Set up main UI
    let continue_button = ButtonBundle {
        style: button_style.clone(),
        image: asset_server.load("textures/continue_button.png").into(),
        ..default()
    };

    let settings_button = ButtonBundle {
        style: button_style.clone(),
        image: asset_server.load("textures/settings_button.png").into(),
        ..default()
    };

    let main_ui = NodeBundle {
        style: Style {
            size: Size {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
            },
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            display: Display::None,
            ..default()
        },
        ..default()
    };

    // Set up settings UI
    let settings_ui = ImageBundle {
        style: Style {
            size: Size {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
            },
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            display: Display::None,
            ..default()
        },
        image: asset_server.load("textures/background.png").into(),
        ..default()
    };

    commands
        .spawn((main_ui, UI::new(UiType::Main)))
        .with_children(|parent| {
            parent.spawn((continue_button, MenuButton::new(ButtonType::Continue)));
            parent.spawn((settings_button, MenuButton::new(ButtonType::Settings)));
        });
    commands
        .spawn((settings_ui, UI::new(UiType::Settings)))
        .with_children(|parent| {
            let back_button = ButtonBundle {
                style: button_style,
                image: asset_server.load("textures/settings_button.png").into(),
                ..default()
            };
            parent.spawn((back_button, MenuButton::new(ButtonType::Continue)));
        });
}

fn hide_all_uis(query: &mut Query<(&mut Style, &mut UI)>) {
    for (mut style, mut ui) in query {
        style.display = Display::None;
        ui.currently_shown = false;
    }
}

fn show_ui(query: &mut Query<(&mut Style, &mut UI)>, ui_type: UiType) {
    for (mut style, mut ui) in query {
        if ui.ui_type == ui_type {
            style.display = Display::Flex;
            ui.currently_shown = true;
        } else {
            style.display = Display::None;
            ui.currently_shown = false;
        }
    }
}

fn toggle_ui(mut query: Query<(&mut Style, &mut UI)>) {
    let some_ui_showing = query.iter().any(|(_, ui)| ui.currently_shown);
    for (mut style, mut ui) in &mut query {
        if some_ui_showing {
            style.display = Display::None;
            ui.currently_shown = false;
        } else if ui.ui_type == UiType::Main {
            style.display = Display::Flex;
            ui.currently_shown = true;
        }
    }
}

pub(crate) fn ui_system(
    ui_query: Query<(&mut Style, &mut UI)>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        toggle_ui(ui_query);
    }
}
