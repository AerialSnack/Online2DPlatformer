use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_ggrs::*;
use bevy_matchbox::prelude::*;

pub const INPUT_UP: u8 = 1 << 0;
pub const INPUT_LEFT: u8 = 1 << 1;
pub const INPUT_RIGHT: u8 = 1 << 2;
pub const INPUT_STRIKE: u8 = 1 << 3;
pub const INPUT_UP_PRESSED: u8 = 1 << 4;  // New flag for just pressed up

pub type Config = bevy_ggrs::GgrsConfig<u8, PeerId>;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(ReadInputs, read_local_inputs);
    }
}

fn read_local_inputs(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    local_players: Res<LocalPlayers>,
) {
    let mut local_inputs = HashMap::new();

    for handle in &local_players.0 {
        let mut input = 0u8;

        if keys.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
            input |= INPUT_UP;
        }
        if keys.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
            input |= INPUT_UP_PRESSED;
        }
        if keys.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
            input |= INPUT_LEFT
        }
        if keys.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
            input |= INPUT_RIGHT;
        }
        if keys.any_pressed([KeyCode::Space, KeyCode::Enter]) {
            input |= INPUT_STRIKE;
        }

        local_inputs.insert(*handle, input);
    }

    commands.insert_resource(LocalInputs::<Config>(local_inputs));
}

// Helper function to get direction from input
pub fn get_input_direction(input: u8) -> Vec2 {
    let mut direction = Vec2::ZERO;

    // Only handle horizontal movement here
    if input & INPUT_RIGHT != 0 {
        direction.x += 1.;
    }
    if input & INPUT_LEFT != 0 {
        direction.x -= 1.;
    }

    direction
} 