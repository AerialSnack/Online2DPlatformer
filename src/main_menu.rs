use bevy::prelude::*;
use crate::GameState;

pub struct MainMenuPlugin;

#[derive(Component)]
struct MainMenu;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_main_menu)
           .add_systems(Update, button_system)
           .add_systems(OnEnter(GameState::InGame), cleanup_main_menu);
    }
}

#[derive(Component)]
enum MenuButtonAction {
    StartGame,
    Quit,
}

fn cleanup_main_menu(
    mut commands: Commands,
    query: Query<Entity, With<MainMenu>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, MainMenu));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::NONE),
            MainMenu,
        ))
        .with_children(|parent| {
            // Start Game button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(65.0),
                        margin: UiRect::all(Val::Px(20.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    MenuButtonAction::StartGame,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Start Game"),
                        TextFont {
                            //font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });

            // Quit button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(65.0),
                        margin: UiRect::all(Val::Px(20.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    MenuButtonAction::Quit,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Quit"),
                        TextFont {
                            //font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
        });
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<bevy::app::AppExit>,
) {
    for (interaction, menu_button_action) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::StartGame => {
                    next_state.set(GameState::InGame);
                }
                MenuButtonAction::Quit => {
                    exit.send(bevy::app::AppExit::default());
                }
            }
        }
    }
}