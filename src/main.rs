use bevy::prelude::*;
use bevy::render::settings::{Backends, WgpuSettings};
mod main_menu;
mod game;
mod input;

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    InGame,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(bevy::render::RenderPlugin {
            render_creation: WgpuSettings {
                backends: Some(Backends::VULKAN),
                ..default()
            }.into(),
            ..default()
        }))
        .init_state::<GameState>()
        .add_plugins(main_menu::MainMenuPlugin)
        .add_plugins(game::GamePlugin)
        .run();
}